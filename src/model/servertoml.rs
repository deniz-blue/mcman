use std::{
    collections::HashMap,
    env,
    fs::{read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::util::dollar_repl;

use super::{ClientSideMod, Downloadable, Hook, ServerLauncher, ServerType, World};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct MarkdownOptions {
    pub files: Vec<String>,
    pub auto_update: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            files: vec!["README.md".to_owned()],
            auto_update: false,
        }
    }
}

impl MarkdownOptions {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct Server {
    #[serde(skip)]
    pub path: PathBuf,

    pub name: String,
    pub mc_version: String, // TODO: version type for comparing
    #[serde(with = "super::servertype::parse")]
    pub jar: ServerType,
    pub variables: HashMap<String, String>,
    pub launcher: ServerLauncher,

    #[serde(default)]
    #[serde(skip_serializing_if = "MarkdownOptions::is_empty")]
    pub markdown: MarkdownOptions,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub hooks: HashMap<String, Hook>,

    #[serde(default)]
    pub options: ServerOptions,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub worlds: HashMap<String, World>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<Downloadable>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<Downloadable>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub clientsidemods: Vec<ClientSideMod>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct ServerOptions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bootstrap_exts: Vec<String>,

    #[serde(
        default = "default_success_line",
        skip_serializing_if = "is_default_success_line"
    )]
    pub success_line: String,

    #[serde(
        default = "default_stop_command",
        skip_serializing_if = "is_default_stop_command"
    )]
    pub stop_command: String,
}

pub fn default_success_line() -> String {
    String::from("]: Done")
}

pub fn is_default_success_line(s: &str) -> bool {
    s == default_success_line()
}

pub fn default_stop_command() -> String {
    String::from("stop")
}

pub fn is_default_stop_command(s: &str) -> bool {
    s == default_stop_command()
}

impl Server {
    pub fn load() -> Result<Self> {
        let mut path = env::current_dir()?;
        let file = Path::new("server.toml");

        let found_path = loop {
            path.push(file);

            if path.is_file() {
                break path;
            }

            if !(path.pop() && path.pop()) {
                bail!("Couldn't find server.toml - use `mcman init` to create one?");
            }
        };

        Self::load_from(&found_path)
    }

    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let data = read_to_string(path)?;
        let mut serv: Self = toml::from_str(&data)?;
        serv.path = path
            .parent()
            .ok_or(anyhow!("Couldnt get parent dir"))?
            .canonicalize()?;
        Ok(serv)
    }

    pub fn save(&self) -> Result<()> {
        let cfg_str = toml::to_string_pretty(&self)?;
        let mut f = File::create(self.path.join("server.toml"))?;
        f.write_all(cfg_str.as_bytes())?;

        Ok(())
    }

    pub fn format(&self, str: &str) -> String {
        dollar_repl(str, |key| match key {
            "mcver" | "mcversion" | "SERVER_VERSION" => Some(self.mc_version.clone()),
            "SERVER_NAME" => Some(self.name.clone()),
            k => self.variables.get(k).cloned(),
        })
    }

    pub fn fill_from_map(&mut self, map: &HashMap<String, String>) {
        if let Some(v) = map.get("minecraft") {
            self.mc_version.clone_from(v);
        }

        if let Some(v) = map.get("forge") {
            self.jar = ServerType::Forge { loader: v.clone() }
        }

        if let Some(v) = map.get("neoforge") {
            self.jar = ServerType::NeoForge { loader: v.clone() }
        }

        if let Some(v) = map.get("fabric-loader").or(map.get("fabric")) {
            self.jar = ServerType::Fabric {
                loader: v.clone(),
                installer: "latest".to_owned(),
            }
        }

        if let Some(v) = map.get("quilt-loader").or(map.get("quilt")) {
            self.jar = ServerType::Quilt {
                loader: v.clone(),
                installer: "latest".to_owned(),
            }
        }
    }

    pub fn to_map(&self, include_loader: bool) -> HashMap<String, String> {
        let mut map = HashMap::from([("minecraft".to_owned(), self.mc_version.clone())]);

        let l = if include_loader { "-loader" } else { "" };

        if let Some((k, v)) = match &self.jar {
            ServerType::Quilt { loader, .. } => Some((format!("quilt{l}"), loader.clone())),
            ServerType::Fabric { loader, .. } => Some((format!("fabric{l}"), loader.clone())),
            ServerType::Forge { loader, .. } => Some(("forge".to_owned(), loader.clone())),
            ServerType::NeoForge { loader, .. } => Some(("neoforge".to_owned(), loader.clone())),
            _ => None,
        } {
            map.insert(k, v);
        }

        map
    }
}

impl Default for Server {
    fn default() -> Self {
        let mut vars = HashMap::new();
        vars.insert("SERVER_PORT".to_owned(), "25565".to_owned());
        Self {
            path: PathBuf::from("."),
            name: String::new(),
            mc_version: "latest".to_owned(),
            jar: ServerType::Vanilla {},
            variables: vars,
            launcher: ServerLauncher::default(),
            markdown: MarkdownOptions::default(),
            hooks: HashMap::new(),
            options: ServerOptions::default(),
            worlds: HashMap::new(),
            plugins: vec![],
            mods: vec![],
            clientsidemods: vec![],
        }
    }
}
