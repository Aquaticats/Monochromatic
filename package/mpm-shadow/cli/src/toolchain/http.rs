//! What:     The seam through which every command reaches the network: an `HttpGet` trait with two methods (a
//!           size-bounded GET for registries and manifests, and a streamed download to a file that returns the file's
//!           SHA-256), and the real implementation on `hyper`, `hyper-util`, `http-body-util`, `hyper-rustls` and a
//!           current-thread `tokio` runtime.
//! Why:      Registry lookups go to `registry.npmjs.org` and `pypi.org` directly over HTTPS, never through `curl` or
//!           `mpm outdated`. The trait keeps the rest of the crate synchronous and lets tests use a local fake. Every
//!           failure names the URL, and every wait is bounded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // http.ts
//! export interface HttpGet {
//!   get(url: string, headers: [string, string][], maxBytes: number): HttpResponse;
//!   download(url: string, target: string, maxBytes: number): string; // returns the SHA-256 of the file
//! }
//! ```

use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper::body::Bytes;
use hyper::header::{HeaderName, HeaderValue, LOCATION};
use hyper::http::Uri;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use super::hash::hex_of_digest;

/// The most redirects one request follows.
pub const MAX_REDIRECTS: usize = 5;

/// How long connecting and receiving the response headers may take, in seconds.
pub const CONNECT_AND_HEADERS_TIMEOUT_SECONDS: u64 = 30;

/// How long a bounded GET may take in all, in seconds.
pub const GET_TIMEOUT_SECONDS: u64 = 120;

/// How long a download may wait for its next chunk, in seconds.
pub const DOWNLOAD_IDLE_TIMEOUT_SECONDS: u64 = 60;

/// How long a whole download may take, in seconds (an archive of a few hundred megabytes on a slow link).
pub const DOWNLOAD_TOTAL_TIMEOUT_SECONDS: u64 = 1800;

/// The largest response a registry or manifest GET accepts, in bytes.
pub const DEFAULT_MAX_GET_BYTES: usize = 32 * 1024 * 1024;

/// The largest file a download accepts, in bytes.
pub const DEFAULT_MAX_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// A response: the status code and the (bounded) body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    /// The HTTP status code of the final response, after redirects.
    pub status: u16,
    /// The body bytes.
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// The body as text (invalid UTF-8 replaced).
    pub fn text(&self) -> String {
        return String::from_utf8_lossy(&self.body).to_string();
    }
}

/// What:     A network failure: the URL asked for and what went wrong.
/// Why:      The rule for this module is that every failure names the URL, so the URL is a field, not an afterthought.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HttpError extends Error { constructor(readonly url: string, message: string) { ... } }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpError {
    /// The URL requested.
    pub url: String,
    /// What went wrong, in operation terms.
    pub message: String,
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}: {}", self.url, self.message);
    }
}

impl std::error::Error for HttpError {}

/// What:     The seam: GET a bounded body, or download to a file and hash it.
/// Why:      The real client is `HyperClient`; tests use a fake that answers from a table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface HttpGet { get(...): HttpResponse; download(...): string }
/// ```
pub trait HttpGet {
    /// GETs `url` (HTTPS only, redirects followed), refusing bodies over `max_bytes`; `headers` are added to the
    /// request. A non-2xx status is returned, not an error, so callers can report the status with their own context.
    fn get(&self, url: &str, headers: &[(String, String)], max_bytes: usize) -> Result<HttpResponse, HttpError>;

    /// Streams `url` (HTTPS only, redirects followed) into the file `target`, refusing more than `max_bytes`, and
    /// returns the SHA-256 of the written bytes as lowercase hex. A non-2xx status is an error.
    fn download(&self, url: &str, target: &Path, max_bytes: u64) -> Result<String, HttpError>;
}

