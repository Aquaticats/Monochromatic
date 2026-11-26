//! What:
//!  Guard for the commit path's notes:
//!  no file of this crate that says one writes standard error on the command's own thread.
//! Why:
//!  A write to a pipe whose reader stopped draining waits in the kernel until it resumes,
//!  so a note written where the commit stands holds a commit that may already have landed
//!  (`SPEC.md`, the passage that begins "That note, and every other note the commit path writes on
//!  standard error"; issue 6421).
//!  The behavioural controls of `src/native/binary/` show one note at a time with a full pipe,
//!  and two of the sites have no control that can run on every host:
//!  a branch whose name is not text cannot be created on a file system that refuses such a name,
//!  and a service that goes away between its answer and the hand-off is an instant no phase marker names.
//!  This guard holds those two and every site whose control somebody later removes:
//!  a note file that asks for the shared handle again fails here.
//!  `std::io::stderr()` is also one lock for the whole process
//!  (`foundation/standard_error_descriptor.rs`),
//!  so a note written through it would hold every later line of the process behind the reader.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(filesSayingTheStream(shippedCommitPathNotes()).flatMap(findings)).toEqual([]);
//! ```

use crate::source_guard::source_scan_tests::source_root;

/// What:
///  Every release file that says a note of the commit path,
///  as its `/`-separated identity under `src/native`.
/// Why:
///  These are the files the notes of a commit and of the steps before it are said from,
///  and each one hands its lines to the thread that writes standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NOTES = ['commit/commit_command.rs', 'landing/service_commit.rs', ...];
/// ```
const NOTE_FILES: &[&str] = &[
    "commit/commit_command.rs",
    "landing/service_commit.rs",
    "landing/service_commit_input_notes_and_embedded_commit.rs",
    "landing/service_commit_rebuild_ceiling.rs",
    "landing/service_commit_requests_and_landing_report.rs",
    "landing/service_landed_aftermath.rs",
    "invocation/wrapped_command_notices_snapshots_and_manual_push.rs",
];

/// What:
///  The words a note file carries to hand a line to the thread that writes it.
/// Why:
///  A file that says notes names no stream of its own, so it names this one;
///  a file that lost the name is a file whose lines went back to the shared handle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_HAND_OVER = 'StandardErrorOffTheCallingThread';
/// ```
const THE_HAND_OVER: &str = "StandardErrorOffTheCallingThread";

/// What:
///  The shapes that ask for this process's shared standard error rather than a stream of their own.
/// Why:
///  A note written through the shared handle waits for the reader where the command stands,
///  and it is also one lock for the whole process;
///  `std::io::Stderr` is the type a file names when it keeps the handle in a value (the counted
///  stream a measured write is made through), so both shapes are the same mistake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_SHARED_HANDLE = ['std::io::stderr(', 'std::io::Stderr'];
/// ```
const THE_SHARED_HANDLE: &[&str] = &["std::io::stderr(", "std::io::Stderr"];

/// What:
///  The one write a note file may still make on the command's own thread:
///  the sentence a commit says when its standard output refused the summary it had to print.
/// Why:
///  A commit that cannot say its summary ends as Git does,
///  with `fatal: write error` on standard error and status 128,
///  and that sentence is the whole of what a person gets there;
///  it is not a note the command can live without.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_ONE_ALLOWED = 'fatal: write error';
/// ```
const THE_ONE_ALLOWED: &str = "fatal: write error";

/// What:
///  The lines of a file that are not comments,
///  joined as one text.
/// Why:
///  A comment says what the code may not do as often as a file says what it does,
///  and a check that read the comments would forbid the explanation of its own rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const codeOnly = (text) => text.split('\n').filter((line) => !line.trim().startsWith('//')).join('\n')
/// ```
fn code_only(text: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        lines.push(line);
    }
    return lines.join("\n");
}

/// What:
///  What one note file's text gets wrong,
///  as one sentence per finding,
///  and nothing when it hands its lines over and writes no stream of its own.
/// Why:
///  The check is a function of the text so a control can plant a note file that asks for the shared
///  handle and read the finding, which is what proves the check can fail at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const findingsOfANoteFile = (path, text) => { const code = codeOnly(text); return [...(code.includes(THE_HAND_OVER) ? [] : [`${path} hands no line to the thread`]), ...code.split('\n').filter((l) => l.includes('std::io::stderr(') && !l.includes(THE_ONE_ALLOWED)).map((l) => `${path} writes standard error where the command stands: ${l}`)] };
/// ```
fn findings_of_a_note_file(
    path: &str,
    text: &str,
) -> Vec<String> {
    let code: String = code_only(text);
    let mut found: Vec<String> = Vec::new();
    if !code.contains(THE_HAND_OVER) {
        found.push(format!(
            "{path} names no `{THE_HAND_OVER}`, so its notes reach no thread of their own"
        ));
    }
    for line in code.lines() {
        let asks_for_the_handle: bool = THE_SHARED_HANDLE
            .iter()
            .any(|shape| return line.contains(*shape));
        if !asks_for_the_handle {
            continue;
        }
        if line.contains(THE_ONE_ALLOWED) {
            continue;
        }
        found.push(format!(
            "{path} writes standard error where the command stands: {line}"
        ));
    }
    return found;
}

