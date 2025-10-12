use std::{
    env,
    io::{BufRead, BufReader},
    process::Command,
    process::Stdio,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use pathdiff::diff_paths;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::{
    model::{InstallMethod, ServerType},
    sources::quilt,
};

use super::BuildContext;

impl<'a> BuildContext<'a> {
    pub async fn get_install_method(&self) -> Result<InstallMethod> {
        let mcver = self.app.mc_version();
        Ok(match self.app.server.jar.clone() {
            ServerType::Quilt { loader, .. } => {
                let mut args = vec!["install", "server", mcver];

                if loader != "latest" {
                    args.push(&loader);
                }

                args.extend(["--install-dir=.", "--download-server"]);

                InstallMethod::Installer {
                    name: "Quilt Server Installer",
                    label: "qsi",
                    args: args.into_iter().map(ToOwned::to_owned).collect(),
                    rename_from: Some("quilt-server-launch.jar".to_owned()),
                    jar_name: format!(
                        "quilt-server-launch-{mcver}-{}.jar",
                        quilt::map_quilt_loader_version(&self.app.http_client, &loader)
                            .await
                            .context("resolving quilt loader version id (latest/latest-beta)")?
                    ),
                }
            }
            ServerType::NeoForge { loader } => InstallMethod::Installer {
                name: "NeoForged Installer",
                label: "nfi",
                args: vec!["--installServer".to_owned(), ".".to_owned()],
                rename_from: None,
                jar_name: format!(
                    "libraries/net/neoforged/forge/{mcver}-{0}/forge-{mcver}-{0}-server.jar",
                    self.app.neoforge().resolve_version(&loader).await?
                ),
            },
            ServerType::Forge { loader } => InstallMethod::Installer {
                name: "Forge Installer",
                label: "fi",
                args: vec!["--installServer".to_owned(), ".".to_owned()],
                rename_from: None,
                jar_name: format!(
                    "libraries/net/minecraftforge/forge/{mcver}-{0}/forge-{mcver}-{0}-server.jar",
                    self.app.forge().resolve_version(&loader).await?
                ),
            },
            ServerType::BuildTools { args, software } => {
                let mut buildtools_args = vec![
                    "--compile",
                    &software,
                    "--compile-if-changed",
                    "--rev",
                    mcver,
                ];

                buildtools_args.extend(args.iter().map(String::as_str));

                InstallMethod::Installer {
                    name: "BuildTools",
                    label: "bt",
                    args: buildtools_args.into_iter().map(ToOwned::to_owned).collect(),
                    rename_from: Some("server.jar".to_owned()),
                    jar_name: format!(
                        "{}-{mcver}.jar",
                        if software == "craftbukkit" {
                            "craftbukkit"
                        } else {
                            "spigot"
                        }
                    ),
                }
            }
            _ => InstallMethod::SingleJar,
        })
    }

    pub async fn download_server_jar(&'a self) -> Result<String> {
        let serverjar_name = match self.get_install_method().await? {
            InstallMethod::Installer {
                name,
                label,
                args,
                rename_from,
                jar_name,
            } => {
                let (_, resolved) = self.downloadable(&self.app.server.jar, "", None).await?;

                let installer_jar = resolved.filename;

                let jar_name = jar_name.replace("${mcver}", &self.app.server.mc_version);

                if !self.force && self.output_dir.join(&jar_name).exists() {
                    self.app.log(format!(
                        "  Skipping server jar ({})",
                        style(if rename_from.is_some() {
                            jar_name.clone()
                        } else {
                            "<in libraries>".to_owned()
                        })
                        .dim()
                    ));
                } else {
                    let pb = self.app.multi_progress.add(
                        ProgressBar::new_spinner()
                            .with_style(ProgressStyle::with_template("  {spinner:.green} {msg}")?),
                    );
                    pb.enable_steady_tick(Duration::from_millis(250));

                    pb.set_message(format!(
                        "Installing server jar... ({})",
                        style(if rename_from.is_some() {
                            jar_name.clone()
                        } else {
                            "<in libraries>".to_owned()
                        })
                        .dim()
                    ));

                    let mut cmd_args = vec!["-jar", &installer_jar];

                    cmd_args.extend(args.iter().map(String::as_str));

                    let java = env::var("JAVA_BIN").unwrap_or("java".to_owned());

                    self.execute_child((&java, &cmd_args), name, label)
                        .await
                        .context(format!("Executing command: 'java {}'", cmd_args.join(" ")))
                        .context(format!("Running installer: {name}"))?;

                    if let Some(from) = &rename_from {
                        let from_path = self.output_dir.join(from);
                        let to_path = self.output_dir.join(&jar_name);
                        if from_path.exists() {
                            pb.set_message(format!(
                                "Renaming... ({})",
                                style(format!("{from} => {jar_name}")).dim()
                            ));

                            fs::rename(from_path, &to_path)
                                .await
                                .context(format!("Renaming: {from} => {jar_name}"))?;
                        } else if to_path.exists() {
                            self.app
                                .log(format!("  Rename skipped ({from} doesn't exist)"));
                        } else {
                            bail!(
                                "Installer did not output '{from}', can't rename to '{jar_name}'"
                            );
                        }
                    }

                    self.app.log(format!(
                        "  Server jar installed successfully ({})",
                        style(if rename_from.is_some() {
                            jar_name.as_str()
                        } else {
                            "<in libraries>"
                        })
                        .dim()
                    ));

                    pb.finish_and_clear();
                }

                jar_name
            }
            InstallMethod::SingleJar => {
                self.downloadable(&self.app.server.jar, "", None)
                    .await?
                    .1
                    .filename
            }
        };

        Ok(serverjar_name)
    }

    pub async fn execute_child(&self, cmd: (&str, &[&str]), label: &str, tag: &str) -> Result<()> {
        // because jre cant understand UNC
        let dir = diff_paths(&self.output_dir, env::current_dir()?.canonicalize()?).unwrap();

        let args: Vec<String> = cmd.1.iter().map(|a| self.app.server.format(a)).collect();
        self.app.dbg(args.join(" "));

        let mut child = Command::new(cmd.0)
            .args(args)
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context(format!("Running {label}"))?;

        let spinner = self
            .app
            .multi_progress
            .add(
                ProgressBar::new_spinner().with_style(ProgressStyle::with_template(
                    "    {spinner:.green} {prefix:.bold} {msg}",
                )?),
            );

        spinner.enable_steady_tick(Duration::from_millis(200));
        spinner.set_prefix(format!("[{tag}]"));

        let mut log_file = File::create(self.output_dir.join(format!(".{tag}.mcman.log"))).await?;

        log_file
            .write_all(format!("=== mcman {tag} / {label} output ===\n\n").as_bytes())
            .await?;

        for buf in BufReader::new(child.stdout.take().unwrap()).lines() {
            let buf = buf.context("Reading child process stdout buffer")?;
            let buf = buf.trim();

            if !buf.is_empty() {
                log_file.write_all(buf.as_bytes()).await?;
                log_file.write_all(b"\n").await?;

                if let Some(last_line) = buf.split('\n').next_back() {
                    spinner.set_message(last_line.to_string());
                }
            }
        }

        if !child.wait()?.success() {
            bail!("{label} exited with non-zero code");
        }

        spinner.disable_steady_tick();
        spinner.finish_and_clear();

        Ok(())
    }
}
