//! What:
//!  The cheap-worktree ladder through the built executable:
//! a worktree added or removed through the wrapper on btrfs is
//!  a subvolume,
//! a forwarded removal destroys it,
//! and every form the ladder does not serve keeps Git's own behavior.
//! Why:
//!  The ladder is a promise about the program a caller runs as `git`:
//! the destination,
//! the exit status and the
//!  caller's streams are what real Git gives,
//! whatever rung served the command.
//! The runs that need btrfs print one loud
//!  line and stop when the fixture root is elsewhere and the run says that is known;
//! without the variable they fail.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(run(wrappedGit, ['worktree', 'add', '-b', 'topic', '../topic'])).toMatchObject({ status: 0 });
//! // expect(isSubvolumeRoot('../topic')).toBe(true);
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, on_btrfs_or_acknowledged, remove, repository, run_direct, run_wrapped, stderr_of};
use git_policy_cli::fs_btrfs_ioctl::subvolume_create_at;
use git_policy_cli::fs_kind::is_subvolume_root;
use git_policy_cli::fs_mount_options::read_mount_facts;
use git_policy_cli::worktree_remove::DESTROY_NOTICE;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Whether the mount holding `path` has the option that lets a regular user destroy a subvolume.
fn removal_allowed(path: &Path) -> bool {
    return read_mount_facts(path).ok().flatten().is_some_and(|facts| { return facts.has_option("user_subvol_rm_allowed"); });
}

/// A repository with tracked files,
/// an ignored directory and a committed `.gitignore`;
/// the main checkout is a subvolume
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

/// A worktree added from the main checkout is a subvolume with its files,
/// is registered,
/// and the caller sees only
///  what real Git prints and the summary line of the copy.
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
    // Git's own words, then the one summary line of the ignored-state copy: this main checkout is no subvolume, so no snapshot
    //  served it and the copy ran.
    let wrapped: String = stderr_of(&created).replace("topic", "NAME");
    let git_words: String = stderr_of(&control).replace("control", "NAME");
    assert!(wrapped.starts_with(git_words.as_str()), "the caller sees Git's own words first: {wrapped}");
    // Then the line that says why this was not a snapshot (the main checkout here is no subvolume), then the copy summary.
    let after_git: Vec<&str> = wrapped[git_words.len()..].lines().collect();
    assert!(after_git.first().is_some_and(|line: &&str| { return line.starts_with("monochromatic-git-policy-cli: this worktree was not made as a snapshot"); }), "{wrapped}");
    assert!(after_git.get(1).is_some_and(|line: &&str| { return line.starts_with("monochromatic-git-policy-cli: copied "); }), "{wrapped}");
    let topic: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(topic.as_path()).expect("probe"), "the new worktree is a subvolume");
    assert!(!is_subvolume_root(fixture.root.join("control").as_path()).expect("probe"), "real Git makes an ordinary directory");
    assert_eq!(std::fs::read(topic.join("src/deep/leaf.txt")).expect("checked out").len(), 5_000);
    assert!(listing(&fixture, repo.as_path()).contains(format!("worktree {}\n", topic.display()).as_str()));
    remove(&fixture);
}

/// A worktree added from a linked worktree is a subvolume too,
/// and the ignored state it carries is copied into it.
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

/// The opt-out and the forms outside the table keep Git's ordinary directory;
/// a failed add leaves no destination.
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
///  one-call destroy or not;
/// a modified one is refused with Git's message and kept.
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

