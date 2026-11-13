//! Controls for the shadow ref store,
//! on both ref backends.

use super::*;
use crate::shadow_refs::{SnapshotSources, list_real_refs};
use crate::transaction_journal::RefFormat;
use crate::private_storage::take_file_syncs;
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};
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
/// A fixture's `HEAD`,
/// refs and `packed-refs` all live in `<repo>/.git`,
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

/// A ref name the reader declines sends the snapshot to Git,
/// which then answers every ref.
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

/// A linked worktree's own namespaces are in the snapshot,
/// and reading them starts no child either.
///
/// Git keeps `refs/bisect`,
/// `refs/worktree` and `refs/rewritten` in the worktree's own Git directory,
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

/// The listing both sides are asked for:
/// the snapshot's own three fields,
/// which is the format the Git child is given.
const LISTING_FORMAT: &str = "--format=%(refname)%00%(objectname)%00%(symref)";

/// What real Git prints for a repository,
/// one line per ref,
/// in the snapshot's own three fields.
///
/// Git is the oracle of every control here:
/// the module's contract is that the snapshot is what Git would print,
/// or a decline that starts Git.
fn git_listing(directory: &Path) -> Vec<u8> {
    let output: std::process::Output = git_output(directory, &["for-each-ref", LISTING_FORMAT]);
    assert!(output.status.success(), "real Git refused the listing: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    // The bytes as Git printed them: a ref name that is not text is compared as the bytes it is.
    return output.stdout;
}

/// What:
///  A listing as a person reads it,
///  one ref to a line,
///  for a failure's message.
/// Why:
///   The comparison is over bytes and stays exact;
/// a message of byte numbers would say nothing about which ref differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rendered = (bytes) => bytes.toString('utf8').replace(/\0/g, ' | ')
/// ```
fn rendered(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).replace('\0', " | ");
}

/// The snapshot's entries as the same kind of lines,
/// so the two answers are compared byte for byte.
fn snapshot_listing(entries: &[RefSnapshotEntry]) -> Vec<u8> {
    let mut text: Vec<u8> = Vec::new();
    for entry in entries {
        text.extend_from_slice(entry.name.as_slice());
        text.push(0);
        text.extend_from_slice(entry.oid.as_bytes());
        text.push(0);
        text.extend_from_slice(entry.symref.as_slice());
        text.push(b'\n');
    }
    return text;
}

/// What:
///  The Git directory a linked worktree keeps for itself,
///  read from the `gitdir:` file its worktree root holds.
/// Why:
///   That directory is where Git keeps the per-worktree namespaces,
/// and the snapshot has to read exactly the ones Git resolves there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ownGitDir = (worktree) => readFileSync(join(worktree, '.git'), 'utf8').trim().replace('gitdir: ', '')
/// ```
fn own_git_dir_of(worktree: &Path) -> PathBuf {
    let text: String = std::fs::read_to_string(worktree.join(".git")).expect("the worktree's gitdir file");
    return PathBuf::from(text.trim().trim_start_matches("gitdir: "));
}

/// What:
///  A reusable repository of the sweep,
///  with one commit so every ref has an object to point at.
/// Why:
///   Each kind differs only in how its refs are stored,
/// so the commit is made once per kind and never again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const storeRoot = (root, kind) => { const repo = repository(join(root, kind), 'repo'); return repo }
/// ```
fn store_of_kind(root: &Path, kind: &str) -> PathBuf {
    let directory: PathBuf = root.join(kind);
    std::fs::create_dir_all(directory.as_path()).expect("a fixture directory");
    return repository(directory.as_path(), "repo");
}

/// What:
///  Writes one loose ref file into a repository's own Git directory,
///  which is how a broken or unusual ref is made.
/// Why:
///   `git update-ref` refuses the very spellings these controls read:
/// a value without a newline,
/// a value with CRLF,
/// a symbolic ref to a name that does not exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeLoose(repo, 'refs/heads/x', 'ref: refs/heads/nope\n')
/// ```
fn write_loose(repo: &Path, name: &str, value: &str) {
    let path: PathBuf = repo.join(".git").join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the ref's directories");
    }
    std::fs::write(path.as_path(), value.as_bytes()).expect("a loose ref file");
}

/// The head commit of a repository of the sweep.
fn head_of(repo: &Path) -> String {
    return git_text(repo, &["rev-parse", "HEAD"]);
}

/// What the reader has to do with one kind of store,
/// which is the whole of this reader's contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Reading {
    /// The reader answers in process,
    /// and its answer is exactly Git's.
    InProcess,
    /// The reader declines,
    /// Git answers,
    /// and the snapshot is Git's answer.
    ByGit,
    /// The reader declines and Git refuses to list the store at all,
    /// so the commit is refused with Git's own words.
    Refused,
}

/// What one kind of store is,
/// and what the reader must do with it.
struct StoreKind {
    /// The name the fixture directory and the failure's clause carry.
    name: &'static str,
    /// Whether the reader has to answer in process,
    /// or has to decline and let Git answer.
    reading: Reading,
}

