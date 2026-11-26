//! Controls for the report of a damaged link:
//! the states,
//! the words a person reads,
//! and what `git worktree repair` really does on each side of Git having
//! pruned the registration.

use super::*;
use crate::test_support::{
    fixture,
    git,
    git_output,
    remove,
};
use crate::worktree_clone_prune_pointer::link_facts::RecordedLink;
use crate::worktree_clone_prune_pointer::pointer_restore::restore_the_slot_pointer;
use crate::worktree_clone_prune_pointer::pointer_state::{
    PointerSite,
    read_pointer_state_at,
};
use std::path::PathBuf;

/// What:
///  The facts of one standing registration with a pointer an account deleted,
///  and of the same registration after a prune.
/// Why:
///  The report exists to tell a repair that rebuilds a link from one that
///  cannot bring a registration back,
///  so the control walks both sides with real Git.
struct Link {
    /// The fixture root.
    root: PathBuf,
    /// The repository the registration belongs to.
    repository: PathBuf,
    /// The slot's path.
    slot: PathBuf,
    /// The registration's path.
    registration: PathBuf,
    /// The bytes Git wrote into the slot.
    pointer: Vec<u8>,
}

/// One real registration of one slot.
fn link(name: &str) -> Link {
    let root: PathBuf = fixture(name);
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir(repository.as_path()).expect("repository");
    git(repository.as_path(), &["init", "--quiet"]);
    git(
        repository.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message", "base"],
    );
    let slot: PathBuf = root.join("one");
    git(
        repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            slot.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let registration: PathBuf = repository.join(".git").join("worktrees").join("one");
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("pointer reads");
    return Link {
        root,
        repository,
        slot,
        registration,
        pointer,
    };
}

/// The facts of one link with the bytes the record keeps.
fn facts_of(
    link: &Link,
    backup: Option<&[u8]>,
) -> crate::worktree_clone_prune_pointer::link_facts::LinkFacts {
    let recorded: RecordedLink<'_> = RecordedLink {
        slot_path: link.slot.clone(),
        registration: Some(link.registration.clone()),
        pointer: backup,
        clone_dataset: "tank/clones/one",
    };
    return crate::worktree_clone_prune_pointer::link_facts::link_facts_of(
        "one",
        &recorded,
        &crate::worktree_clone_slot_state::slot_state_mounts::CloneMount::Nowhere,
    );
}

/// What:
///  A pointer an account deleted with the registration standing is the class a
///  repair puts right,
///  and real Git rebuilds the link and exits nonzero while doing it.
/// Why:
///  A person who read only Git's exit would think the repair failed,
///  and the report exists to say otherwise.
#[test]
fn a_standing_registration_with_a_backup_is_restorable() {
    let link: Link = link("repair-report-damaged");
    std::fs::remove_file(link.slot.join(".git")).expect("pointer removed");
    let facts = facts_of(&link, Some(link.pointer.as_slice()));
    assert_eq!(class_of(&facts), LinkRepair::Damaged);
    let report: LinkRepairReport = link_report_of(&facts);
    let printed: String = report.lines().join("\n");
    assert!(
        printed.contains("link: damaged, and the registration stands"),
        "{printed}"
    );
    assert!(
        printed.contains("restorable: the pointer file"),
        "{printed}"
    );
    assert!(
        printed.contains("exits 1 while rebuilding a missing pointer"),
        "{printed}"
    );
    assert!(
        printed.contains("next safe command: `git worktree repair"),
        "{printed}"
    );
    // The restore writes the record's bytes, and Git's own repair then finds a link it can read.
    assert!(matches!(
        restore_the_slot_pointer(
            link.slot.as_path(),
            &PointerSite::Underlay,
            Some(link.pointer.as_slice())
        ),
        crate::worktree_clone_prune_pointer::pointer_restore::RestoreOutcome::Written { .. }
    ));
    let repaired: std::process::Output = git(
        link.repository.as_path(),
        &["worktree", "repair", link.slot.to_str().expect("utf-8")],
    );
    assert!(
        repaired.status.success(),
        "the restored link repairs cleanly: {}",
        String::from_utf8_lossy(repaired.stderr.as_slice())
    );
    // Git's own repair alone rebuilds the same bytes, and its first run exits 1 while doing it.
    std::fs::remove_file(link.slot.join(".git")).expect("pointer removed again");
    let first: std::process::Output = git_output(
        link.repository.as_path(),
        &["worktree", "repair", link.slot.to_str().expect("utf-8")],
    );
    assert!(
        !first.status.success(),
        "Git 2.56.1 exits 1 while rebuilding a missing pointer"
    );
    assert_eq!(
        std::fs::read(link.slot.join(".git")).expect("pointer reads"),
        link.pointer
    );
    let second: std::process::Output = git(
        link.repository.as_path(),
        &["worktree", "repair", link.slot.to_str().expect("utf-8")],
    );
    assert!(second.status.success(), "the second run exits 0");
    remove(link.root.as_path());
}

