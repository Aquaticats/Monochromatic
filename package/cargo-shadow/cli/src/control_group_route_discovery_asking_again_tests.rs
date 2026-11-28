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
use crate::cgroup_test_support::{
    NOT_RUN_PREFIX,
    this_test_needs_a_delegated_scope,
};
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
/// `use crate::control_group_route_discovery::ASKING_AGAIN_ATTEMPTS;` imports how many times the question is put before a
/// failure is taken for this host's answer.
/// Why:
/// The refusal control requires the question to be put exactly that many times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ASKING_AGAIN_ATTEMPTS } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::ASKING_AGAIN_ATTEMPTS;
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
        elapsed < bound * 3,
        "the discovery answered after {elapsed:?}, so more than the one bound of {bound:?} was spent"
    );
}

/// The control of the rule that decides whether a failure is put again:
/// a program that ended inside the bound it was given is no spent deadline,
/// and one that used its bound is.
/// A discovery that timed both of its programs together would read a slow success and a fast failure as one spent deadline,
/// and this
/// asserts the rule each program is judged by on its own.
#[test]
fn a_program_that_ended_inside_its_bound_is_no_spent_deadline() {
    let settings: RouteSettings = RouteSettings {
        discovery_bound: Duration::from_millis(300),
        ..RouteSettings::default()
    };

    assert!(
        !super::spent_its_bound(&settings, Duration::from_millis(5)),
        "a program that failed at once was read as having spent its deadline"
    );
    assert!(
        !super::spent_its_bound(&settings, Duration::from_millis(250)),
        "a program that ended a twentieth of a second inside its bound was read as having spent it"
    );
    assert!(
        super::spent_its_bound(&settings, Duration::from_millis(300)),
        "a program that used its whole bound was not read as having spent it"
    );
}

/// The control of a question whose words are the refusal:
/// the stand-in refuses every ask,
/// so the discovery ends with the words of its last failure and the route is not found.
#[test]
fn a_question_that_keeps_failing_is_still_a_missing_capability_and_names_its_last_words() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    // What:     `program` is the stand-in: it writes down the ask and refuses it forever.
    // Why:      A capability that really is missing must still be refused, with the manager's own words, however many asks it
    //           took to be sure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-always-refuses', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-always-refuses",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\necho 'the stand-in never answers a slice' >&2\nexit 1\n",
            log = log.display()
        ),
    );
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };

    let refused =
        detect_route(&settings).expect_err("a manager that never answers is a missing capability");

    assert!(
        refused
            .to_string()
            .contains("the user manager cannot be asked where app.slice is (")
            && refused
                .to_string()
                .contains("the stand-in never answers a slice"),
        "the refusal does not name the manager's own words: {refused}"
    );
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    // What:     `usize::try_from(ASKING_AGAIN_ATTEMPTS)` is the attempt count the records name, as a count of lines.
    // Why:      A discovery that stopped short of the attempts would pass a "more than one" check while the README's span
    //           and its cost were no longer the span and the cost.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const wanted = ASKING_AGAIN_ATTEMPTS
    // ```
    let wanted: usize = usize::try_from(ASKING_AGAIN_ATTEMPTS)
        .expect("an attempt count that fits a count of lines");
    assert_eq!(
        calls.lines().count(),
        wanted,
        "the question was not put once per attempt: {calls:?}"
    );
}

