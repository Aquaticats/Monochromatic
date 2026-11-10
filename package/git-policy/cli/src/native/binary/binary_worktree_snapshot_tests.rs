//! What:
//!  Rung 1 of the cheap-worktree ladder through the built executable:
//! `git worktree add` from a clean subvolume worktree
//!  is served by a snapshot,
//! and every worktree it makes equals what Git's own checkout makes.
//! Why:
//!  The snapshot carries the source's ignored state,
//! which Git's checkout from a main checkout never does,
//! so that is the
//!  sign a snapshot served the command;
//! the contents,
//! the status of the new worktree,
//! Git's words,
//! the `post-checkout`
//!  hook and the exit status are compared with a control worktree that real Git made from the same commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(read('../topic/ignored/pkg/index.js')).toBe(read('ignored/pkg/index.js'));
//! // expect(treeOf('../topic')).toEqual(treeOf('../control'));
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, on_btrfs_or_acknowledged, remove, run_direct, run_wrapped, stderr_of};
use git_policy_cli::fs_btrfs_ioctl::subvolume_create_at;
use git_policy_cli::fs_kind::is_subvolume_root;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Move a path's modification time two minutes into the past without following a link,
/// so index entries are not racy.
pub(super) fn age(path: &Path) {
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in a fixture path");
    let past: libc::time_t = (std::time::SystemTime::now() - std::time::Duration::from_secs(120))
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("after the epoch")
        .as_secs() as libc::time_t;
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: past, tv_nsec: 0 }, libc::timespec { tv_sec: past, tv_nsec: 0 }];
    // SAFETY: `name` is a NUL-terminated string that outlives the call and `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "age {}", path.display());
}

/// Write a tracked file with its parents,
/// aged.
pub(super) fn tracked(repo: &Path, relative: &str, content: &[u8]) {
    let path: PathBuf = repo.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
    std::fs::write(path.as_path(), content).expect("tracked file");
    age(path.as_path());
}

/// Commit everything in the repository and refresh its index,
/// so every entry is provably unchanged.
pub(super) fn commit_all(fixture: &Fixture, repo: &Path, message: &str) {
    git(fixture, repo, &["add", "--all"]);
    git(fixture, repo, &["commit", "--quiet", format!("--message={message}").as_str()]);
    git(fixture, repo, &["update-index", "--refresh"]);
}

/// A main checkout that is a subvolume with two commits:
/// the first holds `old.txt` and an older `shared.txt`,
/// the
///  second changes `shared.txt`,
/// adds `new/added.txt`,
/// and deletes `old.txt`.
/// `ignored/pkg/index.js` is ignored state.
pub(super) fn subvolume_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = fixture.root.join("repo");
    subvolume_create_at(repo.as_path()).expect("repository subvolume");
    git(fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    tracked(repo.as_path(), ".gitignore", b"ignored/\n");
    tracked(repo.as_path(), "old.txt", b"old\n");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    tracked(repo.as_path(), "src/deep/leaf.txt", &vec![b'x'; 5_000]);
    commit_all(fixture, repo.as_path(), "first");
    std::fs::remove_file(repo.join("old.txt")).expect("delete");
    tracked(repo.as_path(), "shared.txt", b"second, longer\n");
    tracked(repo.as_path(), "new/added.txt", b"added\n");
    commit_all(fixture, repo.as_path(), "second");
    std::fs::create_dir_all(repo.join("ignored/pkg")).expect("ignored directory");
    std::fs::write(repo.join("ignored/pkg/index.js"), b"module.exports = 1;\n").expect("ignored file");
    return repo;
}