/// A clean subvolume worktree named by the last component of its path is removed by the one-call destroy:
/// Git picks a worktree by the last path components of every registered path before it resolves a real path
/// (`find_worktree` in Git's `worktree.c`),
/// so the wrapper serves that name too.
/// Why:
///  A person names the worktree the way Git lets them name it;
///  a typed branch name that Git resolves must not fall back to Git's file-by-file walk of a subvolume.
#[test]
fn a_worktree_named_by_its_last_component_is_destroyed_in_one_step() {
    let fixture: Fixture = fixture("ladder-remove-last-component");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-remove-last-component") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let topic: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(topic.as_path()).expect("probe"), "the add made a subvolume");
    age_tracked_files(&fixture, topic.as_path());
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "topic"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    assert!(!topic.exists(), "the worktree is gone");
    assert!(!listing(&fixture, repo.as_path()).contains("/topic\n"), "the registration is gone");
    let said: String = stderr_of(&removed);
    if removal_allowed(fixture.root.as_path()) {
        // The sentence is the one the removal owns, so a rewording of what it says the commit service then deletes does
        //  not read as a rung that went missing.
        assert!(said.contains(DESTROY_NOTICE), "the one-call destroy is stated: {said}");
    }
    remove(&fixture);
}

/// A name that differs from the registered worktree's last component only by case is Git's own decision:
/// on a repository whose `core.ignorecase` is false Git's suffix match compares the bytes,
/// so Git answers that the name is not a working tree.
/// Why:
///  Serving that name removed a worktree the person did not name,
///  with every ignored file it held and no other copy,
///  while Git's own refusal was the answer they would have got.
#[test]
fn a_name_that_differs_only_by_case_is_left_to_git() {
    let fixture: Fixture = fixture("ladder-remove-other-case");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-remove-other-case") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    // Git's comparison folds case only when this setting says so, and this is the case-sensitive filesystem's default.
    git(&fixture, repo.as_path(), &["config", "core.ignorecase", "false"]);
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let topic: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(topic.as_path()).expect("probe"), "the add made a subvolume");
    std::fs::create_dir_all(topic.join("ignored")).expect("ignored directory");
    std::fs::write(topic.join("ignored/only-copy.txt"), b"the only copy\n").expect("ignored file");
    age_tracked_files(&fixture, topic.as_path());
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "TOPIC"]);
    assert_ne!(refused.code, Some(0), "Git refuses a name that is not this worktree's: {}", stderr_of(&refused));
    assert!(stderr_of(&refused).contains("is not a working tree"), "Git's own refusal: {}", stderr_of(&refused));
    assert_eq!(std::fs::read(topic.join("ignored/only-copy.txt")).expect("the ignored only copy is kept"), b"the only copy\n");
    assert!(topic.join("README.md").is_file(), "the worktree is kept");
    assert!(listing(&fixture, repo.as_path()).contains("/topic\n"), "the registration is kept");
    remove(&fixture);
}

/// Make a freshly added worktree provably clean:
/// Git trusts the stat fields of a file only when the file is older than the
///  index,
/// so the tracked files are aged and the index refreshed,
/// as a checkout that has been there a while is.
fn age_tracked_files(fixture: &Fixture, worktree: &Path) {
    for name in ["README.md", ".gitignore", "src/deep/leaf.txt"] {
        let Ok(file) = std::fs::OpenOptions::new().write(true).open(worktree.join(name)) else { continue };
        file.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(120)).expect("age the tracked file");
    }
    git(fixture, worktree, &["update-index", "--refresh"]);
}

