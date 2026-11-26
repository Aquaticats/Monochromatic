//! What:
//!  The ban on a control asking a question about the whole kill registry without a process of its own.
//! Why:
//!  `binary/binary_supervisor.rs` keeps one registry for the process,
//!  and the binary tests run as threads of one process under `cargo test`,
//!  so a control that asks `unclaimed_kill` or `kill_to_abort` reads the records every other control planted.
//!  Two controls of a threaded run failed with each other's message,
//!  and a control that fails leaves its planted record unclaimed,
//!  which made the process-wide monitor abort the whole run and take libtest's failure summary with it
//!  (issue 6485).
//!  A control that starts `ran_in_a_registry_of_its_own` first asks its question in a child process of the test binary,
//!  whose registry holds nothing but what that control planted.
//!  This check is a text scan and not a run:
//!  the control it guards is in another test target,
//!  and the crate's source-scan base is already here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control that names `unclaimed_kill(` or `kill_to_abort(` must also name `ran_in_a_registry_of_its_own(`.
//! ```

/// What:
///  `THE_CONTROLS_TEXT` is the whole text of `binary/binary_supervisor_tests.rs`,
///  embedded when this check is built.
/// Why:
///  The check reads that file's controls and never runs them,
///  and `include_str!` reads the text even though that file belongs to another test target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theControlsText = readFileSync(join(__dirname, '../binary/binary_supervisor_tests.rs'), 'utf8');
/// ```
const THE_CONTROLS_TEXT: &str = include_str!("../binary/binary_supervisor_tests.rs");

/// What:
///  `THE_CONTROLS` is the path a finding names,
///  relative to `src/native`,
///  which is how this crate's other findings name a file.
/// Why:
///  A reader opens the path a finding gives from the crate's own folder,
///  without the `../` the `include_str!` needs.
const THE_CONTROLS: &str = "binary/binary_supervisor_tests.rs";

/// What:
///  `REGISTRY_WIDE_QUESTIONS` is the slice of calls whose answer is about every record in the process's registry,
///  rather than about one process number the caller named.
/// Why:
///  Those two are the calls that read another control's record,
///  and naming them in one place keeps the check's subject visible beside the code it guards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registryWideQuestions = ['unclaimed_kill(', 'kill_to_abort('];
/// ```
const REGISTRY_WIDE_QUESTIONS: &[&str] = &["unclaimed_kill(", "kill_to_abort("];

/// What:
///  `IN_A_REGISTRY_OF_ITS_OWN` is the call that starts one control in a child process of the test binary.
/// Why:
///  A control of the read file that makes it asks its registry-wide question where nothing else planted a record,
///  so a control naming such a question and not this call is the finding.
const IN_A_REGISTRY_OF_ITS_OWN: &str = "ran_in_a_registry_of_its_own(";

/// What:
///  `A_CONTROL_STARTS_AT` is the line that begins one control of the read file.
/// Why:
///  Every control of that file is a top-level function marked by this line,
///  so the text from one of these lines to the next is exactly one control's body.
const A_CONTROL_STARTS_AT: &str = "#[test]";

/// What:
///  `struct Control` is one control of the read text:
///  the line its `#[test]` is on,
///  its function's name,
///  and every line it spans.
/// Why:
///  The finding must name the control and the line to open,
///  and a scan of loose lines could not say which control a question belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Control = { startsAt: number; name: string; lines: string[] };
/// ```
struct Control<'a> {
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
    ///  every line of the control,
    ///  borrowed from the read text.
    /// Why:
    ///  the scan looks for calls in these lines alone,
    ///  so a call in one control never excuses another.
    lines: Vec<&'a str>,
}

/// What:
///  `fn function_name_of(line: &str) -> Option<String>` is the name of the function a line declares,
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
    // What:     `?` answers `None` from this function as soon as the line is not a function declaration.
    // Why:      the scan walks every line of a control, and most of them declare nothing.
    let rest: &str = trimmed.strip_prefix("fn ")?;
    // What:     the same early answer where the line has no opening parenthesis after the name.
    // Why:      the name ends at that parenthesis, and a line without one holds no name to report.
    let (name, _): (&str, &str) = rest.split_once('(')?;
    return Some(String::from(name.trim()));
}

