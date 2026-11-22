//! What:
//!  A measurement for the plan,
//! not a control:
//! what one clean same-dataset `git worktree add` saves on disk
//!  against Git's own checkout of the same tree.
//! Why:
//!  The project is choosing between taking the clone rung on ZFS now and taking it once this number exists,
//! so the number is
//!  taken where the rung really runs:
//! in a throwaway guest,
//! on a pool whose block-clone counters and used bytes say what each arm cost.
//! The
//!  arms alternate and repeat,
//! because one reading of a pool that other steps of the same run also wrote to proves little on its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const round of [0, 1]) { git(round); clone(round); }
//! // console.log(`MEASUREMENT zfs clone saved ${saved} bytes of ${bytes} in ${count} files`);
//! ```

/// Import the shared fixtures and the counter readings.
use super::support::{BlockCloneCounters, Fixture, Observed, counters, fixture, on_zfs_or_acknowledged, remove, repository, run_direct, run_wrapped_verbose, stderr_of};
use super::worktree_snapshot::{commit_all, tracked, tracked_content};
use std::path::{Path, PathBuf};

/// How many rounds each arm is measured in,
/// alternating Git's checkout and the wrapper's add.
/// Two rounds are enough to show that one reading is not a lucky one and cheap enough to repeat by hand.
const ROUNDS: usize = 2;

/// What:
///  Every regular file under `directory`,
/// with its path relative to it.
/// Why:
///   The measured tree is this crate's own source,
/// whose size and file count are the run's to report,
/// so the walk names
///  nothing by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const files = walk(manifestDir + '/src').filter(isFile);
/// ```
fn files_under(directory: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut found: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![directory.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in std::fs::read_dir(current.as_path()).expect("a directory of the measured tree").flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.is_file() {
                found.push((path.strip_prefix(directory).expect("below the tree").to_path_buf(), path.clone()));
            }
        }
    }
    found.sort();
    return found;
}

/// What:
///  A repository whose tracked files are this crate's own sources,
/// so the measured tree is a real many-small-files tree of a known size.
/// Why:
///   A tree of one large file would measure a case no checkout meets,
/// and the count and the byte total are what the report states,
/// so
///  they come from the files themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repositoryIn(fixture.root, 'measured'); for (const [rel, abs] of walk(src)) tracked(repo, rel, read(abs));
/// ```
fn measured_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("measured"));
    for (relative, absolute) in files_under(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path()) {
        let content: Vec<u8> = std::fs::read(absolute.as_path()).expect("a source file of the measured tree");
        tracked(repo.as_path(), relative.to_str().expect("a UTF-8 path in this crate"), content.as_slice());
    }
    commit_all(fixture, repo.as_path(), "measured tree");
    return repo;
}

/// What:
///  What one arm cost the pool:
/// the allocated bytes its worktree left there and the cloned blocks the pool counted.
/// Why:
///   The saving is the difference between what Git's checkout allocated and what the clones allocated,
/// and the cloned blocks are what proves the
///  wrapper's arm really shared them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ArmCost = { used: number; blocks: number };
/// ```
#[derive(Clone, Copy, Debug, Default)]
struct ArmCost {
    /// Bytes the pool's `allocated` grew by over the arm.
    used: u64,
    /// Bytes the pool's `bcloneused` grew by over the arm.
    blocks: u64,
}

/// What:
///  What one arm cost the pool,
/// read around it.
/// Why:
///   Both readings are of the same pool and the same fixture root,
/// so only the difference belongs to the arm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = counters(); run(); const after = counters(); return { used: after.used - before.used, blocks: after.bcloneused - before.bcloneused };
/// ```
fn cost_of(before: &BlockCloneCounters, after: &BlockCloneCounters) -> ArmCost {
    return ArmCost { used: after.allocated.saturating_sub(before.allocated), blocks: after.bcloneused.saturating_sub(before.bcloneused) };
}

/// What:
///  Measure both arms over the same tree and print the saving with its method.
/// Why:
///   The project takes the clone rung on ZFS or waits for this number;
/// the line names the arms,
/// the rounds,
/// the tree and the pool's own counters,
/// so a
///  reader can repeat it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.log(`MEASUREMENT zfs clone saved ${saved} bytes`);
/// ```
#[test]
fn a_clean_same_dataset_add_saves_disk_against_gits_checkout() {
    let fixture: Fixture = fixture("zfs-measurement");
    if !on_zfs_or_acknowledged(&fixture, "a_clean_same_dataset_add_saves_disk_against_gits_checkout") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = measured_repository(&fixture);
    let tree: Vec<(String, Vec<u8>, bool)> = tracked_content(&fixture, repo.as_path());
    let files: usize = tree.len();
    let bytes: u64 = tree.iter().map(|(_name, content, _executable)| { return content.len() as u64; }).sum();
    let mut git: ArmCost = ArmCost::default();
    let mut clone: ArmCost = ArmCost::default();
    let mut last_clone: BlockCloneCounters = counters(&fixture);
    for round in 0..ROUNDS {
        let before_git: BlockCloneCounters = counters(&fixture);
        let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", format!("control{round}").as_str(), format!("../control{round}").as_str()]);
        let after_git: BlockCloneCounters = counters(&fixture);
        assert_eq!(control.code, Some(0), "Git's own checkout of round {round}: {}", stderr_of(&control));
        git.used += cost_of(&before_git, &after_git).used;
        let before_clone: BlockCloneCounters = counters(&fixture);
        let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", format!("clone{round}").as_str(), format!("../clone{round}").as_str()]);
        let after_clone: BlockCloneCounters = counters(&fixture);
        assert_eq!(created.code, Some(0), "the wrapper's add of round {round}: {}", stderr_of(&created));
        assert!(
            stderr_of(&created).contains("clones served"),
            "round {round} must measure the clone rung and not Git's checkout:\n{}",
            stderr_of(&created)
        );
        let cost: ArmCost = cost_of(&before_clone, &after_clone);
        clone.used += cost.used;
        clone.blocks += cost.blocks;
        last_clone = after_clone;
    }
    let saved: u64 = git.used.saturating_sub(clone.used);
    println!(
        "MEASUREMENT zfs clone saved {saved} bytes of {bytes} in {files} files over {ROUNDS} alternating rounds (Git's checkout used +{} bytes, the clone rung used +{} bytes and counted +{} cloned bytes; pool bcloneused {}, bclonesaved {}, bcloneratio {})",
        git.used, clone.used, clone.blocks, last_clone.bcloneused, last_clone.bclonesaved, last_clone.bcloneratio
    );
    assert!(saved > 0, "a clone rung that saved nothing is no rung: Git's checkout used +{}, the clone rung +{}", git.used, clone.used);
    remove(&fixture);
}
