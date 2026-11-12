//! What:
//!  The comparison made immediately before Git starts,
//!  and the refusal of a source that will not hold still.
//! Why:
//!  The promise is that the copy holds what Git is about to discard,
//!  and a file written after the copy is the one case the wrapper can still catch;
//!  a file another program keeps writing is the one case it must refuse rather than wait for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const saved = saveBeforeTheCommand({ root, commonDir, git, paths, stagedPaths, milliseconds });
//! writeFileSync(join(root, 'a.txt'), 'later');
//! planTheLastChance(saved.plan);
//! expect(saveAgainBeforeTheCommand()).toContain('second copy');
//! ```

use super::{SAVE_AGAIN_ROUNDS, plan_the_last_chance, refused_save_text, save_again_before_the_command, still_changing_text};
use crate::abort_save::save_before_the_command;
use crate::command_test_support::{REAL_GIT, fixture, remove};
use crate::conflict_side_source::{FailureKind, SaveFailure};
use crate::conflict_sides::SavedSource;
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// The Git context of a fixture repository, with its global option naming the repository.
fn context(repo: &Path) -> GitContext {
    return GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay: Vec::new(),
        global_prefix: vec![std::ffi::OsString::from("-C"), repo.as_os_str().to_os_string()],
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

/// A file written after the copy is copied again at the last moment,
///  into a folder of its own, and the sentence names it.
#[test]
fn a_change_after_the_copy_is_saved_again_before_the_command() {
    let root: PathBuf = fixture("abort-save-again");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli").as_path()).expect("the copy tree");
    std::fs::write(root.join("a.txt"), b"resolved by hand\n").expect("the file");
    let git: GitContext = context(root.as_path());
    let saved = save_before_the_command(root.as_path(), common.as_path(), &git, &[b"a.txt".to_vec()], &[], 1).expect("the first save");
    let plan = saved.plan.clone().expect("a comparison is owed");
    assert_eq!(copies_below(plan.directory.as_path()), vec![(String::from("a.txt"), b"resolved by hand\n".to_vec())]);
    // The edit that lands in the window the first copy cannot see.
    std::fs::write(root.join("a.txt"), b"resolved again, after the copy\n").expect("the later edit");
    plan_the_last_chance(plan);
    let said: Option<String> = save_again_before_the_command().expect("the later edit is copied");
    let said: String = said.expect("a second copy is announced");
    assert!(said.contains("a.txt"), "{said}");
    assert!(said.contains("changed again"), "{said}");
    assert!(said.contains("second copy"), "{said}");
    assert!(said.contains("later-1"), "{said}");
    let directory: &Path = saved.plan.as_ref().expect("a plan").directory.as_path();
    let copies: Vec<(String, Vec<u8>)> = copies_below(directory);
    assert!(
        copies.contains(&(String::from("a.txt"), b"resolved by hand\n".to_vec())),
        "the first copy is still there: {copies:?}"
    );
    assert!(
        copies.contains(&(String::from("later-1/a.txt"), b"resolved again, after the copy\n".to_vec())),
        "the later edit is copied beside the first: {copies:?}"
    );
    assert_eq!(save_again_before_the_command().expect("nothing is pending"), None, "the comparison is made once");
    remove(&root);
}

/// A file that stands still after the copy makes no second copy and says nothing.
#[test]
fn a_source_that_stands_still_makes_no_second_copy() {
    let root: PathBuf = fixture("abort-save-still");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli").as_path()).expect("the copy tree");
    std::fs::write(root.join("a.txt"), b"resolved by hand\n").expect("the file");
    let git: GitContext = context(root.as_path());
    let saved = save_before_the_command(root.as_path(), common.as_path(), &git, &[b"a.txt".to_vec()], &[], 2).expect("the first save");
    plan_the_last_chance(saved.plan.expect("a comparison is owed"));
    assert_eq!(save_again_before_the_command().expect("nothing changed"), None, "no second copy, no sentence");
    remove(&root);
}

/// A file that appears where the listing found nothing is copied too,
///  because the command would discard it as well.
#[test]
fn a_file_that_appears_after_the_copy_is_saved_at_the_last_moment() {
    let root: PathBuf = fixture("abort-save-appeared");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli").as_path()).expect("the copy tree");
    let git: GitContext = context(root.as_path());
    // The listing named a path the worktree did not hold.
    let saved = save_before_the_command(root.as_path(), common.as_path(), &git, &[b"gone.txt".to_vec()], &[], 5).expect("the first save");
    assert_eq!(saved.copies.len(), 1, "{:?}", saved.copies);
    assert_eq!(saved.copies[0].saved_to, None, "there was nothing to copy: {:?}", saved.copies);
    std::fs::write(root.join("gone.txt"), b"written while the wrapper worked\n").expect("the file that appeared");
    plan_the_last_chance(saved.plan.expect("a comparison is owed"));
    let said: Option<String> = save_again_before_the_command().expect("the file that appeared is copied");
    assert!(said.expect("a copy is announced").contains("gone.txt"));
    remove(&root);
}

/// A source that changes while it is copied again refuses the command and names it,
///  rather than reading it for as long as the writer keeps writing.
#[test]
fn a_source_that_keeps_changing_refuses_the_command_and_names_it() {
    let root: PathBuf = fixture("abort-save-changing");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.join("monochromatic-git-policy-cli").as_path()).expect("the copy tree");
    let path: PathBuf = root.join("changing.txt");
    // Large enough that the copy of it outlives the pause between two writes, so a round meets the writer.
    std::fs::write(path.as_path(), vec![b'x'; 8 << 20]).expect("the file");
    let git: GitContext = context(root.as_path());
    let saved = save_before_the_command(root.as_path(), common.as_path(), &git, &[b"changing.txt".to_vec()], &[], 6).expect("the first save");
    // A writer that adds to the file, bounded to the rounds this control needs and no longer:
    //  it stops by itself, so nothing here waits on a loop.
    let mut writer: std::process::Child = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!(
            "for each in $(seq 1 {}); do printf '%s' 'x' >> '{}'; sleep 0.001; done",
            SAVE_AGAIN_ROUNDS * 60,
            path.display()
        ))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the writer starts");
    // A bounded wait for the writer's first append, so the comparison meets a file that really is being written.
    let mut waited: std::time::Duration = std::time::Duration::ZERO;
    while std::fs::metadata(path.as_path()).map(|found| { return found.len(); }).unwrap_or(0) == 8 << 20
        && waited < std::time::Duration::from_secs(5)
    {
        std::thread::sleep(std::time::Duration::from_millis(5));
        waited += std::time::Duration::from_millis(5);
    }
    plan_the_last_chance(saved.plan.expect("a comparison is owed"));
    let refusal: String = save_again_before_the_command().expect_err("a source the writer keeps changing is refused");
    let _ended = writer.wait();
    assert!(refusal.contains("changing.txt"), "{refusal}");
    assert!(refusal.contains("nothing was changed"), "{refusal}");
    assert!(refusal.contains("--no-enforce-worktree"), "{refusal}");
    remove(&root);
}

