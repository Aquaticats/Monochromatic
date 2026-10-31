//! Test doubles for the two seams:
//! a recording `FakeRunner` and a table-driven `FakeHttp`.
//! A test that uses them
//! proves,
//! by what they recorded,
//! that nothing reached the real host or the network.

/// Imports `RefCell` from `std::cell`.
use std::cell::RefCell;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// Imports `sha256_hex` from `super::hash`.
use super::hash::sha256_hex;
/// Imports what this module uses from `super::http`.
use super::http::{HttpError, HttpGet, HttpResponse};
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest, Runner};

/// What:
/// A `Runner` that starts nothing:
/// it records every request and answers from rules.
/// Why:
/// A rule is a prefix of the request line (program base name,
/// then arguments) and the outcome to answer;
///           the first matching rule wins and an unmatched request succeeds with no output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FakeRunner implements Runner { calls: RunRequest[] = []; rules: [string, RunOutcome][] = []; }
/// ```
#[derive(Default)]
pub(crate) struct FakeRunner {
    /// Every request received,
    /// in order.
    calls: RefCell<Vec<RunRequest>>,
    /// Answers by request-line prefix.
    rules: RefCell<Vec<(String, RunOutcome)>>,
    /// Side effects run when a request-line prefix matches.
    effects: RefCell<Vec<(String, Box<dyn Fn(&RunRequest)>)>>,
}

/// The methods of `FakeRunner`.
impl FakeRunner {
    /// An empty runner that has recorded nothing.
    pub(crate) fn new() -> FakeRunner {
        return FakeRunner::default();
    }

    /// Runs `effect` (for example,
    /// creating the files a real tool would create) whenever a request whose line starts
    /// with `prefix` arrives.
    pub(crate) fn on(&self, prefix: &str, effect: Box<dyn Fn(&RunRequest)>) {
        self.effects.borrow_mut().push((prefix.to_string(), effect));
    }

    /// Answers requests whose line starts with `prefix` with this stdout and exit code.
    pub(crate) fn answer(&self, prefix: &str, stdout: &str, code: i32) {
        self.rules.borrow_mut().push((prefix.to_string(), RunOutcome { stdout: stdout.to_string(), stderr: String::new(), code }));
    }

    /// Answers requests whose line starts with `prefix` with a failure printing `stderr`.
    pub(crate) fn fail(&self, prefix: &str, stderr: &str) {
        self.rules.borrow_mut().push((prefix.to_string(), RunOutcome { stdout: String::new(), stderr: stderr.to_string(), code: 1 }));
    }

    /// Every request received,
    /// in order.
    pub(crate) fn calls(&self) -> Vec<RunRequest> {
        return self.calls.borrow().clone();
    }

    /// Every request as a line with the program reduced to its base name,
    /// in order.
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

/// Implements `Runner` for `FakeRunner`.
impl Runner for FakeRunner {
    /// Records the request,
    /// runs any matching effect,
    /// and answers from the first matching rule.
    fn run(&self, request: &RunRequest) -> RunOutcome {
        self.calls.borrow_mut().push(request.clone());
        let line: String = matching_line(request);
        for (prefix, effect) in self.effects.borrow().iter() {
            if line.starts_with(prefix.as_str()) {
                effect(request);
            }
        }
        for (prefix, outcome) in self.rules.borrow().iter() {
            if line.starts_with(prefix.as_str()) {
                return outcome.clone();
            }
        }
        return RunOutcome { stdout: String::new(), stderr: String::new(), code: 0 };
    }
}

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
    pub(crate) fn serve(&self, url: &str, body: &[u8]) {
        self.bodies.borrow_mut().push((url.to_string(), body.to_vec()));
    }

    /// Fails requests for exactly this URL with this message.
    pub(crate) fn fail(&self, url: &str, message: &str) {
        self.failures.borrow_mut().push((url.to_string(), message.to_string(), None));
    }

    /// Fails the first `times` requests for exactly this URL,
    /// then serves it normally.
    pub(crate) fn fail_first(&self, url: &str, message: &str, times: usize) {
        self.failures.borrow_mut().push((url.to_string(), message.to_string(), Some(times)));
    }

    /// Every URL requested,
    /// in order.
    pub(crate) fn requests(&self) -> Vec<String> {
        return self.requests.borrow().clone();
    }

    /// The body served for this URL,
    /// if any.
    fn body_of(&self, url: &str) -> Option<Vec<u8>> {
        for (served, body) in self.bodies.borrow().iter() {
            if served == url {
                return Some(body.clone());
            }
        }
        return None;
    }

    /// The configured failure for this URL,
    /// consuming one of a limited count.
    fn failure_of(&self, url: &str) -> Option<String> {
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

    /// Records the URL and writes the served body to the target,
    /// returning its hash.
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

/// What:
/// Everything a test of a provider,
/// observer or resolver needs:
/// a disposable scope on disk,
/// the sample lock,
/// a
///           fake runner and a fake network,
/// and a Linux x64 host.
/// Why:
/// One constructor instead of the same twelve lines in every test file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Rig { scope; env; runner = new FakeRunner(); http = new FakeHttp(); lock = sampleLock(); host = linuxX64; }
/// ```
pub(crate) struct Rig {
    /// Keeps the disposable directory alive until the rig drops.
    pub(crate) _directory: tempfile::TempDir,
    /// The scope on disk.
    pub(crate) scope: super::scope::Scope,
    /// The fake environment (HOME inside the directory).
    pub(crate) env: super::scope::EnvMap,
    /// The recording fake runner.
    pub(crate) runner: FakeRunner,
    /// The table-driven fake network.
    pub(crate) http: FakeHttp,
    /// The locked tools the context sees.
    pub(crate) lock: super::lock::Lock,
    /// The pretended machine.
    pub(crate) host: super::context::Host,
}

/// The methods of `Rig`.
impl Rig {
    /// A rig with a fresh disposable scope,
    /// the sample lock and empty fakes.
    pub(crate) fn new() -> Rig {
        let (directory, scope, env) = super::fixtures::sample_scope_on_disk();
        return Rig {
            _directory: directory,
            scope,
            env,
            runner: FakeRunner::new(),
            http: FakeHttp::new(),
            lock: super::fixtures::sample_lock(),
            host: super::context::Host { platform: "linux-x64".to_string(), family: Some(super::model::OsFamily::Linux) },
        };
    }

    /// A context borrowing this rig.
    pub(crate) fn context(&self) -> super::context::Context<'_> {
        return super::context::Context {
            scope: &self.scope,
            env: &self.env,
            runner: &self.runner,
            http: &self.http,
            host: &self.host,
            tools: &self.lock.tools,
        };
    }

    /// The locked tool with this id (cloned so the test can keep using the rig).
    pub(crate) fn tool(&self, id: &str) -> super::model::LockedTool {
        return self.lock.tool(id).expect("the sample lock holds the tool").clone();
    }

    /// Writes an executable stand-in at `<install dir of tool>/<name>`.
    pub(crate) fn install_binary(&self, tool_id: &str, name: &str) -> std::path::PathBuf {
        let tool = self.tool(tool_id);
        let path = super::layout::tool_install_dir(&self.scope.toolchain_root, tool_id, &tool.version).join(name);
        write_executable(&path);
        return path;
    }
}

/// Writes a small executable file,
/// creating its directory.
pub(crate) fn write_executable(path: &std::path::Path) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("make the directory");
    std::fs::write(path, "#!/bin/sh\n").expect("write");
    #[cfg(unix)]
    {
        /// Imports `PermissionsExt` from `std::os::unix::fs`.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}
