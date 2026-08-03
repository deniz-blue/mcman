use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

pub fn create_base_http_client() -> ClientWithMiddleware {
    let inner_client = reqwest::Client::builder()
        .user_agent(format!(
            "mcman/{} (https://github.com/deniz-blue/mcman)",
            env!("CARGO_PKG_VERSION")
        ))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to build base reqwest client");

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    ClientBuilder::new(inner_client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}
