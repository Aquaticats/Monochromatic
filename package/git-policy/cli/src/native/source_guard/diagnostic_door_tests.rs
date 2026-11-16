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
///  The file the stop of a push lives in,
/// which is where the path this guard follows begins.
/// Why:
///   The property is about the code a stop runs,
/// and the file of `stop_push` is the only place that knows where that
///  code is;
/// a type named in a file was the earlier reading of it,
/// and it covered the two files that name the type and missed the
///  seven sites that say the words without naming it (finding 2 of the review of `41b3ca5e5e`).
const THE_STOP: &str = "push/stop_of_a_push_and_sweep_of_its_locks.rs";

/// What:
///  The writes of a program's own streams:
/// the shapes that wait for a reader where the caller stands,
/// which is the
///  wait a stop may not have.
/// Why:
///   A write to a file,
/// to a child's pipe or to a stream the caller handed in is the caller's own business,
/// so this
///  list is the process's own standard error and standard output and nothing else:
/// the two handles,
/// the four terse
///  macros,
/// `write_all(` on a stream this file holds,
/// and `libc::write` with a descriptor of its own.
/// The review of `41b3ca5e5e`
///  planted `eprint!`,
/// `println!`,
/// `libc::write` and a helper in another file,
/// and every one of them is on this list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_WRITE_OF_THE_PROGRAMS_OWN_STREAMS = ['std::io::stderr()', 'std::io::stdout()', 'eprintln!(', 'eprint!(', 'println!(', 'print!(', 'write_all(', 'libc::write('];
/// ```
const A_WRITE_OF_THE_PROGRAMS_OWN_STREAMS: [&str; 8] =
    ["std::io::stderr()", "std::io::stdout()", "eprintln!(", "eprint!(", "println!(", "print!(", "write_all(", "libc::write("];

/// What:
///  The files on the stop's path that may write a person's own stream,
/// each with the reason it may.
/// Why:
///   The decision of 2026-11-15 is that a one-shot command's own last words keep an ordinary blocking write
///  (`doc/handover/cli-git-rust-implementation.md`,
/// "Owner decisions in force",
/// "Only where another command can be held"),
/// and the
///  words of the stacked push,
/// of the merge it reports and of the watcher that tells a person a step is slow are exactly
///  those.
/// A file is on this list only while it still has one of the writes,
/// which the control below proves,
/// so no
///  entry outlives the write it excuses.
const ONE_SHOTS_OWN_WORDS: [(&str, &str); 4] = [
    ("push/push_stacking.rs", "the stacked push reports its own failure to the person and ends; no stop waits behind it"),
    ("push/stacked_merge_push.rs", "the merge the wrapper runs reports its outcome to the person and ends; no stop waits behind it"),
    ("push/auto_push_reconcile_branch_advance.rs", "the failed-merge settle hands the person's standard error to the words it says once, and no stop waits behind it"),
    ("push/child_watch_clock_marks_and_stopping.rs", "the notices of a watched step are written by its watcher thread, never by the thread a stop runs on"),
];

/// What:
///  Every release file on the stop's path:
/// the folder the stop lives in,
/// and every file the stop's own file names,
///  as `/`-separated identities under `src/native`.
/// Why:
///   The code a stop runs is the code of its own module and the modules that module names,
/// and the modules that
///  module's siblings name are the rest of the folder;
/// so the folder and the stop file's own names are the path this
///  guard follows,
/// no more and no less.
/// What it does not follow is said:
/// a name the stop file carries that is no file is a finding,
/// and the
///  count of files read is part of the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theStopsPath = (files) => [...files.filter((f) => f.path.startsWith('push/')), ...files.filter((f) => names(f.text).includes(f.stem))];
/// ```
fn the_stops_path(files: &[(String, String)]) -> (Vec<String>, Vec<String>) {
    let mut reached: Vec<String> = Vec::new();
    for (path, _text) in files {
        if path.starts_with("push/") {
            reached.push(path.clone());
        }
    }
    let stop_text: &str = files.iter().find(|(path, _text)| { return path == THE_STOP; }).map(|(_path, text)| { return text.as_str(); }).unwrap_or_default();
    let mut unfollowed: Vec<String> = Vec::new();
    for name in crate_names(stop_text) {
        let mut found: bool = false;
        for (path, _text) in files {
            let stem: &str = path.rsplit('/').next().unwrap_or(path).trim_end_matches(".rs");
            if stem == name {
                reached.push(path.clone());
                found = true;
            }
        }
        if !found {
            unfollowed.push(name);
        }
    }
    reached.sort();
    reached.dedup();
    unfollowed.sort();
    return (reached, unfollowed);
}

