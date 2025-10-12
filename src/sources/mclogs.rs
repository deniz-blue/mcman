use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::app::App;

const API_V1: &str = "https://api.mclo.gs/1";

pub struct MCLogsAPI<'a>(pub &'a App);

impl MCLogsAPI<'_> {
    pub async fn paste_log(&self, content: &str) -> Result<LogFileMetadata> {
        let params = HashMap::from([("content", content)]);

        let json = self
            .0
            .http_client
            .post(format!("{API_V1}/log"))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json::<MaybeSuccess<LogFileMetadata>>()
            .await?;

        json.into()
    }

    #[allow(unused)]
    pub async fn fetch_insights(&self, id: &str) -> Result<LogInsights> {
        let json = self
            .0
            .http_client
            .post(format!("{API_V1}/insights/{id}"))
            .send()
            .await?
            .error_for_status()?
            .json::<MaybeSuccess<LogInsights>>()
            .await?;

        json.into()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum MaybeSuccess<T> {
    Error {
        error: String,
    },
    Success {
        #[serde(flatten)]
        value: T,
    },
}

impl<T> From<MaybeSuccess<T>> for Result<T> {
    fn from(val: MaybeSuccess<T>) -> Self {
        match val {
            MaybeSuccess::Success { value } => Ok(value),
            MaybeSuccess::Error { error } => Err(anyhow!(error)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LogFileMetadata {
    pub id: String,
    pub url: String,
    pub raw: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LogInsights {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub log_type: String,
    pub version: String,
    pub title: String,
    pub analysis: LogAnalysis,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LogAnalysis {
    pub problems: Vec<Problem>,
    pub information: Vec<Information>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Problem {
    pub message: String,
    pub counter: usize,
    pub entry: AnalysisEntry,
    pub solutions: Vec<Solution>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Information {
    pub message: String,
    pub counter: usize,
    pub label: String,
    pub value: String,
    pub entry: AnalysisEntry,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnalysisEntry {
    pub level: usize,
    pub time: Option<String>,
    pub prefix: String,
    pub lines: Vec<AnalysisLine>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnalysisLine {
    pub number: usize,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Solution {
    pub message: String,
}
