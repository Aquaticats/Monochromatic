//! What:
//!  The ban on a control that folds one earlier log twice in the run's own process:
//!  a control that folds again asks its question in a child of its own first.
//! Why:
//!  The fold takes the log's journal `flock` for one fold and drops it as it answers
//!  (`foundation/earlier_log_lines_added_to_current_log.rs`),
//!  so a control that folds twice takes that one lock twice,
//!  and a fork copies every open descriptor while `flock` lasts until every copy of the open file description is
//!  closed:
//!  a child another test's thread forked between the control's `open` and its `close` keeps the lock after the
//!  control dropped its own,
//!  and the second fold answers `LogFold::Busy` where the control asserts `LogFold::Folded`.
//!  The fold of every kept name of a Git directory folds the logs among those names and holds that same lock,
//!  so a control reaches the ban's fact through either call.
//!  Measured on 2026-11-27 for issue 6602:
//!  the control of a log that keeps growing failed one threaded run of ten modules with `left: Busy right:
//!  Folded`,
//!  and held that run's module alone.
//!  The writer descriptor a control keeps across two folds is copied the same way,
//!  and `writer_check` reads the copy of it as a program still holding the moved log open,
//!  so the fold answers `LogFold::LeftForALaterCommand` where that control asserts `LogFold::Folded`.
//!  `cargo test` runs a crate's tests as threads of one process,
//!  and the container gate starts each test in a process of its own,
//!  so the gate cannot show either.
//!  A control that starts `run_the_body_in_a_child_of_its_own` first asks its question in a child of the test
//!  binary,
//!  which holds that one control and forks nothing
//!  (`testing/process_wide_state_child_test_support.rs`).
//!  A control that folds once in the run's own process is no finding:
//!  the lock it takes is one it has not dropped yet,
//!  so no copy of it can exist.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names fold_log( or fold_kept_names( more than once in code must also name
//! // run_the_body_in_a_child_of_its_own(.
//! ```

use std::path::PathBuf;

use super::source_code_places_tests::is_code;
use crate::source_scan_tests::{
    CrateRoot,
    classified_sources,
    real_roots,
    source_root,
};

/// What:
///  `THE_FOLDS` are the calls that fold one earlier log into the current log,
///  and take that log's journal lock for the round they fold.
/// Why:
///  The lock's life is what this ban is about,
///  and each of these calls is a way a control reaches it:
///  the fold of one log's lines,
///  and the fold of every kept name of a Git directory,
///  which folds the logs among those names one after another.
const THE_FOLDS: &[&str] = &["fold_log(", "fold_kept_names("];

/// What:
///  `IN_A_CHILD_OF_ITS_OWN` is the call that starts one control's body in a child process of the test binary.
/// Why:
///  A control that makes it asks its question where no other test's thread forks,
///  so a control that folds twice and does not name this call is the finding.
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  `FOLDS_ONE_PROCESS_OWNS` is how many folds a control may make in the run's own process:
///  one,
///  because the lock of that fold is taken before any is dropped.
/// Why:
///  The next fold takes the one lock again,
///  and that is the fold a forked copy of the dropped descriptor can answer.
const FOLDS_ONE_PROCESS_OWNS: usize = 1;

/// What:
///  `THE_FOLD_FILES` are the test-only sources whose controls fold a log,
///  which the scan must still find folding twice,
///  or it would guard nothing there.
/// Why:
///  The finding this ban refuses was measured in the first of them,
///  and the second folds the same logs through the fold of every kept name;
///  a scan that stopped seeing either would pass every tree.
const THE_FOLD_FILES: &[&str] = &[
    "foundation/earlier_log_lines_added_to_current_log_tests.rs",
    "foundation/earlier_names_folded_into_current_tests.rs",
];

/// What:
///  `Control` is one control of a test-only source:
///  the line its `#[test]` is on,
///  its function's name,
///  how many times its code folds a log,
///  and whether its code starts a child of its own.
/// Why:
///  A finding has to name the control and the line to open,
///  and a scan of loose lines could not say which control folds again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Control = { startsAt: number; name: string; folds: number; ownsItsProcess: boolean };
/// ```
struct Control {
    /// The line of the control's `#[test]`,
    /// counted from one,
    /// which its findings are placed against.
    starts_at: usize,
    /// The name of the control's function,
    /// empty until the scan reaches its `fn` line.
    name: String,
    /// How many times the control's code names the fold.
    folds: usize,
    /// Whether any line of the control starts a child of its own in code.
    owns_its_process: bool,
}

/// What:
///  `function_name_of(line)` is the name of the function a line declares,
///  or nothing when the line declares none.
/// Why:
///  A control's name follows its `#[test]` by a line or two,
///  and the finding has to name the control rather than its position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const functionNameOf = (line) => /^\s*fn ([A-Za-z0-9_]+)\(/.exec(line)?.[1];
/// ```
fn function_name_of(line: &str) -> Option<String> {
    let trimmed: &str = line.trim_start();
    // What: the `?` answers `None` from this function as soon as the line opens with something else.
    // Why: the scan walks every line of a control, and most of them declare nothing.
    let rest: &str = trimmed.strip_prefix("fn ")?;
    // What: the same early answer where the line has no opening parenthesis after the name.
    // Why: the name ends at that parenthesis, and a line without one holds no name to report.
    let (name, _): (&str, &str) = rest.split_once('(')?;
    return Some(String::from(name.trim()));
}

