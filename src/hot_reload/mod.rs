use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    path::{Component, PathBuf},
    process,
    process::{ExitStatus, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use console::style;
use dialoguer::theme::ColorfulTheme;
use indicatif::ProgressBar;
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    DebounceEventResult, Debouncer, FileIdMap,
};
use pathdiff::diff_paths;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout},
    sync::mpsc,
};

use crate::core::BuildContext;

use self::config::{HotReloadAction, HotReloadConfig};

pub mod config;
pub mod pattern_serde;

#[derive(Debug)]
pub struct DevSession<'a> {
    pub builder: BuildContext<'a>,
    pub jar_name: Option<String>,
    // None to disable hot reloading
    pub hot_reload: Option<Arc<Mutex<HotReloadConfig>>>,
    // true if in test mode (exit server after server fully starts, report/upload logs on fail)
    pub test_mode: bool,
}

#[derive(Debug)]
pub enum State {
    Building,
    Starting,
    Online,
    Stopping,
    Stopped,
}

#[allow(clippy::enum_variant_names)]
pub enum Command {
    Start,
    EndSession,
    Rebuild,
    SendCommand(String),
    Log(String),
    WaitUntilExit,
    Bootstrap(PathBuf, PathBuf),
    BootstrapGroup(String, PathBuf, PathBuf),
}

async fn try_read_line(
    opt: &mut Option<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>,
) -> Result<Option<String>> {
    Ok(match opt {
        Some(lines) => lines.next_line().await?,
        None => None,
    })
}

async fn try_wait_child(opt: &mut Option<Child>) -> Result<Option<ExitStatus>> {
    Ok(match opt {
        Some(c) => Some(c.wait().await?),
        None => None,
    })
}

#[derive(Debug, PartialEq, Clone)]
pub enum TestResult {
    Success,
    Failed,
    Crashed,
}

// TODO
// [x] fix stdout nesting for some reason
// [x] commands are not being sent properly
// [x] use debouncer for notify
// [ ] reload server.toml properly
// [x] tests

pub const LINE_CRASHED: &str = "]: Crashed! The full crash report has been saved to";

