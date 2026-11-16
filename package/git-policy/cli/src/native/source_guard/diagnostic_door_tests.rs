//! What:
//!  Guard for the door every diagnostic line goes through:
//!  the door hands each line to the writer that never holds its caller and writes no stream of its own,
//!  that writer writes this process's standard error descriptor and never the shared handle,
//!  and no release file says a line with `eprintln!`.
//! Why:
//!  A diagnostic is said at whatever moment the code reaches it,
//!  inside a landing that holds the index lock or a push that holds its claim included,
//!  so a write that waits for the reader there holds what the command holds,
//!  and every command waiting for that thing waits with it
//!  (`doc/planning/cli-git-no-unbounded-waits.md`, "Writes to the caller's own streams").
//!  A source guard cannot see what a line is said under:
//! holding a lock is not a shape of the text,
//!  so what this guard holds is the shape that makes the question go away:
//!  one door, which never waits for the reader on the caller's thread,
//!  and one writer, which never takes the lock every other writer of this process shares.
//!  `std::io::stderr()` is one handle behind one lock for the whole process
//! (`standard_error_descriptor.rs`),
//! so a writer that took it would make a reader that stopped draining hold every later line of the process,
//!  which is the same harm one step further from the caller.
//  `eprintln!` is banned because it is the one terse way to say a diagnostic without reaching the door.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(findingsOfTheDoor(read(door))).toEqual([]); expect(findingsOfTheWriter(read(writer))).toEqual([]); expect(eprintlnInRelease(sources)).toEqual([]);
//! ```

use crate::source_guard::source_scan_tests::{real_roots, source_root};
use crate::source_guard::source_scan_tests::tree::classified_sources;

/// The file of the door every diagnostic level goes through.
const DOOR: &str = "foundation/diagnostic_log.rs";

/// The file of the thread that writes the lines.
const WRITER: &str = "foundation/diagnostic_lines_off_the_calling_thread.rs";

/// The words the door must carry, which is its call to the writer.
const THE_HAND_OVER: &str = "diagnostic_lines_off_the_calling_thread::write";

/// The words the writer must carry, which is the descriptor it writes rather than the shared handle.
const THE_DESCRIPTOR: &str = "StandardErrorDescriptor";

/// What:
///  The lines of a file that are not comments,
///  joined as one text.
/// Why:
///  A comment says what the code may not do as often as a file says what it does,
/// and a check that read the comments would forbid the explanation of its own rule.
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
///  The words that say a file writes a stream where its caller stands.
/// Why:
///   These are the shapes the door must not have:
/// a write to standard error,
///  a write of bytes to any stream,
///  and the terse line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_HAND_WRITE = ['stderr(', 'write_all(', 'eprintln!(']
/// ```
const A_HAND_WRITE: [&str; 3] = ["stderr(", "write_all(", "eprintln!("];

/// What:
///  What the door's own text gets wrong,
///  as one sentence per finding,
///  and nothing when it hands every line to the writer and writes no stream itself.
/// Why:
///  The check is a function of the text so a control can plant a door that writes the stream and read the finding,
///  which is what proves the check can fail at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const findingsOfTheDoor = (text) => [...A_HAND_WRITE.filter((word) => text.includes(word)), ...(text.includes(THE_HAND_OVER) ? [] : ['no hand-over'])]
/// ```
fn findings_of_the_door(text: &str) -> Vec<String> {
    let code: String = code_only(text);
    let mut found: Vec<String> = Vec::new();
    for word in A_HAND_WRITE {
        if code.contains(word) {
            found.push(format!("the door writes a stream itself: it carries `{word}`"));
        }
    }
    if !code.contains(THE_HAND_OVER) {
        found.push(String::from("the door says no line through `diagnostic_lines_off_the_calling_thread::write`"));
    }
    return found;
}

/// What:
///  What the writer's text gets wrong,
///  as one sentence per finding,
///  and nothing when it writes the descriptor and never the shared handle.
/// Why:
///  A writer that took `std::io::stderr()` would hold the process's one standard-error lock while it waits for a
/// reader,
///  and every later line of the process would wait behind it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const findingsOfTheWriter = (text) => [...(text.includes('stderr(') ? ['it takes the shared handle'] : []), ...(text.includes(THE_DESCRIPTOR) ? [] : ['it writes no descriptor'])]
/// ```
fn findings_of_the_writer(text: &str) -> Vec<String> {
    let code: String = code_only(text);
    let mut found: Vec<String> = Vec::new();
    if code.contains("stderr(") {
        found.push(String::from("the writer takes the process's shared standard error, whose lock a waiting write would hold"));
    }
    if !code.contains(THE_DESCRIPTOR) {
        found.push(String::from("the writer names no `StandardErrorDescriptor`, so it writes through some other stream"));
    }
    return found;
}

