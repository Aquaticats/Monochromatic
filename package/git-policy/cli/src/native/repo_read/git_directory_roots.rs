//! What:
//!  The repository a command works on,
//! resolved the way Git resolves it and without starting Git:
//! the directory its `-C` chain ends in,
//! the Git directory its options or environment name,
//! and the common Git directory the locks and the record folder live in.
//! Why:
//!  The lock pass, the record of the processes a command starts, the reference-lock clearing before a forwarded
//!  `fetch` or `push` and the sweep of a stopped child's locks all ask one question:
//!  which repository does this command work on.
//!  Two answers to it disagree, and a command then clears a lock in one repository and records its Git in another,
//!  or looks in a directory Git never uses (a review of `2af2ecb879`, finding 1).
//!  Git is not asked,
//! because the answer is needed when Git is about to be started or has just failed,
//! and reading a few small files costs nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const repository = repositoryACommandWorksOn({ cwd, globalPrefix, environment }); const roots = repository.roots();
//! ```

/// Owned OS strings of a command's environment.
use std::ffi::{OsStr, OsString};
/// Paths of the repository.
use std::path::{Path, PathBuf};
/// The command whose repository is wanted.
use std::process::Command;
/// The proof that a folder is the Git directory of the command, and that no component of its path is a link.
use crate::git_directory_proof::{ProvenGitDirectory, prove_git_directory};
/// The one walk of the global options that answers the directory a `-C` chain ends in and the Git directory a
///  `--git-dir` names, so neither is resolved twice.
use crate::repository_link_paths::{GIT_COMMON_DIRECTORY_VARIABLE, GIT_DIRECTORY_VARIABLE, chain_directory, last_git_dir_option, last_option_value};
/// The variables that move a file Git locks out of the Git directory.
///  `GIT_INDEX_FILE`, `GIT_SHALLOW_FILE` and `GIT_OBJECT_DIRECTORY` (`environment.h`,
/// lines 14, 17 and 19).
use crate::repository_link_paths::{GIT_INDEX_FILE_VARIABLE, GIT_OBJECT_DIRECTORY_VARIABLE, GIT_SHALLOW_FILE_VARIABLE};

/// The common Git directory a linked worktree's Git directory names in its `commondir` file,
/// or the directory itself.
///  The few lines are repeated here because the module that reads it belongs to the service run,
/// which the client path
///  may not name (`standards_review_boundary_tests.rs`);
/// a path that cannot be resolved is used as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync(join(gitDir, 'commondir')) ? realpath(resolve(gitDir, readFile(...).trimEnd())) : gitDir
/// ```
pub fn common_dir_for(git_dir: &Path) -> PathBuf {
    let Ok(text) = std::fs::read_to_string(git_dir.join("commondir")) else {
        return git_dir.to_path_buf();
    };
    let named: &str = text.trim_end_matches(['\r', '\n']);
    if named.is_empty() {
        return git_dir.to_path_buf();
    }
    let joined: PathBuf = git_dir.join(named);
    return std::fs::canonicalize(&joined).unwrap_or(joined);
}

/// Where the Git directory of a working directory is:
/// `.git` as a directory,
/// a `gitdir:` file,
/// or a bare repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitDirectoryOf(start: string): string | undefined;
/// ```
pub fn git_directory_of(start: &Path) -> Option<PathBuf> {
    for directory in start.ancestors() {
        let dot_git: PathBuf = directory.join(".git");
        if let Ok(metadata) = std::fs::symlink_metadata(dot_git.as_path()) {
            if metadata.is_dir() {
                return Some(dot_git);
            }
            if metadata.is_file() {
                let text: String = std::fs::read_to_string(dot_git.as_path()).ok()?;
                let named: &str = text.lines().next()?.strip_prefix("gitdir:")?.trim();
                return Some(directory.join(named));
            }
        }
        if directory.join("HEAD").is_file() && directory.join("objects").is_dir() {
            return Some(directory.to_path_buf());
        }
    }
    return None;
}

