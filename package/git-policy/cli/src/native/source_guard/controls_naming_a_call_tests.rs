//! What:
//!  The scan two guards share:
//!  one test-only source split into its controls,
//!  each control asked whether its code names one of a list of calls
//!  and whether its code starts a child of its own.
//! Why:
//!  A guard refuses a control that reaches a fact of the whole process without a child of its own,
//!  and two of them ask the same question about different calls:
//!  one about the preparation's ledger
//!  (`no_control_reads_the_preparation_steps_in_the_run_tests.rs`),
//!  and one about the interactive selection, whose Git reads the process's standard input
//!  (`no_control_starts_an_interactive_git_on_the_run_s_input_tests.rs`).
//!  Splitting a source into controls, telling its code from its comments and its text literals,
//!  and answering those two questions is the same work in both,
//!  so it is written once here
//!  (`AGENTS.md`'s UTL).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scan = scanForControlsNamingACall(root, roots, CALLS);
//! expect(scan.findings).toEqual([]); expect(scan.filesNamingACall).toContain('commit/commit_index_tests.rs');
//! ```

/// What:
///  `is_code` answers, for each byte of a source,
///  whether that byte is code rather than part of a comment or a text literal.
///  Its sibling a reader might expect is a lexer per guard,
///  which would answer the same bytes differently in each and could drift.
/// Why:
///  A control's doc comment names the calls it is not making,
///  and a control's fixture plants a call's text in a string,
///  and neither is a call,
///  so the file's own answer to which bytes are code decides.
use super::source_code_places_tests::is_code;
/// What:
///  `CrateRoot` is one crate root of the tree, and `classified_sources` reads the tree's files
///  with each one told apart as release code or test-only code.
///  The sibling a reader might expect is a plain recursive walk, which cannot say which files a test run compiles.
/// Why:
///  Only a control is judged here, so the scan has to know which sources a test build holds.
use crate::source_scan_tests::{
    CrateRoot,
    classified_sources,
};
/// What:
///  `Path` is a borrowed filesystem path.
///  The sibling a reader might expect is `PathBuf`, which owns the path it names.
/// Why:
///  The root the scan reads is the caller's, and nothing here keeps a path of its own.
use std::path::Path;

/// What:
///  `IN_A_CHILD_OF_ITS_OWN` is the call that starts one control's body in a child process of the test binary.
///  The sibling a reader might expect is the child helper's own module path,
///  which a guard's finding would then have to build from a file name.
/// Why:
///  A control that makes this call asks its question where nothing else of the run has set the process's own facts,
///  so a control that names a listed call and not this one is the finding.
pub(crate) const IN_A_CHILD_OF_ITS_OWN: &str = "started_within_a_child_of_its_own(";

/// What:
///  `A_CONTROL_STARTS_AT` is the line that begins one control of a test-only source.
/// Why:
///  Every control of this crate is a top-level function marked by this line,
///  so the text from one of these lines to the next is exactly one control's body.
pub(crate) const A_CONTROL_STARTS_AT: &str = "#[test]";

/// What:
///  `Control` is one control of a test-only source:
///  the line its `#[test]` is on,
///  its function's name,
///  whether its code names a call the caller listed,
///  and whether its code starts a child of its own.
/// Why:
///  A finding has to name the control and the line to open,
///  and a scan of loose lines could not say which control asks the question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Control = { startsAt: number; name: string; namesAListedCall: boolean; startsAChild: boolean };
/// ```
pub(crate) struct Control {
    /// What:
    ///  the line number of the control's `#[test]`,
    ///  counted from one,
    ///  which its findings are placed against.
    /// Why:
    ///  a finding is `path:line`,
    ///  and the line is where the control starts.
    pub(crate) starts_at: usize,
    /// What:
    ///  the name of the control's function,
    ///  empty until the scan reaches its `fn` line.
    /// Why:
    ///  the finding names the control,
    ///  so a reader knows which one to open.
    pub(crate) name: String,
    /// What:
    ///  whether any line of the control names a call the caller listed, in code.
    /// Why:
    ///  that is the question each guard is about.
    pub(crate) names_a_listed_call: bool,
    /// What:
    ///  whether any line of the control starts a child of its own in code.
    /// Why:
    ///  a control that makes that call asks its question in a process no other control of the run shares.
    pub(crate) starts_a_child: bool,
}

