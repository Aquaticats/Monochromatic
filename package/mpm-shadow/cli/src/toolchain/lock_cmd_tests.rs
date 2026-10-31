//! Unit tests of `lock`,
//! `lock --check` and the publish rules (revisions and the scope's OS lock),
//! all in disposable
//! scopes.

use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::Value;

use super::error::ErrorKind;
use super::fixtures::{SAMPLE_PLAN, sample_scope_on_disk};
use super::lock::{parse_lock, render_lock};
use super::lock_cmd::{LockMode, run_lock};
use super::publish::{acquire_guard, publish_lock, read_revisions, try_acquire_guard, write_atomically};

#[test]
fn check_passes_on_a_current_lock() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let line = run_lock(&scope, LockMode::Check).expect("a current lock passes");
    assert!(line.contains("11 tools"), "{line}");
    assert!(line.contains("digest verified"), "{line}");
}

#[test]
fn check_fails_without_a_lock_and_names_update() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    fs::remove_file(&scope.lock_file).expect("remove the lock");
    let error = run_lock(&scope, LockMode::Check).expect_err("no lock");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("mpm-shadow update"), "{error}");
}

#[test]
fn check_fails_after_a_plan_edit_that_the_lock_has_not_seen_and_names_lock() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let edited = SAMPLE_PLAN.replace("\"bins\": [\"node\", \"npm\", \"npx\"]", "\"bins\": [\"node\", \"npm\"]");
    fs::write(&scope.plan_file, edited).expect("edit the plan");
    let error = run_lock(&scope, LockMode::Check).expect_err("the lock is stale");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("node differs between the plan and the lock in: bins"), "{error}");
    assert!(error.message.contains("mpm-shadow lock"), "{error}");
}

#[test]
fn check_fails_on_a_hand_edited_resolved_version() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let text = fs::read_to_string(&scope.lock_file).expect("read");
    fs::write(&scope.lock_file, text.replace("\"version\": \"1.2.3\"", "\"version\": \"6.6.6\"")).expect("edit");
    let error = run_lock(&scope, LockMode::Check).expect_err("the digest no longer matches");
    assert!(error.message.contains("edited by hand"), "{error}");
}

#[test]
fn check_never_writes() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    fs::write(&scope.plan_file, SAMPLE_PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"fd\", \"fdfind\"]")).expect("edit the plan");
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    assert!(run_lock(&scope, LockMode::Check).is_err());
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
}

#[test]
fn lock_projects_a_plan_edit_and_keeps_every_resolved_value() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let edited = SAMPLE_PLAN.replace("\"bins\": [\"node\", \"npm\", \"npx\"]", "\"bins\": [\"node\", \"npm\"]");
    fs::write(&scope.plan_file, edited).expect("edit the plan");
    let before: Value = serde_json::from_str(&fs::read_to_string(&scope.lock_file).expect("read")).expect("JSON");
    let line = run_lock(&scope, LockMode::Write).expect("an offline projection");
    assert!(line.contains("rewritten"), "{line}");
    let after: Value = serde_json::from_str(&fs::read_to_string(&scope.lock_file).expect("read")).expect("JSON");
    assert_eq!(after["tools"]["node"]["bins"].as_array().expect("array").len(), 2);
    assert_eq!(after["tools"]["node"]["version"], before["tools"]["node"]["version"]);
    assert_eq!(after["tools"]["node"]["artifacts"], before["tools"]["node"]["artifacts"]);
    assert_ne!(after["digest"], before["digest"]);
    run_lock(&scope, LockMode::Check).expect("the rewritten lock passes the gate");
}

#[test]
fn lock_drops_a_tool_removed_from_the_plan() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let start = SAMPLE_PLAN.find("    \"pipx:slopo\"").expect("slopo is in the plan");
    let end = SAMPLE_PLAN.find("    \"android-sdk\"").expect("android-sdk is in the plan");
    let edited = format!("{}{}", &SAMPLE_PLAN[..start], &SAMPLE_PLAN[end..]);
    fs::write(&scope.plan_file, edited).expect("edit the plan");
    run_lock(&scope, LockMode::Write).expect("a removal projects offline");
    let parsed = parse_lock(&fs::read_to_string(&scope.lock_file).expect("read"), "lock").expect("parses");
    assert!(parsed.lock.tool("pipx:slopo").is_none());
    assert_eq!(parsed.lock.tools.len(), 10);
}

#[test]
fn lock_refuses_a_new_tool_names_update_and_leaves_the_lock_untouched() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let edited = SAMPLE_PLAN.replace(
        "    \"pipx:slopo\"",
        "    \"cargo:new-thing\": { \"request\": { \"track\": \"latest\" }, \"route\": \"cargo-binstall\", \"packageId\": \"new-thing\", \"bins\": [\"new-thing\"] },\n    \"pipx:slopo\"",
    );
    fs::write(&scope.plan_file, edited).expect("edit the plan");
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let error = run_lock(&scope, LockMode::Write).expect_err("a new tool needs the network");
    assert!(error.message.contains("cargo:new-thing is in the plan but not in the lock"), "{error}");
    assert!(error.message.contains("mpm-shadow update"), "{error}");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
}

#[test]
fn lock_refuses_a_changed_pin_without_touching_the_lock() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let edited = SAMPLE_PLAN.replace("{ \"pinned\": \"0.29.4\" }", "{ \"pinned\": \"0.30.0\" }");
    fs::write(&scope.plan_file, edited).expect("edit the plan");
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let error = run_lock(&scope, LockMode::Write).expect_err("a pin change needs the network");
    assert!(error.message.contains("request"), "{error}");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
}

