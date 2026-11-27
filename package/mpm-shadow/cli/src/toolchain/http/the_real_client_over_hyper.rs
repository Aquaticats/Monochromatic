//! What:
//! The real HTTP client and the programs that reach it:
//! `TlsClient` is the `hyper` type this module builds (an HTTPS connector over TCP with empty request bodies),
//! `HyperClient` owns a current-thread `tokio` runtime and that client,
//! and its `HttpGet` implementation runs the bounded GET and the download on that runtime and
//!           turns a timeout into an error that names the limit.
//! Why:
//! The rest of the crate is synchronous,
//! so one owner of the runtime and one builder of it keep every caller free of async:
//! a client built twice would start two runtimes,
//! and one built without the system's root certificates must fail with a message that says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // http/the_real_client_over_hyper.ts
//! export class HyperClient implements HttpGet { constructor() { ... } get(url, headers, maxBytes) { ... } }
//! ```

/// Imports `Empty` from `http_body_util`,
/// the request body type the client is built with.
use http_body_util::Empty;
/// Imports `Bytes` from `hyper::body`.
use hyper::body::Bytes;
/// Imports what this module uses from `hyper_rustls`.
use hyper_rustls::{
    HttpsConnector,
    HttpsConnectorBuilder,
};
/// Imports `Client` from `hyper_util::client::legacy`.
use hyper_util::client::legacy::Client;
/// Imports `HttpConnector` from `hyper_util::client::legacy::connect`.
use hyper_util::client::legacy::connect::HttpConnector;
/// Imports `TokioExecutor` from `hyper_util::rt`.
use hyper_util::rt::TokioExecutor;
/// Imports `Path` from `std::path`.
use std::path::Path;
/// Imports `Duration` from `std::time`.
use std::time::Duration;

/// Imports the https-only rule from the sibling module that builds one request.
use super::one_request_and_its_redirects::require_https;
/// Imports the GET timeout, the response, the error and the trait from `super`.
use super::{
    GET_TIMEOUT_SECONDS,
    HttpError,
    HttpGet,
    HttpResponse,
};

/// The `hyper` client type this module builds:
/// HTTPS connector over TCP,
/// empty request bodies.
type TlsClient = Client<HttpsConnector<HttpConnector>, Empty<Bytes>>;

/// What:
/// The real HTTP client.
/// Why:
/// It owns a current-thread `tokio` runtime and a TLS client (native root certificates,
/// HTTPS only,
/// HTTP/1.1)
///           so the callers stay synchronous.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HyperClient implements HttpGet { ... }
/// ```
pub struct HyperClient {
    /// The current-thread runtime every call blocks on.
    runtime: tokio::runtime::Runtime,
    /// The TLS client.
    pub(super) client: TlsClient,
    /// The User-Agent header value sent with every request.
    pub(super) user_agent: String,
}

/// The methods of `HyperClient`.
impl HyperClient {
    /// Builds the runtime and the TLS client;
    /// the error says why the system's root certificates or the runtime failed.
    pub fn new() -> Result<HyperClient, String> {
        let runtime: tokio::runtime::Runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(built) => built,
            Err(error) => return Err(format!("cannot start the network runtime: {error}")),
        };
        let builder = match HttpsConnectorBuilder::new().with_native_roots() {
            Ok(roots) => roots,
            Err(error) => {
                return Err(format!(
                    "cannot load the system's root certificates: {error}"
                ));
            }
        };
        let connector: HttpsConnector<HttpConnector> = builder.https_only().enable_http1().build();
        // No connection is kept for reuse:
        // the runtime above runs only while a call blocks on it,
        // so a pooled connection that the server closes during a long install step (cargo binstall, rustup, uv)
        // is never noticed,
        // and the next request over it fails with `client error (SendRequest)`.
        let client: TlsClient = Client::builder(TokioExecutor::new())
            .pool_max_idle_per_host(0)
            .build(connector);
        return Ok(HyperClient {
            runtime,
            client,
            user_agent: format!("monochromatic-mpm-shadow/{}", env!("CARGO_PKG_VERSION")),
        });
    }
}

/// Implements `HttpGet` for `HyperClient`.
impl HttpGet for HyperClient {
    /// Runs the bounded GET on the client's runtime.
    fn get(
        &self,
        url: &str,
        headers: &[(String, String)],
        max_bytes: usize,
    ) -> Result<HttpResponse, HttpError> {
        require_https(url)?;
        // The timer must be created inside the runtime, so the whole thing is one async block.
        let bounded = async {
            return tokio::time::timeout(
                Duration::from_secs(GET_TIMEOUT_SECONDS),
                self.get_async(url, headers, max_bytes),
            )
            .await;
        };
        return match self.runtime.block_on(bounded) {
            Ok(result) => result,
            Err(_elapsed) => Err(HttpError {
                url: url.to_string(),
                message: format!("not finished within {GET_TIMEOUT_SECONDS} seconds"),
            }),
        };
    }

    /// Runs the download on the client's runtime.
    fn download(
        &self,
        url: &str,
        target: &Path,
        max_bytes: u64,
    ) -> Result<String, HttpError> {
        require_https(url)?;
        return self
            .runtime
            .block_on(self.download_async(url, target, max_bytes));
    }
}
