mod actions;
mod caching;
mod downloading;
mod feedback;
mod from_string;
mod hashing;
mod resolvable;

use anyhow::{Context, Result};
pub use caching::*;
use confique::Config;
pub use feedback::*;
use indicatif::MultiProgress;
pub use resolvable::*;

use crate::model::{AppConfig, Downloadable, Network, Server};
use crate::sources;

use std::{
    env,
    fmt::{self, Display, Formatter},
};

pub const APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " - ",
    env!("CARGO_PKG_REPOSITORY"),
);

#[derive(Debug, Clone, Copy)]
pub enum AddonType {
    Plugin,
    Mod,
}

impl AddonType {
    pub const fn folder(self) -> &'static str {
        match self {
            Self::Mod => "mods",
            Self::Plugin => "plugins",
        }
    }
}

impl Display for AddonType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Plugin => "plugin",
            Self::Mod => "mod",
        })
    }
}

pub struct BaseApp {
    pub http_client: reqwest::Client,
}

impl BaseApp {
    pub fn new() -> Result<Self> {
        let b = reqwest::Client::builder().user_agent(APP_USER_AGENT);

        Ok(Self {
            http_client: b.build().context("Failed to create HTTP client")?,
        })
    }

    pub fn upgrade(self) -> Result<App> {
        Ok(App {
            server: Server::load().context("Failed to load server.toml")?,
            ..App::new(self)?
        })
    }

    pub fn upgrade_with_default_server(self) -> Result<App> {
        App::new(self)
    }
}

#[derive(Debug)]
pub struct App {
    pub http_client: reqwest::Client,
    pub server: Server,
    pub network: Option<Network>,

    pub multi_progress: MultiProgress,
    pub config: AppConfig,
}

impl App {
    pub fn new(base_app: BaseApp) -> Result<Self> {
        Ok(Self {
            http_client: base_app.http_client,
            server: Server::default(),
            network: Network::load()?,
            multi_progress: MultiProgress::new(),
            config: Config::builder()
                .env()
                .file(".mcman.toml")
                .file(
                    dirs::config_dir()
                        .unwrap_or_default()
                        .join("mcman/.mcman.toml"),
                )
                .load()?,
        })
    }

    pub fn mc_version(&self) -> &str {
        self.server.mc_version.as_str()
    }

    pub fn reload_server(&mut self) -> Result<()> {
        self.server = Server::load().context("Loading server.toml")?;
        Ok(())
    }

    pub fn reload_network(&mut self) -> Result<()> {
        self.network = Network::load().context("Loading network.toml")?;
        Ok(())
    }

    pub fn get_addons(&self, ty: AddonType) -> Vec<Downloadable> {
        match ty {
            AddonType::Plugin => {
                let mut list = self.server.plugins.clone();
                if let Some(nw) = &self.network {
                    if let Some(entry) = nw.servers.get(&self.server.name) {
                        for group_name in &entry.groups {
                            if let Some(group) = nw.groups.get(group_name) {
                                list.extend_from_slice(&group.plugins);
                            }
                        }
                    }

                    if self.server.name == nw.proxy {
                        for group_name in &nw.proxy_groups {
                            if let Some(group) = nw.groups.get(group_name) {
                                list.extend_from_slice(&group.plugins);
                            }
                        }
                    }

                    if let Some(global) = nw.groups.get("global") {
                        list.extend_from_slice(&global.plugins);
                    }
                }
                list
            }
            AddonType::Mod => {
                let mut list: Vec<Downloadable> = self.server.mods.clone();
                if let Some(nw) = &self.network {
                    if let Some(entry) = nw.servers.get(&self.server.name) {
                        for group_name in &entry.groups {
                            if let Some(group) = nw.groups.get(group_name) {
                                list.extend_from_slice(&group.mods);
                            }
                        }
                    }

                    if self.server.name == nw.proxy {
                        for group_name in &nw.proxy_groups {
                            if let Some(group) = nw.groups.get(group_name) {
                                list.extend_from_slice(&group.mods);
                            }
                        }
                    }

                    if let Some(global) = nw.groups.get("global") {
                        list.extend_from_slice(&global.mods);
                    }
                }
                list
            }
        }
    }

