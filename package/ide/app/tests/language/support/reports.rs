//! Reading the scripted server's report file:
//!  the records it holds,
//!  waiting for one to arrive,
//!  and the fields the tests ask about.

use super::child_process::{
    PATIENCE,
    report_path,
};
use ide_app::language::reply::LanguageReply;
use serde_json::Value;
use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
    time::{
        Duration,
        Instant,
    },
};

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
    pub fn new(
        root: &Path,
        what: &str,
    ) -> Self {
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
pub fn all_received<'a>(
    lines: &'a (impl AsRef<[Value]> + ?Sized),
    method: &str,
) -> Vec<&'a Value> {
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
pub fn count_received(
    lines: &(impl AsRef<[Value]> + ?Sized),
    method: &str,
) -> usize {
    let mut count = 0;
    for received_method in received(lines.as_ref()) {
        if received_method == method {
            count += 1;
        }
    }
    return count;
}

/// The position of the first entry equal to `method`.
pub fn index_of(
    methods: &[String],
    method: &str,
) -> Option<usize> {
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
pub fn server_text_until(
    root: &Path,
    expected: &str,
) -> Vec<Value> {
    let mut wait = ReportWait::new(root, "the server text to equal the document text");
    while !text_equals(wait.lines(), expected) {
        wait.again();
    }
    return wait.into_lines();
}

/// True when the server's latest copy of the text equals `expected`.
fn text_equals(
    lines: &[Value],
    expected: &str,
) -> bool {
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
