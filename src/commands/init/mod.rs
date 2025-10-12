use anyhow::{bail, Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use indicatif::ProgressBar;
use rpackwiz::model::Pack;
use std::{
    borrow::Cow,
    env,
    ffi::OsStr,
    fs::{self, File},
    io::Write,
    path::Path,
};
use tempfile::Builder;

use crate::{
    app::BaseApp,
    interop::{mrpack::MRPackReader, packwiz::FileProvider},
    model::{Network, Server, ServerEntry, ServerType, SoftwareType},
    util::{
        env::{get_docker_version, write_dockerfile, write_dockerignore, write_git},
        SelectItem,
    },
};

#[derive(clap::Args)]
pub struct Args {
    /// The name of the server
    #[arg(long)]
    name: Option<String>,
    /// Import from a modrinth modpack
    #[arg(long, visible_alias = "mr", value_name = "src", group = "type")]
    mrpack: Option<String>,
    /// Import from a packwiz pack
    #[arg(long, visible_alias = "pw", value_name = "src", group = "type")]
    packwiz: Option<String>,
    /// Create a default network.toml
    #[arg(long, visible_alias = "nw", group = "type")]
    network: bool,
}

#[derive(Debug, Clone)]
pub enum InitType {
    Normal,
    MRPack(String),
    Packwiz(FileProvider),
    Network,
}

#[allow(clippy::too_many_lines)]
pub async fn run(base_app: BaseApp, args: Args) -> Result<()> {
    let current_dir = env::current_dir()?;
    let name = if let Some(name) = args.name {
        name.clone()
    } else {
        current_dir
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_owned()
    };

    let mut app = base_app.upgrade_with_default_server()?;

    let ty = if args.network {
        InitType::Network
    } else if let Some(src) = args.mrpack {
        InitType::MRPack(src.clone())
    } else if let Some(src) = args.packwiz {
        InitType::Packwiz(app.packwiz().get_file_provider(&src)?)
    } else {
        InitType::Normal
    };

    // toml checks and init
    if matches!(ty, InitType::Network) {
        if let Some(_nw) = Network::load()? {
            bail!("network.toml already exists");
        }

        app.network = Some(Network::default());
        app.network.as_mut().unwrap().name.clone_from(&name);
    } else {
        if let Ok(serv) = Server::load() {
            bail!(
                "server.toml already exists: server with name '{}'",
                serv.name
            );
        }

        if let Some(nw) = Network::load()? {
            app.info(format!(
                "Creating a server inside the '{}' network",
                nw.name
            ));
            app.network = Some(nw);
        }

        app.server.name.clone_from(&name);
    }

    // Name
    match &ty {
        InitType::Normal | InitType::MRPack(_) => {
            app.server.name = app.prompt_string_filled("Server name?", &app.server.name)?;
        }
        InitType::Packwiz(source) => {
            let pack = source
                .parse_toml::<Pack>("pack.toml")
                .await
                .context("Reading pack.toml - does it exist?")?;

            app.server.name = app.prompt_string_filled("Server name?", &pack.name)?;
        }
        InitType::Network => {
            app.network.as_mut().unwrap().name =
                app.prompt_string_filled("Network name?", &app.network.as_ref().unwrap().name)?;
        }
    }

    match &ty {
        InitType::Normal => {
            let serv_type = app.select(
                "Type of server?",
                &[
                    SelectItem(
                        SoftwareType::Normal,
                        Cow::Borrowed("Normal Server (vanilla, spigot, paper etc.)"),
                    ),
                    SelectItem(
                        SoftwareType::Modded,
                        Cow::Borrowed("Modded Server (forge, fabric, quilt etc.)"),
                    ),
                    SelectItem(
                        SoftwareType::Proxy,
                        Cow::Borrowed("Proxy Server (velocity, bungeecord, waterfall etc.)"),
                    ),
                ],
            )?;

            app.server.launcher.nogui = serv_type != SoftwareType::Proxy;

            app.server.jar = match serv_type {
                SoftwareType::Normal => ServerType::select_jar_interactive(),
                SoftwareType::Modded => ServerType::select_modded_jar_interactive(),
                SoftwareType::Proxy => ServerType::select_proxy_jar_interactive(),
                SoftwareType::Unknown => unreachable!(),
            }?;

            app.server.mc_version = if serv_type == SoftwareType::Proxy {
                "latest".to_owned()
            } else {
                let latest_ver = app
                    .vanilla()
                    .fetch_latest_mcver()
                    .await
                    .context("Fetching latest version")?;

                app.prompt_string_default("Server version?", &latest_ver)?
            };
        }

        InitType::Network => {
            let nw = app.network.as_mut().unwrap();

            let port = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Which port should the network be on?")
                .default(25565_u16)
                .interact_text()?;

            nw.port = port;
        }
        InitType::MRPack(src) => {
            let tmp_dir = Builder::new().prefix("mcman-mrpack-import").tempdir()?;

            let f = if Path::new(&src).exists() {
                File::open(src)?
            } else {
                let dl = app.dl_from_string(src).await?;
                let resolved = app
                    .download(
                        &dl,
                        tmp_dir.path().to_path_buf(),
                        ProgressBar::new_spinner(),
                    )
                    .await?;

                File::open(tmp_dir.path().join(resolved.filename))?
            };

            app.mrpack()
                .import_all(MRPackReader::from_reader(f)?, None)
                .await?;
        }
        InitType::Packwiz(src) => {
            app.packwiz().import_from_source(src.clone()).await?;
        }
    }

    match ty {
        InitType::Network => app
            .network
            .as_ref()
            .unwrap()
            .save()
            .context("Saving network.toml")?,
        _ => app.server.save().context("Saving server.toml")?,
    }

    match ty {
        InitType::Network => {}
        _ => {
            if let Some(ref mut nw) = app.network {
                if nw.servers.contains_key(&app.server.name) {
                    app.warn("Server with that name already exists in network.toml, please add entry manually");
                } else {
                    nw.servers.insert(
                        app.server.name.clone(),
                        ServerEntry {
                            port: nw.next_port(),
                            ..Default::default()
                        },
                    );
                    nw.save()?;
                    app.info("Added server entry to network.toml");
                }
            }
        }
    }

    //env
    if matches!(ty, InitType::Network) {
        fs::create_dir_all("./servers")?;
    } else {
        fs::create_dir_all("./config")?;

        if app.server.jar.get_software_type() != SoftwareType::Proxy {
            let mut f = File::create("./config/server.properties")?;
            f.write_all(include_bytes!("../../../res/server.properties"))?;
        }
    }

    let write_readme = if Path::new("./README.md").exists() {
        app.confirm("Overwrite README.md?")?
    } else {
        true
    };

    if write_readme {
        match ty {
            InitType::Network => app.markdown().init_network()?,
            _ => app.markdown().init_server()?,
        }

        match ty {
            InitType::MRPack(_) | InitType::Packwiz(_) => {
                if app.confirm("Render markdown now?")? {
                    app.markdown().update_files().await?;
                }
            }
            _ => {}
        }
    }

    initialize_environment()?;

    if matches!(ty, InitType::Network) {
        println!(
            " > {} {} {}\n > {}\n   {}\n   {}\n   {}",
            style("Network").green(),
            style(&app.network.unwrap().name).bold(),
            style("has been created!").green(),
            style("Initialize servers in this network using").cyan(),
            style("cd servers").bold(),
            style("mkdir myserver").bold(),
            style("mcman init").bold(),
        );
    } else {
        println!(
            " > {} {}\n > {} {}",
            style(&app.server.name).bold(),
            style("has been initialized!").green(),
            style("Build using").cyan(),
            style("mcman build").bold()
        );
    }

    Ok(())
}

pub fn initialize_environment() -> Result<()> {
    let theme = ColorfulTheme::default();

    if write_git().is_err() {
        println!(
            "{} {}{}{}",
            theme.prompt_prefix,
            style("Didn't find a git repo, use '").dim(),
            style("mcman env gitignore").bold(),
            style("' to generate .gitignore/attributes").dim(),
        );
    } else {
        println!(
            "{} {}",
            theme.success_prefix,
            style("Touched up .gitignore/.gitattributes").dim(),
        );
    }

    if get_docker_version().is_ok() {
        write_dockerfile(Path::new("."))?;
        write_dockerignore(Path::new("."))?;
        println!(
            "{} {}",
            theme.success_prefix,
            style("Docker files were written").dim(),
        );
    } else {
        println!(
            "{} {}{}{}",
            theme.prompt_prefix,
            style("Docker wasn't found, use '").dim(),
            style("mcman env docker").bold(),
            style("' to generate docker files").dim(),
        );
    }

    Ok(())
}
