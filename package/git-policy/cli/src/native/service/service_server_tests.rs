//! Controls for how the service opens its listening socket.

use super::*;
use std::os::unix::net::UnixStream;

/// A directory of this test's own, short enough for a socket path: a socket path holds about a hundred bytes, which a fixture
///  root the person points the tests at would not leave.
fn short_directory(name: &str) -> std::path::PathBuf {
    let home: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory for the short directory"));
    let directory: std::path::PathBuf = home.join("temp/agent").join(format!("ss-{}-{name}", std::process::id()));
    let _stale = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("short directory");
    return directory;
}

/// Removes a directory made by `short_directory`.
fn remove_short(directory: &std::path::Path) {
    let _removed = std::fs::remove_dir_all(directory);
}

/// A socket file nobody listens on any more, as a service that was killed leaves it.
fn stale_socket(directory: &std::path::Path) -> std::path::PathBuf {
    let path: std::path::PathBuf = directory.join("service.sock");
    let listener: UnixListener = UnixListener::bind(&path).expect("first bind");
    drop(listener);
    assert!(path.exists(), "a dropped listener leaves its socket file behind");
    return path;
}

/// A service run bound to a path takes over a socket file that no service listens on.
#[test]
fn a_path_whose_socket_file_is_stale_is_bound_again() {
    let root = short_directory("stale");
    let path = stale_socket(root.as_path());
    let listener: UnixListener = bind(&Bind::Path(path.clone())).expect("a stale socket file does not stop the bind");
    assert!(UnixStream::connect(&path).is_ok(), "the new listener answers");
    drop(listener);
    remove_short(root.as_path());
}

/// A socket some service still listens on is never taken over: the second bind fails and the first keeps answering.
#[test]
fn a_path_whose_socket_a_service_listens_on_is_not_taken_over() {
    let root = short_directory("live");
    let path = root.join("service.sock");
    let live: UnixListener = bind(&Bind::Path(path.clone())).expect("first bind");
    assert!(bind(&Bind::Path(path.clone())).is_err(), "a live socket stays with its service");
    assert!(UnixStream::connect(&path).is_ok(), "the first service still answers");
    drop(live);
    remove_short(root.as_path());
}

/// A path that is not a socket (a regular file the person owns) is never removed.
#[test]
fn a_path_that_holds_a_regular_file_is_left_alone() {
    let root = short_directory("file");
    let path = root.join("service.sock");
    std::fs::write(&path, b"mine").expect("write");
    assert!(bind(&Bind::Path(path.clone())).is_err(), "a regular file is not a socket to replace");
    assert_eq!(std::fs::read(&path).expect("read"), b"mine", "the file is untouched");
    remove_short(root.as_path());
}

/// The documented idle exit:
/// three hours with nothing to do,
/// the default of Bazel's `--max_idle_secs`
///  (`package/git-policy/cli/README.md`,
/// `doc/planning/cli-git-resident-service.md`).
const DOCUMENTED_IDLE_EXIT: Duration = Duration::from_secs(3 * 60 * 60);

/// A service that exits after a few minutes would restart for nearly every commit,
/// and one that never exits would hold a
/// repository's memory for good;
/// the default is the documented three hours.
#[test]
fn the_default_idle_exit_is_the_documented_three_hours() {
    assert_eq!(IDLE_EXIT, DOCUMENTED_IDLE_EXIT);
    assert_eq!(ServeSettings::new(IDLE_EXIT).idle, DOCUMENTED_IDLE_EXIT);
}

/// The process the activation variables are checked against in the controls.
const OWN_PID: u32 = 4242;

/// The variables of a manager that passed this process `count` descriptors are accepted for one or more and refused for none.
#[test]
fn the_manager_passed_at_least_one_descriptor_to_this_process() {
    let own: String = OWN_PID.to_string();
    for (count, passed) in [("0", false), ("1", true), ("2", true), ("-1", false)] {
        let outcome: std::io::Result<()> = ensure_socket_passed_in(Some(own.as_str()), Some(count), OWN_PID);
        assert_eq!(outcome.is_ok(), passed, "{count} descriptors: {outcome:?}");
    }
}

