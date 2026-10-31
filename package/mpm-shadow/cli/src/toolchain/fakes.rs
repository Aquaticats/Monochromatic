//! Test doubles for the two seams: a recording `FakeRunner` and a table-driven `FakeHttp`. A test that uses them
//! proves, by what they recorded, that nothing reached the real host or the network.

use std::cell::RefCell;
use std::path::Path;

use super::hash::sha256_hex;
use super::http::{HttpError, HttpGet, HttpResponse};
use super::runner::{RunOutcome, RunRequest, Runner};

/// What:     A `Runner` that starts nothing: it records every request and answers from rules.
/// Why:      A rule is a prefix of the request line (program base name, then arguments) and the outcome to answer;
///           the first matching rule wins and an unmatched request succeeds with no output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FakeRunner implements Runner { calls: RunRequest[] = []; rules: [string, RunOutcome][] = []; }
/// ```
#[derive(Default)]
pub(crate) struct FakeRunner {
    calls: RefCell<Vec<RunRequest>>,
    rules: RefCell<Vec<(String, RunOutcome)>>,
}

impl FakeRunner {
    pub(crate) fn new() -> FakeRunner {
        return FakeRunner::default();
    }

    /// Answers requests whose line starts with `prefix` with this stdout and exit code.
    pub(crate) fn answer(&self, prefix: &str, stdout: &str, code: i32) {
        self.rules.borrow_mut().push((prefix.to_string(), RunOutcome { stdout: stdout.to_string(), stderr: String::new(), code }));
    }

    /// Answers requests whose line starts with `prefix` with a failure printing `stderr`.
    pub(crate) fn fail(&self, prefix: &str, stderr: &str) {
        self.rules.borrow_mut().push((prefix.to_string(), RunOutcome { stdout: String::new(), stderr: stderr.to_string(), code: 1 }));
    }

    /// Every request received, in order.
    pub(crate) fn calls(&self) -> Vec<RunRequest> {
        return self.calls.borrow().clone();
    }

    /// Every request as a line with the program reduced to its base name, in order.
    pub(crate) fn lines(&self) -> Vec<String> {
        return self.calls.borrow().iter().map(matching_line).collect();
    }
}

/// The request line with the program's base name (`/x/y/uv tool list` becomes `uv tool list`).
fn matching_line(request: &RunRequest) -> String {
    let base: &str = request.command.rsplit('/').next().unwrap_or(&request.command);
    let mut line: String = base.to_string();
    for argument in &request.args {
        line.push(' ');
        line.push_str(argument);
    }
    return line;
}

impl Runner for FakeRunner {
    fn run(&self, request: &RunRequest) -> RunOutcome {
        self.calls.borrow_mut().push(request.clone());
        let line: String = matching_line(request);
        for (prefix, outcome) in self.rules.borrow().iter() {
            if line.starts_with(prefix.as_str()) {
                return outcome.clone();
            }
        }
        return RunOutcome { stdout: String::new(), stderr: String::new(), code: 0 };
    }
}

/// What:     An `HttpGet` that answers from a table of URL to body and records every URL asked for.
/// Why:      Tests run the resolvers and the archive installer without a network; an URL missing from the table is a
///           404 so a test notices an unexpected request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FakeHttp implements HttpGet { requests: string[] = []; bodies = new Map<string, Uint8Array>(); }
/// ```
#[derive(Default)]
pub(crate) struct FakeHttp {
    requests: RefCell<Vec<String>>,
    bodies: RefCell<Vec<(String, Vec<u8>)>>,
    failures: RefCell<Vec<(String, String)>>,
}

impl FakeHttp {
    pub(crate) fn new() -> FakeHttp {
        return FakeHttp::default();
    }

    /// Serves `body` for exactly this URL.
    pub(crate) fn serve(&self, url: &str, body: &[u8]) {
        self.bodies.borrow_mut().push((url.to_string(), body.to_vec()));
    }

    /// Fails requests for exactly this URL with this message.
    pub(crate) fn fail(&self, url: &str, message: &str) {
        self.failures.borrow_mut().push((url.to_string(), message.to_string()));
    }

    /// Every URL requested, in order.
    pub(crate) fn requests(&self) -> Vec<String> {
        return self.requests.borrow().clone();
    }

    fn body_of(&self, url: &str) -> Option<Vec<u8>> {
        for (served, body) in self.bodies.borrow().iter() {
            if served == url {
                return Some(body.clone());
            }
        }
        return None;
    }

    fn failure_of(&self, url: &str) -> Option<String> {
        for (failing, message) in self.failures.borrow().iter() {
            if failing == url {
                return Some(message.clone());
            }
        }
        return None;
    }
}

impl HttpGet for FakeHttp {
    fn get(&self, url: &str, _headers: &[(String, String)], _max_bytes: usize) -> Result<HttpResponse, HttpError> {
        self.requests.borrow_mut().push(url.to_string());
        if let Some(message) = self.failure_of(url) {
            return Err(HttpError { url: url.to_string(), message });
        }
        return match self.body_of(url) {
            Some(body) => Ok(HttpResponse { status: 200, body }),
            None => Ok(HttpResponse { status: 404, body: b"not served by the fake".to_vec() }),
        };
    }

    fn download(&self, url: &str, target: &Path, _max_bytes: u64) -> Result<String, HttpError> {
        self.requests.borrow_mut().push(url.to_string());
        if let Some(message) = self.failure_of(url) {
            return Err(HttpError { url: url.to_string(), message });
        }
        let Some(body) = self.body_of(url) else {
            return Err(HttpError { url: url.to_string(), message: "HTTP 404".to_string() });
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).expect("make the download directory");
        }
        std::fs::write(target, &body).expect("write the fake download");
        return Ok(sha256_hex(&body));
    }
}
