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
//! "limit unless editor"):
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
//! The ceiling is counted from the first look that found the step below this process,
//! never from when the notice started (`SIGHTING_INTERVAL`):
//! a start this host held up would otherwise pass the ceiling before the step existed,
//! and a look that finds nothing would then end the notice and the ceiling with it,
//! which is how a commit landed after a 36,651 ms step under a 3,006 ms ceiling on 2026-11-08.
//! Once the step has been seen,
//! a look that finds nothing means it ended by itself,
//! and the commit lands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = startNotice({ signs, cadence, ceiling }); try { await runGitCommit(); } finally { const stoppedAfter = notice.finish(); }
//! ```

/// The environment variable that sets the ceiling of the step in whole seconds.
pub const SCRATCH_CEILING_VARIABLE: &str = "CLI_GIT_SCRATCH_COMMIT_CEILING_SECONDS";

/// The whole seconds a setting of `SCRATCH_CEILING_VARIABLE` names.
fn seconds_in(text: String) -> Option<u64> {
    return text.trim().parse::<u64>().ok();
}

/// The ceiling of the step:
/// `SCRATCH_CEILING_VARIABLE` when it names a positive number of seconds,
/// else `SCRATCH_COMMIT_STEP_CEILING`.
pub fn scratch_ceiling() -> Duration {
    return match std::env::var(SCRATCH_CEILING_VARIABLE).ok().and_then(seconds_in) {
        Some(seconds) if seconds > 0 => Duration::from_secs(seconds),
        _ => crate::wait_ceilings::SCRATCH_COMMIT_STEP_CEILING,
    };
}

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
/// One member of a process tree.
use crate::process_tree::TreeMember;

/// The shortest pause between two looks of the notice thread,
/// so a very short interval does not spin.
const LOOK_PAUSE: Duration = Duration::from_millis(1);

/// The longest the notice goes between two looks for the step below this process,
/// until it has found it.
/// Why:
///   The ceiling is counted from the first look that found the step,
/// so a start this host held up (a full process table,
/// memory pressure,
/// a stall in starting a process) cannot take the ceiling away:
/// a look that finds nothing is a statement about the start,
/// never about the step.
///  That reading is the one the record could not make for a commit whose step ran 36,651 ms under a
///  3,006 ms ceiling on 2026-11-08:
/// it landed,
/// so the ceiling never ended it.
///  A look reads every `/proc` entry (`descendants_of`):
/// measured on 2026-11-09 on this host (1,210 entries,
/// load average 44 to 91 on 6 threads around the runs),
/// 17.6 ms at the median and 19.7 ms at most over 30 looks.
/// A quarter of a second keeps a step that is started late seen within that much of its start,
/// and a step that ends sooner than that is ended by its own channel before a second look.
const SIGHTING_INTERVAL: Duration = Duration::from_millis(250);

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
         message came with the command may take ({} seconds, three times the slowest `git commit` step this repository's commit timing log holds); nothing was \
         committed, and your index and files are as they were.{signing} Run the same `git commit` command again.\n",
        waited.as_secs(),
        ceiling.as_secs()
    );
}

/// What the notice saw while the step ran,
/// and the ledger the commit's timing record takes it from.
use crate::scratch_commit_step::ScratchStepFacts;
/// The kernel's name,
/// state and processor time of one process.
use crate::process_tree::ProcessUse;
/// What:
///   the process numbers below this one,
/// from the kernel's per-thread child lists.
use crate::process_tree::descendant_numbers_from_thread_lists;
/// What:
///   one process's name,
/// state and processor time,
/// from one `/proc/<pid>/stat`.
use crate::process_tree::use_of;

