//! Shared fixtures for Language integration tests:
//!  a child process per test,
//!  a scripted server,
//! and a polling probe around the worker handle.
//!
//! Helix derives every server root from the process working directory and keeps the first
//! value it reads.
//!  A test that starts a server therefore runs its body in a child process whose
//! working directory was set when it was spawned;
//!  the test process itself never changes directory.

use helix_core::Rope;
use ide_app::document::Document;
use ide_app::language::notice::SOURCE as NOTICE_SOURCE;
use ide_app::language::{
    LanguageWorker,
    config::LanguageSetup,
    diagnostics::DiagnosticsSnapshot,
    hints::HintsSnapshot,
    identity::DocumentStamp,
    reply::{LanguageReply, PositionRequest, RequestKind},
    status::{LanguageStatus, ServerState},
    sync::{DocumentOpen, DocumentReload},
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};

/// Set in the child process to the project root the body works on.
const ROOT_VARIABLE: &str = "IDE_LANGUAGE_TEST_ROOT";

/// Name of the scripted server definition.
pub const SERVER: &str = "scripted-ls";

/// Longest wait for any expected state.
const PATIENCE: Duration = Duration::from_secs(20);

/// Request timeout of the scripted server,
///  in seconds,
///  in tests whose assertions do not depend on
/// it:
///  Helix's default,
///  which the product uses for every server without its own value
/// (`default_timeout` in `helix-core/src/syntax/config.rs`,
///  and `Languages::timeout`).
pub const PRODUCT_TIMEOUT: u64 = 20;

/// When set in the environment,
///  the child process prints the worker's debug log and helix-lsp's
/// protocol log to standard error,
///  with wall-clock times.
///  An intermittent failure is diagnosed
/// from that log together with the scripted server's report.
const LOG_VARIABLE: &str = "IDE_LANGUAGE_TEST_LOG";

/// Install the opt-in log once per process;
///  later probes of the same test find it installed.
fn install_log() {
    if std::env::var_os(LOG_VARIABLE).is_none() {
        return;
    }
    // helix-lsp logs every message it writes and reads at `info`, and dropped answers at `debug`.
    // The application's pipeline re-labels helix-lsp's records as in a real session. Installing
    // fails only when an earlier probe of this process already installed the log.
    let filter = tracing_subscriber::EnvFilter::builder().parse_lossy(
        ide_app::logging::directives("ide_app=debug,helix_lsp=debug", None),
    );
    let _already_installed = ide_app::logging::install(filter, std::io::stderr, None);
}

/// Where the child process works:
///  its project root,
///  its working directory,
///  and its `PWD`.
pub struct Layout {
    /// Project root handed to the worker.
    pub root: PathBuf,
    /// Working directory of the child process.
    pub cwd: PathBuf,
    /// Value of the `PWD` variable,
    ///  which Helix prefers when it names the working directory;
    /// nothing removes the variable.
    pub pwd: Option<PathBuf>,
}

/// The usual layout:
///  the project root is the working directory.
pub fn standard(base: &Path) -> Layout {
    let root = base.join("project");
    fs::create_dir(&root).expect("project directory");
    return Layout {
        root: root.clone(),
        cwd: root,
        pwd: None,
    };
}

/// In the child process:
///  the project root.
///  In the test process:
///  nothing.
pub fn child_root() -> Option<PathBuf> {
    return std::env::var_os(ROOT_VARIABLE).map(PathBuf::from);
}

/// Memory-backed directory for the files of a test,
///  when the system has one.
const MEMORY_DIRECTORY: &str = "/dev/shm";

/// A fresh directory for one test's project and report,
///  in memory when possible.
///
/// The scripted server appends a line to its report for every message it reads or writes,
///  and
/// it answers only after the line is written.
///  On a disk that the rest of the machine keeps busy,
/// one such write can block for seconds;
///  the late answers then look like a slow server,
///  and the
/// server cannot even be killed until the write returns.
///  Memory-backed files never wait for the
/// disk.
///  Elsewhere the usual temporary directory is used.
fn test_directory() -> tempfile::TempDir {
    let memory = Path::new(MEMORY_DIRECTORY);
    if memory.is_dir()
        && let Ok(directory) = tempfile::Builder::new()
            .prefix("ide-language-")
            .tempdir_in(memory)
    {
        return directory;
    }
    return tempfile::tempdir().expect("test directory");
}