/// A count the manager set for another process,
/// or no variables,
/// or a count that is not a number,
/// passes nothing to this one.
#[test]
fn activation_variables_for_another_process_or_unreadable_pass_nothing() {
    let own: String = OWN_PID.to_string();
    let other: String = (OWN_PID + 1).to_string();
    assert!(ensure_socket_passed_in(Some(other.as_str()), Some("1"), OWN_PID).is_err(), "another process's descriptors");
    assert!(ensure_socket_passed_in(None, Some("1"), OWN_PID).is_err(), "no process named");
    assert!(ensure_socket_passed_in(Some(own.as_str()), None, OWN_PID).is_err(), "no count");
    assert!(ensure_socket_passed_in(Some(own.as_str()), Some("one"), OWN_PID).is_err(), "a count that is not a number");
    let refused: std::io::Error = ensure_socket_passed_in(None, None, OWN_PID).expect_err("nothing was passed in");
    assert!(refused.to_string().contains("no socket was passed in"), "{refused}");
}

/// A group key for the branch `reference` of `common`,
/// landing alone or not.
fn key(common: &str, reference: &str, alone: bool) -> GroupKey {
    return (std::path::PathBuf::from(common), String::from(reference), alone);
}

/// The members of each group,
/// in group order,
/// for comparing with what a queue should have made.
fn members_of(groups: &[(GroupKey, Vec<u32>)]) -> Vec<Vec<u32>> {
    return groups.iter().map(|(_, members)| { return members.clone(); }).collect();
}

/// Requests of one branch that do not land alone share a batch in arrival order.
#[test]
fn requests_of_one_branch_share_a_group_in_arrival_order() {
    let mut groups: Vec<(GroupKey, Vec<u32>)> = Vec::new();
    for member in [1, 2, 3] {
        place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", false), member);
    }
    assert_eq!(members_of(groups.as_slice()), vec![vec![1, 2, 3]]);
}

/// A request never lands in a batch of another branch or of another repository,
/// even when the branch name is the same.
#[test]
fn a_request_never_joins_another_branchs_or_repositorys_group() {
    let mut groups: Vec<(GroupKey, Vec<u32>)> = Vec::new();
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", false), 1);
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/side", false), 2);
    place_in_groups(&mut groups, key("/other/.git", "refs/heads/main", false), 3);
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", false), 4);
    assert_eq!(members_of(groups.as_slice()), vec![vec![1, 4], vec![2], vec![3]]);
}

/// An amend is a group of its own:
/// it joins none,
/// and what arrives after it starts another group instead of joining it,
/// so
/// the order on one branch is the order of arrival.
#[test]
fn a_request_that_lands_alone_neither_joins_nor_is_joined() {
    let mut groups: Vec<(GroupKey, Vec<u32>)> = Vec::new();
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", false), 1);
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", true), 2);
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", false), 3);
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", true), 4);
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", true), 5);
    place_in_groups(&mut groups, key("/repo/.git", "refs/heads/main", false), 6);
    assert_eq!(members_of(groups.as_slice()), vec![vec![1], vec![2], vec![3], vec![4], vec![5], vec![6]]);
}

/// A batch an earlier service left open and the restart settled is said, once each, at the `warning` priority with the
/// batch number as a field: an open batch means a service stopped between its intent and its end.
#[test]
fn a_restart_says_each_batch_it_settled() {
    use crate::journald_native::Priority;
    use crate::service_journal::{Intent, write_intent};
    use crate::transaction_git::GitContext;
    let root = crate::test_support::fixture("server-settled");
    let journal = root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    let git_dir = root.join("worktree/.git");
    let intent: Intent = Intent {
        seq: 41,
        repository: root.join("worktree"),
        common_dir: git_dir.clone(),
        reference: String::from("refs/heads/main"),
        old: "0".repeat(40),
        new: "1".repeat(40),
        ids: vec![String::from("r")],
        commits: vec![String::from("c")],
        paths: Vec::new(),
        derived: false,
        cleanup: None,
        index: git_dir.join("index"),
        git_dir: git_dir.clone(),
    };
    write_intent(journal.as_path(), &intent).expect("an open batch");
    let context: LandContext = LandContext {
        git: GitContext { real_git: std::path::PathBuf::from("/usr/bin/git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: None },
        journal,
        on_objects_written: None,
        lock_ceiling: crate::wait_ceilings::LOCK_CEILING,
    };
    let entries = crate::service_timing::capture::entries_of(|| { let _reports = settle_open_batches(&context, None); });
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0].priority, Priority::Warning);
    assert!(entries[0].message.starts_with("settled the batch an earlier service left open batch=41 decision="), "{}", entries[0].message);
    assert!(entries[0].fields.contains(&(String::from("CLI_GIT_BATCH"), String::from("41"))), "{:?}", entries[0].fields);
    crate::test_support::remove(root.as_path());
}

/// The batch cap is the size the measured landing cost had not yet doubled at, so a change of either is made with the other.
#[test]
fn the_batch_cap_is_the_measured_cost_knee() {
    assert_eq!(MAX_BATCH, BATCH_COST_KNEE);
}
