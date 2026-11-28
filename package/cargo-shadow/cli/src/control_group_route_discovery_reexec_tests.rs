//! What:
//! The controls of the span the asking again covers:
//! a moment as long as the longest reexec measured on 2026-11-27,
//! planted by a stand-in,
//! and a real reexec of a container's own user manager,
//! across which every discovery must still answer (issue 6623).
//! Why:
//! `ASKING_AGAIN_ATTEMPTS` and `ASKING_AGAIN_PAUSE` are sized from a measurement of how long asks fail while a manager
//! reexecutes,
//! so one control holds that span against a moment of that length and another against a real reexec,
//! which no test may make against the manager of the machine the suite runs on.
//! The stand-in is a stand-in:
//! no claim about a real manager rests on it,
//! and the control that does rest on a real manager says so where it refuses to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('the span the question is asked again over', () => { it('rides through a moment as long as the measured reexec', ...) })
//! ```

/// What:
/// `use std::fs;` imports the write of a stand-in,
/// the read of its log and the check of a container's marker files.
/// Why:
/// Each control writes the program it plants,
/// reads what that program was asked,
/// and the second one asks whether it is inside a container at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { existsSync, readFileSync, writeFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports the trait that sets a file's mode.
/// Why:
/// A stand-in that is not executable is not on the way to the manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { chmodSync } from 'node:fs';
/// ```
use std::os::unix::fs::PermissionsExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The stand-in of a control and the log it writes are paths of that control's own scratch directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is a plain string in TS
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::process::Command;` imports the builder of a child process.
/// Why:
/// The second control runs the manager's own `daemon-reexec`,
/// and nothing else in this file starts a program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawnSync } from 'node:child_process';
/// ```
use std::process::Command;
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the monotonic clock.
/// Why:
/// Both controls measure how long the discovery took and where it fell against a window of time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds, and performance.now()
/// ```
use std::time::{
    Duration,
    Instant,
};

/// What:
/// `use crate::cgroup_test_support::{NOT_RUN_PREFIX, this_test_needs_a_delegated_scope};` imports the line a control prints
/// when it cannot run here and the check that this host can give a delegated scope.
/// Why:
/// A control that returns green without making its moment proves nothing,
/// so a host that cannot hold it reports it as not run,
/// by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { NOT_RUN_PREFIX, thisTestNeedsADelegatedScope } from 'crate/cgroup_test_support';
/// ```
use crate::cgroup_test_support::{
    NOT_RUN_PREFIX,
    this_test_needs_a_delegated_scope,
};
/// What:
/// `use crate::control_group::{RouteSettings, detect_route};` imports the discovery and the settings that replace the program it
/// asks.
/// Why:
/// Both controls judge the discovery's own question,
/// one through a program the control wrote and one through the real manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RouteSettings, detectRoute } from 'crate/control_group';
/// ```
use crate::control_group::{
    RouteSettings,
    detect_route,
};
/// What:
/// `use crate::control_group_route_discovery::{ASKING_AGAIN_ATTEMPTS, ASKING_AGAIN_PAUSE};` imports the two constants the span is
/// made of.
/// Why:
/// The control below holds them against the figures the records give,
/// which is the only place a lowered constant is caught:
/// a stand-in that refuses for the measured moment passes four attempts of a shorter wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ASKING_AGAIN_ATTEMPTS, ASKING_AGAIN_PAUSE } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::{
    ASKING_AGAIN_ATTEMPTS,
    ASKING_AGAIN_PAUSE,
};
/// What:
/// `use crate::test_scratch::scratch;` imports the disposable directory of one control.
/// Why:
/// The stand-in and its log live in a fixture of the control's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { scratch } from 'crate/test_scratch';
/// ```
use crate::test_scratch::scratch;

/// How long the stand-in of the first control refuses every ask:
/// the longest reexec across which asks failed,
/// measured on 2026-11-27.
/// What it rests on:
/// 0.588 s in a container of its own (Linux 7.3.5,
/// systemd 262,
/// 657 units loaded)
/// and 0.393 s in a guest (Linux 6.8,
/// systemd 255,
/// 760 units),
/// and what the round measured is in `doc/the-route-question-that-is-asked-again-when-one-ask-fails.md`.
/// It is the figure `ASKING_AGAIN_PAUSE` and `ASKING_AGAIN_ATTEMPTS` are sized from,
/// which is why the control plants exactly this moment.
const MEASURED_LONGEST_REEXEC: Duration = Duration::from_millis(600);

