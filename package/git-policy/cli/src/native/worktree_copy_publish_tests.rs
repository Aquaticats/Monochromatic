//! Publishing requests for newly registered worktrees, and finding what a killed wrapper left from its intent.

use super::*;
use crate::service_copy::tests::{COPIED_ENTRIES, Scene};
use crate::test_support::remove;
use crate::worktree_copy_intent::{intents_exist, write_intent};
use crate::worktree_copy_registry::{registration_token, write_copied_marker};
use crate::worktree_copy_request::{read_requests, requests_exist};

/// The worktree the registry would report for the directory named `name` under the scene.
fn created(scene: &Scene, name: &str, locked: bool) -> CreatedWorktree {
    let root: PathBuf = std::fs::canonicalize(scene.root.join(name)).expect("canonical root");
    let identity: String = admin_identity(scene.admin_root().as_path(), name).expect("identity").expect("registered");
    return CreatedWorktree { admin_id: name.to_string(), root, locked, identity };
}

/// Publishing writes one request and a second publication of the same registration finds it instead of writing another.
#[test]
fn a_registration_has_one_request_however_many_writers_publish() {
    let scene: Scene = Scene::new("publish-one");
    scene.add_worktree("topic");
    let worktree: CreatedWorktree = created(&scene, "topic", false);
    let first = publish_request(scene.common.as_path(), scene.source.as_path(), &CopySelection::Ignored, &worktree, false).expect("first");
    let second = publish_request(scene.common.as_path(), scene.source.as_path(), &CopySelection::Ignored, &worktree, false).expect("second");
    assert!(matches!(&first, Published::Request(_)), "{first:?}");
    assert_eq!(first, second);
    assert_eq!(read_requests(scene.common.as_path()).expect("read").0.len(), 1);
    remove(scene.root.as_path());
}

/// A registration whose copy is over gets no request.
#[test]
fn a_marked_registration_gets_no_request() {
    let scene: Scene = Scene::new("publish-marked");
    scene.add_worktree("topic");
    let worktree: CreatedWorktree = created(&scene, "topic", false);
    let token: String = ensure_registration_token(scene.admin_root().as_path(), "topic", Some(worktree.identity.as_str()))
        .expect("token")
        .expect("registered");
    write_copied_marker(scene.admin_root().as_path(), "topic", token.as_str()).expect("marked");
    let published = publish_request(scene.common.as_path(), scene.source.as_path(), &CopySelection::Ignored, &worktree, false).expect("published");
    assert_eq!(published, Published::Already);
    assert!(!requests_exist(scene.common.as_path()).expect("requests"));
    remove(scene.root.as_path());
}

/// A worktree observed under an identity the directory no longer has is gone: nothing is tagged and no request is written.
#[test]
fn a_replaced_registration_is_gone_and_is_not_tagged() {
    let scene: Scene = Scene::new("publish-replaced");
    scene.add_worktree("topic");
    let stale: CreatedWorktree = CreatedWorktree { identity: String::from("9:9:-"), ..created(&scene, "topic", false) };
    let published = publish_request(scene.common.as_path(), scene.source.as_path(), &CopySelection::Ignored, &stale, false).expect("published");
    assert_eq!(published, Published::Gone);
    assert_eq!(registration_token(scene.admin_root().as_path(), "topic").expect("token"), None, "the replacement was tagged");
    assert!(!requests_exist(scene.common.as_path()).expect("requests"));
    remove(scene.root.as_path());
}

/// A wrapper killed after Git returned leaves its intent; the service writes the request for the worktree Git created.
#[test]
fn an_intent_alone_yields_the_request_of_a_worktree_the_dead_wrapper_created() {
    let scene: Scene = Scene::new("publish-intent");
    let intent: Intent = scene.intent(None);
    scene.add_worktree("topic");
    assert!(!requests_exist(scene.common.as_path()).expect("requests"));
    reconcile_intents(scene.common.as_path(), 0).expect("reconciled");
    let (requests, _refused) = read_requests(scene.common.as_path()).expect("read");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].admin_id, "topic");
    assert_eq!(requests[0].source_root, intent.source_root);
    assert!(!requests[0].allow_locked);
    let events = scene.drain();
    assert_eq!(Scene::outcomes(&events).len(), 1);
    assert_eq!(Scene::outcomes(&events)[0].1, crate::service_copy::CopyOutcome::Copied(COPIED_ENTRIES));
    remove(scene.root.as_path());
}

/// A worktree that existed before the command is not the command's: the intent's before-set tells them apart.
#[test]
fn an_intent_ignores_the_worktrees_that_existed_before_it() {
    let scene: Scene = Scene::new("publish-before");
    scene.add_worktree("old");
    scene.intent(None);
    reconcile_intents(scene.common.as_path(), 0).expect("reconciled");
    assert!(!requests_exist(scene.common.as_path()).expect("requests"));
    remove(scene.root.as_path());
}

