use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};
use std::borrow::Cow;

use crate::util::SelectItem;

use super::ServerType;

impl ServerType {
    pub fn select_jar_interactive() -> Result<Self> {
        let items = vec![
            SelectItem(0, Cow::Borrowed("Vanilla    - No patches")),
            SelectItem(1, Cow::Borrowed("PaperMC    - Spigot fork, most popular")),
            SelectItem(2, Cow::Borrowed("Purpur     - Paper fork")),
            SelectItem(3, Cow::Borrowed("BuildTools - Spigot or CraftBukkit")),
        ];

        let jar_type = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which server software to use?")
            .default(0)
            .items(&items)
            .interact()?;

        Ok(match jar_type {
            0 => Self::Vanilla {},
            1 => Self::Paper {},
            2 => Self::Purpur {
                build: "latest".to_owned(),
            },
            3 => {
                let items = vec![
                    SelectItem(
                        Self::BuildTools {
                            args: vec![],
                            software: Cow::Borrowed("spigot"),
                        },
                        Cow::Borrowed("Spigot"),
                    ),
                    SelectItem(
                        Self::BuildTools {
                            args: vec![],
                            software: Cow::Borrowed("craftbukkit"),
                        },
                        Cow::Borrowed("CraftBukkit"),
                    ),
                ];

                let idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Which one?")
                    .default(0)
                    .items(&items)
                    .interact()?;

                items[idx].0.clone()
            }
            _ => unreachable!(),
        })
    }

    pub fn select_modded_jar_interactive() -> Result<Self> {
        let items = [
            (0, "Quilt (fabric compatible)"),
            (1, "Fabric"),
            (2, "NeoForge"),
            (3, "Forge"),
        ];

        let items_str: Vec<String> = items.iter().map(|v| v.1.to_owned()).collect();

        let jar_type = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which mod loader to use?")
            .default(0)
            .items(&items_str)
            .interact()?;

        Ok(match jar_type {
            0 => Self::Quilt {
                loader: "latest".to_owned(),
                installer: "latest".to_owned(),
            },
            1 => Self::Fabric {
                loader: "latest".to_owned(),
                installer: "latest".to_owned(),
            },
            2 => Self::NeoForge {
                loader: "latest".to_owned(),
            },
            3 => Self::Forge {
                loader: "latest".to_owned(),
            },
            _ => unreachable!(),
        })
    }

    pub fn select_proxy_jar_interactive() -> Result<Self> {
        let items = [
            (0, "Velocity   - Modern, high perf. proxy by PaperMC"),
            (1, "Waterfall  - BungeeCord fork by PaperMC"),
            (2, "Bungeecord - By md5 (Spigot)"),
        ];

        let items_str: Vec<String> = items.iter().map(|v| v.1.to_owned()).collect();

        let jar_type = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which proxy software to use?")
            .default(0)
            .items(&items_str)
            .interact()?;

        Ok(match jar_type {
            0 => Self::Velocity {},
            1 => Self::Waterfall {},
            2 => Self::BungeeCord {},
            _ => unreachable!(),
        })
    }
}
