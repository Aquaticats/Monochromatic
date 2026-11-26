//! What:
//!  The control that keeps each control of the heartbeat tests answering for itself:
//!  the fake service manager of that file reads a word from the folder the control's own watch names,
//!  so the file declares no `static`.
//! Why:
//!  A crate's tests run as threads of one process,
//!  and the file's fake manager once read two `static` flags of the whole file.
//!  The two controls that set them slept while the other's value was wanted,
//!  so a threaded run of the file's filter failed one of them:
//!  the manager said the start failed while a control whose whole claim is that the manager says nothing failed was
//!  still waiting (issue 6484,
//!  `doc/troubleshooting/git-policy-cli-two-heartbeat-controls-share-one-manager-switch.md`).
//!  A fake manager asks no session bus,
//!  so the path a watch names is free for the word its own control writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(staticsIn('binary/binary_service_heartbeat_tests.rs')).toEqual([]);
//! ```

use std::path::Path;

use crate::source_scan_tests::{
    CrateRoot,
    SourceFile,
    classified_sources,
    real_roots,
    source_root,
    write_tree,
};

/// The file whose fake managers may read only the value their own control hands them.
const THE_HEARTBEAT_CONTROLS: &str = "binary/binary_service_heartbeat_tests.rs";

/// What:
///  `fn statics_in(text: &str) -> Vec<usize>` are the numbers of the lines of `text` that declare a `static` item.
/// Why:
///  A `static` is one value for the whole process,
///  so every thread of a run reads it and any of them writes it,
///  and a fake manager that answers from one answers a control's question with another control's word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staticsIn(text: string): number[]
/// ```
fn statics_in(text: &str) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        // What: `trim_start` drops the indentation, so a `static` inside a function body counts as one at the top
        //  level does; `line` is one line of the file, and `enumerate` pairs it with its place, counting from zero.
        // Why: an inner `static` is still one value of the whole process.
        let trimmed: &str = line.trim_start();
        // A comment names an item without declaring one, and this file's own prose names the statics it lost.
        if trimmed.starts_with("//") {
            continue;
        }
        let declares: bool = trimmed.starts_with("static ")
            || trimmed.starts_with("pub static ")
            || trimmed.starts_with("pub(crate) static ")
            || trimmed.starts_with("pub(super) static ");
        if declares {
            // What: `index + 1` is the line's number as an editor counts it, from one.
            // Why: a finding that named the count from zero would send its reader one line off.
            found.push(index + 1);
        }
    }
    return found;
}

/// What:
///  `fn findings_in(root, roots) -> Result<Vec<String>, String>` are the `file:line` findings of the sources under
///  `root`,
/// or one text saying why the scan cannot answer.
/// Why:
///  A scan whose listing held no such file would answer no findings,
/// which reads as a pass,
///  so a file the listing does not reach is an error here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsIn(root: string, roots: CrateRoot[]): string[]  // throws Error
/// ```
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let sources: Vec<SourceFile> = classified_sources(root, roots)?;
    let mut found: Vec<String> = Vec::new();
    let mut read: bool = false;
    for file in sources {
        if file.path != THE_HEARTBEAT_CONTROLS {
            continue;
        }
        read = true;
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for number in statics_in(text.as_str()) {
            found.push(format!("{}:{number}", file.path));
        }
    }
    if !read {
        return Err(format!(
            "{THE_HEARTBEAT_CONTROLS} is not a source this listing reaches"
        ));
    }
    return Ok(found);
}

/// The findings in this crate's own sources.
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's sources");
}

/// No fake manager of the heartbeat controls answers from a value the whole process shares.
#[test]
fn no_heartbeat_control_reads_a_value_the_whole_process_shares() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "a `static` is one value for every thread of a run, so a fake manager that answers from one answers another \
         control's question with this control's word; a control writes the word it wants into the folder its watch \
         names as its session bus, and a value that never changes is a `const` (issue 6484):\n{}",
        found.join("\n")
    );
}

/// The scan fires on the value it exists for,
/// so a check that stopped reading would fail here rather than passing every tree.
#[test]
fn the_scan_finds_a_planted_static() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-heartbeat-static");
    write_tree(
        root.as_path(),
        &[
            ("lib.rs", "mod binary;\n"),
            ("binary.rs", "mod binary_service_heartbeat_tests;\n"),
            (
                "binary/binary_service_heartbeat_tests.rs",
                "/// A comment that names a static count is not one.\n\
                 fn a_control() {\n    assert_eq!(COUNT.load(), 0);\n}\n\
                 static COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);\n",
            ),
        ],
    );
    let found: Vec<String> = findings_in(
        root.as_path(),
        &[crate::source_scan_tests::release_root("lib.rs")],
    )
    .expect("the fixture sources");
    assert_eq!(
        found,
        vec![String::from("binary/binary_service_heartbeat_tests.rs:5")],
        "the planted static was not found"
    );
    crate::test_support::remove(root.as_path());
}
