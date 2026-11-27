//! What:
//! The two `HttpGet` implementations for a run that must not reach the network:
//! `UnavailableNetwork` fails every request with the reason the real client could not be built
//! (with `connect_or_explain` choosing it over `HyperClient`),
//! and `NoNetwork` fails every request with one fixed sentence.
//! Why:
//! A machine without root certificates still runs the commands that need no network,
//! and `list` and `bin` never need it at all:
//! handing them one of these makes "they never touch the network" a fact the compiler and a test can check,
//! and a command that does reach for it gets a diagnostic that says why.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // http/a_client_that_never_reaches_the_network.ts
//! export class UnavailableNetwork implements HttpGet { constructor(readonly reason: string) {} }
//! export const noNetwork: HttpGet = { get: refuse, download: refuse };
//! ```

/// Imports `Path` from `std::path`.
use std::path::Path;

/// Imports the error,
/// the trait,
/// the response
/// and the real client from `super`.
use super::{
    HttpError,
    HttpGet,
    HttpResponse,
    HyperClient,
};

/// What:
/// An `HttpGet` that fails every request with the reason the real client could not be built.
/// Why:
/// A machine without root certificates still runs commands that need no network;
/// only a command that
///           reaches for the network fails,
/// and the message says why.
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

/// Implements `HttpGet` for `UnavailableNetwork`.
impl HttpGet for UnavailableNetwork {
    /// Fails with the reason the client could not be built.
    fn get(
        &self,
        url: &str,
        _headers: &[(String, String)],
        _max_bytes: usize,
    ) -> Result<HttpResponse, HttpError> {
        return Err(HttpError {
            url: url.to_string(),
            message: self.reason.clone(),
        });
    }

    /// Fails with the reason the client could not be built.
    fn download(
        &self,
        url: &str,
        _target: &Path,
        _max_bytes: u64,
    ) -> Result<String, HttpError> {
        return Err(HttpError {
            url: url.to_string(),
            message: self.reason.clone(),
        });
    }
}

/// The real client,
/// or an `UnavailableNetwork` carrying the reason it could not be built.
pub fn connect_or_explain() -> Box<dyn HttpGet> {
    return match HyperClient::new() {
        Ok(client) => Box::new(client),
        Err(reason) => Box::new(UnavailableNetwork {
            reason,
        }),
    };
}

/// What:
/// An `HttpGet` that refuses every request.
/// Why:
/// `list` and `bin` never need the network;
/// handing them this makes "they never touch it" a fact the
///           compiler and a test can check,
/// and gives a clear diagnostic if code ever tries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noNetwork: HttpGet = { get: (url) => { throw new HttpError(url, '...'); }, download: ... };
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct NoNetwork;

/// Implements `HttpGet` for `NoNetwork`.
impl HttpGet for NoNetwork {
    /// Refuses:
    /// this command does not use the network.
    fn get(
        &self,
        url: &str,
        _headers: &[(String, String)],
        _max_bytes: usize,
    ) -> Result<HttpResponse, HttpError> {
        return Err(HttpError {
            url: url.to_string(),
            message: "this command does not use the network".to_string(),
        });
    }

    /// Refuses:
    /// this command does not use the network.
    fn download(
        &self,
        url: &str,
        _target: &Path,
        _max_bytes: u64,
    ) -> Result<String, HttpError> {
        return Err(HttpError {
            url: url.to_string(),
            message: "this command does not use the network".to_string(),
        });
    }
}
