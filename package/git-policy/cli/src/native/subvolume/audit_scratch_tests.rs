//! Throwaway probes for the subvolume-and-conversion audit.
//! Not committed: the module registration is added for the runs and removed again.

use crate::fs_beneath::{identity_of, open_directory_at};
use crate::subvolume_arguments::{MigrateArguments, MigrateMode};
use crate::subvolume_error::Interrupt;
use crate::subvolume_journal::Journal;
use crate::subvolume_platform::Platform;
use crate::subvolume_test_support::{PlainPlatform, backdate, checkout_with_worktree, make_checkout, migrate_arguments, run_command};
use crate::test_support::{fixture, git, remove};
use crate::worktree_convert::{ConvertOptions, Verdict, convert_worktree, converting_path, inspect};
use crate::worktree_convert_compare::first_difference;
use crate::worktree_convert_journal::{JOURNAL_NAME, read_plain_file};
use crate::worktree_convert_remove::{Removal, remove_recorded_old_tree};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

const RUN: ConvertOptions = ConvertOptions { dry_run: false, abandon: false, command_line_attributes: false };

/// A checkout with one linked worktree whose tracked files are old and whose index is fresh.
fn quiet_worktree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    let worktree: PathBuf = root.join("wt");
    backdate(worktree.as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    return (parent, root, std::fs::canonicalize(worktree).expect("canonical worktree"));
}

fn run_conversion(platform: &dyn Platform, worktree: &Path, options: ConvertOptions) -> (Result<Verdict, crate::subvolume_error::MigrateError>, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let verdict = convert_worktree(platform, worktree, options, &Interrupt::never(), &mut |text: &str| { lines.push(String::from(text)); });
    return (verdict, lines);
}

/// A Unix socket in the worktree: the copy skips it, the removal counts it as a difference,
/// and every rerun fails at the same step.
#[test]
fn a_socket_in_a_worktree_never_finishes_converting() {
    let (parent, _root, worktree) = quiet_worktree("sock");
    let platform: PlainPlatform = PlainPlatform::new();
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind(worktree.join("agent.sock")).expect("socket");
    let admin: PathBuf = inspect(worktree.as_path()).expect("registered").admin;
    for round in 1..=2 {
        let (verdict, lines) = run_conversion(&platform, worktree.as_path(), RUN);
        println!("--- round {round} verdict: {verdict:?}");
        for line in &lines {
            println!("    {line}");
        }
        println!("    worktree is a subvolume: {:?}", platform.is_subvolume_root(worktree.as_path()));
        println!("    old tree still beside it: {}", converting_path(worktree.as_path()).exists());
        println!("    journal still there: {}", admin.join(JOURNAL_NAME).exists());
        println!("    socket still in the worktree: {}", worktree.join("agent.sock").exists());
    }
    let (abandoned, lines) = run_conversion(&platform, worktree.as_path(), ConvertOptions { abandon: true, ..RUN });
    println!("--- abandon verdict: {abandoned:?}: {lines:?}");
    drop(listener);
    remove(parent.as_path());
}

/// The removal's comparison against the installed tree, on two trees that differ only in a file's
/// permission bits, and on a Unix socket, and on the byte count as a positive control.
#[test]
fn the_removal_compare_is_blind_to_a_permission_change() {
    let parent: PathBuf = fixture("audit-compare");
    let platform: PlainPlatform = PlainPlatform::new();
    for arm in ["mode", "xattr", "size"] {
        let old: PathBuf = parent.join(format!("{arm}-old"));
        let installed: PathBuf = parent.join(format!("{arm}-installed"));
        std::fs::create_dir_all(old.join("sub")).expect("old");
        std::fs::create_dir_all(installed.join("sub")).expect("installed");
        for tree in [&old, &installed] {
            std::fs::write(tree.join("keep.txt"), b"same bytes\n").expect("keep");
            std::fs::write(tree.join("sub/x.txt"), b"x\n").expect("x");
        }
        // One modification time for both trees, so the arms differ in nothing but the arm's own field.
        for tree in [&old, &installed] {
            set_mtime(tree.join("keep.txt").as_path(), 1_700_000_000);
            set_mtime(tree.join("sub/x.txt").as_path(), 1_700_000_000);
            set_mtime(tree.join("sub").as_path(), 1_700_000_000);
            set_mtime(tree.as_path(), 1_700_000_000);
        }
        match arm {
            "mode" => std::fs::set_permissions(old.join("sub/x.txt"), std::fs::Permissions::from_mode(0o755)).expect("mode"),
            "xattr" => set_user_attribute(old.join("sub/x.txt").as_path(), b"only-in-the-old-tree"),
            _ => std::fs::write(old.join("sub/x.txt"), b"xy\n").expect("size"),
        }
        let difference: Option<String> = first_difference(old.as_path(), installed.as_path()).expect("compared");
        println!("--- arm {arm}: first_difference -> {difference:?}");
        let held: std::fs::File = open_directory_at(&std::fs::File::open(parent.as_path()).expect("parent"), OsStr::new(old.file_name().expect("name"))).expect("held");
        let identity = identity_of(&held).expect("identity");
        let outcome: Removal = remove_recorded_old_tree(&platform, &std::fs::File::open(parent.as_path()).expect("parent"), old.file_name().expect("name"), identity, installed.as_path()).expect("removal");
        println!("    removal outcome: {outcome:?}");
        println!("    the old tree is gone: {}", !old.exists());
    }
    remove(parent.as_path());
}

/// Set a path's own modification and access time to one second after the epoch, without following a link.
fn set_mtime(path: &Path, seconds: i64) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL");
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 0 }, libc::timespec { tv_sec: seconds, tv_nsec: 0 }];
    // SAFETY: `name` is NUL-terminated and outlives the call; `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "age {}", path.display());
}

