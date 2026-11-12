//! Controls for the shadow ref store,
//! on both ref backends.

use super::*;
use crate::private_storage::take_file_syncs;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
#[cfg(unix)]
use crate::recording_git::{recorded_starts, recording_context};

/// A bare shadow repository in the fixture,
/// with the given ref backend.
fn bare_shadow(root: &Path, ref_format: &str, real_objects: &Path) -> PathBuf {
    let shadow: PathBuf = root.join("shadow");
    let format_argument: String = format!("--ref-format={ref_format}");
    git(
        root,
        &[
            "init",
            "--bare",
            "--quiet",
            "--template=",
            format_argument.as_str(),
            shadow.to_str().expect("fixture"),
        ],
    );
    std::fs::write(shadow.join("objects").join("info").join("alternates"), format!("{}\n", real_objects.display())).expect("alternates");
    return shadow;
}

/// A repository with a tag,
/// a symbolic branch and a second branch,
/// so the snapshot has every kind.
fn decorated_repository(root: &Path) -> (PathBuf, String) {
    let repo: PathBuf = repository(root, "repo");
    git(repo.as_path(), &["tag", "v1"]);
    git(repo.as_path(), &["branch", "Zeta"]);
    git(repo.as_path(), &["symbolic-ref", "refs/heads/alias", "refs/heads/main"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (repo, head);
}

/// The sources of a fixture repository whose worktree owns its own Git directory.
///
/// A fixture's `HEAD`, refs and `packed-refs` all live in `<repo>/.git`,
/// so the shared directory and the worktree's own are one path.
fn sources_of(repo: &Path, ref_format: RefFormat) -> SnapshotSources {
    let git_dir: PathBuf = repo.join(".git");
    return SnapshotSources::of(git_dir.clone(), git_dir, ref_format, "sha1");
}

/// Whether a files snapshot made no Git start at all.
///
/// The listing of the real refs is read from `packed-refs` and the loose files themselves,
/// and every ref write is the wrapper's own,
///  so a commit's ref snapshot starts no process over the repository.
#[cfg(unix)]
fn assert_no_start(log: &Path) {
    let starts: Vec<String> = recorded_starts(log);
    assert!(starts.is_empty(), "{starts:?}");
}

/// Packed text names the header,
/// every direct ref except the target,
/// and no symbolic ref.
#[test]
fn packed_refs_skip_the_target_and_symbolic_refs() {
    let refs: Vec<RefSnapshotEntry> = vec![
        RefSnapshotEntry {
            name: b"refs/heads/alias".to_vec(),
            oid: String::new(),
            symref: b"refs/heads/main".to_vec(),
        },
        RefSnapshotEntry {
            name: b"refs/heads/main".to_vec(),
            oid: String::from("1111111111111111111111111111111111111111"),
            symref: Vec::new(),
        },
        RefSnapshotEntry {
            name: b"refs/tags/v1".to_vec(),
            oid: String::from("2222222222222222222222222222222222222222"),
            symref: Vec::new(),
        },
    ];
    assert_eq!(
        format_packed_refs(&refs, &crate::ref_name::RefName::from_text("refs/heads/main")),
        b"# pack-refs with: sorted \n2222222222222222222222222222222222222222 refs/tags/v1\n".to_vec()
    );
}

/// The real ref listing sorts by raw bytes,
/// so `Zeta` precedes `main`,
/// and marks symbolic refs.
#[test]
fn real_refs_sort_by_bytes_and_mark_symbolic_refs() {
    let root: PathBuf = fixture("shadow-refs-list");
    let (repo, _head) = decorated_repository(root.as_path());
    let refs: Vec<RefSnapshotEntry> = list_real_refs(&git_context(), repo.as_path()).expect("refs");
    let names: Vec<&[u8]> = refs.iter().map(|entry| { return entry.name.as_slice(); }).collect();
    let zeta: usize = names.iter().position(|name| { return *name == b"refs/heads/Zeta".as_slice(); }).expect("Zeta");
    let main: usize = names.iter().position(|name| { return *name == b"refs/heads/main".as_slice(); }).expect("main");
    assert!(zeta < main, "{names:?}");
    let alias = refs.iter().find(|entry| { return entry.name.as_slice() == b"refs/heads/alias".as_slice(); }).expect("alias");
    assert_eq!(alias.symref, b"refs/heads/main".to_vec());
    remove(root.as_path());
}

/// A files shadow holds the snapshot,
/// the loose target,
/// the symbolic ref and `HEAD` on the branch.
#[test]
fn files_shadow_snapshots_refs_and_points_head_at_the_branch() {
    let root: PathBuf = fixture("shadow-refs-files");
    let (repo, head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/heads/main"]), head);
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/tags/v1"]), head);
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "refs/heads/alias"]), "refs/heads/main");
    remove(root.as_path());
}