/// What:
///  A registration Git already pruned is the class nothing rebuilds,
///  and a pointer restored from the record points at nothing.
/// Why:
///  The design says repair does not reconstruct HEAD,
///  the index or the lock once the administrative entry is gone,
///  and a person must not be told a rebuilt file brings them back.
#[test]
fn a_pruned_registration_is_what_a_repair_cannot_restore() {
    let link: Link = link("repair-report-pruned");
    std::fs::remove_file(link.slot.join(".git")).expect("pointer removed");
    let pruned: std::process::Output = git(
        link.repository.as_path(),
        &["worktree", "prune", "--verbose"],
    );
    assert!(
        pruned.status.success(),
        "prune ran: {}",
        String::from_utf8_lossy(pruned.stderr.as_slice())
    );
    assert!(!link.registration.exists(), "the registration is gone");
    let facts = facts_of(&link, Some(link.pointer.as_slice()));
    assert_eq!(
        class_of(&facts),
        LinkRepair::Pruned {
            backup: true
        }
    );
    let printed: String = link_report_of(&facts).lines().join("\n");
    assert!(printed.contains("lost: the registration"), "{printed}");
    assert!(
        printed.contains("cannot rebuild a registration"),
        "{printed}"
    );
    assert!(
        printed.contains("decide explicitly whether to re-register"),
        "{printed}"
    );
    // A pointer written back from the record points at a registration that is gone, and a repair cannot rebuild one.
    std::fs::write(link.slot.join(".git"), link.pointer.as_slice()).expect("pointer written back");
    let repair: std::process::Output = git_output(
        link.repository.as_path(),
        &["worktree", "repair", link.slot.to_str().expect("utf-8")],
    );
    assert!(
        !repair.status.success(),
        "Git refuses a link whose registration is gone"
    );
    let listed: std::process::Output = git(link.repository.as_path(), &["worktree", "list"]);
    assert!(
        !String::from_utf8_lossy(listed.stdout.as_slice()).contains("one"),
        "the worktree is not registered again"
    );
    remove(link.root.as_path());
}

/// What:
///  A record with no pointer bytes and a standing registration makes no claim
///  about the pointer,
///  while a pruned registration without bytes still names what is lost.
/// Why:
///  The acceptance test asks for both sides with and without a backup,
///  and an uncompared pointer is not a damaged one.
#[test]
fn without_a_backup_the_report_names_what_it_cannot_compare() {
    let link: Link = link("repair-report-no-backup");
    let standing = facts_of(&link, None);
    assert_eq!(class_of(&standing), LinkRepair::Unchecked);
    let printed: String = link_report_of(&standing).lines().join("\n");
    assert!(
        printed.contains("the record keeps no pointer bytes"),
        "{printed}"
    );
    assert!(
        printed.contains("restorable: a missing or changed pointer"),
        "{printed}"
    );
    assert!(
        printed.contains("unknown: whether the pointer needs that repair"),
        "{printed}"
    );
    std::fs::remove_file(link.slot.join(".git")).expect("pointer removed");
    let pruned: std::process::Output = git(link.repository.as_path(), &["worktree", "prune"]);
    assert!(pruned.status.success());
    let absent = facts_of(&link, None);
    assert_eq!(
        class_of(&absent),
        LinkRepair::Pruned {
            backup: false
        }
    );
    let pruned_text: String = link_report_of(&absent).lines().join("\n");
    assert!(
        pruned_text.contains("lost: the pointer bytes as well"),
        "{pruned_text}"
    );
    remove(link.root.as_path());
}

/// What:
///  A registration whose `gitdir` names another path and a record that names
///  no registration are both unprovable,
///  and the report says so rather than repairing the wrong link.
/// Why:
///  A `gitdir` another worktree left would make a repair rebuild a pointer
///  into someone else's registration.
#[test]
fn a_registration_that_names_another_path_is_unprovable() {
    let link: Link = link("repair-report-another-path");
    std::fs::write(link.registration.join("gitdir"), b"/somewhere/else/.git\n")
        .expect("gitdir replaced");
    let facts = facts_of(&link, Some(link.pointer.as_slice()));
    assert!(
        matches!(class_of(&facts), LinkRepair::Unprovable { .. }),
        "{:?}",
        class_of(&facts)
    );
    assert!(
        link_report_of(&facts)
            .lines()
            .join("\n")
            .contains("names another path")
    );
    // The same slot's pointer is still the record's, which the report says nothing about: the link is not this slot's.
    assert_eq!(
        read_pointer_state_at(link.slot.as_path(), Some(link.pointer.as_slice())),
        crate::worktree_clone_prune_pointer::pointer_state::PointerState::Intact
    );
    let unnamed = crate::worktree_clone_prune_pointer::link_facts::LinkFacts {
        slot: String::from("one"),
        slot_path: link.slot.clone(),
        clone_dataset: String::from("tank/clones/one"),
        registration: None,
        registration_stands: false,
        registration_names_the_slot: None,
        pointer: crate::worktree_clone_prune_pointer::pointer_state::PointerState::Intact,
        backup: true,
        site: PointerSite::Underlay,
    };
    assert!(matches!(class_of(&unnamed), LinkRepair::Unprovable { .. }));
    remove(link.root.as_path());
}
