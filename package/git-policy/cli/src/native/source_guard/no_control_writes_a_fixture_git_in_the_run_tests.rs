//! What:
//!  The ban on a control of the Git version cache writing its fixture Git in the run's own process:
//!  such a control runs its body in a child of its own first.
//! Why:
//!  The fixture Git an executable control keeps is a copy of the host's Git,
//!  and `copy_git`,
//!  `std::fs::copy`,
//!  `std::fs::write` and `OpenOptions::new()` open it for writing
//!  in the process that calls them.
//!  `execve` refuses a file that any process holds open for writing,
//!  and a `fork` by another thread duplicates the whole descriptor table,
//!  so a child another test forked inside that window inherits the write descriptor
//!  and holds it until its own exec closes the close-on-exec descriptor.
//!  Measured on 2026-11-26 (issue 6517):
//!  eight threaded runs of `invocation::git_version_cache::` failed two of them,
//!  one control each,
//!  with `Text file busy (os error 26)`,
//!  and a traced failing run showed the holder:
//!  thread 3302960 of the run opened its fixture for writing at 16:28:01.369937,
//!  thread 3302959 forked child 3302962 at .369964,
//!  the write descriptor was closed at .370006,
//!  the control forked its `git version` child at .372063,
//!  and that child's `execve` at .372208 was refused while child 3302962 still held the descriptor,
//!  its own exec running from .371950 to .372762.
//!  A scan of every process's descriptors at the moment the error surfaced found no holder:
//!  the descriptor lives only from the fork to the exec,
//!  so only the code shape can keep it out.
//!  A control that hands its body to
//!  `run_the_body_in_a_child_of_its_own` (`testing/process_wide_state_child_test_support.rs`)
//!  is the only test of its process,
//!  so no other thread forks between its write and its start.
//!  The split of a source into its controls is `source_guard/process_wide_fact_readers_tests.rs`,
//!  which this ban shares with the other bans of its shape;
//!  the ban reads one file,
//!  so it says its own text keeps naming the writes it exists for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every control of invocation/git_version_cache_tests.rs that names copy_git(, std::fs::copy(,
//! // std::fs::write( or OpenOptions::new() must also name run_the_body_in_a_child_of_its_own(.
//! ```

use super::process_wide_fact_readers_tests::{
    Requirement,
    findings_in_text,
};
use crate::source_scan_tests::source_root;

/// What:
///  `FIXTURE_WRITES` is each call a control of this module reaches its fixture Git's write descriptor by,
///  with why that call opens the file.
///  The siblings a reader might expect are `fixture(`,
///  which makes the folder and always writes a `.tmp-` file,
///  and `executable(`,
///  which writes the fixture through a child `tee` and so holds no descriptor here.
/// Why:
///  The descriptor and the fork it is inherited through are the whole defect,
///  so the calls that make the descriptor are what the ban has to find,
///  not one spelling of a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIXTURE_WRITES: ReadonlyArray<readonly [string, string]> = [['copy_git(', 'copies the host Git into the fixture']];
/// ```
const FIXTURE_WRITES: &[(&str, &str)] = &[
    (
        "copy_git(",
        "copies the host Git's bytes into the fixture, opening the copy for writing here",
    ),
    (
        "std::fs::copy(",
        "copies another program's bytes into the fixture, opening the copy for writing here",
    ),
    (
        "std::fs::write(",
        "rewrites the fixture Git in place, opening it for writing here",
    ),
    (
        "OpenOptions::new()",
        "opens the fixture Git for writing to put its modification time back",
    ),
];

/// What:
///  `IN_A_CHILD_OF_ITS_OWN` is the call that starts one control's body in a child process of the test binary.
/// Why:
///  A control that makes it writes its fixture as the only test of its process,
///  so a control that names a fixture write and not this call is the finding.
const IN_A_CHILD_OF_ITS_OWN: &str = "run_the_body_in_a_child_of_its_own(";

/// What:
///  `THE_FILE` is the one test source of this module,
///  which holds every control this ban is about.
/// Why:
///  The ban reads this file and no other,
///  because a `std::fs::write` in another test source usually writes a file nothing starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_FILE = 'invocation/git_version_cache_tests.rs';
/// ```
const THE_FILE: &str = "invocation/git_version_cache_tests.rs";

