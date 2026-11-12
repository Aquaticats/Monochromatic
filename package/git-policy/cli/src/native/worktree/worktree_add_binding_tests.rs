//! Controls for the binding of an add:
//! the pair the repository confirms is bound,
//! every pair it does not confirm is refused,
//!  and the binding is verified again before anything is written:
//! a moved,
//! replaced or edited object stops the step.

use super::*;
use crate::private_storage::take_file_syncs;
use crate::test_support::{fixture, git, remove, repository};
use crate::worktree_ladder_test_support::{commit_all, tracked};

/// An ordinary repository with a worktree registered without a checkout beside it:
/// the repository,
/// the destination,
/// and the
///  administrative directory.
fn registered(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "file.txt", b"content\n");
    commit_all(repo.as_path(), "first");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    return (repo, destination, admin);
}

/// A pair the repository does not confirm is not bound:
/// a pointer outside this repository's worktrees,
/// one that names another
///  worktree's registration (an edited pointer),
/// a registration that names another place,
/// and a destination reached through a
///  link.
#[test]
fn a_pair_the_repository_does_not_confirm_is_not_bound() {
    let root: PathBuf = fixture("rollback-unconfirmed");
    let (repo, destination, admin) = registered(root.as_path());
    let common: PathBuf = repo.join(".git");
    // A pointer at a directory outside the repository's worktrees.
    let foreign: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("foreign");
    std::fs::create_dir_all(foreign.join("admin")).expect("foreign admin");
    std::fs::write(foreign.join("admin/gitdir"), format!("{}\n", foreign.join("look/.git").display())).expect("gitdir");
    std::fs::create_dir_all(foreign.join("look")).expect("look");
    std::fs::write(foreign.join("look/.git"), format!("gitdir: {}\n", foreign.join("admin").display())).expect("pointer");
    assert!(bind_registration(common.as_path(), foreign.join("look").as_path()).expect_err("not registered").contains("not registered"));
    // A pointer edited to name the source's own git directory.
    let edited: PathBuf = foreign.join("edited");
    std::fs::create_dir_all(edited.as_path()).expect("edited");
    std::fs::write(edited.join(".git"), format!("gitdir: {}\n", common.display())).expect("pointer at the main git directory");
    assert!(bind_registration(common.as_path(), edited.as_path()).is_err(), "the main git directory is not a registration");
    // A pointer edited to name another worktree's registration: its back-link names another place.
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "other", "../other"]);
    std::fs::write(destination.join(".git"), format!("gitdir: {}\n", repo.join(".git/worktrees/other").display())).expect("edited pointer");
    assert!(bind_registration(common.as_path(), destination.as_path()).expect_err("not named back").contains("does not name"));
    std::fs::write(destination.join(".git"), format!("gitdir: {}\n", admin.display())).expect("restored");
    // A destination named through a link.
    let link: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("link");
    std::os::unix::fs::symlink(destination.as_path(), link.as_path()).expect("link");
    assert!(bind_registration(common.as_path(), link.as_path()).is_err(), "a link is not the directory");
    // Positive control: the real pair binds.
    assert!(bind_registration(common.as_path(), destination.as_path()).is_ok());
    remove(root.as_path());
}

/// What:
///  The proof that names the command a registration belongs to is published whole and flushes nothing,
/// and the command that reads it back learns the same token.
/// Why:
///  The proof outlives the command that writes it and only answers whose a registration is;
/// a flush of it is a commit of the whole btrfs filesystem on the add's waiting path,
/// where native Git flushes nothing for the worktree add it serves.
/// The control that the recorders see a flush is a private file's own durable write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claimRegistration(); expect(fileSyncs()).toEqual([]); expect(ownerFile()).toEqual('mine');
/// ```
#[test]
fn the_proof_of_who_made_a_registration_flushes_nothing() {
    let root: PathBuf = fixture("binding-owner-unsynced");
    let (repo, destination, admin) = registered(root.as_path());
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    let owner_file: PathBuf = admin.join(REGISTRATION_OWNER_NAME);
    take_file_syncs();
    assert_eq!(claim_registration(&bound.admin, "mine").expect("claimed"), RegistrationOwner::Mine);
    assert_eq!(take_file_syncs(), Vec::<PathBuf>::new(), "the proof of who made the registration flushes no file");
    assert_eq!(std::fs::read(owner_file.as_path()).expect("owner"), b"mine", "the proof is published whole with the token in it");
    assert_eq!(
        claim_registration(&bound.admin, "another").expect("read back"),
        RegistrationOwner::Another(String::from("mine")),
        "a second command reads the token the first one wrote"
    );
    let control: PathBuf = admin.join("durable-control");
    crate::private_storage::write_private_file(control.as_path(), b"control").expect("control");
    assert!(!take_file_syncs().is_empty(), "a durable private file flushes (the control that the recorder sees it)");
    remove(root.as_path());
}

