//! What:
//!  The fixture checkouts and the run helpers the migration's controls share:
//!  a tree whose files and links were all modified a quarter of an hour ago,
//!  a real Git repository of the shapes a copy must carry,
//!  the arguments of a plain migration run,
//!  a command run to completion against a platform,
//!  a checkout with a linked worktree inside it,
//!  that checkout moved behind a link,
//!  and the inode `git ls-files --debug` reports for a path.
//! Why:
//!  A control needs a tree that is already old enough for Git to trust its stat fields,
//!  a real index to rewrite,
//!  and the same three checkouts every time,
//!  so each of these is written once here rather than in each control.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const root = makeCheckout(parent, 'checkout');        // a real Git repository, backdated
//! const (status, lines) = runCommand(platform, root, migrateArguments());
//! ```

use crate::subvolume_arguments::{
    MigrateArguments,
    MigrateMode,
};
use crate::subvolume_command::execute;
use crate::subvolume_error::Interrupt;
use crate::subvolume_platform::Platform;
use crate::test_support::{
    fixture,
    git,
    git_text,
    repository,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Set the modification time of every file and link of a tree,
/// except `.git`,
/// to a quarter of an hour ago.
/// Why:
///   Git trusts the stat fields of an entry only when the file was modified before the index was written.
/// Files made a
///  moment before `git add` are racy,
/// which the stat rewrite rightly leaves alone;
/// a real checkout's files are old.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await backdate(root);
/// ```
pub(crate) fn backdate(root: &Path) {
    let seconds: i64 = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_secs(),
    )
    .expect("seconds")
        - 900;
    let times: [libc::timespec; 2] = [
        libc::timespec {
            tv_sec: seconds,
            tv_nsec: 0,
        },
        libc::timespec {
            tv_sec: seconds,
            tv_nsec: 0,
        },
    ];
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for item in std::fs::read_dir(&directory).expect("listing").flatten() {
            if item.file_name() == ".git" {
                continue;
            }
            let path: PathBuf = item.path();
            if item.file_type().expect("type").is_dir() {
                pending.push(path.clone());
            }
            let name =
                std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path text");
            // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
            let result: libc::c_int = unsafe {
                libc::utimensat(
                    libc::AT_FDCWD,
                    name.as_ptr(),
                    times.as_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            assert_eq!(result, 0, "backdate {}", path.display());
        }
    }
}

/// What:
///  A real Git repository at `parent/name` with tracked files of the shapes the copy must carry:
/// nested directories,
///  an executable,
/// a symbolic link,
/// an empty file,
/// two names of one file,
/// a read-only directory,
/// and an ignored
///  directory;
/// committed with real Git so its index is real.
/// Why:
///   Real Git writes the index the stat rewrite must keep valid,
/// and checks the result afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = makeCheckout(parent, 'checkout');
/// ```
pub(crate) fn make_checkout(
    parent: &Path,
    name: &str,
) -> PathBuf {
    let root: PathBuf = repository(parent, name);
    std::fs::create_dir_all(root.join("src/deep/er")).expect("directories");
    std::fs::write(root.join("README.md"), b"readme\n").expect("readme");
    std::fs::write(root.join("src/main.rs"), b"fn main() {}\n").expect("main");
    std::fs::write(root.join("src/deep/er/leaf.txt"), vec![b'x'; 70_000]).expect("leaf");
    std::fs::write(root.join("empty"), b"").expect("empty");
    std::fs::write(
        root.join("script.sh"),
        b"an executable file that is not run\n",
    )
    .expect("script");
    std::fs::set_permissions(
        root.join("script.sh"),
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .expect("executable");
    std::os::unix::fs::symlink("src/main.rs", root.join("link")).expect("symlink");
    std::fs::write(root.join(".gitignore"), b"ignored/\n").expect("gitignore");
    // The ignored tree, with a second name for a tracked file, exists before `git add`: a new link changes the file's change
    // time, which Git compares.
    std::fs::create_dir_all(root.join("ignored/node_modules/pkg")).expect("ignored tree");
    std::fs::write(
        root.join("ignored/node_modules/pkg/index.js"),
        b"module.exports = 1;\n",
    )
    .expect("ignored file");
    std::fs::hard_link(root.join("src/main.rs"), root.join("ignored/main-link.rs"))
        .expect("hard link");
    backdate(root.as_path());
    git(root.as_path(), &["add", "--all"]);
    git(root.as_path(), &["commit", "--quiet", "--message=files"]);
    return root;
}

/// The arguments of a plain migration run.
pub(crate) fn migrate_arguments() -> MigrateArguments {
    return MigrateArguments {
        mode: MigrateMode::Migrate,
        dry_run: false,
        nested_worktrees: true,
        refresh_indexes: true,
        discard_unreadable: false,
        discard_left_out: false,
        help: false,
    };
}

/// What:
///  Run the command to completion against a platform and collect its exit status and every progress line.
/// Why:
///   The controls assert on the status and the words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { code, lines } = runCommand(platform, args, root, interrupt);
/// ```
pub(crate) fn run_command(
    platform: &dyn Platform,
    arguments: &MigrateArguments,
    root: &Path,
    interrupt: &Interrupt,
) -> (i32, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let code: i32 = execute(platform, arguments, root, interrupt, &mut |text: &str| {
        lines.push(text.to_string());
    });
    return (code, lines);
}

/// A checkout with one linked worktree inside it,
/// as the agents' layout has.
pub(crate) fn checkout_with_worktree(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = make_checkout(parent.as_path(), "checkout");
    git(
        root.as_path(),
        &["worktree", "add", "--quiet", "--detach", "wt"],
    );
    // The agents' layout keeps its worktree directory ignored; without this the main checkout lists `wt/` as untracked.
    let exclude: PathBuf = root.join(".git/info/exclude");
    let mut text: String = std::fs::read_to_string(exclude.as_path()).unwrap_or_default();
    text.push_str("/wt/\n");
    std::fs::create_dir_all(root.join(".git/info")).expect("info directory");
    std::fs::write(exclude.as_path(), text).expect("exclude file");
    return (parent, root);
}

/// What:
///  A checkout with one linked worktree `wt`,
/// after the whole repository moved behind a symbolic link:
/// the checkout is renamed `moved` and a link is left at its old name,
/// so every registration still spells the old name while naming the same files.
/// Why:
///   A registration that spells the same file through a link is not stale,
/// which is what this repository looked like on 2026-11-08,
/// when `/var/home/user/Monochromatic` became a link to `/var/mnt/encrypted/repository/Monochromatic`.
/// The worktree's tracked files are old and its index is fresh,
/// so it is quiet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [parent, root, worktree] = checkoutMovedBehindALink('convert-link');
/// ```
pub(crate) fn checkout_moved_behind_a_link(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    let moved: PathBuf = parent.join("moved");
    std::fs::rename(&root, &moved).expect("move the checkout aside");
    std::os::unix::fs::symlink(&moved, &root).expect("leave a link at the old name");
    let worktree: PathBuf = std::fs::canonicalize(moved.join("wt")).expect("canonical worktree");
    backdate(worktree.as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    return (parent, moved, worktree);
}

/// What `git ls-files --debug` reports as the inode of a tracked path.
pub(crate) fn index_inode(
    root: &Path,
    path: &str,
) -> u64 {
    let text: String = git_text(root, &["ls-files", "--debug", "--", path]);
    let line: &str = text
        .lines()
        .find(|candidate: &&str| {
            return candidate.contains("ino:");
        })
        .expect("a stat line");
    return line
        .rsplit("ino:")
        .next()
        .expect("an inode")
        .trim()
        .parse::<u64>()
        .expect("a number");
}