/// Every kind of ref store the sweep builds.
///
/// The list is the point of the control:
/// the two findings of the review were both
/// "the reader is not Git on input the fixtures never held",
/// so every kind of store a repository can hold is named here,
/// and the ones no fixture can build on this machine are named in the report instead.
const STORE_KINDS: [StoreKind; 21] = [
    StoreKind { name: "packed", reading: Reading::InProcess },
    StoreKind { name: "loose-over-packed", reading: Reading::InProcess },
    StoreKind { name: "peeled-annotated-tag", reading: Reading::InProcess },
    StoreKind { name: "symbolic-ref", reading: Reading::InProcess },
    StoreKind { name: "symbolic-chain", reading: Reading::InProcess },
    StoreKind { name: "symbolic-chain-of-five", reading: Reading::InProcess },
    StoreKind { name: "symbolic-chain-of-six", reading: Reading::InProcess },
    StoreKind { name: "symbolic-to-a-missing-object", reading: Reading::InProcess },
    StoreKind { name: "symbolic-ending-nowhere", reading: Reading::InProcess },
    StoreKind { name: "worktree-namespaces-of-the-main-worktree", reading: Reading::InProcess },
    StoreKind { name: "stale-lock-beside-a-ref", reading: Reading::InProcess },
    StoreKind { name: "empty-repository", reading: Reading::InProcess },
    StoreKind { name: "direct-ref-with-a-missing-object", reading: Reading::InProcess },
    StoreKind { name: "name-with-at-and-braces", reading: Reading::InProcess },
    StoreKind { name: "name-that-is-not-text", reading: Reading::ByGit },
    StoreKind { name: "value-without-a-newline", reading: Reading::ByGit },
    StoreKind { name: "value-with-a-carriage-return", reading: Reading::ByGit },
    StoreKind { name: "damaged-packed-refs", reading: Reading::Refused },
    StoreKind { name: "reftable", reading: Reading::ByGit },
    StoreKind { name: "sha256-object-format", reading: Reading::InProcess },
    StoreKind { name: "ref-file-that-is-a-symlink", reading: Reading::ByGit },
];