/// The Git directory and the common Git directory (when it differs) of a working directory,
/// or of `GIT_DIR` when one is
///  named;
/// nothing when no repository is found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsFor(working: string, named?: string): string[];
/// ```
pub fn roots_for(working: &Path, named: Option<PathBuf>) -> Vec<PathBuf> {
    let git_dir: Option<PathBuf> = match named {
        Some(explicit) => Some(explicit),
        None => git_directory_of(working),
    };
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(directory) = git_dir {
        let common: PathBuf = common_dir_for(directory.as_path());
        roots.push(directory.clone());
        if common != directory {
            roots.push(common);
        }
    }
    return roots;
}

/// What:
///  The repository a command works on:
///  the directory it runs in after its `-C` chain,
///  the Git directory it works in,
///  and the common Git directory its locks and its record folder live in.
/// Why:
///  One value answers every reader that needs to know which repository an invocation works on,
///  so no two of them can disagree about a relative `-C`,
///  a named Git directory or a named common directory.
///  `git_dir` and `common_dir` are `None` when the command works outside a repository,
///  which a pass then costs one failed look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommandRepository = { start: string; gitDir?: string; commonDir?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandRepository {
    /// The directory the command runs in,
    ///  after every `-C` it was given.
    pub start: PathBuf,
    /// The Git directory it works in,
    ///  when one was named or found.
    pub git_dir: Option<PathBuf>,
    /// The common Git directory,
    ///  where a linked worktree's shared references and the record folder live.
    pub common_dir: Option<PathBuf>,
    /// The index file the command reads and writes,
    ///  `GIT_INDEX_FILE` or `<git-dir>/index`.
    pub index_file: Option<PathBuf>,
    /// The shallow file it reads and writes,
    ///  `GIT_SHALLOW_FILE`, `--shallow-file` or `<git-dir>/shallow`.
    pub shallow_file: Option<PathBuf>,
    /// The object directory its pack and graph files are locked in,
    ///  `GIT_OBJECT_DIRECTORY` or `<common-dir>/objects`.
    pub object_dir: Option<PathBuf>,
}

/// What:
///  One place a lock of this command can stand,
///  and how the pass finds it.
/// Why:
///  A pass may open and unlink only what Git itself can have locked,
///  and a file of a person's named `something.lock` in a folder Git happens to share is not one of them:
///  taking every name with a suffix in the directory of a selected index deleted a person's data
///  (a review of `429503238b`,
/// finding 3).
///  So a lock is named exactly where Git names it,
///  and a whole folder is read only where every `*.lock` inside it can only be Git's:
///  the references, whose names Git forbids to end in `.lock`,
///  and the sequencer's own folders, which hold Git's state and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockCandidate = { kind: 'exactly'; path: string } | { kind: 'directlyBelow'; folder: string } | { kind: 'anywhereBelow'; folder: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LockCandidate {
    /// The one name Git locks beside a file the person's settings named,
    ///  such as an index Git was told to use.
    ///  Its own last name is checked as a file, never followed.
    BesideASettingsNamedFile(PathBuf),
    /// The one name Git locks below components of a proven Git directory,
    ///  the last of which names a folder that must be a folder.
    ExactNameBelowProven {
        /// Which of the proof's two folders the components start at.
        base: ProvenFolder,
        /// The components from there,
        ///  each opened as the folder it is named as.
        components: Vec<std::ffi::OsString>,
        /// The name Git locks in that last folder.
        name: std::ffi::OsString,
    },
    /// Every `*.lock` below components of a proven Git directory.
    EveryLockBelowProven {
        /// Which of the proof's two folders the components start at.
        base: ProvenFolder,
        /// The components from there,
        ///  each opened as the folder it is named as.
        components: Vec<std::ffi::OsString>,
        /// Whether the walk goes below the last component.
        recursive: bool,
    },
}

/// What:
///  Which folder of a proven Git directory a candidate's components start at.
/// Why:
///   The worktree's own state lives in the Git directory and the shared references in the common one,
///  and both are the proof's,
///  so a candidate names its base rather than a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProvenFolder = 'git-directory' | 'common-directory';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenFolder {
    /// The Git directory of the command.
    GitDirectory,
    /// The common Git directory its references and records live in.
    CommonDirectory,
}

