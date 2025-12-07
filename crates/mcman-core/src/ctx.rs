use miette::{IntoDiagnostic, Result};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::{Client, IntoUrl};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::de::DeserializeOwned;

pub const APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " - https://mcman.deniz.blue",
);

pub struct Context {
    http: ClientWithMiddleware,
}

impl Context {
    pub fn new() -> Result<Self> {
        let cache_path = dirs::cache_dir().expect("Cache dir to exist").join("mcman");

        let manager = CACacheManager::new(cache_path, true);

        let http = Client::builder().user_agent(APP_USER_AGENT).build().into_diagnostic()?;

        let http = ClientBuilder::new(http)
            .with(Cache(HttpCache {
                manager,
                mode: CacheMode::Default,
                options: HttpCacheOptions::default(),
            }))
            .build();

        Ok(Self { http })
    }

    pub async fn fetch_json<T: DeserializeOwned>(&self, url: impl IntoUrl) -> Result<T> {
        let data: T = self
            .http
            .get(url)
            .send()
            .await
            .into_diagnostic()?
            .error_for_status()
            .into_diagnostic()?
            .json()
            .await
            .into_diagnostic()?;

        Ok(data)
    }

    pub async fn fetch_text(&self, url: impl IntoUrl) -> Result<String> {
        let text = self.http
            .get(url)
            .send()
            .await
            .into_diagnostic()?
            .error_for_status()
            .into_diagnostic()?
            .text()
            .await
            .into_diagnostic()?;

        Ok(text)
    }

    pub async fn fetch_xml_owned<T: DeserializeOwned>(&self, url: impl IntoUrl) -> Result<T> {
        let text = self.fetch_text(url).await?;
        let data = serde_roxmltree::from_str::<T>(&text).into_diagnostic()?;
        Ok(data)
    }
}
