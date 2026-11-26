//! What:
//!  One scan for the bans on a control that reads a fact belonging to the whole process:
//!  a control that names such a fact in its code must also own the process it asks in,
//!  by starting a child of its own or by taking the window that fact lives in.
//! Why:
//!  `cargo test` runs a crate's tests as threads of one process,
//!  so a ledger,
//!  a control window or the process's own children are one for every test of the run,
//!  and a control that reads one passes alone and fails threaded
//!  (issues 6507 and 6509).
//!  Every such ban splits a test-only source into its controls the same way,
//!  so the split and the reading of the code are written once,
//!  here,
//!  and each ban states only its own names:
//!  `no_control_reads_the_preparation_steps_in_the_run_tests.rs`,
//!  `no_control_reads_the_commit_ledgers_in_the_run_tests.rs`
//!  and `notice_controls_take_their_window_tests.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Requirement = { what; named: [needle, why][]; excusedBy: call[]; required; files: [path, why][] };
//! // scan(requirement) => { findings: string[]; filesReading: string[] }
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
///  `Requirement` is one ban:
///  the fact it is about,
///  each name a control reaches that fact by with why that name is the fact's,
///  each call that answers the ban,
///  what a control must do instead,
///  and each file whose controls read the fact.
/// Why:
///  The bans differ only in their names,
///  so one table per ban states the whole difference,
///  and a run that stops seeing a reader fails here instead of passing every tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Requirement = { what: string; named: ReadonlyArray<readonly [string, string]>; excusedBy: readonly string[]; required: string; files: ReadonlyArray<readonly [string, string]> };
/// ```
pub(super) struct Requirement {
    /// What the fact is,
    /// as a finding names it.
    pub(super) what: &'static str,
    /// Each call or module path that reaches the fact,
    /// with why that name is the fact's.
    pub(super) named: &'static [(&'static str, &'static str)],
    /// Each call that answers the ban:
    /// it starts a child process of the control's own,
    /// or takes the window the fact lives in.
    pub(super) excused_by: &'static [&'static str],
    /// What a control that reads the fact must do,
    /// as the finding says it.
    pub(super) required: &'static str,
    /// Each file whose controls still read the fact,
    /// with what its controls ask.
    pub(super) files: &'static [(&'static str, &'static str)],
}

