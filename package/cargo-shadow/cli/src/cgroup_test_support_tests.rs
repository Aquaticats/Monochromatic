//! What:
//! Tests of the support the control-group tests share:
//! the text `/proc/<pid>/cgroup` gives for a process,
//! the version 2 line of that file and the path it names,
//! whatever lines the host writes beside it.
//! Why:
//! Issue 6549:
//! nine controls read the whole file as one cgroup v2 line and failed on this host,
//! which mounts a version 1 `net_cls` controller beside the version 2 hierarchy and writes its line first.
//! The hybrid text below is this host's file;
//! the unified text is what a host without a version 1 mount writes,
//! and both are checked here,
//! so the reading is proven on this host and for the host whose fact differs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('the cgroup file reader', () => { it('reads the `0::` line', ...) })
//! ```

/// The reading of a process's control group file the tests share.
use crate::cgroup_test_support::{
    group_of,
    v2_line_of_cgroup_file,
    v2_path_of_cgroup_file,
};

/// The hybrid text:
/// a version 1 `net_cls` line,
/// then the version 2 line.
/// Captured from `/proc/self/cgroup` on this host on 2026-11-27,
/// and the same two lines are quoted in issue 6549.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HYBRID_CAPTURE = '1:net_cls:/\n0::/user.slice/.../claude-code-bash\n';
/// ```
const HYBRID_CAPTURE: &str =
    "1:net_cls:/\n0::/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash\n";

/// The unified text:
/// the one line a host with no version 1 controller writes.
/// A stand-in.
/// No unified host was captured for this test.
/// Its shape is the file's own:
/// `0::` and the path below the cgroup root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNIFIED_TEXT = '0::/user.slice/run.scope\n';
/// ```
const UNIFIED_TEXT: &str = "0::/user.slice/run.scope\n";

/// What:
/// `fn the_version_2_line_of_a_hybrid_file_is_read_and_the_version_1_lines_ignored()` reads this host's file shape,
/// which gave `1:net_cls:/` before the version 2 line.
/// Why:
/// Issue 6549:
/// every control that took the whole file as one line failed on this host,
/// because `1:net_cls:/` sits before `0::<path>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(v2PathOfCgroupFile(HYBRID_CAPTURE)).toBe('/user.slice/.../claude-code-bash');
/// ```
#[test]
fn the_version_2_line_of_a_hybrid_file_is_read_and_the_version_1_lines_ignored() {
    assert_eq!(
        v2_line_of_cgroup_file(HYBRID_CAPTURE),
        "0::/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash",
        "the version 2 line of the file is not the one the reader returned"
    );
    assert_eq!(
        v2_path_of_cgroup_file(HYBRID_CAPTURE),
        "/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash",
        "the path after `0::` is not the one the reader returned"
    );
}

/// What:
/// `fn the_version_2_line_of_a_unified_file_is_read_the_same_way()` reads the text a host with no version 1 mount writes.
/// Why:
/// The nine controls run on both kinds of host,
/// so the reader must give the same answer from the file that holds one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(v2PathOfCgroupFile(UNIFIED_TEXT)).toBe('/user.slice/run.scope');
/// ```
#[test]
fn the_version_2_line_of_a_unified_file_is_read_the_same_way() {
    assert_eq!(
        v2_line_of_cgroup_file(UNIFIED_TEXT),
        "0::/user.slice/run.scope",
        "the version 2 line of a unified file is not the one the reader returned"
    );
    assert_eq!(
        v2_path_of_cgroup_file(UNIFIED_TEXT),
        "/user.slice/run.scope",
        "the path after `0::` is not the one the reader returned"
    );
}

/// What:
/// `fn a_file_with_no_version_2_line_stops_the_reader_and_quotes_the_file()` reads a version 1 only file.
/// Why:
/// A reader that invented a group from such a file would have a control assert about a group it never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => v2LineOfCgroupFile('1:net_cls:/\n')).toThrow(/no `0::` line/);
/// ```
#[test]
#[should_panic(expected = "so this file names no control group of the version 2 hierarchy")]
fn a_file_with_no_version_2_line_stops_the_reader_and_quotes_the_file() {
    let _line: String = v2_line_of_cgroup_file("1:net_cls:/\n");
}

/// What:
/// `fn the_group_of_this_process_is_the_single_version_2_line_of_its_file()` reads this process's own file,
/// which on this host holds the version 1 line too.
/// Why:
/// The nine controls begin from this read,
/// so it must give one line,
/// and a read that gave the whole file is what failed them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(groupOf('self').startsWith('0::')).toBe(true);
/// ```
#[test]
fn the_group_of_this_process_is_the_single_version_2_line_of_its_file() {
    let line: String = group_of("self");
    assert!(
        line.starts_with("0::"),
        "the group of this process is not a version 2 line: {line:?}"
    );
    assert_eq!(
        line.lines().count(),
        1,
        "the group of this process is more than one line: {line:?}"
    );
}
