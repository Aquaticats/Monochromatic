//! What:
//!  `git monochromatic-git-policy-cli migrate-to-subvolume` and the doctor's filesystem section through the built executable.
//! Why:
//!  The command is a promise about the program a caller runs as `git`: its grammar answers without a repository, a
//!  refusal and a preview change nothing, a real run on btrfs ends with the checkout as a subvolume root and Git clean,
//!  and the rollback and the deletion of the old tree finish the job. The runs that need btrfs print one loud line and
//!  stop when the fixture root is elsewhere and the run says that is known (`CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED`);
//!  without the variable they fail with the filesystem they found.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['monochromatic-git-policy-cli', 'migrate-to-subvolume'])).toMatchObject({ status: 0 });
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, remove, repository, run_direct, run_wrapped};
use git_policy_cli::doctor_filesystem::SECTION_MARKER;
use git_policy_cli::fs_kind::{FilesystemKind, filesystem_kind};
use std::ffi::OsStr;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The inode number of the root of every btrfs subvolume.
const SUBVOLUME_ROOT_INODE: u64 = 256;

/// Standard error as text.
fn stderr_text(observed: &Observed) -> String {
    return String::from_utf8_lossy(&observed.stderr).into_owned();
}

/// Standard output as text.
fn stdout_text(observed: &Observed) -> String {
    return String::from_utf8_lossy(&observed.stdout).into_owned();
}

/// A repository with tracked files, a subdirectory and an ignored directory, committed with real Git.
fn populated_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::create_dir_all(repo.join("src/deep")).expect("directories");
    std::fs::write(repo.join("README.md"), b"readme\n").expect("readme");
    std::fs::write(repo.join("src/deep/leaf.txt"), vec![b'x'; 50_000]).expect("leaf");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("gitignore");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
    std::fs::create_dir_all(repo.join("ignored/pkg")).expect("ignored directory");
    std::fs::write(repo.join("ignored/pkg/index.js"), b"module.exports = 1;\n").expect("ignored file");
    return repo;
}

/// Whether the fixture root is on btrfs; where it is not, the run must have said so, and one loud line names the control.
fn on_btrfs_or_acknowledged(fixture: &Fixture, control: &str) -> bool {
    let kind = filesystem_kind(fixture.root.as_path());
    if kind == Ok(FilesystemKind::Btrfs) {
        return true;
    }
    let acknowledged: bool = std::env::var_os("CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED").is_some_and(|value| { return !value.is_empty(); });
    assert!(
        acknowledged,
        "the control {control} needs a btrfs fixture root, but {} is on {kind:?}; set CLI_GIT_TEST_ROOT to a directory on btrfs, or set CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1",
        fixture.root.display()
    );
    eprintln!("BTRFS CONTROL NOT RUN: {control} (fixture root {} is on {kind:?})", fixture.root.display());
    return false;
}

/// The inode number of a path.
fn inode(path: &Path) -> u64 {
    return std::fs::metadata(path).expect("metadata").ino();
}

