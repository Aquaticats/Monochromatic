//! What:
//!  Guard for the commit path's notes:
//!  no release file of this crate that says a note of the commit path writes standard error on the
//!  command's own thread.
//! Why:
//!  A write to a pipe whose reader stopped draining waits in the kernel until it resumes,
//!  so a note written where the commit stands holds a commit that may already have landed
//!  (`SPEC.md`;
//!  the passage on the notes the commit path writes on standard error;
//!  issue 6421).
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
//! A note file is found rather than remembered:
//! every release source that names the hand-over's own vocabulary is one,
//! so the split that moved `write_stderr` out of `commit/commit_command.rs` into
//! `commit/commit_command_words.rs` covered the receiver the day it landed
//! (issue 6599).
//! The lane is the vocabulary a file names,
//! so a file that writes the shared handle without naming any of its words is one this guard does not read;
//! `THE_NOTE_FILES` fails a file that leaves the lane,
//! and a reader asks whether every file of the commit path that writes a note still names one of the three words.
//! Three files of the lane talk to the shared handle for reasons that are not a note,
//! and `MAY_HOLD` records each use by the line it is written on:
//! two arms of the measured stream,
//! and the sentence a refused summary leaves.
//! The handle is banned by its name where a line spells the call or the path (`std::io::stderr(`,
//! `io::stderr(`,
//! `use std::io::stderr as say;`,
//! `use std::io::{stderr as say};`) and where a type position names it (`Stderr`).
//! What this ban does not see is a call a macro of another file expands to,
//! a name a glob import (`use super::*;`) brings into scope,
//! and a call whose name and `(` a line break separates;
//! a reader checks each of those in a file that adds one.
//! The shared handle is read by its descriptor number too (`libc::STDERR_FILENO`),
//! which is the same handle and holds it the same way;
//! a write to the number itself (`libc::write(2, ...)`) names no constant and no call,
//! and this ban reads nothing of it.
//! The terse ways to say a note are read as well:
//! `eprintln!`,
//! `eprint!` and `dbg!` (`THE_TERSE_WAYS_TO_SAY_A_NOTE`),
//! and the paths that open the same handle (`THE_SHARED_HANDLE_BY_PATH`).
//! `dbg!` is the macro that looks least like a note:
//! it writes standard error through the shared handle on the caller's own thread,
//! and neither of the other two says so in its name.
//! `eprintln!` is refused in every release source by the door's own guard
//! (`source_guard/diagnostic_door_tests.rs`);
//! this guard reads it in the files that say a note,
//! so a note file that spells it fails here too,
//! and this is the guard that says the note was written on the command's own thread.
//! A path this ban misses is one built from parts (`concat!("/dev/", "stderr")`)
//! and one a variable holds rather than a literal,
//! and a macro written with a space before its `!` (`eprintln !("one")`) is no call this reading spells.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(releaseSourcesHolding(NOTE_LANE).flatMap(([file, text]) => linesNoEntryExcuses([...linesCallingOneOf([[file, text]], THE_SHARED_HANDLE_CALLS), ...linesNamingOneOf([[file, text]], THE_SHARED_HANDLE_TYPES)], MAY_HOLD))).toEqual([]);
//! ```

use super::{
    HoldingLine,
    MayHold,
    entries_that_match_nothing,
    lines_calling_one_of,
    lines_holding_one_of,
    lines_naming_one_of,
    lines_no_entry_excuses,
    release_sources_holding,
};

/// What:
///  `NOTE_LANE` is the vocabulary that makes a release file a note file:
///  the hand-over's type,
///  the measured stream built over it,
///  and the two writers `commit/commit_command_words.rs` hands a line to.
/// Why:
///  A file that says a note names one of these,
///  so the set of note files is read from the tree instead of from a list somebody maintains,
///  and a receiver of a split is a note file by its own text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NOTE_LANE = ['StandardErrorOffTheCallingThread', 'MeasuredStandardError', 'write_stderr'];
/// ```
const NOTE_LANE: &[&str] = &[
    "StandardErrorOffTheCallingThread",
    "MeasuredStandardError",
    "write_stderr",
];