/// What:
///  `occurrences_in_code(line, offset, code, call)` counts how many times `line` makes `call` in code,
///  where the line begins at `offset` in the file.
/// Why:
///  A control's doc comment names the calls it makes and a control's fixture plants them in text,
///  and neither is a call,
///  so the file's own answer to which bytes are code decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function occurrencesInCode(line, offset, code, call) { let seen = 0; for (let at = line.indexOf(call); at >= 0; at = line.indexOf(call, at + 1)) { if (code[offset + at]) seen += 1; } return seen; }
/// ```
fn occurrences_in_code(
    line: &str,
    offset: usize,
    code: &[bool],
    call: &str,
) -> usize {
    let mut seen: usize = 0;
    let mut searched: usize = 0;
    while let Some(place) = line[searched..].find(call) {
        // What: the place of this occurrence in the line, counted from the line's start.
        // Why: the code answer is kept for the whole file, so a line's place is added to the line's own offset.
        let column: usize = searched + place;
        if code.get(offset + column) == Some(&true) {
            seen += 1;
        }
        searched = column + call.len();
    }
    return seen;
}

/// What:
///  `controls(text)` splits one test-only source into its controls,
///  in the order they appear.
/// Why:
///  Each control is judged on its own lines,
///  so one control that starts a child never excuses another that does not.
/// Gotcha:
///  The text is split on its newlines and every offset is the byte count the file holds,
///  which is exact because this crate's sources end their lines with one `\n`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function controls(text) { const code = isCode(text); const found = []; let current; let offset = 0; for (const [index, raw] of lines(text)) { const line = raw.replace(/\n$/, ''); if (line.startsWith('#[test]')) { if (current) found.push(current); current = { startsAt: index + 1, name: '', folds: 0, ownsItsProcess: false }; } if (current) { current.name ||= functionNameOf(line) ?? ''; current.folds += occurrencesInCode(line, offset, code, THE_FOLD); current.ownsItsProcess ||= occurrencesInCode(line, offset, code, IN_A_CHILD_OF_ITS_OWN) > 0; } offset += raw.length; } if (current) found.push(current); return found; }
/// ```
fn controls(text: &str) -> Vec<Control> {
    let code: Vec<bool> = is_code(text);
    let mut found: Vec<Control> = Vec::new();
    let mut current: Option<Control> = None;
    let mut offset: usize = 0;
    for (index, raw) in text.split_inclusive('\n').enumerate() {
        // What: the line without its newline, so a name or a call at its end is read whole.
        // Why: `split_inclusive` keeps the newline, and the needle would otherwise carry it.
        let line: &str = raw.strip_suffix('\n').unwrap_or(raw);
        if line.starts_with("#[test]") {
            if let Some(finished) = current.take() {
                found.push(finished);
            }
            current = Some(Control {
                starts_at: index + 1,
                name: String::new(),
                folds: 0,
                owns_its_process: false,
            });
        }
        if let Some(control) = current.as_mut() {
            if control.name.is_empty()
                && let Some(name) = function_name_of(line)
            {
                control.name = name;
            }
            for call in THE_FOLDS {
                control.folds += occurrences_in_code(line, offset, code.as_slice(), call);
            }
            if occurrences_in_code(line, offset, code.as_slice(), IN_A_CHILD_OF_ITS_OWN) > 0 {
                control.owns_its_process = true;
            }
        }
        offset += raw.len();
    }
    if let Some(finished) = current {
        found.push(finished);
    }
    return found;
}

/// What:
///  `findings_of(path, found)` is every control among `found` that folds a log twice in the run's own process,
///  as `path:line (control)`.
/// Why:
///  The finding is the whole answer a ban's control needs:
///  the real tree's files must give none,
///  and a planted text must give exactly the control that folds again there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsOf(path, found) { return found.filter((c) => c.folds > FOLDS_ONE_PROCESS_OWNS && !c.ownsItsProcess).map((c) => `${path}:${c.startsAt} (${c.name})`); }
/// ```
fn findings_of(
    path: &str,
    found: &[Control],
) -> Vec<String> {
    let mut findings: Vec<String> = Vec::new();
    for control in found {
        if control.folds <= FOLDS_ONE_PROCESS_OWNS || control.owns_its_process {
            continue;
        }
        findings.push(format!("{path}:{} ({})", control.starts_at, control.name));
    }
    return findings;
}

/// What:
///  `findings_in_text(path, text)` is every control of `text` that folds a log twice in the run's own process.
/// Why:
///  The ban's own control plants one control that folds twice and one that starts a child,
///  and those two lines are the whole proof that the ban refuses what it says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsInText(path, text) { return findingsOf(path, controls(text)); }
/// ```
pub(super) fn findings_in_text(
    path: &str,
    text: &str,
) -> Vec<String> {
    return findings_of(path, controls(text).as_slice());
}

