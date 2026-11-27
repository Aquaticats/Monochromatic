//! What:
//! The controls of the discovery's question to the user manager:
//! a failure that came back at once is put again and the route is found,
//! and a manager that answered nothing inside its bound is asked once and ends the discovery with its own words.
//! Why:
//! Every run and every control that needs a delegated scope asks that question before it does anything,
//! so one momentary failure fails all of them at once:
//! on 2026-11-27 one run of this package's suite failed ten controls in nine milliseconds each,
//! which is that question being answered with a failure (issue 6619).
//! The stand-ins here are stand-ins:
//! no claim about a real manager rests on them,
//! and the route in the first control is a real one,
//! found through the real `systemctl`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('the route question', () => { it('is put again when it fails at once', ...) })
//! ```

/// What:
/// `use std::fs;` imports the write of a stand-in and the read of the log it keeps.
/// Why:
/// Each control writes the program it plants and then reads what that program was asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, writeFileSync } from 'node:fs';
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
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the monotonic clock.
/// Why:
/// The control of a manager that answers nothing bounds its own question and times how long the discovery took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds, and Date.now()
/// ```
use std::time::{
    Duration,
    Instant,
};

/// What:
/// `use crate::cgroup_test_support::this_test_needs_a_delegated_scope;` imports the line that says a control cannot run where this
/// host has no manager.
/// Why:
/// Both controls ask a manager for a real route,
/// and a host without one must report them as not run rather than as failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { thisTestNeedsADelegatedScope } from 'crate/cgroup_test_support';
/// ```
use crate::cgroup_test_support::this_test_needs_a_delegated_scope;
/// What:
/// `use crate::control_group::{RouteSettings, detect_route};` imports the discovery and the settings that replace the program it
/// asks.
/// Why:
/// The question under control is the discovery's own,
/// asked through a program the control wrote.
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

/// What:
/// `fn a_program(directory: &Path, name: &str, text: &str) -> PathBuf` writes an executable stand-in into a control's directory and
/// answers its path.
/// Why:
/// Both controls plant one program in place of `systemctl`,
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
    let program: PathBuf = directory.join(name);
    fs::write(&program, text).expect("write the stand-in");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return program;
}

/// What:
/// `fn real_systemctl() -> String` is the first `systemctl` on `PATH`.
/// Why:
/// The stand-in of the first control asks the real manager for every request after the one it refuses,
/// so the route that control finds is a real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function realSystemctl(): string
/// ```
fn real_systemctl() -> String {
    let search: String = std::env::var("PATH").unwrap_or_default();
    for directory in search.split(':') {
        if directory.is_empty() {
            continue;
        }
        let candidate: PathBuf = Path::new(directory).join("systemctl");
        if crate::control_group::is_executable_file(&candidate) {
            return candidate.display().to_string();
        }
    }
    panic!("no systemctl is on PATH, so this host has no user manager to ask");
}

/// The control of a question that failed once:
/// the stand-in refuses its first ask and runs the real manager for every ask after it,
/// so a discovery that puts the question again finds the real route.
#[test]
fn a_question_that_fails_at_once_is_put_again_and_the_route_is_found() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let refused_once: PathBuf = directory.path().join("refused-once");
    // What:     `program` is the stand-in: it writes down every ask, refuses the first one, and runs the real manager for the rest.
    // Why:      The first ask is the one a momentary failure met; every ask after it has to succeed for the route to be found.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-fails-once', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-fails-once",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nif [ ! -e '{refused_once}' ]; then\n  : > '{refused_once}'\n  echo 'the stand-in refused one ask' >&2\n  exit 1\nfi\nexec '{real}' \"$@\"\n",
            log = log.display(),
            refused_once = refused_once.display(),
            real = real_systemctl()
        ),
    );
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };
    let wanted: String = settings.slice.clone();

    let route = detect_route(&settings).expect("one failed ask is no missing capability");

    assert_eq!(
        route.slice, wanted,
        "the discovery did not find the slice the settings name"
    );
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    assert!(
        calls.lines().count() > 1,
        "the question was asked once only, so the failed ask ended the discovery: {calls:?}"
    );
}

/// The control of a manager that answers nothing:
/// its stand-in runs until it is killed and writes down every ask,
/// so a discovery that waited for a second bound would be seen in the count and in the clock.
#[test]
fn a_manager_that_answers_nothing_is_asked_once_and_ends_the_discovery_with_its_own_words() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    // What:     `program` is the stand-in: it writes down every ask and then does nothing for ten minutes, so only the bound ends it.
    // Why:      An ask that spends its own deadline is the one failure a second ask cannot change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-never-answers', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-never-answers",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nexec sleep 600\n",
            log = log.display()
        ),
    );
    // The bound of this control is short, so a discovery that asked again would take about twice it.
    let bound: Duration = Duration::from_millis(300);
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        question: Some(vec!["--user".to_string(), "show".to_string()]),
        discovery_bound: bound,
        ..RouteSettings::default()
    };

    let began: Instant = Instant::now();
    let refused = detect_route(&settings)
        .expect_err("a manager that answers nothing is a missing capability");
    let elapsed: Duration = began.elapsed();

    assert!(refused.to_string().contains("did not answer"), "{refused}");
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    assert_eq!(
        calls.lines().count(),
        1,
        "the question was put again after a manager spent its deadline: {calls:?}"
    );
    assert!(
        elapsed >= bound,
        "the discovery answered after {elapsed:?}, before the bound of {bound:?} it gives the manager"
    );
    assert!(
        elapsed < bound * 2,
        "the discovery answered after {elapsed:?}, so more than the one bound of {bound:?} was spent"
    );
}