/// How many reexecs of the container's manager the second control asks across.
/// Why:
/// One reexec,
/// if it was short,
/// could pass between two discoveries and prove nothing;
/// three windows in one run make an overlap likely,
/// and the control still asserts that one happened rather than assuming it.
const REEXEC_COUNT: u32 = 3;

/// How long the second control waits before each reexec.
/// Why:
/// The discoveries must already be going when a window opens,
/// and the gap also keeps the windows apart so each one is its own moment.
const REEXEC_SPACING: Duration = Duration::from_millis(1100);

/// How long the second control keeps asking the question.
/// Why:
/// Long enough for `REEXEC_COUNT` windows at `REEXEC_SPACING` and a moment after the last one,
/// and short enough to be one test in a suite.
const CONTROL_SECONDS: Duration = Duration::from_secs(4);

/// What:
/// `fn a_program(directory: &Path, name: &str, text: &str) -> PathBuf` writes an executable stand-in into a control's directory
/// and answers its path.
/// Why:
/// The first control plants one program in place of `systemctl`,
/// and a stand-in says it is one where it is defined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aProgram(dir: string, name: string, text: string): string
/// ```
fn a_program(
    directory: &Path,
    name: &str,
    text: &str,
) -> PathBuf {
    // Typed binding: the path the stand-in is written to.
    let program: PathBuf = directory.join(name);
    fs::write(&program, text).expect("write the stand-in");
    // `&program`: lends the path, and the address of the mode it is given.
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
    // The stand-in's path, for the settings that ask through it.
    return program;
}

/// What:
/// `fn real_systemctl() -> String` is the first `systemctl` on `PATH`.
/// Why:
/// The stand-in asks the real manager for every ask after the moment it plants,
/// so the route the control finds is a real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function realSystemctl(): string
/// ```
fn real_systemctl() -> String {
    // The search path, as one string of directories.
    let search: String = std::env::var("PATH").unwrap_or_default();
    for directory in search.split(':') {
        if directory.is_empty() {
            continue;
        }
        // Typed binding: one candidate program.
        let candidate: PathBuf = Path::new(directory).join("systemctl");
        if crate::control_group::is_executable_file(&candidate) {
            return candidate.display().to_string();
        }
    }
    panic!("no systemctl is on PATH, so this host has no user manager to ask");
}

/// The control of the span:
/// a stand-in refuses every ask for `MEASURED_LONGEST_REEXEC` and runs the real manager for every ask after it,
/// so a discovery whose asking again spans that moment finds the route and one whose span is shorter does not.
#[test]
fn a_moment_as_long_as_the_longest_measured_reexec_is_asked_through() {
    this_test_needs_a_delegated_scope();
    // Typed binding: the fixture directory of this control.
    let directory = scratch();
    // `&directory.path()`: lends the fixture's path to each join.
    let log: PathBuf = directory.path().join("calls");
    // The moment the stand-in's first ask began, written by the stand-in itself, so the clock is one process's clock.
    let first_ask: PathBuf = directory.path().join("first-ask");
    // What:     `program` is the stand-in: it writes down every ask and refuses it until the moment has passed.
    // Why:      A refusal that lasts the measured window is what an ask meets while a manager reexecutes, and every ask
    //           after it must reach the real manager for the route to be found.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-refuses-through-a-reexec', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-refuses-through-a-reexec",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nnow=$(date +%s%N)\nif [ ! -e '{first_ask}' ]; then printf '%s\\n' \"$now\" > '{first_ask}'\nfi\nbegan=$(cat '{first_ask}')\nif [ $(( (now - began) / 1000000 )) -lt {window} ]; then\n  echo 'the stand-in refused an ask inside the measured moment' >&2\n  exit 1\nfi\nexec '{real}' \"$@\"\n",
            log = log.display(),
            first_ask = first_ask.display(),
            window = MEASURED_LONGEST_REEXEC.as_millis(),
            real = real_systemctl()
        ),
    );
    // Typed binding: the settings of this control, asking through the stand-in.
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };
    // The slice the discovery must report, kept apart because the settings are lent to the discovery below.
    let wanted: String = settings.slice.clone();

    // Typed binding: when the discovery began, so the moment it rode through can be read from the clock.
    let began: Instant = Instant::now();
    // `&settings`: lends the settings; `expect` turns a refusal into this control's failure with its own words.
    let route =
        detect_route(&settings).expect("a moment as long as the measured reexec is asked through");
    // Typed binding: how long the whole discovery took.
    let elapsed: Duration = began.elapsed();

    assert_eq!(
        route.slice, wanted,
        "the discovery did not find the slice the settings name"
    );
    assert!(
        elapsed >= MEASURED_LONGEST_REEXEC,
        "the discovery answered after {elapsed:?}, before the moment of {MEASURED_LONGEST_REEXEC:?} it must outlast"
    );
    // Typed binding: every ask the stand-in was given.
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    assert!(
        calls.lines().count() > 1,
        "the question was asked once only, so nothing was put again across the moment: {calls:?}"
    );
    println!(
        "the discovery rode through a moment of {:?} in {elapsed:?}, with {} asks",
        MEASURED_LONGEST_REEXEC,
        calls.lines().count()
    );
}