/// What:     Resolves a redirect `Location` value against the URL that returned it.
/// Why:      A location is absolute (`https://...`), absolute-path (`/x`) or scheme-relative (`//host/x`); a relative
///           path without a leading slash is not used by the registries this crate talks to and is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveLocation(base: string, location: string): string  // throws
/// ```
pub fn resolve_location(base: &str, location: &str) -> Result<String, String> {
    if location.starts_with("https://") {
        return Ok(location.to_string());
    }
    let uri: Uri = match base.parse::<Uri>() {
        Ok(parsed) => parsed,
        Err(error) => return Err(format!("the request URL is not a valid URL: {error}")),
    };
    let Some(authority) = uri.authority() else {
        return Err("the request URL has no host".to_string());
    };
    if let Some(rest) = location.strip_prefix("//") {
        return Ok(format!("https://{rest}"));
    }
    if location.starts_with('/') {
        return Ok(format!("https://{authority}{location}"));
    }
    return Err(format!("the redirect target {location:?} is neither an https URL nor an absolute path"));
}

/// Fails unless `url` is an `https://` URL.
fn require_https(url: &str) -> Result<(), HttpError> {
    if url.starts_with("https://") {
        return Ok(());
    }
    return Err(HttpError {
        url: url.to_string(),
        message: "only https URLs are fetched; this URL is refused".to_string(),
    });
}

/// The `hyper` client type this module builds: HTTPS connector over TCP, empty request bodies.
type TlsClient = Client<HttpsConnector<HttpConnector>, Empty<Bytes>>;

/// What:     The real HTTP client.
/// Why:      It owns a current-thread `tokio` runtime and a TLS client (native root certificates, HTTPS only, HTTP/1.1)
///           so the callers stay synchronous.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HyperClient implements HttpGet { ... }
/// ```
pub struct HyperClient {
    runtime: tokio::runtime::Runtime,
    client: TlsClient,
    user_agent: String,
}

