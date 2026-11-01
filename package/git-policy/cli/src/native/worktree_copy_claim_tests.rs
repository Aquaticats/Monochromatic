//! Tests of the destination claim: publication, the evidence that tells a live claim from an abandoned one, and the
//!  open-then-scan adoption that lets exactly one process settle an abandoned claim.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_journal::{ensure_journal_root, journal_root};
use crate::worktree_copy_selection::STAGE_PREFIX;
use std::io::BufRead;
use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Command, Stdio};

/// A journal root beside a destination parent, as a copy into `worktrees/topic` would use.
fn scene(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(common.as_path()).expect("common");
    let worktrees: PathBuf = root.join("worktrees");
    std::fs::create_dir_all(worktrees.as_path()).expect("worktrees");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("journal root");
    return (root, journals, worktrees.join("topic"));
}

/// A stage container name beside a destination, as `new_stage_container` would choose.
fn container_for(destination: &Path, label: &str) -> PathBuf {
    return destination
        .parent()
        .expect("parent")
        .join(format!("{STAGE_PREFIX}{label}"));
}

/// Publishes the claim for `topic` and expects to win the name.
fn publish(journals: &Path, destination: &Path, label: &str) -> Claim {
    return publish_claim(journals, "topic", destination, container_for(destination, label).as_path())
        .expect("published")
        .expect("won the name");
}

