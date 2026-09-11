use reqwest_middleware::ClientWithMiddleware;

use crate::{
    core::http::{create_base_http_client, create_cached_http_client},
    store::Store,
};

use std::sync::Arc;

pub mod checksum;
pub mod http;
pub mod kdl;

#[derive(Clone)]
pub struct AppContext {
    pub store: Arc<Store>,
    pub http: ClientWithMiddleware,
    pub cached_http: ClientWithMiddleware,
}

impl AppContext {
    pub fn new(store: Arc<Store>) -> Self {
        let cached_http = create_cached_http_client(store.http_cache_path());

        Self {
            store,
            http: create_base_http_client(),
            cached_http,
        }
    }
}