/// What:
/// `fn inside_a_container() -> bool` says whether this process is inside a container.
/// Why:
/// The control below reexecutes the user manager it finds,
/// and the account's own manager is not a test's to disturb:
/// a container's bus and manager are its own,
/// so the control runs only where the marker of one is present.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function insideAContainer(): boolean
/// ```
fn inside_a_container() -> bool {
    // What:     `/.dockerenv` and `/run/.containerenv` are the markers Docker and Podman leave inside a container.
    // Why:      They are the only cheap proof that the manager a test reexecutes is not this machine's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return existsSync('/.dockerenv') || existsSync('/run/.containerenv')
    // ```
    return Path::new("/.dockerenv").exists() || Path::new("/run/.containerenv").exists();
}

/// The control of a real reexec:
/// the manager of the container this test runs in is told to reexecute itself,
/// and every discovery made across it must answer.
/// It proves the span only where a discovery overlaps the reexec,
/// so it asserts that one did and prints how many.
#[test]
fn the_question_answers_across_a_real_reexec_of_a_containers_manager() {
    if !inside_a_container() {
        // A host run of the whole suite must not go red for a control that is a container's to make, and it must not
        // reexecute this machine's manager either, so it says what it did not do and ends. The command that makes the
        // container and runs this test in it is in the package's own document, and the record of a run there says what
        // the control answered.
        println!(
            "cargo-shadow tests: this control reexecutes a user manager, so it runs only inside a container whose manager and bus are its own, and this run is not one; the command is in `package/cargo-shadow/cli/doc/the-route-question-that-is-asked-again-when-one-ask-fails.md`"
        );
        return;
    }
    // Typed binding: the settings of this control, asking the real manager.
    let settings: RouteSettings = RouteSettings::default();
    // `&settings`: lends the settings to the first question, which must answer or this container cannot hold the control.
    let first = match detect_route(&settings) {
        Ok(found) => found,
        Err(unavailable) => {
            // Typed binding: the name the runner gave this test's thread.
            let test: String = std::thread::current()
                .name()
                .map(str::to_string)
                .unwrap_or_else(|| {
                    return "<a test the runner did not name>".to_string();
                });
            panic!(
                "{NOT_RUN_PREFIX}{test}]: the container this test runs in has no user manager of its own ({unavailable}), so there is no manager to reexecute; the command that builds one is in `package/cargo-shadow/cli/doc/the-route-question-that-is-asked-again-when-one-ask-fails.md`"
            );
        }
    };
    // The slice the manager reported before the reexec; every discovery across it must report the same.
    let wanted: String = first.slice.clone();
    // Typed binding: the program the reexec runs, kept as its own value so the thread does not borrow the settings.
    let systemctl: String = settings.systemctl.clone();

    // The reexecs run in their own thread, after a moment and then apart,
    // so the discoveries below are already going when each one begins.
    let reexec = std::thread::spawn(move || {
        // Typed binding: one window per reexec, as (began, took), in the order the reexecs ran.
        let mut windows: Vec<(Instant, Duration)> = Vec::new();
        for _ in 0..REEXEC_COUNT {
            std::thread::sleep(REEXEC_SPACING);
            // Typed binding: when this reexec was asked for.
            let began: Instant = Instant::now();
            let status = Command::new(&systemctl)
                .args(["--user", "daemon-reexec"])
                .status()
                .expect("run the manager's own daemon-reexec");
            assert!(
                status.success(),
                "the manager's own daemon-reexec failed: {status}"
            );
            // Typed binding: how long the manager's own command took, which is this window.
            let took: Duration = began.elapsed();
            windows.push((began, took));
        }
        return windows;
    });

    // Typed binding: every discovery this control made, as (began, took), in the order it made them.
    let mut discoveries: Vec<(Instant, Duration)> = Vec::new();
    // Typed binding: the moment the control's own loop ends, a fixed window so a reexec that hangs cannot hold it.
    let until: Instant = Instant::now() + CONTROL_SECONDS;
    while Instant::now() < until {
        // Typed binding: when this discovery began.
        let began: Instant = Instant::now();
        // `&settings`: lends the settings; the discovery's answer is judged below, so a refusal is kept as words.
        let answered = detect_route(&settings);
        // Typed binding: how long this discovery took, whether it answered or refused.
        let took: Duration = began.elapsed();
        match answered {
            // `Ok(...)`: the discovery answered, and the slice it names must be the one the first question found.
            Ok(found) => assert_eq!(
                found.slice, wanted,
                "a discovery made across the reexec named another slice than the one before it"
            ),
            // `Err(...)`: the discovery refused, which is the failure this control exists to catch.
            Err(unavailable) => {
                panic!(
                    "the discovery refused {took:?} after it began, inside a container whose manager was reexecuting: {unavailable}"
                );
            }
        }
        discoveries.push((began, took));
    }
    // Typed binding: the windows of the reexecs, now that the thread has ended.
    let windows: Vec<(Instant, Duration)> = reexec.join().expect("the reexec thread ends");
    // Typed binding: the discoveries that overlapped a window, which are the ones this control is evidence for.
    let overlapping: Vec<&(Instant, Duration)> = discoveries
        .iter()
        .filter(|(began, took)| {
            return windows.iter().any(|(window_began, window_took)| {
                return *began < *window_began + *window_took && (*began + *took) > *window_began;
            });
        })
        .collect();
    // Typed binding: the total time the manager spent reexecuting, for the line this control prints.
    let reexecuted: Duration = windows
        .iter()
        .map(|(_, took)| {
            return *took;
        })
        .sum();
    assert!(
        !overlapping.is_empty(),
        "no discovery overlapped a reexec, so this run is no evidence about the span: {} discoveries in {:?}, {} reexecs of {reexecuted:?} together",
        discoveries.len(),
        CONTROL_SECONDS,
        windows.len()
    );
    println!(
        "{} discoveries in {:?}, {} of them inside {} real reexecs of {reexecuted:?} together, every one answered",
        discoveries.len(),
        CONTROL_SECONDS,
        overlapping.len(),
        windows.len()
    );
}

