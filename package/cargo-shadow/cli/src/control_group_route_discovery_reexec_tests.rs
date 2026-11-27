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
