use std::{fs::OpenOptions, io::Write};

use anyhow::Result;
use semver::Version;
use tokio::fs;

use crate::model::{ServerType, StartupMethod};

use super::BuildContext;

impl BuildContext<'_> {
    pub async fn get_startup_method(&self, serverjar_name: &str) -> Result<StartupMethod> {
        let mcver = self.app.mc_version();
        Ok(match &self.app.server.jar {
            ServerType::NeoForge { loader } => {
                use crate::sources::neoforge::NEOFORGE_BREAKOFF_VERSION;
                let loader_ver = self.app.neoforge().resolve_version(loader).await?;

                if Version::parse(mcver)? <= NEOFORGE_BREAKOFF_VERSION {
                    StartupMethod::Custom {
                        windows: vec![format!(
                            "@libraries/net/neoforged/forge/{mcver}-{loader_ver}/win_args.txt"
                        )],
                        linux: vec![format!(
                            "@libraries/net/neoforged/forge/{mcver}-{loader_ver}/unix_args.txt"
                        )],
                    }
                } else {
                    StartupMethod::Custom {
                        windows: vec![format!(
                            "@libraries/net/neoforged/neoforge/{loader_ver}/win_args.txt"
                        )],
                        linux: vec![format!(
                            "@libraries/net/neoforged/neoforge/{loader_ver}/unix_args.txt"
                        )],
                    }
                }
            }
            ServerType::Forge { loader } => {
                let l = self.app.forge().resolve_version(loader).await?;

                StartupMethod::Custom {
                    windows: vec![format!(
                        "@libraries/net/minecraftforge/forge/{mcver}-{l}/win_args.txt"
                    )],
                    linux: vec![format!(
                        "@libraries/net/minecraftforge/forge/{mcver}-{l}/unix_args.txt"
                    )],
                }
            }
            _ => StartupMethod::Jar(serverjar_name.to_owned()),
        })
    }

    pub async fn create_scripts(&self, startup: StartupMethod) -> Result<()> {
        fs::write(
            self.output_dir.join("start.bat"),
            self.app
                .server
                .launcher
                .generate_script_win(&self.app.server.name, &startup),
        )
        .await?;

        let mut file;
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::prelude::OpenOptionsExt;
            file = OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o755)
                .open(self.output_dir.join("start.sh"))?;
        }
        #[cfg(not(target_family = "unix"))]
        {
            file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(self.output_dir.join("start.sh"))?;
        }

        file.write_all(
            self.app
                .server
                .launcher
                .generate_script_linux(&self.app.server.name, &startup)
                .as_bytes(),
        )?;

        Ok(())
    }
}