/// What:
///  Builds one kind of store and gives the directory Git is asked from,
///  the sources the snapshot is read with,
///  and whether the reader has to answer in process or decline.
/// Why:
///   One match names every kind once,
/// so the sweep's list and its builders cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const build = (root, kind) => ({ cwd, sources, readInProcess })
/// ```
fn build_store(root: &Path, kind: &StoreKind) -> (PathBuf, SnapshotSources, Reading) {
    let repo: PathBuf = store_of_kind(root, kind.name);
    let head: String = head_of(repo.as_path());
    let mut ref_format: RefFormat = RefFormat::Files;
    match kind.name {
        "packed" => {
            git(repo.as_path(), &["branch", "packed-one"]);
            git(repo.as_path(), &["pack-refs", "--all"]);
        }
        "loose-over-packed" => {
            // The packed entry names the first commit and the loose file names another, so the two spellings differ.
            git(repo.as_path(), &["update-ref", "refs/heads/twice", head.as_str()]);
            git(repo.as_path(), &["pack-refs", "--all"]);
            write_loose(repo.as_path(), "refs/heads/twice", format!("{head}\n").as_str());
            git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second"]);
            let second: String = head_of(repo.as_path());
            write_loose(repo.as_path(), "refs/heads/twice", format!("{second}\n").as_str());
        }
        "peeled-annotated-tag" => {
            git(repo.as_path(), &["tag", "--annotate", "--message=tagged", "v1"]);
            git(repo.as_path(), &["pack-refs", "--all"]);
        }
        "symbolic-ref" => {
            git(repo.as_path(), &["symbolic-ref", "refs/heads/alias", "refs/heads/main"]);
        }
        "symbolic-chain" => {
            git(repo.as_path(), &["symbolic-ref", "refs/heads/alias", "refs/heads/main"]);
            git(repo.as_path(), &["symbolic-ref", "refs/heads/alias-again", "refs/heads/alias"]);
        }
        // Git follows at most five symbolic refs (`SYMREF_MAXDEPTH`), and the reader has to make the same cut:
        //  the five-link chain ends at `main` and the six-link one does not end at a ref Git will follow.
        "symbolic-chain-of-five" | "symbolic-chain-of-six" => {
            let links: usize = if kind.name == "symbolic-chain-of-five" { 5 } else { 6 };
            for step in 1..=links {
                let target: String = if step == links { String::from("refs/heads/main") } else { format!("refs/heads/link-{}", step + 1) };
                git(repo.as_path(), &["symbolic-ref", format!("refs/heads/link-{step}").as_str(), target.as_str()]);
            }
        }
        "symbolic-to-a-missing-object" => {
            write_loose(repo.as_path(), "refs/heads/ghost", "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n");
            write_loose(repo.as_path(), "refs/heads/alias-to-ghost", "ref: refs/heads/ghost\n");
        }
        "symbolic-ending-nowhere" => {
            // A dangling target and a two-name loop: Git prints neither, and a snapshot that holds them is not Git's answer.
            write_loose(repo.as_path(), "refs/heads/dangling", "ref: refs/heads/nope\n");
            write_loose(repo.as_path(), "refs/heads/cycle-a", "ref: refs/heads/cycle-b\n");
            write_loose(repo.as_path(), "refs/heads/cycle-b", "ref: refs/heads/cycle-a\n");
            write_loose(repo.as_path(), "refs/heads/self", "ref: refs/heads/self\n");
            git(repo.as_path(), &["symbolic-ref", "refs/heads/sound", "refs/heads/main"]);
        }
        "worktree-namespaces-of-the-main-worktree" => {
            for name in ["refs/bisect/start", "refs/rewritten/r1", "refs/worktree/w1"] {
                git(repo.as_path(), &["update-ref", name, head.as_str()]);
            }
        }
        "stale-lock-beside-a-ref" => {
            write_loose(repo.as_path(), "refs/heads/main.lock", "");
        }
        "empty-repository" => {
            // A repository with no commit at all: no ref is written, and `HEAD` names an unborn branch.
            let empty: PathBuf = root.join("empty-repository-empty");
            std::fs::create_dir_all(empty.as_path()).expect("a fixture directory");
            git(empty.as_path(), &["init", "--quiet", "--initial-branch=main"]);
            let git_dir: PathBuf = empty.join(".git");
            return (empty, SnapshotSources::of(git_dir.clone(), git_dir, RefFormat::Files, "sha1"), kind.reading);
        }
        "direct-ref-with-a-missing-object" => {
            write_loose(repo.as_path(), "refs/heads/ghost", "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n");
        }
        "name-with-at-and-braces" => {
            // `@`, `+`, `{` and `}` are bytes Git accepts in a name, and the reader accepts them too.
            git(repo.as_path(), &["update-ref", "refs/heads/a@b", head.as_str()]);
            git(repo.as_path(), &["update-ref", "refs/tags/x{y}", head.as_str()]);
        }
        "name-that-is-not-text" => {
            let name: OsString = crate::path_bytes::os_from_path_bytes(b"refs/heads/nonutf-\xff");
            git(repo.as_path(), &[OsStr::new("update-ref"), name.as_os_str(), OsStr::new(head.as_str())]);
        }
        "value-without-a-newline" => {
            write_loose(repo.as_path(), "refs/heads/torn", head.as_str());
        }
        "value-with-a-carriage-return" => {
            write_loose(repo.as_path(), "refs/heads/crlf", format!("{head}\r\n").as_str());
        }
        "damaged-packed-refs" => {
            // A line with no space names no ref: Git words it and prints the rest, and this reader declines the file.
            git(repo.as_path(), &["branch", "sound"]);
            git(repo.as_path(), &["pack-refs", "--all"]);
            let packed: PathBuf = repo.join(".git").join("packed-refs");
            let text: String = std::fs::read_to_string(packed.as_path()).expect("packed-refs");
            std::fs::write(packed.as_path(), format!("{text}this line names no ref\n").as_str()).expect("a damaged packed-refs");
        }
        "reftable" => {
            let own: PathBuf = root.join("reftable-repository");
            std::fs::create_dir_all(own.as_path()).expect("a fixture directory");
            git(own.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
            std::fs::write(own.join("a.txt"), "a\n").expect("a file");
            git(own.as_path(), &["add", "a.txt"]);
            git(own.as_path(), &["commit", "--quiet", "--message=one"]);
            ref_format = RefFormat::Reftable;
            let git_dir: PathBuf = own.join(".git");
            return (own, SnapshotSources::of(git_dir.clone(), git_dir, ref_format, "sha1"), kind.reading);
        }
        // A repository whose object names are 64 hexadecimal digits: the reader is given that width by its sources.
        "sha256-object-format" => {
            let own: PathBuf = root.join("sha256-repository");
            std::fs::create_dir_all(own.as_path()).expect("a fixture directory");
            git(own.as_path(), &["init", "--quiet", "--object-format=sha256", "--initial-branch=main"]);
            std::fs::write(own.join("a.txt"), "a\n").expect("a file");
            git(own.as_path(), &["add", "a.txt"]);
            git(own.as_path(), &["commit", "--quiet", "--message=one"]);
            let git_dir: PathBuf = own.join(".git");
            return (own, SnapshotSources::of(git_dir.clone(), git_dir, RefFormat::Files, "sha256"), kind.reading);
        }
        // A loose ref that is a symbolic link is not a ref file Git wrote, so the reader declines and Git answers.
        "ref-file-that-is-a-symlink" => {
            let elsewhere: PathBuf = repo.join("elsewhere");
            std::fs::write(elsewhere.as_path(), format!("{head}\n")).expect("a file to point at");
            let path: PathBuf = repo.join(".git").join("refs/heads/linked");
            std::os::unix::fs::symlink(elsewhere.as_path(), path.as_path()).expect("a linked ref file");
        }
        other => panic!("the sweep names a kind it does not build: {other}"),
    }
    let git_dir: PathBuf = repo.join(".git");
    return (repo, SnapshotSources::of(git_dir.clone(), git_dir, ref_format, "sha1"), kind.reading);
}

/// What:
///  Every kind of store the fixtures can build is answered by the reader or declined,
///  and either way the snapshot is exactly what Git prints.
/// Why:
///  The review's two findings were both a store the fixtures never held,
/// so one control that builds every kind and asks both sides is what keeps the reader honest.
///  A kind the reader answers must start no Git child,
/// and a kind it declines must start exactly one `for-each-ref`;
/// both answers are compared byte for byte with real Git's listing of the same store.
#[cfg(unix)]
#[test]
fn the_reader_answers_every_kind_of_store_as_git_does_or_declines() {
    let root: PathBuf = fixture("shadow-refs-every-kind-of-store");
    let mut answered: usize = 0;
    let mut declined: usize = 0;
    let mut refused_by_git: usize = 0;
    for kind in STORE_KINDS {
        let (cwd, sources, reading) = build_store(root.as_path(), &kind);
        let recording: PathBuf = root.join(format!("recording-of-{}", kind.name));
        std::fs::create_dir_all(recording.as_path()).expect("a recording directory");
        let (context, log) = recording_context(recording.as_path());
        let asked: Result<Vec<RefSnapshotEntry>, GitFailure> = list_real_refs_from(&context, cwd.as_path(), &sources);
        let starts: Vec<String> = recorded_starts(log.as_path());
        if reading == Reading::Refused {
            // Git refuses the whole store, so the snapshot refuses with it and nothing is listed by either side.
            assert_eq!(starts.len(), 1, "{}: the reader must decline and let Git refuse, and it started {starts:?}", kind.name);
            assert!(asked.is_err(), "{}: Git refuses this store and the snapshot answered it", kind.name);
            let refused: std::process::Output = git_output(cwd.as_path(), &["for-each-ref", LISTING_FORMAT]);
            assert!(!refused.status.success(), "{}: this kind is in the sweep as refused, and Git listed it", kind.name);
            refused_by_git = refused_by_git.saturating_add(1);
            continue;
        }
        let entries: Vec<RefSnapshotEntry> = asked.unwrap_or_else(|failure| panic!("{}: {failure}", kind.name));
        if reading == Reading::InProcess {
            assert!(starts.is_empty(), "{}: the reader must answer without a child, and it started {starts:?}", kind.name);
            answered = answered.saturating_add(1);
        } else {
            assert_eq!(starts.len(), 1, "{}: the reader must decline and let Git answer, and it started {starts:?}", kind.name);
            assert!(starts[0].starts_with("for-each-ref "), "{}: {starts:?}", kind.name);
            declined = declined.saturating_add(1);
        }
        let snapshot: Vec<u8> = snapshot_listing(entries.as_slice());
        let printed: Vec<u8> = git_listing(cwd.as_path());
        assert!(
            snapshot == printed,
            "{}: the snapshot is not what Git prints\n  snapshot: {}\n  Git:      {}",
            kind.name,
            rendered(snapshot.as_slice()),
            rendered(printed.as_slice())
        );
    }
    // Every kind in the list was built and taken through one of the contract's arms, and each arm was exercised.
    assert_eq!(answered.saturating_add(declined).saturating_add(refused_by_git), STORE_KINDS.len());
    assert!(answered > 0 && declined > 0 && refused_by_git > 0, "{answered} answered, {declined} declined, {refused_by_git} refused");
    remove(root.as_path());
}

/// What:
///  The per-worktree namespaces of the *shared* directory are not this worktree's refs,
///  so the snapshot of a linked worktree is exactly what Git prints from it.
/// Why:
///  Git resolves `refs/bisect`,
/// `refs/worktree` and `refs/rewritten` to the worktree's own Git directory,
///  so a linked worktree never sees the main worktree's copies.
/// The snapshot read the shared directory's whole `refs` tree and then overlaid the worktree's
/// own three namespaces,
/// which kept the shared copies:
/// a hook in the shadow then saw refs this
/// worktree does not have,
/// and could take the main worktree's state for its own.
#[cfg(unix)]
#[test]
fn a_linked_worktree_does_not_snapshot_the_shared_per_worktree_refs() {
    let root: PathBuf = fixture("shadow-refs-linked-shared-namespaces");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = head_of(repo.as_path());
    // The main worktree's per-worktree refs live in the shared Git directory.
    for name in ["refs/bisect/start", "refs/rewritten/r1", "refs/worktree/w1"] {
        git(repo.as_path(), &["update-ref", name, head.as_str()]);
    }
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", linked.to_str().expect("fixture"), "-b", "side"]);
    // The linked worktree has one of its own, so both sides hold something and the two cannot be confused.
    git(linked.as_path(), &["update-ref", "refs/bisect/own", head.as_str()]);
    let sources: SnapshotSources = SnapshotSources::of(own_git_dir_of(linked.as_path()), repo.join(".git"), RefFormat::Files, "sha1");
    let recording: PathBuf = root.join("recording");
    std::fs::create_dir_all(recording.as_path()).expect("a recording directory");
    let (context, log) = recording_context(recording.as_path());
    let entries: Vec<RefSnapshotEntry> = list_real_refs_from(&context, linked.as_path(), &sources).expect("the snapshot");
    assert_no_start(log.as_path());
    let snapshot: Vec<u8> = snapshot_listing(entries.as_slice());
    let printed: Vec<u8> = git_listing(linked.as_path());
    assert!(
        snapshot == printed,
        "the snapshot of a linked worktree is not what Git prints there\n  snapshot: {}\n  Git:      {}",
        rendered(snapshot.as_slice()),
        rendered(printed.as_slice())
    );
    remove(root.as_path());
}

/// What:
///  A symbolic ref whose chain ends at no ref,
///  or in a loop,
///  is not in the snapshot,
///  because Git does not print it either.
/// Why:
///   Git prints a symbolic ref only when its chain ends at a ref that exists;
/// a dangling or looping one is dropped by `for-each-ref`,
///  and the shadow's Git would drop it too,
/// so a snapshot that holds it answers a name Git refuses.
/// A symbolic ref whose target is a *direct* ref with a missing object IS printed by both,
/// and that case is in the same control,
/// so the rule cannot be "drop every symbolic ref".
#[cfg(unix)]
#[test]
fn a_symbolic_ref_that_ends_nowhere_is_not_in_the_snapshot() {
    let root: PathBuf = fixture("shadow-refs-symbolic-ending-nowhere");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = head_of(repo.as_path());
    write_loose(repo.as_path(), "refs/heads/dangling", "ref: refs/heads/nope\n");
    write_loose(repo.as_path(), "refs/heads/cycle-a", "ref: refs/heads/cycle-b\n");
    write_loose(repo.as_path(), "refs/heads/cycle-b", "ref: refs/heads/cycle-a\n");
    write_loose(repo.as_path(), "refs/heads/ghost", "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n");
    write_loose(repo.as_path(), "refs/heads/alias-to-ghost", "ref: refs/heads/ghost\n");
    git(repo.as_path(), &["symbolic-ref", "refs/heads/sound", "refs/heads/main"]);
    let sources: SnapshotSources = sources_of(repo.as_path(), RefFormat::Files);
    let recording: PathBuf = root.join("recording");
    std::fs::create_dir_all(recording.as_path()).expect("a recording directory");
    let (context, log) = recording_context(recording.as_path());
    let entries: Vec<RefSnapshotEntry> = list_real_refs_from(&context, repo.as_path(), &sources).expect("the snapshot");
    assert_no_start(log.as_path());
    let listing: Vec<u8> = snapshot_listing(entries.as_slice());
    let printed: Vec<u8> = git_listing(repo.as_path());
    let shown: String = rendered(listing.as_slice());
    assert!(
        listing == printed,
        "the snapshot holds a symbolic ref Git does not print, or drops one it does\n  snapshot: {shown}\n  Git:      {}",
        rendered(printed.as_slice())
    );
    // The two spellings of the same rule: the unresolvable ones are gone, the resolvable one is there with its object.
    assert!(!shown.contains("refs/heads/dangling"), "{shown}");
    assert!(!shown.contains("refs/heads/cycle-a"), "{shown}");
    assert!(shown.contains("refs/heads/alias-to-ghost | deadbeefdeadbeefdeadbeefdeadbeefdeadbeef | refs/heads/ghost"), "{shown}");
    assert!(shown.contains(format!("refs/heads/sound | {head} | refs/heads/main").as_str()), "{shown}");
    remove(root.as_path());
}

/// How many refs the cost control's store holds.
///
/// The number is this repository's own ref count at the time of the measurement (2026-11-13:
/// 2,282 records of commits over 1,982 packed and 40 loose refs),
/// written as one number so the two arms are timed on the store the commit path meets here.
const COST_STORE_REFS: usize = 2000;

/// How many rounds each arm of the cost control runs.
///
/// The arms alternate,
/// so both see the same page cache and the same host,
/// and twenty rounds are enough to see the spread of each:
/// the numbers are printed,
/// never asserted,
/// because a machine's speed is not this package's contract.
const COST_ROUNDS: usize = 20;

/// What:
///  The median of a list of spans,
///  for a printed measurement.
/// Why:
///   One round is a machine's luck;
/// the median with its range is what a reader of the report can compare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const median = (spans) => spans.sort((a, b) => a - b)[Math.floor(spans.length / 2)]
/// ```
fn median_of(spans: &mut [std::time::Duration]) -> std::time::Duration {
    spans.sort();
    return spans[spans.len() / 2];
}

/// What:
///  What the reader costs beside the one Git child it replaced,
///  over a store of the size the commit path meets here,
///  printed for a reader of the report.
/// Why:
///  The reason for reading the ref files is that a commit should not start a process over the repository's refs,
/// so the two costs have to be seen beside each other rather than assumed.
///  Nothing here asserts a speed:
/// this host's spread over twenty alternating rounds is wider than the difference between the arms when it is busy,
///  so a wall-clock assertion would fail on a loaded host and pass on an idle one,
/// and a machine's speed is not this package's contract.
/// What the control does hold is that both arms answer the same store identically.
#[cfg(unix)]
#[test]
fn the_cost_of_both_arms_is_printed_over_a_store_of_the_repository_s_size() {
    let root: PathBuf = fixture("shadow-refs-cost-of-both-arms");
    let repo: PathBuf = store_of_kind(root.as_path(), "cost");
    let head: String = head_of(repo.as_path());
    // A packed store of the measured size, written whole: one file read is what the reader does, and what Git does too.
    let mut packed: String = String::from("# pack-refs with: sorted \n");
    for index in 0..COST_STORE_REFS {
        packed.push_str(format!("{head} refs/heads/counted-{index:05}\n").as_str());
    }
    std::fs::write(repo.join(".git").join("packed-refs"), packed.as_bytes()).expect("a packed store");
    let sources: SnapshotSources = sources_of(repo.as_path(), RefFormat::Files);
    let context: crate::transaction_git::GitContext = git_context();
    let mut reader_spans: Vec<std::time::Duration> = Vec::with_capacity(COST_ROUNDS);
    let mut child_spans: Vec<std::time::Duration> = Vec::with_capacity(COST_ROUNDS);
    for _round in 0..COST_ROUNDS {
        let reader_began: std::time::Instant = std::time::Instant::now();
        let entries: Vec<RefSnapshotEntry> = list_real_refs_from(&context, repo.as_path(), &sources).expect("the reader's answer");
        reader_spans.push(reader_began.elapsed());
        let child_began: std::time::Instant = std::time::Instant::now();
        let printed: std::process::Output = git_output(repo.as_path(), &["for-each-ref", LISTING_FORMAT]);
        child_spans.push(child_began.elapsed());
        assert!(printed.status.success(), "real Git refused the listing");
        assert_eq!(snapshot_listing(entries.as_slice()), printed.stdout, "the two arms answer the same store differently");
    }
    let reader_median: std::time::Duration = median_of(reader_spans.as_mut_slice());
    let child_median: std::time::Duration = median_of(child_spans.as_mut_slice());
    let reader_least: std::time::Duration = *reader_spans.first().expect("rounds");
    let reader_most: std::time::Duration = *reader_spans.last().expect("rounds");
    let child_least: std::time::Duration = *child_spans.first().expect("rounds");
    let child_most: std::time::Duration = *child_spans.last().expect("rounds");
    println!(
        "{} refs, {} rounds each, alternating: the reader took {} µs at the median ({} to {} µs), one `git for-each-ref` child took {} µs at the median ({} to {} µs)",
        COST_STORE_REFS,
        COST_ROUNDS,
        reader_median.as_micros(),
        reader_least.as_micros(),
        reader_most.as_micros(),
        child_median.as_micros(),
        child_least.as_micros(),
        child_most.as_micros()
    );
    remove(root.as_path());
}

/// How many names one chain of the ordering control is spelled with.
///
/// Five links plus the direct ref they end at,
/// which is the shortest chain Git's own depth bound can cut inside:
/// a listed name is the first of its own chain's levels,
/// so four links are printed and a fifth is not.
const ORDERED_CHAIN_NAMES: usize = 6;

/// How many orders of those names one store of the ordering control holds.
///
/// The chains of one store never share a name,
/// so each is answered on its own,
/// and Git is asked once for the whole store instead of once per order:
/// 720 orders at about four milliseconds a question would be three seconds of a control,
/// and 36 questions are a fifth of a second.
const ORDERS_PER_STORE: usize = 20;

/// What:
///  The next order of a list of indices,
///  the way a person counting would find them,
///  or `false` when the last order has been handed out.
/// Why:
///   A control that must see every order of a chain's names needs each of them once,
/// and next-permutation does that without keeping a table of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nextOrder = (order) => { let i = order.length - 2; while (i >= 0 && order[i] >= order[i + 1]) i--; if (i < 0) return false; let j = order.length - 1; while (order[j] <= order[i]) j--; [order[i], order[j]] = [order[j], order[i]]; order.slice(i + 1).reverse(); return true }
/// ```
fn next_order(order: &mut [usize]) -> bool {
    let Some(pivot_at) = (0..order.len().saturating_sub(1)).rev().find(|at| { return order[*at] < order[at + 1]; }) else {
        return false;
    };
    let Some(swap_at) = (pivot_at + 1..order.len()).rev().find(|at| { return order[*at] > order[pivot_at]; }) else {
        return false;
    };
    order.swap(pivot_at, swap_at);
    order[pivot_at + 1..].reverse();
    return true;
}

/// What:
///  Writes one chain of `ORDERED_CHAIN_NAMES` names in the order `order` gives,
///  under `refs/heads/<prefix>/`,
///  its last name a direct ref at `oid`.
/// Why:
///   The order of a chain's names is the whole variable of the control that follows,
/// so the writer takes it as a parameter and names nothing else:
/// a chain never shares a name with another,
///  so one store can hold many orders of it side by side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeOrderedChain = (repo, prefix, order, oid) => order.forEach((drawn, at) => writeLoose(repo, `refs/heads/${prefix}/p${drawn}`, at + 1 === order.length ? `${oid}\n` : `ref: refs/heads/${prefix}/p${order[at + 1]}\n`))
/// ```
fn write_ordered_chain(repo: &Path, prefix: &str, order: &[usize], oid: &str) {
    for (at, drawn) in order.iter().enumerate() {
        let value: String = match order.get(at + 1) {
            Some(next) => format!("ref: refs/heads/{prefix}/p{next}\n"),
            None => format!("{oid}\n"),
        };
        write_loose(repo, format!("refs/heads/{prefix}/p{drawn}").as_str(), value.as_str());
    }
}

/// What:
///  A symbolic chain is read as Git reads it in every order of the chain's own names.
/// Why:
///  Git counts a chain's depth from the targets as they are stored,
/// so one name's answer never depends on which other name was answered first.
/// The reader once rewrote each entry in place as it resolved it,
///  so a chain that passed through a shorter chain sorting before it was counted shorter than it is:
/// `zz` pointing at `aa`,
///  with `aa` already collapsed to the end of its own chain,
/// reached that end in two steps instead of five,
/// and Git's listing drops `zz` while the snapshot kept it.
///  The fixture is a chain of six names whose last is a direct ref,
/// so the source is five links from it and Git drops the source;
///  every one of the 720 orders of those six names is spelled in one store,
/// and Git answers each store the snapshot must match byte for byte.
#[cfg(unix)]
#[test]
fn a_symbolic_chain_is_read_as_git_reads_it_in_every_order_of_its_names() {
    let root: PathBuf = fixture("shadow-refs-every-order-of-a-chain");
    let repo: PathBuf = store_of_kind(root.as_path(), "orders");
    let head: String = head_of(repo.as_path());
    let sources: SnapshotSources = sources_of(repo.as_path(), RefFormat::Files);
    let recording: PathBuf = root.join("recording");
    std::fs::create_dir_all(recording.as_path()).expect("a recording directory");
    let (context, log) = recording_context(recording.as_path());
    let mut order: Vec<usize> = (0..ORDERED_CHAIN_NAMES).collect();
    let mut orders: usize = 0;
    let mut stores: usize = 0;
    loop {
        // One store holds `ORDERS_PER_STORE` orders, each chain with names of its own.
        let mut drawn: Vec<Vec<usize>> = Vec::with_capacity(ORDERS_PER_STORE);
        loop {
            drawn.push(order.clone());
            orders = orders.saturating_add(1);
            if drawn.len() >= ORDERS_PER_STORE || !next_order(order.as_mut_slice()) {
                break;
            }
        }
        // The inner loop leaves `order` on the last permutation it took, so the next store starts at the following one.
        stores = stores.saturating_add(1);
        for (at, chain) in drawn.iter().enumerate() {
            write_ordered_chain(repo.as_path(), format!("s{stores}c{at}").as_str(), chain.as_slice(), head.as_str());
        }
        let entries: Vec<RefSnapshotEntry> = list_real_refs_from(&context, repo.as_path(), &sources).expect("the snapshot");
        let snapshot: Vec<u8> = snapshot_listing(entries.as_slice());
        let printed: Vec<u8> = git_listing(repo.as_path());
        assert!(
            snapshot == printed,
            "a store of {} orders reads differently from Git: {drawn:?}\n  snapshot: {}\n  Git:      {}",
            drawn.len(),
            rendered(snapshot.as_slice()),
            rendered(printed.as_slice())
        );
        if !next_order(order.as_mut_slice()) {
            break;
        }
    }
    assert_eq!(orders, 720, "every order of six names was seen");
    assert_eq!(stores, 36, "and Git was asked once per store of {ORDERS_PER_STORE}");
    assert_no_start(log.as_path());
    remove(root.as_path());
}

/// The seed the drawn stores start from.
///
/// A fixed seed is what makes a failure reproducible:
/// the control draws the same stores every run,
///  and a store that shows a divergence can be built again from its number.
/// The number itself is arbitrary and carries nothing.
const DRAWN_STORES_SEED: u64 = 0x2026_1113_1a2b_3c4d;

/// How many stores the seeded control draws and asks Git about.
///
/// Each store is thirteen refs at most and one `git for-each-ref`,
/// so forty stores are about a quarter of a second of a control:
/// enough draws to cover chains of every depth in many name orders,
/// and few enough that a suite of thousands of controls is not slowed by it.
const DRAWN_STORES: usize = 40;

/// How many names the seeded control may spend on one store.
const DRAWN_NAMES: usize = 13;

/// What:
///  The next number of a seeded draw.
/// Why:
///   The stores have to be reproducible from their seed and independent of the machine,
/// so the draw is written here rather than taken from a library's generator.
///  This is xorshift64*, chosen for being four lines a reader can check:
/// every seed but zero reaches every value,
///  and nothing here is a cryptographic claim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const next = (state) => { let x = state; x ^= x >> 12n; x ^= (x << 25n) & MASK; x ^= x >> 27n; return [x, (x * 0x2545F4914F6CDD1Dn) & MASK] }
/// ```
fn next_draw(state: &mut u64) -> u64 {
    let mut value: u64 = *state;
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    *state = value;
    return value.wrapping_mul(0x2545_F491_4F6C_DD1D);
}

/// What:
///  A number from `0` up to but not including `bound`,
///  drawn from `state`.
/// Why:
///   Every choice of the seeded control is one of a few alternatives,
/// and the modulo is taken over a bound no machine makes unsafe here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const below = (state, bound) => Number(next(state) % BigInt(bound))
/// ```
fn below(state: &mut u64, bound: usize) -> usize {
    let bound: u64 = u64::try_from(bound).expect("a small bound");
    return usize::try_from(next_draw(state) % bound).expect("a small draw");
}

/// What:
///  One store the seeded control draws:
///  its name order and the chain, plus the names that are not in it.
/// Why:
///   The draw's shape is what the control then builds and asks Git about,
/// so a failure can be read as the numbers it was drawn from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DrawnStore = { order: usize[]; chain: usize; extras: ('direct' | 'dangling' | 'cycle')[] }
/// ```
#[derive(Debug)]
struct DrawnStore {
    /// The order the chain's names appear in,
    ///  as indices into the store's sorted name list.
    order: Vec<usize>,
    /// The other names' values.
    extras: Vec<DrawnExtra>,
}

/// What one name outside the chain holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DrawnExtra {
    /// A direct ref at the repository's head.
    Direct,
    /// A symbolic ref to a name no ref has.
    Dangling,
    /// A symbolic ref to itself,
    ///  which is a chain that never ends.
    Cycle,
}