/// The files backend's shadow ref store is written by the wrapper itself:
/// the one Git start it makes is the listing of the real refs,
/// and no start writes a ref.
///
/// Its ref store is what a hook and the scratch commit read,
/// so the private target,
///  the symbolic refs,
///  and `HEAD` must be exactly what `git update-ref` and `git symbolic-ref` wrote.
#[cfg(unix)]
#[test]
fn a_files_snapshot_writes_every_ref_without_a_git_start() {
    let root: PathBuf = fixture("shadow-refs-files-no-starts");
    let (repo, head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    let (context, log) = recording_context(root.as_path());
    write_shadow_refs(
        &context,
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_no_start(log.as_path());
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/heads/main"]), head);
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/tags/v1"]), head);
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/heads/Zeta"]), head);
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "refs/heads/alias"]), "refs/heads/main");
    remove(root.as_path());
}

/// A detached shadow's `HEAD` is the base itself,
/// written where Git reads it.
#[cfg(unix)]
#[test]
fn a_files_detached_head_is_written_where_git_reads_it() {
    let root: PathBuf = fixture("shadow-refs-files-head");
    let (repo, head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    let (context, log) = recording_context(root.as_path());
    write_shadow_refs(
        &context,
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Detached,
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_no_start(log.as_path());
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "HEAD"]), head);
    remove(root.as_path());
}

/// A branch that is itself a symbolic ref to another branch keeps its chain in the
///  shadow,
/// and the private target is written where the chain ends,
/// as the `git update-ref` this replaces wrote it.
#[cfg(unix)]
#[test]
fn a_files_target_that_is_a_symbolic_ref_is_written_at_the_end_of_its_chain() {
    let root: PathBuf = fixture("shadow-refs-files-chain");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["symbolic-ref", "refs/heads/alias", "refs/heads/main"]);
    git(repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/alias"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("refs/heads/alias"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/alias")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    // The chain survives, and every view of it that a hook asks Git for answers as the real repository does.
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "refs/heads/alias"]), git_text(repo.as_path(), &["symbolic-ref", "refs/heads/alias"]));
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), git_text(repo.as_path(), &["symbolic-ref", "HEAD"]));
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/heads/main"]), head);
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "HEAD"]), head);
    remove(root.as_path());
}

/// The files snapshot is published whole and made no file sync:
/// the shadow is thrown away,
/// and nothing reads its refs
///  after a crash.
#[test]
fn the_files_snapshot_is_not_synced() {
    let root: PathBuf = fixture("shadow-refs-files-sync");
    let (repo, head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    take_file_syncs();
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
        &Base::Commit(head),
    )
    .expect("refs");
    assert!(shadow.join("packed-refs").is_file());
    assert_eq!(take_file_syncs(), Vec::<PathBuf>::new());
    remove(root.as_path());
}

/// A detached shadow has no symbolic `HEAD`,
/// only the base as a direct value.
#[test]
fn detached_shadow_head_is_the_base() {
    let root: PathBuf = fixture("shadow-refs-detached");
    let (repo, head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("HEAD"),
        &SymbolicHead::Detached,
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "HEAD"]), head);
    assert!(run_git_checked(
        &git_context(),
        &shadow_request(shadow.as_path(), shadow.as_path(), &["symbolic-ref", "-q", "HEAD"]),
    )
    .is_err());
    remove(root.as_path());
}

