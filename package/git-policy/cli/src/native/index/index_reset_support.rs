//! Fixtures that compare the in-process path reset with `git reset` on copies of one repository.
//!
//! A scene is a repository whose branch has moved on by one commit made through a second index file,
//! so the main index
//! still holds the old commit's entries with real stat data,
//! as it does when the commit service lands a commit.

use crate::test_support::{REAL_GIT, age, fixture, git, git_text, remove, spawn_owned};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// One change the second commit makes to the first commit's tree.
pub(crate) enum Edit<'a> {
    /// A file (or symbolic link,
    /// by mode) with this content.
    Put { path: &'a str, mode: &'a str, content: &'a [u8] },
    /// A submodule entry naming a commit that is not in this repository.
    Gitlink { path: &'a str, id: &'a str },
    /// The path leaves the tree.
    Delete(&'a str),
}

/// A repository,
/// the commit its index is at,
/// and the commit the branch is at.
pub(crate) struct Scene {
    /// The fixture directory,
    /// removed by the caller.
    pub root: PathBuf,
    /// The repository.
    pub repo: PathBuf,
    /// The commit the index holds.
    pub old: String,
    /// The commit the branch is at.
    pub new: String,
}

/// Run real Git in `dir` with the fixture identity,
/// extra variables and optional standard input.
pub(crate) fn git_with(dir: &Path, env: &[(&str, OsString)], args: &[&str], input: Option<&[u8]>) -> Output {
    let mut command: Command = Command::new(REAL_GIT);
    command
        .current_dir(dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(args)
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in env {
        command.env(name, value);
    }
    let mut child = spawn_owned(&mut command).expect("start Git");
    if let Some(bytes) = input {
        use std::io::Write as _;
        child.stdin.take().expect("piped").write_all(bytes).expect("write input");
    }
    return child.wait_with_output().expect("Git output");
}

/// Real Git in a copy of `dir`'s environment with `index` as its index file,
/// requiring success;
/// its standard output.
fn git_in_index(dir: &Path, index: &Path, args: &[&str], input: Option<&[u8]>) -> String {
    let output: Output = git_with(dir, &[("GIT_INDEX_FILE", index.as_os_str().to_os_string())], args, input);
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8(output.stdout).expect("UTF-8").trim_end().to_string();
}

/// What a scene's first commit holds and its second commit changes.
#[derive(Default)]
pub(crate) struct Plan<'a> {
    /// Regular files of the first commit:
    /// path,
    /// content,
    /// executable.
    pub files: &'a [(&'a str, &'a [u8], bool)],
    /// Symbolic links of the first commit:
    /// path,
    /// target.
    pub links: &'a [(&'a str, &'a str)],
    /// Submodule entries of the first commit:
    /// path,
    /// commit name.
    pub gitlinks: &'a [(&'a str, &'a str)],
    /// What the second commit changes.
    pub edits: &'a [Edit<'a>],
    /// Whether the first commit's files keep the time they were written at (otherwise they are aged).
    pub fresh: bool,
    /// Configuration the repository carries before its first commit,
    /// such as `feature.manyFiles`,
    /// which Git honours on every index file it writes here.
    pub config: &'a [(&'a str, &'a str)],
    /// Whether the configuration asks for an untracked cache,
    /// so a status after the first commit writes one into the index the landing copies.
    pub untracked_cache: bool,
    /// Whether the repository uses SHA-256 object names.
    pub sha256: bool,
}

