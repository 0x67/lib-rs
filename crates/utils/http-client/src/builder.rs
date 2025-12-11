use std::{io::Read, path::Path, sync::Arc};

#[cfg(feature = "tracing")]
use crate::middleware::tracing_middleware;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rustls::{ClientConfig, RootCertStore};
use rustls_pki_types::CertificateDer;
use zeroize::Zeroize;

#[derive(Debug, Clone)]
pub enum CompressionType {
    Brotli,
    Gzip,
    Deflate,
    Zstd,
}

fn build_custom_ca_config_from_certs<I>(
    certs: I,
) -> Result<ClientConfig, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = Vec<u8>>,
{
    let mut root_store = RootCertStore::empty();

    for mut cert_der in certs {
        let cert = rustls::pki_types::CertificateDer::from(cert_der.clone());
        root_store.add(cert)?;
        cert_der.zeroize();
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(config)
}

#[derive(Debug, Clone)]
pub struct HttpClientBuilderConfig {
    pub timeout: Option<std::time::Duration>,
    pub connect_timeout: Option<std::time::Duration>,
    pub max_idle_per_host: Option<usize>,
    pub default_headers: Option<reqwest::header::HeaderMap>,
    pub compressions: Option<Vec<CompressionType>>,
    pub retry_enabled: Option<bool>,
    pub max_retries: Option<u32>,
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
            retry_enabled: Some(true),
            max_retries: Some(3),
        }
    }
}
pub struct HttpClientBuilder {
    base_config: HttpClientBuilderConfig,
    middleware: Vec<Arc<dyn reqwest_middleware::Middleware>>,
    tls_config: Option<ClientConfig>,
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

        let mut middleware = Vec::new();

        // Add retry middleware if enabled
        if matches!(merged.retry_enabled, Some(true)) {
            middleware.push(Arc::new(crate::middleware::retry::retry_middleware(
                merged.max_retries.unwrap_or(3),
            )) as Arc<dyn reqwest_middleware::Middleware>);
        }

        Self {
            base_config: merged,
            middleware,
            tls_config: None,
        }
    }

    #[cfg(feature = "tracing")]
    pub fn with_tracing(mut self) -> Self {
        self.middleware.push(Arc::new(tracing_middleware()));
        self
    }

    /// Add custom middleware
    pub fn with_middleware<M>(mut self, middleware: M) -> Self
    where
        M: reqwest_middleware::Middleware + Send + Sync + 'static,
    {
        self.middleware.push(Arc::new(middleware));
        self
    }

    pub fn with_pinned_certs<I>(mut self, certs: I) -> Result<Self, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = Vec<u8>>,
    {
        let tls_config = build_custom_ca_config_from_certs(certs)?;
        self.tls_config = Some(tls_config);
        Ok(self)
    }

    pub fn with_pinned_pem_files<P, I>(self, paths: I) -> Result<Self, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let mut all_certs: Vec<Vec<u8>> = Vec::new();

        for path in paths {
            let mut pem_data = Vec::new();
            std::fs::File::open(path.as_ref())?.read_to_end(&mut pem_data)?;

            let certs: Vec<CertificateDer> =
                rustls_pki_types::pem::PemObject::pem_slice_iter(&pem_data)
                    .collect::<Result<Vec<_>, _>>()?;

            all_certs.extend(certs.into_iter().map(|c| c.into_owned().to_vec()));
            pem_data.zeroize();
        }

        self.with_pinned_certs(all_certs)
    }

    /// Add pinned certificates from PEM data in memory
    /// NOTE: Caller is responsible for zeroizing the input data after this call
    pub fn with_pinned_pem_data<D, I>(self, pem_data: I) -> Result<Self, Box<dyn std::error::Error>>
    where
        D: AsRef<[u8]>,
        I: IntoIterator<Item = D>,
    {
        let mut all_certs: Vec<Vec<u8>> = Vec::new();

        for pem_bytes in pem_data {
            let certs: Vec<CertificateDer> =
                rustls_pki_types::pem::PemObject::pem_slice_iter(pem_bytes.as_ref())
                    .collect::<Result<Vec<_>, _>>()?;

            all_certs.extend(certs.into_iter().map(|c| c.into_owned().to_vec()));
        }

        self.with_pinned_certs(all_certs)
    }

    pub fn build(self) -> ClientWithMiddleware {
        let mut base = Client::builder();

        // Apply base configuration
        if let Some(timeout) = self.base_config.timeout {
            base = base.timeout(timeout);
        }

        if let Some(default_headers) = self.base_config.default_headers {
            base = base.default_headers(default_headers);
        }

        if let Some(max_idle) = self.base_config.max_idle_per_host {
            base = base.pool_max_idle_per_host(max_idle);
        }

        if let Some(connect_timeout) = self.base_config.connect_timeout {
            base = base.connect_timeout(connect_timeout);
        }

        if let Some(compressions) = self.base_config.compressions {
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

        // Apply TLS config if present
        if let Some(tls_config) = self.tls_config {
            base = base.use_preconfigured_tls(tls_config);
        }

        let client = base.build().unwrap_or_else(|_| {
            panic!("reqwest client builder failed - this should be unreachable in reqwest 0.12+")
        });

        // Build middleware chain
        let mut builder = ClientBuilder::new(client);
        for middleware in self.middleware {
            builder = builder.with_arc(middleware);
        }

        builder.build()
    }
}
