//! What:
//!  The ban on a control reading the preparation's named steps in the run's own process:
//!  a control that asks about that ledger starts a child of its own first.
//! Why:
//!  `commit/prepare_work_spans.rs` keeps one `static` ledger for the whole process,
//!  which every preparation notes into and every timing record empties,
//!  and `cargo test` runs a crate's tests as threads of one process.
//!  So a control that asserts the steps its own preparation noted reads the steps of every other test
//!  that ran a preparation beside it,
//!  and a control that asserts an empty ledger reads whatever another test left there:
//!  measured on 2026-11-26,
//!  `commit::commit_index::tests` failed `every_selection_mode_states_its_one_read_and_its_seed_walk`
//!  with a second `index read` in both threaded runs (issue 6507),
//!  and one process holding a preparation's test and another's record failed the record's list.
//!  The container gate starts each test in a process of its own,
//!  so the gate cannot show it.
//!  A control that hands its body to `run_the_body_in_a_child_of_its_own` first asks its question in a child of the test
//!  binary,
//!  whose ledger holds nothing but what that control noted
//!  (`testing/process_wide_state_child_test_support.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names drain_spans(, note_span(, prepare_work_spans:: or prepareSpans must also name
//! // run_the_body_in_a_child_of_its_own(.
//! ```

use std::path::Path;

use super::source_code_places_tests::is_code;
use crate::source_scan_tests::{
    CrateRoot,
    classified_sources,
    real_roots,
    source_root,
};

/// What:
///  `LEDGER_NAMES` is each call and each name that reaches the preparation's one ledger for the process,
///  with why that name is the ledger's.
///  Its siblings a reader might expect are a list of the two functions alone,
///  which misses an import of the module and the record's own field,
///  and one needle `prepare_work_spans`,
///  which misses a control whose file takes the two functions through `use super::*`.
/// Why:
///  A control that names the ledger asks a question about the whole process,
///  so it is the name a control reaches the ledger by that has to be found,
///  not one spelling of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LEDGER_NAMES: ReadonlyArray<readonly [string, string]> = [['drain_spans(', '...'], ...];
/// ```
const LEDGER_NAMES: &[(&str, &str)] = &[
    ("drain_spans(", "empties the steps the whole process noted"),
    (
        "note_span(",
        "adds one step to the list the whole process noted",
    ),
    (
        "prepare_work_spans::",
        "names the module that keeps the one ledger of the process",
    ),
    (
        "prepareSpans",
        "names the record's field for the steps the whole process noted",
    ),
];

/// What:
///  The call that runs one control's body in a child process of the test binary.
/// Why:
///  A control that hands its body to it asks its question where nothing else noted a step,
///  so a control that names the ledger and not this call is the finding.
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  `A_CONTROL_STARTS_AT` is the line that begins one control of a test-only source.
/// Why:
///  Every control of this crate is a top-level function marked by this line,
///  so the text from one of these lines to the next is exactly one control's body.
const A_CONTROL_STARTS_AT: &str = "#[test]";

/// What:
///  `FILES_THAT_READ_THE_LEDGER` is each file whose controls still ask about the ledger,
///  with what its controls ask.
/// Why:
///  A scan that found no reader at all would pass every tree and guard nothing,
///  so a run that stops seeing these files fails here instead,
///  and a file that moves or drops its control shrinks this list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FILES_THAT_READ_THE_LEDGER = [['commit/commit_index_tests.rs', 'one control counts the steps of one preparation'], ...];
/// ```
const FILES_THAT_READ_THE_LEDGER: &[(&str, &str)] = &[
    (
        "commit/commit_index_tests.rs",
        "one control counts the steps one preparation notes",
    ),
    (
        "commit/commit_timing_tests.rs",
        "one control states the steps its record holds",
    ),
    (
        "commit/prepare_work_spans_tests.rs",
        "three controls ask how the ledger adds, bounds and empties its steps",
    ),
];