/// What:
///  `fn controls(text: &str) -> Vec<Control<'_>>` splits `text` into its controls,
///  in the order they appear.
/// Why:
///  Each control is judged on its own lines,
///  so one control that starts a child never excuses another that does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function controls(text) { const found = []; let current; for (const [i, line] of text.split('\n').entries()) { if (line.startsWith('#[test]')) { if (current) found.push(current); current = { startsAt: i + 1, name: '', lines: [] }; } if (current) { current.lines.push(line); if (!current.name) current.name = functionNameOf(line) ?? ''; } } if (current) found.push(current); return found; }
/// ```
fn controls(text: &str) -> Vec<Control<'_>> {
    let mut found: Vec<Control<'_>> = Vec::new();
    let mut current: Option<Control<'_>> = None;
    for (index, line) in text.lines().enumerate() {
        if line.starts_with(A_CONTROL_STARTS_AT) {
            if let Some(finished) = current.take() {
                found.push(finished);
            }
            current = Some(Control {
                starts_at: index + 1,
                name: String::new(),
                lines: Vec::new(),
            });
        }
        let Some(control) = current.as_mut() else {
            continue;
        };
        control.lines.push(line);
        // What:     the control's name is on the `fn` line just below its `#[test]`,
        //           and it stays empty until the scan reaches that line.
        // Why:      a later `fn` line of the same control (a closure's or a helper's) must not rename it.
        if control.name.is_empty()
            && let Some(name) = function_name_of(line)
        {
            control.name = name;
        }
    }
    if let Some(finished) = current {
        found.push(finished);
    }
    return found;
}

/// What:
///  `fn call_lines(control: &Control<'_>, call: &str) -> Vec<usize>` is the line numbers of `control` that make `call`.
/// Why:
///  A doc comment of a control names the calls it asks about,
///  and a comment is no finding,
///  so the lines a comment holds are skipped before the search.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const callLines = (control, call) => control.lines.flatMap((line, offset) => line.trimStart().startsWith('//') || !line.includes(call) ? [] : [control.startsAt + offset]);
/// ```
fn call_lines(
    control: &Control<'_>,
    call: &str,
) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    for (offset, line) in control.lines.iter().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        if line.contains(call) {
            found.push(control.starts_at + offset);
        }
    }
    return found;
}

/// What:
///  `fn findings_in(text: &str) -> Vec<String>` is every control of `text` that asks a registry-wide question
///  without starting a process of its own,
///  as `path:line (control)`.
/// Why:
///  The finding is the whole answer the controls that read it need:
///  the real file's text must give none,
///  and a planted text must give exactly the control that asks the question in the run's own process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsIn(text) { const found = []; for (const control of controls(text)) { for (const question of registryWideQuestions) { const asked = callLines(control, question); if (asked.length === 0) continue; if (callLines(control, inARegistryOfItsOwn).length > 0) continue; found.push(`${theControls}:${asked[0]} (${control.name})`); break; } } return found; }
/// ```
fn findings_in(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for control in controls(text) {
        for question in REGISTRY_WIDE_QUESTIONS {
            let asked: Vec<usize> = call_lines(&control, question);
            let Some(first) = asked.first() else {
                continue;
            };
            if !call_lines(&control, IN_A_REGISTRY_OF_ITS_OWN).is_empty() {
                continue;
            }
            found.push(format!("{THE_CONTROLS}:{first} ({})", control.name));
            break;
        }
    }
    return found;
}

/// No control of the supervisor's own file asks a question about the whole kill registry in the process the run holds.
#[test]
fn no_control_asks_a_registry_wide_question_in_the_run_s_own_process() {
    let found: Vec<String> = findings_in(THE_CONTROLS_TEXT);
    assert!(
        found.is_empty(),
        "the kill registry belongs to the whole process, and the binary tests run as threads of it, so a control that \
         asks about every record reads what another control planted: start the control in a child of its own first \
         (`ran_in_a_registry_of_its_own`):\n{}",
        found.join("\n")
    );
}

/// The supervisor's controls still ask both registry-wide questions,
/// so a run that finds none of them reads no control at all and says nothing.
#[test]
fn the_supervisor_controls_still_ask_the_registry_wide_questions() {
    let missing: Vec<&str> = REGISTRY_WIDE_QUESTIONS
        .iter()
        .filter(|question| {
            return !THE_CONTROLS_TEXT.contains(*question);
        })
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "the controls no longer name these questions, so this check guards nothing: move it to the file that holds \
         them or delete it:\n{}",
        missing.join("\n")
    );
}

/// The scan finds a control that asks a registry-wide question in the run's own process,
/// and says nothing about the control beside it that starts a child first.
#[test]
fn the_scan_finds_a_control_that_asks_without_a_process_of_its_own() {
    let planted: &str = "\
#[test]\n\
fn asks_in_the_run_s_own_process() {\n\
    let answer = unclaimed_kill(now, grace);\n\
}\n\
\n\
#[test]\n\
fn asks_in_a_registry_of_its_own() {\n\
    if ran_in_a_registry_of_its_own(\"asks_in_a_registry_of_its_own\") {\n\
        return;\n\
    }\n\
    let answer = kill_to_abort(now, grace);\n\
}\n";
    assert_eq!(
        findings_in(planted),
        vec![String::from(
            "binary/binary_supervisor_tests.rs:3 (asks_in_the_run_s_own_process)"
        )],
        "the planted control that asks in the run's own process was not the one finding"
    );
}