/// A scene made from `plan`.
pub(crate) fn scene(name: &str, plan: &Plan<'_>) -> Scene {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    let format: &str = if plan.sha256 { "--object-format=sha256" } else { "--object-format=sha1" };
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", format]);
    for (key, value) in plan.config {
        git(repo.as_path(), &["config", key, value]);
    }
    for (path, content, executable) in plan.files {
        let file: PathBuf = repo.join(path);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
        std::fs::write(&file, content).expect("write");
        if *executable {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        }
        // The index records these files' times; aged, none is as new as the index file, so no entry is racily clean.
        if !plan.fresh {
            age(&file);
        }
    }
    for (path, target) in plan.links {
        std::os::unix::fs::symlink(target, repo.join(path)).expect("symlink");
        if !plan.fresh {
            age(repo.join(path).as_path());
        }
    }
    git(repo.as_path(), &["add", "--all"]);
    for (path, id) in plan.gitlinks {
        git(repo.as_path(), &["update-index", "--add", "--cacheinfo", format!("160000,{id},{path}").as_str()]);
    }
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=first"]);
    let old: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    // The untracked cache is written by a status that walks the worktree, as the person's own status writes it; every later
    // write of this index keeps it.
    if plan.untracked_cache {
        git(repo.as_path(), &["status", "--porcelain", "--untracked-files=normal"]);
    }
    let temporary: PathBuf = repo.join(".git").join("scene-index");
    git_in_index(repo.as_path(), temporary.as_path(), &["read-tree", "HEAD"], None);
    for edit in plan.edits {
        match edit {
            Edit::Put { path, mode, content } => {
                let blob: String = git_in_index(repo.as_path(), temporary.as_path(), &["hash-object", "-w", "--stdin"], Some(content));
                git_in_index(repo.as_path(), temporary.as_path(), &["update-index", "--add", "--cacheinfo", format!("{mode},{blob},{path}").as_str()], None);
            }
            Edit::Gitlink { path, id } => {
                git_in_index(repo.as_path(), temporary.as_path(), &["update-index", "--add", "--cacheinfo", format!("160000,{id},{path}").as_str()], None);
            }
            Edit::Delete(path) => {
                git_in_index(repo.as_path(), temporary.as_path(), &["update-index", "--force-remove", path], None);
            }
        }
    }
    let tree: String = git_in_index(repo.as_path(), temporary.as_path(), &["write-tree"], None);
    let new: String = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-p", old.as_str(), "-m", "second"]);
    git(repo.as_path(), &["update-ref", "HEAD", new.as_str()]);
    std::fs::remove_file(&temporary).expect("scene index removed");
    return Scene { root, repo, old, new };
}

/// A copy of the scene's repository beside it,
/// with times and modes kept.
pub(crate) fn copy_of(scene: &Scene) -> PathBuf {
    let copy: PathBuf = scene.root.join("copy");
    let output: Output = Command::new("cp").args(["--archive", "--no-target-directory"]).arg(&scene.repo).arg(&copy).output().expect("cp");
    assert!(output.status.success(), "cp failed: {}", String::from_utf8_lossy(&output.stderr));
    return copy;
}

/// Run `git reset --no-refresh <commit> -- <names>` in `repo` as the follow-up's fallback does.
pub(crate) fn git_reset(repo: &Path, commit: &str, names: &[Vec<u8>]) {
    let mut input: Vec<u8> = Vec::new();
    for name in names {
        input.extend_from_slice(b":(top,literal)");
        input.extend_from_slice(name.as_slice());
        input.push(0);
    }
    let output: Output = git_with(repo, &[], &["reset", "--no-refresh", commit, "--pathspec-from-file=-", "--pathspec-file-nul"], Some(input.as_slice()));
    assert!(output.status.success(), "git reset: {}", String::from_utf8_lossy(&output.stderr));
}

/// Whether `git reset --no-refresh <commit> -- <names>` succeeds in `repo`.
pub(crate) fn git_reset_status(repo: &Path, commit: &str, names: &[Vec<u8>]) -> bool {
    let mut input: Vec<u8> = Vec::new();
    for name in names {
        input.extend_from_slice(b":(top,literal)");
        input.extend_from_slice(name.as_slice());
        input.push(0);
    }
    return git_with(repo, &[], &["reset", "--no-refresh", commit, "--pathspec-from-file=-", "--pathspec-file-nul"], Some(input.as_slice())).status.success();
}

/// The bytes of the repository's index file.
pub(crate) fn index_bytes(repo: &Path) -> Vec<u8> {
    return std::fs::read(repo.join(".git").join("index")).expect("index");
}

/// What:
///  The stat data an index entry records for the file at `path`,
/// each field cut to the 32 bits an entry stores.
/// Why:
///   A control that says the landing recorded the worktree's stat data compares the entry with this,
/// and a comparison
///  of the full 64-bit fields would never be equal.
pub(crate) fn stat_of_file(path: &Path) -> crate::index_stat::StatData {
    use std::os::unix::fs::MetadataExt as _;
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).expect("the file");
    return crate::index_stat::StatData {
        ctime_sec: metadata.ctime() as u32,
        ctime_nsec: metadata.ctime_nsec() as u32,
        mtime_sec: metadata.mtime() as u32,
        mtime_nsec: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
    };
}