/// A detached `HEAD` without a base cannot be written,
/// and the failure says why.
#[test]
fn detached_without_a_base_fails() {
    let root: PathBuf = fixture("shadow-refs-detached-unborn");
    let (repo, _head) = decorated_repository(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    let error: GitFailure = write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("HEAD"),
        &SymbolicHead::Detached,
        &Base::Unborn,
    )
    .expect_err("no base");
    assert_eq!(error.0, "A detached HEAD has no unborn preparation base.");
    remove(root.as_path());
}

/// An unborn branch names the branch in `HEAD` but writes no target ref.
#[test]
fn unborn_branch_has_no_target_ref() {
    let root: PathBuf = fixture("shadow-refs-unborn");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repo");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
        &Base::Unborn,
    )
    .expect("refs");
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main");
    assert!(run_git_checked(
        &git_context(),
        &shadow_request(shadow.as_path(), shadow.as_path(), &["rev-parse", "--verify", "refs/heads/main"]),
    )
    .is_err());
    remove(root.as_path());
}

/// A reftable shadow receives the same snapshot through one `update-ref --stdin` transaction.
#[test]
fn reftable_shadow_snapshots_refs_in_one_transaction() {
    let root: PathBuf = fixture("shadow-refs-reftable");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repo");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    git(repo.as_path(), &["tag", "v1"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let shadow: PathBuf = bare_shadow(root.as_path(), "reftable", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Reftable),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_eq!(git_text(shadow.as_path(), &["rev-parse", "refs/tags/v1"]), head);
    assert_eq!(git_text(shadow.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main");
    remove(root.as_path());
}

/// The tag,
/// branch and symbolic ref of the byte-name fixture.
const NON_TEXT_TAG: &[u8] = b"refs/tags/nonutf-\xff";
const NON_TEXT_BRANCH: &[u8] = b"refs/heads/nonutf-\xff";
const NON_TEXT_ALIAS: &[u8] = b"refs/heads/alias-\xfe";

/// A repository holding a tag,
/// a branch and a symbolic ref whose names are valid bytes that are not text.
fn byte_named_refs(root: &Path) -> (PathBuf, String) {
    let repo: PathBuf = repository(root, "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let tag: OsString = crate::path_bytes::os_from_path_bytes(NON_TEXT_TAG);
    let branch: OsString = crate::path_bytes::os_from_path_bytes(NON_TEXT_BRANCH);
    let alias: OsString = crate::path_bytes::os_from_path_bytes(NON_TEXT_ALIAS);
    for name in [&tag, &branch] {
        git(repo.as_path(), &[OsStr::new("update-ref"), name.as_os_str(), OsStr::new(head.as_str())]);
    }
    git(repo.as_path(), &[OsStr::new("symbolic-ref"), alias.as_os_str(), branch.as_os_str()]);
    return (repo, head);
}

/// The shadow holds every byte-named ref the real repository holds,
/// as the same name,
/// and the symbolic ref still points at the same branch.
fn assert_byte_named_refs(shadow: &Path, head: &str) {
    let answer = run_git_checked(&git_context(), &shadow_request(shadow, shadow, &["for-each-ref", "--format=%(refname)%00%(objectname)"])).expect("shadow refs");
    let mut records: Vec<(&[u8], &[u8])> = Vec::new();
    for record in answer.stdout.split(|byte| { return *byte == b'\n'; }) {
        let fields: Vec<&[u8]> = record.split(|byte| { return *byte == 0; }).collect();
        if let [name, oid] = fields.as_slice() {
            records.push((name, oid));
        }
    }
    for name in [NON_TEXT_TAG, NON_TEXT_BRANCH, NON_TEXT_ALIAS] {
        let found: Option<&(&[u8], &[u8])> = records.iter().find(|(found, _)| { return *found == name; });
        assert!(found.is_some(), "{:?} is missing from the shadow", String::from_utf8_lossy(name));
    }
    // A tag and a branch point at the commit, and the symbolic name resolves through the branch it names.
    for name in [NON_TEXT_TAG, NON_TEXT_BRANCH] {
        let oid: &&[u8] = records.iter().find(|(found, _)| { return *found == name; }).map(|(_, oid)| { return oid; }).expect("a listed ref");
        assert_eq!(*oid, head.as_bytes(), "{:?}", String::from_utf8_lossy(name));
    }
    let alias: OsString = crate::path_bytes::os_from_path_bytes(NON_TEXT_ALIAS);
    let resolved = run_git_checked(&git_context(), &shadow_request(shadow, shadow, &[OsStr::new("symbolic-ref"), alias.as_os_str()])).expect("symbolic ref");
    assert_eq!(resolved.stdout, [NON_TEXT_BRANCH, b"\n"].concat());
}

/// The files snapshot writes a name that is not text as its bytes,
/// so the shadow's Git reads the same ref.
#[test]
fn the_files_snapshot_keeps_a_name_that_is_not_text() {
    let root: PathBuf = fixture("shadow-refs-files-non-text");
    let (repo, head) = byte_named_refs(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_byte_named_refs(shadow.as_path(), head.as_str());
    remove(root.as_path());
}

/// The reftable snapshot sends a name that is not text as its bytes too.
#[test]
fn the_reftable_snapshot_keeps_a_name_that_is_not_text() {
    let root: PathBuf = fixture("shadow-refs-reftable-non-text");
    let (repo, head) = byte_named_refs(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "reftable", repo.join(".git").join("objects").as_path());
    write_shadow_refs(
        &git_context(),
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Reftable),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_byte_named_refs(shadow.as_path(), head.as_str());
    remove(root.as_path());
}

/// How many branches the count control's repository holds above the two its fixture makes.
///
/// The number is the fixture's own,
/// and the control's claim is that no reading of the snapshot is asked about it:
/// a count smaller than one pipe holds would pass for the wrong reason,
/// so it is above 64 KiB of `packed-refs` lines.
const MANY_REFS_ABOVE_THE_FIXTURE: usize = 700;

/// A repository whose ref count the caller chose,
/// with every branch packed,
/// as a repository that has been used for a while keeps them.
fn repository_with_many_refs(root: &Path, above_the_fixture: usize) -> (PathBuf, String) {
    let repo: PathBuf = repository(root, "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    for index in 0..above_the_fixture {
        let name: String = format!("refs/heads/counted-{index:04}");
        git(repo.as_path(), &["update-ref", name.as_str(), head.as_str()]);
    }
    git(repo.as_path(), &["pack-refs", "--all"]);
    return (repo, head);
}

/// What:
///  The refs a repository lists,
///  as the shadow's Git lists them,
///  for a comparison with the snapshot.
///
/// Why:
///   The snapshot's claim is that it holds every ref Git would print,
/// so the control compares it with Git's own answer over the same repository.
fn listed_refs(directory: &Path) -> String {
    return git_text(directory, &["for-each-ref", "--format=%(refname)"]);
}

/// A repository of many refs starts no Git child and still holds every ref.
///
/// The cost of one commit's ref snapshot must not follow the number of refs the repository has:
/// on 2026-11-12 the whole listing as a child of a commit took 3.8 to 26.4 seconds while real Git
///  answered it in 4 to 5 milliseconds,
/// and this control counts the children the snapshot starts at two sizes of repository.
#[cfg(unix)]
#[test]
fn a_snapshot_of_many_refs_starts_no_git_child() {
    let mut starts_per_size: Vec<usize> = Vec::new();
    let mut refs_per_size: Vec<usize> = Vec::new();
    for above in [0, MANY_REFS_ABOVE_THE_FIXTURE] {
        let root: PathBuf = fixture(format!("shadow-refs-count-{above}").as_str());
        let (repo, head) = repository_with_many_refs(root.as_path(), above);
        let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
        let (context, log) = recording_context(root.as_path());
        write_shadow_refs(
            &context,
            repo.as_path(),
            shadow.as_path(),
            &sources_of(repo.as_path(), RefFormat::Files),
            &crate::ref_name::RefName::from_text("refs/heads/main"),
            &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
            &Base::Commit(head.clone()),
        )
        .expect("refs");
        assert_no_start(log.as_path());
        // The snapshot is the whole listing: every ref the repository lists, and not one more.
        let expected: String = listed_refs(repo.as_path());
        let written: String = listed_refs(shadow.as_path());
        assert_eq!(written, expected, "a repository of {above} refs above the fixture's");
        starts_per_size.push(recorded_starts(log.as_path()).len());
        refs_per_size.push(expected.lines().count());
        remove(root.as_path());
    }
    // The repository grew; what the snapshot asked a Git child did not.
    assert!(refs_per_size[1] > refs_per_size[0] * 10, "{refs_per_size:?}");
    assert_eq!(starts_per_size, vec![0, 0]);
}

/// A ref name the reader declines sends the snapshot to Git, which then answers every ref.
///
/// The fallback is what keeps a repository this reader cannot prove from changing what a commit does:
/// a start is added,
/// and the snapshot is Git's answer,
/// not a shorter one.
#[cfg(unix)]
#[test]
fn a_ref_name_the_reader_declines_asks_git_for_the_whole_listing() {
    let root: PathBuf = fixture("shadow-refs-fall-back-to-git");
    let (repo, head) = byte_named_refs(root.as_path());
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    let (context, log) = recording_context(root.as_path());
    write_shadow_refs(
        &context,
        repo.as_path(),
        shadow.as_path(),
        &sources_of(repo.as_path(), RefFormat::Files),
        &crate::ref_name::RefName::from_text("refs/heads/main"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/main")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    let starts: Vec<String> = recorded_starts(log.as_path());
    assert_eq!(starts.len(), 1, "{starts:?}");
    assert!(starts[0].starts_with("for-each-ref "), "{starts:?}");
    assert_byte_named_refs(shadow.as_path(), head.as_str());
    remove(root.as_path());
}

/// A linked worktree's own namespaces are in the snapshot, and reading them starts no child either.
///
/// Git keeps `refs/bisect`, `refs/worktree` and `refs/rewritten` in the worktree's own Git directory,
///  and `git for-each-ref` in that worktree prints them;
/// a snapshot read from the shared directory alone would leave them out and change what a hook sees.
#[cfg(unix)]
#[test]
fn a_linked_worktree_keeps_its_own_namespaces_without_a_git_child() {
    let root: PathBuf = fixture("shadow-refs-linked-worktree");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(
        repo.as_path(),
        &["worktree", "add", "--quiet", linked.to_str().expect("fixture"), "-b", "side"],
    );
    // A bisect in progress is what puts a name in `refs/bisect` of that worktree's own Git directory.
    git(linked.as_path(), &["update-ref", "refs/bisect/bad", head.as_str()]);
    let git_dir: PathBuf = linked.join(".git");
    let git_dir_text: String = std::fs::read_to_string(git_dir.as_path()).expect("the worktree's gitdir file");
    let worktree_git_dir: PathBuf = PathBuf::from(git_dir_text.trim().trim_start_matches("gitdir: "));
    let shadow: PathBuf = bare_shadow(root.as_path(), "files", repo.join(".git").join("objects").as_path());
    let (context, log) = recording_context(root.as_path());
    write_shadow_refs(
        &context,
        linked.as_path(),
        shadow.as_path(),
        &SnapshotSources::of(worktree_git_dir.clone(), repo.join(".git"), RefFormat::Files, "sha1"),
        &crate::ref_name::RefName::from_text("refs/heads/side"),
        &SymbolicHead::Branch(crate::ref_name::RefName::from_text("refs/heads/side")),
        &Base::Commit(head.clone()),
    )
    .expect("refs");
    assert_no_start(log.as_path());
    assert_eq!(listed_refs(shadow.as_path()), listed_refs(linked.as_path()));
    assert!(listed_refs(shadow.as_path()).contains("refs/bisect/bad\n"), "{}", listed_refs(shadow.as_path()));
    remove(root.as_path());
}
