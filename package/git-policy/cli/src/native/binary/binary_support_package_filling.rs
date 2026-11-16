//! What:
//!  The filling a new worktree's copied build state is given by the commit service,
//!  and the ending a control may give it.
//! Why:
//!   The add stages the worktree's copied state and hands the filling to the service,
//! which cleans the worktree's own crates out of a staging copy and publishes it (`package_repair_prewarm_job.rs`).
//! A filling whose clean is ended before it takes anything keeps nothing,
//! and at the gate of 2026-11-15T06:00-05:00 one was ended by a signal 9 from outside the tool's own runner
//! (`real_cargo_leaves_a_stale_lockfile_as_checked_out`).
//!  A control cannot produce that ending by waiting: the runner words its own ceiling and its own quiet window as
//! `CeilingReached` and `TimedOut`, and only a signal it did not send is worded `stopped by signal <n>`
//! (`package_repair_command.rs`, `run_tool_reaping_started_within`).
//! So the one ending a control needs a stand-in for is here,
//! beside the reader that says how a filling ended,
//! and it is asked for by name, so a run that does not ask for it never meets it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = process.env.CLI_GIT_TEST_ONLY_FILL_CLEAN; // 'killed' in the runs that end a filling's clean
//! ```

use crate::support_fixture_service::Fixture;

/// What:
///  The ending a filling's clean is given, when a control asks for one.
/// Why:
///   The runner words a tool that ended by a signal from outside its own ceiling and quiet window as
/// `stopped by signal <n>`,
/// which is how the gate's own line words the clean of the filling it could not finish,
/// and no wait of a control reaches that wording.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_FILLS_CLEAN_ENDING_VARIABLE = 'CLI_GIT_TEST_ONLY_FILL_CLEAN';
/// ```
pub const A_FILLS_CLEAN_ENDING_VARIABLE: &str = "CLI_GIT_TEST_ONLY_FILL_CLEAN";

/// The one ending a control may name: the clean ends itself with `SIGKILL`,
/// which the runner reports exactly as it reports a signal from outside its own bounds.
pub const ENDED_WITH_SIGKILL: &str = "killed";

/// The other ending a control may name: the filling's clean works for `A_SLOW_FILLS_CLEAN_SECONDS` before it is the real
/// Cargo.
/// Why:
///  A filling that fails at once settles at once, so it cannot say whether a control waits for a filling that is still
/// working. One that works for seconds can: a run of the control that reads the displaced bytes only after the sleep is a
/// run whose wait covers a filling in progress,
/// which is what tells the control's own teardown apart from the machine as the sender of a signal at the gate.
pub const SLOW_FILLS_CLEAN: &str = "slow";

/// How long a filling's clean is made to work when a run asks for [`SLOW_FILLS_CLEAN`].
///  Eight seconds is more than every other step of the control takes together, so a run that waited for it is told apart
/// from one that read the folder while the filling still worked.
pub const A_SLOW_FILLS_CLEAN_SECONDS: u32 = 8;

/// What:
///  How many seconds a reader waits for the service's own words about a filling it did not finish.
/// Why:
///   The service writes that line before it answers the record's verdict,
/// and the line reaches the fixture's log through a thread that copies the service's error stream,
/// so a control that reads the log the moment the record settles can be a moment early.
/// Five seconds is far past a copy the machine is not stalled in, and a wait that ends without the line still says
/// what the record came to.
pub const A_FILLINGS_WORDS_PATIENCE: std::time::Duration = std::time::Duration::from_secs(5);

/// What:
///  The sentence a control fails with when the worktree's filling did not run to its end,
///  or nothing when it did.
/// Why:
///   A filling that did not run to its end never displaced a byte,
/// so a control that reads the bytes a clean displaced must read first whether the filling ran:
/// at the gate of 2026-11-15T06:00-05:00 this control failed with
/// `the bytes the clean displaced were not kept: []`
/// for a filling that was still in the queue when it looked,
/// and the fill's own reason, which the service wrote a moment later, never reached the person.
///  So a record the queue still holds is waited for, within `SERVICE_WORK_PATIENCE`,
/// because the states that mean "still to come" are not the three the commit cases wait on:
/// a filling the drain holds stands in `held` while it runs,
/// and a control that counted only `pending`, `claimed` and `recovering` read an empty folder in that window.
/// `started` is the moment the add that wrote the record was taken up,
/// which is what tells this fill's record from an earlier filling of the same path.
/// The sentence then names where the record stands and quotes the service's words about this worktree,
/// which is the whole of what a reader needs to know why nothing is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = aFillingThatDidNotRunToItsEnd(fixture, gitDir, worktree, startedAt); // undefined when the record settled in done
/// ```
pub fn a_filling_that_did_not_run_to_its_end(
    fixture: &Fixture,
    git_dir: &std::path::Path,
    worktree: &std::path::Path,
    started: std::time::SystemTime,
) -> Option<String> {
    return a_filling_that_did_not_run_to_its_end_watching(fixture, git_dir, worktree, started, &mut |_state: &str| {});
}

