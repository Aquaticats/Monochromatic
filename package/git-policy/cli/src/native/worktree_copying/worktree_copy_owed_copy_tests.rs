//! What:
//!  The copy an interrupted `git worktree add` still owes:
//!  the next command makes it from the source the record names,
//!  never over something the worktree gained,
//!  and a copy that cannot be made is named on every later command with what ended it.
//! Why:
//!  A worktree that stands without its source's ignored files looks ordinary and holds less than the source ignores;
//!  a person who must remove it and add it again loses the work they already did there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { repo, destination } = await interruptedAdd('name'); told(common); expect(read(destination, 'cache/data')).toEqual('payload');
//! ```

use super::*;
use crate::test_support::{dead_copy_owner, fixture, git, git_context, remove, repository};
use crate::worktree_add_binding::{Binding, OwedCopy, OwedSelection};
use crate::worktree_copy_recovery::recover_pending_worktree_copies;
use crate::worktree_copy_stage_claim::stage_claims_exist;
use crate::worktree_copy_stage_claim_reaping::read_claim;
use crate::worktree_copy_unfilled_record::unfilled_records_exist;

/// What:
///  A repository whose `cache/` is ignored,
/// with a worktree Git registered with a whole checkout and no ignored state,
/// and the claim an add killed before its copy ran leaves.
/// Why:
///   That is exactly the state a kill between real Git's checkout and the copy's own records leaves,
/// so a control
///  starts from the state a kill produces instead of racing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(root, 'repo'); write(repo, 'cache/data', 'payload'); commit(repo); git(repo, ['worktree', 'add', '-b', 'topic', '../topic']); claimStageForAddAsOwing({ common, destination, owner: deadCopyOwner(), owed }).forget();
/// ```
fn interrupted_add(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(repo.join("cache")).expect("cache");
    std::fs::write(repo.join("cache/data"), b"payload").expect("data");
    git(repo.as_path(), &["add", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=ignored"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("canonical destination");
    interrupted_add_owing(repo.as_path(), destination.as_path());
    return (root, repo, destination);
}

/// What:
///  The claim an add killed just after its checkout leaves for one destination.
/// Why:
///   The destination and the ignored-state copy the add owed are the whole record,
/// and a control writes it rather than racing a kill.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owed = { source: canonical(repo), selection: 'main-checkout' }; const claim = await claimStageForAddAsOwing({ common, destination, owner: deadCopyOwner(), owed }); claim.forget();
/// ```
fn interrupted_add_owing(repo: &Path, destination: &Path) {
    let source: PathBuf = std::fs::canonicalize(repo).expect("canonical source");
    let owed: OwedCopy = OwedCopy { source, selection: OwedSelection::MainCheckout };
    let claim: StageClaim = crate::worktree_copy_stage_claim::claim_stage_for_add_as_owing(
        repo.join(".git").as_path(),
        destination,
        &dead_copy_owner(),
        Some(&owed),
    )
    .expect("claim");
    // The process dies here: neither record is released nor dropped cleanly.
    std::mem::forget(claim);
}

/// The one claim an interrupted add left under a journal root.
fn claim_path_of(root: &Path) -> PathBuf {
    let mut found: Vec<PathBuf> = std::fs::read_dir(root).expect("records").flatten().map(|entry| { return entry.path(); }).filter(|path| { return path.extension().is_some_and(|extension| { return extension == "stage"; }); }).collect();
    found.sort();
    return found.first().expect("a claim").clone();
}

/// The notices one recovery pass printed,
/// followed by what the next pass says.
fn told(common: &Path) -> String {
    let mut said: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(common, &git_context(), &mut said).expect("the recovery pass");
    return String::from_utf8_lossy(said.as_slice()).into_owned();
}

/// An interrupted add's worktree is filled by the next command,
/// from the source its record names,
/// and its records go.
#[test]
fn the_next_command_makes_the_copy_an_interrupted_add_owed() {
    let (root, repo, destination) = interrupted_add("owed-copy-made");
    assert!(!destination.join("cache/data").exists(), "Git's own checkout writes no ignored file");
    let said: String = told(repo.join(".git").as_path());
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the ignored file arrived"), b"payload");
    let shown: String = crate::json_record::quote(destination.to_string_lossy().as_ref());
    assert!(said.contains("finished the ignored-state copy an interrupted `git worktree add` owed") && said.contains(shown.as_str()), "the line names the worktree it filled: {said}");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the claim and its records go");
    assert!(!unfilled_records_exist(repo.join(".git").as_path()).expect("records"), "and nothing promises a missing file");
    assert_eq!(told(repo.join(".git").as_path()), "", "the next command has nothing left to say");
    remove(root.as_path());
}

/// What an ignored entry the worktree already holds stops:
/// the copy never replaces it,
/// the person's bytes stay,
/// and every later command names the entry that differs.
#[test]
fn a_copy_that_would_replace_what_the_worktree_gained_is_named_with_the_entry_instead() {
    let (root, repo, destination) = interrupted_add("owed-copy-differs");
    std::fs::create_dir_all(destination.join("cache")).expect("cache");
    std::fs::write(destination.join("cache/data"), b"work started here").expect("their work");
    let said: String = told(repo.join(".git").as_path());
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the person's file"), b"work started here", "nothing the worktree gained is replaced");
    assert!(said.contains("could not be made") && said.contains("git worktree remove --force"), "the line names the worktree and what settles it: {said}");
    assert!(unfilled_records_exist(repo.join(".git").as_path()).expect("records"), "the record that names the unfilled worktree stays");
    // The same words come again, and no command stages the source again trying what just failed.
    let again: String = told(repo.join(".git").as_path());
    assert!(again.contains("holds less than") && again.contains("differing destination entry"), "every later command says which entry differs: {again}");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the claim is settled, so nothing runs the copy again");
    remove(root.as_path());
}

/// A record whose source is not a worktree of this repository any more ends the copy with that reason,
/// so no command stages from a path that is no source.
#[test]
fn a_source_that_is_no_worktree_ends_the_copy_with_the_reason() {
    let root: PathBuf = fixture("owed-copy-source-gone");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(repo.join("cache")).expect("cache");
    std::fs::write(repo.join("cache/data"), b"payload").expect("data");
    git(repo.as_path(), &["add", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=ignored"]);
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("canonical destination");
    // The worktree the state would come from was removed after the add was killed.
    let gone: PathBuf = root.join("gone");
    std::fs::create_dir_all(gone.as_path()).expect("a path that is no worktree");
    let owed: OwedCopy = OwedCopy { source: std::fs::canonicalize(gone.as_path()).expect("canonical"), selection: OwedSelection::Ignored };
    let claim: StageClaim = crate::worktree_copy_stage_claim::claim_stage_for_add_as_owing(
        repo.join(".git").as_path(),
        destination.as_path(),
        &dead_copy_owner(),
        Some(&owed),
    )
    .expect("claim");
    std::mem::forget(claim);
    let said: String = told(repo.join(".git").as_path());
    assert!(said.contains("no longer a worktree of this repository"), "the line says why nothing was copied: {said}");
    assert!(!destination.join("cache/data").exists(), "nothing was staged from a path that is no worktree");
    assert!(unfilled_records_exist(repo.join(".git").as_path()).expect("records"), "the worktree stays named");
    remove(root.as_path());
}

/// The stage a kill left while it was being filled is removed when it is provably this copy's own,
/// and the copy is made;
/// a container that records no binding is left where it is,
/// so no directory another process put at that name is ever removed.
#[test]
fn a_stage_this_copy_left_is_removed_and_a_container_nothing_proves_stays() {
    let (root, repo, destination) = interrupted_add("owed-copy-stage");
    let common: PathBuf = repo.join(".git");
    let records: PathBuf = journal_root(common.as_path());
    let (container, _owner) = read_claim(claim_path_of(records.as_path()).as_path()).expect("claim").expect("named");
    std::fs::create_dir(container.as_path()).expect("the stage the kill left");
    std::fs::set_permissions(container.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    let kept: String = told(common.as_path());
    assert!(container.exists(), "a container nothing proves is the copy's own stays, and the settlement names it: {kept}");
    assert!(!destination.join("cache/data").exists(), "and no copy is written from a stage it could not clear");
    // The binding a staging copy records first proves the container is its own.
    let binding: Binding = crate::worktree_add_binding::bind_registration(common.as_path(), destination.as_path()).expect("bound").binding;
    crate::worktree_copy_stage_binding::record_stage_binding(container.as_path(), &binding).expect("recorded");
    let made: String = told(common.as_path());
    assert!(!container.exists(), "the copy's own stage goes before the copy is made again: {made}");
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the ignored file arrived"), b"payload");
    remove(root.as_path());
}
