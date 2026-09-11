use std::path::PathBuf;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

pub fn create_base_http_client() -> ClientWithMiddleware {
    ClientBuilder::new(inner_client())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy()))
        .build()
}

pub fn create_cached_http_client(cache: PathBuf) -> ClientWithMiddleware {
    ClientBuilder::new(inner_client())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::new(cache, false),
            options: HttpCacheOptions::default(),
        }))
        .with(RetryTransientMiddleware::new_with_policy(retry_policy()))
        .build()
}

fn inner_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(format!(
            "mcman/{} (https://github.com/deniz-blue/mcman)",
            env!("CARGO_PKG_VERSION")
        ))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to build base reqwest client")
}

fn retry_policy() -> ExponentialBackoff {
    ExponentialBackoff::builder().build_with_max_retries(3)
}