impl DevSession<'_> {
    pub async fn spawn_child(&mut self) -> Result<Child> {
        let platform = if env::consts::FAMILY == "windows" {
            "windows"
        } else {
            "linux"
        };

        let server_jar = self.jar_name.as_ref().unwrap().clone();
        let startup = self.builder.get_startup_method(&server_jar).await?;
        let launcher = &self.builder.app.server.launcher;
        let java = launcher.get_java();
        let args = launcher.get_arguments(&startup, platform);

        self.builder
            .app
            .dbg(format!("Running: {java} {}", args.join(" ")));

        let cwd = env::current_dir()?.canonicalize()?;
        // because jre is stupid
        let dir = diff_paths(&self.builder.output_dir, cwd).unwrap();

        //self.builder.app.info(&format!("Output: {}", self.builder.output_dir.display()))?;
        //self.builder.app.info(&format!("Cwd: {}", cwd.display()))?;
        //self.builder.app.info(&format!("Working directory: {}", dir.display()))?;

        Ok(tokio::process::Command::new(java)
            .args(args)
            .current_dir(&dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?)
    }

    #[allow(unused_assignments)]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::await_holding_lock)]
    async fn handle_commands(
        mut self,
        mut rx: mpsc::Receiver<Command>,
        tx: mpsc::Sender<Command>,
    ) -> Result<()> {
        let mp = self.builder.app.multi_progress.clone();

        let mut child: Option<Child> = None;
        let mut stdout_lines: Option<Lines<BufReader<ChildStdout>>> = None;
        let mut child_stdin: Option<ChildStdin> = None;

        let mut is_stopping = false;
        let mut is_session_ending = false;
        let mut test_result = TestResult::Failed;
        let mut exit_status = None;

        let state = Arc::new(Mutex::new(State::Stopped));

        let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();

        'l: loop {
            tokio::select! {
                Some(cmd) = rx.recv() => {
                    match cmd {
                        Command::Start => {
                            self.builder.app.ci("::group::Starting server process");
                            self.builder.app.log_dev("Starting server process...");
                            if child.is_none() {
                                let mut spawned_child = self.spawn_child().await?;
                                stdout_lines = Some(tokio::io::BufReader::new(spawned_child.stdout.take().expect("child stdout None")).lines());
                                child_stdin = Some(spawned_child.stdin.take().expect("child stdin None"));
                                child = Some(spawned_child);
                                let mut lock = state.lock().unwrap();
                                *lock = State::Starting;
                            }
                        }
                        Command::SendCommand(command) => {
                            self.builder.app.log_dev(format!("$ {}", command.trim()));
                            if let Some(ref mut stdin) = &mut child_stdin {
                                stdin.write_all(command.as_bytes()).await?;
                            }
                        }
                        Command::Log(message) => {
                            self.builder.app.log_dev(message);
                        }
                        Command::WaitUntilExit => {
                            self.builder.app.log_dev("Waiting for process exit...");
                            is_stopping = true;
                            let mut lock = state.lock().unwrap();
                            *lock = State::Stopping;
                            drop(lock);
                            if let Some(ref mut child) = &mut child {
                                let should_kill = tokio::select! {
                                    () = async {
                                        // future to keep printing logs
                                        loop {
                                            if let Ok(Some(line)) = try_read_line(&mut stdout_lines).await {
                                                mp.suspend(|| {
                                                    println!(
                                                        "{}{}",
                                                        style("| ").bold(),
                                                        line.trim()
                                                    );
                                                });
                                            }
                                        }
                                    // should be unreachable since infinite loop
                                    // but still, return false, idk
                                    } => false,
                                    status = child.wait() => {
                                        exit_status = status.ok();
                                        false
                                    },
                                    () = tokio::time::sleep(Duration::from_secs(30)) => {
                                        self.builder.app.info("Timeout reached, killing...");
                                        true
                                    },
                                    _ = tokio::signal::ctrl_c() => {
                                        self.builder.app.info("^C recieved, killing...");
                                        true
                                    },
                                };

                                if should_kill {
                                    child.kill().await?;
                                    exit_status = None;
                                }
                            }
                            is_stopping = false;
                            child = None;
                            stdout_lines = None;
                            child_stdin = None;
                            self.builder.app.log_dev("Server process ended");
                        }
                        Command::Rebuild => {
                            self.builder.app.log_dev("Building...");
                            let mut lock = state.lock().unwrap();
                            *lock = State::Building;
                            drop(lock);

                            match self.builder.app.reload_server() {
                                Ok(()) => {},
                                Err(e) => {
                                    self.builder.app.error("Error while reloading server.toml");
                                    self.builder.app.error(e);
                                    return Ok(());
                                }
                            }

                            match self.builder.app.reload_network() {
                                Ok(()) => {},
                                Err(e) => {
                                    self.builder.app.error("Error while reloading network.toml");
                                    self.builder.app.error(e);
                                    return Ok(());
                                }
                            }

                            match self.builder.build_all().await {
                                Ok(jar_name) => {
                                    self.jar_name = Some(jar_name);
                                    tx.send(Command::Start).await?;
                                },
                                Err(e) => {
                                    self.builder.app.error("Error while building");
                                    self.builder.app.error(e);
                                    return Ok(());
                                }
                            };
                        }
                        Command::BootstrapGroup(group_name, full_path, rel_path) => {
                            let should = group_name == "global" || self.builder.app.network.as_ref().is_some_and(|nw| {
                                (self.builder.app.server.name == nw.proxy && nw.proxy_groups.contains(&group_name)) || nw.servers.get(&self.builder.app.server.name)
                                    .is_some_and(|serv| serv.groups.contains(&group_name))
                            });

                            if should {
                                tx.send(Command::Bootstrap(full_path, rel_path)).await?;
                            }
                        }
                        Command::Bootstrap(full_path, rel_path) => {
                            //self.builder.app.log_dev(format!("Bootstrapping: {}", rel_path.to_string_lossy().trim()));
                            if let Err(e) = self.builder.bootstrap_file(&full_path, &rel_path, None).await {
                                self.builder.app.warn(format!("Error while bootstrapping:
                                - Path: {}
                                - Relative: {}
                                - Err: {e}", full_path.display(), rel_path.display()));
                            }
                        }
                        Command::EndSession => {
                            self.builder.app.log_dev("Ending session...");
                            self.builder.app.ci("::endgroup::");
                            break 'l;
                        }
                    }
                },
                Ok(Some(line)) = try_read_line(&mut stdout_lines) => {
                    let s = line.trim();

                    if self.test_mode
                        && !is_stopping
                        && test_result == TestResult::Failed {
                        if s.contains(&self.builder.app.server.options.success_line) /* && s.ends_with("For help, type \"help\"") */ {
                            test_result = TestResult::Success;

                            let mut lock = state.lock().unwrap();
                            *lock = State::Online;
                            drop(lock);

                            self.builder.app.success("Test passed!");

                            tx.send(Command::SendCommand(format!(
                                "{}\n",
                                &self.builder.app.server.options.stop_command
                            ))).await?;
                            tx.send(Command::WaitUntilExit).await?;
                            tx.send(Command::EndSession).await?;
                        } else if s.contains(LINE_CRASHED) || s == "---- end of report ----" {
                            self.builder.app.warn("Server crashed!");
                            test_result = TestResult::Crashed;

                            let mut lock = state.lock().unwrap();
                            *lock = State::Stopping;
                            drop(lock);

                            tx.send(Command::WaitUntilExit).await?;
                            tx.send(Command::EndSession).await?;
                        }
                    }

                    mp.suspend(|| {
                        println!(
                            "{}{s}",
                            style("| ").bold()
                        );
                    });
                },
                Ok(Some(line)) = stdin_lines.next_line() => {
                    let cmd = line.trim();

                    self.builder.app.log_dev(format!("$ {cmd}"));

                    if let Some(ref mut stdin) = &mut child_stdin {
                        stdin.write_all(format!("{cmd}\n").as_bytes()).await?;
                    } else {
                        self.builder.app.log_dev("Server offline");
                    }
                },
                Ok(Some(status)) = try_wait_child(&mut child) => {
                    exit_status = Some(status);
                    self.builder.app.ci("::endgroup::");
                    self.builder.app.log_dev("Server process exited");

                    is_stopping = false;
                    child = None;
                    stdout_lines = None;
                    child_stdin = None;

                    if self.test_mode {
                        tx.send(Command::EndSession).await?;
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    if is_session_ending {
                        self.builder.app.log_dev("Force-stopping development session...");
                        break 'l;
                    } else if !is_stopping {
                        is_session_ending = true;
                        self.builder.app.log_dev("Stopping development session...");

                        tx.send(Command::SendCommand("stop\nend\n".to_owned())).await?;
                        tx.send(Command::WaitUntilExit).await?;
                        tx.send(Command::EndSession).await?;
                    }
                }
            }
        }

        // end of loop > tokio::select!

        if let Some(ref mut child) = &mut child {
            self.builder.app.info("Killing undead child process...");
            child.kill().await?;
        }

        self.builder.app.ci("::endgroup::");

        if self.test_mode {
            match test_result {
                TestResult::Success => {
                    self.builder.app.success("Test passed");
                    process::exit(0);
                }
                TestResult::Crashed | TestResult::Failed => {
                    mp.suspend(|| {
                        println!("{} Test failed!", ColorfulTheme::default().error_prefix);

                        if let Some(status) = &exit_status {
                            if let Some(code) = status.code() {
                                println!(
                                    "  - Process exited with code {}",
                                    if code == 0 {
                                        style(code).green()
                                    } else {
                                        style(code).red().bold()
                                    }
                                );
                            } else if !status.success() {
                                println!("  - Process didn't exit successfully");
                            }
                        }

                        if let TestResult::Crashed = test_result {
                            println!("  - Server crashed");
                        }
                    });

                    if self.builder.app.config.services.mclogs.enabled {
                        let pb =
                            mp.add(ProgressBar::new_spinner().with_message("Uploading to mclo.gs"));

                        pb.enable_steady_tick(Duration::from_millis(250));

                        let latest_log_path =
                            self.builder.output_dir.join("logs").join("latest.log");

                        if latest_log_path.exists() {
                            let content = std::fs::read_to_string(&latest_log_path)
                                .context("Reading latest.log file")?;

                            let is_crash = if test_result == TestResult::Crashed {
                                true
                            } else {
                                content.contains(LINE_CRASHED)
                            };

                            let crash_log_path = if is_crash {
                                let folder = self.builder.output_dir.join("crash-reports");
                                if !folder.exists() {
                                    bail!("crash-reports folder doesn't exist, cant upload to mclo.gs");
                                }

                                // get latest crash report
                                let (report_path, _) = folder
                                    .read_dir()?
                                    .filter_map(Result::ok)
                                    .filter_map(|f| {
                                        Some((f.path(), f.metadata().ok()?.modified().ok()?))
                                    })
                                    .max_by_key(|(_, t)| *t)
                                    .ok_or(anyhow!("can't find crash report"))?;

                                Some(report_path)
                            } else {
                                None
                            };

                            let log = self.builder.app.mclogs().paste_log(&content).await?;
                            drop(content);

                            let crash_log = if let Some(log_path) = crash_log_path {
                                let content = std::fs::read_to_string(&log_path).context(
                                    format!("Reading crash log file: {}", log_path.display()),
                                )?;

                                Some(self.builder.app.mclogs().paste_log(&content).await?)
                            } else {
                                None
                            };

                            pb.finish_and_clear();
                            self.builder.app.log("  - Logs uploaded to mclo.gs");
                            mp.suspend(|| {
                                println!("\n latest.log [ {} ]", log.url);
                                if let Some(log) = crash_log {
                                    println!(" crash report [ {} ]", log.url);
                                }
                                println!();
                            });
                        } else {
                            pb.finish_and_clear();
                            mp.suspend(|| {
                                println!(
                                    "{} '{}' does not exist! Can't upload log.",
                                    ColorfulTheme::default().error_prefix,
                                    style(latest_log_path.to_string_lossy()).dim()
                                );
                            });
                        }
                    }

                    process::exit(1);
                }
            }
        }

        Ok(())
    }

    pub fn create_hotreload_watcher(
        config: Arc<Mutex<HotReloadConfig>>,
        tx: mpsc::Sender<Command>,
    ) -> Result<Debouncer<RecommendedWatcher, FileIdMap>> {
        Ok(new_debouncer(
            Duration::from_secs(1),
            None,
            move |e: DebounceEventResult| {
                if let Ok(_e) = e {
                    let mut guard = config.lock().unwrap();

                    match HotReloadConfig::load_from(&guard.path) {
                        Ok(updated) => {
                            tx.blocking_send(Command::Log(String::from("Reloaded hotreload.toml")))
                                .unwrap();
                            *guard = updated;
                        }
                        Err(e) => {
                            tx.blocking_send(Command::Log(format!(
                                "Error reloading hotreload.toml: {e}"
                            )))
                            .unwrap();
                        }
                    }
                }
            },
        )?)
    }

    pub fn create_config_watcher(
        config: Arc<Mutex<HotReloadConfig>>,
        tx: mpsc::Sender<Command>,
        base: PathBuf,
    ) -> Result<Debouncer<RecommendedWatcher, FileIdMap>> {
        Ok(new_debouncer(
            Duration::from_secs(1),
            None,
            move |e: DebounceEventResult| {
                if let Ok(e) = e {
                    for path in e
                        .into_iter()
                        .flat_map(|e| e.paths.clone())
                        .collect::<HashSet<_>>()
                    {
                        if path.is_dir() || !path.exists() {
                            continue;
                        }

                        let rel_path = diff_paths(&path, &base).expect("Cannot diff paths");

                        tx.blocking_send(Command::Bootstrap(path.clone(), rel_path.clone()))
                            .unwrap();

                        let guard = config.lock().unwrap();
                        let Some(file) = guard
                            .files
                            .iter()
                            .find(|f| f.path.matches_path(&rel_path))
                            .cloned()
                        else {
                            continue;
                        };
                        drop(guard);

                        match &file.action {
                            HotReloadAction::Reload => {
                                tx.blocking_send(Command::SendCommand(
                                    "reload confirm\n".to_owned(),
                                ))
                                .expect("tx send err");
                            }
                            HotReloadAction::Restart => {
                                tx.blocking_send(Command::SendCommand("stop\nend\n".to_owned()))
                                    .expect("tx send err");
                                tx.blocking_send(Command::WaitUntilExit)
                                    .expect("tx send err");
                                tx.blocking_send(Command::Start).expect("tx send err");
                            }
                            HotReloadAction::RunCommand(cmd) => {
                                tx.blocking_send(Command::SendCommand(format!("{cmd}\n")))
                                    .expect("tx send err");
                            }
                        }
                    }
                }
            },
        )?)
    }

    pub fn create_network_groups_watcher(
        config: Arc<Mutex<HotReloadConfig>>,
        tx: mpsc::Sender<Command>,
        groups_path: PathBuf,
    ) -> Result<Debouncer<RecommendedWatcher, FileIdMap>> {
        Ok(new_debouncer(
            Duration::from_secs(1),
            None,
            move |e: DebounceEventResult| {
                if let Ok(e) = e {
                    for path in e
                        .into_iter()
                        .flat_map(|e| e.paths.clone())
                        .collect::<HashSet<_>>()
                    {
                        if path.is_dir() || !path.exists() {
                            continue;
                        }

                        let g_rel_path =
                            diff_paths(&path, &groups_path).expect("Cannot diff paths");

                        let mut comps = g_rel_path.components();

                        let Some(Component::Normal(group_name)) = comps.next() else {
                            continue;
                        };

                        if comps.next() != Some(Component::Normal(OsStr::new("config"))) {
                            continue;
                        }

                        let rel_path = comps.collect::<PathBuf>();

                        tx.blocking_send(Command::BootstrapGroup(
                            group_name.to_string_lossy().into_owned(),
                            path.clone(),
                            rel_path.clone(),
                        ))
                        .unwrap();

                        let guard = config.lock().unwrap();
                        let Some(file) = guard
                            .files
                            .iter()
                            .find(|f| f.path.matches_path(&rel_path))
                            .cloned()
                        else {
                            continue;
                        };
                        drop(guard);

                        match &file.action {
                            HotReloadAction::Reload => {
                                tx.blocking_send(Command::SendCommand(
                                    "reload confirm\n".to_owned(),
                                ))
                                .expect("tx send err");
                            }
                            HotReloadAction::Restart => {
                                tx.blocking_send(Command::SendCommand("stop\nend\n".to_owned()))
                                    .expect("tx send err");
                                tx.blocking_send(Command::WaitUntilExit)
                                    .expect("tx send err");
                                tx.blocking_send(Command::Start).expect("tx send err");
                            }
                            HotReloadAction::RunCommand(cmd) => {
                                tx.blocking_send(Command::SendCommand(format!("{cmd}\n")))
                                    .expect("tx send err");
                            }
                        }
                    }
                }
            },
        )?)
    }

    pub fn create_restarter_watcher(
        tx: mpsc::Sender<Command>,
    ) -> Result<Debouncer<RecommendedWatcher, FileIdMap>> {
        Ok(new_debouncer(
            Duration::from_secs(1),
            None,
            move |e: DebounceEventResult| {
                if let Ok(e) = e {
                    if !e.iter().any(|e| e.kind.is_modify()) {
                        return;
                    }
                    tx.blocking_send(Command::SendCommand("stop\nend".to_owned()))
                        .expect("tx send err");
                    tx.blocking_send(Command::WaitUntilExit)
                        .expect("tx send err");
                    tx.blocking_send(Command::Rebuild).expect("tx send err");
                }
            },
        )?)
    }

    pub async fn start(self) -> Result<()> {
        let (tx, rx) = mpsc::channel(32);

        let cfg_mutex_w = self.hot_reload.clone().unwrap_or_default();

        let mut config_watcher = Self::create_config_watcher(
            cfg_mutex_w.clone(),
            tx.clone(),
            self.builder.app.server.path.join("config"),
        )?;
        let mut hotreload_watcher =
            Self::create_hotreload_watcher(cfg_mutex_w.clone(), tx.clone())?;
        let mut servertoml_watcher = Self::create_restarter_watcher(tx.clone())?;
        let mut networktoml_watcher = Self::create_restarter_watcher(tx.clone())?;
        let mut network_groups_watcher = Self::create_network_groups_watcher(
            cfg_mutex_w.clone(),
            tx.clone(),
            self.builder
                .app
                .network
                .as_ref()
                .map(|nw| nw.path.join("groups"))
                .unwrap_or_default(),
        )?;

        if self.hot_reload.is_some() {
            self.builder.app.log_dev("Watching config/**");
            config_watcher.watcher().watch(
                self.builder.app.server.path.join("config").as_path(),
                RecursiveMode::Recursive,
            )?;
            self.builder.app.log_dev("Watching server.toml");
            servertoml_watcher.watcher().watch(
                self.builder.app.server.path.join("server.toml").as_path(),
                RecursiveMode::NonRecursive,
            )?;
            self.builder.app.log_dev("Watching hotrload.toml");
            hotreload_watcher.watcher().watch(
                self.builder
                    .app
                    .server
                    .path
                    .join("hotreload.toml")
                    .as_path(),
                RecursiveMode::NonRecursive,
            )?;

            if let Some(nw) = &self.builder.app.network {
                self.builder
                    .app
                    .log_dev("Watching [network.toml] groups/*/config/**");
                network_groups_watcher
                    .watcher()
                    .watch(&nw.path.join("groups"), RecursiveMode::Recursive)?;
                networktoml_watcher.watcher().watch(
                    nw.path.join("network.toml").as_path(),
                    RecursiveMode::NonRecursive,
                )?;
            }
        }

        tx.send(Command::Rebuild).await?;

        self.handle_commands(rx, tx.clone()).await?;

        Ok(())
    }
}
