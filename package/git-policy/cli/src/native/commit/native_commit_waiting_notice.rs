//! What:
//!  The waiting notice of the commit Git builds in the scratch repository:
//! while that `git commit` runs,
//! a line on standard error
//! says what the command waits for,
//! how long it has waited,
//! and that the step has no time limit here.
//! Why:
//!  Every other wait of the wrapper says what it waits for (issue 1811),
//! but this step runs with the terminal attached and no watch,
//! because an editor may open in it,
//! so a commit that spent four minutes in it printed nothing.
//! The line names the `git commit` the command started (its process),
//! says the commit is signed when `commit.gpgsign` or `-S` asks
//! for it,
//! and names the signing program while one runs below the `git commit`.
//! No ceiling is added:
//! whether this step may be ended is the owner's question.
//! A commit that opens the person's editor prints nothing,
//! because the person is the wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = startNotice({ signs, cadence }); try { await runGitCommit(); } finally { notice.finish(); }
//! ```

/// The cadence and the clock of a wait notice.
use crate::wait_progress::{Cadence, Ticker};
/// The processes started below this one.
use crate::process_tree::descendants_of;
/// The configuration the commit is built under.
use crate::config_snapshot::ConfigSnapshot;
/// Whether the commit's arguments open an editor.
use crate::service_commit_args_editor_opening::opens_editor;
/// The notice's sink.
use std::io::Write;
/// The channel that ends the notice thread.
use std::sync::mpsc::{Sender, RecvTimeoutError, channel};
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// The shortest pause between two looks of the notice thread,
/// so a very short interval does not spin.
const LOOK_PAUSE: Duration = Duration::from_millis(1);

/// The running notice:
/// the thread that prints it and the sender that ends it.
pub struct NativeCommitNotice {
    /// Ends the thread when it is sent to or dropped.
    stop: Option<Sender<()>>,
    /// The thread,
    /// joined when the step has ended.
    thread: Option<std::thread::JoinHandle<()>>,
}

/// Whether the commit is signed:
/// the configuration says so or the command line asks for it.
pub fn signs_commit(snapshot: Option<&ConfigSnapshot>, commit_arguments: &[String]) -> bool {
    if crate::service_prepare::config_signs(snapshot) {
        return true;
    }
    let mut asked: bool = false;
    for argument in commit_arguments {
        if argument == "--" {
            break;
        }
        if argument == "--no-gpg-sign" {
            asked = false;
        } else if argument.starts_with("--gpg-sign") || (argument.starts_with("-S") && !argument.starts_with("--")) {
            asked = true;
        }
    }
    return asked;
}

/// The program names of what runs below this process,
/// other than Git itself:
/// the signer,
/// for instance.
fn programs_below() -> Vec<(i32, String)> {
    let mut found: Vec<(i32, String)> = Vec::new();
    let own: i32 = i32::try_from(std::process::id()).unwrap_or(0);
    for member in descendants_of(own) {
        let name: String = std::fs::read_to_string(format!("/proc/{}/comm", member.pid)).map(trim_name).unwrap_or_default();
        if !name.is_empty() && name != "git" {
            found.push((member.pid, name));
        }
    }
    return found;
}

/// A name from `comm` without its newline.
fn trim_name(text: String) -> String {
    return text.trim().to_string();
}

/// What:
///  One notice line.
/// Why:
///   The first one adds what to do when the step looks stuck;
/// the repeats stay short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noticeLine({ signs, waited, first, below })
/// ```
pub fn notice_line(signs: bool, waited: Duration, first: bool, below: &[(i32, String)]) -> String {
    let signing: String = if signs { String::from(", which signs the commit because `commit.gpgsign` or `-S` asks for it") } else { String::new() };
    let signer: String = match below.first() {
        Some((pid, name)) => format!("; `{name}` (process {pid}) runs below it"),
        None => String::new(),
    };
    let hint: &str = if first { " Hint: it is still running; if it looks stuck, check it with `ps` and report it." } else { "" };
    return format!(
        "monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository{signing}{signer}, {} seconds so far; \
         it has no time limit here, so it is waited for as long as it lives.{hint}\n",
        waited.as_secs()
    );
}

impl NativeCommitNotice {
    /// A notice that prints nothing:
    /// the commit opens an editor,
    /// or no thread could be started.
    fn silent() -> Self {
        return Self { stop: None, thread: None };
    }

    /// What:
    ///  Starts the notice for a `git commit` that is about to run.
    /// Why:
    ///   A commit that opens the editor gets none;
    /// a failure to start the thread leaves the commit as it was,
    /// without a notice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const notice = NativeCommitNotice.start({ snapshot, commitArguments, cadence });
    /// ```
    pub fn start(snapshot: Option<&ConfigSnapshot>, commit_arguments: &[String], cadence: Cadence, mut sink: Box<dyn Write + Send>) -> Self {
        let words: Vec<std::ffi::OsString> = commit_arguments.iter().map(std::ffi::OsString::from).collect();
        if opens_editor(words.as_slice()) {
            return Self::silent();
        }
        let signs: bool = signs_commit(snapshot, commit_arguments);
        let (stop, ended) = channel::<()>();
        let began: Instant = Instant::now();
        let spawned = std::thread::Builder::new().name(String::from("native-commit-notice")).spawn(move || {
            let mut ticker: Ticker = Ticker::new(cadence);
            let mut first: bool = true;
            loop {
                let waited: Duration = began.elapsed();
                if ticker.due(waited) {
                    let line: String = notice_line(signs, waited, first, programs_below().as_slice());
                    first = false;
                    // A closed terminal is not a reason to stop the commit.
                    let _written = sink.write_all(line.as_bytes());
                    let _flushed = sink.flush();
                }
                match ended.recv_timeout(ticker.next_in(began.elapsed()).max(LOOK_PAUSE)) {
                    Err(RecvTimeoutError::Timeout) => {}
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
                }
            }
        });
        return match spawned {
            Ok(thread) => Self { stop: Some(stop), thread: Some(thread) },
            Err(_) => Self::silent(),
        };
    }

    /// Ends the notice once the step has ended.
    pub fn finish(mut self) {
        if let Some(stop) = self.stop.take() {
            let _sent = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _joined = thread.join();
        }
    }
}

/// The line and the signing question are controlled in the test module.
#[cfg(test)]
#[path = "native_commit_waiting_notice_tests.rs"]
mod tests;
