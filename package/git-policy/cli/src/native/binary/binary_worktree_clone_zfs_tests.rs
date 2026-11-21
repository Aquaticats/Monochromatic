//! What:
//!  Rung 2 of the cheap-worktree ladder on ZFS through the built executable:
//! a clean add inside one dataset is served by clones of its tracked files,
//! and every add the dataset
//!  domain forbids ends in Git's own checkout instead.
//! Why:
//!  A reflink shares data blocks only inside one ZFS dataset,
//! and a successful `copy_file_range` can share blocks across datasets that hold
//!  the same data key,
//! so the domain is proven by the mount source and the device and never by whether a copy happened to share
//!  blocks.
//! ZFS answers `FIEMAP` with no physical extent (`filefrag` reports `FIBMAP/FIEMAP unsupported` on a ZFS file),
//! so the shared-block reading a btrfs control takes from two equal extents is the pool's own cloned-block
//!  counters here,
//! read after `zpool sync` has committed the arm's transaction group.
//! The controls therefore pair that reading with the refusal's own words:
//! a destination on the second
//!  dataset,
//! a mounted clone at the destination and a mounted clone as the source must each be refused,
//! and a dataset whose block cloning is off must
//!  end in Git's checkout with its own bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(stderr).toContain('clones served'); expect(firstExtent(topic)).toBe(firstExtent(source));
//! // expect(stderr).toContain('a reflink shares data blocks only inside one dataset');
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{
    BlockCloneCounters, BlockCloningSwitch, Fixture, Observed, block_cloning_off, counters, destroy_dataset, different_key_dataset, fixture, git, mount_clone_over,
    on_zfs_or_acknowledged, remove, repository, run_direct, run_wrapped_verbose, second_dataset, stderr_of,
};
use super::worktree_snapshot::{commit_all, tracked, tracked_content};
use git_policy_cli::fs_kind::filesystem_kind;
use git_policy_cli::fs_mount_options::read_mount_facts;
use std::ffi::OsStr;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

/// The words every ZFS domain refusal ends with,
/// so a control asserts the cause and not the paths alone.
const ONE_DATASET_PHRASE: &str = "a reflink shares data blocks only inside one dataset";

/// What:
///  The mount source of the dataset that holds a path,
/// as the mount table names it.
/// Why:
///   A refusal test names the datasets it expects,
/// and the dataset names are the pool's own,
/// which no control may guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dataset = mountSource('/home/...'); // 'pool/dataset'
/// ```
fn dataset_of(path: &Path) -> String {
    let facts = read_mount_facts(path).ok().flatten();
    let source: String = facts.unwrap_or_else(|| { panic!("the mount table names no source for {}", path.display()) }).source;
    return source;
}

/// What:
///  A repository whose main checkout is an ordinary directory on the fixture's root dataset,
/// with a large aged file,
/// a second commit and an ignored directory.
/// Why:
///   The same shape the btrfs clone controls use,
/// so a clone rung is offered the same tracked files on ZFS and the
///  extents can be compared file for file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repositoryIn(fixture.root, 'repo');
/// ```
fn ordinary_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    tracked(repo.as_path(), ".gitignore", b"ignored/\n");
    tracked(repo.as_path(), "old.txt", b"old\n");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    tracked(repo.as_path(), "big.bin", &(0..200_000u32).flat_map(|value: u32| { return value.to_le_bytes(); }).collect::<Vec<u8>>());
    commit_all(fixture, repo.as_path(), "first");
    std::fs::remove_file(repo.join("old.txt")).expect("delete");
    tracked(repo.as_path(), "shared.txt", b"second, longer\n");
    tracked(repo.as_path(), "new/added.txt", b"added\n");
    commit_all(fixture, repo.as_path(), "second");
    std::fs::create_dir_all(repo.join("ignored/pkg")).expect("ignored directory");
    std::fs::write(repo.join("ignored/pkg/index.js"), b"module.exports = 1;\n").expect("ignored file");
    return repo;
}

