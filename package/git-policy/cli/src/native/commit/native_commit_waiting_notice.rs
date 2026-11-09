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
//! the same reading `look_below` already makes.
//! A commit that opens the person's editor prints nothing and is never stopped,
//! because the person is the wait.
//! The ceiling is counted from the first look that found the step below this process,
//! never from when the notice started (`LOOK_INTERVAL`):
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
use crate::process_tree::{Delivered, descendants_of_read, kill_members};
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

/// The longest the notice goes between two looks at what runs below this process.
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
///  The same look is what the record's `scratchSteps` reports,
/// so this is also how finely the time of a program below the step is known.
///  A look reads the kernel's per-thread child lists and one `/proc/<pid>/stat` per descendant
/// (`look_below`),
/// which cost 19.6 microseconds on 2026-11-09 on this host (1,210 entries,
/// load average 44 to 91 on 6 threads around the runs);
/// the pass over `/proc` it falls back to where the kernel has no child lists took 17.6 ms at the median and 19.7 ms at most over 30 looks.
/// A quarter of a second keeps a step that is started late seen within that much of its start,
/// and a step that ends sooner than that is ended by its own channel before a second look.
const LOOK_INTERVAL: Duration = Duration::from_millis(250);

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
/// const below = lookBelow(); // [{ pid, name, state, cpu }], undefined when the look failed
/// ```
fn look_below() -> Option<Vec<(i32, ProcessUse)>> {
    let own: i32 = i32::try_from(std::process::id()).unwrap_or(0);
    let numbers: Vec<i32> = match descendant_numbers_from_thread_lists(own) {
        Some(found) => found,
        None => descendants_of_read(own)?.iter().map(member_number).collect(),
    };
    let mut found: Vec<(i32, ProcessUse)> = Vec::new();
    for pid in numbers.iter().copied() {
        if let Some(used) = use_of(pid)
            && used.state != 'Z'
        {
            found.push((pid, used));
        }
    }
    // Every process the walk named that could not be read is a look that failed,
    // as long as none of them could be read at all.
    if !numbers.is_empty() && found.is_empty() {
        return None;
    }
    return Some(found);
}

/// What:
///  What the ceiling came to when it came.
/// Why:
///   Three answers the notice must keep apart:
/// the step ended by itself, so the commit lands;
/// a signal was really sent, so the step was stopped;
/// and the look that came with the ceiling failed, so nothing is decided and the ceiling stays armed.
///  Treating a failed look as an empty one takes the ceiling away from a step that is still running,
/// which is how a commit lands after a long step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CeilingCame = 'the-step-ended' | 'stopped';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CeilingCame {
    /// Nothing is below this process any more: the step ended by itself.
    TheStepEnded,
    /// The kernel took a signal for the step or a program below it.
    Stopped,
}

/// What:
///  End what the ceiling's look found below this process, and answer what that came to.
/// Why:
///   A stop is recorded only when a signal was really sent:
/// a step that ended between the look and the signal landed,
/// and refusing it as stopped would fail a commit that succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const came = ceilingCame(look);
/// ```
fn ceiling_came(look: Option<&[TreeMember]>) -> CeilingCame {
    let Some(members) = look else {
        return CeilingCame::TheStepEnded;
    };
    if members.is_empty() {
        return CeilingCame::TheStepEnded;
    }
    // Everything below this process is the step and its signer; nothing of the person's runs there.
    let delivered: Vec<Delivered> = kill_members(members);
    if delivered.contains(&Delivered::Sent) {
        return CeilingCame::Stopped;
    }
    return CeilingCame::TheStepEnded;
}

/// The number of one member of a tree.
fn member_number(member: &TreeMember) -> i32 {
    return member.pid;
}

/// What:
///  One process a look saw below the step,
/// as the next look needs it:
/// the number and the start time that prove it is the same process,
/// and the name the record keeps.
/// Why:
///   Two programs can carry one name,
/// so the name alone cannot say that the program a look credited was below the step at both ends of the interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SeenBelow = { pid: number; started: number; name: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct SeenBelow {
    /// The process number.
    pid: i32,
    /// The start time the kernel recorded for it.
    started: u64,
    /// The kernel's name for it.
    name: String,
}

/// What:
///  The processes one look saw below the step,
/// numbered once each.
/// Why:
///   The next look credits a program with the interval only when it was there at both ends,
/// so this is what the look is remembered by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seen = below.map(([pid, used]) => ({ pid, started: used.started, name: used.name }));
/// ```
fn seen_below(below: &[(i32, ProcessUse)]) -> Vec<SeenBelow> {
    return below
        .iter()
        .map(|(pid, used)| { return SeenBelow { pid: *pid, started: used.started, name: used.name.clone() }; })
        .collect();
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
///  Credit the time since the previous look to every program that was below the step at both looks,
/// and keep every program the look saw that cannot be credited at all.
/// Why:
///   The kernel records no start and no end of a short program that the notice could read,
/// so it measures what it can:
/// a program that starts and ends between two looks is not seen at all,
/// and the time an entry holds is a lower bound of the time that program ran.
///  Only the same process at both looks is credited:
/// a name is not an identity,
/// and a program that took another's name between the looks ran in neither interval it would be credited with.
///  A program seen once is kept with a zero lower bound,
/// so the record says it was below the step even when no interval can be measured for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [pid, used] of now) facts.credit(used.name, seenAtBothLooks(before, pid, used.started) ? sinceLastLook : 0);
/// ```
fn credit_between_looks(facts: &mut ScratchStepFacts, before: &[SeenBelow], now: &[(i32, ProcessUse)], interval: Duration) {
    let mut credited: Vec<&str> = Vec::new();
    for (pid, used) in now {
        if credited.contains(&used.name.as_str()) {
            continue;
        }
        let at_both_looks: bool = before.iter().any(|known| { return known.pid == *pid && known.started == used.started; });
        facts.credit(used.name.as_str(), if at_both_looks { interval } else { Duration::ZERO });
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
            // The processes the previous look saw below the step, and when that look was.
            let mut before: Vec<SeenBelow> = Vec::new();
            let mut last_look: Duration = Duration::ZERO;
            let own: i32 = i32::try_from(std::process::id()).unwrap_or(0);
            loop {
                let waited: Duration = began.elapsed();
                // One look: the step, the programs below it, and what they are using.
                let look: Option<Vec<(i32, ProcessUse)>> = look_below();
                let below: &[(i32, ProcessUse)] = look.as_deref().unwrap_or_default();
                facts.looks = facts.looks.saturating_add(1);
                if !below.is_empty() {
                    facts.looks_found = facts.looks_found.saturating_add(1);
                }
                if seen_at.is_none() && !below.is_empty() {
                    seen_at = Some(waited);
                }
                credit_between_looks(&mut facts, before.as_slice(), below, waited.saturating_sub(last_look));
                before = seen_below(below);
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
                    match ceiling_came(descendants_of_read(own).as_deref()) {
                        CeilingCame::Stopped => {
                            facts.stopped_after = Some(waited);
                            return facts;
                        }
                        CeilingCame::TheStepEnded => return facts,
                    }
                }
                if ticker.due(waited) {
                    let line: String = notice_line(signs, waited, first, other_programs(below).as_slice(), ceiling);
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
                    Some(_seen) => to_next.min(ceiling.saturating_sub(since_seen)).min(LOOK_INTERVAL),
                    None => to_next.min(LOOK_INTERVAL),
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