/// What:
///  `Control` is one control of a test-only source:
///  the line its `#[test]` is on,
///  its function's name,
///  whether its code names the ledger,
///  and whether its code starts a child of its own.
/// Why:
///  A finding has to name the control and the line to open,
///  and a scan of loose lines could not say which control asks the question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Control = { startsAt: number; name: string; namesTheLedger: boolean; startsAChild: boolean };
/// ```
struct Control {
    /// What:
    ///  the line number of the control's `#[test]`,
    ///  counted from one,
    ///  which its findings are placed against.
    /// Why:
    ///  a finding is `path:line`,
    ///  and the line is where the control starts.
    starts_at: usize,
    /// What:
    ///  the name of the control's function,
    ///  empty until the scan reaches its `fn` line.
    /// Why:
    ///  the finding names the control,
    ///  so a reader knows which one to open.
    name: String,
    /// What:
    ///  whether any line of the control names the ledger in code.
    /// Why:
    ///  that is the question the check is about.
    names_the_ledger: bool,
    /// What:
    ///  whether any line of the control starts a child of its own in code.
    /// Why:
    ///  a control that makes that call asks its question in a process no other test noted in.
    starts_a_child: bool,
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
///  `names_the_call_in_code(line, offset, code, call)` answers whether `line` makes `call` in code,
///  where the line begins at `offset` in the file.
/// Why:
///  A control's doc comment names the calls it asks about and a control's fixture plants them in text,
///  and neither is a call,
///  so the file's own answer to which bytes are code decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesTheCallInCode(line, offset, code, call) { for (let at = line.indexOf(call); at >= 0; at = line.indexOf(call, at + 1)) { if (code[offset + at]) return true; } return false; }
/// ```
fn names_the_call_in_code(
    line: &str,
    offset: usize,
    code: &[bool],
    call: &str,
) -> bool {
    let mut searched: usize = 0;
    while let Some(place) = line[searched..].find(call) {
        // What: the place of this occurrence in the line, counted from the line's start.
        // Why: the code answer is kept for the whole file, so a line's place is added to the line's own offset.
        let column: usize = searched + place;
        if code.get(offset + column) == Some(&true) {
            return true;
        }
        searched = column + call.len();
    }
    return false;
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
/// function controls(text) { const code = isCode(text); const found = []; let current; let offset = 0; for (const [index, raw] of lines(text)) { const line = raw.replace(/\n$/, ''); if (line.startsWith('#[test]')) { if (current) found.push(current); current = { startsAt: index + 1, name: '', namesTheLedger: false, startsAChild: false }; } if (current) { current.name ||= functionNameOf(line) ?? ''; current.namesTheLedger ||= LEDGER_NAMES.some(([n]) => namesTheCallInCode(line, offset, code, n)); current.startsAChild ||= namesTheCallInCode(line, offset, code, IN_A_CHILD_OF_ITS_OWN); } offset += raw.length; } if (current) found.push(current); return found; }
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
        if line.starts_with(A_CONTROL_STARTS_AT) {
            if let Some(finished) = current.take() {
                found.push(finished);
            }
            current = Some(Control {
                starts_at: index + 1,
                name: String::new(),
                names_the_ledger: false,
                starts_a_child: false,
            });
        }
        if let Some(control) = current.as_mut() {
            if control.name.is_empty()
                && let Some(name) = function_name_of(line)
            {
                control.name = name;
            }
            for (call, _why) in LEDGER_NAMES {
                if names_the_call_in_code(line, offset, code.as_slice(), call) {
                    control.names_the_ledger = true;
                }
            }
            if names_the_call_in_code(line, offset, code.as_slice(), IN_A_CHILD_OF_ITS_OWN) {
                control.starts_a_child = true;
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
///  `findings_of(path, found)` is every control among `found` that names the ledger without starting a child of
///  its own,
///  as `path:line (control)`.
/// Why:
///  The finding is the whole answer the controls of this check need:
///  the real tree's files must give none,
///  and a planted text must give exactly the control that asks in the run's own process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsOf(path, found) { return found.filter((control) => control.namesTheLedger && !control.startsAChild).map((control) => `${path}:${control.startsAt} (${control.name})`); }
/// ```
fn findings_of(
    path: &str,
    found: &[Control],
) -> Vec<String> {
    let mut findings: Vec<String> = Vec::new();
    for control in found {
        if !control.names_the_ledger || control.starts_a_child {
            continue;
        }
        findings.push(format!("{path}:{} ({})", control.starts_at, control.name));
    }
    return findings;
}

/// What:
///  `Scan` is what reading every test-only source of a tree answers:
///  each control that names the ledger in the run's own process,
///  and each file that holds a control naming the ledger.
/// Why:
///  The findings are what must be empty,
///  and the files are what says the scan still reads the readers it was written for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scan = { findings: string[]; filesNamingTheLedger: string[] };
/// ```
struct Scan {
    /// What:
    ///  each control that names the ledger without a child of its own,
    ///  as `path:line (control)`.
    /// Why:
    ///  one finding is one control the check refuses.
    findings: Vec<String>,
    /// What:
    ///  each file that holds a control naming the ledger.
    /// Why:
    ///  these are the files a run must still find,
    ///  so the check is not vacuous.
    files_naming_the_ledger: Vec<String>,
}

/// What:
///  `scan(root, roots)` reads every test-only source under `root` and answers what `Scan` states,
///  or one text saying why the scan cannot answer.
/// Why:
///  Release code notes into the ledger and is no finding:
///  the wrapper's own preparation is what the ledger is for,
///  and only a control must ask its question in a child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(root, roots) { const findings = []; const filesNamingTheLedger = []; for (const file of classifiedSources(root, roots).filter((f) => f.testOnly)) { const found = controls(read(file)); if (found.some((c) => c.namesTheLedger)) filesNamingTheLedger.push(file.path); findings.push(...findingsOf(file.path, found)); } return { findings, filesNamingTheLedger }; }
/// ```
fn scan(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Scan, String> {
    let mut scan: Scan = Scan {
        findings: Vec::new(),
        files_naming_the_ledger: Vec::new(),
    };
    for file in classified_sources(root, roots)? {
        if !file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        let found: Vec<Control> = controls(text.as_str());
        if found.iter().any(|control| return control.names_the_ledger) {
            scan.files_naming_the_ledger.push(file.path.clone());
        }
        scan.findings
            .extend(findings_of(file.path.as_str(), found.as_slice()));
    }
    return Ok(scan);
}

/// No control reads the preparation's steps in the run's own process.
/// A control that asks about them starts a child of its own first.
#[test]
fn no_control_reads_the_preparation_steps_in_the_run_s_own_process() {
    let scan: Scan = scan(&source_root(), &real_roots()).expect("the crate's sources");
    assert!(
        scan.findings.is_empty(),
        "the preparation's steps are one ledger for the whole process, and a test run holds its controls as \
         threads of it, so a control that asks about them reads another control's work: start the control's body \
         in a child of its own first (`run_the_body_in_a_child_of_its_own`, \
         `testing/process_wide_state_child_test_support.rs`, issue 6507):\n{}",
        scan.findings.join("\n")
    );
}

/// Every file whose controls ask about the ledger is still read,
/// so a run that finds no reader fails here instead of passing every tree.
#[test]
fn the_files_that_read_the_ledger_still_do() {
    let scan: Scan = scan(&source_root(), &real_roots()).expect("the crate's sources");
    let missing: Vec<String> = FILES_THAT_READ_THE_LEDGER
        .iter()
        .filter(|(path, _why)| return !scan.files_naming_the_ledger.contains(&String::from(*path)))
        .map(|(path, why)| return format!("{path}: {why}"))
        .collect();
    assert!(
        missing.is_empty(),
        "these files hold no control that names the preparation's steps any more, so this check guards nothing \
         there: move each entry to the file that holds such a control, or delete it:\n{}",
        missing.join("\n")
    );
}

/// The scan finds a control that asks in the run's own process,
/// and says nothing about the control beside it that starts a child first,
/// nor about a call a comment or a planted text only names.
#[test]
fn the_scan_finds_a_control_that_asks_without_a_child_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn asks_in_the_run_s_own_process() {\n\
    let taken = crate::prepare_work_spans::drain_spans();\n\
}\n\
\n\
#[test]\n\
fn asks_in_a_child_of_its_own() {\n\
    /// The doc comment names drain_spans( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"asks_in_a_child_of_its_own\", || {\n\
        let taken = crate::prepare_work_spans::drain_spans();\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_name_in_a_text() {\n\
    let planted: &str = \"drain_spans(); prepareSpans\";\n\
}\n";
    assert_eq!(
        findings_of("planted_tests.rs", controls(planted).as_slice()),
        vec![String::from(
            "planted_tests.rs:1 (asks_in_the_run_s_own_process)"
        )],
        "the planted control that asks in the run's own process was not the one finding"
    );
}