/// The number of attempts the records name.
/// Why:
/// `README.md`,
/// "Development",
/// says the question is put "five times,
/// two hundred milliseconds apart",
/// and `doc/the-route-question-that-is-asked-again-when-one-ask-fails.md` gives the same figures beside the measurement
/// they were sized from.
/// A change here is a change to both records and to the measurement behind them.
const RECORDED_ATTEMPTS: u32 = 5;

/// The pause between two attempts the records name,
/// in milliseconds.
/// Why:
/// It is the second half of the same sentence,
/// and it is what makes the span of the asking again the figure the records give.
const RECORDED_PAUSE_MILLISECONDS: u64 = 200;

/// The control of the span's size:
/// the two constants are the figures the records name,
/// and the waits they make outlast the longest reexec measured across which asks fail.
/// Why it is a control and not a comment:
/// a constant lowered by one attempt or by a shorter pause leaves the stand-in control green
/// (a moment of 0.600 s is still asked through by four attempts 0.150 s apart,
/// in the asks' own run time),
/// so nothing else in this file would show the change,
/// and the README's "the span is spent" sentence and its cost would be about a span the code no longer spends.
#[test]
fn the_span_the_records_name_is_the_span_the_discovery_spends() {
    assert_eq!(
        ASKING_AGAIN_ATTEMPTS, RECORDED_ATTEMPTS,
        "the number of attempts is not the five the records name, so a measurement has to be made and both records changed"
    );
    assert_eq!(
        ASKING_AGAIN_PAUSE,
        Duration::from_millis(RECORDED_PAUSE_MILLISECONDS),
        "the pause between two attempts is not the two hundred milliseconds the records name"
    );
    // What:     `waits` is the time the discovery spends waiting between its attempts: one wait fewer than there are attempts.
    // Why:      The span the records give is the waits; the asks' own run time comes on top of it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const waits = ASKING_AGAIN_PAUSE * (ASKING_AGAIN_ATTEMPTS - 1)
    // ```
    let waits: Duration = ASKING_AGAIN_PAUSE * (ASKING_AGAIN_ATTEMPTS - 1);
    assert!(
        waits > MEASURED_LONGEST_REEXEC,
        "the waits of the asking again are {waits:?}, which does not outlast the {MEASURED_LONGEST_REEXEC:?} a manager was measured to refuse asks across on 2026-11-27"
    );
}