/// What:
///  `THE_SHARED_HANDLE_CALLS` is the call that asks for this process's shared standard error rather than a stream of
///  its own.
/// Why:
///  A note written through the shared handle waits for the reader where the command stands,
///  and it is also one lock for the whole process.
///  The call is read by its name where a line spells it as a call or as a path's last segment,
///  so `std::io::stderr(`,
///  `io::stderr(` and the import a rename is made from are findings,
///  while a local variable named `stderr` and the field `output.stderr` are not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_SHARED_HANDLE_CALLS = ['stderr'];
/// ```
const THE_SHARED_HANDLE_CALLS: &[&str] = &["stderr"];

/// What:
///  `THE_SHARED_HANDLE_TYPES` is the type a file names when it keeps the shared handle in a value (the counted stream a
///  measured write is made through).
/// Why:
///  The type is named in a `use` list and in a signature as well as at a call,
///  so it is read wherever a line writes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_SHARED_HANDLE_TYPES = ['Stderr'];
/// ```
const THE_SHARED_HANDLE_TYPES: &[&str] = &["Stderr"];

/// What:
///  `THE_SHARED_HANDLE_BY_NUMBER` is the constant that names this process's standard error by its descriptor
///  number,
///  which writes to the same handle as `std::io::stderr()` and holds it the same way.
/// Why:
///  a note written with `libc::write(libc::STDERR_FILENO, ...)` asks for the shared handle without writing either
///  of its names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_SHARED_HANDLE_BY_NUMBER = ['STDERR_FILENO'];
/// ```
const THE_SHARED_HANDLE_BY_NUMBER: &[&str] = &["STDERR_FILENO"];

/// What:
///  `THE_TERSE_WAYS_TO_SAY_A_NOTE` is the three macros that write standard error without asking for a stream.
/// Why:
///  `eprintln!`,
///  `eprint!` and `dbg!` write this process's shared standard error on the caller's own thread,
///  which is the write this ban exists for,
///  and none names the handle,
///  the type or the descriptor number.
///  The door's own guard refuses `eprintln!` in every release source;
///  this list is what says a note file may not spell any of the three either.
///  `dbg!` was read by no rule of this crate until the alpha.53 no-change read (its L5),
///  and it is the one of the three whose name says nothing about a stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_TERSE_WAYS_TO_SAY_A_NOTE = ['eprintln!', 'eprint!', 'dbg!'];
/// ```
const THE_TERSE_WAYS_TO_SAY_A_NOTE: &[&str] = &["eprintln!", "eprint!", "dbg!"];

/// What:
///  `THE_SHARED_HANDLE_BY_PATH` is every path a file can open this process's standard error by.
/// Why:
///  `File::create("/dev/stderr")` and `std::fs::write("/dev/fd/2", ..)` reach the same handle the call
///  and the descriptor constant reach,
///  and they name none of them.
///  `/proc/self/fd/2` is the same descriptor under the name the kernel gives it,
///  which is what `/dev/stderr` resolves to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_SHARED_HANDLE_BY_PATH = ['/dev/stderr', '/dev/fd/2', '/proc/self/fd/2'];
/// ```
const THE_SHARED_HANDLE_BY_PATH: &[&str] = &["/dev/stderr", "/dev/fd/2", "/proc/self/fd/2"];

/// What:
///  `THE_NOTE_FILES` is every file `main` says a note of the commit path from.
/// Why:
///  The lane is found by its vocabulary,
///  and a file that stops naming it leaves the lane without saying so:
///  this list is what fails then,
///  so a split that moves the last hand-over out of one of these names the file that holds it now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_NOTE_FILES = ['commit/commit_command.rs', 'landing/service_commit_input_notes_and_embedded_commit.rs', ...];
/// ```
const THE_NOTE_FILES: &[&str] = &[
    "commit/commit_command.rs",
    "invocation/wrapped_command_notices_snapshots_and_manual_push.rs",
    "landing/service_commit_input_notes_and_embedded_commit.rs",
    "landing/service_commit_rebuild_ceiling.rs",
    "landing/service_commit_requests_and_landing_report.rs",
    "landing/service_commit_the_answers_of_the_service.rs",
    "landing/service_landed_aftermath.rs",
];