/// The running notice:
/// the thread that prints it and the sender that ends it.
pub struct NativeCommitNotice {
    /// Ends the thread when it is sent to or dropped.
    stop: Option<Sender<()>>,
    /// The thread,
    /// joined when the step has ended;
    /// it answers what it saw while the step ran.
    thread: Option<std::thread::JoinHandle<ScratchStepFacts>>,
    /// What is reported when no thread ran,
    /// so a record still says that no ceiling was armed and why.
    unarmed: ScratchStepFacts,
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

/// What:
///  Every living process below this one,
/// with its name,
/// state and processor time.
/// Why:
///   One read of one small file per process,
/// from the kernel's per-thread child lists,
/// so a notice that looks many times stays cheap:
/// a walk costs 19.6 microseconds and a pass over `/proc` 17.6 ms on 2026-11-09 (`process_tree.rs`).
///  `/proc` itself is read only where the kernel has no child lists.
///  A process the kernel holds as a zombie is not below the step any more,
/// as `descendants_of` counts it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const below = lookBelow(); // [{ pid, name, state, cpu }]
/// ```
fn look_below() -> Vec<(i32, ProcessUse)> {
    let own: i32 = i32::try_from(std::process::id()).unwrap_or(0);
    let numbers: Vec<i32> = match descendant_numbers_from_thread_lists(own) {
        Some(found) => found,
        None => descendants_of(own).iter().map(member_number).collect(),
    };
    let mut found: Vec<(i32, ProcessUse)> = Vec::new();
    for pid in numbers {
        if let Some(used) = use_of(pid)
            && used.state != 'Z'
        {
            found.push((pid, used));
        }
    }
    return found;
}

/// The number of one member of a tree.
fn member_number(member: &TreeMember) -> i32 {
    return member.pid;
}

/// What:
///  The names one look saw below the step,
/// once each.
/// Why:
///   The next look credits a program with the interval only when it was there at both ends,
/// so the names are what the look is remembered by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = [...new Set(below.map(([, used]) => used.name))];
/// ```
fn names_seen(below: &[(i32, ProcessUse)]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for (_pid, used) in below {
        if !names.contains(&used.name) {
            names.push(used.name.clone());
        }
    }
    return names;
}

/// The processor time one process below the step has used.
fn process_cpu(entry: &(i32, ProcessUse)) -> Duration {
    return entry.1.cpu;
}

/// Whether one process below the step is in uninterruptible sleep.
fn process_is_blocked(entry: &(i32, ProcessUse)) -> bool {
    return entry.1.state == 'D';
}

/// The programs below the step other than Git itself,
/// which the notice line names.
fn other_programs(below: &[(i32, ProcessUse)]) -> Vec<(i32, String)> {
    let mut others: Vec<(i32, String)> = Vec::new();
    for (pid, used) in below {
        if used.name != "git" {
            others.push((*pid, used.name.clone()));
        }
    }
    return others;
}

/// What:
///  Credit the time since the previous look to every program that was below the step at both looks.
/// Why:
///   The kernel records no start and no end of a short program that the notice could read,
/// so it measures what it can:
/// a program that starts and ends between two looks is not seen at all,
/// and the time an entry holds is a lower bound of the time that program ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of namesAtBothLooks) facts.credit(name, sinceLastLook);
/// ```
fn credit_between_looks(facts: &mut ScratchStepFacts, before: &[String], now: &[(i32, ProcessUse)], interval: Duration) {
    let mut credited: Vec<&str> = Vec::new();
    for (_pid, used) in now {
        if !before.contains(&used.name) || credited.contains(&used.name.as_str()) {
            continue;
        }
        facts.credit(used.name.as_str(), interval);
        credited.push(used.name.as_str());
    }
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
    fn silent(ceiling: Option<Duration>) -> Self {
        return Self { stop: None, thread: None, unarmed: ScratchStepFacts::unarmed(ceiling) };
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
            // The person is the wait, and a message typed into an editor would be lost by a stop.
            return Self::silent(None);
        }
        let signs: bool = signs_commit(snapshot, commit_arguments);
        let (stop, ended) = channel::<()>();
        let began: Instant = Instant::now();
        let spawned = std::thread::Builder::new().name(String::from("native-commit-notice")).spawn(move || {
            let mut ticker: Ticker = Ticker::new(cadence);
            let mut first: bool = true;
            let mut facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(ceiling));
            facts.armed = true;
            // When the step was first found below this process; the ceiling is counted from then.
            let mut seen_at: Option<Duration> = None;
            // The names the previous look saw below the step, and when that look was.
            let mut before: Vec<String> = Vec::new();
            let mut last_look: Duration = Duration::ZERO;
            loop {
                let waited: Duration = began.elapsed();
                // One look: the step, the programs below it, and what they are using.
                let below: Vec<(i32, ProcessUse)> = look_below();
                facts.looks = facts.looks.saturating_add(1);
                if !below.is_empty() {
                    facts.looks_found = facts.looks_found.saturating_add(1);
                }
                if seen_at.is_none() && !below.is_empty() {
                    seen_at = Some(waited);
                }
                credit_between_looks(&mut facts, before.as_slice(), below.as_slice(), waited.saturating_sub(last_look));
                before = names_seen(below.as_slice());
                last_look = waited;
                facts.below_cpu = below.iter().map(process_cpu).sum();
                if below.iter().any(process_is_blocked) {
                    facts.blocked_looks = facts.blocked_looks.saturating_add(1);
                }
                // How long the step has been running, and whether the ceiling has come, once the step has been seen.
                let since_seen: Duration = match seen_at {
                    Some(seen) => waited.saturating_sub(seen),
                    None => Duration::ZERO,
                };
                if seen_at.is_some() && since_seen >= ceiling {
                    // Everything below this process is the step and its signer; nothing of the person's runs there.
                    let members = descendants_of(i32::try_from(std::process::id()).unwrap_or(0));
                    if members.is_empty() {
                        // The step ended on its own after it was seen running.
                        return facts;
                    }
                    kill_members(members.as_slice());
                    facts.stopped_after = Some(waited);
                    return facts;
                }
                if ticker.due(waited) {
                    let line: String = notice_line(signs, waited, first, other_programs(below.as_slice()).as_slice(), ceiling);
                    first = false;
                    // A closed terminal is not a reason to stop the commit.
                    let writing: Instant = Instant::now();
                    let _written = sink.write_all(line.as_bytes());
                    let _flushed = sink.flush();
                    facts.wrote = facts.wrote.saturating_add(writing.elapsed());
                }
                // Wake for the sooner of the next line, the look, and the ceiling.
                let to_next: Duration = ticker.next_in(waited);
                let pause: Duration = match seen_at {
                    Some(_seen) => to_next.min(ceiling.saturating_sub(since_seen)).min(SIGHTING_INTERVAL),
                    None => to_next.min(SIGHTING_INTERVAL),
                };
                match ended.recv_timeout(pause.max(LOOK_PAUSE)) {
                    Err(RecvTimeoutError::Timeout) => {}
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => return facts,
                }
            }
        });
        return match spawned {
            Ok(thread) => Self { stop: Some(stop), thread: Some(thread), unarmed: ScratchStepFacts::unarmed(Some(ceiling)) },
            // The system refused the thread: the step runs with no ceiling, and the record says so.
            Err(_) => Self::silent(Some(ceiling)),
        };
    }

    /// Ends the notice once the step has ended,
    /// and answers what it saw:
    /// how long the step had lasted when the notice stopped it,
    /// how long the notice's own join took,
    /// and what ran below the step.
    /// Why:
    ///   The join is timed here because a notice line waits for its reader:
    /// a step whose figure held that wait would name the terminal as the work.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const began = now(); notice.stop(); const seen = notice.join(); seen.notice = now() - began; return seen;
    /// ```
    pub fn finish(mut self) -> ScratchStepFacts {
        let began: Instant = Instant::now();
        if let Some(stop) = self.stop.take() {
            let _sent = stop.send(());
        }
        let mut facts: ScratchStepFacts = match self.thread.take() {
            Some(thread) => thread.join().unwrap_or(self.unarmed),
            None => self.unarmed,
        };
        facts.notice = began.elapsed();
        return facts;
    }
}

/// The line and the signing question are controlled in the test module.
#[cfg(test)]
#[path = "native_commit_waiting_notice_tests.rs"]
mod tests;