/// The tracked files of a worktree with their bytes and executable bit,
/// sorted by path.
pub(super) fn tracked_content(fixture: &Fixture, worktree: &Path) -> Vec<(String, Vec<u8>, bool)> {
    use std::os::unix::fs::PermissionsExt;
    let listing = git(fixture, worktree, &["ls-files", "-z"]);
    let mut files: Vec<(String, Vec<u8>, bool)> = Vec::new();
    for name in listing.stdout.split(|byte| { return *byte == 0; }).filter(|name| { return !name.is_empty(); }) {
        let path: PathBuf = worktree.join(std::ffi::OsStr::from_bytes(name));
        let executable: bool = std::fs::metadata(path.as_path()).expect("metadata").permissions().mode() & 0o111 != 0;
        files.push((String::from_utf8_lossy(name).into_owned(), std::fs::read(path.as_path()).expect("content"), executable));
    }
    return files;
}

/// A worktree added from the clean subvolume main checkout is a snapshot:
/// it carries the ignored state,
/// equals the
///  control Git made,
/// is clean on its first status,
/// and the caller sees Git's own words.
#[test]
fn a_clean_main_checkout_is_snapshotted_and_equals_gits_own_checkout() {
    let fixture: Fixture = fixture("snapshot-main");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-main") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "-b", "topic", "../topic"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "-b", "control", "../control"]);
    // Git's own words, then the one line that says a snapshot of the main checkout made the worktree. The trash a routine
    //  snapshot add hands over is the one it made itself (the stand-ins and its own `.git` copy), and its hand-over says
    //  nothing.
    let wrapped: String = stderr_of(&created).replace("topic", "NAME");
    let git_words: String = stderr_of(&control).replace("control", "NAME");
    assert!(wrapped.starts_with(git_words.as_str()), "Git's own words come first: {wrapped}");
    assert_eq!(&wrapped[git_words.len()..], git_policy_cli::worktree_trash::MAIN_SNAPSHOT_NOTE, "a routine hand-over says nothing of its own: {wrapped}");
    assert_eq!(String::from_utf8_lossy(&created.stdout), String::from_utf8_lossy(&control.stdout), "Git's checkout line");
    let topic: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(topic.as_path()).expect("probe"));
    assert_eq!(std::fs::read(topic.join("ignored/pkg/index.js")).expect("the snapshot carries ignored state"), b"module.exports = 1;\n");
    assert_eq!(tracked_content(&fixture, topic.as_path()), tracked_content(&fixture, fixture.root.join("control").as_path()));
    assert_eq!(git(&fixture, topic.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]).stdout, b"", "clean on the first status");
    assert_eq!(git(&fixture, topic.as_path(), &["rev-parse", "HEAD"]).stdout, git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout);
    assert!(std::fs::read(topic.join(".git")).expect("pointer").starts_with(b"gitdir: "));
    assert!(!repo.join(".git/index.lock").exists());
    remove(&fixture);
}

/// Whether a worktree holds the ignored file only a snapshot carries from a main checkout.
pub(super) fn carries_ignored_state(worktree: &Path) -> bool {
    return worktree.join("ignored/pkg/index.js").is_file();
}

/// Add a worktree through the wrapper and one through real Git from the same place,
/// and require both to succeed.
pub(super) fn add_pair(fixture: &Fixture, from: &Path, wrapped_name: &str, control_name: &str, extra: &[&str]) -> (Observed, Observed) {
    let mut wrapped_arguments: Vec<String> = vec![String::from("worktree"), String::from("add")];
    wrapped_arguments.extend(extra.iter().map(|word: &&str| { return word.replace("NAME", wrapped_name); }));
    let mut control_arguments: Vec<String> = vec![String::from("worktree"), String::from("add")];
    control_arguments.extend(extra.iter().map(|word: &&str| { return word.replace("NAME", control_name); }));
    let created: Observed = run_wrapped(fixture, from, &wrapped_arguments.iter().map(String::as_str).collect::<Vec<&str>>());
    let control: Observed = run_direct(fixture, from, &control_arguments.iter().map(String::as_str).collect::<Vec<&str>>());
    assert_eq!(created.code, control.code, "{} against {}", stderr_of(&created), stderr_of(&control));
    return (created, control);
}