/// What:
///  `MAY_HOLD` is every use of the shared handle a note file may keep,
///  each entry naming the line it is written on and the reason it may.
/// Why:
///  The measured stream is built over any stream,
///  and the shared one is the other arm of that measurement rather than a note of the commit path;
///  the sentence a commit says when its summary could not be printed is the whole of what a person gets there,
///  and it is not a note the command can live without.
///  An entry whose line went fails here instead of excusing the file for ever,
///  and a second use of either text in that file fails,
///  because the entry names one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAY_HOLD = [{ file: 'foundation/time_spent_outside_the_phases.rs', text: 'stderr', line: '...', why: '...' }];
/// ```
const MAY_HOLD: &[MayHold] = &[
    MayHold {
        file: "foundation/time_spent_outside_the_phases.rs",
        text: "stderr",
        line: "inner: std::io::stderr(),",
        why: "the measured stream's other arm is built over this process's own standard error, which is what the \
              timing record counts; it says no note of the commit path",
    },
    MayHold {
        file: "foundation/time_spent_outside_the_phases.rs",
        text: "Stderr",
        line: "impl MeasuredStandardError<std::io::Stderr> {",
        why: "the same arm, named as the type the measured stream is built over",
    },
    MayHold {
        file: "landing/service_commit_input_notes_and_embedded_commit.rs",
        text: "stderr",
        line: "let _reported = writeln!(std::io::stderr(), \"fatal: write error: {error}\");",
        why: "a commit that cannot say its summary ends as Git does, with `fatal: write error` on standard error and \
              status 128, and that sentence is the whole of what a person gets there; the entry goes when the \
              summary is printed another way",
    },
];

/// What:
///  `note_files()` is every release source that says a note,
///  each as its identity and its text.
/// Why:
///  The set is the tree's answer,
///  so a new note file joins the guard by being written and no receiver of a split is left behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noteFiles = () => releaseSourcesHolding(NOTE_LANE);
/// ```
fn note_files() -> Vec<(String, String)> {
    return release_sources_holding(NOTE_LANE).expect("the crate's release sources");
}

/// What:
///  `writes_where_the_command_stands(sources)` is every code line of a note file that asks for the shared handle.
/// Why:
///  The two shapes are the same mistake,
///  and the sentence a refused summary leaves is excused by the line `MAY_HOLD` names rather than by the words it
///  holds,
///  so no line is dropped before the list is asked what it excuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writesWhereTheCommandStands = (sources) => [...linesCallingOneOf(sources, THE_SHARED_HANDLE_CALLS), ...linesCallingOneOf(sources, THE_TERSE_WAYS_TO_SAY_A_NOTE), ...linesNamingOneOf(sources, THE_SHARED_HANDLE_TYPES), ...linesNamingOneOf(sources, THE_SHARED_HANDLE_BY_NUMBER), ...linesHoldingOneOf(sources, THE_SHARED_HANDLE_BY_PATH)];
/// ```
fn writes_where_the_command_stands(sources: &[(String, String)]) -> Vec<HoldingLine> {
    let mut holding: Vec<HoldingLine> = lines_calling_one_of(sources, THE_SHARED_HANDLE_CALLS);
    holding.extend(lines_calling_one_of(sources, THE_TERSE_WAYS_TO_SAY_A_NOTE));
    holding.extend(lines_naming_one_of(sources, THE_SHARED_HANDLE_TYPES));
    holding.extend(lines_naming_one_of(sources, THE_SHARED_HANDLE_BY_NUMBER));
    holding.extend(lines_holding_one_of(sources, THE_SHARED_HANDLE_BY_PATH));
    return holding;
}

/// What:
///  `files_the_lane_no_longer_holds(sources)` is every file `THE_NOTE_FILES` names that the lane no longer answered
///  with.
/// Why:
///  A file that leaves the lane is read by no rule of this guard,
///  which is exactly the hole the split left,
///  so its leaving has to fail here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filesTheLaneNoLongerHolds = (sources) => THE_NOTE_FILES.filter((name) => !sources.some(([file]) => file === name));
/// ```
fn files_the_lane_no_longer_holds(sources: &[(String, String)]) -> Vec<String> {
    return THE_NOTE_FILES
        .iter()
        .filter(|name| {
            return !sources.iter().any(|(file, _)| {
                return file == *name;
            });
        })
        .map(|name| return String::from(*name))
        .collect();
}

