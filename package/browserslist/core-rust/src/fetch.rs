//! What:
//!  One blocking HTTP GET over the approved stack (hyper, hyper-util, tokio, http-body-util, hyper-rustls).
//! Why:
//!  The cache fetch and the update command both download a few small files; they share one bounded, checked GET.
//!  The async runtime is hidden here so callers stay synchronous, as TypeScript's `await fetch` would hide it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function getBytes(url: string, maxBytes: number): Promise<Uint8Array>;
//! ```

use crate::error::{BrowserslistError, Outcome};
use http_body_util::{BodyExt, Empty, Limited};
use hyper::Request;
use hyper::Uri;
use hyper::body::Bytes;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::time::Duration;

/// How long one whole request (connect, headers and body) may take.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Hosts for which plain `http` is allowed: this machine only, so tests can use a fake registry.
const LOOPBACK_HOSTS: [&str; 3] = ["127.0.0.1", "localhost", "[::1]"];

/// Build a network error for a URL.
fn network_error(url: &str, message: &str) -> BrowserslistError {
    return BrowserslistError::Network { url: url.to_string(), message: message.to_string() };
}

/// Accept `https` URLs, and `http` URLs only for loopback hosts.
fn check_url(url: &str) -> Outcome<Uri> {
    let uri: Uri = url.parse().map_err(|error| network_error(url, &format!("not a valid URL: {error}")))?;
    let host = uri.host().unwrap_or("");
    match uri.scheme_str() {
        Some("https") => return Ok(uri),
        Some("http") if LOOPBACK_HOSTS.contains(&host) => return Ok(uri),
        _ => return Err(network_error(url, "only https URLs (or http to a loopback host) are fetched")),
    }
}

/// The asynchronous request: one GET, no redirects, a status of exactly 200, and a body of at most `max_bytes`.
async fn get_once(uri: Uri, max_bytes: usize) -> Result<Vec<u8>, String> {
    let connector = HttpsConnectorBuilder::new()
        .with_native_roots()
        .map_err(|error| format!("cannot load the system root certificates: {error}"))?
        .https_or_http()
        .enable_http1()
        .build();
    let client: Client<_, Empty<Bytes>> = Client::builder(TokioExecutor::new()).build(connector);
    let request = Request::get(uri)
        .header("user-agent", "monochromatic-browserslist")
        .header("accept-encoding", "identity")
        .body(Empty::<Bytes>::new())
        .map_err(|error| format!("cannot build the request: {error}"))?;
    let response = client.request(request).await.map_err(|error| format!("{error}"))?;
    if response.status().as_u16() != 200 {
        return Err(format!("the server answered HTTP {}, not 200", response.status().as_u16()));
    }
    let body = Limited::new(response.into_body(), max_bytes)
        .collect()
        .await
        .map_err(|error| format!("cannot read the body within {max_bytes} bytes: {error}"))?;
    return Ok(body.to_bytes().to_vec());
}

/// Run the request on a fresh single-threaded runtime inside its own thread,
/// so the caller works whether or not it is already inside an async runtime.
fn run_blocking(uri: Uri, max_bytes: usize) -> Result<Vec<u8>, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("cannot start the async runtime: {error}"))?;
    return runtime.block_on(async {
        return match tokio::time::timeout(REQUEST_TIMEOUT, get_once(uri, max_bytes)).await {
            Ok(result) => result,
            Err(_) => Err(format!("no complete answer within {} seconds", REQUEST_TIMEOUT.as_secs())),
        };
    });
}

/// What:
///  GET `url` and return its body, failing on anything but a 200 answer within `max_bytes`.
/// Why:
///  The single door for every network read in the crate.
/// Gotcha:
///  Redirects are not followed: the registry serves tarballs directly, and a redirect would widen what is trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function getBytes(url: string, maxBytes: number): Promise<Uint8Array>; // throws on non-200, redirects, size
/// ```
pub fn get_bytes(url: &str, max_bytes: usize) -> Outcome<Vec<u8>> {
    let uri = check_url(url)?;
    tracing::debug!(url, max_bytes, "GET");
    let outcome = std::thread::scope(|scope| {
        return scope.spawn(|| run_blocking(uri, max_bytes)).join();
    });
    return match outcome {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(message)) => Err(network_error(url, &message)),
        Err(_) => Err(network_error(url, "the request thread panicked")),
    };
}

#[cfg(test)]
#[path = "fetch_tests.rs"]
mod tests;