/// What:
///  A repository at a path that is not under the fixture root,
/// on the dataset that path was made on.
/// Why:
///   The source of a linked-worktree add is the worktree the person runs the add from,
/// so a source inside a mounted clone is
///  what the checkout-dataset slot must refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = gitInitAt(mountedCloneRoot + '/repo');
/// ```
fn repository_at(fixture: &Fixture, root: &Path) -> PathBuf {
    std::fs::create_dir_all(root).expect("repository directory");
    git(fixture, root, &["init", "--quiet", "--initial-branch=main"]);
    git(fixture, root, &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    tracked(root, "src/deep/leaf.txt", b"leaf\n");
    tracked(root, "big.bin", &(0..200_000u32).flat_map(|value: u32| { return value.to_le_bytes(); }).collect::<Vec<u8>>());
    commit_all(fixture, root, "files");
    return root.to_path_buf();
}

/// What:
///  Copy a whole file with `copy_file_range`,
/// which may share blocks across datasets that hold the same key,
/// into a destination sized first.
/// Why:
///   The domain decision never reads the data keys,
/// and this copy is how a control shows that blocks can be shared across two
///  datasets while the add is still refused.
/// A fresh destination made by this call shares nothing,
/// because `zfs_clone_range` cannot grow a
///  destination whose block size it would have to set from a clone,
/// so the destination is sized to the source before the copy,
/// which is what a
///  copy into an existing file does;
/// the readings this control takes say which of the two happened,
/// and the run that found it is in the
///  step 3 report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // what a copy into an existing file between two datasets does: copy_file_range, not FICLONE
/// ```
fn copy_file_range(from: &Path, to: &Path) -> u64 {
    let source: std::fs::File = std::fs::File::open(from).expect("open the copy source");
    let size: u64 = source.metadata().expect("the source's size").len();
    let destination: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).open(to).expect("create the copy");
    destination.set_len(size).expect("size the copy's destination");
    let mut copied: u64 = 0;
    loop {
        // SAFETY: both descriptors are open for the call and the offsets are null, so each call reads and writes where the
        //  descriptors stand.
        let written: libc::ssize_t = unsafe {
            libc::copy_file_range(source.as_raw_fd(), std::ptr::null_mut(), destination.as_raw_fd(), std::ptr::null_mut(), 1 << 20, 0)
        };
        assert!(written >= 0, "copy_file_range of {} to {} failed with errno {}", from.display(), to.display(), std::io::Error::last_os_error());
        if written == 0 {
            return copied;
        }
        copied += written as u64;
    }
}

/// What:
///  The reason the clone rung gave,
/// as the diagnostic log prints it,
/// or nothing when the log holds none.
/// Why:
///   The refusal is silent to the person by design,
/// so a control that must show the add was refused for the dataset reason reads it there
/// rather
///  than trusting that Git's checkout means a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = stderr.match(/clone: Some\("(.*)"\)/)?.[1];
/// ```
fn clone_refusal_reason(stderr: &str) -> Option<String> {
    let marked: &str = stderr.split("clone: Some(\"").nth(1)?;
    return marked.split("\")").next().map(str::to_owned);
}

/// What:
///  Assert that the add was refused for the dataset reason,
/// naming both datasets,
/// and that Git's checkout made the worktree.
/// Why:
///   A control that asserted only the fallback would pass with no domain decision at all,
/// because a failed `FICLONE`
///  also ends in Git's checkout;
/// naming both datasets is what proves the decision ran and said why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reason).toContain(ONE_DATASET_PHRASE); expect(reason).toContain(sourceDataset); expect(reason).toContain(otherDataset);
/// ```
fn assert_refused_across_datasets(stderr: &str, source_dataset: &str, other_dataset: &str) {
    let reason: String = clone_refusal_reason(stderr).unwrap_or_else(|| { panic!("the add was not refused: the diagnostic log names no clone refusal\n{stderr}") });
    assert!(reason.contains(ONE_DATASET_PHRASE), "the refusal does not name the dataset rule: {reason}");
    assert!(reason.contains(source_dataset), "the refusal does not name {source_dataset}: {reason}");
    assert!(reason.contains(other_dataset), "the refusal does not name {other_dataset}: {reason}");
}

