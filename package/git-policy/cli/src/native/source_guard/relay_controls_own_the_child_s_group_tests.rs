//! What:
//!  The control that refuses a test file which starts or reads a relayed child
//!  and never hands that child's group to the test thread's owner.
//! Why:
//!   The relay ends with the child and leaves whatever the child started holding the child's error stream
//!  (`src/native/invocation/typed_network_command_relay/git_error_stream_relayed_as_it_arrives.rs`),
//!  and the control of that end rule returned with the process the child left still running:
//!  the container gate counted the test as one that leaks (issue 6501).
//!  The child is started with `process_group(0)` so what it leaves stays in one group the control can name,
//!  and that group goes to `test_process_owner::own_group_of_number`,
//!  which ends it whatever way the test ends.
//! Gotcha:
//!  The scan is textual and reads test-only files.
//!  A control that reaches the read through a helper sees the finding at that helper's own read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(relayReadsWithoutAnOwner(sourceRoot, roots)).toEqual([]);
//! ```

use std::path::Path;

use super::source_code_places_tests::is_code;
use super::source_scan_tests::{
    CrateRoot,
    classified_sources,
    real_roots,
    source_root,
    write_tree,
};

/// The call that starts a child the shipped relay reads and ends itself.
const STARTS_A_RELAYED_CHILD: &str = "start_relaying(";

/// The call that reads a relayed child to its end,
/// whose answer carries that child's number.
const READS_TO_THE_CHILD_S_END: &str = "read_to_the_child_s_end(";

/// The call that makes a group a child of a shipped call leads die with the test thread.
const OWNS_THE_GROUP_OF_A_NUMBER: &str = "own_group_of_number(";

/// The repair every finding names,
/// said once so each message sends the reader the same way.
const REPAIR: &str = "hand the child's number to `test_process_owner::own_group_of_number`, start the child with \
                      `process_group(0)`, and end the group before the test returns";

/// What:
///  `fn code_lines_of(text, needle) -> Vec<usize>` lists the one-based line number of every place `needle` appears as
///  code rather than in a comment or a text.
/// Why:
///  A comment or a fixture text that names a call starts and reads nothing,
///  and a finding that counted one would send the reader to a line with no child on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function codeLinesOf(text: string, needle: string): number[]
/// ```
fn code_lines_of(
    text: &str,
    needle: &str,
) -> Vec<usize> {
    let code: Vec<bool> = is_code(text);
    let mut lines: Vec<usize> = Vec::new();
    for (index, _found) in text.match_indices(needle) {
        if code.get(index) != Some(&true) {
            continue;
        }
        lines.push(text[..index].matches('\n').count() + 1);
    }
    return lines;
}

/// What:
///  `fn relay_reads_without_an_owner(text) -> Vec<usize>` lists the lines of one test file that start or read a
///  relayed child while nothing in that file hands a group number to the test thread's owner.
/// Why:
///  A control of the relay's end rule is the one caller that keeps no `Child` of its own,
///  so the number the read answers is the only handle it has on what the child left,
///  and a file without that call leaves the process holding the stream past its test (issue 6501).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relayReadsWithoutAnOwner(text: string): number[]
/// ```
fn relay_reads_without_an_owner(text: &str) -> Vec<usize> {
    if !code_lines_of(text, OWNS_THE_GROUP_OF_A_NUMBER).is_empty() {
        return Vec::new();
    }
    let mut found: Vec<usize> = code_lines_of(text, STARTS_A_RELAYED_CHILD);
    found.extend(code_lines_of(text, READS_TO_THE_CHILD_S_END));
    // `sort` puts the places in file order, and `dedup` keeps a line that holds both calls once.
    found.sort_unstable();
    found.dedup();
    return found;
}