/// Run the named test again in a child process laid out by `layout`,
///  and require it to pass.
/// `test` is the full test name,
///  including its module.
pub fn run_child(test: &str, layout: fn(&Path) -> Layout) {
    let directory = test_directory();
    let base = directory
        .path()
        .canonicalize()
        .expect("canonical test directory");
    fs::create_dir(base.join("scratch")).expect("scratch directory");
    let placed = layout(&base);
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        // An ignored test that was asked for by name must run in the child too; without this the
        // child would run nothing and report success.
        .args([
            test,
            "--exact",
            "--nocapture",
            "--test-threads=1",
            "--include-ignored",
        ])
        .current_dir(&placed.cwd)
        .env(ROOT_VARIABLE, &placed.root);
    match &placed.pwd {
        Some(pwd) => command.env("PWD", pwd),
        // The parent's value names another directory and would only mislead a reader of the log.
        None => command.env_remove("PWD"),
    };
    let status = command.status().expect("child test process");
    assert!(status.success(), "child process of {test} failed: {status}");
}

/// The scratch directory beside the project,
///  for report files and outside-project fixtures.
pub fn scratch(root: &Path) -> PathBuf {
    let mut current = root;
    loop {
        let candidate = current.join("scratch");
        if candidate.is_dir() && !candidate.starts_with(root) {
            return candidate;
        }
        current = current
            .parent()
            .expect("scratch directory beside the project");
    }
}

/// Path of the scripted server's report file.
pub fn report_path(root: &Path) -> PathBuf {
    return scratch(root).join("report.jsonl");
}

/// Language definitions for the scripted server with the given environment and request timeout.
pub fn scripted(root: &Path, variables: &[(&str, &str)], timeout: u64) -> String {
    return scripted_with_roots(root, variables, timeout, "[]");
}

/// Like `scripted`,
///  with root-marker file names in TOML array syntax.
pub fn scripted_with_roots(
    root: &Path,
    variables: &[(&str, &str)],
    timeout: u64,
    roots: &str,
) -> String {
    let mut environment = format!("IDE_SCRIPTED_REPORT = '{}'", report_path(root).display());
    for (name, value) in variables {
        environment.push_str(&format!(", IDE_SCRIPTED_{name} = '{value}'"));
    }
    return format!(
        r#"
[language-server.{SERVER}]
command = '{program}'
timeout = {timeout}
environment = {{ {environment} }}

[language-server.{SERVER}.config.scripted]
flag = true
nested = {{ value = 42 }}

[[language]]
name = "scripted"
scope = "source.scripted"
file-types = ["scripted"]
roots = {roots}
language-servers = ["{SERVER}"]
"#,
        program = env!("CARGO_BIN_EXE_ide-scripted-lsp"),
    );
}

/// The worker handle plus everything polled from it so far.
pub struct Probe {
    /// The handle under test.
    pub worker: LanguageWorker,
    /// Latest status seen.
    pub status: Arc<LanguageStatus>,
    /// Every reply the fence let through,
    ///  in arrival order.
    pub replies: Vec<LanguageReply>,
    /// Latest diagnostics seen.
    pub diagnostics: Option<Arc<DiagnosticsSnapshot>>,
    /// Latest hints seen.
    pub hints: Option<Arc<HintsSnapshot>>,
    /// The document as the application would hold it.
    pub document: Document,
    /// File-open generation of the displayed file.
    pub file: u64,
    /// Path of the displayed file.
    pub path: PathBuf,
    /// What the current wait is for,
    ///  named in the failure when the wait times out.
    waiting_for: String,
    /// When the current wait began.
    wait_started: Instant,
}

impl Probe {
    /// Start a worker for `root` with application-supplied definitions.
    pub fn new(root: &Path, definitions: String) -> Self {
        let setup = LanguageSetup {
            extra_languages: Some(definitions),
            ..LanguageSetup::unconfined()
        };
        return Self::with_setup(root, setup);
    }