/// A clean add whose source and destination are one dataset is served by clones of its tracked files:
/// the diagnostic log says so,
/// the data blocks are shared,
/// the pool counts the cloned bytes,
/// and the content equals Git's own checkout.
#[test]
fn a_same_dataset_add_is_served_by_clones_that_share_blocks() {
    let fixture: Fixture = fixture("zfs-clone-same-dataset");
    if !on_zfs_or_acknowledged(&fixture, "a_same_dataset_add_is_served_by_clones_that_share_blocks") {
        remove(&fixture);
        return;
    }
    assert_eq!(filesystem_kind(fixture.root.as_path()), Ok(git_policy_cli::fs_kind::FilesystemKind::Zfs));
    let repo: PathBuf = ordinary_repository(&fixture);
    let source_dataset: String = dataset_of(repo.as_path());
    assert_eq!(dataset_of(fixture.root.as_path()), source_dataset, "the fixture root and the repository are one dataset");
    let before: BlockCloneCounters = counters(&fixture);
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "-b", "topic", "../topic"]);
    let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "control", "../control"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    let after: BlockCloneCounters = counters(&fixture);
    let topic: PathBuf = fixture.root.join("topic");
    assert!(stderr_of(&created).contains(format!("clones served {}", topic.display()).as_str()), "the clone rung served the add on {}:\n{}", source_dataset, stderr_of(&created));
    assert!(after.bcloneused > before.bcloneused, "the pool counts the cloned blocks (bcloneused {} to {})", before.bcloneused, after.bcloneused);
    assert!(after.bclonesaved > before.bclonesaved, "the pool saved what the clones hold (bclonesaved {} to {})", before.bclonesaved, after.bclonesaved);
    assert_eq!(tracked_content(&fixture, topic.as_path()), tracked_content(&fixture, fixture.root.join("control").as_path()));
    assert_eq!(git(&fixture, topic.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]).stdout, b"", "clean on the first status");
    remove(&fixture);
}

/// A destination on the second dataset of the pool is refused with both dataset names,
/// and Git fills the worktree with its own bytes.
#[test]
fn a_destination_on_the_second_dataset_is_refused_and_git_fills_the_worktree() {
    let fixture: Fixture = fixture("zfs-clone-second-dataset");
    if !on_zfs_or_acknowledged(&fixture, "a_destination_on_the_second_dataset_is_refused_and_git_fills_the_worktree") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    let second: PathBuf = second_dataset();
    let source_dataset: String = dataset_of(repo.as_path());
    let second_dataset_name: String = dataset_of(second.as_path());
    assert_ne!(source_dataset, second_dataset_name, "the fixture's two datasets are two");
    let destination: PathBuf = second.join(format!("topic-{}", std::process::id()));
    let before: BlockCloneCounters = counters(&fixture);
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", destination.to_str().expect("a path")]);
    let after: BlockCloneCounters = counters(&fixture);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_refused_across_datasets(stderr_of(&created).as_str(), source_dataset.as_str(), second_dataset_name.as_str());
    assert_eq!(after.bcloneused, before.bcloneused, "no block was cloned across the two datasets");
    assert_eq!(tracked_content(&fixture, destination.as_path()), tracked_content(&fixture, repo.as_path()));
    assert!(after.allocated > before.allocated, "Git's checkout wrote the tree anew (allocated {} to {})", before.allocated, after.allocated);
    std::fs::remove_dir_all(destination.as_path()).expect("remove the worktree");
    remove(&fixture);
}

/// A slot a mounted clone of the checkout dataset occupies is refused,
/// and Git fills the worktree there.
#[test]
fn a_slot_a_mounted_clone_occupies_is_refused() {
    let fixture: Fixture = fixture("zfs-clone-slot");
    if !on_zfs_or_acknowledged(&fixture, "a_slot_a_mounted_clone_occupies_is_refused") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    let slot: PathBuf = fixture.root.join(format!("slot-{}", std::process::id()));
    let clone: String = mount_clone_over(&fixture, slot.as_path(), format!("slot-{}", std::process::id()).as_str());
    let source_dataset: String = dataset_of(repo.as_path());
    assert_eq!(dataset_of(slot.as_path()), clone, "the slot holds the mounted clone's dataset");
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "slot", slot.to_str().expect("a path")]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_refused_across_datasets(stderr_of(&created).as_str(), source_dataset.as_str(), clone.as_str());
    assert_eq!(tracked_content(&fixture, slot.as_path()), tracked_content(&fixture, repo.as_path()));
    destroy_dataset(clone.as_str());
    remove(&fixture);
}

/// A mounted clone used as the source of a slot in the checkout dataset is refused,
/// and Git fills the worktree.
#[test]
fn a_mounted_clone_is_no_source_for_a_checkout_dataset_slot() {
    let fixture: Fixture = fixture("zfs-clone-source");
    if !on_zfs_or_acknowledged(&fixture, "a_mounted_clone_is_no_source_for_a_checkout_dataset_slot") {
        remove(&fixture);
        return;
    }
    let mountpoint: PathBuf = fixture.root.join(format!("mounted-{}", std::process::id()));
    let clone: String = mount_clone_over(&fixture, mountpoint.as_path(), format!("source-{}", std::process::id()).as_str());
    let repo: PathBuf = repository_at(&fixture, mountpoint.join("repo").as_path());
    let checkout_dataset: String = dataset_of(fixture.root.as_path());
    assert_eq!(dataset_of(mountpoint.as_path()), clone, "the repository is in the mounted clone's dataset");
    let destination: PathBuf = fixture.root.join(format!("from-clone-{}", std::process::id()));
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "fromclone", destination.to_str().expect("a path")]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_refused_across_datasets(stderr_of(&created).as_str(), clone.as_str(), checkout_dataset.as_str());
    assert_eq!(tracked_content(&fixture, destination.as_path()), tracked_content(&fixture, repo.as_path()));
    destroy_dataset(clone.as_str());
    remove(&fixture);
}

