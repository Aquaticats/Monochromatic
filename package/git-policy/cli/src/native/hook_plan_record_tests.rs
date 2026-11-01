//! Controls for the plan record: its bytes, its missing sync, the refusal of an existing directory and the absence of
//!  any hook entry.

use super::*;
use crate::private_storage::take_file_syncs;
use crate::test_support::{fixture, remove};

/// A plan with fixed values.
fn sample_plan() -> HookDispatchPlan {
    return HookDispatchPlan {
        disabled_events: Vec::new(),
        lease: String::from("lease"),
        hook_lock_path: String::from("/repo/.git/cli-git/hook.lock"),
        skip_hook_lock: false,
        named_hooks_possible: true,
    };
}

/// The plan file holds the encoded plan and one newline.
#[test]
fn the_plan_file_holds_the_encoded_plan_and_a_newline() {
    let root: PathBuf = fixture("hook-plan-record-bytes");
    let hooks: PathBuf = root.join("hooks");
    let plan: HookDispatchPlan = sample_plan();
    write_hook_plan_record(hooks.as_path(), &plan).expect("write");
    let bytes: String = std::fs::read_to_string(hooks.join(PLAN_FILENAME)).expect("plan");
    assert_eq!(bytes, format!("{}\n", encode_hook_dispatch_plan(&plan)));
    remove(root.as_path());
}

/// The plan is written without a file sync: recovery completes or discards whatever a power loss left of it.
#[test]
fn the_plan_file_is_not_synced() {
    let root: PathBuf = fixture("hook-plan-record-sync");
    take_file_syncs();
    write_hook_plan_record(root.join("hooks").as_path(), &sample_plan()).expect("write");
    assert_eq!(take_file_syncs(), Vec::<PathBuf>::new());
    remove(root.as_path());
}

/// The hooks directory holds the plan and nothing else, whatever the configuration defines: no hook entry names the
///  wrapper, so Git has nothing in the directory to start.
#[test]
fn the_hooks_directory_holds_only_the_plan() {
    let root: PathBuf = fixture("hook-plan-record-only");
    for named_hooks_possible in [false, true] {
        let hooks: PathBuf = root.join(format!("hooks-{named_hooks_possible}"));
        let plan: HookDispatchPlan = HookDispatchPlan { named_hooks_possible, ..sample_plan() };
        write_hook_plan_record(hooks.as_path(), &plan).expect("write");
        let names: Vec<String> = std::fs::read_dir(&hooks)
            .expect("hooks directory")
            .flatten()
            .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
            .collect();
        assert_eq!(names, vec![String::from(PLAN_FILENAME)], "named hooks possible: {named_hooks_possible}");
    }
    remove(root.as_path());
}

/// The hooks directory is new, so an existing one is refused and left untouched.
#[test]
fn an_existing_hooks_directory_is_refused() {
    let root: PathBuf = fixture("hook-plan-record-existing");
    let hooks: PathBuf = root.join("hooks");
    std::fs::create_dir(&hooks).expect("existing");
    std::fs::write(hooks.join("marker"), b"kept").expect("marker");
    let result = write_hook_plan_record(hooks.as_path(), &sample_plan());
    assert!(result.is_err());
    assert_eq!(std::fs::read(hooks.join("marker")).expect("marker"), b"kept");
    remove(root.as_path());
}