/// The control of the pause the call site really sleeps:
/// a stand-in refuses every ask and writes down when each one began and when it ended,
/// and the time from one ask's end to the next one's beginning must be at least `ASKING_AGAIN_PAUSE`.
/// Why it is a control and not a comment:
/// `the_span_the_records_name_is_the_span_the_discovery_spends` holds the two constants against the records,
/// and neither it nor any other control reads the sleep at the call site
/// (`std::thread::sleep` in `ask_where_the_slice_is`),
/// so a wait of `ASKING_AGAIN_PAUSE * 3 / 4` there leaves that control green
/// with a span of 0.6 s where the README and the package document name 0.800 s.
/// Measured 2026-11-27 by a same-vendor second read of this release,
/// which named that edit.
/// A gap measured this way holds the next stand-in's own start and the discovery's work between two programs as well as the
/// sleep,
/// so a host slow enough to spend a quarter of the pause there can hide the edit:
/// measured on this host on 2026-11-27,
/// the four gaps were 209.0 ms to 209.9 ms with the sleep in place,
/// and 174.6 ms,
/// 229.7 ms,
/// 358.7 ms and 460.3 ms with it at three quarters of the pause,
/// where the edit showed in one gap of the four.
/// A green is safe,
/// because a sleep never ends early;
/// a red needs one gap that the work around it did not stretch past the pause,
/// and every gap is printed so a reader sees which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // describe('the span the question is asked again over', () => { it('spends the pause the call site sleeps', ...) })
/// ```
#[test]
fn the_gap_between_two_asks_is_the_pause_the_call_site_sleeps() {
    this_test_needs_a_delegated_scope();
    // The fixture directory of this control.
    let directory = scratch();
    // Where the stand-in writes down the moment each ask began and the moment it ended.
    let log: PathBuf = directory.path().join("asks");
    // What:     `program` is the stand-in: it writes down the nanosecond clock at its start and at its end and refuses the ask.
    // Why:      The two moments are what a gap between two asks is measured from, and a stand-in that answered would let the first
    //           ask end the asking.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-that-refuses-and-writes-down-when', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-that-refuses-every-ask-and-writes-down-when",
        &format!(
            "#!/bin/sh\nprintf 'begin %s %s\\n' \"$(date +%s%N)\" \"$*\" >> '{log}'\necho 'the stand-in refuses every ask' >&2\nprintf 'end %s\\n' \"$(date +%s%N)\" >> '{log}'\nexit 1\n",
            log = log.display()
        ),
    );
    // The settings of this control, asking through the stand-in.
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };

    // What:     `refused` is the discovery's failure, which a host whose every ask fails must be.
    // Why:      An answer here would mean the stand-in let an ask through, and the gaps below would be no record of the asking.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let refused: string
    // try { detectRoute(settings); throw new Error('the stand-in answered') } catch (error) { refused = String(error) }
    // ```
    let refused = detect_route(&settings)
        .expect_err("a stand-in that refuses every ask is a missing capability");
    assert!(
        refused
            .to_string()
            .contains("the stand-in refuses every ask"),
        "the refusal is not the last ask's own words: {refused}"
    );
    // Every line the stand-in wrote, one per moment.
    let written: String = fs::read_to_string(&log).expect("the stand-in's log");
    // What:     `wanted` is the attempt count as the width a count of lines answers in.
    // Why:      The count of attempts is a `u32` and a count of lines is a `usize`, and the two are compared below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const wanted = ASKING_AGAIN_ATTEMPTS
    // ```
    let wanted: usize = usize::try_from(ASKING_AGAIN_ATTEMPTS)
        .expect("an attempt count that fits a count of lines");
    // The moments the asks began, in the order they were made.
    let mut begins: Vec<u128> = Vec::new();
    // The moments the asks ended, in the same order.
    let mut ends: Vec<u128> = Vec::new();
    for line in written.lines() {
        // What:     `words` is that line split at its spaces: `begin` or `end`, then the clock, then the ask's own words.
        // Why:      The kind of moment is the first word and the clock is the second, so the rest of the line is not read.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const words = line.split(/\s+/)
        // ```
        let mut words = line.split_whitespace();
        // What:     `kind` is `begin` for the moment an ask started and `end` for the moment it ended.
        // Why:      The two are collected apart, because a gap is measured from an end to the next beginning.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const kind = words.next()
        // ```
        let kind: &str = words.next().expect("a kind before the clock");
        // What:     `moment` is that clock reading: nanoseconds since the epoch, as the stand-in's `date +%s%N` printed it.
        // Why:      A gap between two of them is a duration, and no clock of this process was running while the stand-in held it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const moment = Number(words.next())
        // ```
        let moment: u128 = words
            .next()
            .expect("a clock after the kind")
            .parse()
            .expect("nanoseconds since the epoch");
        if kind == "begin" {
            begins.push(moment);
        } else {
            ends.push(moment);
        }
    }
    assert_eq!(
        begins.len(),
        wanted,
        "the questioning did not walk the whole span: {} asks began where {wanted} attempts are made: {written:?}",
        begins.len()
    );
    assert_eq!(
        ends.len(),
        wanted,
        "every ask that began did not end: {} ends where {wanted} attempts are made: {written:?}",
        ends.len()
    );
    // What:     `shortest` is the shortest time from one ask's end to the next ask's beginning: the pause the discovery slept there.
    // Why:      It is the one figure the records' span is built from, and a pause shorter than the constant shows here and nowhere
    //           else.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let shortest = Number.POSITIVE_INFINITY
    // ```
    let mut shortest: Duration = Duration::MAX;
    // What:     `take(wanted - 1)` walks every ask but the last, which has no ask after it to measure a pause to.
    // Why:      The number of pauses is one fewer than the number of asks, so the last end has no next beginning.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [index, ended] of ends.slice(0, wanted - 1).entries()) { ... }
    // ```
    for (index, ended) in ends.iter().take(wanted - 1).enumerate() {
        // What:     `began_next` is the moment the ask after this one began, which is the end of the pause this control is about.
        // Why:      Nothing but the discovery's own bookkeeping and its sleep stands between an ask's end and the next one's
        //           beginning, and `index + 1` is that next ask.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const beganNext = begins[index + 1]
        // ```
        let began_next: u128 = begins[index + 1];
        // What:     `gap` is `began_next - ended` as a `Duration`: the pause plus the bookkeeping between two programs.
        // Why:      A `Duration` compares against `ASKING_AGAIN_PAUSE` and prints the figure a reader needs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const gap = (beganNext - ended) / 1e6   // milliseconds
        // ```
        let gap: Duration = Duration::from_nanos(
            u64::try_from(began_next - ended).expect("a gap short enough for a duration"),
        );
        println!(
            "the pause before ask {} of {} was {gap:?}",
            index + 2,
            wanted
        );
        if gap < shortest {
            shortest = gap;
        }
    }
    assert!(
        shortest >= ASKING_AGAIN_PAUSE,
        "the shortest pause between two asks was {shortest:?}, shorter than the {ASKING_AGAIN_PAUSE:?} the call site sleeps, so the span is not the {waits:?} the records name: {written:?}",
        waits = ASKING_AGAIN_PAUSE * (ASKING_AGAIN_ATTEMPTS - 1)
    );
    println!(
        "the {} pauses between the asks were at least {shortest:?}, and the call site sleeps {ASKING_AGAIN_PAUSE:?}",
        wanted - 1
    );
}

