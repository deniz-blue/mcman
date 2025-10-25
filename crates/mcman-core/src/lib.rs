use anyhow::Result;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use serde::de::DeserializeOwned;

pub trait Context {
    pub async fn fetch_json<T: DeserializeOwned>(&self, url: &str) -> Result<T>;
}

pub struct DefaultContext {
    http: Client,
}

impl DefaultContext {
    pub fn new() -> Self {
        let cache_path = dirs::cache_dir().expect("Cache dir to exist").join("mcman");

        let manager = CACacheManager::new(cache_path, true);

        let http = ClientBuilder::new(Client::new())
            .with(Cache(HttpCache {
                manager,
                mode: CacheMode::Default,
                options: HttpCacheOptions(),
            }))
            .build();

        Self { http }
    }
}

impl Context for DefaultContext {
    async fn fetch_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let res = self.http.get(url)
            .send()
            .await?
            .error_for_status()?
            .json();

        let content = res.text().await?;

        let data: T = serde_json::from_str(&content)?;

        Ok(data)
    }
}