/// The control of a refusal the manager answers with itself:
/// a stand-in answers every ask of where the slice is with no value and refuses the start with the manager's own words
/// for a masked slice,
/// so the discovery must end the asking and name the refusal.
/// The words are the ones the container captured at `doc/the-route-question-that-is-asked-again-when-one-ask-fails.md`.
/// The stand-in is a stand-in:
/// it is built from that capture,
/// and no claim about a real manager rests on it here
/// (the same state was measured against a container's own manager:
/// see the package document).
#[test]
fn a_refusal_the_manager_answers_ends_the_asking() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    // What:     `program` is the stand-in: it writes down every ask, answers an ask of where the slice is with nothing, and
    //           refuses the start with the manager's words for a masked slice.
    // Why:      The manager answered the first ask, so the start's failure is its own answer and the asking must end there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-with-a-masked-slice', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-with-a-masked-slice",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\ncase \" $* \" in\n  *\" show \"*) exit 0 ;;\nesac\necho 'Failed to start app.slice: Unit app.slice is masked.' >&2\nexit 1\n",
            log = log.display()
        ),
    );
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };

    let refused =
        detect_route(&settings).expect_err("a slice the manager refuses to start is no route");

    assert!(
        refused
            .to_string()
            .contains("the user manager cannot start app.slice (")
            && refused.to_string().contains("Unit app.slice is masked."),
        "the refusal does not name the manager's own words: {refused}"
    );
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    // The three asks of one answer: where the slice is, the start, and the ask that says the manager is still there.
    assert_eq!(
        calls.lines().count(),
        3,
        "a refusal the manager answered was put again, so the asking spent its span on one answer: {calls:?}"
    );
}

/// The control of an answer that is not a path:
/// a stand-in answers every ask of where the slice is with no value while the start is the real manager's,
/// so the manager answered twice that the slice has no control group and the asking must end there.
#[test]
fn an_answer_that_is_not_a_path_ends_the_asking() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    // What:     `program` is the stand-in: an ask of where the slice is answers nothing, and the start is handed to the real
    //           `systemctl`, so the slice is started by the manager this host has.
    // Why:      A manager that reports no control group for a slice it started is answering, and no wait changes that.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-with-no-control-group', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-with-no-control-group",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\ncase \" $* \" in\n  *\" show \"*) exit 0 ;;\nesac\nexec '{real}' \"$@\"\n",
            log = log.display(),
            real = real_systemctl()
        ),
    );
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };

    let refused = detect_route(&settings)
        .expect_err("a slice the manager reports no control group for is no route");

    assert!(
        refused
            .to_string()
            .contains("the user manager reports no control group for app.slice"),
        "the refusal does not say what the manager answered: {refused}"
    );
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    // The three asks of one answer: where the slice is, the start, and where it is again after that.
    assert_eq!(
        calls.lines().count(),
        3,
        "an answer that is not a path was put again, so the asking spent its span on one answer: {calls:?}"
    );
}

/// The control of a failed start whose reply was lost:
/// the stand-in answers the first ask with no control group,
/// fails the start with the words an in-flight request gives when the manager goes away under it,
/// and hands every ask after that to the real manager.
/// The start really ran,
/// so the ask after it answers the path the slice has,
/// and that path is the route.
#[test]
fn a_path_the_ask_after_a_failed_start_reports_is_the_route() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-with-a-lost-start-reply",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nasked=$(wc --lines < '{log}')\nif [ \"$asked\" -eq 1 ]; then\n  exit 0\nfi\nif [ \"$asked\" -eq 2 ]; then\n  echo \"Failed to start app.slice: Transport endpoint is not connected See user logs and 'systemctl --user status app.slice' for details.\" >&2\n  exit 1\nfi\nexec '{real}' \"$@\"\n",
            log = log.display(),
            real = real_systemctl()
        ),
    );
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };
    let wanted: String = settings.slice.clone();

    let route =
        detect_route(&settings).expect("the path an ask after a failed start reports is the route");

    assert_eq!(
        route.slice, wanted,
        "the discovery did not find the slice the settings name"
    );
    assert!(
        route.slice_dir.ends_with(&wanted),
        "the route's directory is not the slice's: {}",
        route.slice_dir.display()
    );
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    // The three asks of that answer: where the slice is, the start whose reply was lost, and the path it reported.
    assert_eq!(
        calls.lines().count(),
        3,
        "the path the ask after a lost reply reported was thrown away, so the discovery refused a route it had: {calls:?}"
    );
}

