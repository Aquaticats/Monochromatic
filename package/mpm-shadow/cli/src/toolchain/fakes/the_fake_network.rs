//! What:
//! An `HttpGet` that answers from a table of URL to body,
//! records every URL asked for,
//! and can fail one URL a fixed number of times.
//! Why:
//! Tests run the resolvers and the archive installer without a network,
//! and a URL missing from the table answers 404,
//! so a test notices an unexpected request and proves by what the fake recorded that nothing dialled out.
//! A caller imports from `fakes` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export class FakeHttp implements HttpGet {
//!   requests: string[] = [];
//!   serve(url: string, body: Uint8Array): void;
//!   fail(url: string, message: string): void;
//!   failFirst(url: string, message: string, times: number): void;
//! }
//! ```

/// Imports `RefCell` from `std::cell`,
/// which is how a fake records through a shared reference.
use std::cell::RefCell;
/// Imports `Path` from `std::path`,
/// the file a download is written to.
use std::path::Path;

/// Imports `sha256_hex` from `super::super::hash`,
/// which answers the digest of the body it wrote.
use super::super::hash::sha256_hex;
/// Imports what this file uses from `super::super::http`.
use super::super::http::{
    HttpError,
    HttpGet,
    HttpResponse,
};
/// What:
/// An `HttpGet` that answers from a table of URL to body and records every URL asked for.
/// Why:
/// Tests run the resolvers and the archive installer without a network;
/// an URL missing from the table is a
///           404 so a test notices an unexpected request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FakeHttp implements HttpGet { requests: string[] = []; bodies = new Map<string, Uint8Array>(); }
/// ```
#[derive(Default)]
pub(crate) struct FakeHttp {
    /// Every URL requested,
    /// in order.
    requests: RefCell<Vec<String>>,
    /// URL to served body.
    bodies: RefCell<Vec<(String, Vec<u8>)>>,
    /// URL to failure message,
    /// with an optional count of failures left.
    failures: RefCell<Vec<(String, String, Option<usize>)>>,
}

/// The methods of `FakeHttp`.
impl FakeHttp {
    /// An empty network table.
    pub(crate) fn new() -> FakeHttp {
        return FakeHttp::default();
    }

    /// Serves `body` for exactly this URL.
    pub(crate) fn serve(
        &self,
        url: &str,
        body: &[u8],
    ) {
        self.bodies
            .borrow_mut()
            .push((url.to_string(), body.to_vec()));
    }

    /// Fails requests for exactly this URL with this message.
    pub(crate) fn fail(
        &self,
        url: &str,
        message: &str,
    ) {
        self.failures
            .borrow_mut()
            .push((url.to_string(), message.to_string(), None));
    }

    /// Fails the first `times` requests for exactly this URL,
    /// then serves it normally.
    pub(crate) fn fail_first(
        &self,
        url: &str,
        message: &str,
        times: usize,
    ) {
        self.failures
            .borrow_mut()
            .push((url.to_string(), message.to_string(), Some(times)));
    }

    /// Every URL requested,
    /// in order.
    pub(crate) fn requests(&self) -> Vec<String> {
        return self.requests.borrow().clone();
    }

    /// The body served for this URL,
    /// if any.
    fn body_of(
        &self,
        url: &str,
    ) -> Option<Vec<u8>> {
        for (served, body) in self.bodies.borrow().iter() {
            if served == url {
                return Some(body.clone());
            }
        }
        return None;
    }

    /// The configured failure for this URL,
    /// consuming one of a limited count.
    fn failure_of(
        &self,
        url: &str,
    ) -> Option<String> {
        for (failing, message, remaining) in self.failures.borrow_mut().iter_mut() {
            if failing != url {
                continue;
            }
            match remaining {
                None => return Some(message.clone()),
                Some(0) => continue,
                Some(left) => {
                    *left -= 1;
                    return Some(message.clone());
                }
            }
        }
        return None;
    }
}

/// Implements `HttpGet` for `FakeHttp`.
impl HttpGet for FakeHttp {
    /// Records the URL and answers from the table (404 for an unknown URL).
    fn get(
        &self,
        url: &str,
        _headers: &[(String, String)],
        _max_bytes: usize,
    ) -> Result<HttpResponse, HttpError> {
        self.requests.borrow_mut().push(url.to_string());
        if let Some(message) = self.failure_of(url) {
            return Err(HttpError {
                url: url.to_string(),
                message,
            });
        }
        return match self.body_of(url) {
            Some(body) => Ok(HttpResponse {
                status: 200,
                body,
            }),
            None => Ok(HttpResponse {
                status: 404,
                body: b"not served by the fake".to_vec(),
            }),
        };
    }

    /// Records the URL and writes the served body to the target,
    /// returning its hash.
    fn download(
        &self,
        url: &str,
        target: &Path,
        _max_bytes: u64,
    ) -> Result<String, HttpError> {
        self.requests.borrow_mut().push(url.to_string());
        if let Some(message) = self.failure_of(url) {
            return Err(HttpError {
                url: url.to_string(),
                message,
            });
        }
        let Some(body) = self.body_of(url) else {
            return Err(HttpError {
                url: url.to_string(),
                message: "HTTP 404".to_string(),
            });
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).expect("make the download directory");
        }
        std::fs::write(target, &body).expect("write the fake download");
        return Ok(sha256_hex(&body));
    }
}
