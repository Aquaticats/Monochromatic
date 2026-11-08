//! What:
//!  The waiting notice and the ceiling of the commit Git builds in the scratch repository:
//! while that `git commit` runs,
//! a line on standard error
//! says what the command waits for,
//! how long it has waited,
//! and when the step is stopped;
//! when the ceiling comes,
//! everything below this process is ended.
//! Why:
//!  Every other wait of the wrapper says what it waits for (issue 1811),
//! but this step runs with the terminal attached and no watch,
//! because an editor may open in it,
//! so a commit that spent four minutes in it printed nothing.
//! The line names the `git commit` the command started (its process),
//! says the commit is signed when `commit.gpgsign` or `-S` asks
//! for it,
//! and names the signing program while one runs below the `git commit`.
//! The owner's rule is that nothing waits for ever (2026-11-03),
//! and his answer on this step (2026-11-07,
//! "A: limit unless editor"):
//! a ceiling when the message came with the command,
//! none when an editor may open,
//! where the person is the wait and a stop would lose what they typed.
//! The ceiling is `SCRATCH_COMMIT_STEP_CEILING` (`service/wait_ceilings.rs`),
//! with its measurement.
//! A stop ends every process below this one with `SIGKILL`:
//! the step runs in a scratch repository on a private copy of the index,
//! and the objects it wrote are content-addressed files the next build writes again,
//! so nothing of the person's is in the step.
//! The thread that watches the step is the only caller that needs to find it,
//! because the step runs below `SignalGuard`,
//! which relays signals but names no child;
//! this is the only child of the command at that point,
//! the same reading `programs_below` already makes.
//! A commit that opens the person's editor prints nothing and is never stopped,
//! because the person is the wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = startNotice({ signs, cadence, ceiling }); try { await runGitCommit(); } finally { const stoppedAfter = notice.finish(); }
//! ```

/// The cadence and the clock of a wait notice.
use crate::wait_progress::{Cadence, Ticker};
/// The processes started below this one,
/// and their end.
use crate::process_tree::{descendants_of, kill_members};
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

/// What a started notice is given.
pub struct NoticeStart<'a> {
    /// The configuration the commit is built under.
    pub snapshot: Option<&'a ConfigSnapshot>,
    /// The commit command's own arguments.
    pub commit_arguments: &'a [String],
    /// The cadence of the lines.
    pub cadence: Cadence,
    /// How long the step lasts before it is stopped,
    /// when no editor may open in it.
    pub ceiling: Duration,
    /// Where the lines go.
    pub sink: Box<dyn Write + Send>,
}

/// What:
///  The words of a stop:
/// what ended,
/// after how long,
/// why this long,
/// what became of the person's work,
/// and what to run.
/// Why:
///   A commit that stops with a bare exit status leaves the person guessing whether anything was committed;
/// the step runs in a scratch repository,
/// so nothing was,
/// and the message is on the command line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopWords({ waited, ceiling, signs })
/// ```
pub fn stop_words(waited: Duration, ceiling: Duration, signs: bool) -> String {
    let signing: &str = if signs {
        " The commit signs the commit because `commit.gpgsign` or `-S` asks for it, so the signing program is the first thing to check."
    } else {
        ""
    };
    return format!(
        "monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after {} seconds, the most a commit whose \
         message came with the command may take ({} seconds, three times the longest build this repository's commit timing log holds); nothing was \
         committed, and your index and files are as they were.{signing} Run the same `git commit` command again.\n",
        waited.as_secs(),
        ceiling.as_secs()
    );
}

/// The running notice:
/// the thread that prints it and the sender that ends it.
pub struct NativeCommitNotice {
    /// Ends the thread when it is sent to or dropped.
    stop: Option<Sender<()>>,
    /// The thread,
    /// joined when the step has ended;
    /// it answers how long the step had lasted when it ended it,
    /// when it did.
    thread: Option<std::thread::JoinHandle<Option<Duration>>>,
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
pub fn notice_line(signs: bool, waited: Duration, first: bool, below: &[(i32, String)], ceiling: Duration) -> String {
    let signing: String = if signs { String::from(", which signs the commit because `commit.gpgsign` or `-S` asks for it") } else { String::new() };
    let signer: String = match below.first() {
        Some((pid, name)) => format!("; `{name}` (process {pid}) runs below it"),
        None => String::new(),
    };
    let hint: &str = if first { " Hint: it is still running; if it looks stuck, check it with `ps` and report it." } else { "" };
    return format!(
        "monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository{signing}{signer}, {} seconds so far; \
         it is stopped if it lasts {} seconds in all.{hint}\n",
        waited.as_secs(),
        ceiling.as_secs()
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
    pub fn start(snapshot: Option<&ConfigSnapshot>, commit_arguments: &[String], cadence: Cadence, sink: Box<dyn Write + Send>) -> Self {
        return Self::start_within(NoticeStart { snapshot, commit_arguments, cadence, ceiling: crate::wait_ceilings::SCRATCH_COMMIT_STEP_CEILING, sink });
    }

    /// Starts the notice with the ceiling given.
    pub fn start_within(start: NoticeStart<'_>) -> Self {
        let NoticeStart { snapshot, commit_arguments, cadence, ceiling, mut sink } = start;
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
                if waited >= ceiling {
                    // Everything below this process is the step and its signer; nothing of the person's runs there.
                    let below = descendants_of(i32::try_from(std::process::id()).unwrap_or(0));
                    if below.is_empty() {
                        // The step ended on its own as the ceiling came.
                        return None;
                    }
                    kill_members(below.as_slice());
                    return Some(waited);
                }
                if ticker.due(waited) {
                    let line: String = notice_line(signs, waited, first, programs_below().as_slice(), ceiling);
                    first = false;
                    // A closed terminal is not a reason to stop the commit.
                    let _written = sink.write_all(line.as_bytes());
                    let _flushed = sink.flush();
                }
                let to_ceiling: Duration = ceiling.saturating_sub(began.elapsed());
                match ended.recv_timeout(ticker.next_in(began.elapsed()).min(to_ceiling).max(LOOK_PAUSE)) {
                    Err(RecvTimeoutError::Timeout) => {}
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => return None,
                }
            }
        });
        return match spawned {
            Ok(thread) => Self { stop: Some(stop), thread: Some(thread) },
            Err(_) => Self::silent(),
        };
    }

    /// Ends the notice once the step has ended,
    /// and answers how long the step had lasted when the notice stopped it,
    /// nothing when it ended by itself.
    pub fn finish(mut self) -> Option<Duration> {
        if let Some(stop) = self.stop.take() {
            let _sent = stop.send(());
        }
        return match self.thread.take() {
            Some(thread) => thread.join().unwrap_or(None),
            None => None,
        };
    }
}

/// The line and the signing question are controlled in the test module.
#[cfg(test)]
#[path = "native_commit_waiting_notice_tests.rs"]
mod tests;
