//! What:
//! The controls of one ask of the user manager:
//! the question it puts,
//! the reading of the answer's two named lines,
//! and the reading of a failed start's words that tells the manager's own refusal from a transport that went away.
//! The question asks for the slice's control group and the job queued on it in one program start,
//! which is the shape a slice being stopped is told by.
//! Why:
//! Each of the three decides something a discovery's whole answer rests on,
//! and until 2026-11-28 no control read any of them:
//! every stand-in answered by call order and ignored the words of its question,
//! so dropping `--property=Job`,
//! or deleting either half of the test on a failed start's words,
//! left the whole suite green
//! (a same-vendor second read of `0.1.5-alpha.18`,
//! `doc/second-pass-reads-of-the-alpha-releases-what-each-confirmed.md`).
//! The stand-ins here are stand-ins:
//! no claim about a real manager rests on them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('one ask of the manager', () => { it('asks for the control group and the job in one program start', ...) })
//! ```

/// What:
/// `use std::fs;` imports the write of a stand-in and the read of the log it keeps.
/// Why:
/// Each control writes the program it plants and then reads what that program was asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, writeFileSync } from 'node:fs'
/// ```
use std::fs;
/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports the trait that sets a file's mode.
/// Why:
/// A stand-in that is not executable cannot stand where `systemctl` stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { chmodSync } from 'node:fs'
/// ```
use std::os::unix::fs::PermissionsExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types (`PathBuf` is the owning sibling of `&Path`,
/// as `String` is to `&str`).
/// Why:
/// The stand-in of a control and the log it writes are paths of that control's own fixture directory.
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
/// `use crate::control_group::RouteSettings;` imports the settings one ask is made with.
/// Why:
/// A control replaces the program the ask starts,
/// which is a field of those settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RouteSettings } from 'crate/control_group'
/// ```
use crate::control_group::RouteSettings;
/// What:
/// `use super::{WhereTheSliceIs, ask_where_the_slice_is_once, the_manager_answered_a_start, what_the_manager_reported};`
/// imports the three items under control from the module this file belongs to.
/// Why:
/// The two readings are private to that module,
/// and a control of them belongs beside them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { WhereTheSliceIs, askWhereTheSliceIsOnce, theManagerAnsweredAStart, whatTheManagerReported } from './control_group_route_one_ask'
/// ```
use super::{
    WhereTheSliceIs,
    ask_where_the_slice_is_once,
    the_manager_answered_a_start,
    what_the_manager_reported,
};
/// What:
/// `use crate::test_scratch::scratch;` imports the disposable directory of one control.
/// Why:
/// The stand-in and its log live in a fixture of the control's own,
/// never in another control's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { scratch } from 'crate/test_scratch'
/// ```
use crate::test_scratch::scratch;

/// What:
/// `fn a_program(directory: &Path, name: &str, text: &str) -> PathBuf` writes an executable stand-in into a control's directory
/// and answers its path.
/// Why:
/// Every control here plants one program in place of `systemctl`,
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
    // What:     `directory.join(name)` is the path the stand-in is written at.
    // Why:      Each control names its own stand-in, so a failure names which one ran.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = path.join(dir, name)
    // ```
    let program: PathBuf = directory.join(name);
    fs::write(&program, text).expect("write the stand-in");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
    // What:     `program` is that path, handed to the caller as the program its settings name.
    // Why:      The settings take the program as text, and the caller needs the path to build them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return program
    // ```
    return program;
}