/// What:
///  Draws one store from `state`.
/// Why:
///  The control's breadth is this function:
/// it draws how many names there are,
/// how long the chain is,
///  the order the chain's names sort in,
/// and what the names outside it hold,
///  so chains inside and beyond Git's depth arrive in orders no fixture names by hand.
///  A chain longer than Git's depth whose prefix sorts before it is exactly the store
/// the ordering control proves,
///  and drawing it here is what keeps a relation between names from going untested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const draw = (state) => { const names = 2 + below(state, DRAWN_NAMES - 1); const chain = 2 + below(state, names - 1); const order = shuffle(range(names), state).slice(0, chain); const extras = ...; return { order, extras } }
/// ```
fn draw_store(state: &mut u64) -> DrawnStore {
    let names: usize = 2usize.saturating_add(below(state, DRAWN_NAMES.saturating_sub(1)));
    // A chain is at least two names and at most all but none of them, so a store always has a direct ref at its end.
    let chain: usize = 2usize.saturating_add(below(state, names.saturating_sub(1)));
    // The chain's names are drawn without replacement, so its order among them is whatever the draw produced.
    let mut spare: Vec<usize> = (0..names).collect();
    let mut order: Vec<usize> = Vec::with_capacity(chain);
    for _step in 0..chain {
        let at: usize = below(state, spare.len());
        order.push(spare[at]);
        spare.remove(at);
    }
    let mut extras: Vec<DrawnExtra> = Vec::with_capacity(spare.len());
    for _name in spare {
        extras.push(match below(state, 3) {
            0 => DrawnExtra::Direct,
            1 => DrawnExtra::Dangling,
            _ => DrawnExtra::Cycle,
        });
    }
    return DrawnStore { order, extras };
}