/// What:
///  `a_filling_that_did_not_run_to_its_end`,
///  telling `watching` the name of each state of the queue as the look reaches it.
/// Why:
///   A record that moves between two states of one look is a moment no control can hit by waiting,
///  and it is the moment this reader exists to survive,
/// so a control is told each state as it is looked at and moves the record then.
/// Every run of the reader that is not a control passes a closure that does nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = aFillingThatDidNotRunToItsEndWatching(fixture, gitDir, worktree, startedAt, (state) => { if (state === 'recovering') settle(); });
/// ```
pub fn a_filling_that_did_not_run_to_its_end_watching(
    fixture: &Fixture,
    git_dir: &std::path::Path,
    worktree: &std::path::Path,
    started: std::time::SystemTime,
    watching: &mut dyn FnMut(&str),
) -> Option<String> {
    let work: std::path::PathBuf = git_policy_cli::service_work::work_directory(git_dir);
    let began: std::time::Instant = std::time::Instant::now();
    // A record the queue still holds is a filling that may yet run: it is waited for, and a wait that ends with the record
    //  still open says so instead of claiming a verdict it did not see.
    let standing: Standing = loop {
        let first: Standing = look(work.as_path(), worktree, started, watching);
        let second: Standing = look(work.as_path(), worktree, started, watching);
        // What the queue has settled is an answer, and so is a queue that answered the same twice, because a record found
        //  nowhere is only believed when two looks agree: a record that moved while the first look read the states is read
        //  again by the second.
        let answered: bool = matches!(second, Standing::Done | Standing::GivenUp(_)) || (first == second && matches!(second, Standing::Nowhere));
        if answered || began.elapsed() >= crate::support_observation::SERVICE_WORK_PATIENCE {
            break second;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    if matches!(standing, Standing::Done) {
        return None;
    }
    let where_it_stands: String = match standing {
        Standing::Done => String::from(git_policy_cli::work_item::DONE),
        Standing::GivenUp(state) => String::from(state),
        Standing::Open(state) => format!("{state}, where it was still to come after the wait for its filling"),
        Standing::Nowhere => String::from(NAMED_NOWHERE),
    };
    let said: String = match the_services_words(fixture, worktree) {
        Some(words) => format!("the service said: {words}"),
        None => String::from("the service wrote nothing about it in its log"),
    };
    return Some(format!(
        "the filling of {} did not run to its end, so no byte was ever displaced for this control to read: its record stands in {where_it_stands} of {}, \
         whose queue holds {}, and {said}. The build directory the filling publishes was left as it was carried, and the next add fills it",
        worktree.display(),
        work.display(),
        crate::support_observation::work_listing(work.as_path()),
    ));
}

/// What:
///  Where one worktree's filling stands in its queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Standing {
    /// Its record settled in `done`: the service ran the filling to its end.
    Done,
    /// Its record stands where the queue keeps work it gave up on, named.
    GivenUp(&'static str),
    /// Its record stands in a state that may yet run, named.
    Open(&'static str),
    /// No record of this worktree's filling is in the queue at all.
    Nowhere,
}

/// The word a filling's record is given when no state of the work directory holds it.
/// It is not a state of the queue: no record there means the add handed nothing over, which is its own answer.
const NAMED_NOWHERE: &str = "no state of the work directory";

/// The states of the queue in the order a record moves through them:
/// `pending` to `recovering` to `claimed`, and from there to `done` or `failed`,
/// with `held` beside a claim for the work a gate holds.
/// The order is what keeps a record that moves during one look from being missed:
/// a record only moves on, so it is met by a state read after the one it left.
const WHERE_A_RECORD_MOVES: [&str; 6] = [
    git_policy_cli::work_item::PENDING,
    git_policy_cli::work_item::RECOVERING,
    git_policy_cli::work_item::CLAIMED,
    git_policy_cli::work_item::HELD,
    git_policy_cli::work_item::DONE,
    git_policy_cli::work_item::FAILED,
];

/// What:
///  Where this worktree's filling stands, read in one pass through the states a record moves through.
/// Why:
///   One pass answers what the queue holds now,
/// and reading the states in the order records move through them is what makes a record that moves during the pass
///  meet a state the pass has not read yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = states.find((state) => holds(work, state, worktree, started)) ?? 'nowhere';
/// ```
fn look(work: &std::path::Path, worktree: &std::path::Path, started: std::time::SystemTime, watching: &mut dyn FnMut(&str)) -> Standing {
    for state in WHERE_A_RECORD_MOVES {
        if !holds_the_record(work, state, worktree, started, watching) {
            continue;
        }
        return match state {
            git_policy_cli::work_item::DONE => Standing::Done,
            git_policy_cli::work_item::FAILED => Standing::GivenUp(git_policy_cli::work_item::FAILED),
            open => Standing::Open(open),
        };
    }
    return Standing::Nowhere;
}

/// What:
///  Whether the work directory holds this worktree's filling in `state` now,
///  of the records the add taken up at `started` wrote.
/// Why:
///   The record of a filling is a regular file holding the work item,
/// and only a prewarm of this worktree whose stamp is not older than `started` is this fill's:
/// a worktree removed and added again at one path leaves the earlier filling's record in the queue,
/// and what became of that fill is no answer about this one.
///  Only a regular file is read: a queue directory is a place anything with write access can put anything,
/// and opening a pipe for reading waits for a writer that may never come,
/// which no deadline of this reader can end because the open happens inside one look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holds = entries(stateDir).some((e) => e.isFile() && (r = decode(read(e))) && r.kind === 'prewarm' && r.repository === worktree && Date.parse(r.created) >= started);
/// ```
fn holds_the_record(
    work: &std::path::Path,
    state: &str,
    worktree: &std::path::Path,
    started: std::time::SystemTime,
    watching: &mut dyn FnMut(&str),
) -> bool {
    let directory: std::path::PathBuf = git_policy_cli::work_item::state_directory(work, state);
    let found: bool = std::fs::read_dir(directory).ok().is_some_and(|entries| {
        return entries.flatten().any(|entry| {
            let Ok(kind) = entry.file_type() else { return false };
            if !kind.is_file() {
                return false;
            }
            let Ok(text) = std::fs::read_to_string(entry.path()) else { return false };
            let Ok(item) = git_policy_cli::work_item::WorkItem::decode(text.as_str()) else { return false };
            if item.kind != git_policy_cli::work_item::WorkKind::Prewarm || item.repository != worktree {
                return false;
            }
            let not_older: std::time::SystemTime = the_millisecond_of(started);
            return git_policy_cli::iso_time::instant_of_iso(item.created.as_str()).is_some_and(|created| { return created >= not_older; });
        });
    });
    watching(state);
    return found;
}

/// What:
///  The instant `moment` names, cut down to the millisecond a record's stamp is written in.
/// Why:
///   A record's `created` is written in whole milliseconds,
/// and a moment read from the clock carries finer parts,
/// so a record written in the same millisecond as the add was taken up names an instant a fraction of a millisecond before it:
/// compared as it stands, the fill's own record would be older than the moment its add was taken up,
///  and the reader would answer for an earlier filling's record or for none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const toTheMillisecond = (moment) => new Date(Math.floor(moment.getTime() / 1000) * 1000 + Math.floor(moment.getMilliseconds()));
/// ```
fn the_millisecond_of(moment: std::time::SystemTime) -> std::time::SystemTime {
    let Ok(since_the_epoch) = moment.duration_since(std::time::UNIX_EPOCH) else { return moment };
    let millis: u64 = u64::try_from(since_the_epoch.as_millis()).unwrap_or(u64::MAX);
    return std::time::UNIX_EPOCH + std::time::Duration::from_millis(millis);
}

/// What:
///  The last thing the service wrote about this worktree's filling,
///  waited for within a bound.
/// Why:
///   The service writes that line before it answers the record's verdict,
/// and the line reaches the fixture's log through a thread that copies the service's error stream,
/// so a reader that looks the moment the record settles can be a moment early.
/// The line names the build directory it could not fill, so the worktree's own path is what tells this filling's
///  line from those of every other worktree the same service filled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = lines(serviceLog(root)).filter((line) => line.includes('could not be filled') && line.includes(worktree)).at(-1);
/// ```
fn the_services_words(fixture: &Fixture, worktree: &std::path::Path) -> Option<String> {
    let needle: String = worktree.display().to_string();
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        let log: String = crate::support_fixture_service_log::service_log(fixture.root.as_path());
        let found: Option<String> = log
            .lines()
            .rev()
            .find(|line| { return line.contains("could not be filled") && line.contains(needle.as_str()); })
            .map(str::to_owned);
        if found.is_some() || began.elapsed() >= A_FILLINGS_WORDS_PATIENCE {
            return found;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// What:
///  Give a filling's clean the ending the run asked for,
///  or leave the fixture's Cargo alone.
/// Why:
///   The naming of the ending is the environment's,
/// and the work is a named function so a control can ask for an ending without setting a variable of the whole process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.env.CLI_GIT_TEST_ONLY_FILL_CLEAN === 'killed') endTheFillsClean(fixture);
/// ```
pub fn end_the_fills_clean(fixture: &Fixture) {
    let asked: Option<String> = std::env::var(A_FILLS_CLEAN_ENDING_VARIABLE).ok();
    end_the_fills_clean_within(fixture, asked.as_deref());
}

/// What:
///  Replace the fixture's link to the toolchain's Cargo with a stand-in that ends with `SIGKILL` when it is the filling's
///  own clean,
///  fails the resolution that follows it,
///  and is the real Cargo for everything else.
/// Why:
///   The gate's line names two ends in one filling:
/// a clean `stopped by signal 9`,
/// and then nothing displaced and nothing kept,
/// which is the resolution after it either failing or writing no lock (`package_repair_cargo_lock.rs`).
/// The stand-in does what those lines say in the order they say it,
/// so a control meets the state the gate met and not a state it invented:
/// the fill's clean ends by signal,
/// the resolution after that signals fails as a resolution on a machine that has just lost one of its tools,
///  and the checked-out lock is never displaced.
/// Every other run of Cargo in the fixture is the real one, and the fixture's own build happens before this link is
/// replaced at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const script = `case " $* " in *' clean '*) case "$CARGO_BUILD_BUILD_DIR" in *'.monochromatic-prewarm') : > '${marker}'; kill -9 $$ ;; esac ;; esac; exec '${real}' "$@"`;
/// ```
pub fn end_the_fills_clean_within(fixture: &Fixture, ending: Option<&str>) {
    let Some(asked) = ending else { return };
    let link: std::path::PathBuf = fixture.root.join("bin/cargo");
    let real: std::path::PathBuf = std::fs::read_link(link.as_path()).unwrap_or_else(|error| {
        panic!("the fixture's Cargo link at {} cannot be replaced, so the control asked for an ending of a filling's clean it never prepared: {error}", link.display());
    });
    let marker: std::path::PathBuf = fixture.root.join("filling-clean-ended");
    // A single quote in a path ends the quoted text, is written as itself, and starts it again.
    let quote = |path: &std::path::Path| -> String { return path.display().to_string().replace('\'', "'\\''"); };
    // What the filling's own clean does, and what the resolution after it does, which is nothing unless the clean was ended
    //  with the signal.
    let (clean, resolution): (String, String) = match asked {
        ENDED_WITH_SIGKILL => (
            format!(": > '{}'; kill -9 $$", quote(marker.as_path())),
            format!(
                "if [ -f '{0}' ]; then printf 'error: failed to resolve the workspace: the lock file was taken from under this resolution\\n' >&2; exit 101; fi",
                quote(marker.as_path())
            ),
        ),
        SLOW_FILLS_CLEAN => (format!("sleep {A_SLOW_FILLS_CLEAN_SECONDS}"), String::from(":")),
        other => panic!(
            "the endings a control may name for a filling's clean are {ENDED_WITH_SIGKILL} and {SLOW_FILLS_CLEAN}, and {A_FILLS_CLEAN_ENDING_VARIABLE} says {other:?}"
        ),
    };
    let script: String = format!(
        "#!/bin/sh\n\
         # The filling's own clean is ended or made to work, and the resolution after an ended clean fails: what the machine\n\
         #  did to the filling of the gate of 2026-11-15T06:00-05:00 did the same. Every other run is the real Cargo.\n\
         case \" $* \" in\n\
         \x20 *\" clean \"*)\n\
         \x20   case \"$CARGO_BUILD_BUILD_DIR\" in\n\
         \x20     *'.monochromatic-prewarm') {clean} ;;\n\
         \x20   esac\n\
         \x20   ;;\n\
         \x20 *\" metadata \"*)\n\
         \x20   {resolution}\n\
         \x20   ;;\n\
         esac\n\
         exec '{real}' \"$@\"\n",
        real = quote(real.as_path()),
    );
    std::fs::remove_file(link.as_path()).expect("the fixture's Cargo link goes before the stand-in takes its name");
    crate::support_fixture_directories::executable(link.as_path(), script.as_bytes());
}