/// A child process that holds a file open and says so, the way a running copy holds its claim.
fn holder_of(path: &Path) -> Child {
    let mut child: Child = Command::new("sh")
        .args(["-c", "exec 3<\"$1\"; echo ready; exec sleep 3600", "holder"])
        .arg(path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("holder starts");
    let stdout = child.stdout.take().expect("stdout");
    let mut line: String = String::new();
    std::io::BufReader::new(stdout).read_line(&mut line).expect("ready line");
    assert_eq!(line, "ready\n");
    return child;
}

/// Kills a child process the way an out-of-memory kill does and waits until the kernel has released everything it held.
fn kill_and_reap(mut child: Child) {
    child.kill().expect("killed");
    child.wait().expect("reaped");
}

/// A published claim says what the copy will use, is private to the account, and leaves no temporary behind.
#[test]
fn a_published_claim_names_the_container_and_leaves_no_temporary() {
    let (root, journals, destination) = scene("claim-published");
    let claim: Claim = publish(journals.as_path(), destination.as_path(), "one");
    assert_eq!(claim.path(), journals.join("topic.claim").as_path());
    assert_eq!(claim.container(), container_for(destination.as_path(), "one"));
    assert!(!claim.container().exists(), "the claim precedes the container");
    let text: String = std::fs::read_to_string(claim.path()).expect("claim file");
    assert_eq!(parse_content(text.as_str()).as_ref(), Some(claim.content()));
    assert_eq!(claim.content().destination_root, destination.to_string_lossy());
    assert_eq!(claim.content().pid_namespace, current_pid_namespace());
    assert!(claim.content().observable, "an ordinary process can be seen holding its claim");
    assert_eq!(
        std::fs::metadata(claim.path()).expect("metadata").permissions().mode() & 0o777,
        CLAIM_MODE
    );
    let names: Vec<String> = std::fs::read_dir(journals.as_path())
        .expect("listing")
        .map(|entry| return entry.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec![String::from("topic.claim")], "no temporary stays");
    remove(root.as_path());
}

/// A claim name is taken by exactly one creator: the second is refused while the first lives, and the first claim's
///  content is untouched.
#[test]
fn a_second_claim_for_one_destination_is_refused_while_the_first_lives() {
    let (root, journals, destination) = scene("claim-second");
    let first: Claim = publish(journals.as_path(), destination.as_path(), "first");
    let second = publish_claim(
        journals.as_path(),
        "topic",
        destination.as_path(),
        container_for(destination.as_path(), "second").as_path(),
    )
    .expect("attempted");
    assert!(second.is_none(), "the name is taken");
    let kept: String = std::fs::read_to_string(first.path()).expect("claim file");
    assert_eq!(parse_content(kept.as_str()).as_ref(), Some(first.content()));
    assert_eq!(std::fs::read_dir(journals.as_path()).expect("listing").count(), 1, "no temporary stays");
    remove(root.as_path());
}

/// Different destinations never contend: their claims are independent files.
#[test]
fn claims_for_different_destinations_are_independent() {
    let (root, journals, destination) = scene("claim-independent");
    let topic: Claim = publish(journals.as_path(), destination.as_path(), "topic");
    let other: Claim = publish_claim(
        journals.as_path(),
        "other",
        destination.as_path(),
        container_for(destination.as_path(), "other").as_path(),
    )
    .expect("published")
    .expect("a different name is free");
    assert_ne!(topic.path(), other.path());
    remove(root.as_path());
}

/// A claim whose creator still holds it open is held, not adoptable, and stays as it is.
#[test]
fn a_live_claim_is_held_and_not_adopted() {
    let (root, journals, destination) = scene("claim-live");
    let claim: Claim = publish(journals.as_path(), destination.as_path(), "live");
    match examine_claim(claim.path()).expect("examined") {
        Examination::Held { content, reason } => {
            assert_eq!(&content, claim.content());
            assert!(reason.contains("descriptor"), "{reason}");
        }
        other => panic!("expected a held claim, got {other:?}"),
    }
    assert!(claim.path().exists());
    remove(root.as_path());
}

/// A claim another process holds open is held until that process dies, however it dies, and then it is adoptable.
#[test]
fn a_claim_another_process_holds_is_held_until_that_process_is_killed() {
    let (root, journals, destination) = scene("claim-process");
    let path: PathBuf = journals.join("topic.claim");
    drop(publish(journals.as_path(), destination.as_path(), "process"));
    let child: Child = holder_of(path.as_path());
    let pid: u32 = child.id();
    match examine_claim(path.as_path()).expect("examined") {
        Examination::Held { reason, .. } => assert!(reason.contains(pid.to_string().as_str()), "{reason}"),
        other => panic!("expected a held claim, got {other:?}"),
    }
    kill_and_reap(child);
    match examine_claim(path.as_path()).expect("examined") {
        Examination::Abandoned(claim) => assert_eq!(claim.path(), path.as_path()),
        other => panic!("expected an abandoned claim, got {other:?}"),
    }
    remove(root.as_path());
}

/// A claim nobody holds is adopted by the first examiner, and the adopter is then the claim's one holder:
///  a second examiner finds it held.
#[test]
fn an_abandoned_claim_is_adopted_by_one_examiner() {
    let (root, journals, destination) = scene("claim-adopt");
    drop(publish(journals.as_path(), destination.as_path(), "dead"));
    let path: PathBuf = journals.join("topic.claim");
    let adopted: Claim = match examine_claim(path.as_path()).expect("examined") {
        Examination::Abandoned(claim) => claim,
        other => panic!("expected an abandoned claim, got {other:?}"),
    };
    assert!(matches!(examine_claim(path.as_path()).expect("second"), Examination::Held { .. }));
    adopted.release().expect("released");
    assert!(!path.exists());
    assert!(matches!(examine_claim(path.as_path()).expect("after"), Examination::Gone));
    remove(root.as_path());
}

/// A holder that appears between the first look and the open stops the adoption: the examiner scans again after it
///  holds the file itself, so two examiners that open together each see the other and neither acts.
#[test]
fn a_holder_that_appears_between_the_looks_stops_the_adoption() {
    let (root, journals, destination) = scene("claim-between");
    drop(publish(journals.as_path(), destination.as_path(), "racing"));
    let path: PathBuf = journals.join("topic.claim");
    let mut rival: Option<std::fs::File> = None;
    let examined = examine_claim_with(path.as_path(), &mut || {
        rival = Some(std::fs::File::open(path.as_path()).expect("the rival opens the claim"));
    })
    .expect("examined");
    assert!(matches!(examined, Examination::Held { .. }), "{examined:?}");
    assert!(rival.is_some());
    drop(rival);
    assert!(matches!(examine_claim(path.as_path()).expect("later"), Examination::Abandoned(_)));
    remove(root.as_path());
}

/// A claim replaced between the first look and the open is not the one that was looked at.
#[test]
fn a_claim_replaced_between_the_looks_is_gone() {
    let (root, journals, destination) = scene("claim-replaced");
    drop(publish(journals.as_path(), destination.as_path(), "old"));
    let path: PathBuf = journals.join("topic.claim");
    let examined = examine_claim_with(path.as_path(), &mut || {
        std::fs::remove_file(path.as_path()).expect("removed");
        std::mem::forget(publish(journals.as_path(), destination.as_path(), "new"));
    })
    .expect("examined");
    assert!(matches!(examined, Examination::Gone), "{examined:?}");
    remove(root.as_path());
}

/// A claim that was removed is gone, not an error.
#[test]
fn a_missing_claim_is_gone() {
    let (root, journals, _destination) = scene("claim-missing");
    assert!(matches!(
        examine_claim(journals.join("absent.claim").as_path()).expect("examined"),
        Examination::Gone
    ));
    remove(root.as_path());
}

/// A claim made in another PID namespace is judged by its age: held while young, abandoned once it outlives the patience.
#[test]
fn a_claim_of_another_namespace_is_judged_by_age() {
    let (root, journals, destination) = scene("claim-foreign");
    let path: PathBuf = journals.join("topic.claim");
    let content: ClaimContent = ClaimContent {
        destination_root: destination.to_string_lossy().into_owned(),
        observable: true,
        pid_namespace: Some(String::from("pid:[1]")),
        stage_container: container_for(destination.as_path(), "foreign").to_string_lossy().into_owned(),
    };
    crate::private_storage::write_private_file(path.as_path(), content.encode().as_bytes()).expect("claim");
    match examine_claim(path.as_path()).expect("young") {
        Examination::Held { reason, .. } => assert!(reason.contains("another PID namespace"), "{reason}"),
        other => panic!("expected a held claim, got {other:?}"),
    }
    let old: std::time::SystemTime = std::time::SystemTime::now()
        - crate::pid_namespace::FOREIGN_OWNER_PATIENCE
        - std::time::Duration::from_secs(60);
    std::fs::File::options()
        .write(true)
        .open(path.as_path())
        .expect("open")
        .set_modified(old)
        .expect("aged");
    assert!(matches!(examine_claim(path.as_path()).expect("old"), Examination::Abandoned(_)));
    remove(root.as_path());
}

/// A claim whose creator hides its descriptors from other processes cannot be judged by the process table, so it is
///  judged by age like a claim of another namespace.
#[test]
fn a_claim_whose_creator_hides_its_descriptors_is_judged_by_age() {
    let (root, journals, destination) = scene("claim-hidden");
    let path: PathBuf = journals.join("topic.claim");
    let content: ClaimContent = ClaimContent {
        destination_root: destination.to_string_lossy().into_owned(),
        observable: false,
        pid_namespace: current_pid_namespace(),
        stage_container: container_for(destination.as_path(), "hidden").to_string_lossy().into_owned(),
    };
    crate::private_storage::write_private_file(path.as_path(), content.encode().as_bytes()).expect("claim");
    match examine_claim(path.as_path()).expect("young") {
        Examination::Held { reason, .. } => assert!(reason.contains("hides its descriptors"), "{reason}"),
        other => panic!("expected a held claim, got {other:?}"),
    }
    let old: std::time::SystemTime = std::time::SystemTime::now()
        - crate::pid_namespace::FOREIGN_OWNER_PATIENCE
        - std::time::Duration::from_secs(60);
    std::fs::File::options()
        .write(true)
        .open(path.as_path())
        .expect("open")
        .set_modified(old)
        .expect("aged");
    assert!(matches!(examine_claim(path.as_path()).expect("old"), Examination::Abandoned(_)));
    remove(root.as_path());
}

/// A process that hides its own descriptors from other processes would not be seen holding what it adopts, so it
///  leaves an abandoned claim to others.
#[test]
fn a_process_that_hides_its_descriptors_does_not_adopt() {
    let (root, journals, destination) = scene("claim-hiding-examiner");
    drop(publish(journals.as_path(), destination.as_path(), "dead"));
    let path: PathBuf = journals.join("topic.claim");
    // SAFETY: `prctl` with `PR_SET_DUMPABLE` only sets a flag of the calling process.
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) }, 0);
    let hidden = examine_claim(path.as_path());
    // SAFETY: as above; the flag is restored before any assertion can end the test.
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 1, 0, 0, 0) }, 0);
    match hidden.expect("examined") {
        Examination::Held { reason, .. } => assert!(reason.contains("does not adopt"), "{reason}"),
        other => panic!("expected a held claim, got {other:?}"),
    }
    assert!(matches!(examine_claim(path.as_path()).expect("visible again"), Examination::Abandoned(_)));
    remove(root.as_path());
}

