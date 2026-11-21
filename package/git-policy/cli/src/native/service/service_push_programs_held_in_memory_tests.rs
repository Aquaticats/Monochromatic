//! Controls for the programs a service push carries:
//! which variables travel,
//! whether the transport was chosen by the command that asked,
//! and that a value belongs to one push record and is given up with it.

use super::*;

/// An environment of the variables a shell sets.
fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    return pairs
        .iter()
        .map(|(name, value)| { return (OsString::from(name), OsString::from(value)); })
        .collect();
}

/// What:
///  The list a push carries is the shell's `PATH` and the variables that name its programs,
///  and no variable another program of the host could read something else from.
/// Why:
///   The service's push must find the programs the shell that asked for it would,
/// and a variable this list leaves out keeps the service's own value.
#[test]
fn the_carried_list_is_the_shells_own_programs() {
    let shell: Vec<(OsString, OsString)> = environment(&[
        ("PATH", "/home/a/.local/share/mise/shims:/usr/bin"),
        ("GIT_SSH_COMMAND", "ssh -i /home/a/.ssh/id_work"),
        ("GIT_EXEC_PATH", "/usr/lib/git-core"),
        ("TERM", "xterm-256color"),
        ("AWS_SECRET_ACCESS_KEY", "not-carried"),
    ]);
    let programs: PushPrograms = of(shell.as_slice());
    assert_eq!(
        programs.variables,
        vec![
            (String::from("PATH"), String::from("/home/a/.local/share/mise/shims:/usr/bin")),
            (String::from("GIT_EXEC_PATH"), String::from("/usr/lib/git-core")),
            (String::from("GIT_SSH_COMMAND"), String::from("ssh -i /home/a/.ssh/id_work")),
        ],
        "the search list and the programs a variable names travel, nothing else"
    );
    assert!(programs.transport, "the command chose its transport with GIT_SSH_COMMAND");
}

/// What:
///  A name that is set twice travels once,
///  with the value the last one gave.
/// Why:
///   Git reads the last value of a name,
/// so a push that carried the first would run a program the person's own Git would not.
#[test]
fn the_last_value_of_a_name_is_the_one_that_travels() {
    let shell: Vec<(OsString, OsString)> = environment(&[("PATH", "/first/bin:/usr/bin"), ("PATH", "/second/bin:/usr/bin")]);
    assert_eq!(
        of(shell.as_slice()).variables,
        vec![(String::from("PATH"), String::from("/second/bin:/usr/bin"))],
        "the value Git would read is the value that travels"
    );
}

/// What:
///  A value that is not text does not travel,
///  and a transport whose value is not text still marks the push.
/// Why:
///   A protocol line cannot hold bytes that are not text without inventing them,
/// and a transport this release cannot spell is still one the person chose:
/// a push that fell back to another would speak a way they did not ask for.
#[test]
fn a_value_that_is_not_text_does_not_travel_but_its_transport_does() {
    use std::os::unix::ffi::OsStringExt;
    let shell: Vec<(OsString, OsString)> = vec![
        (OsString::from("PATH"), OsString::from("/a/bin:/usr/bin")),
        (OsString::from("GIT_SSH_COMMAND"), OsString::from_vec(b"ssh -i /home/a/not-text-\xff".to_vec())),
    ];
    let programs: PushPrograms = of(shell.as_slice());
    assert_eq!(programs.variables, vec![(String::from("PATH"), String::from("/a/bin:/usr/bin"))], "only the readable value travels");
    assert!(programs.transport, "the transport the person set is still marked as one they chose");
}

/// What:
///  A command that set no transport variable is not marked as having chosen one,
///  and each of the three variables that name a transport does mark it.
/// Why:
///   The mark is what decides whether a push whose values are gone may search the service's own programs,
/// and a mark that was always set would stop every push a restart has to try again.
#[test]
fn only_the_variables_that_name_a_transport_mark_one() {
    let plain: Vec<(OsString, OsString)> = environment(&[("PATH", "/usr/bin"), ("GIT_EXEC_PATH", "/usr/lib/git-core")]);
    assert!(!of(plain.as_slice()).transport, "a list and an exec path name no transport");
    for name in TRANSPORT {
        let set: Vec<(OsString, OsString)> = environment(&[("PATH", "/usr/bin"), (name, "/usr/bin/ssh")]);
        assert!(of(set.as_slice()).transport, "{name} names the transport a push speaks through");
    }
}

