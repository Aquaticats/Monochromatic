//! What:
//!  The in-process update as a landing runs it,
//!  under the same index lock protocol the service uses,
//!  and a commit whose tree holds one raw entry Git's own name checks would
//!  refuse.
//! Why:
//!  The controls compare index files after the update,
//!  so they run it the way the landing does:
//!  take the live index lock,
//!  copy the live index to the private name,
//!  reset the private copy and install it by the lock's release,
//!  and drop the copy and the lock when the update declines.
//!  A name Git refuses can still be in a tree,
//!  so the fallback's controls make one with `hash-object --literally`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = landingReset(context, repo, newCommit, paths);
//! const made = commitWithRawEntry(scene, name, mode, content);
//! ```

/// The Git start a raw tree entry is written with.
use super::git_starts::git_with;
/// The repository the reset runs in,
/// and the paths it resets.
use super::scene_asked_for::Scene;
/// The aging the private copy is given,
/// and the text one Git start answers with.
use crate::test_support::{
    age,
    git_text,
};
/// The private copy and the repository it is written to.
use std::path::{
    Path,
    PathBuf,
};
/// The answer of one Git start.
use std::process::Output;
/// The batch numbers of the landings the controls make.
static NEXT_BATCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// What:
///  The in-process update as a landing runs it:
/// take the live index lock,
/// copy the live index to the private name,
/// reset the
///  private copy,
/// and install it by the lock's release;
/// when the update declines,
/// drop the copy and the lock and leave the
///  live index as it was.
/// Why:
///   The controls compare index files after the update,
/// so they run it under the same lock protocol the service does.
pub(crate) fn landing_reset(
    context: &crate::transaction_git::GitContext,
    repo: &Path,
    new: &str,
    paths: crate::service_land_follow::IndexPaths<'_>,
) -> bool {
    return landing_reset_copy(context, repo, new, paths, false);
}

/// `landing_reset`,
/// with the private copy given an old modification time when `old_copy` is set.
/// The copy's time is the one
/// the update compares the entries' times with,
/// so a control for racily clean entries needs the copy older than they are.
pub(crate) fn landing_reset_copy(
    context: &crate::transaction_git::GitContext,
    repo: &Path,
    new: &str,
    paths: crate::service_land_follow::IndexPaths<'_>,
    old_copy: bool,
) -> bool {
    let live: PathBuf = repo.join(".git").join("index");
    let seq: u64 = NEXT_BATCH.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let Some(held) = crate::service_index_lock::try_acquire(live.as_path(), seq) else {
        return false;
    };
    let copy = crate::service_land_follow::PrivateCopy::make(&held, seq).expect("the private copy");
    let built: PathBuf = copy.path().to_path_buf();
    if old_copy && built.exists() {
        age(built.as_path());
    }
    let done: bool =
        crate::index_reset_file::reset_index_file(context, repo, new, paths, &held, &copy)
            .is_some();
    if done {
        held.release_with(built.as_path()).expect("installed");
    } else {
        let _removed = std::fs::remove_file(&built);
        drop(held);
    }
    return done;
}

/// A commit on top of the scene's `new` whose whole tree is the one entry `name` of `mode` naming a blob with `content`,
/// made
/// without Git's name checks (`hash-object --literally`),
/// so a name Git refuses can be in a tree.
pub(crate) fn commit_with_raw_entry(
    scene: &Scene,
    name: &str,
    mode: &str,
    content: &[u8],
) -> String {
    let blob: String = String::from_utf8(
        git_with(
            scene.repo.as_path(),
            &[],
            &["hash-object", "-w", "--stdin"],
            Some(content),
        )
        .stdout,
    )
    .expect("UTF-8")
    .trim()
    .to_string();
    let mut tree: Vec<u8> = format!("{mode} {name}\0").into_bytes();
    for pair in blob.as_bytes().as_chunks::<2>().0 {
        tree.push(
            u8::from_str_radix(std::str::from_utf8(pair).expect("hex"), 16)
                .expect("hex digit pair"),
        );
    }
    let made: Output = git_with(
        scene.repo.as_path(),
        &[],
        &["hash-object", "-t", "tree", "-w", "--literally", "--stdin"],
        Some(tree.as_slice()),
    );
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let tree_name: String = String::from_utf8(made.stdout)
        .expect("UTF-8")
        .trim()
        .to_string();
    return git_text(
        scene.repo.as_path(),
        &[
            "commit-tree",
            tree_name.as_str(),
            "-p",
            scene.new.as_str(),
            "-m",
            "raw entry",
        ],
    );
}