/// What:
///  The bytes one copy of a fixture's large file holds,
/// for the counter readings that must move by at least them.
/// Why:
///   A control asserts the pool counted shared blocks,
/// and naming the size it copied keeps that assertion about this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const size = statSync(bigFile).size;
/// ```
fn big_file_bytes(fixture: &Fixture) -> u64 {
    let repo: PathBuf = fixture.root.join("repo");
    return std::fs::metadata(repo.join("big.bin")).expect("the large tracked file").len();
}

/// What:
///  A reflink of the large tracked file inside the checkout dataset,
/// and how many bytes the pool counted as cloned for it.
/// Why:
///   An arm that reads no cloned block after a cross-dataset copy must first show that this guest counts a clone at all,
/// otherwise the reading
///  says nothing about the copy;
/// the clone is made by the same `FICLONE` the ladder uses.
/// The count is physical:
/// `bcloneused` is the space the cloned blocks take after compression,
/// so a compressed file is
///  counted as fewer bytes than it holds and only a count above none is asserted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, clone, constants.COPYFILE_FICLONE_FORCE); expect(bcloneusedDelta).toBeGreaterThanOrEqual(size);
/// ```
fn same_dataset_reflink_bytes(fixture: &Fixture, repo: &Path) -> u64 {
    let source: std::fs::File = std::fs::File::open(repo.join("big.bin")).expect("the large tracked file");
    let destination_path: PathBuf = repo.join("reflink-control.bin");
    let destination: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).open(destination_path.as_path()).expect("the reflink's destination");
    let before: BlockCloneCounters = counters(fixture);
    git_policy_cli::fs_clone_exchange::clone_file(&source, &destination, destination_path.as_path()).expect("FICLONE inside one dataset");
    let after: BlockCloneCounters = counters(fixture);
    std::fs::remove_file(destination_path.as_path()).expect("remove the reflink");
    let cloned: u64 = after.bcloneused.saturating_sub(before.bcloneused);
    assert!(
        cloned > 0,
        "this guest counts a same-dataset reflink of {} bytes as cloned blocks, and it counted {cloned}",
        big_file_bytes(fixture)
    );
    return cloned;
}

/// What:
///  A pair of datasets under one data key does not qualify,
/// and the copy between them is measured rather than assumed to share blocks.
/// Why:
///   The domain decision never reads the data keys,
/// so a pair that shares a key must be refused exactly as a pair that does not;
/// the arm first
///  shows this guest counts a same-dataset reflink,
/// then learns what a `copy_file_range` between the two datasets did here,
/// and then
///  asserts that the add into the second dataset was refused with both dataset names.
/// The plan's premise that a same-key
///  cross-dataset copy shares blocks did not hold in this guest,
/// so the copy's reading is a measurement and the refusal alone is
///  asserted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(sameDatasetReflinkBytes()).toBeGreaterThanOrEqual(size); copyAcrossDatasets(); expect(stderr).toContain(ONE_DATASET_PHRASE);
/// ```
#[test]
fn a_same_key_cross_dataset_pair_does_not_qualify() {
    let fixture: Fixture = fixture("zfs-cross-key-same");
    if !on_zfs_or_acknowledged(&fixture, "a_same_key_cross_dataset_pair_does_not_qualify") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    let second: PathBuf = second_dataset();
    let source_dataset: String = dataset_of(repo.as_path());
    let second_dataset_name: String = dataset_of(second.as_path());
    assert_ne!(source_dataset, second_dataset_name, "the fixture's two datasets are two");
    let same_dataset: u64 = same_dataset_reflink_bytes(&fixture, repo.as_path());
    let copied: PathBuf = second.join(format!("copied-{}.bin", std::process::id()));
    let before_copy: BlockCloneCounters = counters(&fixture);
    let bytes: u64 = copy_file_range(repo.join("big.bin").as_path(), copied.as_path());
    let after_copy: BlockCloneCounters = counters(&fixture);
    assert_eq!(bytes, big_file_bytes(&fixture), "the copy is the whole file");
    println!(
        "MEASUREMENT zfs cross-dataset copy same-key: {bytes} bytes copied, cloned blocks {} to {} (a same-dataset reflink of the same file counted {same_dataset})",
        before_copy.bcloneused, after_copy.bcloneused
    );
    let destination: PathBuf = second.join(format!("topic-same-{}", std::process::id()));
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "samekey", destination.to_str().expect("a path")]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_refused_across_datasets(stderr_of(&created).as_str(), source_dataset.as_str(), second_dataset_name.as_str());
    std::fs::remove_file(copied.as_path()).expect("remove the copy");
    std::fs::remove_dir_all(destination.as_path()).expect("remove the worktree");
    remove(&fixture);
}

