//! Controls for the hook entry writer: the plan bytes, the entry names and the refusal of an existing directory.

use super::*;
use crate::hook_dispatch_plan::{ConfigParameters, parse_hook_dispatch_plan};
use crate::test_support::{fixture, remove};

/// A plan with fixed values.
fn sample_plan() -> HookDispatchPlan {
    return HookDispatchPlan {
        git_path: String::from("/usr/bin/git"),
        disabled_events: Vec::new(),
        config_parameters: ConfigParameters::Absent,
        worktree_root: String::from("/repo"),
        lease: String::from("lease"),
        hook_lock_path: String::from("/repo/.git/cli-git/hook.lock"),
        skip_hook_lock: false,
        named_hooks_possible: true,
    };
}

/// The plan file holds the encoded plan and one newline, as the incumbent writes it.
#[test]
fn the_plan_file_holds_the_encoded_plan_and_a_newline() {
    let root: PathBuf = fixture("hook-entries-plan");
    let hooks: PathBuf = root.join("hooks");
    let plan: HookDispatchPlan = sample_plan();
    write_hook_dispatch(hooks.as_path(), &plan, Path::new("/opt/native/bin/git")).expect("write");
    let bytes: String = std::fs::read_to_string(hooks.join(PLAN_FILENAME)).expect("plan");
    assert_eq!(bytes, format!("{}\n", encode_hook_dispatch_plan(&plan)));
    assert_eq!(parse_hook_dispatch_plan(bytes.as_str()), Some(plan));
    remove(root.as_path());
}

/// Only the three preparation events get entries, and post-commit never does.
#[cfg(unix)]
#[test]
fn entries_are_symbolic_links_named_after_preparation_events() {
    let root: PathBuf = fixture("hook-entries-links");
    let hooks: PathBuf = root.join("hooks");
    let executable: PathBuf = PathBuf::from("/opt/native/bin/git");
    write_hook_dispatch(hooks.as_path(), &sample_plan(), executable.as_path()).expect("write");
    for event in PREPARATION_HOOK_EVENTS {
        let entry: PathBuf = hooks.join(event);
        assert_eq!(std::fs::read_link(&entry).expect("link"), executable, "{event}");
    }
    assert!(std::fs::symlink_metadata(hooks.join("post-commit")).is_err());
    assert!(std::fs::symlink_metadata(hooks.join("dispatch.mjs")).is_err());
    assert_eq!(
        std::fs::read_dir(&hooks).expect("entries").count(),
        PREPARATION_HOOK_EVENTS.len() + 1
    );
    remove(root.as_path());
}

/// The entry names a plan writes, sorted, without the plan file.
fn entry_names(hooks: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(hooks)
        .expect("hooks directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name != PLAN_FILENAME; })
        .collect();
    names.sort();
    return names;
}

/// A configuration that defines no hook by name gets no entry, whatever sits in the repository's hooks directory.
#[cfg(unix)]
#[test]
fn a_configuration_without_named_hooks_gets_no_entry() {
    let root: PathBuf = fixture("hook-entries-omitted");
    let user_hooks: PathBuf = root.join("user-hooks");
    std::fs::create_dir(&user_hooks).expect("user hooks");
    std::fs::write(user_hooks.join("commit-msg"), b"not executable").expect("plain file");
    std::fs::create_dir(user_hooks.join("pre-commit")).expect("directory");
    let plan: HookDispatchPlan = HookDispatchPlan {
        named_hooks_possible: false,
        ..sample_plan()
    };
    let executable: PathBuf = PathBuf::from("/opt/native/bin/git");
    write_hook_dispatch(root.join("none").as_path(), &plan, executable.as_path()).expect("write");
    assert_eq!(entry_names(root.join("none").as_path()), Vec::<String>::new());
    assert!(root.join("none").join(PLAN_FILENAME).is_file(), "the plan is still written");
    remove(root.as_path());
}

/// A configuration that defines hooks by name keeps all three entries whatever the directory holds.
#[cfg(unix)]
#[test]
fn named_hooks_keep_every_entry() {
    let root: PathBuf = fixture("hook-entries-named");
    let plan: HookDispatchPlan = HookDispatchPlan {
        named_hooks_possible: true,
        ..sample_plan()
    };
    write_hook_dispatch(root.join("hooks").as_path(), &plan, Path::new("/opt/native/bin/git")).expect("write");
    assert_eq!(
        entry_names(root.join("hooks").as_path()),
        vec![String::from("commit-msg"), String::from("pre-commit"), String::from("prepare-commit-msg")]
    );
    remove(root.as_path());
}

/// The hooks directory is new, so an existing one is refused and left untouched.
#[test]
fn an_existing_hooks_directory_is_refused() {
    let root: PathBuf = fixture("hook-entries-existing");
    let hooks: PathBuf = root.join("hooks");
    std::fs::create_dir(&hooks).expect("existing");
    std::fs::write(hooks.join("marker"), b"kept").expect("marker");
    let result = write_hook_dispatch(hooks.as_path(), &sample_plan(), Path::new("/opt/git"));
    assert!(result.is_err());
    assert_eq!(std::fs::read(hooks.join("marker")).expect("marker"), b"kept");
    remove(root.as_path());
}

/// The absent hooks directory sits beside the dispatcher directory in the transaction and is never created.
#[test]
fn the_absent_hooks_directory_sits_beside_the_dispatcher_directory() {
    let root: PathBuf = fixture("hook-entries-no-hooks");
    let transaction: PathBuf = root.join("tx");
    std::fs::create_dir(&transaction).expect("transaction");
    write_hook_dispatch(transaction.join("hooks").as_path(), &sample_plan(), Path::new("/opt/git")).expect("write");
    let absent: PathBuf = transaction_no_hooks_directory(transaction.as_path());
    assert_eq!(absent, transaction.join("no-hooks"));
    assert!(!absent.exists(), "writing the dispatcher directory must not create the absent one");
    remove(root.as_path());
}

/// The entry name is the event on Unix and the event with `.exe` on Windows.
#[test]
fn entry_names_follow_the_platform() {
    if cfg!(windows) {
        assert_eq!(hook_entry_name("pre-commit"), "pre-commit.exe");
    } else {
        assert_eq!(hook_entry_name("pre-commit"), "pre-commit");
    }
}
