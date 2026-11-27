//! What:
//!  The shapes a merge's write meets at one path:
//!  a symbolic link replaced by a file,
//!  executable or not,
//!  a nested directory replaced by a file,
//!  a directory the merge does not reach left empty,
//!  and a branch moved under a checkout that was partly written.
//! Why:
//!  A link's identity is the text of its target and never the bytes it points at,
//!  and the mode Git checks out is the mode the install has to leave.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(lstat(path).mode).toBe(theModeGitChecksOut(world, path));
//! ```

use super::super::*;
use super::fixtures::{
    World,
    aside_names,
    dead_owner,
    deferred_record_of,
    nested_directory_to_file_world,
    place,
    record_of,
    symlink_to_file_world,
    symlink_to_file_world_with,
    world,
};
use super::readings::{
    snapshot,
    the_mode_git_checks_out,
};
use crate::test_support::{
    git,
    git_text,
    remove,
};
use std::path::PathBuf;

use crate::merge_isolation_recovery_records::recover_interrupted_merges;
/// What:
///  A saved merge that replaces an existing symbolic link finishes.
/// Why:
///  The take hashed the path it moved aside with `git hash-object`,
/// which follows a link,
/// so the bytes it compared
///  were the link's target's and the link's own object never matched:
/// the link was put back,
/// the finish was refused,
///  and every retry met the same mismatch,
/// so a saved merge the command had already reported as saved never landed
///  (a cross-vendor review of the ninth round,
/// its sixth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = symlinkToFileWorld(); const record = deferredRecord(); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed'); expect(read('link')).toEqual("link as a file\n"); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_saved_merge_that_replaces_a_symlink_finishes() {
    let world: World = symlink_to_file_world("merge-recovery-symlink-replaced");
    let record: PathBuf = deferred_record_of(
        &world,
        "51515151-0000-4000-8000-000000000051",
        "refs/cli-git/kept/51",
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the merge lands: {text}"
    );
    assert!(
        !record.exists(),
        "the record only goes once the merge landed"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid,
        "the branch names the merge"
    );
    assert_eq!(
        std::fs::read(world.repo.join("link")).expect("the merge's file"),
        b"link as a file\n",
        "the merge's file is in place of the link"
    );
    assert_eq!(
        std::fs::read(world.repo.join("target.txt")).expect("the link's target"),
        b"payload\n",
        "the file the link pointed at is untouched"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing is left at a private name"
    );
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &["status", "--porcelain=v1", "--untracked-files=all"]
        ),
        ""
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge that replaces a symbolic link with a plain file leaves the file the mode Git's entry implies.
/// Why:
///  Installing a saved merge gives each file back the permission bits it had when the take read it
///  (a Kimi K3.1 Pro review of `1a98764d1f`,
/// its fifth finding).
/// The entry a link change takes aside is the link itself,
/// whose own permission bits are the platform's `0777` and say
///  nothing of the link,
/// so the mask replaced `0644` with `0666`:
/// a world-writable file the person never set,
/// which Git
///  would not have written either
///  (a Qwen review of `72013e1710`,
/// its first finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = symlinkToFileWorld('merge-recovery-symlink-mode'); record(sideRef); await recover(...); expect(mode('link')).toBe(theModeGitChecksOut(world, 'link'));
/// ```
#[cfg(unix)]
#[test]
fn a_saved_merge_that_replaces_a_symlink_leaves_the_mode_git_checks_out() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = symlink_to_file_world("merge-recovery-symlink-mode");
    let record: PathBuf = deferred_record_of(
        &world,
        "53535353-0000-4000-8000-000000000053",
        "refs/cli-git/kept/53",
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the merge lands: {text}"
    );
    assert!(
        !record.exists(),
        "the record only goes once the merge landed"
    );
    let link: PathBuf = world.repo.join("link");
    let written: u32 = std::fs::symlink_metadata(link.as_path())
        .expect("the merge's file")
        .permissions()
        .mode()
        & 0o7777;
    let git_writes: u32 = the_mode_git_checks_out(&world, "link");
    assert_eq!(
        written, git_writes,
        "the merge's file was given {written:o} where Git's entry writes {git_writes:o}, and the entry the merge replaced was a link, whose own mode is not the person's"
    );
    remove(world.parent.as_path());
}

