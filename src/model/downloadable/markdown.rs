use indexmap::IndexMap;

use crate::{model::Downloadable, sources::jenkins::JenkinsAPI};
use std::{borrow::Cow, fmt::Display};

impl Downloadable {
    pub fn get_md_link(&self) -> String {
        match self {
            Self::Url { url, filename, .. } => {
                let hyperlink = format!("[URL]({url})");

                if let Some(f) = filename {
                    format!("`{f}` (Custom {hyperlink})")
                } else {
                    format!("Custom {hyperlink}")
                }
            }
            Self::GithubRelease { repo, .. } => {
                format!("[{repo}](https://github.com/{repo})")
            }
            Self::Hangar { id, .. } => {
                format!("[{id}](https://hangar.papermc.io/{id})")
            }
            Self::Jenkins { url, job, .. } => {
                format!("[{job}]({})", JenkinsAPI::get_url(url, job))
            }
            Self::Maven { url, group, .. } => {
                format!("[{g}]({url}/{g})", g = group.replace('.', "/"))
            }
            Self::Spigot { id, .. } => {
                format!("[{id}](https://www.spigotmc.org/resources/{id})")
            }
            Self::Modrinth { id, .. } => {
                format!("[{id}](https://modrinth.com/mod/{id})")
            }
            Self::CurseRinth { id, .. } => {
                format!("`{id}`<sup>[CF](https://www.curseforge.com/minecraft/mc-mods/{id}) [CR](https://curserinth.kuylar.dev/mod/{id})</sup>")
            }
            Self::CurseForge { id, .. } => {
                format!("`{id}`<sup>[CF](https://www.curseforge.com/minecraft/mc-mods/{id})</sup>")
            }
        }
    }

    pub fn get_type_name(&self) -> String {
        match self {
            Self::Url { .. } => "URL",
            Self::GithubRelease { .. } => "GithubRel",
            Self::Jenkins { .. } => "Jenkins",
            Self::Hangar { .. } => "Hangar",
            Self::Modrinth { .. } => "Modrinth",
            Self::CurseRinth { .. } => "CurseRinth",
            Self::CurseForge { .. } => "CurseForge",
            Self::Spigot { .. } => "Spigot",
            Self::Maven { .. } => "Maven",
        }
        .to_owned()
    }

    pub fn fields_to_map(&self) -> IndexMap<Cow<'static, str>, String> {
        let mut map = IndexMap::new();

        map.insert(Cow::Borrowed("Type"), self.get_type_name());

        let (project_url, asset, version) = match self {
            Self::Url { url, filename, .. } => (
                url.clone(),
                Some(filename.as_ref().unwrap_or(&String::new()).clone()),
                None,
            ),

            Self::GithubRelease { repo, tag, asset } => {
                (repo.clone(), Some(asset.clone()), Some(tag.clone()))
            }

            Self::Modrinth { id, version }
            | Self::CurseRinth { id, version }
            | Self::CurseForge { id, version }
            | Self::Hangar { id, version }
            | Self::Spigot { id, version } => (id.clone(), None, Some(version.clone())),

            Self::Jenkins {
                url,
                job,
                build,
                artifact,
            } => (
                format!("{job} - ({url})"),
                Some(artifact.clone()),
                Some(build.clone()),
            ),

            Self::Maven {
                url,
                group,
                artifact,
                version,
                filename,
            } => (
                format!("{group}.{artifact} - ({url})"),
                Some(filename.clone()),
                Some(version.clone()),
            ),
        };

        map.insert(Cow::Borrowed("Project/URL"), project_url);

        if let Some(version) = version {
            map.insert(Cow::Borrowed("Version/Release"), version);
        }

        if let Some(asset) = asset {
            map.insert(Cow::Borrowed("Asset/File"), asset);
        }

        map
    }

    pub fn to_short_string(&self) -> String {
        match self {
            Self::Modrinth { id, .. } => format!("Modrinth:{id}"),
            Self::Hangar { id, .. } => format!("Hangar:{id}"),
            Self::CurseRinth { id, .. } => format!("CurseRinth:{id}"),
            Self::CurseForge { id, .. } => format!("CurseForge:{id}"),
            Self::Spigot { id, .. } => format!("Spigot:{id}"),
            Self::GithubRelease { repo, .. } => format!("Github:{repo}"),
            Self::Jenkins { job, .. } => format!("Jenkins:{job}"),
            Self::Url { filename, .. } => {
                if let Some(f) = filename {
                    format!("URL:{f}")
                } else {
                    "URL".to_string()
                }
            }
            Self::Maven {
                group, artifact, ..
            } => {
                format!("Maven:{group}.{artifact}")
            }
        }
    }
}

impl Display for Downloadable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_short_string())
    }
}