/// What:
///  `fn findings_in(root, roots) -> Result<Vec<String>, String>` is every start or read of a relayed child in a test file
///  of the tree under `root` that names no owner,
///  as `identity:line`.
/// `Result<Vec<String>, String>` is "the findings or one text saying what is wrong" (sibling: a bare list,
/// which cannot say why a tree could not be read).
/// Why:
///  A finding names a place a person can open,
///  and the control over a planted tree and the control over this crate ask the same question,
///  so only the tree differs between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsIn(root: string, roots: CrateRoot[]): string[]  // throws Error
/// ```
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in classified_sources(root, roots)? {
        if !file.test_only {
            continue;
        }
        let path: std::path::PathBuf = root.join(file.path.as_str());
        let text: String = std::fs::read_to_string(path.as_path())
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for line in relay_reads_without_an_owner(text.as_str()) {
            found.push(format!("{}:{line}", file.path));
        }
    }
    return Ok(found);
}

/// Every test file of this crate that starts or reads a relayed child and names no owner of its group.
fn findings() -> Vec<String> {
    return findings_in(source_root().as_path(), real_roots().as_slice())
        .expect("the crate's own sources");
}

/// No test file starts or reads a relayed child without owning the group that child leads.
#[test]
fn no_test_file_reads_a_relayed_child_it_does_not_own() {
    let found: Vec<String> = findings();
    assert!(found.is_empty(), "{REPAIR}:\n{}", found.join("\n"));
}

/// What:
///  `fn test_root(path) -> CrateRoot` is a root the build compiles for tests only,
///  which is how a planted test file is reached.
/// Why:
///  The scan reads test-only files alone,
///  and a root is what the module tree starts from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const testRoot = (path: string): CrateRoot => ({ path, testOnly: true })
/// ```
fn test_root(path: &str) -> CrateRoot {
    return CrateRoot {
        path: String::from(path),
        test_only: true,
    };
}

/// The scan finds a planted read with no owner,
/// and passes over a file that owns its group and a call only a comment names.
#[test]
fn the_scan_finds_a_planted_read_with_no_owner() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-relay-controls");
    let owned: &str = "fn reads_a_child_it_owns() {\n    let relayed = read_to_the_child_s_end(start, &mut bytes).expect(\"the end\");\n    own_group_of_number(relayed.child_number);\n}\n";
    let bare: &str = "fn reads_a_child() {\n    let relayed = read_to_the_child_s_end(start, &mut bytes).expect(\"the end\");\n    let _kept = relayed.kept;\n}\n";
    let named: &str = "// This comment names read_to_the_child_s_end(start, &mut bytes) and starts nothing.\nconst A_NEEDLE: &str = \"read_to_the_child_s_end(start, bytes)\";\n";
    write_tree(
        root.as_path(),
        &[
            ("owned_tests.rs", owned),
            ("bare_tests.rs", bare),
            ("named_tests.rs", named),
        ],
    );
    let roots: [CrateRoot; 3] = [
        test_root("owned_tests.rs"),
        test_root("bare_tests.rs"),
        test_root("named_tests.rs"),
    ];
    let found: Vec<String> =
        findings_in(root.as_path(), roots.as_slice()).expect("the fixture sources");
    crate::test_support::remove(root.as_path());
    assert_eq!(
        found,
        vec![String::from("bare_tests.rs:2")],
        "only the read with no owner beside it and no ownership in its file is a site"
    );
}

/// A planted file that starts a relayed child and reads nothing is a site too,
/// so a control that starts the child and leaves it to the relay fails here.
#[test]
fn the_scan_finds_a_planted_start_with_no_owner() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-relay-starts");
    let planted: &str = "fn starts_a_child() {\n    let start = start_relaying(ErrorStreamShape::Pipe, None, &mut command).expect(\"the child\");\n}\n";
    write_tree(root.as_path(), &[("planted_tests.rs", planted)]);
    let roots: [CrateRoot; 1] = [test_root("planted_tests.rs")];
    let found: Vec<String> =
        findings_in(root.as_path(), roots.as_slice()).expect("the fixture sources");
    crate::test_support::remove(root.as_path());
    assert_eq!(found, vec![String::from("planted_tests.rs:2")]);
}
