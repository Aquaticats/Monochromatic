//! What:
//!  Controls of the namespace scan and its two comparisons:
//!  two tasks in one namespace are one table,
//!  only a `zfs` mount of the exact source counts,
//!  a namespace the pass did not expect is named,
//!  a namespace it expected and the scan did not find is named,
//!  and a table it cannot read is a refusal rather than a clear result.
//! Why:
//!  The arm's first pass requires exactly the worker's and the holder's
//!  namespaces,
//!  and a scan that read no table has proved nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(scanNamespaces('/proc', source)).toEqual([workerNs, holderNs]);
//! ```

use super::{NamespaceTable, ScanRefusal, compare_namespaces, namespaces_outside, scan_namespaces};
use crate::fs_mount_options::parse_mount_table;
use crate::testing::test_support::fixture;
use std::path::{Path, PathBuf};

/// The mount line a probe's namespace would hold for the source.
const MENTIONING: &str = "41 30 0:32 / /run/probe/scratch/slot ro,nosuid,nodev - zfs zpcachyos/nosnap/git-worktree-probe@ready ro\n";
/// A mount line of the same dataset that is no source the probe asked for.
const OTHER_SOURCE: &str = "41 30 0:32 / /home/user/Monochromatic rw - zfs zpcachyos/ROOT/cos/home rw\n";

/// One fixture `/proc` root:
/// two tasks in one namespace that mention the source,
/// one in a second that does not,
/// and one file that is no task.
fn proc_root(root: &Path, mentioning: &str, other: &str) -> PathBuf {
    let proc_root_dir: PathBuf = root.join("proc");
    for (pid, namespace, mountinfo) in [("11", "mnt:[4026531840]", mentioning), ("12", "mnt:[4026531840]", mentioning), ("21", "mnt:[4026532001]", other)] {
        let task: PathBuf = proc_root_dir.join(pid);
        std::fs::create_dir_all(task.join("ns")).expect("the fixture task is made");
        std::os::unix::fs::symlink(namespace, task.join("ns/mnt")).expect("the namespace link is made");
        std::fs::write(task.join("mountinfo"), mountinfo).expect("the mount table is written");
    }
    std::fs::create_dir_all(proc_root_dir.join("self")).expect("the non-task directory is made");
    return proc_root_dir;
}

/// One namespace is one table however many tasks it holds,
/// and only a `zfs` mount of the exact source counts.
#[test]
fn one_namespace_is_one_table_and_only_the_source_counts() {
    let root: PathBuf = fixture("start-probe-scan");
    let proc: PathBuf = proc_root(root.as_path(), MENTIONING, OTHER_SOURCE);
    let mentioned: Vec<String> = scan_namespaces(proc.as_path(), "zpcachyos/nosnap/git-worktree-probe@ready").expect("the scan reads");
    assert_eq!(mentioned, vec![String::from("mnt:[4026531840]")]);
    let tables: Vec<NamespaceTable> = namespaces_outside(proc.as_path()).expect("the scan reads");
    assert_eq!(tables.len(), 2);
    let other = parse_mount_table(OTHER_SOURCE.as_bytes());
    assert!(!NamespaceTable { namespace: String::from("x"), mounts: other }.mentions("zpcachyos/nosnap/git-worktree-probe@ready"));
}

/// A namespace the pass did not expect is named,
/// and one it expected and did not find is named.
#[test]
fn the_comparison_names_both_directions() {
    let mentioned: Vec<String> = vec![String::from("mnt:[1]"), String::from("mnt:[2]")];
    assert_eq!(compare_namespaces(mentioned.as_slice(), &["mnt:[1]"]), Some(super::ScanDifference::Unexpected { namespace: String::from("mnt:[2]") }));
    assert_eq!(
        compare_namespaces(&[String::from("mnt:[1]")], &["mnt:[1]", "mnt:[3]"]),
        Some(super::ScanDifference::Missing { namespace: String::from("mnt:[3]") })
    );
    assert_eq!(compare_namespaces(mentioned.as_slice(), &["mnt:[1]", "mnt:[2]"]), None);
    assert_eq!(compare_namespaces(&[], &[]), None);
}

/// A `/proc` root that cannot be listed is a refusal.
#[test]
fn an_unlistable_proc_root_is_a_refusal() {
    let root: PathBuf = fixture("start-probe-scan-missing");
    let missing: PathBuf = root.join("no-such-proc");
    assert!(matches!(scan_namespaces(missing.as_path(), "source"), Err(ScanRefusal::Proc { .. })));
}

/// A task that ends between the listing and the table read is skipped,
/// and a namespace whose every task ended is skipped whole,
/// because a namespace no live task holds is invisible to the scan.
#[test]
fn a_task_gone_between_listing_and_read_is_skipped() {
    let root: PathBuf = fixture("start-probe-scan-gone");
    let proc: PathBuf = proc_root(root.as_path(), MENTIONING, OTHER_SOURCE);
    std::fs::remove_file(proc.join("11/mountinfo")).expect("the vanished task's table is removed");
    let tables: Vec<NamespaceTable> = namespaces_outside(proc.as_path()).expect("the scan reads the live task's table");
    assert_eq!(tables.len(), 2);
    for (pid, namespace) in [("31", "mnt:[4026533001]"), ("32", "mnt:[4026533001]")] {
        let task: PathBuf = proc.join(pid);
        std::fs::create_dir_all(task.join("ns")).expect("the fixture task is made");
        std::os::unix::fs::symlink(namespace, task.join("ns/mnt")).expect("the namespace link is made");
    }
    std::fs::remove_file(proc.join("21/mountinfo")).expect("the second namespace's first table is removed");
    let mentioned: Vec<String> = scan_namespaces(proc.as_path(), "zpcachyos/nosnap/git-worktree-probe@ready").expect("the scan reads");
    assert_eq!(mentioned, vec![String::from("mnt:[4026531840]")]);
}

/// A read error that is not a task's ending still refuses the scan,
/// so a table this process may not read is never reported as clear.
#[test]
fn a_persistent_read_error_on_a_live_task_still_refuses() {
    let root: PathBuf = fixture("start-probe-scan-error");
    let proc: PathBuf = proc_root(root.as_path(), MENTIONING, OTHER_SOURCE);
    std::fs::remove_file(proc.join("21/mountinfo")).expect("the table is removed");
    std::fs::create_dir(proc.join("21/mountinfo")).expect("a directory stands where the table was");
    match namespaces_outside(proc.as_path()) {
        Err(ScanRefusal::Table { path, .. }) => assert_eq!(path, proc.join("21/mountinfo")),
        other => panic!("{other:?}"),
    }
}
