//! What:
//!  The cheap-worktree ladder through the built executable: a worktree added or removed through the wrapper on btrfs is
//!  a subvolume, a forwarded removal destroys it, and every form the ladder does not serve keeps Git's own behavior.
//! Why:
//!  The ladder is a promise about the program a caller runs as `git`: the destination, the exit status and the
//!  caller's streams are what real Git gives, whatever rung served the command. The runs that need btrfs print one loud
//!  line and stop when the fixture root is elsewhere and the run says that is known; without the variable they fail.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['worktree', 'add', '-b', 'topic', '../topic'])).toMatchObject({ status: 0 });
//! // expect(isSubvolumeRoot('../topic')).toBe(true);
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, remove, repository, run_direct, run_wrapped, stderr_of};
use git_policy_cli::fs_btrfs_ioctl::subvolume_create_at;
use git_policy_cli::fs_kind::{FilesystemKind, filesystem_kind, is_subvolume_root};
use git_policy_cli::fs_mount_options::read_mount_facts;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

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

/// Whether the mount holding `path` has the option that lets a regular user destroy a subvolume.
fn removal_allowed(path: &Path) -> bool {
    return read_mount_facts(path).ok().flatten().is_some_and(|facts| { return facts.has_option("user_subvol_rm_allowed"); });
}

/// A repository with tracked files, an ignored directory and a committed `.gitignore`; the main checkout is a subvolume
///  when `as_subvolume` is set.
fn populated_repository(fixture: &Fixture, as_subvolume: bool) -> PathBuf {
    if as_subvolume {
        subvolume_create_at(fixture.root.join("repo").as_path()).expect("repository subvolume");
        git(fixture, fixture.root.join("repo").as_path(), &["init", "--quiet", "--initial-branch=main"]);
        git(fixture, fixture.root.join("repo").as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    }
    let repo: PathBuf = if as_subvolume { fixture.root.join("repo") } else { repository(fixture, OsStr::new("repo")) };
    std::fs::create_dir_all(repo.join("src/deep")).expect("directories");
    std::fs::write(repo.join("README.md"), b"readme\n").expect("readme");
    std::fs::write(repo.join("src/deep/leaf.txt"), vec![b'x'; 5_000]).expect("leaf");
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("gitignore");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
    return repo;
}

/// The porcelain `worktree list` of a repository as text.
fn listing(fixture: &Fixture, repo: &Path) -> String {
    return String::from_utf8_lossy(&git(fixture, repo, &["worktree", "list", "--porcelain"]).stdout).into_owned();
}

/// A worktree added from the main checkout is a subvolume with its files, is registered, and the caller sees only
///  what real Git prints.
#[test]
fn a_worktree_added_from_the_main_checkout_is_a_subvolume() {
    let fixture: Fixture = fixture("ladder-add-main");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-add-main") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "-b", "topic", "../topic"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "-b", "control", "../control"]);
    assert_eq!(stderr_of(&created).replace("topic", "NAME"), stderr_of(&control).replace("control", "NAME"), "the caller sees Git's own words");
    let topic: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(topic.as_path()).expect("probe"), "the new worktree is a subvolume");
    assert!(!is_subvolume_root(fixture.root.join("control").as_path()).expect("probe"), "real Git makes an ordinary directory");
    assert_eq!(std::fs::read(topic.join("src/deep/leaf.txt")).expect("checked out").len(), 5_000);
    assert!(listing(&fixture, repo.as_path()).contains(format!("worktree {}\n", topic.display()).as_str()));
    remove(&fixture);
}

/// A worktree added from a linked worktree is a subvolume too, and the ignored state it carries is copied into it.
#[test]
fn a_worktree_added_from_a_linked_worktree_is_a_subvolume_with_the_ignored_state() {
    let fixture: Fixture = fixture("ladder-add-linked");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-add-linked") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    let first: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "first", "../first"]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    let first_root: PathBuf = fixture.root.join("first");
    std::fs::create_dir_all(first_root.join("ignored/pkg")).expect("ignored directory");
    std::fs::write(first_root.join("ignored/pkg/index.js"), b"module.exports = 1;\n").expect("ignored file");
    let second: Observed = run_wrapped(&fixture, first_root.as_path(), &["worktree", "add", "--quiet", "-b", "second", "../second"]);
    assert_eq!(second.code, Some(0), "{}", stderr_of(&second));
    let second_root: PathBuf = fixture.root.join("second");
    assert!(is_subvolume_root(second_root.as_path()).expect("probe"));
    assert_eq!(std::fs::read(second_root.join("ignored/pkg/index.js")).expect("ignored state"), b"module.exports = 1;\n");
    assert_eq!(std::fs::read(second_root.join("README.md")).expect("tracked"), b"readme\n");
    remove(&fixture);
}

/// The opt-out and the forms outside the table keep Git's ordinary directory; a failed add leaves no destination.
#[test]
fn the_opt_out_and_unknown_forms_keep_git_s_ordinary_directory() {
    let fixture: Fixture = fixture("ladder-add-plain");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-add-plain") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    for (name, arguments) in [
        ("optout", vec!["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "optout", "../optout"]),
        ("forced", vec!["worktree", "add", "--force", "--quiet", "-b", "forced", "../forced"]),
        ("nocheckout", vec!["worktree", "add", "--no-checkout", "--quiet", "-b", "nocheckout", "../nocheckout"]),
    ] {
        let created: Observed = run_wrapped(&fixture, repo.as_path(), &arguments);
        assert_eq!(created.code, Some(0), "{name}: {}", stderr_of(&created));
        assert!(!is_subvolume_root(fixture.root.join(name).as_path()).expect("probe"), "{name}");
    }
    git(&fixture, repo.as_path(), &["branch", "taken"]);
    let failed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "-b", "taken", "../failed"]);
    let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "-b", "taken", "../failed-control"]);
    assert_eq!(failed.code, control.code);
    assert_ne!(failed.code, Some(0));
    assert!(!fixture.root.join("failed").exists(), "the failed add leaves nothing");
    remove(&fixture);
}

/// `git worktree remove` of a clean subvolume worktree removes it and its registration whether the mount allows the
///  one-call destroy or not; a modified one is refused with Git's message and kept.
#[test]
fn a_subvolume_worktree_is_removed_and_a_dirty_one_is_refused() {
    let fixture: Fixture = fixture("ladder-remove");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-remove") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    for name in ["clean", "dirty"] {
        let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", name, format!("../{name}").as_str()]);
        assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    }
    std::fs::write(fixture.root.join("dirty/README.md"), b"changed\n").expect("modify");
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../dirty"]);
    assert_eq!(refused.code, Some(128), "{}", stderr_of(&refused));
    assert!(stderr_of(&refused).contains("contains modified or untracked files, use --force to delete it"), "{}", stderr_of(&refused));
    assert!(fixture.root.join("dirty/README.md").is_file());
    // Real Git's refusal of the same state, as the control of the wording.
    let settled: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../clean"]);
    assert_eq!(settled.code, Some(0), "{}", stderr_of(&settled));
    assert!(!fixture.root.join("clean").exists());
    assert!(!listing(&fixture, repo.as_path()).contains("/clean\n"));
    eprintln!("the mount allows the one-call destroy: {}", removal_allowed(fixture.root.as_path()));
    remove(&fixture);
}