/// A source that keeps changing through every round has a sentence of its own:
///  it names the file, the bound and the copies that were made.
#[test]
fn the_bound_sentence_names_the_file_the_rounds_and_the_copies() {
    let changed: Vec<SavedSource> = vec![SavedSource { path: b"changing.txt".to_vec(), stamp: None }];
    let text: String = still_changing_text(changed.as_slice(), Path::new("/repo/.git/discarded/2026-11-11T00-00-00"));
    assert!(text.contains("changing.txt"), "{text}");
    assert!(text.contains(&format!("through {SAVE_AGAIN_ROUNDS} rounds")), "{text}");
    assert!(text.contains("nothing was changed"), "{text}");
    assert!(text.contains("/repo/.git/discarded/2026-11-11T00-00-00"), "the copies are named: {text}");
    assert!(text.contains("--no-enforce-worktree"), "{text}");
    let failure: SaveFailure = SaveFailure { subject: String::from("a.txt"), reason: String::from("changed while it was copied"), kind: FailureKind::Changed };
    let refused: String = refused_save_text(&failure, Path::new("/repo/.git/discarded/2026-11-11T00-00-00"), changed.as_slice());
    assert!(refused.contains("could not be saved again"), "{refused}");
    assert!(refused.contains("changed while it was copied"), "{refused}");
}