/// What:
///  The module names a file says with `crate::`,
/// each once.
/// Why:
///   A call into the crate is written as `crate::module::thing` or imported as `use crate::module::thing`,
/// and the module's file is
///  the unit this guard can find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const crateNames = (text) => [...new Set([...text.matchAll(/crate::([a-z_][a-z0-9_]*)/g)].map((m) => m[1]))];
/// ```
fn crate_names(text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut rest: &str = text;
    while let Some(at) = rest.find("crate::") {
        let after: &str = &rest[at + "crate::".len()..];
        let name: String = after.chars().take_while(|character| { return character.is_ascii_lowercase() || character.is_ascii_digit() || *character == '_'; }).collect();
        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
        rest = after;
    }
    return names;
}

/// What:
///  What the stop's path gets wrong,
///  as one sentence per finding,
///  and nothing when every file of it says its lines through the door and writes no stream of its own.
/// Why:
///  The check is a function of the text so a control can plant each shape and read the finding,
/// which is what proves the
///  check can fail at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const findingsOfTheStopsPath = (files) => { const [reached, unfollowed] = theStopsPath(files); return [...unfollowed.map((n) => `cannot follow crate::${n}`), ...reached.flatMap((p) => excused(p) ? [] : shapesOf(p).filter((s) => text(p).includes(s)).map((s) => `${p} carries ${s}`))]; };
/// ```
fn findings_of_the_stops_path(files: &[(String, String)]) -> Vec<String> {
    let (reached, unfollowed): (Vec<String>, Vec<String>) = the_stops_path(files);
    let mut found: Vec<String> = Vec::new();
    for name in unfollowed {
        found.push(format!("cannot follow `crate::{name}`: no release file is named that, so the path of the stop may go through code this guard cannot read"));
    }
    for path in reached {
        if ONE_SHOTS_OWN_WORDS.iter().any(|(excused, _reason)| { return excused == &path; }) {
            continue;
        }
        let Some((_path, text)) = files.iter().find(|(other, _text)| { return other == &path; }) else {
            continue;
        };
        let code: String = code_only(text.as_str());
        for shape in A_WRITE_OF_THE_PROGRAMS_OWN_STREAMS {
            if code.contains(shape) {
                found.push(format!("{path} carries `{shape}`, a write that waits for a reader where the caller stands"));
            }
        }
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
///  Every release file on the stop's path says its lines through the door and writes no stream of its own,
/// and every file the path names is a file this guard can read.
/// Why:
///   The path is the code a stop runs (`the_stops_path`),
/// and a write to the process's own standard error or standard output on it
///  waits for a reader where the caller stands,
/// which is the one wait a stop may not have (finding 2 of the review of `41b3ca5e5e`).
#[test]
fn every_file_on_the_stops_path_writes_no_stream_of_its_own() {
    let files: Vec<(String, String)> = release_sources();
    let (reached, unfollowed): (Vec<String>, Vec<String>) = the_stops_path(files.as_slice());
    let findings: Vec<String> = findings_of_the_stops_path(files.as_slice());
    assert!(reached.len() > 1, "the path is the stop's folder and the modules its file names, so it holds files: {}", reached.len());
    eprintln!("the stop's path: {} files; not followed: {unfollowed:?}", reached.len());
    assert_eq!(
        findings,
        Vec::<String>::new(),
        "every file of the stop's path says its lines through the door and writes no stream of its own"
    );
}

/// What:
///  Each shape the review of `41b3ca5e5e` planted on the stop's path is a finding of this check.
/// Why:
///   The guard stayed green for six of the eight shapes it was meant to catch:
/// a plain write in a file that does not name the type,
///  `eprint!`,
/// `println!`,
/// `libc::write`,
/// a helper in another file,
/// and a file whose one `warn(` excused the
///  rest of it.
/// Every plant here is put in a copy of the stop's folder in memory,
/// so each shape is read as it would be in a
///  real file and nothing is written to the worktree.
#[test]
fn each_shape_the_review_planted_on_the_stops_path_is_a_finding() {
    const PLANTED: &str = "push/bounded_query.rs";
    for (shape, plant) in [
        ("write_all(", "fn say() { let _ = std::io::stderr().write_all(b\"x\"); }\n"),
        ("eprint!(", "fn say() { eprint!(\"x\"); }\n"),
        ("println!(", "fn say() { println!(\"x\"); }\n"),
        ("libc::write(", "fn say() { let _ = unsafe { libc::write(2, b\"x\".as_ptr().cast(), 1) }; }\n"),
        ("std::io::stdout()", "fn say() { let _ = std::io::stdout(); }\n"),
    ] {
        let planted: Vec<(String, String)> = planted_into(PLANTED, plant);
        let found: Vec<String> = findings_of_the_stops_path(planted.as_slice());
        assert!(
            found.iter().any(|sentence| return sentence.contains(PLANTED) && sentence.contains(shape)),
            "the check names `{shape}` planted in {PLANTED}: {found:?}"
        );
    }
}

/// What:
///  A write reached through a helper in another file is a finding of this check,
///  and so is a name on the stop's path that this guard cannot follow.
/// Why:
///   The review's fourth plant was a call of a helper in another file that does the write,
/// and its fifth was a call
///  the guard could not see at all.
/// A helper the stop file names is read;
/// a name it carries that is no file is a finding,
/// so the check never
///  stays green on code it could not read.
#[test]
fn a_helper_in_another_file_is_read_and_a_name_that_is_no_file_is_a_finding() {
    let files: Vec<(String, String)> = release_sources();
    let mut with_helper: Vec<(String, String)> = files.clone();
    with_helper.push((String::from("lock/a_helper_the_stop_names.rs"), String::from("fn say() { let _ = std::io::stderr().write_all(b\"x\"); }\n")));
    // The stop file names the helper, as it names every module it calls.
    for (path, text) in with_helper.iter_mut() {
        if path == THE_STOP {
            text.push_str("\n// the control names the helper as the stop would:\ncrate::a_helper_the_stop_names::say();\n");
        }
    }
    let found: Vec<String> = findings_of_the_stops_path(with_helper.as_slice());
    assert!(
        found.iter().any(|sentence| return sentence.contains("lock/a_helper_the_stop_names.rs") && sentence.contains("write_all(")),
        "a helper the stop names is on the path and is read: {found:?}"
    );
    let mut with_unknown: Vec<(String, String)> = files.clone();
    for (path, text) in with_unknown.iter_mut() {
        if path == THE_STOP {
            text.push_str("\ncrate::a_module_no_file_holds::say();\n");
        }
    }
    let found: Vec<String> = findings_of_the_stops_path(with_unknown.as_slice());
    assert!(
        found.iter().any(|sentence| return sentence.contains("cannot follow `crate::a_module_no_file_holds`")),
        "a name the guard cannot follow is said and not passed over: {found:?}"
    );
}

/// What:
///  Every file the guard excuses for the person's own words still carries one of the writes,
///  and every one of its findings is a sentence with a reason.
/// Why:
///   An excuse that outlives its write is a hole a later write can hide in,
/// and an excuse without a reason is one nobody can
///  judge.
#[test]
fn every_excused_file_still_carries_the_write_it_is_excused_for() {
    for (path, reason) in ONE_SHOTS_OWN_WORDS {
        assert!(!reason.trim().is_empty(), "the excuse of {path} carries a reason");
        let text: String = std::fs::read_to_string(source_root().join(path)).unwrap_or_else(|error| panic!("{path} cannot be read: {error}"));
        let code: String = code_only(text.as_str());
        assert!(
            A_WRITE_OF_THE_PROGRAMS_OWN_STREAMS.iter().any(|shape| { return code.contains(shape); }),
            "{path} is excused for a write it no longer makes, so the excuse is stale"
        );
    }
}

/// What:
///  The release sources with `plant` appended to `path`,
/// so a control reads a plant where a real file would be.
/// Why:
///   A plant in the worktree would be a change nobody asked for;
/// a plant in memory is the same text to the check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plantedInto = (path, plant) => releaseSources().map(([p, t]) => [p, p === path ? t + plant : t]);
/// ```
fn planted_into(path: &str, plant: &str) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = release_sources();
    for (other, text) in files.iter_mut() {
        if other == path {
            text.push('\n');
            text.push_str(plant);
        }
    }
    return files;
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
