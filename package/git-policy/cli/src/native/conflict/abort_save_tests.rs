//! What:
//!  What an abort or a skip copies before it runs:
//!  the worktree files the listing named and the content that stands only in the index.
//! Why:
//!  The promise is that the copy holds what Git is about to discard,
//!  including the one kind of content no worktree file holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const saved = saveBeforeTheCommand({ root, commonDir, git, paths, stagedPaths, milliseconds });
//! expect(saved.copies.map((copy) => copy.path)).toEqual(['a.txt', 'new.txt (index-stage-0)']);
//! ```

use super::{STAGED_FOLDER, save_before_the_command, save_before_the_command_watching, save_staged_content};
use crate::command_test_support::{REAL_GIT, fixture, git, remove, repository};
use crate::conflict_side_source::{FailureKind, SaveFailure};
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The Git context of a fixture repository, with its global option naming the repository.
fn context(repo: &Path) -> GitContext {
    return GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay: Vec::new(),
        global_prefix: vec![OsString::from("-C"), repo.as_os_str().to_os_string()],
        common_dir: Some(repo.join(".git")),
    };
}

/// The bytes of every copy below `directory`, by their path below it.
fn copies_below(directory: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found: Vec<(String, Vec<u8>)> = Vec::<(String, Vec<u8>)>::new();
    let mut pending: Vec<PathBuf> = vec![directory.to_path_buf()];
    while let Some(current) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(current.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            found.push((
                path.strip_prefix(directory).unwrap_or(path.as_path()).display().to_string(),
                std::fs::read(path.as_path()).unwrap_or_default(),
            ));
        }
    }
    found.sort();
    return found;
}

/// A file that grows while it is being copied is not a reason to refuse the command:
///  the copy stops at the size it opened,
///  the grown source is a source that changed,
///  and it is copied again within the same bound the last comparison uses.
#[test]
fn a_file_that_grows_while_it_is_copied_is_saved_again_instead_of_refused() {
    let root: PathBuf = fixture("abort-save-growing");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli").as_path()).expect("the copy tree");
    let path: PathBuf = root.join("big.bin");
    // Larger than one chunk of the copy loop, so the change lands inside the copy and not after it.
    std::fs::write(path.as_path(), vec![b'x'; 300_000]).expect("the file");
    let git: GitContext = context(root.as_path());
    // A writer through the copy's own pause: it adds to the file once, after the first chunk.
    let mut wrote: bool = false;
    let mut grow_once = || {
        if wrote {
            return;
        }
        wrote = true;
        let mut file: std::fs::File = std::fs::OpenOptions::new().append(true).open(path.as_path()).expect("the growing file");
        std::io::Write::write_all(&mut file, b"grown while the wrapper copied it\n").expect("the growth");
    };
    let saved = save_before_the_command_watching(root.as_path(), common.as_path(), &git, &[b"big.bin".to_vec()], &[], 9, &mut grow_once)
        .expect("the command is not refused over a file that grew");
    let directory: &Path = saved.directory.as_path();
    let copies: Vec<(String, Vec<u8>)> = copies_below(directory);
    // The copy that met the writer stopped at the size the file had when it was opened and was removed as torn;
    //  the only copy is the one the second round made, which holds the growth whole.
    assert_eq!(copies.len(), 1, "the grown source is copied again beside the first folder: {copies:?}");
    assert_eq!(copies[0].0, "again-1/big.bin", "{copies:?}");
    assert!(copies[0].1.len() > 300_000, "the second copy holds the growth: {}", copies[0].1.len());
    assert_eq!(saved.copies.len(), 1, "the notice names the copy that holds the whole file: {:?}", saved.copies);
    assert!(saved.plan.is_some(), "the comparison the command owes is still owed");
    remove(&root);
}

/// Content that stands only in the index is copied from the index itself,
///  into a folder that names the stage,
///  and the copy holds the bytes the index held after the worktree file went.
#[test]
fn the_staged_content_is_copied_beside_the_worktree_copies() {
    let fixture_root: PathBuf = fixture("abort-save-staged");
    let repo: PathBuf = repository(fixture_root.as_path(), "repo");
    std::fs::write(repo.join("new.txt"), b"content that exists only in the index\n").expect("the staged file");
    git(repo.as_path(), &["add", "--", "new.txt"]);
    std::fs::remove_file(repo.join("new.txt")).expect("the worktree file goes");
    let git_context: GitContext = context(repo.as_path());
    let directory: PathBuf = repo.join(".git/monochromatic-git-policy-cli/discarded/2026-11-11T00-00-00");
    let copies = save_staged_content(&git_context, repo.join(".git").as_path(), directory.as_path(), &[b"new.txt".to_vec()])
        .expect("the index writes the content");
    assert_eq!(copies.len(), 1, "{copies:?}");
    assert_eq!(copies[0].path, format!("new.txt ({STAGED_FOLDER})"), "the copy names the path and the stage: {copies:?}");
    let place: &Path = copies[0].saved_to.as_deref().expect("a place");
    assert!(place.ends_with(format!("{STAGED_FOLDER}/new.txt")), "{}", place.display());
    assert_eq!(std::fs::read(place).expect("the copy"), b"content that exists only in the index\n");
    remove(&fixture_root);
}

/// A path the index does not hold cannot be copied, and the refusal names it.
#[test]
fn a_staged_path_the_index_cannot_write_refuses_the_save() {
    let fixture_root: PathBuf = fixture("abort-save-staged-missing");
    let repo: PathBuf = repository(fixture_root.as_path(), "repo");
    let git_context: GitContext = context(repo.as_path());
    let directory: PathBuf = repo.join(".git/monochromatic-git-policy-cli/discarded/2026-11-11T00-00-01");
    let failure: SaveFailure = save_staged_content(&git_context, repo.join(".git").as_path(), directory.as_path(), &[b"absent.txt".to_vec()])
        .expect_err("the index holds no such path");
    assert_eq!(failure.kind, FailureKind::Io, "{failure:?}");
    assert!(failure.subject.contains("absent.txt"), "{failure:?}");
    remove(&fixture_root);
}

/// The worktree copy and the index's copy of one path can both be made,
///  and the copies of one command are one folder's worth.
#[test]
fn one_command_copies_its_worktree_files_and_its_staged_content_together() {
    let fixture_root: PathBuf = fixture("abort-save-both");
    let repo: PathBuf = repository(fixture_root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"resolved by hand\n").expect("the hand resolution");
    std::fs::write(repo.join("new.txt"), b"staged and then gone from the worktree\n").expect("the staged file");
    git(repo.as_path(), &["add", "--", "a.txt", "new.txt"]);
    std::fs::remove_file(repo.join("new.txt")).expect("the worktree file goes");
    let git_context: GitContext = context(repo.as_path());
    let saved = save_before_the_command(
        repo.as_path(),
        repo.join(".git").as_path(),
        &git_context,
        &[b"a.txt".to_vec()],
        &[b"new.txt".to_vec()],
        8,
    )
    .expect("both copies are made");
    assert_eq!(saved.copies.len(), 2, "{:?}", saved.copies);
    let directory: &Path = saved.plan.as_ref().expect("a comparison is owed").directory.as_path();
    assert_eq!(
        copies_below(directory),
        vec![
            (String::from("a.txt"), b"resolved by hand\n".to_vec()),
            (format!("{STAGED_FOLDER}/new.txt"), b"staged and then gone from the worktree\n".to_vec()),
        ]
    );
    remove(&fixture_root);
}
