//! What:
//!  Rung 2 of the cheap-worktree ladder through the built executable: `git worktree add` from a clean worktree that is no
//!  subvolume is served by clones of its tracked files, and the result equals what Git's own checkout makes.
//! Why:
//!  A clone shares data blocks with the source, which Git's checkout never does, so a shared physical extent is the sign
//!  that clones served the command; contents, status, words and the ignored-state copy are compared with a control that
//!  real Git made from the same commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(physicalExtent('../topic/big.bin')).toBe(physicalExtent('big.bin'));
//! ```

/// Import the shared fixtures and the helpers of the snapshot controls.
use super::support::{Fixture, Observed, fixture, git, remove, repository, run_wrapped, stderr_of};
use super::worktree_snapshot::{add_pair, age, commit_all, on_btrfs_or_acknowledged, tracked, tracked_content};
use git_policy_cli::fs_kind::is_subvolume_root;
use std::ffi::OsStr;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

/// `FS_IOC_FIEMAP`, `_IOWR('f', 11, struct fiemap)`.
const FS_IOC_FIEMAP: libc::c_ulong = 0xc020_660b;

/// The physical address of the first extent of a file, which two files share when one is a clone of the other.
fn first_physical_extent(path: &Path) -> u64 {
    #[repr(C)]
    struct Request {
        start: u64,
        length: u64,
        flags: u32,
        mapped: u32,
        count: u32,
        reserved: u32,
        extent: [u8; 56],
    }
    let file: std::fs::File = std::fs::File::open(path).expect("open");
    let mut request: Request = Request { start: 0, length: u64::MAX, flags: 1, mapped: 0, count: 1, reserved: 0, extent: [0; 56] };
    // SAFETY: `request` is a `struct fiemap` followed by room for the one extent it asks for, and the descriptor is open.
    let result: libc::c_int = unsafe { libc::ioctl(file.as_raw_fd(), FS_IOC_FIEMAP, &mut request) };
    assert_eq!(result, 0, "FIEMAP of {}", path.display());
    assert_eq!(request.mapped, 1, "{} has an extent", path.display());
    return u64::from_ne_bytes(request.extent[8..16].try_into().expect("eight bytes"));
}

/// A repository that is an ordinary directory with a large aged file, a second commit and an ignored directory.
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
    age(repo.join("ignored/pkg/index.js").as_path());
    return repo;
}

/// A clean worktree that is no subvolume is cloned: the data blocks are shared, the content equals the control, the first
///  status is clean, and the caller sees Git's own words.
#[test]
fn a_clean_ordinary_main_checkout_is_cloned_and_equals_gits_own_checkout() {
    let fixture: Fixture = fixture("clone-main");
    if !on_btrfs_or_acknowledged(&fixture, "clone-main") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    let (created, control) = add_pair(&fixture, repo.as_path(), "topic", "control", &["-b", "NAME", "../NAME"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let wrapped: String = stderr_of(&created).replace("topic", "NAME");
    let git_words: String = stderr_of(&control).replace("control", "NAME");
    assert!(wrapped.starts_with(git_words.as_str()), "Git's own words come first: {wrapped}");
    // Then the line that says why this was not a snapshot (the main checkout here is no subvolume), then the summary line of the
    //  ignored-state copy.
    let after_git: Vec<&str> = wrapped[git_words.len()..].lines().collect();
    assert!(after_git.first().is_some_and(|line: &&str| { return line.starts_with("monochromatic-git-policy-cli: this worktree was not made as a snapshot"); }), "{wrapped}");
    assert!(after_git.get(1).is_some_and(|line: &&str| { return line.starts_with("monochromatic-git-policy-cli: copied "); }), "then the summary line of the ignored-state copy: {wrapped}");
    assert_eq!(String::from_utf8_lossy(&created.stdout), String::from_utf8_lossy(&control.stdout));
    let topic: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(topic.as_path()).expect("probe"), "the clones went into a subvolume");
    assert_eq!(first_physical_extent(topic.join("big.bin").as_path()), first_physical_extent(repo.join("big.bin").as_path()), "a clone shares the extent");
    assert_ne!(first_physical_extent(fixture.root.join("control/big.bin").as_path()), first_physical_extent(repo.join("big.bin").as_path()), "Git's checkout wrote its own");
    assert_eq!(tracked_content(&fixture, topic.as_path()), tracked_content(&fixture, fixture.root.join("control").as_path()));
    assert_eq!(git(&fixture, topic.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]).stdout, b"", "clean on the first status");
    assert!(topic.join("ignored").exists(), "the clones carry no ignored state; the copy of the main checkout does");
    remove(&fixture);
}

/// An older target gets the difference applied by Git on top of the clones, and equals the control.
#[test]
fn an_older_target_commit_is_cloned_and_updated() {
    let fixture: Fixture = fixture("clone-older");
    if !on_btrfs_or_acknowledged(&fixture, "clone-older") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    let (created, _control) = add_pair(&fixture, repo.as_path(), "old", "oldcontrol", &["--detach", "../NAME", "HEAD~1"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let old: PathBuf = fixture.root.join("old");
    assert_eq!(first_physical_extent(old.join("big.bin").as_path()), first_physical_extent(repo.join("big.bin").as_path()));
    assert_eq!(std::fs::read(old.join("shared.txt")).expect("shared"), b"first\n");
    assert!(old.join("old.txt").is_file() && !old.join("new").exists());
    assert_eq!(tracked_content(&fixture, old.as_path()), tracked_content(&fixture, fixture.root.join("oldcontrol").as_path()));
    assert_eq!(git(&fixture, old.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    remove(&fixture);
}

/// A dirty source is cloned from by nothing: Git's checkout runs and the data blocks are not shared.
#[test]
fn a_dirty_ordinary_source_is_not_cloned() {
    let fixture: Fixture = fixture("clone-dirty");
    if !on_btrfs_or_acknowledged(&fixture, "clone-dirty") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = ordinary_repository(&fixture);
    std::fs::write(repo.join("untracked.txt"), b"u").expect("untracked");
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "plain", "../plain"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let plain: PathBuf = fixture.root.join("plain");
    assert_ne!(first_physical_extent(plain.join("big.bin").as_path()), first_physical_extent(repo.join("big.bin").as_path()));
    assert!(is_subvolume_root(plain.as_path()).expect("probe"));
    remove(&fixture);
}
