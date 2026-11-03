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

/// What:
/// `use std::fmt;` imports `fmt` from the standard library so this file can name it unqualified.
/// Why:
/// `fmt` is used by the `impl` of `RegistryError`,
/// `fmt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fmt } from 'std';
/// ```
use std::fmt;
/// What:
/// `use std::io::Read;` imports `Read` from the standard library so this file can name it unqualified.
/// Why:
/// `Read` is used by `open`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Read } from 'std/io';
/// ```
use std::io::Read;
/// What:
/// `use std::time::Duration;` imports `Duration` from the standard library so this file can name it unqualified.
/// Why:
/// `Duration` is used by `new`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Duration } from 'std/time';
/// ```
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

/// What:
/// `impl fmt::Display for RegistryError` lets the error be printed with `{}`.
/// Why:
/// Registry failures are shown to the user in the install and update reports,
/// and the URL and status are already in the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RegistryError extends Error {}
/// ```
impl fmt::Display for RegistryError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the error message into the formatter.
    /// Why:
    /// The message carries the URL and the HTTP status,
    /// so nothing else needs adding.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.message; }
    /// ```
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
    /// What:
    /// Field `agent: ureq::Agent` is the HTTP client used for the sparse index and the API (`ureq::Agent` keeps connections alive between requests).
    /// Why:
    /// Those requests are small and many,
    /// and they carry an overall time limit so a stuck server cannot hang an install.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// agent: HttpClient
    /// ```
    agent: ureq::Agent,
    /// What:
    /// Field `stream_agent: ureq::Agent` is a second HTTP client with no overall deadline.
    /// Why:
    /// The database dump is gigabytes long and is read as it arrives;
    /// an overall limit would cut it off.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// streamAgent: HttpClient  // no total timeout
    /// ```
    stream_agent: ureq::Agent,
}

/// What:
/// `impl UreqFetcher { ... }` attaches the constructor to the fetcher struct.
/// Why:
/// Callers build the real network client in one call and tests build their fake instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UreqFetcher { constructor() { ... } }
/// ```
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

/// What:
/// `impl Default for UreqFetcher` makes `UreqFetcher::default()` mean `UreqFetcher::new()`.
/// Why:
/// Clippy asks for it on a type with a no-argument constructor,
/// and generic code can then build a fetcher without knowing its name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a no-argument constructor is already the default in TS
/// ```
impl Default for UreqFetcher {
    /// What:
    /// `fn default() -> UreqFetcher` returns a freshly built fetcher.
    /// Why:
    /// It forwards to `new()` so there is one place that configures the agents.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static default(): UreqFetcher { return new UreqFetcher(); }
    /// ```
    fn default() -> UreqFetcher {
        return UreqFetcher::new();
    }
}

/// What:
/// `impl TextFetcher for UreqFetcher` makes the real network usable as a `TextFetcher`.
/// Why:
/// The registry asks for a fetcher by trait,
/// so tests pass canned responses and production passes this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UreqFetcher implements TextFetcher { get(url) { ... } }
/// ```
impl TextFetcher for UreqFetcher {
    /// What:
    /// `fn get(&self, url)` fetches a URL and returns its body text,
    /// or a `RegistryError` carrying the status and `Retry-After` on any non-2xx answer.
    /// Why:
    /// The registry layer needs the status number to decide whether to retry a 429,
    /// so statuses are data,
    /// not exceptions.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get(url: string): string  // throws RegistryError
    /// ```
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

/// What:
/// `impl DumpSource for UreqFetcher` makes the real network usable as a source of the database dump.
/// Why:
/// The dump fill reads a stream through the trait,
/// so tests pass an archive built in memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UreqFetcher implements DumpSource { open(url) { ... } }
/// ```
impl DumpSource for UreqFetcher {
    /// What:
    /// `fn open(&self, url)` starts the download and returns the response body as a byte stream (`Box<dyn Read>`,
    /// any reader behind one owner).
    /// Why:
    /// The dump is read once,
    /// in order,
    /// and abandoned after the two tables are read,
    /// so it is never held whole in memory.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// open(url: string): ReadableStream  // throws RegistryError
    /// ```
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