/// What:
///  The entry the index file of `repo` holds for `path`.
/// Why:
///   A control that asserts what the landing recorded for one path reads the one entry,
/// with everything the entry carries.
pub(crate) fn entry_of(repo: &Path, path: &str) -> crate::index_full::FullEntry {
    let bytes: Vec<u8> = index_bytes(repo);
    let index: crate::index_full::FullIndex = crate::index_full::parse_full_index(bytes.as_slice(), 20).expect("parsed index");
    return index.entries.into_iter().find(|entry| { return entry.path == path.as_bytes(); }).expect("the entry the path names");
}

/// What `git ls-files --stage --debug` says,
/// for a failure message.
pub(crate) fn listing(repo: &Path) -> String {
    return String::from_utf8_lossy(&git_with(repo, &[], &["ls-files", "--stage", "--debug"], None).stdout).into_owned();
}

/// The paths as the byte lists the follow-up passes.
pub(crate) fn named(paths: &[&str]) -> Vec<Vec<u8>> {
    return paths.iter().map(|path| { return path.as_bytes().to_vec(); }).collect();
}

/// End a scene.
pub(crate) fn finish(scene: Scene) {
    remove(scene.root.as_path());
}

/// The batch numbers of the landings the controls make.
static NEXT_BATCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// What:
///  The in-process update as a landing runs it:
/// take the live index lock,
/// copy the live index to the private name,
/// reset the
///  private copy,
/// and install it by the lock's release;
/// when the update declines,
/// drop the copy and the lock and leave the
///  live index as it was.
/// Why:
///   The controls compare index files after the update,
/// so they run it under the same lock protocol the service does.
pub(crate) fn landing_reset(context: &crate::transaction_git::GitContext, repo: &Path, new: &str, paths: crate::service_land_follow::IndexPaths<'_>) -> bool {
    return landing_reset_copy(context, repo, new, paths, false);
}

/// `landing_reset`,
/// with the private copy given an old modification time when `old_copy` is set.
/// The copy's time is the one
/// the update compares the entries' times with,
/// so a control for racily clean entries needs the copy older than they are.
pub(crate) fn landing_reset_copy(context: &crate::transaction_git::GitContext, repo: &Path, new: &str, paths: crate::service_land_follow::IndexPaths<'_>, old_copy: bool) -> bool {
    let live: PathBuf = repo.join(".git").join("index");
    let seq: u64 = NEXT_BATCH.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let Some(held) = crate::service_index_lock::try_acquire(live.as_path(), seq) else {
        return false;
    };
    let copy = crate::service_land_follow::PrivateCopy::make(&held, seq).expect("the private copy");
    let built: PathBuf = copy.path().to_path_buf();
    if old_copy && built.exists() {
        age(built.as_path());
    }
    let done: bool = crate::index_reset_file::reset_index_file(context, repo, new, paths, &held, &copy).is_some();
    if done {
        held.release_with(built.as_path()).expect("installed");
    } else {
        let _removed = std::fs::remove_file(&built);
        drop(held);
    }
    return done;
}

/// A commit on top of the scene's `new` whose whole tree is the one entry `name` of `mode` naming a blob with `content`,
/// made
/// without Git's name checks (`hash-object --literally`),
/// so a name Git refuses can be in a tree.
pub(crate) fn commit_with_raw_entry(scene: &Scene, name: &str, mode: &str, content: &[u8]) -> String {
    let blob: String = String::from_utf8(git_with(scene.repo.as_path(), &[], &["hash-object", "-w", "--stdin"], Some(content)).stdout).expect("UTF-8").trim().to_string();
    let mut tree: Vec<u8> = format!("{mode} {name}\0").into_bytes();
    for pair in blob.as_bytes().as_chunks::<2>().0 {
        tree.push(u8::from_str_radix(std::str::from_utf8(pair).expect("hex"), 16).expect("hex digit pair"));
    }
    let made: Output = git_with(scene.repo.as_path(), &[], &["hash-object", "-t", "tree", "-w", "--literally", "--stdin"], Some(tree.as_slice()));
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    let tree_name: String = String::from_utf8(made.stdout).expect("UTF-8").trim().to_string();
    return git_text(scene.repo.as_path(), &["commit-tree", tree_name.as_str(), "-p", scene.new.as_str(), "-m", "raw entry"]);
}