/// What:
///  `Control` is one control of a test-only source:
///  the line its `#[test]` is on,
///  its function's name,
///  whether its code names the fact,
///  and whether its code is excused.
/// Why:
///  A finding has to name the control and the line to open,
///  and a scan of loose lines could not say which control asks the question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Control = { startsAt: number; name: string; namesTheFact: boolean; isExcused: boolean };
/// ```
struct Control {
    /// The line of the control's `#[test]`,
    /// counted from one,
    /// which its findings are placed against.
    starts_at: usize,
    /// The name of the control's function,
    /// empty until the scan reaches its `fn` line.
    name: String,
    /// Whether any line of the control names the fact in code.
    names_the_fact: bool,
    /// Whether any line of the control is excused in code.
    is_excused: bool,
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
///  `controls(text, requirement)` splits one test-only source into its controls,
///  in the order they appear.
/// Why:
///  Each control is judged on its own lines,
///  so one control that is excused never excuses another that is not.
/// Gotcha:
///  The text is split on its newlines and every offset is the byte count the file holds,
///  which is exact because this crate's sources end their lines with one `\n`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function controls(text, requirement) { const code = isCode(text); const found = []; let current; let offset = 0; for (const [index, raw] of lines(text)) { const line = raw.replace(/\n$/, ''); if (line.startsWith('#[test]')) { if (current) found.push(current); current = { startsAt: index + 1, name: '', namesTheFact: false, isExcused: false }; } if (current) { current.name ||= functionNameOf(line) ?? ''; current.namesTheFact ||= requirement.named.some(([n]) => namesTheCallInCode(line, offset, code, n)); current.isExcused ||= requirement.excusedBy.some((n) => namesTheCallInCode(line, offset, code, n)); } offset += raw.length; } if (current) found.push(current); return found; }
/// ```
fn controls(
    text: &str,
    requirement: &Requirement,
) -> Vec<Control> {
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
                names_the_fact: false,
                is_excused: false,
            });
        }
        if let Some(control) = current.as_mut() {
            if control.name.is_empty()
                && let Some(name) = function_name_of(line)
            {
                control.name = name;
            }
            for (call, _why) in requirement.named {
                if names_the_call_in_code(line, offset, code.as_slice(), call) {
                    control.names_the_fact = true;
                }
            }
            for call in requirement.excused_by {
                if names_the_call_in_code(line, offset, code.as_slice(), call) {
                    control.is_excused = true;
                }
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
///  `findings_of(path, found)` is every control among `found` that names the fact and is not excused,
///  as `path:line (control)`.
/// Why:
///  The finding is the whole answer a ban's control needs:
///  the real tree's files must give none,
///  and a planted text must give exactly the control that asks in the run's own process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsOf(path, found) { return found.filter((control) => control.namesTheFact && !control.isExcused).map((control) => `${path}:${control.startsAt} (${control.name})`); }
/// ```
fn findings_of(
    path: &str,
    found: &[Control],
) -> Vec<String> {
    let mut findings: Vec<String> = Vec::new();
    for control in found {
        if !control.names_the_fact || control.is_excused {
            continue;
        }
        findings.push(format!("{path}:{} ({})", control.starts_at, control.name));
    }
    return findings;
}

/// What:
///  `findings_in_text(requirement, path, text)` is every control of `text` that reads the fact without owning
///  its process.
/// Why:
///  A ban's own control plants one control that reads and one that is excused,
///  and those two lines are the whole proof that the ban refuses what it says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsInText(requirement, path, text) { return findingsOf(path, controls(text, requirement)); }
/// ```
pub(super) fn findings_in_text(
    requirement: &Requirement,
    path: &str,
    text: &str,
) -> Vec<String> {
    return findings_of(path, controls(text, requirement).as_slice());
}

/// What:
///  `Scan` is what reading every test-only source of the crate answers:
///  each control that reads a fact in the run's own process,
///  and each file that holds such a control.
/// Why:
///  The findings are what must be empty,
///  and the files are what says the scan still reads the readers it was written for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scan = { findings: string[]; filesReading: string[] };
/// ```
pub(super) struct Scan {
    /// Each control that reads the fact without owning its process,
    /// as `path:line (control)`.
    pub(super) findings: Vec<String>,
    /// Each file that holds a control naming the fact.
    pub(super) files_reading: Vec<String>,
}

/// What:
///  `scan(requirement)` reads every test-only source under the crate's source root and answers what `Scan`
///  states,
///  or one text saying why the scan cannot answer.
/// Why:
///  Release code notes into a ledger and is no finding:
///  the wrapper's own work is what the ledger is for,
///  and only a control must ask its question in a process of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(requirement) { const findings = []; const filesReading = []; for (const file of classifiedSources(sourceRoot(), realRoots()).filter((f) => f.testOnly)) { const found = controls(read(file), requirement); if (found.some((c) => c.namesTheFact)) filesReading.push(file.path); findings.push(...findingsOf(file.path, found)); } return { findings, filesReading }; }
/// ```
pub(super) fn scan(requirement: &Requirement) -> Result<Scan, String> {
    let root: PathBuf = source_root();
    let roots: Vec<CrateRoot> = real_roots();
    let mut scan: Scan = Scan {
        findings: Vec::new(),
        files_reading: Vec::new(),
    };
    for file in classified_sources(root.as_path(), roots.as_slice())? {
        if !file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        let found: Vec<Control> = controls(text.as_str(), requirement);
        if found.iter().any(|control| return control.names_the_fact) {
            scan.files_reading.push(file.path.clone());
        }
        scan.findings
            .extend(findings_of(file.path.as_str(), found.as_slice()));
    }
    return Ok(scan);
}

/// What:
///  `missing_reader_files(scan, requirement)` is every file `requirement` names as a reader that the scan did not
///  find reading,
///  as `path: why`.
/// Why:
///  A scan that found no reader of a fact would pass every tree and guard nothing,
///  so a run that stops seeing these files fails instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingReaderFiles(scan, requirement) { return requirement.files.filter(([path]) => !scan.filesReading.includes(path)).map(([path, why]) => `${path}: ${why}`); }
/// ```
pub(super) fn missing_reader_files(
    scan: &Scan,
    requirement: &Requirement,
) -> Vec<String> {
    let mut missing: Vec<String> = Vec::new();
    for (path, why) in requirement.files {
        let found: bool = scan.files_reading.iter().any(|reading| {
            return reading == path;
        });
        if !found {
            missing.push(format!("{path}: {why}"));
        }
    }
    return missing;
}