/// The binding of a fresh worktree holds,
/// and holds again after a failed look:
/// the verification changes nothing.
#[test]
fn a_binding_holds_until_an_object_it_names_changes() {
    let root: PathBuf = fixture("binding-holds");
    let (repo, destination, admin) = registered(root.as_path());
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    assert_eq!(verify_association(&bound), Ok(()));
    assert_eq!(verify_association(&bound), Ok(()));
    assert!(destination.join(".git").exists() && admin.exists());
    remove(root.as_path());
}

/// A destination that was moved away,
/// with another worktree made at its path,
/// is not the bound one:
/// the verification names
///  it,
/// and the replacement is untouched.
#[test]
fn a_destination_moved_and_replaced_no_longer_verifies() {
    let root: PathBuf = fixture("binding-replaced");
    let (repo, destination, _admin) = registered(root.as_path());
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    git(repo.as_path(), &["worktree", "move", "../topic", "../moved"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "again", "../topic"]);
    std::fs::write(destination.join("mine.txt"), b"work in the replacement").expect("their work");
    let reason: String = verify_association(&bound).expect_err("replaced");
    assert!(reason.contains("no longer the directory Git made"), "{reason}");
    assert_eq!(std::fs::read(destination.join("mine.txt")).expect("kept"), b"work in the replacement");
    remove(root.as_path());
}

/// A worktree moved to another path changes its registration:
/// the verification refuses,
/// and nothing is changed.
#[test]
fn a_moved_worktree_no_longer_verifies() {
    let root: PathBuf = fixture("binding-moved");
    let (repo, destination, admin) = registered(root.as_path());
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    git(repo.as_path(), &["worktree", "move", "../topic", "../moved"]);
    assert!(verify_association(&bound).is_err());
    assert!(admin.exists(), "the registration of the moved worktree is untouched");
    remove(root.as_path());
}

/// A registration replaced by another directory,
/// a pointer edited after the binding and a registration re-pointed each stop
///  the verification with a reason of their own.
#[test]
fn a_replaced_registration_an_edited_pointer_and_a_repointed_registration_stop_the_verification() {
    let root: PathBuf = fixture("binding-edited");
    let (repo, destination, admin) = registered(root.as_path());
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    let original: Vec<u8> = std::fs::read(destination.join(".git")).expect("pointer");
    std::fs::write(destination.join(".git"), b"gitdir: /elsewhere\n").expect("edited pointer");
    assert!(verify_association(&bound).expect_err("pointer").contains("no longer the pointer"));
    std::fs::write(destination.join(".git"), original).expect("restored");
    assert_eq!(verify_association(&bound), Ok(()), "positive control");
    std::fs::write(admin.join("gitdir"), format!("{}\n", root.join("elsewhere/.git").display())).expect("re-pointed");
    assert!(verify_association(&bound).expect_err("repointed").contains("no longer names"));
    std::fs::write(admin.join("gitdir"), format!("{}\n", destination.join(".git").display())).expect("restored");
    std::fs::rename(admin.as_path(), admin.with_file_name("topic.moved")).expect("registration moved");
    std::fs::create_dir(admin.as_path()).expect("another registration");
    assert!(verify_association(&bound).expect_err("replaced").contains("no longer the one Git made"));
    remove(root.as_path());
}

/// The record text of an add round-trips,
/// a name with a newline cannot forge a field,
/// and anything unreadable is nothing.
#[test]
fn a_record_round_trips_and_a_forged_one_is_nothing() {
    let planned: PathBuf = PathBuf::from("/base/line\nphase=registered");
    assert_eq!(decode_record(planned_record(planned.as_path()).as_str()), Some(Recorded::Planned(planned)));
    let binding: Binding = Binding {
        destination: PathBuf::from("/base/topic"),
        destination_id: ObjectId { device: 1, inode: 2, birth: Some((3, 4)) },
        admin: PathBuf::from("/repo/.git/worktrees/topic"),
        admin_id: ObjectId { device: 5, inode: 6, birth: None },
        pointer: b"gitdir: /repo/.git/worktrees/topic\n".to_vec(),
    };
    assert_eq!(decode_record(binding.encode().as_str()), Some(Recorded::Registered(binding.clone())));
    for broken in ["", "destination=zz\nphase=planned\n", "destination=2f\nphase=other\n", "phase=planned\n"] {
        assert_eq!(decode_record(broken), None, "{broken:?}");
    }
    let missing_field: String = binding.encode().replace("pointer=", "pointe=");
    assert_eq!(decode_record(missing_field.as_str()), None);
}