/// The control of a start that never reached the manager:
/// the stand-in answers the first ask with no control group,
/// fails the start with the words of a transport that is not there,
/// answers the ask after it with no control group too,
/// and hands every ask after that to the real manager,
/// so the asking must be put again rather than ended on the start's words.
#[test]
fn a_start_that_never_reached_the_manager_is_put_again() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-that-cannot-reach-the-manager",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nasked=$(wc --lines < '{log}')\ncase \"$asked\" in\n  1) exit 0 ;;\n  2) echo 'Failed to connect to user scope bus via local transport: Connection refused' >&2; exit 1 ;;\n  3) exit 0 ;;\nesac\nexec '{real}' \"$@\"\n",
            log = log.display(),
            real = real_systemctl()
        ),
    );
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };
    let wanted: String = settings.slice.clone();

    let route = detect_route(&settings)
        .expect("a start that never reached the manager is a moment, not the manager's answer");

    assert_eq!(
        route.slice, wanted,
        "the discovery did not find the slice the settings name"
    );
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    // Three asks of the attempt that met the moment, and one more ask, which is the whole question put again.
    assert_eq!(
        calls.lines().count(),
        4,
        "a start that never reached the manager ended the asking, so a moment was read as the manager's refusal: {calls:?}"
    );
}

/// The control of a start that failed and an ask after it that failed too:
/// the stand-in fails the first ask of where the slice is,
/// the start,
/// and the ask after the start,
/// and hands every ask after those to the real manager,
/// so the whole question must be put again.
#[test]
fn a_start_that_failed_and_the_ask_after_it_failing_too_is_put_again() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-that-loses-the-start-and-the-ask-after-it",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nasked=$(wc --lines < '{log}')\ncase \"$asked\" in\n  1) exit 0 ;;\n  2) echo \"Failed to start app.slice: Transport endpoint is not connected See user logs and 'systemctl --user status app.slice' for details.\" >&2; exit 1 ;;\n  3) echo 'Failed to get properties: Transport endpoint is not connected' >&2; exit 1 ;;\nesac\nexec '{real}' \"$@\"\n",
            log = log.display(),
            real = real_systemctl()
        ),
    );
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };

    let route = detect_route(&settings)
        .expect("a start and the ask after it failing is a transport, not the manager's answer");

    assert_eq!(
        route.slice, settings.slice,
        "the discovery did not find the slice the settings name"
    );
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    assert_eq!(
        calls.lines().count(),
        4,
        "a failed start followed by a failed ask ended the asking: {calls:?}"
    );
}

/// The control of a slice with a stop job queued:
/// the stand-in answers the first ask with the path the real manager reports for the slice and a job queued on it,
/// answers the start,
/// and hands the ask after the start to the real manager,
/// so the discovery must ask the start through before it takes the path as its route.
#[test]
fn a_stop_job_queued_on_the_slice_is_asked_through_before_the_route_is_taken() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let log: PathBuf = directory.path().join("calls");
    let real: String = real_systemctl();
    let known = std::process::Command::new(&real)
        .args([
            "--user",
            "show",
            "app.slice",
            "--property=ControlGroup",
            "--value",
        ])
        .output()
        .expect("the manager answers where its slice is");
    let reported: String = String::from_utf8_lossy(&known.stdout).trim().to_string();
    if reported.is_empty() {
        println!(
            "{NOT_RUN_PREFIX}{}]: app.slice has no control group on this host, so a stand-in here cannot plant a slice that holds a path and a queued job at once",
            std::thread::current().name().unwrap_or(
                "a_stop_job_queued_on_the_slice_is_asked_through_before_the_route_is_taken"
            )
        );
        return;
    }
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-with-a-stop-job-queued",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nasked=$(wc --lines < '{log}')\ncase \"$asked\" in\n  1) printf 'ControlGroup=%s\\nJob=42\\n' '{reported}'; exit 0 ;;\n  2) exit 0 ;;\nesac\nexec '{real}' \"$@\"\n",
            log = log.display(),
            reported = reported,
            real = real
        ),
    );
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };
    let wanted: String = settings.slice.clone();

    let route = detect_route(&settings)
        .expect("a slice with a stop job queued is a route the call waits out");

    assert_eq!(
        route.slice, wanted,
        "the discovery did not find the slice the settings name"
    );
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    let asked: Vec<&str> = calls.lines().collect();
    assert_eq!(
        asked.len(),
        3,
        "a slice with a job queued was taken as the route without asking the start through: {calls:?}"
    );
    assert!(
        asked[1].contains("--user start app.slice"),
        "the discovery did not ask the start through a slice that has a job queued: {calls:?}"
    );
}