/// A locked new worktree is another command's while it is created, except the one this command named on its command line.
#[test]
fn an_intent_fills_the_locked_worktree_it_named_and_leaves_a_locked_peer() {
    let scene: Scene = Scene::new("publish-locked");
    let mine: PathBuf = scene.root.join("mine");
    scene.intent(Some(mine.as_path()));
    scene.add_locked_worktree("mine");
    scene.add_locked_worktree("peer");
    reconcile_intents(scene.common.as_path(), 0).expect("reconciled");
    let (requests, _refused) = read_requests(scene.common.as_path()).expect("read");
    assert_eq!(requests.iter().map(|request| return request.admin_id.as_str()).collect::<Vec<&str>>(), vec!["mine"]);
    assert!(requests[0].allow_locked, "the creator's own lock is allowed at run time too");
    remove(scene.root.as_path());
}

/// An intent that says Git returned is dropped as soon as its requests are written, and the worktrees it found are filled.
#[test]
fn a_returned_intent_goes_once_its_requests_are_written() {
    let scene: Scene = Scene::new("publish-returned");
    let intent: Intent = scene.intent(None);
    scene.add_worktree("topic");
    scene.mark_returned(&intent);
    reconcile_intents(scene.common.as_path(), 0).expect("reconciled");
    assert!(!intents_exist(scene.common.as_path()).expect("intents"), "a returned intent stays");
    assert_eq!(read_requests(scene.common.as_path()).expect("read").0.len(), 1);
    remove(scene.root.as_path());
}

/// An intent that does not say Git returned is kept: its wrapper may be alive and waiting for Git.
#[test]
fn an_intent_whose_git_has_not_returned_is_kept_after_it_is_read() {
    let scene: Scene = Scene::new("publish-started");
    scene.intent(None);
    scene.add_worktree("topic");
    reconcile_intents(scene.common.as_path(), 0).expect("reconciled");
    assert!(intents_exist(scene.common.as_path()).expect("intents"));
    assert_eq!(read_requests(scene.common.as_path()).expect("read").0.len(), 1, "the worktree it found is still requested");
    remove(scene.root.as_path());
}

/// An intent is read again at every drain until it is an hour old; after that it is removed, because no Git command the
/// wrapper started can still be running.
#[test]
fn an_intent_is_kept_for_an_hour_and_then_removed() {
    let scene: Scene = Scene::new("publish-expiry");
    let intent: Intent = scene.intent(None);
    let written: u128 = intent.written_at().expect("a time");
    reconcile_intents(scene.common.as_path(), written + INTENT_KEPT_MILLISECONDS).expect("on the last minute");
    assert!(intents_exist(scene.common.as_path()).expect("intents"), "an intent just inside the hour was removed");
    reconcile_intents(scene.common.as_path(), written + INTENT_KEPT_MILLISECONDS + 1).expect("after the hour");
    assert!(!intents_exist(scene.common.as_path()).expect("intents"), "an old intent stays for ever");
    remove(scene.root.as_path());
}

/// An intent that cannot be read is named in a notice instead of being dropped.
#[test]
fn an_unreadable_intent_is_named_in_a_notice() {
    let scene: Scene = Scene::new("publish-unreadable");
    let intent: Intent = scene.intent(None);
    std::fs::write(crate::worktree_copy_intent::intents_directory(scene.common.as_path()).join("0000-broken.json"), b"{").expect("broken");
    let notices: Vec<String> = reconcile_intents(scene.common.as_path(), 0).expect("reconciled");
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(notices[0].contains("0000-broken.json"));
    write_intent(scene.common.as_path(), &intent).expect("rewritten");
    remove(scene.root.as_path());
}

/// The root of a worktree comes from the pointer file Git keeps, absolute or relative to its administrative directory.
#[test]
fn the_root_comes_from_the_pointer_file() {
    let scene: Scene = Scene::new("publish-pointer");
    let topic: PathBuf = scene.add_worktree("topic");
    assert_eq!(root_from_pointer(scene.admin_root().join("topic").as_path()), Some(topic.clone()));
    let relative: PathBuf = scene.root.join("relative-admin");
    std::fs::create_dir(relative.as_path()).expect("directory");
    let target: PathBuf = pathdiff(topic.join(".git").as_path(), relative.as_path());
    std::fs::write(relative.join("gitdir"), format!("{}\n", target.display())).expect("pointer");
    assert_eq!(root_from_pointer(relative.as_path()), Some(topic.clone()));
    std::fs::write(relative.join("gitdir"), "/somewhere/else\n").expect("not a .git pointer");
    assert_eq!(root_from_pointer(relative.as_path()), None);
    assert_eq!(root_from_pointer(scene.root.join("absent").as_path()), None);
    remove(scene.root.as_path());
}

/// A relative path from `from` to `to`, both inside one fixture root.
fn pathdiff(to: &Path, from: &Path) -> PathBuf {
    let mut up: PathBuf = PathBuf::new();
    let mut ancestor: &Path = from;
    while !to.starts_with(ancestor) {
        up.push("..");
        ancestor = ancestor.parent().expect("a common ancestor");
    }
    return up.join(to.strip_prefix(ancestor).expect("inside"));
}