/// No note file of the commit path writes standard error where the command stands.
#[test]
fn no_note_of_the_commit_path_is_written_on_the_command_s_own_thread() {
    let sources: Vec<(String, String)> = note_files();
    let unexplained: Vec<String> = lines_no_entry_excuses(
        writes_where_the_command_stands(sources.as_slice()).as_slice(),
        MAY_HOLD,
    );
    assert!(
        unexplained.is_empty(),
        "a note of the commit path is handed to the thread that writes standard error, never written by the \
         command's own thread:\n{}",
        unexplained.join("\n")
    );
}

/// Every `MAY_HOLD` entry still excuses a line,
/// so an arm that went leaves the list no longer than it is.
#[test]
fn every_entry_of_the_list_still_excuses_a_line() {
    let sources: Vec<(String, String)> = note_files();
    let stale: Vec<String> = entries_that_match_nothing(
        writes_where_the_command_stands(sources.as_slice()).as_slice(),
        MAY_HOLD,
    );
    assert!(
        stale.is_empty(),
        "an entry names a line that no longer holds its text, so the list can only shrink:\n{}",
        stale.join("\n")
    );
}

/// The lane still reaches every file `main` says a note of the commit path from.
#[test]
fn the_lane_still_reads_every_file_that_says_a_note() {
    let sources: Vec<(String, String)> = note_files();
    let left: Vec<String> = files_the_lane_no_longer_holds(sources.as_slice());
    assert!(
        left.is_empty(),
        "these files name no word of the note lane, so this guard reads nothing in them: move each entry to the \
         file that says the note now, or delete it:\n{}",
        left.join("\n")
    );
}

/// What:
///  Every spelling of a note file's ask for the shared handle:
///  the path the standard library spells,
///  the path of an
///  `io` import,
///  the import a rename is made from,
///  the type kept in a value,
///  the descriptor number,
///  the three macros that write it,
///  and the paths that open it.
/// Why:
///  The review of release 0.6.1-alpha.50 found that `use std::io; io::stderr().write_all(..)` held neither text the
///  ban read (issue 6613),
///  the read of release 0.6.1-alpha.52 found the macros and the paths (its L2),
///  and the read of release 0.6.1-alpha.53 found `dbg!` (its L5).
const SPELLINGS_OF_THE_SHARED_HANDLE: &[&str] = &[
    "let _ = writeln!(std::io::stderr(), \"one\");",
    "let _ = io::stderr().write_all(b\"one\");",
    "use std::io::stderr as say;",
    "fn kept(handle: std::io::Stderr) {}",
    "let _ = unsafe { libc::write(libc::STDERR_FILENO, bytes.as_ptr().cast(), bytes.len()) };",
    "eprintln!(\"one\");",
    "eprint!(\"one\");",
    "dbg!(one);",
    "let _ = std::fs::File::create(\"/dev/stderr\");",
    "let _ = std::fs::write(\"/dev/fd/2\", b\"one\");",
    "let _ = std::fs::File::create(\"/proc/self/fd/2\");",
];

/// What:
///  The lines a note file may still write:
///  a stream of its own,
///  and a variable or a field that says `stderr`.
/// Why:
///  The name is read where the line spells the call,
///  so a local variable and a field that say the word are no ask for the shared handle.
const NO_ASK_FOR_THE_SHARED_HANDLE: &[&str] = &[
    "let mut stderr: String = String::new();",
    "let _ = stderr.push_str(line);",
    "let _ = refusal.write_all(output.stderr.as_slice());",
    "let _ = write_stderr(line);",
];