/// A pair of datasets under different data keys is refused all the same,
/// and the copy between them shares no block.
#[test]
fn a_different_key_cross_dataset_pair_does_not_qualify() {
    let fixture: Fixture = fixture("zfs-cross-key-different");
    if !on_zfs_or_acknowledged(&fixture, "a_different_key_cross_dataset_pair_does_not_qualify") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    let other: PathBuf = different_key_dataset();
    let same_dataset: u64 = same_dataset_reflink_bytes(&fixture, repo.as_path());
    let copied: PathBuf = other.join(format!("copied-{}.bin", std::process::id()));
    let before_copy: BlockCloneCounters = counters(&fixture);
    let bytes: u64 = copy_file_range(repo.join("big.bin").as_path(), copied.as_path());
    let after_copy: BlockCloneCounters = counters(&fixture);
    assert_eq!(bytes, big_file_bytes(&fixture), "the copy is the whole file");
    assert_eq!(
        after_copy.bcloneused, before_copy.bcloneused,
        "a copy under another data key shares no block ({} to {})",
        before_copy.bcloneused, after_copy.bcloneused
    );
    println!(
        "MEASUREMENT zfs cross-dataset copy different-key: {bytes} bytes copied, cloned blocks {} to {} (a same-dataset reflink of the same file counted {same_dataset})",
        before_copy.bcloneused, after_copy.bcloneused
    );
    let destination: PathBuf = other.join(format!("topic-different-{}", std::process::id()));
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "otherkey", destination.to_str().expect("a path")]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_refused_across_datasets(stderr_of(&created).as_str(), dataset_of(repo.as_path()).as_str(), dataset_of(other.as_path()).as_str());
    std::fs::remove_file(copied.as_path()).expect("remove the copy");
    std::fs::remove_dir_all(destination.as_path()).expect("remove the worktree");
    remove(&fixture);
}

/// With the kernel's block cloning off,
/// the clone rung ends and Git's checkout makes the worktree,
/// which the pool shows by writing new bytes and counting no cloned block.
#[test]
fn a_dataset_without_block_cloning_ends_in_gits_checkout_not_a_byte_copy() {
    let fixture: Fixture = fixture("zfs-block-cloning-off");
    if !on_zfs_or_acknowledged(&fixture, "a_dataset_without_block_cloning_ends_in_gits_checkout_not_a_byte_copy") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = repository_at(&fixture, fixture.root.join(format!("off-{}", std::process::id())).join("repo").as_path());
    let destination: PathBuf = repo.parent().expect("the arm's directory").join("topic");
    let switch: BlockCloningSwitch = block_cloning_off();
    let before: BlockCloneCounters = counters(&fixture);
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "plain", destination.to_str().expect("a path")]);
    let after: BlockCloneCounters = counters(&fixture);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let stderr: String = stderr_of(&created);
    assert!(stderr.contains("the clone rung ended"), "the clone rung tried and ended:\n{stderr}");
    assert!(!stderr.contains("clones served"), "no clone served the add:\n{stderr}");
    assert_eq!(after.bcloneused, before.bcloneused, "no block was cloned (bcloneused {} to {})", before.bcloneused, after.bcloneused);
    // Compression decides how many bytes a checkout of this tree costs, so the proof that the bytes were written anew is the
    //  pool's cloned-block count staying still beside space being spent at all.
    assert!(after.allocated > before.allocated, "Git's checkout spent pool space (allocated {} to {})", before.allocated, after.allocated);
    assert_eq!(tracked_content(&fixture, destination.as_path()), tracked_content(&fixture, repo.as_path()));
    drop(switch);
    std::fs::remove_dir_all(destination.as_path()).expect("remove the worktree");
    remove(&fixture);
}