/// A pointer that reaches the registration through a link is refused at the binding,
/// with a message that names the pointer and
///  the remedy:
/// Git resolves the same text again at every use,
/// and the link can be retargeted to another registration.
#[test]
fn a_pointer_through_a_link_is_not_bound_and_the_message_names_the_remedy() {
    let root: PathBuf = fixture("binding-alias");
    let (repo, destination, admin) = registered(root.as_path());
    let alias: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("alias");
    std::os::unix::fs::symlink(repo.join(".git"), alias.as_path()).expect("alias of the git directory");
    std::fs::write(destination.join(".git"), format!("gitdir: {}/worktrees/topic\n", alias.display())).expect("pointer through the alias");
    let reason: String = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect_err("refused");
    assert!(reason.contains("without a symbolic link") && reason.contains("canonical path") && reason.contains(".git"), "{reason}");
    std::fs::write(destination.join(".git"), format!("gitdir: {}\n", admin.display())).expect("restored");
    assert!(bind_registration(repo.join(".git").as_path(), destination.as_path()).is_ok(), "positive control");
    remove(root.as_path());
}

/// A relative pointer resolves by its text against the destination and binds when no link is involved.
#[test]
fn a_relative_pointer_without_links_is_bound() {
    let root: PathBuf = fixture("binding-relative");
    let (repo, destination, _admin) = registered(root.as_path());
    std::fs::write(destination.join(".git"), "gitdir: ../repo/.git/worktrees/topic\n").expect("relative pointer");
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    assert_eq!(bound.binding.admin, repo.join(".git/worktrees/topic"));
    assert!(pointer_target(destination.as_path(), b"gitdir: ../../../../../../..\n").is_ok(), "the root is a place");
    assert!(pointer_target(destination.as_path(), b"gitdir: ").is_ok());
    let climbing: String = "../".repeat(40);
    assert!(pointer_target(destination.as_path(), format!("gitdir: {climbing}x\n").as_bytes()).is_err(), "a pointer cannot climb out of the root");
    remove(root.as_path());
}

/// The verification resolves the pointer again:
/// a binding whose pointer text goes through a link fails even when the bytes,
/// the
///  identities and the back-link all still hold.
#[test]
fn verification_resolves_the_pointer_again() {
    let root: PathBuf = fixture("binding-verify-alias");
    let (repo, destination, admin) = registered(root.as_path());
    let mut bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    let alias: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical").join("alias");
    std::os::unix::fs::symlink(repo.join(".git"), alias.as_path()).expect("alias");
    let text: Vec<u8> = format!("gitdir: {}/worktrees/topic\n", alias.display()).into_bytes();
    std::fs::write(destination.join(".git"), text.as_slice()).expect("pointer through the alias");
    bound.binding.pointer = text;
    let reason: String = verify_association(&bound).expect_err("an alias");
    assert!(reason.contains("without a symbolic link"), "{reason}");
    assert!(admin.exists());
    remove(root.as_path());
}

/// A worktree Git registered with relative paths (`--relative-paths`,
/// or `worktree.useRelativePaths`) is bound and stays
///  verified:
/// the registration's `gitdir` file names the destination relative to the registration,
/// and the pair is the same.
#[test]
fn a_registration_written_with_relative_paths_is_bound_and_verified() {
    let root: PathBuf = fixture("binding-relative-paths");
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    tracked(repo.as_path(), "file.txt", b"content\n");
    commit_all(repo.as_path(), "first");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--relative-paths", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let recorded: String = std::fs::read_to_string(repo.join(".git/worktrees/topic/gitdir")).expect("gitdir file");
    assert!(recorded.starts_with("../"), "Git recorded the destination relative to the registration: {recorded}");
    let bound: Bound = bind_registration(repo.join(".git").as_path(), destination.as_path()).expect("bound");
    assert_eq!(bound.binding.admin, repo.join(".git/worktrees/topic"));
    verify_association(&bound).expect("verified");
    // A relative back-link that names another place is still refused.
    std::fs::write(repo.join(".git/worktrees/topic/gitdir"), "../../../../elsewhere/.git\n").expect("edited back-link");
    assert!(bind_registration(repo.join(".git").as_path(), destination.as_path()).expect_err("another place").contains("does not name"));
    remove(root.as_path());
}
