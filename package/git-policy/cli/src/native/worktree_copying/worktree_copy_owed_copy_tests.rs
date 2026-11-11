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
//! const { repo, destination } = await interruptedAdd('name'); toldIn(common, destination); expect(read(destination, 'cache/data')).toEqual('payload');
//! ```

use super::*;
use crate::test_support::{dead_copy_owner, fixture, git, git_context, remove, repository};
use crate::worktree_add_binding::{Binding, OwedCopy, OwedSelection};
use crate::worktree_copy_recovery::{RecoveryDestinations, recover_pending_worktree_copies};
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
fn told_in(common: &Path, worktree_root: &Path) -> String {
    let mut said: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(common, RecoveryDestinations::OnlyThisWorktree(worktree_root), &git_context(), &mut said).expect("the recovery pass");
    return String::from_utf8_lossy(said.as_slice()).into_owned();
}

/// An interrupted add's worktree is filled by the next command,
/// from the source its record names,
/// and its records go.
#[test]
fn the_next_command_makes_the_copy_an_interrupted_add_owed() {
    let (root, repo, destination) = interrupted_add("owed-copy-made");
    assert!(!destination.join("cache/data").exists(), "Git's own checkout writes no ignored file");
    let said: String = told_in(repo.join(".git").as_path(), destination.as_path());
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the ignored file arrived"), b"payload");
    let shown: String = crate::json_record::quote(destination.to_string_lossy().as_ref());
    assert!(said.contains("finished the ignored-state copy an interrupted `git worktree add` owed") && said.contains(shown.as_str()), "the line names the worktree it filled: {said}");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the claim and its records go");
    assert!(!unfilled_records_exist(repo.join(".git").as_path()).expect("records"), "and nothing promises a missing file");
    assert_eq!(told_in(repo.join(".git").as_path(), destination.as_path()), "", "the next command has nothing left to say");
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
    let said: String = told_in(repo.join(".git").as_path(), destination.as_path());
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the person's file"), b"work started here", "nothing the worktree gained is replaced");
    assert!(said.contains("holds less than") && said.contains("differing destination entry") && said.contains("git worktree remove --force"), "the line names the worktree, the entry that differs and what settles it: {said}");
    assert!(unfilled_records_exist(repo.join(".git").as_path()).expect("records"), "the record that names the unfilled worktree stays");
    // The same words come again, and no command stages the source again trying what just failed.
    let again: String = told_in(repo.join(".git").as_path(), destination.as_path());
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
    let said: String = told_in(repo.join(".git").as_path(), destination.as_path());
    assert!(said.contains("no longer a worktree of this repository"), "the line says why nothing was copied: {said}");
    assert!(told_in(repo.join(".git").as_path(), destination.as_path()).contains("no longer a worktree of this repository"), "and every later command says the same");
    assert!(!destination.join("cache/data").exists(), "nothing was staged from a path that is no worktree");
    assert!(unfilled_records_exist(repo.join(".git").as_path()).expect("records"), "the worktree stays named");
    remove(root.as_path());
}

