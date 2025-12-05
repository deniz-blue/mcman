use serde::{Deserialize, Serialize};

use crate::sources::minecraft::models::PistonRule;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VersionArguments {
    pub game: Vec<PistonArgument>,
    pub jvm: Vec<PistonArgument>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum PistonArgument {
    Normal(String),
    Ruled {
        rules: Vec<PistonRule>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Many(Vec<String>),
}