/// What:
/// `const THE_MOST_A_GAP_OUTLASTS_THE_PAUSE: Duration` is the most the time from one ask's end to the next one's beginning may hold
/// beyond `ASKING_AGAIN_PAUSE`:
/// the pause the discovery sleeps, plus the work of starting, reading and reaping the next stand-in.
/// Why:
/// The span the records name is the four pauses and nothing else,
/// and a sleep at the call site that grew to twice the pause would double it while every lower bound stayed satisfied.
/// What it rests on:
/// measured 2026-11-28 on this host,
/// four gaps of the asking again with the pause the call site sleeps,
/// the shortest 2026-11-28T00:00:00-05:00's own figure is in
/// `doc/second-pass-reads-of-the-alpha-releases-what-each-confirmed.md`,
/// and the same four gaps under `std::thread::sleep(ASKING_AGAIN_PAUSE)` are 209.0 ms to 209.9 ms in the run the package document
/// records.
/// The allowance is a bound on the shortest of four gaps, not on anyone of them:
/// a loaded host stretches one gap and leaves the others,
/// so the figure is the smallest overhead the four met,
/// where a whole span would add all four.
/// Measured 2026-11-28 in the round that added this control:
/// the shortest gap was 218.3 ms with the pause in place,
/// and 475.4 ms with the call site sleeping twice the pause,
/// so the allowance of a hundred milliseconds separates the two with room on both sides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_MOST_A_GAP_OUTLASTS_THE_PAUSE = 100 // milliseconds
/// ```
const THE_MOST_A_GAP_OUTLASTS_THE_PAUSE: Duration = Duration::from_millis(150);

