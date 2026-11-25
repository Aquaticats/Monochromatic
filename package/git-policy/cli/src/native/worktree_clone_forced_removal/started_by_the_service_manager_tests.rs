//! Controls for the refusal of a command the service manager started:
//!  a parent that is the first process refuses,
//!  a user manager is recognised by its cgroup and its own name and a look-alike is not,
//!  the session leader's parent is looked for too,
//!  an unreadable leader refuses as an orphan,
//!  and the sentence names the manager and the remedy.

use crate::worktree_clone_forced_removal::started_by_the_service_manager::*;

/// Reads of a command nothing started but a shell,
/// whose session leader's parent is another shell.
fn a_shell_shape() -> ManagerReads<'static> {
    return ManagerReads {
        uid: 1000,
        parent_pid: 4000,
        parent_command: Some("zsh"),
        parent_cgroup: Some("/user.slice/user-1000.slice/session-3.scope"),
        session_leader_pid: 4200,
        leader_parent: Some(4100),
        leader_parent_command: Some("claude"),
        leader_parent_cgroup: Some("/user.slice/user-1000.slice/session-2.scope"),
    };
}

/// A user manager's cgroup path for one account.
fn a_user_manager_init(uid: u32) -> String {
    return format!("/user.slice/user-{uid}.slice/user@{uid}.service/init.scope");
}

/// A command whose parent is the first process refuses as the system manager's,
/// and the sentence names the system manager and the remedy.
#[test]
fn the_first_process_is_the_system_manager() {
    let shape: ManagerReads<'static> = a_shell_shape();
    let reads: ManagerReads<'static> = ManagerReads { parent_pid: THE_FIRST_PROCESS, ..shape };
    let refusal: ManagerRefusal = started_by_the_service_manager(&reads).expect("the first process refuses");
    assert_eq!(refusal.place, ManagerPlace::Parent { pid: THE_FIRST_PROCESS, manager: ManagerKind::System });
    let sentence: String = the_refusal_text("<slot>", &refusal);
    assert!(sentence.contains("pid 1, systemd"), "{sentence}");
    assert!(sentence.contains("setsid --wait"), "{sentence}");
    assert!(sentence.contains("nothing was changed and nothing was ended"), "{sentence}");
}

/// A user manager is its cgroup path and its own name together,
/// and neither alone is enough.
#[test]
fn a_user_manager_is_its_cgroup_and_its_name() {
    let uid: u32 = 1000;
    let init: String = a_user_manager_init(uid);
    assert_eq!(manager_of(4000, Some(MANAGER_COMMAND), Some(init.as_str()), uid), Some(ManagerKind::User));
    // The same cgroup without the manager's own name is another program running under it.
    assert_eq!(manager_of(4000, Some("bash"), Some(init.as_str()), uid), None);
    // The manager's name under another cgroup is not the account's manager.
    assert_eq!(manager_of(4000, Some(MANAGER_COMMAND), Some("/user.slice/user-1000.slice/session-3.scope"), uid), None);
    assert_eq!(manager_of(4000, Some(MANAGER_COMMAND), None, uid), None);
    // Another account's manager path is not this account's.
    assert_eq!(manager_of(4000, Some(MANAGER_COMMAND), Some(a_user_manager_init(1001).as_str()), uid), None);
}

/// A session leader whose parent is the manager refuses as that place,
/// which is how a unit whose main process is a script is caught.
#[test]
fn the_session_leaders_parent_is_looked_for_too() {
    let uid: u32 = 1000;
    let init: String = a_user_manager_init(uid);
    let shape: ManagerReads<'static> = a_shell_shape();
    let reads: ManagerReads<'_> = ManagerReads {
        leader_parent_command: Some(MANAGER_COMMAND),
        leader_parent_cgroup: Some(init.as_str()),
        ..shape
    };
    let refusal: ManagerRefusal = started_by_the_service_manager(&reads).expect("the leader's parent refuses");
    assert_eq!(refusal.place, ManagerPlace::SessionLeaderParent { pid: 4100, manager: ManagerKind::User });
    let sentence: String = the_refusal_text("one", &refusal);
    assert!(sentence.contains("session leader's parent is the manager (pid 4100, systemd --user)"), "{sentence}");
}

/// A session leader whose own record could not be read refuses as an orphan,
/// with the sentence saying nothing proves what started the session.
#[test]
fn an_unreadable_leader_is_an_orphan() {
    let shape: ManagerReads<'static> = a_shell_shape();
    let reads: ManagerReads<'static> = ManagerReads { leader_parent: None, ..shape };
    let refusal: ManagerRefusal = started_by_the_service_manager(&reads).expect("an unreadable leader refuses");
    assert_eq!(refusal.place, ManagerPlace::Orphan { leader: 4200 });
    let sentence: String = the_refusal_text("one", &refusal);
    assert!(sentence.contains("nothing proves what started the session"), "{sentence}");
}

/// A shape the measured agent runs have is allowed:
/// a shell's own child in an ordinary session scope.
#[test]
fn the_measured_agent_shapes_are_allowed() {
    assert_eq!(started_by_the_service_manager(&a_shell_shape()), None);
}

/// The unified hierarchy's own path is read from the one line whose hierarchy is zero,
/// and a text of another shape answers nothing.
#[test]
fn the_unified_cgroup_path_is_the_zero_line() {
    let text: &str = "0::/user.slice/user-1000.slice/user@1000.service/init.scope\n";
    assert_eq!(
        unified_cgroup_path(text),
        Some(String::from("/user.slice/user-1000.slice/user@1000.service/init.scope"))
    );
    assert_eq!(unified_cgroup_path("1:name=systemd:/x\n"), None);
    assert_eq!(unified_cgroup_path("0::\n"), None);
    assert_eq!(unified_cgroup_path(""), None);
}

/// A command name is escaped before it enters the sentence,
/// and a record that could not be read gets the sentence's own word instead.
#[test]
fn a_command_name_is_escaped_in_the_sentence() {
    assert_eq!(command_word(Some("a\nb")).text(), "a\\x0ab");
    assert_eq!(command_word(None).text(), "a command this wrapper could not read");
}
