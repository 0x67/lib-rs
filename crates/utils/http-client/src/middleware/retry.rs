use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

pub fn retry_middleware(max_retries: u32) -> RetryTransientMiddleware<ExponentialBackoff> {
    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(
            std::time::Duration::from_millis(100),
            std::time::Duration::from_secs(30),
        )
        .build_with_max_retries(max_retries);

    RetryTransientMiddleware::new_with_policy(retry_policy)
}

/// Creates a default retry policy with 3 retries and exponential backoff
pub fn default_retry_policy() -> RetryTransientMiddleware<ExponentialBackoff> {
    retry_middleware(3)
}
