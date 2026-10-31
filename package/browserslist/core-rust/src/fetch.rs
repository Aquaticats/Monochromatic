//! What:
//!  One blocking HTTP GET over the approved stack (hyper,
//! hyper-util,
//! tokio,
//! http-body-util,
//! hyper-rustls).
//! Why:
//!  The cache fetch and the update command both download a few small files;
//! they share one bounded,
//! checked GET.
//!  The async runtime is hidden here so callers stay synchronous,
//! as TypeScript's `await fetch` would hide it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function getBytes(url: string, maxBytes: number): Promise<Uint8Array>;
//! ```

/// What:
///  Import `BrowserslistError, Outcome` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome } from './error';
/// ```
use crate::error::{BrowserslistError, Outcome};
/// What:
///  Import `BodyExt, Empty, Limited` from `http_body_util`.
/// Why:
///  The response body is collected with a size limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BodyExt, Empty, Limited } from 'http_body_util';
/// ```
use http_body_util::{BodyExt, Empty, Limited};
/// What:
///  Import `Request` from `hyper::Request`.
/// Why:
///  The HTTP request and URI types come from hyper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Request } from 'hyper';
/// ```
use hyper::Request;
/// What:
///  Import `Uri` from `hyper::Uri`.
/// Why:
///  The HTTP request and URI types come from hyper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Uri } from 'hyper';
/// ```
use hyper::Uri;
/// What:
///  Import `Bytes` from `hyper::body::Bytes`.
/// Why:
///  The HTTP request and URI types come from hyper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Bytes } from 'hyper/body';
/// ```
use hyper::body::Bytes;
/// What:
///  Import `HttpsConnectorBuilder` from `hyper_rustls::HttpsConnectorBuilder`.
/// Why:
///  The connector adds TLS to the HTTP client.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { HttpsConnectorBuilder } from 'hyper_rustls';
/// ```
use hyper_rustls::HttpsConnectorBuilder;
/// What:
///  Import `Client` from `hyper_util::client::legacy::Client`.
/// Why:
///  The legacy client wraps hyper's connection handling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Client } from 'hyper_util/client/legacy';
/// ```
use hyper_util::client::legacy::Client;
/// What:
///  Import `TokioExecutor` from `hyper_util::rt::TokioExecutor`.
/// Why:
///  The legacy client wraps hyper's connection handling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TokioExecutor } from 'hyper_util/rt';
/// ```
use hyper_util::rt::TokioExecutor;
/// What:
///  Import `Duration` from `std::time::Duration`.
/// Why:
///  Waits and timeouts are bounded durations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Duration } from 'std/time';
/// ```
use std::time::Duration;

/// How long one whole request (connect,
/// headers and body) may take.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Hosts for which plain `http` is allowed:
/// this machine only,
/// so tests can use a fake registry.
const LOOPBACK_HOSTS: [&str; 3] = ["127.0.0.1", "localhost", "[::1]"];

/// Build a network error for a URL.
fn network_error(url: &str, message: &str) -> BrowserslistError {
    return BrowserslistError::Network { url: url.to_string(), message: message.to_string() };
}

/// Accept `https` URLs,
/// and `http` URLs only for loopback hosts.
fn check_url(url: &str) -> Outcome<Uri> {
    let uri: Uri = url.parse().map_err(|error| return network_error(url, &format!("not a valid URL: {error}")))?;
    let host = uri.host().unwrap_or("");
    match uri.scheme_str() {
        Some("https") => return Ok(uri),
        Some("http") if LOOPBACK_HOSTS.contains(&host) => return Ok(uri),
        _ => return Err(network_error(url, "only https URLs (or http to a loopback host) are fetched")),
    }
}

/// What:
///  An error with every cause it carries,
/// joined by `: `.
/// Why:
///  hyper's client error prints only `client error (Connect)`;
/// the cause (name resolution, refused connection, TLS alert) is what a person needs to act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeChain(error: Error): string
/// ```
fn describe_chain(error: &(dyn std::error::Error + 'static)) -> String {
    let mut text = error.to_string();
    let mut cause = error.source();
    while let Some(inner) = cause {
        text.push_str(": ");
        text.push_str(&inner.to_string());
        cause = inner.source();
    }
    return text;
}

/// The asynchronous request:
/// one GET,
/// no redirects,
/// a status of exactly 200,
/// and a body of at most `max_bytes`.
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
    let response = client.request(request).await.map_err(|error| return describe_chain(&error))?;
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
///  GET `url` and return its body,
/// failing on anything but a 200 answer within `max_bytes`.
/// Why:
///  The single door for every network read in the crate.
/// Gotcha:
///  Redirects are not followed:
/// the registry serves tarballs directly,
/// and a redirect would widen what is trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function getBytes(url: string, maxBytes: number): Promise<Uint8Array>; // throws on non-200, redirects, size
/// ```
pub fn get_bytes(url: &str, max_bytes: usize) -> Outcome<Vec<u8>> {
    let uri = check_url(url)?;
    tracing::debug!(url, max_bytes, "GET");
    let outcome = std::thread::scope(|scope| {
        return scope.spawn(|| return run_blocking(uri, max_bytes)).join();
    });
    return match outcome {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(message)) => Err(network_error(url, &message)),
        Err(_) => Err(network_error(url, "the request thread panicked")),
    };
}

/// What:
///  The unit tests of `fetch.rs`,
/// kept in the sidecar file `fetch_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './fetch.test';
/// ```
#[cfg(test)]
#[path = "fetch_tests.rs"]
mod tests;