/// What:
///  The stage a kill left is removed and the copy is made when the wrapper can tell nothing in it can be lost,
/// and a container holding anything else is left where it is.
/// Why:
///   A copy creates its container and then records the registration it fills,
/// so a kill between the two leaves an empty container
///  nothing in it can be lost from;
/// an add killed before its copy ran leaves the same empty container at a name its own claim chose.
/// Neither a folder holding an
///  entry nothing proves the wrapper made,
/// nor one whose binding names another worktree,
/// is ever removed.
#[test]
fn a_stage_this_copy_left_is_removed_and_a_container_holding_anything_stays() {
    // The empty container a kill between the container's creation and its binding record leaves.
    let (root, repo, destination) = interrupted_add("owed-copy-stage");
    let common: PathBuf = repo.join(".git");
    let records: PathBuf = journal_root(common.as_path());
    let (container, _owner) = read_claim(claim_path_of(records.as_path()).as_path()).expect("claim").expect("named");
    std::fs::create_dir(container.as_path()).expect("the stage the kill left");
    std::fs::set_permissions(container.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    let emptied: String = told_in(common.as_path(), destination.as_path());
    assert!(!container.exists(), "a container holding nothing can lose nothing, so it goes: {emptied}");
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the ignored file arrived"), b"payload", "and the copy is made: {emptied}");
    remove(root.as_path());
    // A container holding an entry nothing proves the wrapper made is named and left, and no copy is written.
    let (holding_root, holding_repo, holding_destination) = interrupted_add("owed-copy-stage-holding");
    let holding_common: PathBuf = holding_repo.join(".git");
    let holding_records: PathBuf = journal_root(holding_common.as_path());
    let (holding_container, _holding_owner) = read_claim(claim_path_of(holding_records.as_path()).as_path()).expect("claim").expect("named");
    std::fs::create_dir_all(holding_container.join("payload")).expect("the stage the kill left");
    std::fs::set_permissions(holding_container.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    std::fs::write(holding_container.join("payload/mine.txt"), b"a person's file").expect("a file nothing proves");
    let kept: String = told_in(holding_common.as_path(), holding_destination.as_path());
    assert!(holding_container.exists(), "a container holding an entry nothing proves is the copy's own stays: {kept}");
    assert!(!holding_destination.join("cache/data").exists(), "and no copy is written from a stage it could not clear");
    assert_eq!(std::fs::read(holding_container.join("payload/mine.txt")).expect("the file stays"), b"a person's file");
    // The binding a staging copy records first proves the container is its own.
    let binding: Binding = crate::worktree_add_binding::bind_registration(holding_common.as_path(), holding_destination.as_path()).expect("bound").binding;
    crate::worktree_copy_stage_binding::record_stage_binding(holding_container.as_path(), &binding).expect("recorded");
    let made: String = told_in(holding_common.as_path(), holding_destination.as_path());
    assert!(!holding_container.exists(), "the copy's own stage goes before the copy is made again: {made}");
    assert_eq!(std::fs::read(holding_destination.join("cache/data")).expect("the ignored file arrived"), b"payload");
    remove(holding_root.as_path());
}

/// The stage of another worktree's copy is never removed by the binding rule:
/// a container whose binding names a destination that is not the one this copy fills stays.
#[test]
fn a_stage_bound_to_another_destination_is_not_this_copy_s_own() {
    let (root, repo, destination) = interrupted_add("owed-copy-stage-other-binding");
    let common: PathBuf = repo.join(".git");
    let records: PathBuf = journal_root(common.as_path());
    let (container, _owner) = read_claim(claim_path_of(records.as_path()).as_path()).expect("claim").expect("named");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "other", "../other"]);
    let other: PathBuf = std::fs::canonicalize(root.join("other")).expect("canonical other");
    std::fs::create_dir(container.as_path()).expect("the stage the kill left");
    std::fs::set_permissions(container.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("private stage");
    let other_binding: Binding = crate::worktree_add_binding::bind_registration(common.as_path(), other.as_path()).expect("bound").binding;
    crate::worktree_copy_stage_binding::record_stage_binding(container.as_path(), &other_binding).expect("recorded");
    let kept: String = told_in(common.as_path(), destination.as_path());
    assert!(container.exists(), "a container bound to another destination is not this copy's: {kept}");
    assert!(!destination.join("cache/data").exists(), "and nothing is written from it");
    remove(root.as_path());
}

/// The copy fills one worktree,
/// so only a command in that worktree makes it:
/// a command in another linked worktree of the same repository names the worktree that is short of
/// files and starts nothing.
/// Why:
///  Before this, any command of the repository finished the copy and waited for it:
/// 63.5 s for a `git merge` and 119.8 s for a `git push` in another worktree, silently,
/// after an add was killed while copying 15,151 ignored entries
/// (`doc/planning/cli-git-owed-copy-off-other-commands-path.md`).
#[test]
fn a_command_in_another_worktree_leaves_the_copy_to_the_worktree_it_fills() {
    let (root, repo, destination) = interrupted_add("owed-copy-other-worktree");
    let common: PathBuf = repo.join(".git");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "other", "../other"]);
    let other: PathBuf = std::fs::canonicalize(root.join("other")).expect("canonical other");
    let said: String = told_in(common.as_path(), other.as_path());
    assert!(!destination.join("cache/data").exists(), "a command in another worktree writes nothing into a worktree it does not run in");
    assert!(!said.contains("finishing the ignored-state copy"), "and it starts no copy: {said}");
    let shown: String = crate::json_record::quote(destination.to_string_lossy().as_ref());
    assert!(said.contains(shown.as_str()), "the line names the worktree that is short of files: {said}");
    assert!(
        said.contains("the first command run in that worktree makes it, and this command does not wait for it"),
        "the line says who makes it and that nothing waits here: {said}"
    );
    assert!(stage_claims_exist(common.as_path()).expect("claims"), "the claim stays for that worktree's own command");
    assert!(!unfilled_records_exist(common.as_path()).expect("records"), "nothing was given up on");
    // The worktree's own first command makes it, and the line names what it is about to do before it starts.
    let made: String = told_in(common.as_path(), destination.as_path());
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the ignored file arrived"), b"payload");
    assert!(made.contains("finished the ignored-state copy"), "{made}");
    remove(root.as_path());
}