/// What:
///  The release files that say a line with `eprintln!`,
///  from one `(path, text)` pair per file.
/// Why:
///   `eprintln!` is the one terse way to say a diagnostic without reaching the door,
/// and the write it makes waits for the reader where the caller stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const eprintlnInRelease = (files) => files.filter(([, text]) => text.includes('eprintln!(')).map(([path]) => path)
/// ```
fn eprintln_in_release(files: &[(String, String)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (path, text) in files {
        if code_only(text.as_str()).contains("eprintln!(") {
            found.push(path.clone());
        }
    }
    return found;
}

/// What:
///  Every release source of this crate,
///  as its `/`-separated identity under `src/native` and the text of the file.
/// Why:
///  The module tree decides what is release code,
/// so a module declared under `#[cfg(test)]` is never read as one that ships.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const releaseSources = () => classifiedSources(sourceRoot(), realRoots()).filter((file) => !file.testOnly).map((file) => [file.path, read(file.path)])
/// ```
fn release_sources() -> Vec<(String, String)> {
    let root: std::path::PathBuf = source_root();
    let sources = classified_sources(root.as_path(), real_roots().as_slice()).expect("the crate's source tree");
    let mut release: Vec<(String, String)> = Vec::new();
    for file in sources {
        if file.test_only {
            continue;
        }
        let path: std::path::PathBuf = root.join(file.path.as_str());
        let text: String = std::fs::read_to_string(path.as_path()).unwrap_or_else(|error| panic!("{} cannot be read: {error}", path.display()));
        release.push((file.path, text));
    }
    return release;
}

/// The door hands every line over and writes no stream of its own.
#[test]
fn the_door_hands_every_line_to_the_writer_and_writes_no_stream_itself() {
    let door: String = std::fs::read_to_string(source_root().join(DOOR)).expect("the door's source");
    assert_eq!(
        findings_of_the_door(door.as_str()),
        Vec::<String>::new(),
        "a diagnostic must not be written where the command stands: it goes through the thread that never holds its caller"
    );
}

/// The writer writes the descriptor and never the process's shared standard error.
#[test]
fn the_writer_writes_the_descriptor_and_never_the_shared_standard_error() {
    let writer: String = std::fs::read_to_string(source_root().join(WRITER)).expect("the writer's source");
    assert_eq!(
        findings_of_the_writer(writer.as_str()),
        Vec::<String>::new(),
        "a writer that blocked in the shared standard error would hold every other line of this process"
    );
}

/// No release file says a line with `eprintln!`.
#[test]
fn no_release_file_says_a_line_with_eprintln() {
    assert_eq!(
        eprintln_in_release(release_sources().as_slice()),
        Vec::<String>::new(),
        "a line that waits for the reader where the command stands is what this guard exists to prevent; say it through the door"
    );
}

/// What:
///  A tree of one planted door and its writer,
///  under a directory of the control's own.
/// Why:
///   A check that cannot fail proves nothing,
/// so each control plants the mistake beside the file it covers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const planted = (root, door, writer) => { mkdir(root); write(root + '/diagnostic_log.rs', door); write(root + '/writer.rs', writer) }
/// ```
fn a_planted_tree(name: &str, door: &str, writer: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root: std::path::PathBuf = crate::test_support::fixture(name);
    let door_path: std::path::PathBuf = root.join("diagnostic_log.rs");
    let writer_path: std::path::PathBuf = root.join("writer.rs");
    std::fs::write(door_path.as_path(), door).expect("the planted door");
    std::fs::write(writer_path.as_path(), writer).expect("the planted writer");
    return (door_path, writer_path);
}

/// The door's check fails on a door that writes standard error on the calling thread.
#[test]
fn the_door_check_fails_on_a_door_that_writes_the_stream_itself() {
    let (door_path, _writer_path) = a_planted_tree(
        "diagnostic-door-planted-door",
        "fn write_line(line: &str) { let _ = std::io::stderr().write_all(line.as_bytes()); }\n",
        "const THE_DESCRIPTOR: &str = \"StandardErrorDescriptor\";\n",
    );
    let text: String = std::fs::read_to_string(door_path.as_path()).expect("the planted door");
    let found: Vec<String> = findings_of_the_door(text.as_str());
    assert!(!found.is_empty(), "a door that writes the stream itself is a finding");
    assert!(
        found.iter().any(|sentence| return sentence.contains("stderr(")),
        "the finding names the write the door must not make: {found:?}"
    );
    assert!(
        found.iter().any(|sentence| return sentence.contains("no line through")),
        "the finding names the hand-over the door lost: {found:?}"
    );
}

/// The writer's check fails on a writer that takes the shared standard error.
#[test]
fn the_writer_check_fails_on_a_writer_that_takes_the_shared_standard_error() {
    let (_door_path, writer_path) = a_planted_tree(
        "diagnostic-door-planted-writer",
        "fn write_line(line: &str) { crate::diagnostic_lines_off_the_calling_thread::write(line); }\n",
        "fn write_it(bytes: &[u8]) { let _ = std::io::stderr().write_all(bytes); }\n",
    );
    let text: String = std::fs::read_to_string(writer_path.as_path()).expect("the planted writer");
    assert!(!findings_of_the_writer(text.as_str()).is_empty(), "a writer that takes the shared handle is a finding");
}

/// The `eprintln!` check fails on a planted line, and answers nothing for no file at all.
#[test]
fn the_eprintln_check_fails_on_a_planted_line_and_ignores_test_support() {
    let planted: Vec<(String, String)> = vec![(
        String::from("landing/a_settling.rs"),
        String::from("fn say() { eprintln!(\"the trash was left as it is\"); }\n"),
    )];
    assert_eq!(
        eprintln_in_release(planted.as_slice()),
        vec![String::from("landing/a_settling.rs")],
        "a release file that says a line with the terse macro is a finding"
    );
    let nothing: Vec<(String, String)> = Vec::new();
    assert!(eprintln_in_release(nothing.as_slice()).is_empty(), "no file, no finding");
    let a_line_through_the_door: Vec<(String, String)> = vec![(String::from("landing/a_settling.rs"), String::from("fn say() { crate::service_log!(\"the trash was left as it is\"); }\n"))];
    assert!(eprintln_in_release(a_line_through_the_door.as_slice()).is_empty(), "a line through a door is no finding");
}