    /// Start a worker for `root` with an explicit setup.
    pub fn with_setup(root: &Path, setup: LanguageSetup) -> Self {
        install_log();
        let worker = LanguageWorker::with_setup(root, setup).expect("language worker");
        return Self {
            worker,
            status: Arc::new(LanguageStatus::closed()),
            replies: Vec::new(),
            diagnostics: None,
            hints: None,
            document: Document::new(""),
            file: 0,
            path: PathBuf::new(),
            waiting_for: String::new(),
            wait_started: Instant::now(),
        };
    }

    /// The stamp of the displayed text.
    pub fn stamp(&self) -> DocumentStamp {
        return DocumentStamp {
            file: self.file,
            revision: self.document.revision(),
        };
    }

    /// Write `text` to `path` and display it,
    ///  as the application does after reading a file.
    pub fn open(&mut self, path: &Path, text: &str) {
        fs::write(path, text).expect("source file");
        self.display(path, text);
    }

    /// Display `text` for an existing `path` without writing it.
    pub fn display(&mut self, path: &Path, text: &str) {
        self.file += 1;
        self.document = Document::new(text);
        self.path = path.to_path_buf();
        let sent = self
            .worker
            .open(DocumentOpen {
                path: path.to_path_buf(),
                text: Rope::from_str(text),
                stamp: self.stamp(),
            })
            .expect("worker accepts open");
        assert!(sent, "command queue full");
    }

    /// Replace the file on disk and apply the reload exactly as the application's reload path does.
    pub fn reload(&mut self, text: &str) {
        fs::write(&self.path, text).expect("changed source file");
        let reload = self.document.prepare_reload(text);
        let command = DocumentReload::from_reload(self.file, &reload);
        assert!(self.document.apply_reload(reload), "reload accepted");
        assert!(self.worker.reload(command).expect("worker accepts reload"));
    }

    /// Send a position request and return its number.
    pub fn request(&mut self, kind: RequestKind, position: usize) -> u64 {
        let request = PositionRequest {
            stamp: self.stamp(),
            kind,
            position,
        };
        return self
            .worker
            .request(request)
            .expect("worker accepts request")
            .expect("command queue has room");
    }

    /// Drain everything the worker published since the last poll.
    pub fn poll(&mut self) {
        if let Some(status) = self.worker.try_take_status().expect("status poll") {
            self.status = status;
        }
        while let Some(reply) = self.worker.try_take_reply().expect("reply poll") {
            self.replies.push(reply);
        }
        if let Some(diagnostics) = self
            .worker
            .try_take_diagnostics()
            .expect("diagnostics poll")
        {
            self.diagnostics = Some(diagnostics);
        }
        if let Some(hints) = self.worker.try_take_hints().expect("hints poll") {
            self.hints = Some(hints);
        }
    }

    /// Begin a wait for `what` and poll once,
    ///  so the first check of the caller's loop reads fresh state.
    /// The caller loops `while !condition { probe.step(); }`.
    pub fn wait_for(&mut self, what: &str) {
        self.waiting_for = what.to_string();
        self.wait_started = Instant::now();
        self.poll();
    }

    /// One more round of the wait begun by `wait_for`:
    ///  fail with the last observed state when the patience ran out,
    ///  else pause and poll again.
    pub fn step(&mut self) {
        assert!(
            self.wait_started.elapsed() < PATIENCE,
            "timed out waiting for {}; status {:?}; replies {:?}; diagnostics {:?}; hints {:?}",
            self.waiting_for,
            self.status,
            self.replies,
            self.diagnostics,
            self.hints
        );
        std::thread::sleep(Duration::from_millis(5));
        self.poll();
    }

    /// True when some reply answers request `number`.
    pub fn has_reply(&self, number: u64) -> bool {
        for reply in &self.replies {
            if reply.request == number {
                return true;
            }
        }
        return false;
    }

    /// True when the last reply of request `number` arrived.
    pub fn is_answered(&self, number: u64) -> bool {
        for reply in &self.replies {
            if reply.request == number && reply.remaining == 0 {
                return true;
            }
        }
        return false;
    }

    /// True when the held diagnostics carry `stamp`.
    pub fn diagnostics_stamped(&self, stamp: DocumentStamp) -> bool {
        return match self.diagnostics.as_ref() {
            Some(found) => found.stamp == stamp,
            None => false,
        };
    }