/// The help prints on standard output with status 0 and names every option, in a directory that is no repository.
#[test]
fn the_help_answers_anywhere() {
    let fixture: Fixture = fixture("subvolume-help");
    let help: Observed = run_wrapped(&fixture, fixture.root.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume", "--help"]);
    assert_eq!(help.code, Some(0), "{}", stderr_text(&help));
    let text: String = stdout_text(&help);
    for option in ["--dry-run", "--rollback", "--remove-old", "--discard-unreadable", "--no-nested-worktrees", "--no-refresh-indexes"] {
        assert!(text.contains(option), "{option} missing from {text}");
    }
    remove(&fixture);
}

/// A malformed invocation exits 2 with the reason and the way to the help; an invocation outside a repository exits 1
///  with the instruction. Neither creates anything.
#[test]
fn a_malformed_invocation_and_a_run_outside_a_repository_are_refused() {
    let fixture: Fixture = fixture("subvolume-refused");
    let malformed: Observed = run_wrapped(&fixture, fixture.root.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume", "--rollback", "--remove-old"]);
    assert_eq!(malformed.code, Some(2), "{}", stderr_text(&malformed));
    assert!(stderr_text(&malformed).contains("exclude each other"), "{}", stderr_text(&malformed));
    let unknown: Observed = run_wrapped(&fixture, fixture.root.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume", "--force"]);
    assert_eq!(unknown.code, Some(2));
    assert!(stderr_text(&unknown).contains("unknown argument --force"));
    let outside: Observed = run_wrapped(&fixture, fixture.root.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume"]);
    assert_eq!(outside.code, Some(1), "{}", stderr_text(&outside));
    assert!(stderr_text(&outside).contains("not inside a Git working tree") && stderr_text(&outside).ends_with("Nothing was changed.\n"), "{}", stderr_text(&outside));
    assert!(!fixture.root.join("repo.subvol-new").exists());
    remove(&fixture);
}

/// The whole life of the command through the executable: preview, migration, a repeat, and the deletion of the old
///  tree, each with the status and the words a person sees, on a fixture that is on btrfs.
#[test]
fn the_command_migrates_a_checkout_previews_repeats_and_deletes_the_old_tree() {
    let fixture: Fixture = fixture("subvolume-life");
    if !on_btrfs_or_acknowledged(&fixture, "subvolume-life") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture);
    let preview: Observed = run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume", "--dry-run"]);
    assert_eq!(preview.code, Some(0), "{}", stderr_text(&preview));
    assert!(stderr_text(&preview).contains("dry run complete") && !fixture.root.join("repo.before-subvolume").exists());
    assert_ne!(inode(repo.as_path()), SUBVOLUME_ROOT_INODE);
    let migrated: Observed = run_wrapped(&fixture, repo.as_path().join("src").as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume"]);
    assert_eq!(migrated.code, Some(0), "{}", stderr_text(&migrated));
    let text: String = stderr_text(&migrated);
    assert!(text.contains("is now a btrfs subvolume") && text.contains("--remove-old") && text.contains("migration took"), "{text}");
    assert_eq!(inode(repo.as_path()), SUBVOLUME_ROOT_INODE);
    assert_eq!(stdout_text(&run_direct(&fixture, repo.as_path(), &["status", "--porcelain"])), "");
    assert_eq!(std::fs::read(repo.join("ignored/pkg/index.js")).expect("ignored file"), b"module.exports = 1;\n");
    let again: Observed = run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume"]);
    assert_eq!(again.code, Some(0));
    assert!(stderr_text(&again).contains("already a btrfs subvolume"), "{}", stderr_text(&again));
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume", "--remove-old"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_text(&removed));
    assert!(!fixture.root.join("repo.before-subvolume").exists() && repo.join("README.md").exists());
    remove(&fixture);
}

/// The rollback through the executable restores an ordinary directory and leaves Git clean.
#[test]
fn the_rollback_restores_the_ordinary_directory() {
    let fixture: Fixture = fixture("subvolume-rollback");
    if !on_btrfs_or_acknowledged(&fixture, "subvolume-rollback") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture);
    let migrated: Observed = run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume"]);
    assert_eq!(migrated.code, Some(0), "{}", stderr_text(&migrated));
    let rolled: Observed = run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "migrate-to-subvolume", "--rollback"]);
    assert_eq!(rolled.code, Some(0), "{}", stderr_text(&rolled));
    assert!(stderr_text(&rolled).contains("rolled back"), "{}", stderr_text(&rolled));
    assert_ne!(inode(repo.as_path()), SUBVOLUME_ROOT_INODE);
    assert_eq!(stdout_text(&run_direct(&fixture, repo.as_path(), &["status", "--porcelain"])), "");
    assert!(!fixture.root.join("repo.before-subvolume").exists() && !fixture.root.join("repo.subvol-new").exists());
    remove(&fixture);
}

/// The standard error from the first line of the filesystem section on.
fn from_filesystem_section(text: &str) -> String {
    let start: usize = text.find(SECTION_MARKER).expect("the filesystem section is on standard error");
    return String::from(&text[start..]);
}

/// The doctor ends its standard error with the filesystem section for the checkout's own filesystem, keeps its exit
///  status, and the section names the command that applies.
#[test]
fn the_doctor_ends_with_the_filesystem_section_and_keeps_its_status() {
    let fixture: Fixture = fixture("subvolume-doctor");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let report: Observed = run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "doctor"]);
    assert_eq!(report.code, Some(0), "{}", stderr_text(&report));
    assert_eq!(stdout_text(&report), "");
    // The fsmonitor line comes before the section, which ends the standard error.
    let text: String = from_filesystem_section(stderr_text(&report).as_str());
    assert!(text.starts_with(SECTION_MARKER) && text.contains(repo.to_str().expect("UTF-8")), "{text}");
    let kind = filesystem_kind(repo.as_path()).expect("kind");
    let expected: &str = match kind {
        FilesystemKind::Btrfs => "btrfs",
        FilesystemKind::Overlayfs => "overlayfs",
        FilesystemKind::Other => "other than btrfs or overlayfs",
    };
    assert!(text.lines().next().is_some_and(|line: &str| { return line.contains(expected); }), "{text}");
    // Git chose the repository, so `-C` from a directory that is no repository reports the same section.
    let chosen: Observed = run_wrapped(&fixture, fixture.root.as_path(), &["-C", repo.to_str().expect("UTF-8"), "monochromatic-git-policy-cli", "doctor"]);
    assert_eq!(chosen.code, Some(0), "{}", stderr_text(&chosen));
    assert_eq!(from_filesystem_section(stderr_text(&chosen).as_str()), text);
    if kind == FilesystemKind::Btrfs && inode(repo.as_path()) != SUBVOLUME_ROOT_INODE {
        assert!(text.contains("git monochromatic-git-policy-cli migrate-to-subvolume --dry-run") && text.contains("can help with performance"), "{text}");
    }
    remove(&fixture);
}
