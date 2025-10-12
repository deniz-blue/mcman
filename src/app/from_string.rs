use std::{borrow::Cow, path::Path};

use anyhow::{bail, Context, Result};

use crate::{model::Downloadable, util::SelectItem};

use super::App;

impl App {
    pub async fn dl_from_string(&self, s: &str) -> Result<Downloadable> {
        if s.starts_with("http") {
            Ok(self
                .dl_from_url(s)
                .await
                .context(format!("Importing URL: {s}"))?)
        } else if s.contains(':') {
            match s.split_once(':').unwrap() {
                ("mr" | "modrinth", id) => {
                    let (id, version) = id.split_once(',').unwrap_or((id, "latest"));
                    Ok(Downloadable::Modrinth {
                        id: id.to_owned(),
                        version: version.to_owned(),
                    })
                }
                ("cr" | "cf" | "curseforge" | "curserinth", id) => {
                    let (id, version) = id.split_once(',').unwrap_or((id, "latest"));
                    Ok(Downloadable::CurseRinth {
                        id: id.to_owned(),
                        version: version.to_owned(),
                    })
                }
                ("hangar" | "h", id) => {
                    let (id, version) = id.split_once(',').unwrap_or((id, "latest"));
                    Ok(Downloadable::Hangar {
                        id: id.to_owned(),
                        version: version.to_owned(),
                    })
                }
                ("spigot" | "spiget", id) => {
                    let (id, version) = id.split_once(',').unwrap_or((id, "latest"));
                    Ok(Downloadable::Spigot {
                        id: id.to_owned(),
                        version: version.to_owned(),
                    })
                }
                ("ghrel" | "gh" | "github", id) => {
                    let (repo, tag) = id.split_once(',').unwrap_or((id, "latest"));

                    Ok(Downloadable::GithubRelease {
                        repo: repo.to_owned(),
                        tag: tag.to_owned(),
                        asset: "first".to_owned(),
                    })
                }
                (ty, _) => bail!("Unknown identifier '{ty}'"),
            }
        } else {
            bail!("I dont know what to do with '{s}'...");
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn dl_from_url(&self, urlstr: &str) -> Result<Downloadable> {
        let url = reqwest::Url::parse(urlstr.trim_end_matches('/'))?;

        match (
            url.domain(),
            url.path_segments()
                .map(Iterator::collect::<Vec<_>>)
                .unwrap_or_default()
                .as_slice(),
        ) {
            // https://cdn.modrinth.com/data/{ID}/versions/{VERSION}/{FILENAME}
            (Some("cdn.modrinth.com"), ["data", id, "versions", version, _filename]) => {
                Ok(Downloadable::Modrinth {
                    id: id.to_owned().to_owned(),
                    version: version.to_owned().to_owned(),
                })
            }

            (Some("modrinth.com"), ["mod" | "plugin" | "datapack", id, rest @ ..]) => {
                let version = if let ["version", v] = rest {
                    (*v).to_string()
                } else {
                    let versions = self.modrinth().fetch_versions(id).await?;

                    if versions.is_empty() {
                        bail!("No compatible versions found");
                    }

                    let version = self.select(
                        "Select a version",
                        &versions
                            .iter()
                            .map(|v| {
                                SelectItem(
                                    v.clone(),
                                    Cow::Owned(if v.version_number == v.name {
                                        v.version_number.clone()
                                    } else {
                                        format!("[{}] {}", v.version_number, v.name)
                                    }),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )?;

                    version.id.clone()
                };

                Ok(Downloadable::Modrinth {
                    id: id.to_owned().to_owned(),
                    version: version.clone(),
                })
            }

            (Some("curserinth.kuylar.dev"), ["mod", id, rest @ ..]) => {
                let version = if let ["version", v] = rest {
                    (*v).to_string()
                } else {
                    let (versions, _) = self.curserinth().fetch_versions(id).await?;

                    if versions.is_empty() {
                        bail!("No compatible versions found");
                    }

                    let version = self.select(
                        "Select a version",
                        &versions
                            .iter()
                            .map(|v| {
                                SelectItem(
                                    v.clone(),
                                    Cow::Owned(if v.version_number == v.name {
                                        v.version_number.clone()
                                    } else {
                                        format!("[{}] {}", v.version_number, v.name)
                                    }),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )?;

                    version.id.clone()
                };

                Ok(Downloadable::CurseRinth {
                    id: (*id).to_string(),
                    version,
                })
            }

            // https://www.curseforge.com/minecraft/mc-mods/betterwithpatches
            (Some("www.curseforge.com"), ["minecraft", "mc-mods", id, rest @ ..]) => {
                let id = format!("mod__{id}");

                let version = if let [_, ver] = rest {
                    (*ver).to_string()
                } else {
                    let (versions, _) = self.curserinth().fetch_versions(&id).await?;

                    if versions.is_empty() {
                        bail!("No compatible versions found");
                    }

                    let version = self.select(
                        "Select a version",
                        &versions
                            .iter()
                            .map(|v| {
                                SelectItem(
                                    v.clone(),
                                    Cow::Owned(if v.version_number == v.name {
                                        v.version_number.clone()
                                    } else {
                                        format!("[{}] {}", v.version_number, v.name)
                                    }),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )?;

                    version.id.clone()
                };

                Ok(Downloadable::CurseRinth { id, version })
            }

            // https://www.spigotmc.org/resources/http-requests.101253/
            (Some("www.spigotmc.org"), ["resources", id]) => Ok(Downloadable::Spigot {
                id: (*id).to_string(),
                version: "latest".to_owned(),
            }),

            // https://github.com/{owner}/{repo}/releases/{'tag'|'download'}/{tag}/{filename}
            (Some("github.com"), [owner, repo_name, rest @ ..]) => {
                let repo = format!("{owner}/{repo_name}");

                let (tag, asset) =
                    if let ["releases", "tag" | "download", tag, filename @ ..] = rest {
                        (
                            (*tag).to_string(),
                            match filename {
                                [f] => Some(f.replace(tag, "${tag}")),
                                _ => None,
                            },
                        )
                    } else {
                        let releases = self.github().fetch_releases(&repo).await?;

                        let version = self.select(
                            "Select a release",
                            &[SelectItem(
                                "latest".to_owned(),
                                Cow::Borrowed("Always use latest release"),
                            )]
                            .into_iter()
                            .chain(releases.iter().map(|r| {
                                SelectItem(
                                    r.tag_name.clone(),
                                    Cow::Owned(if r.tag_name == r.name {
                                        r.name.clone()
                                    } else {
                                        format!("[{}] {}", r.tag_name, r.name)
                                    }),
                                )
                            }))
                            .collect::<Vec<_>>(),
                        )?;

                        (version, None)
                    };

                let asset = if let Some(a) = asset {
                    a
                } else {
                    let rel = self.github().fetch_release(&repo, &tag).await?;

                    if rel.assets.len() <= 1 {
                        "first".to_owned()
                    } else {
                        match self.select(
                            "Which asset to use?",
                            &[SelectItem(
                                Some("first".to_owned()),
                                Cow::Owned(format!(
                                    "Use the first asset ('{}' for '{}')",
                                    rel.assets[0].name, rel.tag_name
                                )),
                            )]
                            .into_iter()
                            .chain(rel.assets.iter().map(|a| {
                                SelectItem(Some(a.name.clone()), Cow::Owned(a.name.clone()))
                            }))
                            .chain(vec![SelectItem(None, Cow::Borrowed("Set manually"))])
                            .collect::<Vec<_>>(),
                        )? {
                            Some(a) => a,
                            None => self.prompt_string("Enter asset name")?,
                        }
                    }
                };

                Ok(Downloadable::GithubRelease {
                    repo,
                    tag: tag.to_string(),
                    asset,
                })
            }

            (domain, path) => {
                let def = match domain {
                    Some(d) if d.starts_with("ci.") => 1,
                    Some(d) if d.starts_with("maven.") || d.starts_with("mvn") => 2,
                    _ => {
                        if path.ends_with(&["maven-metadata.xml"]) {
                            2
                        } else {
                            0
                        }
                    }
                };

                let selection = self.select_with_default(
                    urlstr,
                    &[
                        SelectItem(0, Cow::Borrowed("Add as Custom URL")),
                        SelectItem(1, Cow::Borrowed("Add as Jenkins")),
                        SelectItem(2, Cow::Borrowed("Add as Maven")),
                    ],
                    def,
                )?;

                match selection {
                    0 => {
                        let inferred = urlstr
                            .split('?')
                            .next()
                            .unwrap_or(urlstr)
                            .split('/')
                            .next_back()
                            .unwrap();

                        let input = self.prompt_string_filled("Filename?", inferred)?;
                        let desc =
                            self.prompt_string_default("Optional description/comment?", "")?;

                        Ok(Downloadable::Url {
                            url: urlstr.to_owned(),
                            filename: Some(input),
                            desc: if desc.is_empty() { None } else { Some(desc) },
                        })
                    }
                    1 => {
                        // TODO: ...
                        let j_url = if self.confirm(&format!(
                            "Is the Jenkins URL 'https://{}'?",
                            url.domain().unwrap()
                        ))? {
                            format!("https://{}", url.domain().unwrap())
                        } else {
                            self.prompt_string_filled(
                                "Enter Jenkins URL",
                                &format!("https://{}", url.domain().unwrap()),
                            )?
                        };

                        let inferred_job = {
                            let mut job = String::new();

                            if let Some(mut segments) = url.path_segments() {
                                loop {
                                    if segments.next().unwrap_or_default() == "job" {
                                        if let Some(job_name) = segments.next() {
                                            if !job.is_empty() {
                                                job += "/";
                                            }

                                            job += job_name;
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }

                            job
                        };

                        let job = self.prompt_string_filled("Job", &inferred_job)?;
                        let build = self.prompt_string_filled("Build", "latest")?;
                        let artifact = self.prompt_string_filled("Artifact", "first")?;

                        Ok(Downloadable::Jenkins {
                            url: j_url,
                            job,
                            build,
                            artifact,
                        })
                    }
                    2 => {
                        let mut repo = None;
                        let mut group_id = None;
                        let mut artifact_id = None;
                        //let mut ver = None;

                        if let Ok(meta) = self.maven().find_maven_artifact(urlstr).await {
                            if let Some((inferred_repo, _rest)) = meta.find_url(urlstr) {
                                if self.confirm(&format!(
                                    "Is '{inferred_repo}' the maven repository url?"
                                ))? {
                                    repo = Some(inferred_repo);
                                }

                                /* ver = if !rest.is_empty() {
                                    rest.split('/').next()
                                } else { None } */
                            }
                            group_id = meta.group_id;
                            artifact_id = meta.artifact_id;
                        }

                        let repo = match repo {
                            Some(r) => r,
                            None => self.prompt_string_filled("Maven repository url?", urlstr)?,
                        };

                        let group = if let Some(r) = group_id {
                            r
                        } else {
                            let inferred = if urlstr.starts_with(&repo) {
                                let p = urlstr.strip_prefix(&repo).unwrap();
                                if Path::new(p)
                                    .extension()
                                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
                                {
                                    let mut li = p.rsplit('/').skip(2).collect::<Vec<_>>();
                                    li.reverse();
                                    li
                                } else {
                                    p.split('/').collect::<Vec<_>>()
                                }
                                .into_iter()
                                .filter(|x| !x.is_empty())
                                .collect::<Vec<_>>()
                                .join(".")
                            } else {
                                String::new()
                            };

                            self.prompt_string_filled("Group (split by .)?", &inferred)?
                        };

                        let suggest = format!("{repo}/{}", group.replace('.', "/"));

                        let artifact = match artifact_id {
                            Some(r) => r,
                            None => self.prompt_string_filled(
                                "Artifact?",
                                if urlstr.starts_with(&suggest) {
                                    urlstr
                                        .strip_prefix(&suggest)
                                        .unwrap()
                                        .split('/')
                                        .find(|x| !x.is_empty())
                                        .unwrap_or_default()
                                } else {
                                    ""
                                },
                            )?,
                        };

                        let mut versions = vec![SelectItem(
                            "latest".to_owned(),
                            Cow::Borrowed("Always use latest"),
                        )];

                        for v in self
                            .maven()
                            .fetch_metadata(&repo, &group, &artifact)
                            .await?
                            .versions
                        {
                            versions.push(SelectItem(v.clone(), Cow::Owned(v.clone())));
                        }

                        let version = self.select("Which version?", &versions)?;

                        let filename = if Path::new(urlstr)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
                        {
                            urlstr.rsplit('/').next().unwrap().to_owned()
                        } else {
                            self.prompt_string_default("Filename?", "${artifact}-${version}.jar")?
                        };

                        Ok(Downloadable::Maven {
                            url: repo,
                            group,
                            artifact,
                            version,
                            filename,
                        })
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