/// The control of the pause's other side:
/// the time from one ask's end to the next one's beginning is no more than the pause the call site sleeps plus
/// `THE_MOST_A_GAP_OUTLASTS_THE_PAUSE`.
/// Why it is a control and not a comment:
/// `the_gap_between_two_asks_is_the_pause_the_call_site_sleeps` bounds that gap only from below,
/// so `std::thread::sleep(ASKING_AGAIN_PAUSE * 2)` at the call site stays green there
/// with a span of 1.6 s where the records name 0.800 s
/// (a same-vendor second read of `0.1.5-alpha.18` named that edit, 2026-11-28).
/// The stand-in is a stand-in:
/// it writes the clock down and refuses every ask,
/// so the discovery's own work between two programs is all the gap holds besides the sleep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // describe('the span the question is asked again over', () => { it('spends no more than the pause the call site sleeps', ...) })
/// ```
#[test]
fn the_pauses_of_the_asking_again_are_no_longer_than_the_pause_the_call_site_sleeps() {
    this_test_needs_a_delegated_scope();
    // The fixture directory of this control.
    let directory = scratch();
    // Where the stand-in writes down the moment each ask began and the moment it ended.
    let log: PathBuf = directory.path().join("asks");
    // What:     `program` is the stand-in: it writes down the nanosecond clock at its start and at its end and refuses the ask.
    // Why:      A gap measured from an end to the next beginning holds the sleep and the work around it, and an ask that answered
    //           would end the asking before the four gaps exist.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-that-refuses-and-writes-down-when', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-that-refuses-every-ask-and-writes-down-when-too",
        &format!(
            "#!/bin/sh\nprintf 'begin %s %s\\n' \"$(date +%s%N)\" \"$*\" >> '{log}'\necho 'the stand-in refuses every ask' >&2\nprintf 'end %s\\n' \"$(date +%s%N)\" >> '{log}'\nexit 1\n",
            log = log.display()
        ),
    );
    // The settings of this control, asking through the stand-in.
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };

    // What:     `refused` is the discovery's failure, which every ask of this stand-in must be.
    // Why:      An answer would mean an ask got through, and the gaps below would not be the asking again's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let refused: string
    // try { detectRoute(settings); throw new Error('the stand-in answered') } catch (error) { refused = String(error) }
    // ```
    let refused = detect_route(&settings)
        .expect_err("a stand-in that refuses every ask is a missing capability");
    assert!(
        refused
            .to_string()
            .contains("the stand-in refuses every ask"),
        "the refusal is not the last ask's own words: {refused}"
    );
    // Every line the stand-in wrote, one per moment.
    let written: String = fs::read_to_string(&log).expect("the stand-in's log");
    // What:     `wanted` is the attempt count as the width a count of lines answers in.
    // Why:      The count of attempts is a `u32` and a count of lines is a `usize`, and the two are compared below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const wanted = ASKING_AGAIN_ATTEMPTS
    // ```
    let wanted: usize = usize::try_from(ASKING_AGAIN_ATTEMPTS)
        .expect("an attempt count that fits a count of lines");
    // The moments the asks began, in the order they were made.
    let mut begins: Vec<u128> = Vec::new();
    // The moments the asks ended, in the same order.
    let mut ends: Vec<u128> = Vec::new();
    for line in written.lines() {
        // What:     `words` is that line split at its spaces: `begin` or `end`, then the clock, then the ask's own words.
        // Why:      The kind of moment is the first word and the clock is the second.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const words = line.split(/\s+/)
        // ```
        let mut words = line.split_whitespace();
        // What:     `kind` is `begin` for the moment an ask started and `end` for the moment it ended.
        // Why:      The two are collected apart, because a gap runs from an end to the next beginning.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const kind = words.next()
        // ```
        let kind: &str = words.next().expect("a kind before the clock");
        // What:     `moment` is that clock reading: nanoseconds since the epoch, as the stand-in's `date +%s%N` printed it.
        // Why:      A gap between two of them is a duration no clock of this process read.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const moment = Number(words.next())
        // ```
        let moment: u128 = words
            .next()
            .expect("a clock after the kind")
            .parse()
            .expect("nanoseconds since the epoch");
        if kind == "begin" {
            begins.push(moment);
        } else {
            ends.push(moment);
        }
    }
    assert_eq!(
        begins.len(),
        wanted,
        "the questioning did not walk the whole span: {} asks began where {wanted} attempts are made: {written:?}",
        begins.len()
    );
    assert_eq!(
        ends.len(),
        wanted,
        "every ask that began did not end: {} ends where {wanted} attempts are made: {written:?}",
        ends.len()
    );
    // What:     `shortest` is the shortest time from one ask's end to the next ask's beginning.
    // Why:      A longest sleep shows in every gap, and the shortest gap is the one a loaded host can stretch least.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let shortest = Number.POSITIVE_INFINITY
    // ```
    let mut shortest: Duration = Duration::MAX;
    // What:     `take(wanted - 1)` walks every ask but the last, which has no ask after it to measure a pause to.
    // Why:      The number of pauses is one fewer than the number of asks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [index, ended] of ends.slice(0, wanted - 1).entries()) { ... }
    // ```
    for (index, ended) in ends.iter().take(wanted - 1).enumerate() {
        // What:     `began_next` is the moment the ask after this one began, which is the end of the pause.
        // Why:      Nothing but the discovery's own bookkeeping and its sleep stands between the two moments.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const beganNext = begins[index + 1]
        // ```
        let began_next: u128 = begins[index + 1];
        // What:     `gap` is `began_next - ended` as a `Duration`: the pause plus the work around it.
        // Why:      A `Duration` compares against the constant and prints the figure a reader needs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const gap = (beganNext - ended) / 1e6   // milliseconds
        // ```
        let gap: Duration = Duration::from_nanos(
            u64::try_from(began_next - ended).expect("a gap short enough for a duration"),
        );
        println!(
            "the pause before ask {} of {} was {gap:?}",
            index + 2,
            wanted
        );
        if gap < shortest {
            shortest = gap;
        }
    }
    // What:     `allowed` is the pause the call site sleeps plus the work `THE_MOST_A_GAP_OUTLASTS_THE_PAUSE` allows it.
    // Why:      The span the records name is the four pauses, so a gap longer than the pause by more than that work is a sleep that
    //           is not the pause.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const allowed = ASKING_AGAIN_PAUSE + THE_MOST_A_GAP_OUTLASTS_THE_PAUSE
    // ```
    let allowed: Duration = ASKING_AGAIN_PAUSE + THE_MOST_A_GAP_OUTLASTS_THE_PAUSE;
    assert!(
        shortest <= allowed,
        "the shortest pause between two asks was {shortest:?}, longer than the {ASKING_AGAIN_PAUSE:?} the call site sleeps plus the {THE_MOST_A_GAP_OUTLASTS_THE_PAUSE:?} the work around it is measured to take, so the span is not the {waits:?} the records name: {written:?}",
        waits = ASKING_AGAIN_PAUSE * (ASKING_AGAIN_ATTEMPTS - 1)
    );
    println!(
        "the {} pauses between the asks were at least {shortest:?}, and the call site sleeps {ASKING_AGAIN_PAUSE:?} plus at most {THE_MOST_A_GAP_OUTLASTS_THE_PAUSE:?} of work",
        wanted - 1
    );
}