/// A claim that does not parse, or names something that is not a stage container, is refused and kept for inspection.
#[test]
fn an_unreadable_claim_is_refused_and_kept() {
    let (root, journals, destination) = scene("claim-corrupt");
    let relative: String = format!(
        "{{\"destinationRoot\":\"/d\",\"stageContainer\":\"{STAGE_PREFIX}relative\",\"version\":1}}\n"
    );
    let outside: String = format!(
        "{{\"destinationRoot\":\"/d\",\"stageContainer\":{},\"version\":1}}\n",
        quote(destination.to_string_lossy().as_ref())
    );
    for (name, text) in [
        ("empty", String::new()),
        ("prose", String::from("not json\n")),
        ("relative", relative),
        ("outside", outside),
    ] {
        let path: PathBuf = journals.join(format!("{name}.claim"));
        crate::private_storage::write_private_file(path.as_path(), text.as_bytes()).expect("claim");
        let failure: WorktreeCopyError = examine_claim(path.as_path()).expect_err(name);
        assert!(failure.0.contains("does not name a stage container"), "{name}: {}", failure.0);
        assert!(path.exists(), "{name} stays for inspection");
    }
    remove(root.as_path());
}

/// A claim file others can read could name any directory for removal, so it is refused before it is believed.
#[test]
fn a_claim_others_can_read_is_refused() {
    let (root, journals, destination) = scene("claim-shared");
    let claim: Claim = publish(journals.as_path(), destination.as_path(), "shared");
    std::fs::set_permissions(claim.path(), std::fs::Permissions::from_mode(0o644)).expect("shared");
    let failure: WorktreeCopyError = examine_claim(claim.path()).expect_err("refused");
    assert!(failure.0.contains("claim file is unsafe"), "{}", failure.0);
    remove(root.as_path());
}