/// What:
///  A value belongs to the push record it was held for,
///  and no other record of the same worktree takes it.
/// Why:
///   Two commands of one worktree may queue pushes a moment apart,
/// and a push must run with the values of the command that asked for it,
/// never with those of a later one (`AGENTS.md` rule `NLK`: one owner per item).
#[test]
fn a_value_is_taken_by_its_own_record_only() {
    let git_dir: PathBuf = crate::test_support::fixture_with_prefix("native-push-programs", "one-record");
    let first: PushPrograms = of(environment(&[("PATH", "/first/bin")]).as_slice());
    let second: PushPrograms = of(environment(&[("PATH", "/second/bin")]).as_slice());
    remember(git_dir.as_path(), "push-aaa-b", first.clone());
    remember(git_dir.as_path(), "push-bbb-b", second.clone());
    assert_eq!(held(git_dir.as_path(), "push-aaa-b"), Some(first), "the first record keeps the first command's list");
    assert_eq!(held(git_dir.as_path(), "push-bbb-b"), Some(second), "the second record keeps the second command's list");
    assert_eq!(held(git_dir.as_path(), "push-ccc-b"), None, "a record nobody held values for has none");
}

/// What:
///  The same record held again takes the newer values,
///  and the values of a record that can no longer run are given up.
/// Why:
///   A record could be held twice by one service (a landing replayed before its branch moved),
/// and a finished push must not keep a credential in the service's memory.
#[test]
fn a_record_held_again_takes_the_newer_values_and_forgetting_removes_them() {
    let git_dir: PathBuf = crate::test_support::fixture_with_prefix("native-push-programs", "held-again");
    let older: PushPrograms = of(environment(&[("PATH", "/older/bin")]).as_slice());
    let newer: PushPrograms = of(environment(&[("PATH", "/newer/bin")]).as_slice());
    remember(git_dir.as_path(), "push-aaa-b", older);
    remember(git_dir.as_path(), "push-aaa-b", newer.clone());
    assert_eq!(held(git_dir.as_path(), "push-aaa-b"), Some(newer), "the landing that holds the record last is the one that asked for its push");
    forget(git_dir.as_path(), "push-aaa-b");
    assert_eq!(held(git_dir.as_path(), "push-aaa-b"), None, "a settled record keeps nothing");
}

/// What:
///  Giving up a worktree's values gives up every record of it and no other worktree's.
/// Why:
///   A repository whose directory is gone has no push that can run,
/// and a worktree of the same repository keeps its own.
#[test]
fn giving_up_a_worktree_gives_up_its_own_values_only() {
    let gone: PathBuf = crate::test_support::fixture_with_prefix("native-push-programs", "gone");
    let kept: PathBuf = crate::test_support::fixture_with_prefix("native-push-programs", "kept");
    let programs: PushPrograms = of(environment(&[("PATH", "/usr/bin")]).as_slice());
    remember(gone.as_path(), "push-aaa-b", programs.clone());
    remember(kept.as_path(), "push-aaa-b", programs.clone());
    forget_every(gone.as_path());
    assert_eq!(held(gone.as_path(), "push-aaa-b"), None, "the worktree that is gone keeps nothing");
    assert_eq!(held(kept.as_path(), "push-aaa-b"), Some(programs), "the worktree beside it keeps its own list");
    forget_every(kept.as_path());
}

/// What:
///  A pair list is what a child's environment takes,
///  and a name the list holds is the only one that came from the command.
/// Why:
///   The push's overlay is built from this list,
/// and a name the list does not hold must keep the service's own value.
#[test]
fn the_pairs_are_the_list_and_the_list_says_which_names_it_holds() {
    let programs: PushPrograms = of(environment(&[("PATH", "/usr/bin"), ("GIT_SSH", "/usr/bin/ssh")]).as_slice());
    assert_eq!(
        programs.as_pairs(),
        vec![(OsString::from("PATH"), OsString::from("/usr/bin")), (OsString::from("GIT_SSH"), OsString::from("/usr/bin/ssh"))],
        "the list is what the push's children receive"
    );
    assert!(programs.holds("GIT_SSH"), "the command named its ssh");
    assert!(!programs.holds("GIT_PROXY_COMMAND"), "and named no proxy command");
}
