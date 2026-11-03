//! What:
//! The one HTTP GET the registry lookups need:
//! a URL in,
//! the body text out,
//! with the status and `Retry-After` kept on
//! failure.
//! Why:
//! crates.io asks API clients to identify themselves and to back off on HTTP 429;
//! the registry layer needs the status number
//! to decide,
//! and tests replace the network with a fake fetcher.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // http.ts
//! export type FetchText = (url: string) => Promise<string>;
//! export class RegistryError extends Error { status?: number; retryAfterSeconds: number }
//! ```

use std::fmt;
use std::io::Read;
use std::time::Duration;

/// The user agent crates.io's data-access policy asks for:
/// it names the tool and where to reach its owner.
pub const USER_AGENT: &str = "monochromatic-cargo-shadow (https://github.com/Aquaticats/Monochromatic)";

/// The longest response body read,
/// in bytes.
/// A crate's whole version list is the largest response (a few megabytes).
const MAX_BODY_BYTES: u64 = 128 * 1024 * 1024;

/// How long one request may take before it counts as failed.
const REQUEST_TIMEOUT_SECONDS: u64 = 120;

/// A registry lookup that failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryError {
    /// What failed,
    /// with the URL.
    pub message: String,
    /// The HTTP status,
    /// when the server answered.
    pub status: Option<u16>,
    /// The `Retry-After` header in seconds;
    /// zero when absent.
    pub retry_after_seconds: u64,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.message);
    }
}

/// What:
/// `pub trait TextFetcher { fn get(&self, url: &str) -> Result<String, RegistryError>; }` is the seam the registry layer
/// reads the network through.
/// Why:
/// The registry tests give it canned bodies and statuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TextFetcher { get(url: string): string }  // throws RegistryError
/// ```
pub trait TextFetcher {
    /// Fetches the body of `url` as text.
    fn get(&self, url: &str) -> Result<String, RegistryError>;
}

/// What:
/// `pub trait DumpSource { fn open(&self, url: &str) -> Result<Box<dyn Read>, RegistryError>; }` opens a large download as a
/// byte stream.
/// Why:
/// The database dump is gigabytes and is read once,
/// as it arrives;
/// tests give it an archive built in memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface DumpSource { open(url: string): AsyncIterable<Uint8Array> }
/// ```
pub trait DumpSource {
    /// Opens `url` for streaming.
    fn open(&self, url: &str) -> Result<Box<dyn Read>, RegistryError>;
}

/// The real network,
/// through `ureq`.
pub struct UreqFetcher {
    agent: ureq::Agent,
    stream_agent: ureq::Agent,
}

impl UreqFetcher {
    /// Builds an agent that sends the user agent and does not turn HTTP statuses into errors (the status is data here).
    pub fn new() -> UreqFetcher {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .user_agent(USER_AGENT)
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(REQUEST_TIMEOUT_SECONDS)))
            .build()
            .into();
        // The dump download has no overall deadline: it is gigabytes long, and the caller stops reading it early.
        let stream_agent: ureq::Agent = ureq::Agent::config_builder()
            .user_agent(USER_AGENT)
            .http_status_as_error(false)
            .timeout_global(None)
            .build()
            .into();
        return UreqFetcher { agent, stream_agent };
    }
}

impl Default for UreqFetcher {
    fn default() -> UreqFetcher {
        return UreqFetcher::new();
    }
}

impl TextFetcher for UreqFetcher {
    fn get(&self, url: &str) -> Result<String, RegistryError> {
        let mut response = self.agent.get(url).call().map_err(|error| {
            return RegistryError { message: format!("{url}: {error}"), status: None, retry_after_seconds: 0 };
        })?;
        let status: u16 = response.status().as_u16();
        if !(200..300).contains(&status) {
            let retry_after: u64 = response
                .headers()
                .get("retry-after")
                .and_then(|value| { return value.to_str().ok(); })
                .and_then(|text| { return text.trim().parse::<u64>().ok(); })
                .unwrap_or(0);
            return Err(RegistryError {
                message: format!("{url} answered HTTP {status}"),
                status: Some(status),
                retry_after_seconds: retry_after,
            });
        }
        return response.body_mut().with_config().limit(MAX_BODY_BYTES).read_to_string().map_err(|error| {
            return RegistryError { message: format!("{url}: {error}"), status: None, retry_after_seconds: 0 };
        });
    }
}

impl DumpSource for UreqFetcher {
    fn open(&self, url: &str) -> Result<Box<dyn Read>, RegistryError> {
        let response = self.stream_agent.get(url).call().map_err(|error| {
            return RegistryError { message: format!("{url}: {error}"), status: None, retry_after_seconds: 0 };
        })?;
        let status: u16 = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(RegistryError { message: format!("{url} answered HTTP {status}"), status: Some(status), retry_after_seconds: 0 });
        }
        return Ok(Box::new(response.into_body().into_reader()));
    }
}
