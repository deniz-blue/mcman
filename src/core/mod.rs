use reqwest_middleware::ClientWithMiddleware;

use crate::store::Store;

use std::sync::Arc;

pub mod http;

#[derive(Clone)]
pub struct AppContext {
    pub store: Arc<Store>,
    pub http: ClientWithMiddleware,
}