/// A worktree that another repository registered,
/// and an empty name,
/// are not removed:
/// the wrapper leaves both to Git,
/// which
///  answers that the name is not a working tree.
#[test]
fn a_worktree_of_another_repository_and_an_empty_name_are_not_removed() {
    let fixture: Fixture = fixture("ladder-remove-foreign");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-remove-foreign") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    let other: PathBuf = repository(&fixture, OsStr::new("other"));
    std::fs::write(other.join("README.md"), b"readme\n").expect("readme");
    git(&fixture, other.as_path(), &["add", "README.md"]);
    git(&fixture, other.as_path(), &["commit", "--quiet", "--message=files"]);
    for (repository_root, branch) in [(other.as_path(), "foreign"), (repo.as_path(), "own")] {
        let created: Observed = run_wrapped(&fixture, repository_root, &["worktree", "add", "--quiet", "-b", branch, format!("../{branch}").as_str()]);
        assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    }
    age_tracked_files(&fixture, fixture.root.join("own").as_path());
    age_tracked_files(&fixture, fixture.root.join("foreign").as_path());
    let foreign: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../foreign"]);
    assert_eq!(foreign.code, Some(128), "{}", stderr_of(&foreign));
    assert!(stderr_of(&foreign).contains("is not a working tree"), "{}", stderr_of(&foreign));
    assert!(fixture.root.join("foreign/.git").is_file(), "the other repository's worktree is untouched");
    let own_root: PathBuf = fixture.root.join("own");
    let empty: Observed = run_wrapped(&fixture, own_root.as_path(), &["worktree", "remove", ""]);
    assert_eq!(empty.code, Some(128), "{}", stderr_of(&empty));
    assert!(stderr_of(&empty).contains("is not a working tree"), "{}", stderr_of(&empty));
    assert!(own_root.join("README.md").is_file(), "an empty name does not name the current worktree");
    remove(&fixture);
}

/// Where the destroy runs but the registration cannot be removed,
/// the command fails,
/// says which directory is left and which
///  command finishes the removal,
/// and that command works.
#[test]
fn a_registration_left_after_the_destroy_fails_the_command_with_the_repair_steps() {
    use std::os::unix::fs::PermissionsExt;
    let fixture: Fixture = fixture("ladder-remove-incomplete");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-remove-incomplete") {
        remove(&fixture);
        return;
    }
    if !removal_allowed(fixture.root.as_path()) {
        eprintln!("CONTROL NOT RUN: ladder-remove-incomplete (the mount of {} lacks user_subvol_rm_allowed)", fixture.root.display());
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "clean", "../clean"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    age_tracked_files(&fixture, fixture.root.join("clean").as_path());
    let worktrees: PathBuf = repo.join(".git/worktrees");
    std::fs::set_permissions(worktrees.as_path(), std::fs::Permissions::from_mode(0o555)).expect("read-only registrations");
    let failed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../clean"]);
    std::fs::set_permissions(worktrees.as_path(), std::fs::Permissions::from_mode(0o755)).expect("restore");
    assert_ne!(failed.code, Some(0), "{}", stderr_of(&failed));
    assert!(!fixture.root.join("clean").exists(), "the files were destroyed");
    let report: String = stderr_of(&failed);
    assert!(report.contains(worktrees.join("clean").to_str().expect("UTF-8")), "the report names the leftover directory: {report}");
    assert!(report.contains(format!("git -C {} worktree prune", repo.display()).as_str()), "and the command that finishes the removal: {report}");
    let finished: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "prune"]);
    assert_eq!(finished.code, Some(0), "{}", stderr_of(&finished));
    assert!(!listing(&fixture, repo.as_path()).contains("/clean\n"));
    assert!(!worktrees.join("clean").exists(), "the prune removed the leftover registration");
    remove(&fixture);
}

/// Where the destroy runs,
/// the command says on standard error that it cannot be undone and what the person accepts.
#[test]
fn the_destroy_of_a_subvolume_worktree_says_it_cannot_be_undone() {
    let fixture: Fixture = fixture("ladder-remove-notice");
    if !on_btrfs_or_acknowledged(&fixture, "ladder-remove-notice") {
        remove(&fixture);
        return;
    }
    if !removal_allowed(fixture.root.as_path()) {
        eprintln!("CONTROL NOT RUN: ladder-remove-notice (the mount of {} lacks user_subvol_rm_allowed)", fixture.root.display());
        remove(&fixture);
        return;
    }
    let repo: PathBuf = populated_repository(&fixture, false);
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "notice", "../notice"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    age_tracked_files(&fixture, fixture.root.join("notice").as_path());
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../notice"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    assert!(!fixture.root.join("notice").exists());
    assert!(stderr_of(&removed).contains("this cannot be undone"), "{}", stderr_of(&removed));
    remove(&fixture);
}