/// Releasing removes the claim only while its name still refers to the released file.
#[test]
fn releasing_a_claim_leaves_a_replacement_alone() {
    let (root, journals, destination) = scene("claim-release");
    let claim: Claim = publish(journals.as_path(), destination.as_path(), "released");
    let path: PathBuf = claim.path().to_path_buf();
    claim.release().expect("released");
    assert!(!path.exists());
    let replaced: Claim = publish(journals.as_path(), destination.as_path(), "replaced");
    let stale: Claim = Claim {
        path: path.clone(),
        file: std::fs::File::open(journals.as_path()).expect("some other file"),
        content: replaced.content().clone(),
    };
    stale.release().expect("released");
    assert!(path.exists(), "a name that refers to another file stays");
    remove(root.as_path());
}

/// Claiming publishes when the name is free, yields to a live claim without waiting, and settles an abandoned one first.
#[test]
fn claiming_yields_to_a_live_claim_and_settles_an_abandoned_one() {
    let (root, journals, destination) = scene("claim-destination");
    let container: PathBuf = container_for(destination.as_path(), "fresh");
    let mut settled: Vec<PathBuf> = Vec::new();
    let live: Claim = publish(journals.as_path(), destination.as_path(), "live");
    match claim_destination(journals.as_path(), "topic", destination.as_path(), container.as_path(), &mut |claim| {
        settled.push(claim.path().to_path_buf());
        return claim.release();
    })
    .expect("claiming")
    {
        Claiming::Held(reason) => assert!(reason.contains("descriptor"), "{reason}"),
        Claiming::Claimed(_) => panic!("a live claim was taken"),
    }
    assert!(settled.is_empty(), "a live claim is never settled");
    drop(live);
    match claim_destination(journals.as_path(), "topic", destination.as_path(), container.as_path(), &mut |claim| {
        settled.push(claim.path().to_path_buf());
        return claim.release();
    })
    .expect("claiming")
    {
        Claiming::Claimed(claim) => assert_eq!(claim.container(), container),
        Claiming::Held(reason) => panic!("an abandoned claim was not settled: {reason}"),
    }
    assert_eq!(settled, vec![journals.join("topic.claim")]);
    remove(root.as_path());
}

/// A settlement that leaves the abandoned claim in place cannot loop: claiming gives up after a few rounds.
#[test]
fn claiming_gives_up_when_settling_changes_nothing() {
    let (root, journals, destination) = scene("claim-stuck");
    drop(publish(journals.as_path(), destination.as_path(), "stuck"));
    let mut rounds: usize = 0;
    let claiming = claim_destination(
        journals.as_path(),
        "topic",
        destination.as_path(),
        container_for(destination.as_path(), "fresh").as_path(),
        &mut |_claim| {
            rounds += 1;
            return Ok(());
        },
    )
    .expect("claiming");
    assert!(matches!(claiming, Claiming::Held(_)));
    assert_eq!(rounds, CLAIM_ROUNDS);
    remove(root.as_path());
}

/// A failed settlement is the claiming's failure, and the claim stays for the next pass.
#[test]
fn a_failed_settlement_fails_the_claiming_and_keeps_the_claim() {
    let (root, journals, destination) = scene("claim-settle-fails");
    drop(publish(journals.as_path(), destination.as_path(), "dead"));
    let failure: WorktreeCopyError = claim_destination(
        journals.as_path(),
        "topic",
        destination.as_path(),
        container_for(destination.as_path(), "fresh").as_path(),
        &mut |_claim| return Err(WorktreeCopyError(String::from("cli-git: settlement failed."))),
    )
    .expect_err("settlement failed");
    assert_eq!(failure.0, "cli-git: settlement failed.");
    assert!(journals.join("topic.claim").exists());
    remove(root.as_path());
}