/// What:
///  `function_name_of(line)` is the name of the function a line declares,
///  or nothing when the line declares none.
///  `Option<String>` is "a name or nothing" (its sibling `Result` carries a reason for the nothing).
/// Why:
///  A control's name follows its `#[test]` by a line or two,
///  and the finding has to name the control rather than its position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const functionNameOf = (line: string): string | undefined => /^\s*fn ([A-Za-z0-9_]+)\(/.exec(line)?.[1];
/// ```
fn function_name_of(line: &str) -> Option<String> {
    // What: the line without the blanks at its start, so an indented `fn` reads like one at the margin.
    // Why: a `fn` inside a control's body is never a control, but the same cut keeps this reader simple.
    let trimmed: &str = line.trim_start();
    // What: the `?` answers `None` from this function as soon as the line opens with something else.
    // Why: the scan walks every line of a control, and most of them declare nothing.
    let rest: &str = trimmed.strip_prefix("fn ")?;
    // What: the same early answer where the line has no opening parenthesis after the name.
    // Why: the name ends at that parenthesis, and a line without one holds no name to report.
    let (name, _): (&str, &str) = rest.split_once('(')?;
    // What: `Some(...)` constructs the present variant of `Option`, wrapping the name, which is this
    //       function's tail expression and so its answer.
    // Why: the finding needs the control's name, and a line that is no declaration has none.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return name.trim();
    // ```
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
/// function namesTheCallInCode(line: string, offset: number, code: boolean[], call: string): boolean {
///   for (let at = line.indexOf(call); at >= 0; at = line.indexOf(call, at + 1)) {
///     if (code[offset + at]) return true;
///   }
///   return false;
/// }
/// ```
fn names_the_call_in_code(
    line: &str,
    offset: usize,
    code: &[bool],
    call: &str,
) -> bool {
    // What: how much of the line this walk has looked at, so the next search starts past this occurrence.
    // Why: a call named twice on one line is answered by its first occurrence, and the walk must end.
    let mut searched: usize = 0;
    while let Some(place) = line[searched..].find(call) {
        // What: the place of this occurrence in the line, counted from the line's start.
        // Why: the code answer is kept for the whole file, so a line's place is added to the line's own offset.
        let column: usize = searched + place;
        // What: `Some(&true)` is the code answer of that byte; `code.get` answers nothing past the file's end.
        // Why: a byte the answer does not cover is no call this guard can prove.
        if code.get(offset + column) == Some(&true) {
            return true;
        }
        searched = column + call.len();
    }
    // What: `false` is this function's tail expression, answering that the line names the call nowhere in code.
    // Why: the caller counts a control only when its code names the call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return false;
    // ```
    return false;
}

/// What:
///  `controls(text, calls)` splits one test-only source into its controls,
///  in the order they appear,
///  each asked whether its code names one of `calls`.
/// Why:
///  Each control is judged on its own lines,
///  so one control that starts a child never excuses another that does not.
/// Gotcha:
///  The text is split on its newlines and every offset is the byte count the file holds,
///  which is exact because this crate's sources end their lines with one `\n`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function controls(text: string, calls: [string, string][]): Control[] {
///   const code = isCode(text); const found: Control[] = []; let current: Control | undefined; let offset = 0;
///   for (const [index, raw] of lines(text)) {
///     const line = raw.replace(/\n$/, '');
///     if (line.startsWith('#[test]')) { if (current) found.push(current); current = { startsAt: index + 1, name: '', namesAListedCall: false, startsAChild: false }; }
///     if (current) {
///       current.name ||= functionNameOf(line) ?? '';
///       current.namesAListedCall ||= calls.some(([call]) => namesTheCallInCode(line, offset, code, call));
///       current.startsAChild ||= namesTheCallInCode(line, offset, code, IN_A_CHILD_OF_ITS_OWN);
///     }
///     offset += raw.length;
///   }
///   if (current) found.push(current);
///   return found;
/// }
/// ```
pub(crate) fn controls(
    text: &str,
    calls: &[(&str, &str)],
) -> Vec<Control> {
    // What: the code answer for every byte of the file, read once.
    // Why: reading it per line would read the whole file again for every line.
    let code: Vec<bool> = is_code(text);
    // What: the controls found so far, in the order their `#[test]` lines appear.
    // Why: a finding is reported in the order its control appears, so a run names them as a reader meets them.
    let mut found: Vec<Control> = Vec::new();
    // What: the control the walk is inside, if any; `Option` is "a control or nothing".
    // Why: a source starts with no control, and the first `#[test]` line is what opens one.
    let mut current: Option<Control> = None;
    // What: the byte offset in the file of the line about to be read.
    // Why: the code answer is kept for the whole file, so a line's place is added to this offset.
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
                names_a_listed_call: false,
                starts_a_child: false,
            });
        }
        if let Some(control) = current.as_mut() {
            if control.name.is_empty()
                && let Some(name) = function_name_of(line)
            {
                control.name = name;
            }
            for (call, _why) in calls {
                if names_the_call_in_code(line, offset, code.as_slice(), call) {
                    control.names_a_listed_call = true;
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
    // What: every control of the source, which is this function's tail expression and so its answer.
    // Why: the caller judges each control on its own, so the whole list is what it reads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return found;
    // ```
    return found;
}

/// What:
///  `findings_of(path, found)` is every control among `found` that names a listed call without starting a child of
///  its own,
///  as `path:line (control)`.
/// Why:
///  The finding is the whole answer the guards need:
///  the real tree's files must give none,
///  and a planted text must give exactly the control that asks in the run's own process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsOf(path: string, found: Control[]): string[] {
///   return found.filter((c) => c.namesAListedCall && !c.startsAChild).map((c) => `${path}:${c.startsAt} (${c.name})`);
/// }
/// ```
pub(crate) fn findings_of(
    path: &str,
    found: &[Control],
) -> Vec<String> {
    // What: the findings of one file, in the order its controls appear.
    // Why: a run that fails prints them as a list, and a reader opens them in that order.
    let mut findings: Vec<String> = Vec::new();
    for control in found {
        if !control.names_a_listed_call || control.starts_a_child {
            continue;
        }
        // What: `format!` builds the finding's text: the file, the line counted from one, and the control's name.
        // Why: the finding has to name the control and the line to open.
        findings.push(format!("{path}:{} ({})", control.starts_at, control.name));
    }
    return findings;
}

/// What:
///  `Scan` is what reading every test-only source of a tree answers:
///  each control that names a listed call in the run's own process,
///  and each file that holds a control naming one.
/// Why:
///  The findings are what must be empty,
///  and the files are what says the scan still reads the callers it was written for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scan = { findings: string[]; filesNamingACall: string[] };
/// ```
pub(crate) struct Scan {
    /// What:
    ///  each control that names a listed call without a child of its own,
    ///  as `path:line (control)`.
    /// Why:
    ///  one finding is one control a guard refuses.
    pub(crate) findings: Vec<String>,
    /// What:
    ///  each file that holds a control naming a listed call.
    /// Why:
    ///  these are the files a run must still find,
    ///  so a guard over them is not vacuous.
    pub(crate) files_naming_a_call: Vec<String>,
}

/// What:
///  `scan(root, roots, calls)` reads every test-only source under `root` and answers what `Scan` states,
///  or one text saying why the scan cannot answer.
/// Why:
///  Release code reaches these calls too and is no finding:
///  the wrapper's own preparation and its service are what the calls are for,
///  and only a control must ask its question in a child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(root: string, roots: CrateRoot[], calls: [string, string][]): Scan {
///   const findings: string[] = []; const filesNamingACall: string[] = [];
///   for (const file of classifiedSources(root, roots).filter((f) => f.testOnly)) {
///     const found = controls(read(file), calls);
///     if (found.some((c) => c.namesAListedCall)) filesNamingACall.push(file.path);
///     findings.push(...findingsOf(file.path, found));
///   }
///   return { findings, filesNamingACall };
/// }
/// ```
pub(crate) fn scan(
    root: &Path,
    roots: &[CrateRoot],
    calls: &[(&str, &str)],
) -> Result<Scan, String> {
    // What: the answer being filled, starting with no finding and no file.
    // Why: a tree that holds none of either must still answer, and an empty answer says so.
    let mut scan: Scan = Scan {
        findings: Vec::new(),
        files_naming_a_call: Vec::new(),
    };
    for file in classified_sources(root, roots)? {
        if !file.test_only {
            continue;
        }
        // What: the file's text, or the reason it could not be read, which `?` on `map_err` would also carry.
        // Why: a source the scan cannot read would leave a control unjudged.
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        // What: the file's controls, each already asked the two questions.
        // Why: both the findings and the file list are read from them.
        let found: Vec<Control> = controls(text.as_str(), calls);
        if found
            .iter()
            .any(|control| return control.names_a_listed_call)
        {
            scan.files_naming_a_call.push(file.path.clone());
        }
        scan.findings
            .extend(findings_of(file.path.as_str(), found.as_slice()));
    }
    // What: the answer, which is this function's tail expression.
    // Why: a guard prints its findings and lists its files, so it needs both.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return scan;
    // ```
    return Ok(scan);
}