/// The control of the question's shape:
/// one program start asks for the slice's control group and the job queued on it,
/// so the two facts a route turns on come from one answer.
/// Why it is a control and not a comment:
/// every other stand-in in this package answers by call order and ignores the words of its question,
/// so dropping `--property=Job` from the question left every control green
/// while a real manager's answer stops carrying a `Job=` line at all
/// (a same-vendor second read of `0.1.5-alpha.18`,
/// 2026-11-28),
/// and the queued stop the discovery waits out would then never be seen.
#[test]
fn the_question_asks_for_the_slice_s_control_group_and_the_job_queued_on_it() {
    // The fixture directory of this control.
    let directory = scratch();
    // Where the stand-in writes down each ask.
    let log: PathBuf = directory.path().join("calls");
    // What:     `program` is the stand-in: it writes down every ask and answers the slice's path with no job queued on it.
    // Why:      The answer is the calm host's, so the question's own words are the only thing this control judges.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = aProgram(dir, 'systemctl-that-writes-down-its-question', `...`)
    // ```
    let program: PathBuf = a_program(
        directory.path(),
        "systemctl-that-writes-down-its-question",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nprintf 'ControlGroup=/user.slice/user-1000.slice/user@1000.service/app.slice\\nJob=\\n'\nexit 0\n",
            log = log.display()
        ),
    );
    // The settings of this control, asking through the stand-in.
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };

    // What:     `answered` is the one ask's answer, read as the path the slice has and the job queued on it.
    // Why:      A manager that answered is no failure, so the reading of its answer is what this control sees.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const answered = askWhereTheSliceIsOnce(settings)   // throws FailedAsk
    // ```
    let answered: WhereTheSliceIs = match ask_where_the_slice_is_once(&settings) {
        Ok(found) => found,
        Err(failed) => panic!(
            "a manager that answers is no failure, and this ask failed with: {}",
            failed.reason
        ),
    };
    // What:     `(path, a_job_is_queued)` is the answer's two facts, or a panic naming the shape that was read instead.
    // Why:      Both facts are the subject: a path is the route, and a job queued means the route has to be waited for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (answered.kind !== 'aPath') throw new Error(`the answer was read as ${answered.kind}`)
    // const { path, aJobIsQueued } = answered
    // ```
    let (path, a_job_is_queued): (String, bool) = match answered {
        // What:     `WhereTheSliceIs::APath { path, a_job_is_queued }` is the manager reporting both facts from the one answer.
        // Why:      A control that reached any other variant would say nothing about a host that answered with a path.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [path, aJobIsQueued]
        // ```
        WhereTheSliceIs::APath {
            path,
            a_job_is_queued,
        } => (path, a_job_is_queued),
        // What:     `WhereTheSliceIs::NoControlGroup` is any other answer, which a stand-in that printed a path cannot give.
        // Why:      The control's fixture is a path with a `Job=` line, so this arm is the failure of the reading under test.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // throw new Error('the answer a stand-in printed a path for was read as no control group at all')
        // ```
        WhereTheSliceIs::NoControlGroup => {
            panic!("the answer a stand-in printed a path for was read as no control group at all")
        }
    };
    assert_eq!(
        path, "/user.slice/user-1000.slice/user@1000.service/app.slice",
        "the answer's ControlGroup line was not read as the slice's path"
    );
    assert!(
        !a_job_is_queued,
        "an empty Job= line was read as a job queued on the slice"
    );
    // Every ask the stand-in was given, one line each.
    let calls: String = fs::read_to_string(&log).expect("the stand-in's log");
    // What:     `asked` is the first ask's words: the program, its subcommand, the unit, and the two properties.
    // Why:      The properties are the question's shape, and a `systemctl show` without `--property=Job` answers no `Job=` line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const asked = calls.split('\n')[0]
    // ```
    let asked: Option<&str> = calls.lines().next();
    assert_eq!(
        asked,
        Some("--user show app.slice --property=ControlGroup --property=Job"),
        "the question does not ask for the slice's control group and the job queued on it in one program start: {calls:?}"
    );
    assert_eq!(
        calls.lines().count(),
        1,
        "one answer was asked for with more than one program start: {calls:?}"
    );
}