/// What:
///  `Scan` is what reading every test-only source of the crate answers:
///  each control that folds a log twice in the run's own process,
///  and each file that holds a control folding a log at all.
/// Why:
///  The findings are what must be empty,
///  and the files are what says the scan still reads the controls that fold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scan = { findings: string[]; filesFolding: string[] };
/// ```
struct Scan {
    /// Each control that folds a log twice without a child of its own,
    /// as `path:line (control)`.
    findings: Vec<String>,
    /// Each file that holds a control naming the fold.
    files_folding: Vec<String>,
}

/// What:
///  `scan()` reads every test-only source under the crate's source root and answers what `Scan` states,
///  or one text saying why the scan cannot answer.
/// Why:
///  The release code folds a log once per command and is the fold's own caller,
///  so it is no finding:
///  only a control asks the question beside another test of one process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan() { const findings = []; const filesFolding = []; for (const file of classifiedSources(sourceRoot(), realRoots()).filter((f) => f.testOnly)) { const found = controls(read(file)); if (found.some((c) => c.folds > 0)) filesFolding.push(file.path); findings.push(...findingsOf(file.path, found)); } return { findings, filesFolding }; }
/// ```
fn scan() -> Result<Scan, String> {
    let root: PathBuf = source_root();
    let roots: Vec<CrateRoot> = real_roots();
    let mut read: Scan = Scan {
        findings: Vec::new(),
        files_folding: Vec::new(),
    };
    for file in classified_sources(root.as_path(), roots.as_slice())? {
        if !file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        let found: Vec<Control> = controls(text.as_str());
        if found.iter().any(|control| return control.folds > 0) {
            read.files_folding.push(file.path.clone());
        }
        read.findings
            .extend(findings_of(file.path.as_str(), found.as_slice()));
    }
    return Ok(read);
}

/// The fold takes one lock for one fold and drops it as it answers,
/// so a control that folds twice in the run's own process reads a lock another test's fork can keep.
#[test]
fn no_control_folds_a_log_twice_in_the_run_s_own_process() {
    let read: Scan = scan().expect("the crate's sources");
    assert!(
        read.findings.is_empty(),
        "the fold of an earlier log takes the journal's `flock` for one fold and drops it as it answers, and a fork \
         copies every open descriptor while `flock` lasts until every copy is closed, so beside its neighbours a \
         control that folds twice reads a lock another test's child keeps: start the control's body in a child of its \
         own first (`run_the_body_in_a_child_of_its_own`, `testing/process_wide_state_child_test_support.rs`, issue \
         6602):\n{}",
        read.findings.join("\n")
    );
}

/// The controls that fold a log are still read,
/// so a run that finds none fails here instead of passing every tree.
#[test]
fn the_controls_that_fold_a_log_still_fold_one() {
    let read: Scan = scan().expect("the crate's sources");
    for file in THE_FOLD_FILES {
        assert!(
            read.files_folding.iter().any(|path| return path == file),
            "{file} holds controls that fold a log, and this scan read none of them there, so it guards nothing: \
             these files name a fold now: {:?}",
            read.files_folding
        );
    }
}

/// The scan finds a control that folds twice in the run's own process,
/// and says nothing about the control that folds once beside it,
/// nor about the one that folds twice in a child of its own,
/// nor about names a comment or a planted text only holds.
#[test]
fn the_scan_finds_a_control_that_folds_again_without_a_child_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn folds_twice_in_the_run_s_process() {\n\
    fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect(\"first\");\n\
    fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect(\"second\");\n\
}\n\
\n\
#[test]\n\
fn folds_once_in_the_run_s_process() {\n\
    fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect(\"only\");\n\
}\n\
\n\
#[test]\n\
fn folds_twice_in_a_child_of_its_own() {\n\
    /// The doc comment names fold_log( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"folds_twice_in_a_child_of_its_own\", || {\n\
        fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect(\"first\");\n\
        fold_log(root.as_path(), COMMIT_TIMING_LOG, &mut NeverStopped).expect(\"second\");\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_name_in_a_text() {\n\
    let planted: &str = \"fold_log(a); fold_log(b);\";\n\
}\n";
    assert_eq!(
        findings_in_text("planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (folds_twice_in_the_run_s_process)"
        )],
        "the planted control that folds a log twice in the run's own process was not the one finding"
    );
    let planted_directory: &str = "\
#[test]\n\
fn folds_a_directory_twice_in_the_run_s_process() {\n\
    fold_kept_names(root.as_path(), NAMES, &surroundings, &mut NeverStopped);\n\
    fold_kept_names(root.as_path(), NAMES, &surroundings, &mut NeverStopped);\n\
}\n";
    assert_eq!(
        findings_in_text("planted_tests.rs", planted_directory),
        vec![String::from(
            "planted_tests.rs:1 (folds_a_directory_twice_in_the_run_s_process)"
        )],
        "the planted control that folds every kept name twice in the run's own process was not the one finding"
    );
}
