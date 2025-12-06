#[cfg(feature = "tracing")]
use crate::middleware::tracing_middleware;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

#[derive(Debug, Clone)]
pub enum CompressionType {
    Brotli,
    Gzip,
    Deflate,
    Zstd,
}

#[derive(Debug, Clone)]
pub struct HttpClientBuilderConfig {
    pub timeout: Option<std::time::Duration>,
    pub connect_timeout: Option<std::time::Duration>,
    pub max_idle_per_host: Option<usize>,
    pub default_headers: Option<reqwest::header::HeaderMap>,
    pub compressions: Option<Vec<CompressionType>>,
    pub retry_enabled: bool,
    pub max_retries: u32,
}

impl Default for HttpClientBuilderConfig {
    fn default() -> Self {
        Self {
            timeout: Some(std::time::Duration::from_secs(10)),
            connect_timeout: Some(std::time::Duration::from_secs(5)),
            max_idle_per_host: Some(8),
            default_headers: Some({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::ACCEPT,
                    reqwest::header::HeaderValue::from_static("application/json"),
                );
                headers
            }),
            compressions: Some(vec![CompressionType::Gzip]),
            retry_enabled: true,
            max_retries: 3,
        }
    }
}
pub struct HttpClientBuilder {
    inner: ClientBuilder,
}

impl HttpClientBuilder {
    pub fn new(config: Option<HttpClientBuilderConfig>) -> Self {
        let mut merged = HttpClientBuilderConfig::default();

        if let Some(custom) = config {
            merged.timeout = custom.timeout;
            merged.connect_timeout = custom.connect_timeout;
            merged.max_idle_per_host = custom.max_idle_per_host;
            merged.default_headers = custom.default_headers;
            merged.compressions = custom.compressions;
            merged.retry_enabled = custom.retry_enabled;
            merged.max_retries = custom.max_retries;
        }

        let mut base = Client::builder();

        if let Some(timeout) = merged.timeout {
            base = base.timeout(timeout);
        }

        if let Some(default_headers) = merged.default_headers {
            base = base.default_headers(default_headers);
        }

        if let Some(max_idle) = merged.max_idle_per_host {
            base = base.pool_max_idle_per_host(max_idle);
        }

        if let Some(connect_timeout) = merged.connect_timeout {
            base = base.connect_timeout(connect_timeout);
        }

        if let Some(compressions) = merged.compressions {
            for compression in compressions {
                match compression {
                    CompressionType::Brotli => {
                        base = base.brotli(true);
                    }
                    CompressionType::Gzip => {
                        base = base.gzip(true);
                    }
                    CompressionType::Deflate => {
                        base = base.deflate(true);
                    }
                    CompressionType::Zstd => {
                        base = base.zstd(true);
                    }
                }
            }
        }

        let client = base.build().expect("Failed to create base reqwest client");
        let mut builder = ClientBuilder::new(client);

        // Add retry middleware if enabled
        if merged.retry_enabled {
            builder = builder.with(crate::middleware::retry::retry_middleware(
                merged.max_retries,
            ));
        }

        Self { inner: builder }
    }

    #[cfg(feature = "tracing")]
    pub fn with_tracing(mut self) -> Self {
        self.inner = self.inner.with(tracing_middleware());
        self
    }

    // custom middleware
    pub fn with_middleware<M>(mut self, middleware: M) -> Self
    where
        M: reqwest_middleware::Middleware + Send + Sync + 'static,
    {
        self.inner = self.inner.with(middleware);
        self
    }

    /// build the final reqwest client with middleware
    pub fn build(self) -> ClientWithMiddleware {
        self.inner.build()
    }

    pub fn inner(&self) -> &ClientBuilder {
        &self.inner
    }
}
