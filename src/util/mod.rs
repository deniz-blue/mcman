use std::{borrow::Cow, cmp::Ordering, fmt::Display};

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

pub mod env;
pub mod maven_import;
pub mod md;

pub struct SelectItem<T>(pub T, pub Cow<'static, str>);

impl<T> Display for SelectItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}

pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub fn is_default_str(s: &str) -> bool {
    s == "latest"
}

pub fn get_latest_semver(list: &[String]) -> Option<String> {
    let mut list = list
        .iter()
        .map(|s| s.split('.').collect())
        .collect::<Vec<Vec<_>>>();

    list.sort_by(|a, b| {
        let mut ia = a.iter();
        let mut ib = b.iter();
        loop {
            break match (ia.next(), ib.next()) {
                (Some(a), Some(b)) => {
                    let a = a.parse::<u32>();
                    let b = b.parse::<u32>();

                    match (a, b) {
                        (Ok(a), Ok(b)) => match a.cmp(&b) {
                            Ordering::Equal => continue,
                            ord => ord,
                        },
                        (Err(_), Ok(_)) => Ordering::Less,
                        (Ok(_), Err(_)) => Ordering::Greater,
                        _ => Ordering::Equal,
                    }
                }
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                _ => Ordering::Equal,
            };
        }
    });

    list.last().map(|v| v.join("."))
}

/// ci.luckto.me => ci-lucko-me
pub fn url_to_folder(url: &str) -> String {
    url.replace("https://", "")
        .replace("http://", "")
        .replace('/', " ")
        .trim()
        .replace(' ', "-")
}

static SANITIZE_R1: &str = "<(?:\"[^\"]*\"['\"]*|'[^']*'['\"]*|[^'\">])+>";

pub fn sanitize(s: &str) -> Result<String> {
    let re = Regex::new(SANITIZE_R1)?;

    Ok(re
        .replace_all(
            &s.replace('\n', " ").replace('\r', "").replace("<br>", " "),
            "",
        )
        .to_string())
}

lazy_static! {
    static ref DOLLAR_REGEX: Regex = Regex::new(r"\$\{(\w+)?\}").unwrap();
}

/// Utility fn for replacing strings containing "${}"
pub fn dollar_repl<F>(input: &str, replacer: F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    DOLLAR_REGEX
        .replace_all(input, |caps: &regex::Captures| {
            let var_name = caps.get(1).map(|v| v.as_str()).unwrap_or_default();

            if let Some(v) = replacer(var_name) {
                v
            } else {
                format!("${{{var_name}}}")
            }
        })
        .into_owned()
}
