use std::{collections::HashMap, env, process::Stdio, time::Duration};

use anyhow::{bail, Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{io::AsyncBufReadExt, process::Command};

use crate::{
    app::App,
    model::{HookEvent, HookFailBehavior},
};

pub struct HooksAPI<'a>(pub &'a App);

impl HooksAPI<'_> {
    pub fn resolve_filename(&self, entry: &str) -> String {
        let hook = self
            .0
            .server
            .hooks
            .get(entry)
            .or(self.0.network.as_ref().and_then(|nw| nw.hooks.get(entry)))
            .unwrap();

        match env::consts::FAMILY {
            "windows" => hook.windows.clone(),
            "unix" => hook.linux.clone(),
            _ => None,
        }
        .unwrap_or(String::from(entry))
    }

    pub async fn event(&self, event: HookEvent, data: HashMap<String, String>) -> Result<()> {
        for (name, hook) in self.0.server.hooks.iter().chain(
            self.0
                .network
                .as_ref()
                .map_or(HashMap::default(), |nw| nw.hooks.clone())
                .iter(),
        ) {
            if !hook.disabled && hook.when == event {
                let filename = self.resolve_filename(name);

                let path = self.0.server.path.join("hooks").join(&filename);

                if !path.exists() {
                    self.0.warn(format!("Hook '{filename}' was not found"));
                    continue;
                }

                let spinner = self
                    .0
                    .multi_progress
                    .add(
                        ProgressBar::new_spinner().with_style(ProgressStyle::with_template(
                            "    {spinner:.green} {prefix:.bold} {msg}",
                        )?),
                    );

                spinner.enable_steady_tick(Duration::from_millis(200));
                spinner.set_prefix(format!("Running hook {}", style(filename.clone()).blue()));

                if hook.show_output {
                    self.0.log_dev(format!("Running {filename}"));
                }

                let mut cmd = Command::new(path);
                cmd.kill_on_drop(true)
                    .current_dir(&self.0.server.path)
                    .stdout(Stdio::piped());

                for (k, v) in &data {
                    cmd.env(k, v);
                }

                let mut child = cmd.spawn().context(format!("Spawning hook {filename}"))?;

                let stdout = child.stdout.take().unwrap();
                let mut lines = tokio::io::BufReader::new(stdout).lines();

                while let Some(line) = lines.next_line().await? {
                    spinner.set_message(line.clone());
                    if hook.show_output {
                        self.0.multi_progress.suspend(|| {
                            println!("{}{}", style("| ").bold(), line.trim());
                        });
                    }
                }

                let status = child
                    .wait()
                    .await
                    .context(format!("waiting hook {filename}"))?;
                spinner.finish_and_clear();
                if status.success() {
                    self.0.success(format!("Hook {filename}"));
                } else {
                    match hook.onfail {
                        HookFailBehavior::Ignore => {}
                        HookFailBehavior::Warn => self.0.warn(format!("Hook {filename} failed")),
                        HookFailBehavior::Error => bail!("Hook {filename} failed"),
                    }
                }
            }
        }

        Ok(())
    }
}