/// What:
///  The lock file beside the file Git locks,
///  which is the name Git gives it (`lockfile.c`,
/// lines 99 to 110).
/// Why:
///  Every candidate of this answer is written the way Git writes it,
///  so no candidate can name a file Git would not have made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lockedName('/repo/.git/index') // '/repo/.git/index.lock'
/// ```
fn locked_name(file: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = file.as_os_str().to_os_string();
    name.push(".lock");
    return PathBuf::from(name);
}

impl CommandRepository {
    /// What:
    ///  The Git directories a lock pass searches:
    ///  the Git directory,
    ///  and the common one when it differs.
    /// Why:
    ///  A lock of the worktree's state sits in the first,
    ///  and the shared references and a linked worktree's locks in the second,
    ///  so a pass that read only one would miss what the other holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// repository.roots() // ['/repo/.git'] or ['/repo/.git', '/repo/.git']
    /// ```
    pub fn roots(&self) -> Vec<PathBuf> {
        let Some(git_dir) = self.git_dir.as_ref() else {
            return Vec::new();
        };
        let mut roots: Vec<PathBuf> = vec![git_dir.clone()];
        if let Some(common) = self.common_dir.as_ref()
            && common != git_dir
        {
            roots.push(common.clone());
        }
        return roots;
    }

    /// What:
    ///  The proof that this command's Git directory is a repository,
    ///  taken from the one answer this invocation resolved,
    ///  or nothing when it is not one.
    /// Why:
    ///  Every pass, every record write and every root starts from this proof,
    ///  so a command whose `--git-dir` names a folder Git would refuse has no pass at all
    ///  (a review of `551084db3c`,
    /// finding 3).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// repository.provenGitDirectory() // { gitDir, commonDir } | undefined
    /// ```
    pub fn proven_git_directory(&self) -> Option<ProvenGitDirectory> {
        let git_dir: &Path = self.git_dir.as_deref()?;
        let common_dir: &Path = self.common_dir.as_deref().unwrap_or(git_dir);
        return prove_git_directory(git_dir, common_dir);
    }