/// The command that finishes a copy names it before the first entry is read,
/// so the wait it takes is never silent,
/// and says what ends it.
#[test]
fn the_worktrees_own_command_says_what_it_waits_for_before_it_starts() {
    let (root, repo, destination) = interrupted_add("owed-copy-named");
    let said: String = told_in(repo.join(".git").as_path(), destination.as_path());
    let first: &str = said.lines().next().expect("a line");
    let shown: String = crate::json_record::quote(destination.to_string_lossy().as_ref());
    assert!(first.contains("finishing the ignored-state copy an interrupted `git worktree add` owed"), "{said}");
    assert!(first.contains(shown.as_str()), "{said}");
    assert!(first.contains("because this command runs in that worktree and its files are missing there"), "{said}");
    assert!(first.contains("it stops at the ceiling one command holds for a copy it did not start"), "{said}");
    assert!(said.contains("finished the ignored-state copy"), "and the finished line follows it: {said}");
    remove(root.as_path());
}

/// A copy that runs past the ceiling a command holds for work it did not start ends as a worktree named
/// for removal,
/// names what ended it,
/// and is never staged again on every later command.
#[test]
fn a_copy_that_passes_its_ceiling_ends_named_and_is_not_staged_again() {
    let (root, repo, destination) = interrupted_add("owed-copy-ceiling");
    let common: PathBuf = repo.join(".git");
    let said: String = {
        let _shorter: crate::worktree_copy_ceiling::ShorterCeilingForTheControl =
            crate::worktree_copy_ceiling::a_shorter_ceiling_for_this_thread(std::time::Duration::ZERO);
        told_in(common.as_path(), destination.as_path())
    };
    assert!(!destination.join("cache/data").exists(), "it copied nothing: {said}");
    assert!(unfilled_records_exist(common.as_path()).expect("records"), "the worktree is named as one whose files are missing: {said}");
    assert!(said.contains("did not finish within the"), "the line says what ended it: {said}");
    let again: String = told_in(common.as_path(), destination.as_path());
    assert!(again.contains("git worktree remove --force"), "every later command names what settles it: {again}");
    assert!(!again.contains("finishing the ignored-state copy"), "and nothing stages the source again: {again}");
    assert!(!destination.join("cache/data").exists(), "nothing is installed after the ceiling");
    remove(root.as_path());
}

/// What:
///  An add killed after it bound its record and before its copy ran still names the copy it owes,
/// and the command run in that worktree makes it.
/// Why:
///   The binding is recorded the moment real Git returns,
/// which is before the copy runs,
/// so a record written with the binding alone
///  named no source and no later command could ever make that copy;
/// the person was left with a worktree that looks ordinary,
/// holds less than its source ignores,
///  and a message that offered removing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claim.recordBinding(binding); expect(owedCopyOf(read(record))).toBeDefined(); toldIn(common, destination); expect(read(destination, 'cache/data')).toEqual('payload');
/// ```
#[test]
fn a_bound_record_keeps_the_copy_it_owes_until_the_worktree_makes_it() {
    let (root, repo, destination) = interrupted_add("owed-copy-bound");
    let common: PathBuf = repo.join(".git");
    let records: PathBuf = journal_root(common.as_path());
    let claim_path: PathBuf = claim_path_of(records.as_path());
    let record: PathBuf = claim_path.with_extension("add");
    let (container, _owner) = read_claim(claim_path.as_path()).expect("claim").expect("named");
    let binding: Binding = crate::worktree_add_binding::bind_registration(common.as_path(), destination.as_path()).expect("bound").binding;
    // The binding is recorded the moment Git returns, which is before the copy runs.
    let claim: StageClaim = StageClaim::adopted(claim_path.as_path(), container.as_path(), Some(record.as_path())).expect("adopted");
    claim.record_binding(&binding).expect("recorded");
    let text: String = std::fs::read_to_string(record.as_path()).expect("the record");
    assert!(
        crate::worktree_add_binding::owed_copy_of(text.as_str()).is_some(),
        "the record written with the binding keeps the copy the add owes: {text}"
    );
    let said: String = told_in(common.as_path(), destination.as_path());
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the ignored file arrived"), b"payload", "the command in that worktree makes the copy: {said}");
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "and the records go: {said}");
    remove(root.as_path());
}