impl HyperClient {
    /// Builds the runtime and the TLS client; the error says why the system's root certificates or the runtime failed.
    pub fn new() -> Result<HyperClient, String> {
        let runtime: tokio::runtime::Runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(built) => built,
            Err(error) => return Err(format!("cannot start the network runtime: {error}")),
        };
        let builder = match HttpsConnectorBuilder::new().with_native_roots() {
            Ok(roots) => roots,
            Err(error) => return Err(format!("cannot load the system's root certificates: {error}")),
        };
        let connector: HttpsConnector<HttpConnector> = builder.https_only().enable_http1().build();
        let client: TlsClient = Client::builder(TokioExecutor::new()).build(connector);
        return Ok(HyperClient {
            runtime,
            client,
            user_agent: format!("monochromatic-mpm-shadow/{}", env!("CARGO_PKG_VERSION")),
        });
    }

    /// Sends one request, following up to `MAX_REDIRECTS` redirects, and returns the final response's parts.
    async fn send(&self, url: &str, headers: &[(String, String)]) -> Result<hyper::Response<hyper::body::Incoming>, HttpError> {
        let mut current: String = url.to_string();
        for _hop in 0..=MAX_REDIRECTS {
            require_https(&current)?;
            let uri: Uri = match current.parse::<Uri>() {
                Ok(parsed) => parsed,
                Err(error) => {
                    return Err(HttpError { url: url.to_string(), message: format!("not a valid URL: {error}") });
                }
            };
            let mut builder = Request::builder().method("GET").uri(uri).header("user-agent", self.user_agent.as_str());
            // Credentials only go to the host they were meant for; a redirect to another host drops them.
            let same_host: bool = current == url || host_of(&current) == host_of(url);
            for (name, value) in headers {
                if !same_host && name.eq_ignore_ascii_case("authorization") {
                    continue;
                }
                let (Ok(header_name), Ok(header_value)) = (HeaderName::from_bytes(name.as_bytes()), HeaderValue::from_str(value)) else {
                    return Err(HttpError { url: url.to_string(), message: format!("the header {name:?} is not valid") });
                };
                builder = builder.header(header_name, header_value);
            }
            let request = match builder.body(Empty::<Bytes>::new()) {
                Ok(built) => built,
                Err(error) => return Err(HttpError { url: url.to_string(), message: format!("cannot build the request: {error}") }),
            };
            let pending = self.client.request(request);
            let response = match tokio::time::timeout(Duration::from_secs(CONNECT_AND_HEADERS_TIMEOUT_SECONDS), pending).await {
                Ok(Ok(answered)) => answered,
                Ok(Err(error)) => {
                    return Err(HttpError { url: current.clone(), message: format!("the request failed: {error}") });
                }
                Err(_elapsed) => {
                    return Err(HttpError {
                        url: current.clone(),
                        message: format!("no response within {CONNECT_AND_HEADERS_TIMEOUT_SECONDS} seconds"),
                    });
                }
            };
            if response.status().is_redirection() {
                let Some(location) = response.headers().get(LOCATION).and_then(|value| return value.to_str().ok()) else {
                    return Err(HttpError { url: current.clone(), message: "a redirect without a Location header".to_string() });
                };
                current = match resolve_location(&current, location) {
                    Ok(next) => next,
                    Err(message) => return Err(HttpError { url: current.clone(), message }),
                };
                continue;
            }
            return Ok(response);
        }
        return Err(HttpError { url: url.to_string(), message: format!("more than {MAX_REDIRECTS} redirects") });
    }

    /// The bounded GET, inside the runtime.
    async fn get_async(&self, url: &str, headers: &[(String, String)], max_bytes: usize) -> Result<HttpResponse, HttpError> {
        let response = self.send(url, headers).await?;
        let status: u16 = response.status().as_u16();
        let mut body = response.into_body();
        let mut collected: Vec<u8> = Vec::new();
        loop {
            let next = match tokio::time::timeout(Duration::from_secs(DOWNLOAD_IDLE_TIMEOUT_SECONDS), body.frame()).await {
                Ok(frame) => frame,
                Err(_elapsed) => {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("the body stalled for {DOWNLOAD_IDLE_TIMEOUT_SECONDS} seconds"),
                    });
                }
            };
            let Some(received_frame) = next else {
                break;
            };
            let frame = match received_frame {
                Ok(received) => received,
                Err(error) => return Err(HttpError { url: url.to_string(), message: format!("reading the body failed: {error}") }),
            };
            if let Some(data) = frame.data_ref() {
                if collected.len() + data.len() > max_bytes {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("the response is larger than the {max_bytes} bytes this lookup accepts"),
                    });
                }
                collected.extend_from_slice(data);
            }
        }
        return Ok(HttpResponse { status, body: collected });
    }

    /// The download, inside the runtime.
    async fn download_async(&self, url: &str, target: &Path, max_bytes: u64) -> Result<String, HttpError> {
        let response = self.send(url, &[]).await?;
        let status: u16 = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(HttpError { url: url.to_string(), message: format!("HTTP {status}") });
        }
        if let Some(parent) = target.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            return Err(HttpError { url: url.to_string(), message: format!("cannot create {}: {error}", parent.display()) });
        }
        let mut file: File = match File::create(target) {
            Ok(created) => created,
            Err(error) => {
                return Err(HttpError { url: url.to_string(), message: format!("cannot create {}: {error}", target.display()) });
            }
        };
        let mut hasher: Sha256 = Sha256::new();
        let mut written: u64 = 0;
        let mut body = response.into_body();
        let started = std::time::Instant::now();
        loop {
            if started.elapsed() > Duration::from_secs(DOWNLOAD_TOTAL_TIMEOUT_SECONDS) {
                return Err(HttpError {
                    url: url.to_string(),
                    message: format!("the download took longer than {DOWNLOAD_TOTAL_TIMEOUT_SECONDS} seconds"),
                });
            }
            let next = match tokio::time::timeout(Duration::from_secs(DOWNLOAD_IDLE_TIMEOUT_SECONDS), body.frame()).await {
                Ok(frame) => frame,
                Err(_elapsed) => {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("the download stalled for {DOWNLOAD_IDLE_TIMEOUT_SECONDS} seconds"),
                    });
                }
            };
            let Some(received_frame) = next else {
                break;
            };
            let frame = match received_frame {
                Ok(received) => received,
                Err(error) => return Err(HttpError { url: url.to_string(), message: format!("reading the body failed: {error}") }),
            };
            if let Some(data) = frame.data_ref() {
                written += data.len() as u64;
                if written > max_bytes {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("the file is larger than the {max_bytes} bytes this download accepts"),
                    });
                }
                hasher.update(data);
                if let Err(error) = file.write_all(data) {
                    return Err(HttpError { url: url.to_string(), message: format!("cannot write {}: {error}", target.display()) });
                }
            }
        }
        return Ok(hex_of_digest(hasher.finalize().as_slice()));
    }
}