/// The scan fails on a note file that asks for the shared standard error,
/// in every spelling,
/// and it fails on a second use in the file an entry excuses,
/// because that entry names one line.
#[test]
fn the_check_fails_on_a_note_file_that_takes_the_shared_standard_error() {
    for spelling in SPELLINGS_OF_THE_SHARED_HANDLE {
        let planted: Vec<(String, String)> = vec![(
            String::from("landing/a_note.rs"),
            format!("fn say(line: &str) {{\n    {spelling}\n}}\n"),
        )];
        let found: Vec<String> = lines_no_entry_excuses(
            writes_where_the_command_stands(planted.as_slice()).as_slice(),
            MAY_HOLD,
        );
        assert_eq!(
            found.len(),
            1,
            "{spelling} asks for the shared handle: {found:?}"
        );
    }
    for fine in NO_ASK_FOR_THE_SHARED_HANDLE {
        let planted: Vec<(String, String)> = vec![(
            String::from("landing/a_note.rs"),
            format!("fn say(line: &str) {{\n    {fine}\n}}\n"),
        )];
        assert!(
            lines_no_entry_excuses(
                writes_where_the_command_stands(planted.as_slice()).as_slice(),
                MAY_HOLD
            )
            .is_empty(),
            "{fine} asks for no shared handle"
        );
    }
    let excused_file: Vec<(String, String)> = vec![(
        String::from("foundation/time_spent_outside_the_phases.rs"),
        String::from(
            "use crate::standard_error_off_the_calling_thread::MeasuredStandardError;\nfn say() { let _ = writeln!(std::io::stderr(), \"one\"); }\nfn also() { let _ = writeln!(std::io::stderr(), \"two\"); }\n",
        ),
    )];
    let excused: Vec<String> = lines_no_entry_excuses(
        writes_where_the_command_stands(excused_file.as_slice()).as_slice(),
        MAY_HOLD,
    );
    assert_eq!(
        excused.len(),
        2,
        "a second write in the file the entry excuses fails, because the entry names one line: {excused:?}"
    );
}

/// What:
///  The sentence a refused summary leaves is excused where `MAY_HOLD` names its line,
///  and the same sentence in another file is a finding.
/// Why:
///  An exemption that named only the words excused every note file that said them,
///  with no file and no use,
///  which is the hole the review found (issue 6613).
#[test]
fn the_check_excuses_the_refused_summary_in_the_file_the_entry_names() {
    let named: Vec<(String, String)> = vec![(
        String::from("landing/service_commit_input_notes_and_embedded_commit.rs"),
        String::from(
            "fn print_summary() {\n    let _reported = writeln!(std::io::stderr(), \"fatal: write error: {error}\");\n}\n",
        ),
    )];
    assert_eq!(
        lines_no_entry_excuses(
            writes_where_the_command_stands(named.as_slice()).as_slice(),
            MAY_HOLD
        ),
        Vec::<String>::new(),
        "the sentence a commit says when its summary could not be printed is the use the entry names"
    );
    let elsewhere: Vec<(String, String)> = vec![(
        String::from("landing/another_note.rs"),
        String::from(
            "fn print_summary() {\n    let _reported = writeln!(std::io::stderr(), \"fatal: write error: {error}\");\n}\n",
        ),
    )];
    assert_eq!(
        lines_no_entry_excuses(
            writes_where_the_command_stands(elsewhere.as_slice()).as_slice(),
            MAY_HOLD
        )
        .len(),
        1,
        "the sentence in a file no entry names is a finding"
    );
}

/// A file that says no note is read by no rule of this guard,
/// and a file that says one is read by both.
#[test]
fn the_lane_is_what_the_hand_over_makes_it() {
    let lane: Vec<(String, String)> =
        release_sources_holding(NOTE_LANE).expect("the crate's release sources");
    assert!(
        !lane.is_empty(),
        "the note lane reached no release file, so this guard passes every tree"
    );
    let notes: Vec<&str> = lane
        .iter()
        .filter(|(_, text)| return text.contains("StandardErrorOffTheCallingThread"))
        .map(|(file, _)| return file.as_str())
        .collect();
    assert!(
        notes.contains(&"commit/commit_command.rs"),
        "the commit command hands no note over: {notes:?}"
    );
    assert!(
        !notes.contains(&"foundation/earlier_names_folded_into_current.rs"),
        "a file that writes the shared handle itself is no note file: {notes:?}"
    );
}