/// What:
///  Writes one drawn store into the repository,
///  under a prefix of its own.
/// Why:
///   The store's names are its own, so the control can put many draws in one repository,
/// and each draw's chain is written exactly as drawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeDrawn = (repo, prefix, store, oid) => { writeOrderedChain(repo, prefix, store.order, oid); store.extras.forEach((extra, at) => writeLoose(repo, `refs/heads/${prefix}/x${at}`, extra === 'direct' ? `${oid}\n` : extra === 'dangling' ? `ref: refs/heads/${prefix}/absent${at}\n` : `ref: refs/heads/${prefix}/x${at}\n`)) }
/// ```
fn write_drawn_store(repo: &Path, prefix: &str, store: &DrawnStore, oid: &str) {
    write_ordered_chain(repo, prefix, store.order.as_slice(), oid);
    for (at, extra) in store.extras.iter().enumerate() {
        let value: String = match extra {
            DrawnExtra::Direct => format!("{oid}\n"),
            DrawnExtra::Dangling => format!("ref: refs/heads/{prefix}/absent{at}\n"),
            DrawnExtra::Cycle => format!("ref: refs/heads/{prefix}/x{at}\n"),
        };
        write_loose(repo, format!("refs/heads/{prefix}/x{at}").as_str(), value.as_str());
    }
}