#[test]
fn lock_on_a_current_lock_writes_nothing() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let line = run_lock(&scope, LockMode::Write).expect("nothing to do");
    assert!(line.contains("already current"), "{line}");
}

#[test]
fn a_lock_without_a_digest_needs_adopt_and_adopt_adds_it() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let mut value: Value = serde_json::from_str(&fs::read_to_string(&scope.lock_file).expect("read")).expect("JSON");
    value.as_object_mut().expect("object").remove("digest");
    fs::write(&scope.lock_file, serde_json::to_string_pretty(&value).expect("render")).expect("write the digest-less lock");

    let refused = run_lock(&scope, LockMode::Write).expect_err("a digest-less lock is not adopted silently");
    assert!(refused.message.contains("no digest"), "{refused}");
    assert!(run_lock(&scope, LockMode::Check).is_err());

    run_lock(&scope, LockMode::Adopt).expect("adopt adds the digest");
    run_lock(&scope, LockMode::Check).expect("the adopted lock passes");
    let again = run_lock(&scope, LockMode::Adopt).expect_err("a lock with a digest is not adopted again");
    assert_eq!(again.kind, ErrorKind::Config);
}

#[test]
fn adopt_still_refuses_a_lock_that_disagrees_with_the_plan() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let mut value: Value = serde_json::from_str(&fs::read_to_string(&scope.lock_file).expect("read")).expect("JSON");
    value.as_object_mut().expect("object").remove("digest");
    fs::write(&scope.lock_file, serde_json::to_string_pretty(&value).expect("render")).expect("write");
    fs::write(&scope.plan_file, SAMPLE_PLAN.replace("{ \"pinned\": \"0.29.4\" }", "{ \"pinned\": \"0.30.0\" }")).expect("edit");
    assert!(run_lock(&scope, LockMode::Adopt).is_err());
}

#[test]
fn a_bad_plan_is_a_configuration_error_with_exit_code_two() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    fs::write(&scope.plan_file, "{ \"tools\": ").expect("break the plan");
    let error = run_lock(&scope, LockMode::Check).expect_err("the plan is broken");
    assert_eq!(error.kind, ErrorKind::Config);
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn publishing_refuses_when_the_plan_changed_after_it_was_read() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let revisions = read_revisions(&scope).expect("revisions");
    fs::write(&scope.plan_file, format!("{SAMPLE_PLAN}\n// edited while the command ran\n")).expect("concurrent edit");
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let error = publish_lock(&scope, &revisions, "{}").expect_err("the plan moved");
    assert!(error.message.contains("toolchain.plan.jsonc"), "{error}");
    assert!(error.message.contains("run the command again"), "{error}");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
}

#[test]
fn publishing_refuses_when_the_lock_changed_after_it_was_read() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let revisions = read_revisions(&scope).expect("revisions");
    fs::write(&scope.lock_file, "{\"someone\": \"else\"}").expect("concurrent write");
    let error = publish_lock(&scope, &revisions, &render_lock(&super::fixtures::sample_lock())).expect_err("the lock moved");
    assert!(error.message.contains("toolchain.lock.json"), "{error}");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), "{\"someone\": \"else\"}");
}

#[test]
fn publishing_with_unchanged_revisions_writes_the_text_and_leaves_no_temporary_file() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let revisions = read_revisions(&scope).expect("revisions");
    publish_lock(&scope, &revisions, "new text\n").expect("published");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), "new text\n");
    let leftovers: Vec<String> = fs::read_dir(&scope.root)
        .expect("list")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().to_string())
        .filter(|name| return name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn a_lock_published_when_none_existed_needs_the_missing_revision_to_still_be_missing() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    fs::remove_file(&scope.lock_file).expect("remove");
    let revisions = read_revisions(&scope).expect("revisions");
    assert_eq!(revisions.lock.0, None);
    fs::write(&scope.lock_file, "raced into existence").expect("concurrent create");
    assert!(publish_lock(&scope, &revisions, "mine").is_err());
}

#[test]
fn the_scope_lock_is_exclusive_and_released_on_drop() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let held = acquire_guard(&scope).expect("the first holder");
    assert!(try_acquire_guard(&scope).expect("a probe").is_none(), "a second holder must not get the lock");
    drop(held);
    assert!(try_acquire_guard(&scope).expect("a probe").is_some(), "the lock is free after the drop");
}

#[test]
fn a_publish_waits_for_the_holder_of_the_scope_lock() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let released = Arc::new(AtomicBool::new(false));
    let held = acquire_guard(&scope).expect("the holder");
    let revisions = read_revisions(&scope).expect("revisions");
    let publisher_released = Arc::clone(&released);
    let publisher_scope = scope.clone();
    let publisher = std::thread::spawn(move || {
        publish_lock(&publisher_scope, &revisions, "after the holder\n").expect("published");
        return publisher_released.load(Ordering::SeqCst);
    });
    std::thread::sleep(Duration::from_millis(200));
    released.store(true, Ordering::SeqCst);
    drop(held);
    let saw_release_first = publisher.join().expect("the publisher thread ends");
    assert!(saw_release_first, "the publish returned before the holder released the lock");
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), "after the holder\n");
}

#[test]
fn atomic_writes_replace_the_target_in_one_step() {
    let (_directory, scope, _env) = sample_scope_on_disk();
    let target = scope.root.join("file.txt");
    write_atomically(&target, "one").expect("first");
    write_atomically(&target, "two").expect("second");
    assert_eq!(fs::read_to_string(&target).expect("read"), "two");
}