/// What:
///  Every finding of every note file of this crate.
/// Why:
///  One call for the whole class, so a new note file joins the guard by being named once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const findingsOfTheNotes = (files) => files.flatMap(([path, text]) => findingsOfANoteFile(path, text));
/// ```
fn findings_of_the_notes(files: &[(String, String)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (path, text) in files {
        found.extend(findings_of_a_note_file(path, text.as_str()));
    }
    return found;
}

/// What:
///  The note files of this crate,
///  each read as its `/`-separated identity and its text.
/// Why:
///   The guard reads the sources that ship, not a list of remembered lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noteFiles = () => NOTES.map((path) => [path, readFile(join(sourceRoot(), path))]);
/// ```
fn note_files() -> Vec<(String, String)> {
    let root: std::path::PathBuf = source_root();
    let mut files: Vec<(String, String)> = Vec::new();
    for path in NOTE_FILES {
        let full: std::path::PathBuf = root.join(path);
        let text: String = std::fs::read_to_string(full.as_path())
            .unwrap_or_else(|error| panic!("{} cannot be read: {error}", full.display()));
        files.push((String::from(*path), text));
    }
    return files;
}

/// No note file of the commit path writes standard error where the command stands.
#[test]
fn no_note_of_the_commit_path_is_written_on_the_command_s_own_thread() {
    assert_eq!(
        findings_of_the_notes(note_files().as_slice()),
        Vec::<String>::new(),
        "a note of the commit path is handed to the thread that writes standard error, never written by the commit's own thread"
    );
}

/// The check fails on a note file that asks for the shared standard error.
#[test]
fn the_check_fails_on_a_note_file_that_takes_the_shared_standard_error() {
    let planted: Vec<(String, String)> = vec![(
        String::from("landing/a_note.rs"),
        String::from(
            "fn say(line: &str) { let mut notes: StandardErrorOffTheCallingThread<fn(&[u8])> = StandardErrorOffTheCallingThread::new(); let _ = writeln!(std::io::stderr(), \"{line}\"); }\n",
        ),
    )];
    let found: Vec<String> = findings_of_the_notes(planted.as_slice());
    assert!(
        !found.is_empty(),
        "a note file that writes standard error where the command stands is a finding"
    );
    assert!(
        found
            .iter()
            .any(|sentence| return sentence.contains("std::io::stderr(")),
        "the finding names the write: {found:?}"
    );
}

/// The check fails on a note file that lost its hand-over.
#[test]
fn the_check_fails_on_a_note_file_with_no_hand_over() {
    let planted: Vec<(String, String)> = vec![(
        String::from("landing/a_note.rs"),
        String::from("fn say(line: &str) -> usize { return line.len(); }\n"),
    )];
    let found: Vec<String> = findings_of_the_notes(planted.as_slice());
    assert!(
        found
            .iter()
            .any(|sentence| return sentence.contains("names no `StandardErrorOffTheCallingThread`")),
        "a note file that hands its lines to no thread is a finding: {found:?}"
    );
}

/// The check answers nothing for the one sentence a refused summary leaves.
#[test]
fn the_check_allows_the_sentence_a_refused_summary_leaves() {
    let allowed: Vec<(String, String)> = vec![(
        String::from("landing/a_note.rs"),
        String::from(
            "fn print_summary() { let _ = writeln!(std::io::stderr(), \"fatal: write error: {error}\"); }\n",
        ),
    )];
    assert!(
        findings_of_the_notes(allowed.as_slice())
            .iter()
            .all(|sentence| return !sentence.contains("std::io::stderr(")),
        "the sentence a commit says when its summary could not be printed is allowed"
    );
    let mut with_hand_over: Vec<(String, String)> = allowed;
    with_hand_over[0]
        .1
        .push_str("fn hand() { let _ = StandardErrorOffTheCallingThread::new(); }\n");
    assert_eq!(
        findings_of_the_notes(with_hand_over.as_slice()),
        Vec::<String>::new(),
        "the allowed sentence and the hand-over are no finding at all"
    );
}