/// What:
///  `THE_VERSION_CACHE` is the ban itself:
///  the calls that open a fixture Git for writing,
///  the one call that answers the ban,
///  and what a control must do instead.
/// Why:
///  The scan in `process_wide_fact_readers_tests.rs` takes a requirement like this one,
///  so the ban states its own names and the scan stays the same for every ban of its shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_VERSION_CACHE = { what: "the fixture Git's write descriptor", named: FIXTURE_WRITES, excusedBy: [IN_A_CHILD_OF_ITS_OWN], required: '...', files: [] };
/// ```
const THE_VERSION_CACHE: Requirement = Requirement {
    what: "the fixture Git's write descriptor",
    named: FIXTURE_WRITES,
    excused_by: &[IN_A_CHILD_OF_ITS_OWN],
    required: "start the control's body in a child of its own first (`run_the_body_in_a_child_of_its_own`, \
               `testing/process_wide_state_child_test_support.rs`, issue 6517)",
    files: &[],
};

/// What:
///  The text of `THE_FILE` as this crate's sources hold it.
/// Why:
///  The ban reads one file,
///  so a file that cannot be read is a reason this check cannot answer,
///  and never a tree it passed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFile(): string { return readFileSync(join(sourceRoot(), THE_FILE), 'utf8'); }
/// ```
fn the_file() -> String {
    let path: std::path::PathBuf = source_root().join(THE_FILE);
    let read: std::io::Result<String> = std::fs::read_to_string(&path);
    return read.unwrap_or_else(|error| {
        panic!(
            "the ban reads {THE_FILE}, and {} cannot be read: {error}",
            path.display()
        )
    });
}

/// No control of the version cache writes its fixture Git in the run's own process.
/// A control that writes one gives its body a process whose other threads fork nothing.
#[test]
fn no_control_writes_a_fixture_git_in_the_run_s_own_process() {
    let text: String = the_file();
    let findings: Vec<String> = findings_in_text(&THE_VERSION_CACHE, THE_FILE, text.as_str());
    assert!(
        findings.is_empty(),
        "{} is opened for writing in whatever process runs the code that makes it, and `execve` refuses a file open \
         for writing anywhere, so a fork by another thread of a threaded run holds this control's start off with \
         `ETXTBSY` until that child execs, and a test run holds its controls as threads of one process: {}:\n{}",
        THE_VERSION_CACHE.what,
        THE_VERSION_CACHE.required,
        findings.join("\n")
    );
}

/// The file this ban reads still names every write it exists for,
/// so a ban whose subject moved away fails here instead of passing every tree.
#[test]
fn the_file_this_ban_reads_still_writes_a_fixture_git() {
    let text: String = the_file();
    let mut missing: Vec<String> = Vec::new();
    for (call, why) in FIXTURE_WRITES {
        if !text.contains(call) {
            missing.push(format!("{call}: {why}"));
        }
    }
    assert!(
        missing.is_empty(),
        "{THE_FILE} no longer names these calls, so this ban guards nothing in it: move the entry to the call that \
         opens the fixture Git for writing now, or delete it:\n{}",
        missing.join("\n")
    );
}

/// The scan finds a control that writes the fixture in the run's own process,
/// and says nothing about the control beside it that starts a child first,
/// nor about a call a comment or a planted text only names.
#[test]
fn the_scan_finds_a_control_that_writes_the_fixture_without_a_child_of_its_own() {
    // What: a source holding three controls: one that copies the fixture in the run's own process, one that
    //       starts a child first, and one that only writes the call's name into a string.
    // Why: the scan has to name the first and only the first, so a ban that excused it would fail here.
    let planted: &str = "\
#[test]\n\
fn writes_in_the_run_s_own_process() {\n\
    let git_copy: PathBuf = copy_git(root.as_path(), \"git\");\n\
}\n\
\n\
#[test]\n\
fn writes_in_a_child_of_its_own() {\n\
    /// The doc comment names std::fs::write( and it is no call.\n\
    run_the_body_in_a_child_of_its_own(module_path!(), \"writes_in_a_child_of_its_own\", || {\n\
        std::fs::write(&git_copy, bytes.as_slice()).expect(\"fixture\");\n\
    });\n\
}\n\
\n\
#[test]\n\
fn plants_the_name_in_a_text() {\n\
    let planted: &str = \"OpenOptions::new()\";\n\
}\n";
    assert_eq!(
        findings_in_text(&THE_VERSION_CACHE, "planted_tests.rs", planted),
        vec![String::from(
            "planted_tests.rs:1 (writes_in_the_run_s_own_process)"
        )],
        "the planted control that writes the fixture in the run's own process was not the one finding"
    );
}
