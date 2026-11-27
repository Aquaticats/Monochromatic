//! What:
//! One request and its redirects:
//! `resolve_location` turns a redirect's `Location` value into the next URL (https,
//! absolute-path or scheme-relative, everything else refused),
//! `require_https` refuses any URL that is not `https://`,
//! and `send` builds the request with its headers,
//! follows up to `MAX_REDIRECTS` redirects and returns the final response.
//! Why:
//! Every failure of this module must name the URL,
//! and a credential must never follow a redirect to another host,
//! so both rules sit where the request is built rather than at each caller.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // http/one_request_and_its_redirects.ts
//! export function resolveLocation(base: string, location: string): string;
//! async function send(url: string, headers: [string, string][]): Promise<Response>;
//! ```

/// Imports `Empty` from `http_body_util`,
/// the empty request body every GET sends.
use http_body_util::Empty;
/// Imports `Request` from `hyper`.
use hyper::Request;
/// Imports `Bytes` from `hyper::body`.
use hyper::body::Bytes;
/// Imports what this module uses from `hyper::header`.
use hyper::header::{
    HeaderName,
    HeaderValue,
    LOCATION,
};
/// Imports `Uri` from `hyper::http`.
use hyper::http::Uri;
/// Imports `Duration` from `std::time`.
use std::time::Duration;

/// Imports the header timeout, the error, the client and the redirect limit from `super`.
use super::{
    CONNECT_AND_HEADERS_TIMEOUT_SECONDS,
    HttpError,
    HyperClient,
    MAX_REDIRECTS,
};

/// What:
/// Resolves a redirect `Location` value against the URL that returned it.
/// Why:
/// A location is absolute (`https://...`),
/// absolute-path (`/x`) or scheme-relative (`//host/x`);
/// a relative
///           path without a leading slash is not used by the registries this crate talks to and is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveLocation(base: string, location: string): string  // throws
/// ```
pub fn resolve_location(
    base: &str,
    location: &str,
) -> Result<String, String> {
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
    return Err(format!(
        "the redirect target {location:?} is neither an https URL nor an absolute path"
    ));
}

/// Fails unless `url` is an `https://` URL.
pub(super) fn require_https(url: &str) -> Result<(), HttpError> {
    if url.starts_with("https://") {
        return Ok(());
    }
    return Err(HttpError {
        url: url.to_string(),
        message: "only https URLs are fetched; this URL is refused".to_string(),
    });
}

/// The methods of `HyperClient` that build one request and follow its redirects.
impl HyperClient {
    /// Sends one request,
    /// following up to `MAX_REDIRECTS` redirects,
    /// and returns the final response's parts.
    pub(super) async fn send(
        &self,
        url: &str,
        headers: &[(String, String)],
    ) -> Result<hyper::Response<hyper::body::Incoming>, HttpError> {
        let mut current: String = url.to_string();
        for _hop in 0..=MAX_REDIRECTS {
            require_https(&current)?;
            let uri: Uri = match current.parse::<Uri>() {
                Ok(parsed) => parsed,
                Err(error) => {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("not a valid URL: {error}"),
                    });
                }
            };
            let mut builder = Request::builder()
                .method("GET")
                .uri(uri)
                .header("user-agent", self.user_agent.as_str());
            // Credentials only go to the host they were meant for; a redirect to another host drops them.
            let same_host: bool = current == url || host_of(&current) == host_of(url);
            for (name, value) in headers {
                if !same_host && name.eq_ignore_ascii_case("authorization") {
                    continue;
                }
                let (Ok(header_name), Ok(header_value)) = (
                    HeaderName::from_bytes(name.as_bytes()),
                    HeaderValue::from_str(value),
                ) else {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("the header {name:?} is not valid"),
                    });
                };
                builder = builder.header(header_name, header_value);
            }
            let request = match builder.body(Empty::<Bytes>::new()) {
                Ok(built) => built,
                Err(error) => {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("cannot build the request: {error}"),
                    });
                }
            };
            let pending = self.client.request(request);
            let response = match tokio::time::timeout(
                Duration::from_secs(CONNECT_AND_HEADERS_TIMEOUT_SECONDS),
                pending,
            )
            .await
            {
                Ok(Ok(answered)) => answered,
                Ok(Err(error)) => {
                    return Err(HttpError {
                        url: current.clone(),
                        message: format!("the request failed: {error}"),
                    });
                }
                Err(_elapsed) => {
                    return Err(HttpError {
                        url: current.clone(),
                        message: format!(
                            "no response within {CONNECT_AND_HEADERS_TIMEOUT_SECONDS} seconds"
                        ),
                    });
                }
            };
            if response.status().is_redirection() {
                let Some(location) = response.headers().get(LOCATION).and_then(header_text) else {
                    return Err(HttpError {
                        url: current.clone(),
                        message: "a redirect without a Location header".to_string(),
                    });
                };
                current = match resolve_location(&current, location) {
                    Ok(next) => next,
                    Err(message) => {
                        return Err(HttpError {
                            url: current.clone(),
                            message,
                        });
                    }
                };
                continue;
            }
            return Ok(response);
        }
        return Err(HttpError {
            url: url.to_string(),
            message: format!("more than {MAX_REDIRECTS} redirects"),
        });
    }
}

/// A header value as text,
/// or `None` when it is not visible ASCII.
fn header_text(value: &HeaderValue) -> Option<&str> {
    return value.to_str().ok();
}

/// The host part of a URL (`registry.npmjs.org`),
/// or the empty text.
fn host_of(url: &str) -> String {
    return match url.parse::<Uri>() {
        Ok(uri) => uri.host().unwrap_or("").to_string(),
        Err(_) => String::new(),
    };
}