/// Set one user extended attribute; true when the filesystem accepted it.
fn set_user_attribute(path: &Path, value: &[u8]) {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL");
    let attribute: std::ffi::CString = std::ffi::CString::new("user.audit-note").expect("no NUL");
    // SAFETY: both strings are NUL-terminated and outlive the call; `value` holds `value.len()` bytes.
    let result: libc::c_int = unsafe { libc::lsetxattr(name.as_ptr(), attribute.as_ptr(), value.as_ptr().cast(), value.len(), 0) };
    println!("    setxattr accepted: {}", result == 0);
}

/// A rollback after a removal that stopped part way: what the checkout holds afterwards.
#[test]
fn a_rollback_after_a_partly_deleted_old_tree() {
    let platform: PlainPlatform = PlainPlatform::new();
    let parent: PathBuf = fixture("audit-partial-rollback");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let old: PathBuf = parent.join("checkout.before-subvolume");
    let (code, _lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    println!("migrate exit {code}");
    println!("before: the checkout holds README.md: {}", root.join("README.md").exists());
    println!("before: the old tree holds README.md: {}", old.join("README.md").exists());
    // The state a kill inside `remove_tree` leaves: the walk had unlinked some entries.
    std::fs::remove_file(old.join("README.md")).expect("unlink 1");
    std::fs::remove_file(old.join("src/deep/er/leaf.txt")).expect("unlink 2");
    println!("after the stopped removal: the old tree holds README.md: {}", old.join("README.md").exists());
    let rollback: MigrateArguments = MigrateArguments { mode: MigrateMode::Rollback, ..migrate_arguments() };
    let (code, lines) = run_command(&platform, &rollback, root.as_path(), &Interrupt::never());
    println!("rollback exit {code}");
    for line in &lines {
        println!("    {line}");
    }
    println!("after: the checkout holds README.md: {}", root.join("README.md").exists());
    println!("after: the checkout holds src/deep/er/leaf.txt: {}", root.join("src/deep/er/leaf.txt").exists());
    println!("after: the new subvolume is gone: {}", !root.with_extension("").join("checkout.subvol-new").exists());
    println!("after: the checkout is a subvolume: {:?}", platform.is_subvolume_root(root.as_path()));
    let status: String = crate::test_support::git_text(root.as_path(), &["status", "--porcelain"]);
    println!("after: git status --porcelain: {status:?}");
    // The preview of the same two modes, with no journal at all.
    let preview: MigrateArguments = MigrateArguments { mode: MigrateMode::RemoveOld, dry_run: true, ..migrate_arguments() };
    let (code, lines) = run_command(&platform, &preview, root.as_path(), &Interrupt::never());
    println!("dry-run --remove-old with no journal: exit {code}: {lines:?}");
    remove(parent.as_path());
}

/// A named pipe where the migration's journal is expected, read by the two readers the route has.
#[test]
fn a_named_pipe_where_the_journal_belongs() {
    let parent: PathBuf = fixture("audit-journal-pipe");
    let pipe: PathBuf = parent.join(".cli-git-migrate-checkout.journal.json");
    let made: std::process::Output = std::process::Command::new("mkfifo").arg(pipe.as_os_str()).output().expect("mkfifo");
    println!("mkfifo exit: {:?}", made.status.code());
    let regular: PathBuf = parent.join("regular.json");
    std::fs::write(regular.as_path(), b"{\"schema\":1,\"started\":\"x\",\"steps\":{}}").expect("journal");
    for (label, path) in [("regular file", regular.clone()), ("named pipe", pipe.clone())] {
        let (sender, receiver) = std::sync::mpsc::channel();
        let read: PathBuf = path.clone();
        std::thread::spawn(move || {
            let outcome: String = match Journal::load(read.as_path()) {
                Ok(journal) => format!("Journal::load returned (persisted {})", journal.is_persisted()),
                Err(error) => format!("Journal::load failed: {error}"),
            };
            let _sent = sender.send(outcome);
        });
        match receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(outcome) => println!("{label}: {outcome}"),
            Err(_late) => println!("{label}: Journal::load did not return within 5 s"),
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        let read: PathBuf = path.clone();
        std::thread::spawn(move || {
            let outcome: String = match read_plain_file(read.as_path(), 4096) {
                Ok(bytes) => format!("read_plain_file returned {} bytes", bytes.len()),
                Err(error) => format!("read_plain_file refused it: {error}"),
            };
            let _sent = sender.send(outcome);
        });
        match receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(outcome) => println!("{label}: {outcome}"),
            Err(_late) => println!("{label}: read_plain_file did not return within 5 s"),
        }
    }
    remove(parent.as_path());
}

/// The same skipped entry, in the route that reports it: a socket inside the main checkout.
#[test]
fn a_socket_in_a_checkout_is_named_by_the_migration() {
    let platform: PlainPlatform = PlainPlatform::new();
    let parent: PathBuf = fixture("s");
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind(root.join("agent.sock")).expect("socket");
    let (code, lines) = run_command(&platform, &migrate_arguments(), root.as_path(), &Interrupt::never());
    println!("migrate exit {code}");
    for line in &lines {
        println!("    {line}");
    }
    let (code, lines) = run_command(&platform, &MigrateArguments { mode: MigrateMode::RemoveOld, ..migrate_arguments() }, root.as_path(), &Interrupt::never());
    println!("remove-old exit {code}");
    for line in &lines {
        println!("    {line}");
    }
    println!("the socket is gone with the old tree: {}", !root.join("agent.sock").exists());
    drop(listener);
    remove(parent.as_path());
}