    /// What:
    ///  Every lock this command's repository can hold,
    ///  each named where Git names it,
    ///  and each reached through a folder of the proof and never through a link.
    /// Why:
    ///  Git locks a file beside it,
    ///  and which files it locks is fixed by its own layout while where they lie follows the settings this answer
    ///  read.
    ///  The names below are the ones Git's source writes (see the table in `abandoned_locks.rs`),
    ///  and each is named below a folder of the proof:
    ///  the references,
    ///  the sequencer's three folders,
    ///  and the linked worktrees,
    ///  every one of them reached by opening each component as the folder it is named as
    ///  (`abandoned_locks.rs`,
    /// `folder_below_proven`).
    ///  So a link at a root itself, or at any component below it, is no root.
    ///  The index and the shallow file are the two names the person's settings place anywhere,
    ///  and beside them only the one name Git locks there is judged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// repository.lockCandidatesIn(proven) // [{kind: 'exactNameBelowProven', base: 'git-directory', components: [], name: 'HEAD.lock'}, {kind: 'everyLockBelowProven', base: 'common-directory', components: ['refs'], recursive: true}]
    /// ```
    pub fn lock_candidates_in(&self, proven: &ProvenGitDirectory) -> Vec<LockCandidate> {
        let common: &Path = proven.common();
        let mut candidates: Vec<LockCandidate> = Vec::new();
        let mut add = |candidate: LockCandidate| {
            if candidates.contains(&candidate) {
                return;
            }
            candidates.push(candidate);
        };
        // A name Git locks in a folder of the proof, added the one way every candidate is added.
        macro_rules! named {
            ($base:expr, [$($component:expr),*], $name:expr) => {
                add(LockCandidate::ExactNameBelowProven {
                    base: $base,
                    components: vec![$(std::ffi::OsString::from($component)),*],
                    name: std::ffi::OsString::from($name),
                })
            };
        }
        // The index and the shallow file are the two Git locks whose place the person's settings move.
        if let Some(index) = self.index_file.as_ref() {
            add(LockCandidate::BesideASettingsNamedFile(locked_name(index.as_path())));
        }
        if let Some(shallow) = self.shallow_file.as_ref() {
            add(LockCandidate::BesideASettingsNamedFile(locked_name(shallow.as_path())));
        }
        for name in ["HEAD.lock", "ORIG_HEAD.lock", "MERGE_RR.lock", "gc.pid.lock", "gc.log.lock"] {
            named!(ProvenFolder::GitDirectory, [], name);
        }
        named!(ProvenFolder::GitDirectory, ["info"], "sparse-checkout.lock");
        for name in ["packed-refs.lock", "config.lock"] {
            named!(ProvenFolder::CommonDirectory, [], name);
        }
        named!(ProvenFolder::CommonDirectory, ["reftable"], "tables.list.lock");
        // Every reference Git locks lies under `refs`, and a reference name cannot end in `.lock`, so a `.lock` file
        //  there is a lock and only a lock.
        add(LockCandidate::EveryLockBelowProven { base: ProvenFolder::CommonDirectory, components: vec![std::ffi::OsString::from("refs")], recursive: true });
        for folder in ["sequencer", "rebase-merge", "rebase-apply"] {
            add(LockCandidate::EveryLockBelowProven { base: ProvenFolder::GitDirectory, components: vec![std::ffi::OsString::from(folder)], recursive: false });
        }
        // A linked worktree's Git directory holds the same names as this one, and the folder of each is reached below
        //  `worktrees`, which is opened as the folder it is named as.
        //  A link at `worktrees`, or at an entry below it, is no folder of this repository
        //  (a review of `551084db3c`,
        // finding 2).
        add(LockCandidate::EveryLockBelowProven { base: ProvenFolder::CommonDirectory, components: vec![std::ffi::OsString::from("worktrees")], recursive: true });
        // The object store's own locks, each by the name Git writes, reached below `objects` when the settings leave it
        //  where Git puts it.
        let default_objects: PathBuf = common.join("objects");
        let objects_are_git_s: bool = self.object_dir.as_ref().is_none_or(|object_dir| { return object_dir == &default_objects; });
        if objects_are_git_s {
            named!(ProvenFolder::CommonDirectory, ["objects"], "maintenance.lock");
            named!(ProvenFolder::CommonDirectory, ["objects"], "schedule.lock");
            named!(ProvenFolder::CommonDirectory, ["objects"], "loose-object-idx.lock");
            named!(ProvenFolder::CommonDirectory, ["objects", "info"], "alternates.lock");
            named!(ProvenFolder::CommonDirectory, ["objects", "info"], "commit-graph.lock");
            named!(ProvenFolder::CommonDirectory, ["objects", "pack"], "multi-pack-index.lock");
            named!(ProvenFolder::CommonDirectory, ["objects", "pack"], "multi-pack-index-chain.lock");
        } else if let Some(object_dir) = self.object_dir.as_ref() {
            for name in ["maintenance.lock", "schedule.lock", "loose-object-idx.lock", "info/alternates.lock", "info/commit-graph.lock", "pack/multi-pack-index.lock", "pack/multi-pack-index-chain.lock"] {
                add(LockCandidate::BesideASettingsNamedFile(object_dir.join(name)));
            }
        }
        return candidates;
    }
}

