//! What:
//!  The repositories the controls of a merge's recovery build,
//!  the intent records they leave in one,
//!  and the private names those records put beside a path.
//! Why:
//!  Every control below starts from one of these shapes,
//!  and a shape built by hand in each file drifts from what a killed merge leaves.
//!  This file builds only;
//!  what a control reads is `merge_isolation_recovery_test_readings_tests.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const world = world('name'); const record = recordOf(world, deadOwner(id), 'refs/heads/main');
//! ```

use super::super::*;
use crate::merge_isolation_journal::write_merge_intent;
use crate::test_support::{fixture, git, git_context, git_text, repository};
use std::path::{Path, PathBuf};
use crate::transaction_owner::{TransactionOwner, current_transaction_owner};

/// A repository where `main` and `feature` have diverged and the merge commit exists but is not applied.
pub(super) struct World {
    /// The fixture directory removed at the end.
    pub(super) parent: PathBuf,
    /// The worktree.
    pub(super) repo: PathBuf,
    /// The Git directory.
    pub(super) git_dir: PathBuf,
    /// `main` before the merge.
    pub(super) old_oid: String,
    /// The merge commit.
    pub(super) new_oid: String,
}

/// Build the world:
/// `feature` changes `f.txt`,
/// adds `dir/new.txt` and deletes `gone.txt`;
/// `main` changes `h.txt`.
pub(super) fn world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for file in ["u.txt", "f.txt", "gone.txt", "h.txt"] {
        std::fs::write(repo.join(file), format!("{file} base\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--", "u.txt", "f.txt", "gone.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/new.txt"), b"new\n").expect("file");
    git(repo.as_path(), &["rm", "--quiet", "--", "gone.txt"]);
    git(repo.as_path(), &["add", "--", "f.txt", "dir/new.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]);
    let new_oid: String = git_text(
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    );
    let git_dir: PathBuf = repo.join(".git");
    return World { parent, repo, git_dir, old_oid, new_oid };
}

/// The recovery place for a world.
pub(super) fn place(world: &World) -> RecoveryPlace {
    return RecoveryPlace { git: git_context(), cwd: world.repo.clone(), common_dir: world.git_dir.clone(), git_dir: world.git_dir.clone() };
}

/// An owner that is certainly gone:
/// this process's PID with a birth identity no process has.
pub(super) fn dead_owner(id: &str) -> TransactionOwner {
    let mut owner: TransactionOwner = current_transaction_owner(id, "2026-10-29T00:00:00.000Z").expect("owner");
    owner.owner_identity = String::from("linux:1");
    return owner;
}

/// Write the record of a merge of the world,
/// owned by `owner`,
/// and return its file.
pub(super) fn record_of(world: &World, owner: TransactionOwner, target_ref: Option<&str>) -> PathBuf {
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: world.repo.to_path_buf(),
        target_ref: target_ref.map(String::from),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: None,
        carried: None,
    };
    return write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
}

/// What:
///  A repository where the branch holds `dir/child.txt` and the merge deletes it and adds a file named `dir`,
///  with the merge commit ready and nothing of it in the checkout.
/// Why:
///  That shape makes the worktree state of the added path a directory,
/// which is neither side of the change and used to
///  refuse the merge for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(...); write('dir/child.txt'); commit(); /* feature: rm dir/child.txt, write file 'dir' */ const merge = mergeTreeCommit(...);
/// ```
pub(super) fn directory_to_file_world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/child.txt"), b"child base\n").expect("file");
    std::fs::write(repo.join("h.txt"), b"h.txt base\n").expect("file");
    git(repo.as_path(), &["add", "--", "dir/child.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    // `git rm` prunes the emptied directory, so the path is free for a file.
    git(repo.as_path(), &["rm", "--quiet", "--", "dir/child.txt"]);
    std::fs::write(repo.join("dir"), b"the merge's file\n").expect("a file where the directory stood");
    git(repo.as_path(), &["add", "--", "dir"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]);
    let new_oid: String = git_text(
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    );
    let git_dir: PathBuf = repo.join(".git");
    return World { parent, repo, git_dir, old_oid, new_oid };
}

/// What:
///  A repository where the branch holds `dir/sub/child.txt` and the merge deletes it and adds a file named `dir`,
///  with the merge commit ready and nothing of it
///  in the checkout.
/// Why:
///  The tree diff of that merge names the deleted file and no entry for `dir/sub`,
/// so a check of the directory's
///  immediate children needs the whole subtree to judge it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(...); write('dir/sub/child.txt'); commit(); /* feature: rm dir/sub/child.txt, write file 'dir' */ const merge = mergeTreeCommit(...);
/// ```
pub(super) fn nested_directory_to_file_world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::create_dir_all(repo.join("dir/sub")).expect("directory");
    std::fs::write(repo.join("dir/sub/child.txt"), b"child base\n").expect("file");
    std::fs::write(repo.join("h.txt"), b"h.txt base\n").expect("file");
    git(repo.as_path(), &["add", "--", "dir/sub/child.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    // `git rm` prunes the emptied directories, so the path is free for a file.
    git(repo.as_path(), &["rm", "--quiet", "--", "dir/sub/child.txt"]);
    std::fs::write(repo.join("dir"), b"the merge's file\n").expect("a file where the directory stood");
    git(repo.as_path(), &["add", "--", "dir"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]);
    let new_oid: String = git_text(
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    );
    let git_dir: PathBuf = repo.join(".git");
    return World { parent, repo, git_dir, old_oid, new_oid };
}

/// What:
///  A repository whose merge changes exactly one path,
///  `f.txt`,
///  with the merge commit ready and nothing of it in the checkout.
/// Why:
///  With one changed path,
/// every entry matching is what an independent commit of that content leaves,
/// so the
///  control needs a merge that touches one path only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(...); write('f.txt'); write('h.txt'); commit(); /* feature changes f.txt */ const merge = mergeTreeCommit(...);
/// ```
pub(super) fn one_path_merge_world(name: &str) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("f.txt"), b"f.txt base\n").expect("file");
    std::fs::write(repo.join("h.txt"), b"h.txt base\n").expect("file");
    git(repo.as_path(), &["add", "--", "f.txt", "h.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]);
    let new_oid: String = git_text(
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    );
    let git_dir: PathBuf = repo.join(".git");
    return World { parent, repo, git_dir, old_oid, new_oid };
}

/// What:
///  A repository whose branch holds a symbolic link and whose merge replaces it with a regular file,
///  with the merge commit
///  ready and nothing of it in the checkout.
/// Why:
///  A link's identity is the text of its target and never the bytes of the file it points at,
/// so a control needs a
///  path that is a link on the old side and a file on the merged side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(...); write('target.txt', 'payload\n'); symlink('target.txt', 'link'); commit(); /* feature: rm link, write file 'link' */ const merge = mergeTreeCommit(...);
/// ```
pub(super) fn symlink_to_file_world(name: &str) -> World {
    return symlink_to_file_world_with(name, false);
}

/// What:
///  That repository with the merge's file executable when `executable_after` is set,
///  so Git records `100755` instead of `100644`.
/// Why:
///  The install keeps the executable bits of the file Git's entry checked out and gives back every other bit,
///  so the executable entry is the second branch of that mask and a control needs one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = symlinkToFileWorld(name); chmod('link', 0o755); git.add('link'); // feature writes 'link' executable
/// ```
pub(super) fn symlink_to_file_world_with(name: &str, executable_after: bool) -> World {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("target.txt"), b"payload\n").expect("the link's target");
    std::fs::write(repo.join("h.txt"), b"h.txt base\n").expect("file");
    std::os::unix::fs::symlink("target.txt", repo.join("link")).expect("the link");
    git(repo.as_path(), &["add", "--", "target.txt", "h.txt", "link"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::remove_file(repo.join("link")).expect("the link goes");
    std::fs::write(repo.join("link"), b"link as a file\n").expect("a file where the link stood");
    if executable_after {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(repo.join("link"), std::fs::Permissions::from_mode(0o755)).expect("the merge's executable file");
    }
    git(repo.as_path(), &["add", "--", "link"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]);
    let new_oid: String = git_text(
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    );
    let git_dir: PathBuf = repo.join(".git");
    return World { parent, repo, git_dir, old_oid, new_oid };
}

/// What:
///  The record of a merge that was saved on `side_ref` because the index was locked,
///  and the side ref itself.
/// Why:
///  A merge that met a live holder leaves that state,
/// and the controls about a saved merge start there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMergeRecord({ ...intent, sideRef }); await git(['update-ref', '--no-deref', sideRef, newOid]);
/// ```
pub(super) fn deferred_record_of(world: &World, id: &str, side_ref: &str) -> PathBuf {
    git(world.repo.as_path(), &["update-ref", "--no-deref", side_ref, world.new_oid.as_str()]);
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner(id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: Some(String::from(side_ref)),
        carried: None,
    };
    return write_merge_intent(world.git_dir.as_path(), &intent).expect("the record of a saved merge");
}

/// The identifier a control's own take names itself by,
/// which no record of a control's fixture claims unless the control writes one.
pub(super) const A_CONTROLS_MERGE: &str = "17171717-0000-4000-8000-000000000017";

/// The names in `repository` that are the private names a take leaves.
pub(super) fn aside_names(repository: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let listing = std::fs::read_dir(repository).expect("the worktree root");
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(".cli-git-recovery-") {
            found.push(name);
        }
    }
    return found;
}

/// What:
///  The name a killed take leaves a file under,
///  built the way recovery builds it,
///  so a control can make that state without the race.
/// Why:
///  The control reproduces the instant after the rename of a take,
/// which no seam of the recovery reaches otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stranded = `${dirname}/.cli-git-recovery-${basename}-${process.pid}-0-1`;
/// ```
pub(super) fn stranded_name(location: &Path, transaction_id: &str) -> PathBuf {
    return crate::merge_isolation_take_aside_names::aside_name(location, transaction_id);
}

/// What:
///  A private name an earlier release of the wrapper wrote carries no merge key.
/// Why:
///  The name is the only thing a command has to go on,
/// and the earlier form is exactly this one:
///  the key of the path's name,
///  the process,
///  the count and the moment,
/// with no key of a merge between
///  them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${dirname}/.cli-git-recovery-${nameKey(basename)}-${pid}-0-1`
/// ```
pub(super) fn an_earlier_releases_name(location: &Path) -> PathBuf {
    let found: Option<std::ffi::OsString> = location.file_name().map(std::ffi::OsStr::to_os_string);
    let name: String = found.unwrap_or_default().to_string_lossy().into_owned();
    return location.with_file_name(format!(
        ".cli-git-recovery-{}-{}-0-1",
        crate::merge_isolation_take_aside_names::name_key(name.as_str()),
        std::process::id()
    ));
}

/// What:
///  One change the merge does not have,
///  for a path this control made,
///  with the object it should end as.
/// Why:
///  The seam a take that fails needs is a path whose directory refuses the rename,
/// and the merge's own paths are all in
///  the worktree root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const extra = { path: Buffer.from('sealed/zeta.txt'), oldMode: '100644', newMode: '100644', oldOid: old, newOid: next };
/// ```
pub(super) fn extra_change(path: &str, old_mode: &str, new_mode: &str, old_oid: &str, new_oid: &str) -> Change {
    return Change {
        path: path.as_bytes().to_vec(),
        old_mode: String::from(old_mode),
        new_mode: String::from(new_mode),
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
    };
}

/// Write the record of a merge of the world whose paths held the person's own work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMergeIntent(gitDir, { ...record(id), carried: capture });
/// ```
pub(super) fn record_with_a_capture(world: &World, id: &str, capture: &crate::merge_isolation_carry_parts::Capture) -> PathBuf {
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner(id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: None,
        carried: Some(capture.clone()),
    };
    return write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
}

/// What:
///  A merge the world's own steps left half carried,
///  with the person's own edit in `f.txt` captured and the merge already in the branch.
/// Why:
///  The state a kill between Git's update and the carry leaves is the one thing these controls need,
/// and every
///  step of it is a step the merge makes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', `${BRANCH}\n${PERSON_LINE}\n`); const capture = keepCapture(captureWork([f.txt])); record(capture); clear(); readTree(old, new); updateRef(main, new);
/// ```
pub(super) fn half_carried(world: &World, id: &str) -> crate::merge_isolation_carry_parts::Capture {
    let person_lines: &str = "f.txt base\nthe person's line\n";
    std::fs::write(world.repo.join("f.txt"), person_lines).expect("the person's edit");
    let context = crate::test_support::git_context();
    let capture: crate::merge_isolation_carry_parts::Capture = crate::merge_isolation_carry::keep_capture(
        &context,
        world.repo.as_path(),
        &crate::merge_isolation_carry::capture_work(&context, world.repo.as_path(), &[b"f.txt".to_vec()]).expect("the capture"),
    )
    .expect("kept");
    let _record: PathBuf = record_with_a_capture(world, id, &capture);
    // The person's path is put back to the branch, which is what the merge's own clear does before Git's update.
    git(world.repo.as_path(), &["checkout", "--quiet", "--", "f.txt"]);
    git(world.repo.as_path(), &["read-tree", "-u", "-m", world.old_oid.as_str(), world.new_oid.as_str()]);
    git(world.repo.as_path(), &["update-ref", "refs/heads/main", world.new_oid.as_str()]);
    return capture;
}