/// What:
///  That saved merge leaves an executable file both the executable bits Git's entry decided and no others.
/// Why:
///  The mask takes the bits Git does not record from the entry it replaces and the executable bits from the checkout,
/// so the executable entry is the other half of the same decision:
/// a link's `0777` turned Git's `0755` into `0777`,
///  three bits more than the entry holds (a Qwen review of `72013e1710`,
/// its first finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = symlinkToFileWorldWith('merge-recovery-symlink-executable-mode', true); record(sideRef); await recover(...); expect(mode('link') & 0o111).not.toBe(0); expect(mode('link')).toBe(theModeGitChecksOut(world, 'link'));
/// ```
#[cfg(unix)]
#[test]
fn a_saved_merge_that_replaces_a_symlink_leaves_an_executable_file_the_mode_git_checks_out() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = symlink_to_file_world_with("merge-recovery-symlink-executable-mode", true);
    let record: PathBuf = deferred_record_of(
        &world,
        "54545454-0000-4000-8000-000000000054",
        "refs/cli-git/kept/54",
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the merge lands: {text}"
    );
    assert!(
        !record.exists(),
        "the record only goes once the merge landed"
    );
    let link: PathBuf = world.repo.join("link");
    let written: u32 = std::fs::symlink_metadata(link.as_path())
        .expect("the merge's file")
        .permissions()
        .mode()
        & 0o7777;
    let git_writes: u32 = the_mode_git_checks_out(&world, "link");
    assert_ne!(
        written & 0o111,
        0,
        "the executable bits Git's entry decided are kept: {written:o}"
    );
    assert_eq!(
        written, git_writes,
        "the merge's executable file was given {written:o} where Git's entry writes {git_writes:o}, and the entry the merge replaced was a link, whose own mode is not the person's"
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge that replaces a directory holding only nested tracked files it deletes finishes.
/// Why:
///  The directory was judged by its immediate children compared with the tree diff,
/// and that diff names the
///  deleted file `dir/sub/child.txt` and never `dir/sub`,
/// so a directory of the branch's own files was read as someone
///  else's and the saved merge could never finish
///  (a cross-vendor review of the ninth round,
/// its eighth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = nestedDirectoryToFileWorld(); const record = deferredRecord(); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed'); expect(read('dir')).toEqual("the merge's file\n"); expect(exists('dir/sub')).toBe(false);
/// ```
#[test]
fn a_saved_merge_that_replaces_a_nested_directory_finishes() {
    let world: World = nested_directory_to_file_world("merge-recovery-nested-directory");
    let record: PathBuf = deferred_record_of(
        &world,
        "52525252-0000-4000-8000-000000000052",
        "refs/cli-git/kept/52",
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the merge lands: {text}"
    );
    assert!(
        !record.exists(),
        "the record only goes once the merge landed"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid,
        "the branch names the merge"
    );
    assert_eq!(
        std::fs::read(world.repo.join("dir")).expect("the merge's file"),
        b"the merge's file\n",
        "the merge's file stands where the branch's directory stood"
    );
    assert!(
        !world.repo.join("dir/sub").exists(),
        "the emptied directories of the branch's files are pruned"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing is left at a private name"
    );
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &["status", "--porcelain=v1", "--untracked-files=all"]
        ),
        ""
    );
    remove(world.parent.as_path());
}

/// What:
///  An emptied directory the merge's own pruning never reaches stops the finish before anything is written.
/// Why:
///  Judging a directory through its whole subtree must not accept a directory with nothing in it:
/// the pruning walks up
///  from a deleted path,
/// so an emptied directory with no such path below it keeps the checkout from holding the
///  merge's file,
/// and the merge would be half installed before the failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = nestedDirectoryToFileWorld(); mkdir('dir/notes'); const record = deferredRecord(); const before = snapshot(world); const outcomes = await recover(...); expect(outcomes[0].action).toBe('deferred-refused'); expect(snapshot(world)).toEqual(before);
/// ```
#[test]
fn an_emptied_directory_the_merge_does_not_reach_refuses_before_anything_is_written() {
    let world: World = nested_directory_to_file_world("merge-recovery-nested-directory-emptied");
    std::fs::create_dir(world.repo.join("dir/notes")).expect("the person's empty directory");
    let record: PathBuf = deferred_record_of(
        &world,
        "53535353-0000-4000-8000-000000000053",
        "refs/cli-git/kept/53",
    );
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("a saved merge that wrote nothing lets the command go on");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::DeferredRefused,
        "it is refused, not installed"
    );
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(text.contains("dir"), "the path is named: {text}");
    assert_eq!(
        snapshot(&world),
        before,
        "nothing was written and nothing was taken"
    );
    assert!(
        world.repo.join("dir/sub/child.txt").is_file(),
        "the branch's nested file is where it was"
    );
    assert!(
        world.repo.join("dir/notes").is_dir(),
        "the person's directory is where it was"
    );
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A checkout a killed merge wrote one file of,
/// whose branch another command then advanced,
///  is set back to the moved
///  head,
///  and the notice that says the files were put back is true.
/// Why:
///  An install was read as every path holding the merge,
/// so a checkout holding the merge's file for one path and the
///  branch's content for the others looked untouched:
/// the record was removed without putting anything back,
/// the
///  notice claimed the files were put back and the checkout matched the branch,
/// and the person's `git status` showed the
///  abandoned merge's file as their own edit
///  (a cross-vendor review of the ninth round,
/// its fourth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = world(); write('f.txt', mergeContent); git commit -m elsewhere -- h.txt; const record = deferredRecord(); const outcomes = await recover(...); expect(outcomes[0].action).toBe('branch-moved'); expect(read('f.txt')).toEqual(base); expect(status()).toEqual('');
/// ```
#[test]
fn a_partly_written_checkout_whose_branch_moved_puts_the_written_paths_back() {
    let world: World = world("merge-recovery-moved-after-partial-write");
    // Git wrote the merge's `f.txt` and was killed before the other paths or the index.
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    // Another command advances the branch by a commit that leaves both paths as the old side has them.
    std::fs::write(world.repo.join("h.txt"), b"h.txt elsewhere\n").expect("file");
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=elsewhere", "--", "h.txt"],
    );
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let record: PathBuf = record_of(
        &world,
        dead_owner("54545454-0000-4000-8000-000000000054"),
        Some("refs/heads/main"),
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("recovery");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::BranchMoved,
        "the moved branch settles the merge: {text}"
    );
    assert!(
        !record.exists(),
        "the record only goes once the checkout is the moved head's"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        moved,
        "the branch is the other command's"
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("file"),
        b"f.txt base\n",
        "the merge's file is put back"
    );
    assert!(
        !world.repo.join("dir").exists(),
        "nothing the merge adds is left"
    );
    assert_eq!(
        std::fs::read(world.repo.join("gone.txt")).expect("file"),
        b"gone.txt base\n",
        "nothing the merge deletes is left deleted"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing is left at a private name"
    );
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &["status", "--porcelain=v1", "--untracked-files=all"]
        ),
        "",
        "the checkout matches the branch, as the notice says"
    );
    remove(world.parent.as_path());
}
