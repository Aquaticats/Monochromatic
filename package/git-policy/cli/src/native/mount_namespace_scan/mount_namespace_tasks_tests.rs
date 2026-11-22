//! What:
//!  Controls of the task grouping:
//!  two tasks of one namespace are one group,
//!  the group's tasks are in pid order,
//!  a task that ended before its link was read is left out,
//!  and a `/proc` root that cannot be listed is a refusal naming it.
//! Why:
//!  The scan's cost follows the namespaces,
//!  so a grouping that made one group per task would read one table per task,
//!  and a group a task entered the wrong way would decide which task's table
//!  the scan reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(tasksByMountNamespace(fixture)).toEqual([{ namespace: 'mnt:[7]', tasks: ['/proc/11', '/proc/12'] }]);
//! ```

use super::{ListingRefusal, MountNamespaceTasks, mount_namespace_of, tasks_by_mount_namespace};
use crate::testing::test_support::fixture;
use std::path::{Path, PathBuf};

/// One fixture task,
/// with the namespace its `ns/mnt` link names when one is given.
fn task(root: &Path, pid: &str, namespace: Option<&str>) {
    let folder: PathBuf = root.join(pid);
    std::fs::create_dir_all(folder.join("ns")).expect("the fixture task is made");
    if let Some(target) = namespace {
        std::os::unix::fs::symlink(target, folder.join("ns/mnt")).expect("the namespace link is made");
    }
}

/// Two tasks of one namespace are one group,
/// and its tasks are in the kernel's pid order,
/// whatever order the listing answered in.
#[test]
fn two_tasks_of_one_namespace_are_one_group_in_pid_order() {
    let root: PathBuf = fixture("mount-namespace-tasks-one-group");
    task(root.as_path(), "12", Some("mnt:[7]"));
    task(root.as_path(), "11", Some("mnt:[7]"));
    task(root.as_path(), "21", Some("mnt:[9]"));
    let groups: Vec<MountNamespaceTasks> = tasks_by_mount_namespace(root.as_path()).expect("the fixture lists");
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].namespace, String::from("mnt:[7]"));
    assert_eq!(groups[0].tasks, vec![root.join("11"), root.join("12")]);
    assert_eq!(groups[1].namespace, String::from("mnt:[9]"));
}

/// A task that ended before its link was read is left out,
/// and a directory that is no task is left out with it.
#[test]
fn a_task_without_a_namespace_link_is_left_out() {
    let root: PathBuf = fixture("mount-namespace-tasks-no-link");
    task(root.as_path(), "11", Some("mnt:[7]"));
    task(root.as_path(), "31", None);
    std::fs::create_dir_all(root.join("self")).expect("the non-task directory is made");
    std::fs::create_dir_all(root.join("acpi")).expect("the second non-task directory is made");
    let groups: Vec<MountNamespaceTasks> = tasks_by_mount_namespace(root.as_path()).expect("the fixture lists");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].tasks, vec![root.join("11")]);
    assert_eq!(mount_namespace_of(root.join("31").as_path()), None);
    assert_eq!(mount_namespace_of(root.join("11").as_path()), Some(String::from("mnt:[7]")));
}

/// A `/proc` root that cannot be listed is a refusal,
/// and its sentence names the directory and the way forward.
#[test]
fn an_unlistable_proc_root_is_a_refusal() {
    let root: PathBuf = fixture("mount-namespace-tasks-missing");
    let missing: PathBuf = root.join("no-such-proc");
    let refusal: ListingRefusal = tasks_by_mount_namespace(missing.as_path()).expect_err("a missing root cannot be listed");
    assert_eq!(refusal.path, missing);
    let said: String = refusal.message();
    assert!(said.contains(missing.display().to_string().as_str()), "{said}");
    assert!(said.contains("has not shown there is none"), "{said}");
}