/// Only a plain component can name a claim, and a name that would not fit once its temporary is added is refused.
#[test]
fn only_a_plain_short_key_names_a_claim() {
    let root: &Path = Path::new("/journals");
    for key in ["", ".", "..", "a/b", "nul\0"] {
        assert!(claim_path(root, key).is_err(), "{key:?}");
    }
    assert!(claim_path(root, "x".repeat(LONGEST_KEY + 1).as_str()).is_err());
    assert_eq!(
        claim_path(root, "x".repeat(LONGEST_KEY).as_str()).expect("longest").file_name().expect("name").len(),
        LONGEST_KEY + CLAIM_SUFFIX.len()
    );
    assert_eq!(claim_path(root, "topic").expect("plain"), Path::new("/journals/topic.claim"));
}

/// Only claim files and claim temporaries count as copies in flight; journals and other files do not.
#[test]
fn only_claims_and_claim_temporaries_make_a_repository_pending() {
    let (root, journals, destination) = scene("claim-listing");
    assert!(!claims_exist(journals.as_path()).expect("empty"));
    assert!(!claims_exist(journal_root(root.join("absent").as_path()).as_path()).expect("no root"));
    std::fs::write(journals.join("id.json"), b"{}").expect("journal-shaped file");
    std::fs::write(journals.join("id.json.abc.tmp"), b"a journal write").expect("journal temporary");
    std::fs::write(journals.join("notes.txt"), b"x").expect("note");
    assert!(!claims_exist(journals.as_path()).expect("others"));
    assert!(list_claims(journals.as_path()).expect("listing").is_empty());
    let claim: Claim = publish(journals.as_path(), destination.as_path(), "listed");
    assert!(claims_exist(journals.as_path()).expect("claimed"));
    assert_eq!(list_claims(journals.as_path()).expect("listing"), vec![claim.path().to_path_buf()]);
    claim.release().expect("released");
    assert!(!claims_exist(journals.as_path()).expect("released"));
    std::fs::write(journals.join("topic.claim.abc.tmp"), b"cut off").expect("claim temporary");
    assert!(claims_exist(journals.as_path()).expect("temporary"));
    assert!(list_claims(journals.as_path()).expect("listing").is_empty(), "a temporary is no claim");
    remove(root.as_path());
}

/// A temporary whose publisher died is removed; one a live publisher still holds open stays, and journal temporaries
///  belong to journal recovery.
#[test]
fn an_abandoned_temporary_is_removed_and_a_held_one_stays() {
    let (root, journals, _destination) = scene("claim-temporaries");
    let abandoned: PathBuf = journals.join("topic.claim.dead.tmp");
    let held: PathBuf = journals.join("other.claim.live.tmp");
    let journal_temporary: PathBuf = journals.join("abc.json.def.tmp");
    std::fs::write(abandoned.as_path(), b"cut off").expect("abandoned");
    std::fs::write(held.as_path(), b"being published").expect("held");
    std::fs::write(journal_temporary.as_path(), b"a journal write").expect("journal temporary");
    let publisher = std::fs::File::open(held.as_path()).expect("the publisher holds its temporary");
    remove_abandoned_temporaries(journals.as_path()).expect("cleaned");
    assert!(!abandoned.exists());
    assert!(held.exists(), "a live publisher keeps its temporary");
    assert!(journal_temporary.exists());
    drop(publisher);
    remove_abandoned_temporaries(journals.as_path()).expect("cleaned again");
    assert!(!held.exists());
    remove(root.as_path());
}

/// A journal root that does not exist has no claims, and one that cannot be listed is a failure and not an empty root.
#[test]
fn an_unlistable_journal_root_is_a_failure_and_not_an_empty_one() {
    let root: PathBuf = fixture("claim-unlistable");
    assert!(list_claims(root.join("absent").as_path()).expect("absent").is_empty());
    let file: PathBuf = root.join("a-file");
    std::fs::write(file.as_path(), b"a file where the directory belongs").expect("file");
    list_claims(file.as_path()).expect_err("a file cannot be listed");
    claims_exist(file.as_path()).expect_err("a file cannot be listed");
    remove_abandoned_temporaries(file.as_path()).expect_err("a file cannot be listed");
    remove(root.as_path());
}
