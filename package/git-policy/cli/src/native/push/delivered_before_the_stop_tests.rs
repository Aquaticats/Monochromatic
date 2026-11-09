//! Controls for the proof that a stopped push had already delivered its references:
//! the questions it asks of the arguments and of the remote.
//!  The end-to-end control,
//! a stopped push whose local remote ran a hook after the update,
//! is in `stopped_push_delivery_tests.rs`.

use super::*;
use crate::test_support::{REAL_GIT, fixture, git, git_text, remove, repository};

/// One argument list of the shape the wrapper builds,
/// as `OsString`s.
fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(|word| { return OsString::from(word); }).collect();
}

/// The hooks-off switches of a real start come before the subcommand and take a value each,
/// and the proof reads the push that follows them.
#[test]
fn the_region_is_what_follows_the_push_word_past_the_options_before_it() {
    let typed: Vec<OsString> = arguments(&[
        "-c",
        "core.hooksPath=/dev/null/cli-git-no-hooks",
        "-c",
        "hook.pre-push.enabled=false",
        "--no-pager",
        "push",
        "origin",
        "refs/heads/main:refs/heads/main",
    ]);
    assert_eq!(
        after_the_subcommand(typed.as_slice()).map(<[OsString]>::to_vec),
        Some(arguments(&["origin", "refs/heads/main:refs/heads/main"]))
    );
    assert_eq!(after_the_subcommand(arguments(&["--version"]).as_slice()), None, "no subcommand");
    assert_eq!(after_the_subcommand(arguments(&["fetch", "origin"]).as_slice()), None, "not a push");
}

/// A refspec names the reference to ask the local repository and the remote about.
#[test]
fn a_refspec_gives_the_reference_on_each_side() {
    assert_eq!(
        refspec_pair(OsStr::new("refs/heads/main:refs/heads/main")),
        Some((OsString::from("refs/heads/main"), OsString::from("refs/heads/main")))
    );
    assert_eq!(
        refspec_pair(OsStr::new("+refs/heads/feature:refs/heads/topic")),
        Some((OsString::from("refs/heads/feature"), OsString::from("refs/heads/topic")))
    );
    assert_eq!(
        refspec_pair(OsStr::new("0123456789abcdef0123456789abcdef01234567:refs/heads/main")),
        Some((OsString::from("0123456789abcdef0123456789abcdef01234567"), OsString::from("refs/heads/main")))
    );
    // A deletion, a forced deletion and a short name on the remote side are not questions the proof can ask.
    assert_eq!(refspec_pair(OsStr::new(":refs/heads/gone")), None);
    assert_eq!(refspec_pair(OsStr::new("refs/heads/main:")), None);
    assert_eq!(refspec_pair(OsStr::new("main")), None);
    assert_eq!(refspec_pair(OsStr::new("refs/heads/main^:refs/heads/main")), None);
}

/// The push the wrapper plans carries its destination and refspec,
/// and the forms the proof cannot read answer nothing.
#[test]
fn the_question_is_read_from_the_push_the_wrapper_plans() {
    let named: Vec<OsString> = arguments(&["push", "origin", "refs/heads/main:refs/heads/main"]);
    let question: DeliveryQuestion = DeliveryQuestion::of(named.as_slice()).expect("a question");
    assert_eq!(question.remote, OsString::from("origin"));
    assert_eq!(question.refs, vec![(OsString::from("refs/heads/main"), OsString::from("refs/heads/main"))]);

    let upstream: Vec<OsString> = arguments(&["push", "--set-upstream", "origin", "refs/heads/feature:refs/heads/feature"]);
    assert_eq!(
        DeliveryQuestion::of(upstream.as_slice()).expect("a question").refs.len(),
        1,
        "the option before the destination changes nothing"
    );

    let matching: Vec<OsString> = arguments(&["push", "origin"]);
    assert_eq!(DeliveryQuestion::of(matching.as_slice()), None, "a matching push names no reference");
    let all: Vec<OsString> = arguments(&["push", "--all", "origin"]);
    assert_eq!(DeliveryQuestion::of(all.as_slice()), None, "--all names no reference");
    let tags: Vec<OsString> = arguments(&["push", "--tags", "origin"]);
    assert_eq!(DeliveryQuestion::of(tags.as_slice()), None, "--tags names no reference");
    let mirror: Vec<OsString> = arguments(&["push", "--mirror", "origin"]);
    assert_eq!(DeliveryQuestion::of(mirror.as_slice()), None, "--mirror names no reference");
    let delete: Vec<OsString> = arguments(&["push", "--delete", "origin", "refs/heads/gone"]);
    assert_eq!(DeliveryQuestion::of(delete.as_slice()), None, "a deletion has nothing to deliver");
}