/// A target commit older than the source's applies the difference (a changed file,
/// an added one,
/// a deleted one) and
///  equals the control.
#[test]
fn an_older_target_commit_gets_the_difference_applied() {
    let fixture: Fixture = fixture("snapshot-older");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-older") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let (created, _control) = add_pair(&fixture, repo.as_path(), "old", "oldcontrol", &["--detach", "../NAME", "HEAD~1"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let old: PathBuf = fixture.root.join("old");
    assert!(carries_ignored_state(old.as_path()), "a snapshot served the command");
    assert_eq!(std::fs::read(old.join("shared.txt")).expect("shared"), b"first\n");
    assert!(old.join("old.txt").is_file());
    assert!(!old.join("new/added.txt").exists());
    assert_eq!(tracked_content(&fixture, old.as_path()), tracked_content(&fixture, fixture.root.join("oldcontrol").as_path()));
    assert_eq!(git(&fixture, old.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    remove(&fixture);
}

/// A source with an entry no reset can undo (skip-worktree here) is not snapshotted:
/// Git checks the worktree out,
/// into a subvolume.
#[test]
fn a_source_with_an_entry_no_reset_can_undo_falls_to_gits_checkout() {
    let fixture: Fixture = fixture("snapshot-skipped");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-skipped") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    git(&fixture, repo.as_path(), &["update-index", "--skip-worktree", "shared.txt"]);
    let (created, _control) = add_pair(&fixture, repo.as_path(), "plain", "plaincontrol", &["-b", "NAME", "../NAME"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let plain: PathBuf = fixture.root.join("plain");
    // No snapshot served it (a snapshot prints `copied 0` because it carries the ignored state itself); the ignored-state copy of
    //  the main checkout brought the ignored state after Git's checkout.
    assert!(!stderr_of(&created).contains("copied 0 ignored"), "{}", stderr_of(&created));
    assert!(carries_ignored_state(plain.as_path()), "the main checkout hands over its ignored state by copy");
    assert!(is_subvolume_root(plain.as_path()).expect("probe"), "Git filled a subvolume");
    assert_eq!(std::fs::read(plain.join("shared.txt")).expect("committed content"), b"second, longer\n");
    assert_eq!(tracked_content(&fixture, plain.as_path()), tracked_content(&fixture, fixture.root.join("plaincontrol").as_path()));
    remove(&fixture);
}

/// Differing `.gitattributes` files between the source and the target decline the snapshot.
#[test]
fn differing_attributes_decline_the_snapshot() {
    let fixture: Fixture = fixture("snapshot-attributes");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-attributes") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=lf\n");
    commit_all(&fixture, repo.as_path(), "attributes");
    let (created, _control) = add_pair(&fixture, repo.as_path(), "attr", "attrcontrol", &["--detach", "../NAME", "HEAD~1"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let attr: PathBuf = fixture.root.join("attr");
    assert!(!stderr_of(&created).contains("copied 0 ignored"), "no snapshot served it: {}", stderr_of(&created));
    assert!(carries_ignored_state(attr.as_path()), "the main checkout hands over its ignored state by copy");
    assert!(!attr.join(".gitattributes").exists());
    assert_eq!(tracked_content(&fixture, attr.as_path()), tracked_content(&fixture, fixture.root.join("attrcontrol").as_path()));
    remove(&fixture);
}

/// `--quiet` silences Git's checkout line on both rungs.
#[test]
fn quiet_silences_the_checkout_line() {
    let fixture: Fixture = fixture("snapshot-quiet");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-quiet") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let (created, control) = add_pair(&fixture, repo.as_path(), "quiet", "quietcontrol", &["--quiet", "-b", "NAME", "../NAME"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert!(created.stdout.is_empty() && control.stdout.is_empty(), "{}", String::from_utf8_lossy(&created.stdout));
    assert!(carries_ignored_state(fixture.root.join("quiet").as_path()));
    remove(&fixture);
}

/// No hook runs on any rung through the wrapper,
/// which switches hooks off for every Git start;
/// real Git started directly
///  runs the same hook,
/// which is the positive control that the hook below would have been noticed.
#[test]
fn no_post_checkout_hook_runs_through_the_wrapper() {
    use std::os::unix::fs::PermissionsExt;
    let fixture: Fixture = fixture("snapshot-hook");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-hook") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let marker: PathBuf = fixture.root.join("marker");
    let hook: PathBuf = repo.join(".git/hooks/post-checkout");
    std::fs::write(hook.as_path(), format!("#!/bin/sh\nprintf '%s\\n' \"$(pwd -P)\" >> '{}'\n", marker.display())).expect("hook");
    std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    let (snapshot, _control) = add_pair(&fixture, repo.as_path(), "hooked", "hookcontrol", &["-b", "NAME", "../NAME"]);
    assert_eq!(snapshot.code, Some(0), "{}", stderr_of(&snapshot));
    assert!(carries_ignored_state(fixture.root.join("hooked").as_path()), "a snapshot served the command");
    let lines: String = std::fs::read_to_string(marker.as_path()).expect("real Git ran the hook for the control");
    let canonical_root: PathBuf = std::fs::canonicalize(fixture.root.as_path()).expect("canonical");
    assert!(lines.lines().any(|line: &str| { return Path::new(line) == canonical_root.join("hookcontrol"); }), "{lines}");
    assert!(!lines.lines().any(|line: &str| { return Path::new(line) == canonical_root.join("hooked"); }), "the wrapper ran no hook: {lines}");
    std::fs::write(repo.join("shared.txt"), b"dirty edit\n").expect("make the source dirty");
    let plain: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "plain", "../plain"]);
    assert_eq!(plain.code, Some(0), "{}", stderr_of(&plain));
    let after: String = std::fs::read_to_string(marker.as_path()).expect("marker");
    assert!(!after.lines().any(|line: &str| { return Path::new(line) == canonical_root.join("plain"); }), "{after}");
    remove(&fixture);
}

/// A worktree made by a snapshot is itself a source:
/// the next add from it is a snapshot too.
#[test]
fn a_snapshot_worktree_is_a_source_in_turn() {
    let fixture: Fixture = fixture("snapshot-chain");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-chain") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    let first: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "first", "../first"]);
    assert_eq!(first.code, Some(0), "{}", stderr_of(&first));
    std::fs::write(fixture.root.join("first/ignored/pkg/more.js"), b"more\n").expect("more ignored state");
    let second: Observed = run_wrapped(&fixture, fixture.root.join("first").as_path(), &["worktree", "add", "--quiet", "-b", "second", "../second"]);
    assert_eq!(second.code, Some(0), "{}", stderr_of(&second));
    let second_root: PathBuf = fixture.root.join("second");
    assert_eq!(std::fs::read(second_root.join("ignored/pkg/more.js")).expect("carried from the linked source"), b"more\n");
    assert_eq!(git(&fixture, second_root.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    remove(&fixture);
}

/// A destination inside the source (an ignored `.claude-wt` directory) gets no copy of itself,
/// and the source stays
///  clean;
/// a nested worktree that is an ordinary directory is set aside in the snapshot of the main checkout.
#[test]
fn a_destination_inside_the_source_is_not_carried_into_itself() {
    let fixture: Fixture = fixture("snapshot-nested");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-nested") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    std::fs::create_dir(repo.join(".claude-wt")).expect("the ignored worktree directory");
    tracked(repo.as_path(), ".gitignore", b"ignored/\n.claude-wt/\n");
    commit_all(&fixture, repo.as_path(), "ignore the worktree directory");
    let inside: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "inside", ".claude-wt/inside"]);
    assert_eq!(inside.code, Some(0), "{}", stderr_of(&inside));
    let inside_root: PathBuf = repo.join(".claude-wt/inside");
    assert!(carries_ignored_state(inside_root.as_path()), "a snapshot served the command");
    assert!(!inside_root.join(".claude-wt").exists() || std::fs::read_dir(inside_root.join(".claude-wt")).expect("listing").count() == 0, "no copy of the destination in itself");
    assert_eq!(git(&fixture, repo.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    assert_eq!(git(&fixture, inside_root.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    // The ordinary directory worktree now inside the main checkout no longer declines the snapshot: it is set aside in the copy.
    std::fs::remove_dir_all(inside_root.as_path()).ok();
    git(&fixture, repo.as_path(), &["worktree", "prune"]);
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "ordinary", ".claude-wt/ordinary"]);
    let declined: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "after", "../after"]);
    assert_eq!(declined.code, Some(0), "{}", stderr_of(&declined));
    let after: PathBuf = fixture.root.join("after");
    assert!(carries_ignored_state(after.as_path()), "the snapshot carried the ignored state");
    assert!(!after.join(".claude-wt/ordinary").exists(), "the ordinary nested worktree was set aside, not carried");
    assert_eq!(git(&fixture, repo.join(".claude-wt/ordinary").as_path(), &["status", "--porcelain=v1"]).stdout, b"", "the real nested worktree is untouched");
    assert_eq!(git(&fixture, after.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    remove(&fixture);
}

/// A snapshot of the main checkout leaves out what the rule names:
/// the ordinary nested worktree (its pointer unlinked),
/// the
///  caches and the run records are set aside under an ignored path and emptied;
/// what a new worktree needs stays,
/// and both
///  worktrees are clean.
#[test]
fn a_snapshot_of_main_sets_aside_what_the_rule_leaves_out() {
    let fixture: Fixture = fixture("snapshot-trash");
    if !on_btrfs_or_acknowledged(&fixture, "snapshot-trash") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    tracked(repo.as_path(), ".gitignore", b"ignored/\n.claude/\nnode_modules/\n");
    commit_all(&fixture, repo.as_path(), "ignore the places the rule works on");
    for (path, bytes) in [("node_modules/pkg/index.js", b"module\n".as_slice()), ("node_modules/.cache/jiti/x.js", b"cache\n"), ("node_modules/.monochromatic/log.jsonl", b"log\n")] {
        let full: PathBuf = repo.join(path);
        std::fs::create_dir_all(full.parent().expect("parent")).expect("directories");
        std::fs::write(full, bytes).expect("ignored file");
    }
    std::fs::create_dir_all(repo.join(".claude/worktrees")).expect("worktrees directory");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "ordinary", ".claude/worktrees/ordinary"]);
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert!(created.stderr.is_empty(), "--quiet stays silent: {}", stderr_of(&created));
    let topic: PathBuf = fixture.root.join("topic");
    assert!(is_subvolume_root(topic.as_path()).expect("probe"));
    assert_eq!(std::fs::read(topic.join("node_modules/pkg/index.js")).expect("dependency"), b"module\n");
    assert!(carries_ignored_state(topic.as_path()));
    for gone in ["node_modules/.cache", "node_modules/.monochromatic", ".claude/worktrees/ordinary"] {
        assert!(std::fs::symlink_metadata(topic.join(gone)).is_err(), "{gone} was left out");
    }
    // The fixture has no session bus and no service for the new worktree yet, so no route to a service exists and the trash is
    //  emptied before the command returns; the record written for the service is withdrawn. The hand-over itself is controlled
    //  with an injected route (`worktree_trash_tests.rs`).
    assert!(!topic.join(".claude/cli-git-trash").exists(), "the trash is gone when the command returns");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let leftover: usize = std::fs::read_dir(admin.join("monochromatic-git-policy-cli/work/pending")).map_or(0, |listing| { return listing.flatten().count(); });
    assert_eq!(leftover, 0, "no work item outlives the foreground emptying");
    assert_eq!(git(&fixture, topic.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    assert_eq!(git(&fixture, repo.join(".claude/worktrees/ordinary").as_path(), &["status", "--porcelain=v1"]).stdout, b"", "the real nested worktree is untouched");
    remove(&fixture);
}