/// What:
///  Stores drawn from a seed are read as Git reads them.
/// Why:
///  The guard of this reader was written from fixtures named by hand,
/// and two reviews in a row found a store those fixtures never built:
/// the order of a chain's names was the second,
///  and no hand-named fixture can promise it named the orders that matter.
/// A draw that varies the count of names,
/// the depth of the chain,
///  the order its names sort in,
/// and what the other names hold is what turns "the fixtures happened to agree" into
///  "these stores agree, and the seed says which".
///  Git answers every store, and the snapshot must be that answer byte for byte,
/// or decline and let Git answer.
///  A seed whose draw finds a divergence fails the control with the store printed,
/// and the same store is drawn again on the next run.
#[cfg(unix)]
#[test]
fn stores_drawn_from_a_seed_are_read_as_git_reads_them() {
    let root: PathBuf = fixture("shadow-refs-drawn-stores");
    let repo: PathBuf = store_of_kind(root.as_path(), "drawn");
    let head: String = head_of(repo.as_path());
    let sources: SnapshotSources = sources_of(repo.as_path(), RefFormat::Files);
    let recording: PathBuf = root.join("recording");
    std::fs::create_dir_all(recording.as_path()).expect("a recording directory");
    let (context, log) = recording_context(recording.as_path());
    let mut state: u64 = DRAWN_STORES_SEED;
    for drawn in 0..DRAWN_STORES {
        let store: DrawnStore = draw_store(&mut state);
        let prefix: String = format!("seed{drawn}");
        write_drawn_store(repo.as_path(), prefix.as_str(), &store, head.as_str());
        let entries: Vec<RefSnapshotEntry> = list_real_refs_from(&context, repo.as_path(), &sources).expect("the snapshot");
        let snapshot: Vec<u8> = snapshot_listing(entries.as_slice());
        let printed: Vec<u8> = git_listing(repo.as_path());
        assert!(
            snapshot == printed,
            "store {drawn} of seed {DRAWN_STORES_SEED:#x} reads differently from Git: {store:?}\n  snapshot: {}\n  Git:      {}",
            rendered(snapshot.as_slice()),
            rendered(printed.as_slice())
        );
    }
    assert_no_start(log.as_path());
    remove(root.as_path());
}