    pub fn var<I: AsRef<str>>(&self, var: I) -> Option<String> {
        let k = var.as_ref();
        match k {
            "SERVER_NAME" => Some(self.server.name.clone()),
            "SERVER_VERSION" | "mcver" | "mcversion" => Some(self.server.mc_version.clone()),

            "SERVER_PORT" => env::var(format!("PORT_{}", self.server.name)).ok().or(self
                .network
                .as_ref()
                .and_then(|nw| nw.servers.get(&self.server.name))
                .map(|s| s.port.to_string())),
            "SERVER_IP" => env::var(format!("IP_{}", self.server.name)).ok().or(self
                .network
                .as_ref()
                .and_then(|nw| nw.servers.get(&self.server.name))
                .and_then(|s| s.ip_address.clone())),

            "PLUGIN_COUNT" => Some(self.server.plugins.len().to_string()),
            "MOD_COUNT" => Some(self.server.mods.len().to_string()),
            "WORLD_COUNT" => Some(self.server.worlds.len().to_string()),
            "CLIENTSIDE_MOD_COUNT" => Some(self.server.clientsidemods.len().to_string()),

            "NETWORK_NAME" => Some(self.network.as_ref()?.name.clone()),
            "NETWORK_PORT" => Some(self.network.as_ref()?.port.to_string()),
            "NETWORK_SERVERS_COUNT" => Some(self.network.as_ref()?.servers.len().to_string()),

            "NETWORK_VELOCITY_SERVERS" => self.network.as_ref().map(|nw| {
                "# generated by mcman\n".to_owned()
                    + &nw
                        .servers
                        .iter()
                        .map(|(name, serv)| {
                            format!(
                                "{name} = \"{}:{}\"",
                                env::var(format!("IP_{name}"))
                                    .ok()
                                    .or(serv.ip_address.clone())
                                    .unwrap_or("127.0.0.1".to_owned()),
                                env::var(format!("PORT_{name}"))
                                    .ok()
                                    .unwrap_or(serv.port.to_string()),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
            }),

            "NETWORK_BUNGEECORD_SERVERS" => self.network.as_ref().map(|nw| {
                "# generated by mcman\nservers:".to_owned()
                    + &nw
                        .servers
                        .iter()
                        .map(|(name, serv)| {
                            format!(
                            "  {name}:\n    motd: {}\n    address: {}:{}\n    restricted: false",
                            self.var("MOTD").unwrap_or("a mcman-powered server".to_owned()),
                            env::var(format!("IP_{name}")).ok()
                                .or(serv.ip_address.clone())
                                .unwrap_or("127.0.0.1".to_owned()),
                            env::var(format!("PORT_{name}")).ok()
                                .unwrap_or(serv.port.to_string()),
                        )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
            }),

            // why not
            "TECHNOBLADE" => Some("Technoblade never dies".to_owned()),
            "denizs_gf" => Some("ily may".to_owned()),

            _ => None,
        }
        .or_else(|| {
            if let Ok(v) = std::env::var(k) {
                Some(v)
            } else if k.starts_with("NW_") {
                if let Some(nw) = &self.network {
                    if k.starts_with("NW_SERVER_") {
                        let (name, ty) = k.strip_prefix("NW_SERVER_").unwrap().split_once('_')?;

                        let serv = nw.servers.get(name)?;

                        let ip = env::var(format!("IP_{name}"))
                            .ok()
                            .or(serv.ip_address.clone())
                            .unwrap_or("127.0.0.1".to_owned());

                        let port = env::var(format!("PORT_{name}"))
                            .ok()
                            .unwrap_or(serv.port.to_string());

                        match ty.to_lowercase().as_str() {
                            "ip" => Some(ip),
                            "port" => Some(port),
                            "address" => Some(format!("{ip}:{port}")),
                            _ => None,
                        }
                    } else {
                        nw.variables.get(k.strip_prefix("NW_").unwrap()).cloned()
                    }
                } else {
                    None
                }
            } else {
                self.server.variables.get(k).cloned()
            }
        })
    }

    pub fn get_cache(&self, ns: &str) -> Option<Cache> {
        if self.config.disable_cache.iter().any(|s| s.as_str() == ns) {
            None
        } else {
            Cache::get_cache(ns)
        }
    }
}

macro_rules! api_methods {
    ($(
        $name:ident => $t:ident,
    )*) => {$(
        pub fn $name(&self) -> sources::$name::$t<'_> {
            sources::$name::$t(&self)
        }
    )*};
}

macro_rules! interop_methods {
    ($(
        $name:ident => $t:ident,
    )*) => {$(
        pub fn $name(&self) -> crate::interop::$name::$t<'_> {
            crate::interop::$name::$t(self)
        }
    )*};
}

macro_rules! interop_methods_mut {
    ($(
        $name:ident => $t:ident,
    )*) => {$(
        pub fn $name(&mut self) -> crate::interop::$name::$t<'_> {
            crate::interop::$name::$t(self)
        }
    )*};
}

impl App {
    api_methods! {
        vanilla => VanillaAPI,
        github => GithubAPI,
        maven => MavenAPI,
        jenkins => JenkinsAPI,
        modrinth => ModrinthAPI,
        curserinth => CurserinthAPI,
        neoforge => NeoforgeAPI,
        forge => ForgeAPI,
        fabric => FabricAPI,
        quilt => QuiltAPI,
        papermc => PaperMCAPI,
        hangar => HangarAPI,
        purpur => PurpurAPI,
        spigot => SpigotAPI,
        mclogs => MCLogsAPI,
    }

    interop_methods! {
        markdown => MarkdownAPI,
        worlds => WorldsAPI,
        hooks => HooksAPI,
    }

    interop_methods_mut! {
        packwiz => PackwizInterop,
        mrpack => MRPackInterop,
    }
}
