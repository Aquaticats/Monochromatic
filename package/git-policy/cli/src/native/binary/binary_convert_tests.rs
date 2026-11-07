//! What:
//!  `git monochromatic-git-policy-cli convert-worktree-to-subvolume` through the built executable,
//! in the environment a person's shell gives it:
//!  no forward-target variable,
//! which the wrapper puts only into the environment of the Git processes it starts.
//! Why:
//!  The conversion reads the Git configuration and the system attributes file in process,
//! and where those files are is a
//!  fact only the real Git the wrapper forwards to can give (`git var GIT_CONFIG_SYSTEM`,
//! `git var GIT_ATTR_SYSTEM`).
//! A
//!  worktree whose index entries the stat data cannot vouch for is judged by content,
//! and that judgment must know the real
//!  Git without being handed the variable by its own caller:
//! the command is started by a person,
//! not by the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['monochromatic-git-policy-cli', 'convert-worktree-to-subvolume', '--dry-run', 'wt'])).not.toContain('without running Git');
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, observe, remove, repository, wrapped};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The words of the refusal that says the configuration cannot be read without starting Git.
const CONFIGURATION_UNKNOWN: &str = "the Git configuration cannot be read without running Git";

/// The words of the refusal that says the system attributes file cannot be located without starting Git.
const SYSTEM_ATTRIBUTES_UNKNOWN: &str = "the location of the system attributes file cannot be established without running Git";

/// The words with which a worktree that really differs from its commit is refused.
const DIFFERS: &str = "differs from the committed content";

/// How far back a tracked file's modification time is set,
/// so its index entry no longer matches it.
const STALE_BY: std::time::Duration = std::time::Duration::from_secs(900);

/// Standard error as text.
fn stderr_text(observed: &Observed) -> String {
    return String::from_utf8_lossy(&observed.stderr).into_owned();
}

/// A repository with one tracked file and two linked worktrees inside it,
/// `clean` and `edited`,
/// whose index entries the
///  stat data cannot vouch for:
/// the tracked file's modification time is moved,
/// and in `edited` its bytes are changed too.
fn repository_with_unvouched_worktrees(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"committed\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "tracked.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    for name in ["clean", "edited"] {
        git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "--detach", name]);
    }
    let exclude: PathBuf = repo.join(".git/info/exclude");
    std::fs::write(&exclude, "/clean/\n/edited/\n").expect("exclude");
    std::fs::write(repo.join("edited/tracked.txt"), b"changed after the commit\n").expect("edit");
    for name in ["clean", "edited"] {
        let file = std::fs::OpenOptions::new().write(true).open(repo.join(name).join("tracked.txt")).expect("open for the time");
        file.set_modified(std::time::SystemTime::now() - STALE_BY).expect("move the modification time");
    }
    return repo;
}

/// The dry run of the conversion for a worktree,
/// started as a person starts it:
/// the fixture's environment,
/// and `unset`
///  variables removed from it.
fn dry_run(fixture: &Fixture, repo: &Path, worktree: &str, unset: &[&str]) -> Observed {
    let mut command = wrapped(fixture);
    command.current_dir(repo).args(["monochromatic-git-policy-cli", "convert-worktree-to-subvolume", "--dry-run", worktree]);
    for name in unset {
        command.env_remove(name);
    }
    return observe(&mut command, b"");
}

/// Red until the conversion knows the real Git it forwards to:
/// with the system configuration file and the system
///  attributes file both left to the real Git (the person's own environment,
/// no `GIT_CONFIG_NOSYSTEM`),
/// a worktree whose
///  stat data does not vouch for it is judged by content.
/// The unchanged one is not refused for a reason about reading
///  Git's files,
/// and the changed one is refused because it differs,
/// which proves the content was compared.
#[test]
fn a_worktree_the_stat_data_cannot_vouch_for_is_judged_by_content_not_refused_for_not_knowing_git() {
    let fixture: Fixture = fixture("convert-knows-real-git");
    let repo: PathBuf = repository_with_unvouched_worktrees(&fixture);
    let clean: Observed = dry_run(&fixture, repo.as_path(), "clean", &["GIT_CONFIG_NOSYSTEM"]);
    let clean_text: String = stderr_text(&clean);
    assert!(!clean_text.contains(CONFIGURATION_UNKNOWN), "{clean_text}");
    assert!(!clean_text.contains(SYSTEM_ATTRIBUTES_UNKNOWN), "{clean_text}");
    assert!(!clean_text.contains("is not quiet and unmodified"), "{clean_text}");
    let edited: Observed = dry_run(&fixture, repo.as_path(), "edited", &["GIT_CONFIG_NOSYSTEM"]);
    let edited_text: String = stderr_text(&edited);
    assert!(edited_text.contains(DIFFERS), "{edited_text}");
    remove(&fixture);
}

/// Red for the same defect through the other door:
/// the fixture's own `GIT_CONFIG_NOSYSTEM` removes the system
///  configuration file,
/// so the configuration is readable,
/// but the system attributes file (`GIT_ATTR_SYSTEM`) is still
///  the real Git's to locate.
#[test]
fn the_system_attributes_file_is_located_through_the_real_git_too() {
    let fixture: Fixture = fixture("convert-knows-attributes");
    let repo: PathBuf = repository_with_unvouched_worktrees(&fixture);
    let clean: Observed = dry_run(&fixture, repo.as_path(), "clean", &[]);
    let clean_text: String = stderr_text(&clean);
    assert!(!clean_text.contains(SYSTEM_ATTRIBUTES_UNKNOWN), "{clean_text}");
    assert!(!clean_text.contains("is not quiet and unmodified"), "{clean_text}");
    let edited: Observed = dry_run(&fixture, repo.as_path(), "edited", &[]);
    assert!(stderr_text(&edited).contains(DIFFERS), "{}", stderr_text(&edited));
    remove(&fixture);
}