/// What:
///  A record that names a copy is kept by a command that runs elsewhere,
/// and says which worktree makes it,
/// while the same record settles at once when it names no copy.
/// Why:
///   The record is the only place the copy's source and selection survive,
/// so a command that removes it leaves a worktree nothing can fill;
/// a record written by an earlier release names no source,
/// and nothing can fill that one,
/// so it is said once instead of on every command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owed = reap(common); expect(owed[0]).toContain('is still owed'); expect(claimsExist(common)).toBe(true);
/// ```
#[test]
fn a_record_that_names_a_copy_is_kept_by_a_command_in_another_worktree() {
    let (root, repo, destination) = interrupted_add("owed-copy-kept");
    let common: PathBuf = repo.join(".git");
    let records: PathBuf = journal_root(common.as_path());
    let claim_path: PathBuf = claim_path_of(records.as_path());
    let record: PathBuf = claim_path.with_extension("add");
    let (container, _owner) = read_claim(claim_path.as_path()).expect("claim").expect("named");
    let binding: Binding = crate::worktree_add_binding::bind_registration(common.as_path(), destination.as_path()).expect("bound").binding;
    let claim: StageClaim = StageClaim::adopted(claim_path.as_path(), container.as_path(), Some(record.as_path())).expect("adopted");
    claim.record_binding(&binding).expect("recorded");
    let notices: Vec<String> = crate::worktree_copy_stage_claim_reaping::reap_abandoned_stages(common.as_path()).expect("reap");
    assert!(notices.iter().any(|notice: &String| { return notice.contains("that copy is still owed") && notice.contains("the first command run in that worktree makes it"); }), "{notices:?}");
    assert!(stage_claims_exist(common.as_path()).expect("claims"), "the record that names the copy stays");
    // Once the worktree's own command made the copy, nothing is owed and the records go.
    let said: String = told_in(common.as_path(), destination.as_path());
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("the ignored file arrived"), b"payload", "{said}");
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "the records go with the copy: {said}");
    remove(root.as_path());
}

/// A destination that holds the object the add's own snapshot became owes no copy, and the directory Git made at the
/// stage name goes with the add's records.
///
/// This is the kill point right after the exchange that publishes an add: the record still names the copy the add owed,
/// so every command of the repository used to finish a copy nobody needed and to say the worktree was missing files it
/// already held, and the stage name held Git's own directory for ever (sonnet audit of alpha.10, F6).
#[test]
fn a_destination_that_holds_the_adds_snapshot_owes_no_copy_and_leaves_no_stage() {
    let (root, repo, destination) = interrupted_add("owed-copy-snapshot-placed");
    let common: PathBuf = repo.join(".git");
    let claim_path: PathBuf = claim_path_of(crate::worktree_copy_journal::journal_root(common.as_path()).as_path());
    let (container, _owner): (PathBuf, Option<String>) = read_claim(claim_path.as_path()).expect("a claim that reads").expect("one claim");
    let record: PathBuf = claim_path.with_extension("add");
    let owed: OwedCopy = crate::worktree_add_binding::owed_copy_of(std::fs::read_to_string(record.as_path()).expect("the record").as_str()).expect("the record names the copy the add owed");
    let admin: PathBuf = registration_directory(destination.as_path()).expect("the worktree's registration");
    let placed = crate::worktree_add_binding::path_identity(destination.as_path()).expect("the identity of the published snapshot");
    crate::worktree_copy_stage_made_object::record_made(common.as_path(), container.as_path(), placed).expect("the record of the snapshot");
    // The exchange put the snapshot at the destination and the directory Git made at the stage name, holding the pointer
    // Git wrote there.
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).expect("the pointer Git wrote");
    std::fs::create_dir_all(container.as_path()).expect("the directory Git made");
    std::fs::write(container.join(".git"), pointer.as_slice()).expect("Git's pointer");
    let git_made = crate::worktree_add_binding::path_identity(container.as_path()).expect("the identity of the directory Git made");
    std::fs::write(
        record.as_path(),
        crate::worktree_add_binding::registered_record_with(
            &Binding { destination: destination.clone(), destination_id: git_made, admin: admin.clone(), admin_id: crate::worktree_add_binding::path_identity(admin.as_path()).expect("the admin's identity"), pointer },
            Some(&owed),
            None,
        )
        .as_bytes(),
    )
    .expect("the add's record");
    let said: String = told_in(common.as_path(), destination.as_path());
    assert!(!destination.join("cache/data").exists(), "nothing is copied into a worktree whose snapshot already holds the ignored files: {said}");
    assert!(!said.contains("finishing the ignored-state copy"), "and no copy is started: {said}");
    assert!(!said.contains("still owed"), "nothing is said to be owed: {said}");
    assert!(!said.contains("may be missing"), "a snapshot that is in place is not missing the files its source ignores: {said}");
    assert!(!stage_claims_exist(common.as_path()).expect("claims"), "the add's records settle, because the snapshot is whole: {said}");
    assert!(!container.exists(), "the directory Git made is removed with them: {said}");
    remove(root.as_path());
}
