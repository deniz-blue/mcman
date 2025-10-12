use std::{borrow::Cow, env, fmt::Display};

use anyhow::Result;
use console::{style, StyledObject};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::util::SelectItem;

use super::App;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressPrefix {
    Resolving,
    Checking,
    Downloading,
    Copying,
    Fetching,
    Exporting,
}

macro_rules! enum_to_string {
    ($input:ident,$enum_type:ty,$($value:ident,)*) => {
        match $input {
            $(
                <$enum_type>::$value => stringify!($value),
            )*
        }
    };
}

impl From<ProgressPrefix> for Cow<'static, str> {
    fn from(val: ProgressPrefix) -> Self {
        Cow::Borrowed(enum_to_string!(
            val,
            ProgressPrefix,
            Resolving,
            Checking,
            Downloading,
            Copying,
            Fetching,
            Exporting,
        ))
    }
}

// 12
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Prefix {
    Skipped,
    SkippedWarning,
    Copied,
    Downloaded,

    Imported,
    Exported,
    Rendered,
    Unpacked,
    Packed,

    Warning,
    Error,
    Info,
    Debug,
}

impl Prefix {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Copied => "      Copied",
            Self::Skipped => "     Skipped",
            Self::SkippedWarning => "   ! Skipped",
            Self::Downloaded => "  Downloaded",

            Self::Imported => "    Imported",
            Self::Exported => "    Exported",
            Self::Rendered => "    Rendered",
            Self::Unpacked => "    Unpacked",
            Self::Packed => "      Packed",

            Self::Error => "     ⚠ Error",
            Self::Warning => "      ⚠ Warn",
            Self::Info => "      🛈 Info",
            Self::Debug => "       debug",
        }
    }

    pub fn styled(self) -> StyledObject<&'static str> {
        match self {
            Self::Downloaded
            | Self::Imported
            | Self::Exported
            | Self::Rendered
            | Self::Packed
            | Self::Unpacked => style(self.as_str()).green().bold(),
            Self::Copied | Self::Skipped => style(self.as_str()).green(),
            Self::Error => style(self.as_str()).red().bold(),
            Self::Warning | Self::SkippedWarning => style(self.as_str()).yellow().bold(),
            Self::Info => style(self.as_str()).bold(),
            Self::Debug => style(self.as_str()).dim(),
        }
    }
}

impl From<Prefix> for Cow<'static, str> {
    fn from(val: Prefix) -> Self {
        Cow::Borrowed(val.as_str().trim_start())
    }
}

impl App {
    pub fn println<S: Display>(&self, message: S) {
        self.multi_progress.suspend(|| println!("{message}"));
    }

    pub fn success<S: Display>(&self, message: S) {
        self.println(format!(
            "  {} {message}",
            ColorfulTheme::default().success_prefix
        ));
    }

    pub fn log<S: Display>(&self, message: S) {
        self.println(format!("  {message}"));
    }

    pub fn log_dev<S: Display>(&self, message: S) {
        self.println(format!("🛈 {message}"));
    }

    pub fn notify<S: Display>(&self, prefix: Prefix, message: S) {
        self.println(format!("{} {message}", prefix.styled()));
    }

    pub fn warn<S: Display>(&self, message: S) {
        self.notify(Prefix::Warning, message);
    }

    #[allow(dead_code)]
    pub fn error<S: Display>(&self, message: S) {
        self.notify(Prefix::Error, message);
    }

    pub fn info<S: Display>(&self, message: S) {
        self.notify(Prefix::Info, message);
    }

    pub fn dbg<S: Display>(&self, message: S) {
        if env::var("MCMAN_DEBUG").is_ok_and(|s| s.as_str() == "true") {
            self.notify(Prefix::Debug, message);
        }
    }

    pub fn print_job(&self, job: &str) {
        if !self.is_ci() {
            self.println(format!(
                "{} {}",
                ColorfulTheme::default().active_item_prefix,
                style(job).cyan().bold()
            ));
        }
    }

    #[allow(clippy::unused_self)]
    pub fn is_ci(&self) -> bool {
        env::var("CI").is_ok_and(|s| s.as_str() == "true")
    }

    pub fn ci(&self, cmd: &str) {
        if self.is_ci() {
            self.println(cmd);
        }
    }

    pub fn prompt_string(&self, prompt: &str) -> Result<String> {
        Ok(self.multi_progress.suspend(|| {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .interact_text()
        })?)
    }

    pub fn prompt_string_default(&self, prompt: &str, default: &str) -> Result<String> {
        Ok(self.multi_progress.suspend(|| {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .default(default.to_owned())
                .interact_text()
        })?)
    }

    pub fn prompt_string_filled(&self, prompt: &str, default: &str) -> Result<String> {
        Ok(self.multi_progress.suspend(|| {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .default(default.to_owned())
                .with_initial_text(default.to_owned())
                .interact_text()
        })?)
    }

    pub fn confirm(&self, prompt: &str) -> Result<bool> {
        Ok(self.multi_progress.suspend(|| {
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .wait_for_newline(true)
                .interact()
        })?)
    }

    pub fn select<T: Clone>(&self, prompt: &str, items: &[SelectItem<T>]) -> Result<T> {
        self.select_with_default(prompt, items, 0)
    }

    pub fn select_with_default<T: Clone>(
        &self,
        prompt: &str,
        items: &[SelectItem<T>],
        def: usize,
    ) -> Result<T> {
        let item = &items[self.multi_progress.suspend(|| {
            Select::with_theme(&ColorfulTheme::default())
                .items(items)
                .with_prompt(prompt)
                .default(def)
                .max_length(5)
                .interact()
        })?];

        Ok(item.0.clone())
    }
}