/// What:
///  The repository a command works on,
///  from the directory it runs in,
///  the global options it was given,
///  and its environment.
///  `&[OsString]` borrows the tokens before the subcommand.
/// Why:
///  This is the one answer to "which repository does this invocation work on",
///  resolved the way Git resolves it:
///
///  - the `-C` chain moves the directory,
///    each link from the one before,
///    and only it moves that directory (`--work-tree` does not);
///  - the last `--git-dir` wins over `GIT_DIR`,
///    and a relative one is read from the directory the chain ends in;
///  - `GIT_COMMON_DIR` names the common directory,
///    read from that same directory when it is relative,
///    and it wins over the `commondir` file a linked worktree carries;
///  - without a named Git directory the repository is found by walking up from that directory,
///    which reads a few small files and starts no Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryACommandWorksOn({ cwd, globalPrefix: ['-C', 'sub', '--git-dir=../cfg.git'], environment })
/// // { start: '/w/sub', gitDir: '/w/sub/../cfg.git', commonDir: '/w/sub/../cfg.git' }
/// ```
pub fn repository_a_command_works_on(cwd: &Path, global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> CommandRepository {
    let start: PathBuf = chain_directory(cwd, global_prefix);
    let named: Option<PathBuf> = last_git_dir_option(global_prefix)
        .or_else(|| { return environment_value(environment, GIT_DIRECTORY_VARIABLE).map(PathBuf::from); });
    let git_dir: Option<PathBuf> = match named {
        Some(value) => Some(start.join(value)),
        None => git_directory_of(start.as_path()),
    };
    let Some(git_dir_found) = git_dir.as_ref() else {
        return CommandRepository { start, ..CommandRepository::default() };
    };
    let common_dir: PathBuf = match environment_value(environment, GIT_COMMON_DIRECTORY_VARIABLE) {
        Some(value) => start.join(value),
        None => common_dir_for(git_dir_found.as_path()),
    };
    // What: The three files Git may lock outside the Git directory are named from the same settings Git reads.
    // Why:  A pass that searched only the Git and common directories never saw an alternate index's lock, and then called
    //       the dead writer's record unused (a review of `4e9b9fff1d`, finding 2).
    let index_file: PathBuf = match environment_value(environment, GIT_INDEX_FILE_VARIABLE) {
        Some(value) => start.join(value),
        None => git_dir_found.join("index"),
    };
    let shallow_file: PathBuf = match last_option_value(global_prefix, "--shallow-file")
        .or_else(|| { return environment_value(environment, GIT_SHALLOW_FILE_VARIABLE).map(PathBuf::from); }) {
        Some(value) => start.join(value),
        None => git_dir_found.join("shallow"),
    };
    let object_dir: PathBuf = match environment_value(environment, GIT_OBJECT_DIRECTORY_VARIABLE) {
        Some(value) => start.join(value),
        None => common_dir.join("objects"),
    };
    return CommandRepository {
        start,
        git_dir: Some(git_dir_found.clone()),
        common_dir: Some(common_dir),
        index_file: Some(index_file),
        shallow_file: Some(shallow_file),
        object_dir: Some(object_dir),
    };
}

/// What:
///  The last value an environment list carries for `name`,
///  or nothing when it carries none or an empty one.
/// Why:
///   A command's environment is its own list,
///  and Git reads the last assignment of a variable it is started with;
///  an empty value names nothing, as an empty `GIT_DIR` names no Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// environmentValue([['GIT_DIR', '/g']], 'GIT_DIR') // '/g'
/// ```
fn environment_value<'a>(environment: &'a [(OsString, OsString)], name: &str) -> Option<&'a OsStr> {
    let mut found: Option<&'a OsStr> = None;
    for (variable, value) in environment {
        if variable == OsStr::new(name) && !value.is_empty() {
            found = Some(value.as_os_str());
        }
    }
    return found;
}

/// What:
///  The roots of a command that has not started yet:
///  the Git directories of the repository it will work in.
/// Why:
///   Such a command names its repository the way any invocation does,
///  through its working directory and its environment,
///  so it is resolved by the one resolver every other reader uses:
///  `GIT_DIR` names its Git directory and `GIT_COMMON_DIR` its common one,
///  and a variable the command removes names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsOfCommand(command: Command): string[];
/// ```
pub fn roots_of_command(command: &Command) -> Vec<PathBuf> {
    let working: PathBuf = match command.get_current_dir() {
        Some(directory) => directory.to_path_buf(),
        None => std::env::current_dir().unwrap_or_default(),
    };
    // Only what the command sets: a removal carries no value and names nothing.
    let mut environment: Vec<(OsString, OsString)> = Vec::new();
    for (name, set_to) in command.get_envs() {
        // A removal of a variable carries no value and names nothing.
        let Some(value) = set_to else {
            continue;
        };
        environment.push((name.to_os_string(), value.to_os_string()));
    }
    return repository_a_command_works_on(working.as_path(), &[], environment.as_slice()).roots();
}