/// A reference the remote holds at the object the local repository names is delivered;
/// one it does not hold,
/// or holds at another object,
/// is not.
#[test]
fn a_reference_is_delivered_when_the_remote_holds_the_object_the_local_reference_names() {
    let root: PathBuf = fixture("push-delivery-proof");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--quiet", "--bare", remote.to_str().expect("UTF-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("file"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "file"]);
    git(repo.as_path(), &["-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "--quiet", "-m", "one"]);
    let question: DeliveryQuestion =
        DeliveryQuestion::of(arguments(&["push", "origin", "refs/heads/main:refs/heads/main"]).as_slice()).expect("a question");
    let asking: AskingGit = AskingGit {
        program: PathBuf::from(REAL_GIT),
        cwd: Some(repo.clone()),
        environment: crate::test_support::scratch_environment(),
    };
    let sending: Vec<String> = crate::lazy_runtime::run_on_runtime(objects_the_push_will_send(
        &asking,
        &question,
        Duration::from_secs(30),
        Duration::from_secs(2),
    ))
    .expect("the runtime")
    .expect("the object the push is going to send");
    assert_eq!(sending.len(), 1, "one object is recorded for the one reference");
    let before: Option<Vec<String>> = crate::lazy_runtime::run_on_runtime(refs_already_delivered(
        &asking,
        &question,
        sending.as_slice(),
        Duration::from_secs(30),
        Duration::from_secs(2),
    ))
    .expect("the runtime");
    assert_eq!(before, None, "nothing was pushed yet");
    git(repo.as_path(), &["push", "--quiet", "origin", "refs/heads/main:refs/heads/main"]);
    let after: Option<Vec<String>> = crate::lazy_runtime::run_on_runtime(refs_already_delivered(
        &asking,
        &question,
        sending.as_slice(),
        Duration::from_secs(30),
        Duration::from_secs(2),
    ))
    .expect("the runtime");
    assert_eq!(after, Some(vec![String::from("refs/heads/main")]), "the remote holds what the push sent");
    // A local reference that moved after the recording does not change what was sent, so the answer stands.
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "-m", "two"]);
    let moved: Option<Vec<String>> = crate::lazy_runtime::run_on_runtime(refs_already_delivered(
        &asking,
        &question,
        sending.as_slice(),
        Duration::from_secs(30),
        Duration::from_secs(2),
    ))
    .expect("the runtime");
    assert_eq!(moved, Some(vec![String::from("refs/heads/main")]), "the recorded objects still decide the answer");
    // A remote reference at another object than the one a later push would send is not the delivered one.
    let later: Vec<String> = crate::lazy_runtime::run_on_runtime(objects_the_push_will_send(
        &asking,
        &question,
        Duration::from_secs(30),
        Duration::from_secs(2),
    ))
    .expect("the runtime")
    .expect("the object the next push would send");
    assert_ne!(later, sending, "the next push would send another object");
    let ahead: Option<Vec<String>> = crate::lazy_runtime::run_on_runtime(refs_already_delivered(
        &asking,
        &question,
        later.as_slice(),
        Duration::from_secs(30),
        Duration::from_secs(2),
    ))
    .expect("the runtime");
    assert_eq!(ahead, None, "the remote does not hold the object that push would send");
    assert_ne!(git_text(repo.as_path(), &["rev-parse", "refs/heads/main"]), String::new());
    remove(root.as_path());
}