/// The control of the reading of a failed start's words:
/// the manager's own refusal about the slice is its answer,
/// and a transport that went away,
/// or words that name the slice only in the verb's own opening,
/// are moments a wait can change.
/// Why it is a control and not a comment:
/// no control named `the_manager_answered_a_start` before 2026-11-28,
/// so deleting either half of its test stayed green:
/// without the transport's words read,
/// a start that lost its reply reads as the manager's refusal;
/// without the slice's name required after the verb,
/// any `Failed to start app.slice: ` failure does
/// (a same-vendor second read of `0.1.5-alpha.18`).
/// The masked slice's words are the container's capture,
/// and the transport's words are the ones the same read recorded:
/// the last two are built for the boundary they test and stand for no program's output.
#[test]
fn the_manager_answered_a_start_tells_its_refusal_from_a_moment_and_from_other_words() {
    // The slice the question names, which every start failure names in its opening.
    let slice: &str = "app.slice";

    assert!(
        the_manager_answered_a_start(
            "Failed to start app.slice: Unit app.slice is masked.",
            slice
        ),
        "the manager's own words about a masked slice were read as a moment, so the asking would spend its whole span on an answer"
    );
    assert!(
        !the_manager_answered_a_start(
            "Failed to start app.slice: Transport endpoint is not connected See user logs and 'systemctl --user status app.slice' for details.",
            slice
        ),
        "a start whose reply was lost was read as the manager's refusal, so a call that really started the slice would be refused"
    );
    assert!(
        !the_manager_answered_a_start("Failed to start app.slice: Job canceled.", slice),
        "words that name the slice only in the verb's opening were read as the manager's answer about it, so a refusal that names no unit would end the asking"
    );
    assert!(
        !the_manager_answered_a_start(
            "Failed to connect to user scope bus via local transport: Connection refused",
            slice
        ),
        "words of a program that never reached the manager were read as the manager's answer"
    );
}

/// The control of the reading of an answer:
/// the two named lines `systemctl show` prints,
/// and the one value a program that replaced the question prints.
/// Why it is a control and not a comment:
/// the queued job is the fact a slice being stopped is told by,
/// and an empty path is no route;
/// a reading that took an empty `ControlGroup=` for a path,
/// or ignored the `Job=` line,
/// would send a call into a slice it cannot make a scope in.
#[test]
fn what_the_manager_reported_reads_the_two_named_lines_and_one_value_alone() {
    // What:     `cases` holds one answer, the path this control expects it to be read as (`None` for an answer that is no route), and
    //           whether a job is expected to be read out of it.
    // Why:      The cases are laid out together so a reader sees every branch of the reading in one place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cases: [string, string | undefined, boolean][] = [...]
    // ```
    let cases: [(&str, Option<&str>, bool); 8] = [
        (
            "ControlGroup=/user.slice/app.slice\nJob=\n",
            Some("/user.slice/app.slice"),
            false,
        ),
        (
            "ControlGroup=/user.slice/app.slice\nJob=42\n",
            Some("/user.slice/app.slice"),
            true,
        ),
        ("ControlGroup=\nJob=\n", None, false),
        ("/user.slice/app.slice", Some("/user.slice/app.slice"), false),
        ("", None, false),
        ("not-a-path", None, false),
        ("/one/path\n/another/path", None, false),
        ("Foo=bar\n", None, false),
    ];
    for (answered, wanted_path, wanted_job) in cases {
        // What:     `read` is that answer as the reading under control answers it.
        // Why:      The three fields below are what every caller of this reading turns on.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const read = whatTheManagerReported(answered)
        // ```
        let read: (Option<String>, bool) = match what_the_manager_reported(answered) {
            // What:     `WhereTheSliceIs::APath { path, a_job_is_queued }` is a route, with the job's own fact.
            // Why:      A path the manager reported is what the scope is planned under.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return [path, aJobIsQueued]
            // ```
            WhereTheSliceIs::APath {
                path,
                a_job_is_queued,
            } => (Some(path), a_job_is_queued),
            // What:     `WhereTheSliceIs::NoControlGroup` is an answer that is no route.
            // Why:      An empty or relative value, and a name that is not the slice's, are the two ways to it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return [undefined, false]
            // ```
            WhereTheSliceIs::NoControlGroup => (None, false),
        };
        assert_eq!(
            read.0.as_deref(),
            wanted_path,
            "the answer {answered:?} was read as the path {read_path:?}, where this control expects {wanted_path:?}",
            read_path = read.0
        );
        assert_eq!(
            read.1, wanted_job,
            "the answer {answered:?} was read as a queued job of {read_job}, where this control expects {wanted_job}",
            read_job = read.1
        );
    }
}
