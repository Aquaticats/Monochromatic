//! What:
//! The seam through which every command reaches the network:
//! an `HttpGet` trait with two methods (a
//!           size-bounded GET for registries and manifests,
//! and a streamed download to a file that returns the file's
//!           SHA-256),
//! the `HttpResponse` and `HttpError` types those methods answer with,
//! and the bounds every call respects.
//! The client that does the work lives in the files beside this one,
//! each named for what it holds:
//! the real client over `hyper` (`the_real_client_over_hyper`),
//! one request and its redirects (`one_request_and_its_redirects`),
//! the bounded GET and the download (`the_bounded_get_and_the_download`),
//! and the clients a run that must not reach the network gets
//! (`a_client_that_never_reaches_the_network`).
//! Why:
//! Registry lookups go to `registry.npmjs.org` and `pypi.org` directly over HTTPS,
//! never through `curl` or
//!           `mpm outdated`.
//! The trait keeps the rest of the crate synchronous and lets tests use a local fake.
//! Every
//!           failure names the URL,
//! and every wait is bounded.
//! This file keeps the seam and its vocabulary,
//! so a caller imports `super::http::HttpGet` and never one of the parts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // http.ts
//! export interface HttpGet {
//!   get(url: string, headers: [string, string][], maxBytes: number): HttpResponse;
//!   download(url: string, target: string, maxBytes: number): string; // returns the SHA-256 of the file
//! }
//! export * from './http/the_real_client_over_hyper.ts';
//! ```

/// Imports `fmt` from `std`.
use std::fmt;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// The clients a run that must not reach the network gets.
mod a_client_that_never_reaches_the_network;
/// The redirect rules and the one request that follows them.
mod one_request_and_its_redirects;
/// The two bounded reads of the real client:
/// the GET that collects a body,
/// and the download that hashes the file it writes.
mod the_bounded_get_and_the_download;
/// The client that owns the runtime and answers `HttpGet`.
mod the_real_client_over_hyper;

/// A client that fails every request,
/// for the commands that never need the network.
pub use self::a_client_that_never_reaches_the_network::NoNetwork;
/// A client that fails every request with the reason the real one could not be built.
pub use self::a_client_that_never_reaches_the_network::UnavailableNetwork;
/// The real client,
/// or a client that fails every request with the reason it could not be built.
pub use self::a_client_that_never_reaches_the_network::connect_or_explain;
/// The redirect rule the real client applies,
/// exported so its tests can state it alone.
pub use self::one_request_and_its_redirects::resolve_location;
/// The real client over `hyper`,
/// which every command that needs the network builds.
pub use self::the_real_client_over_hyper::HyperClient;

/// The most redirects one request follows.
pub const MAX_REDIRECTS: usize = 5;

/// How long connecting and receiving the response headers may take,
/// in seconds.
pub const CONNECT_AND_HEADERS_TIMEOUT_SECONDS: u64 = 30;

/// How long a bounded GET may take in all,
/// in seconds.
pub const GET_TIMEOUT_SECONDS: u64 = 120;

/// How long a download may wait for its next chunk,
/// in seconds.
pub const DOWNLOAD_IDLE_TIMEOUT_SECONDS: u64 = 60;

/// How long a whole download may take,
/// in seconds (an archive of a few hundred megabytes on a slow link).
pub const DOWNLOAD_TOTAL_TIMEOUT_SECONDS: u64 = 1800;

/// The largest response a registry or manifest GET accepts,
/// in bytes.
pub const DEFAULT_MAX_GET_BYTES: usize = 32 * 1024 * 1024;

/// The largest file a download accepts,
/// in bytes.
pub const DEFAULT_MAX_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// A response:
/// the status code and the (bounded) body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    /// The HTTP status code of the final response,
    /// after redirects.
    pub status: u16,
    /// The body bytes.
    pub body: Vec<u8>,
}

/// The methods of `HttpResponse`.
impl HttpResponse {
    /// The body as text (invalid UTF-8 replaced).
    pub fn text(&self) -> String {
        return String::from_utf8_lossy(&self.body).to_string();
    }
}

/// What:
/// A network failure:
/// the URL asked for and what went wrong.
/// Why:
/// The rule for this module is that every failure names the URL,
/// so the URL is a field,
/// not an afterthought.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HttpError extends Error { constructor(readonly url: string, message: string) { ... } }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpError {
    /// The URL requested.
    pub url: String,
    /// What went wrong,
    /// in operation terms.
    pub message: String,
}

/// Implements `fmt::Display` for `HttpError`.
impl fmt::Display for HttpError {
    /// Prints `url: message`.
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        return write!(formatter, "{}: {}", self.url, self.message);
    }
}

/// Implements `std::error::Error` for `HttpError`.
impl std::error::Error for HttpError {}

/// What:
/// The seam:
/// GET a bounded body,
/// or download to a file and hash it.
/// Why:
/// The real client is `HyperClient`;
/// tests use a fake that answers from a table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface HttpGet { get(...): HttpResponse; download(...): string }
/// ```
pub trait HttpGet {
    /// GETs `url` (HTTPS only,
    /// redirects followed),
    /// refusing bodies over `max_bytes`;
    /// `headers` are added to the
    /// request.
    /// A non-2xx status is returned,
    /// not an error,
    /// so callers can report the status with their own context.
    fn get(
        &self,
        url: &str,
        headers: &[(String, String)],
        max_bytes: usize,
    ) -> Result<HttpResponse, HttpError>;

    /// Streams `url` (HTTPS only,
    /// redirects followed) into the file `target`,
    /// refusing more than `max_bytes`,
    /// and
    /// returns the SHA-256 of the written bytes as lowercase hex.
    /// A non-2xx status is an error.
    fn download(
        &self,
        url: &str,
        target: &Path,
        max_bytes: u64,
    ) -> Result<String, HttpError>;
}