/// The host part of a URL (`registry.npmjs.org`), or the empty text.
fn host_of(url: &str) -> String {
    return match url.parse::<Uri>() {
        Ok(uri) => uri.host().unwrap_or("").to_string(),
        Err(_) => String::new(),
    };
}

impl HttpGet for HyperClient {
    fn get(&self, url: &str, headers: &[(String, String)], max_bytes: usize) -> Result<HttpResponse, HttpError> {
        require_https(url)?;
        // The timer must be created inside the runtime, so the whole thing is one async block.
        let bounded = async { return tokio::time::timeout(Duration::from_secs(GET_TIMEOUT_SECONDS), self.get_async(url, headers, max_bytes)).await };
        return match self.runtime.block_on(bounded) {
            Ok(result) => result,
            Err(_elapsed) => Err(HttpError { url: url.to_string(), message: format!("not finished within {GET_TIMEOUT_SECONDS} seconds") }),
        };
    }

    fn download(&self, url: &str, target: &Path, max_bytes: u64) -> Result<String, HttpError> {
        require_https(url)?;
        return self.runtime.block_on(self.download_async(url, target, max_bytes));
    }
}

/// What:     An `HttpGet` that fails every request with the reason the real client could not be built.
/// Why:      A machine without root certificates still runs commands that need no network; only a command that
///           reaches for the network fails, and the message says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unavailable = (reason: string): HttpGet => ({ get: (url) => { throw new HttpError(url, reason); }, ... });
/// ```
#[derive(Clone, Debug)]
pub struct UnavailableNetwork {
    /// Why the client could not be built.
    pub reason: String,
}

impl HttpGet for UnavailableNetwork {
    fn get(&self, url: &str, _headers: &[(String, String)], _max_bytes: usize) -> Result<HttpResponse, HttpError> {
        return Err(HttpError { url: url.to_string(), message: self.reason.clone() });
    }

    fn download(&self, url: &str, _target: &Path, _max_bytes: u64) -> Result<String, HttpError> {
        return Err(HttpError { url: url.to_string(), message: self.reason.clone() });
    }
}

/// The real client, or an `UnavailableNetwork` carrying the reason it could not be built.
pub fn connect_or_explain() -> Box<dyn HttpGet> {
    return match HyperClient::new() {
        Ok(client) => Box::new(client),
        Err(reason) => Box::new(UnavailableNetwork { reason }),
    };
}

/// What:     An `HttpGet` that refuses every request.
/// Why:      `status` and `bin-paths` never need the network; handing them this makes "they never touch it" a fact the
///           compiler and a test can check, and gives a clear diagnostic if code ever tries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noNetwork: HttpGet = { get: (url) => { throw new HttpError(url, '...'); }, download: ... };
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct NoNetwork;

impl HttpGet for NoNetwork {
    fn get(&self, url: &str, _headers: &[(String, String)], _max_bytes: usize) -> Result<HttpResponse, HttpError> {
        return Err(HttpError { url: url.to_string(), message: "this command does not use the network".to_string() });
    }

    fn download(&self, url: &str, _target: &Path, _max_bytes: u64) -> Result<String, HttpError> {
        return Err(HttpError { url: url.to_string(), message: "this command does not use the network".to_string() });
    }
}