    /// True when the held hints carry `stamp`.
    pub fn hints_stamped(&self, stamp: DocumentStamp) -> bool {
        return match self.hints.as_ref() {
            Some(held) => held.stamp == stamp,
            None => false,
        };
    }

    /// True when the notice entry exists and its text contains `part`.
    pub fn notice_contains(&self, part: &str) -> bool {
        return match self.notice() {
            Some(entry) => entry.contains(part),
            None => false,
        };
    }

    /// State of the named server in the latest status.
    pub fn state(&self, name: &str) -> Option<&ServerState> {
        for row in &self.status.servers {
            if row.server.name == name {
                return Some(&row.state);
            }
        }
        return None;
    }

    /// Wait until the scripted server is ready.
    pub fn until_ready(&mut self) {
        self.wait_for("the scripted server to be ready");
        while self.state(SERVER) != Some(&ServerState::Ready) {
            self.step();
        }
    }

    /// Wait for the last reply of request `number` and return every reply it produced.
    pub fn answers(&mut self, number: u64) -> Vec<LanguageReply> {
        self.wait_for("the request to be answered");
        while !self.is_answered(number) {
            self.step();
        }
        let mut found = Vec::new();
        for reply in &self.replies {
            if reply.request == number {
                found.push(reply.clone());
            }
        }
        return found;
    }

    /// Every message of the servers' own diagnostics on display,
    ///  in group and position order;
    ///  the entry that explains missing language support is not one of them (see `notice`).
    pub fn messages(&self) -> Vec<String> {
        let mut found = Vec::new();
        if let Some(snapshot) = &self.diagnostics {
            for group in &snapshot.groups {
                if group.source == NOTICE_SOURCE {
                    continue;
                }
                for item in &group.items {
                    found.push(item.message.clone());
                }
            }
        }
        return found;
    }

    /// The message of the entry that explains missing language support,
    ///  when the displayed diagnostics hold it.
    pub fn notice(&self) -> Option<String> {
        let snapshot = self.diagnostics.as_ref()?;
        let mut noticed = None;
        for group in snapshot.groups.iter() {
            if group.source == NOTICE_SOURCE {
                noticed = Some(group);
                break;
            }
        }
        let group = noticed?;
        let item = group.items.first()?;
        return Some(item.message.clone());
    }
}

/// Every line the scripted server reported so far.
pub fn report(root: &Path) -> Vec<Value> {
    let Ok(text) = fs::read_to_string(report_path(root)) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for line in text.lines() {
        // A line still being written is skipped; the next read sees it whole.
        if let Ok(value) = serde_json::from_str(line) {
            lines.push(value);
        }
    }
    return lines;
}

/// What:
///  A wait for the scripted server's report to show something.
///  The caller loops `while !condition(wait.lines()) { wait.again(); }`
///  and takes the lines with `into_lines`.
/// Why:
///  The report is a file the server appends to,
///  so the wait rereads it between pauses.
pub struct ReportWait {
    /// The project root whose report is read.
    root: PathBuf,
    /// What the wait is for,
    ///  named in the failure when it times out.
    what: String,
    /// When the wait began.
    started: Instant,
    /// The report as last read.
    lines: Vec<Value>,
}

impl ReportWait {
    /// Begin a wait for `what` and read the report once.
    pub fn new(root: &Path, what: &str) -> Self {
        return Self {
            root: root.to_path_buf(),
            what: what.to_string(),
            started: Instant::now(),
            lines: report(root),
        };
    }

    /// The report as last read.
    pub fn lines(&self) -> &[Value] {
        return &self.lines;
    }

    /// Fail with the last report when the patience ran out,
    ///  else pause and read the report again.
    pub fn again(&mut self) {
        assert!(
            self.started.elapsed() < PATIENCE,
            "timed out waiting for the server report to show {}: {:?}",
            self.what,
            self.lines
        );
        std::thread::sleep(Duration::from_millis(5));
        self.lines = report(&self.root);
    }

    /// The report as last read,
    ///  once the wait is over.
    pub fn into_lines(self) -> Vec<Value> {
        return self.lines;
    }
}

/// The first reported line whose `received` method is `method`.
pub fn first_received<'a>(
    lines: &'a (impl AsRef<[Value]> + ?Sized),
    method: &str,
) -> Option<&'a Value> {
    for line in lines.as_ref() {
        if line["received"] == method {
            return Some(line);
        }
    }
    return None;
}

/// Every reported line whose `received` method is `method`,
///  in order.
pub fn all_received<'a>(lines: &'a (impl AsRef<[Value]> + ?Sized), method: &str) -> Vec<&'a Value> {
    let mut found = Vec::new();
    for line in lines.as_ref() {
        if line["received"] == method {
            found.push(line);
        }
    }
    return found;
}

/// The first reported line that records a server start.
pub fn first_started<'a>(lines: &'a (impl AsRef<[Value]> + ?Sized)) -> Option<&'a Value> {
    for line in lines.as_ref() {
        if !line["started"].is_null() {
            return Some(line);
        }
    }
    return None;
}

/// How many times the server received `method`.
pub fn count_received(lines: &(impl AsRef<[Value]> + ?Sized), method: &str) -> usize {
    let mut count = 0;
    for received_method in received(lines.as_ref()) {
        if received_method == method {
            count += 1;
        }
    }
    return count;
}

/// The position of the first entry equal to `method`.
pub fn index_of(methods: &[String], method: &str) -> Option<usize> {
    for (index, candidate) in methods.iter().enumerate() {
        if candidate == method {
            return Some(index);
        }
    }
    return None;
}

/// Methods the server received,
///  in order.
pub fn received(lines: &[Value]) -> Vec<String> {
    let mut methods = Vec::new();
    for line in lines {
        if let Some(method) = line["received"].as_str() {
            methods.push(method.to_string());
        }
    }
    return methods;
}

/// The process instance of the server that answered a request,
///  or nothing when no server was asked.
pub fn instance_of(answer: &LanguageReply) -> Option<u64> {
    // The trailing `?` returns `None` when no server answered.
    let server = answer.server.as_ref()?;
    return Some(server.instance);
}

/// The server's latest copy of the document text,
///  with the version it was sent under.
pub fn server_text(lines: &[Value]) -> Option<(i64, String)> {
    let mut latest = None;
    for line in lines {
        if let Some(text) = line["text"]["text"].as_str() {
            latest = Some((
                line["text"]["version"].as_i64().unwrap_or(-1),
                text.to_string(),
            ));
        }
    }
    return latest;
}

/// Wait until the server's copy of the text equals `expected`.
pub fn server_text_until(root: &Path, expected: &str) -> Vec<Value> {
    let mut wait = ReportWait::new(root, "the server text to equal the document text");
    while !text_equals(wait.lines(), expected) {
        wait.again();
    }
    return wait.into_lines();
}

/// True when the server's latest copy of the text equals `expected`.
fn text_equals(lines: &[Value], expected: &str) -> bool {
    return match server_text(lines) {
        Some((_, text)) => text == expected,
        None => false,
    };
}

/// Processes whose parent is this process,
///  as `(pid, state letter, command name)`.
pub fn children() -> Vec<(u32, char, String)> {
    let own = std::process::id();
    let mut found = Vec::new();
    for entry in fs::read_dir("/proc").expect("process table").flatten() {
        let name = entry.file_name();
        let Some(text) = name.to_str() else {
            continue;
        };
        let Ok(pid) = text.parse::<u32>() else {
            continue;
        };
        let Ok(stat) = fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        // The command name is in parentheses and may contain spaces; fields follow the last `)`.
        let Some(close) = stat.rfind(')') else {
            continue;
        };
        let open = stat.find('(').unwrap_or(0);
        let fields: Vec<&str> = stat[close + 1..].split_whitespace().collect();
        if fields.len() < 2 {
            continue;
        }
        if fields[1].parse::<u32>().ok() == Some(own) {
            let state = fields[0].chars().next().unwrap_or('?');
            found.push((pid, state, stat[open + 1..close].to_string()));
        }
    }
    return found;
}

/// Wait until this process has no child left,
///  alive or unreaped.
pub fn children_until_none() {
    let start = Instant::now();
    loop {
        let left = children();
        if left.is_empty() {
            return;
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "child processes remain after the worker was dropped: {left:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
