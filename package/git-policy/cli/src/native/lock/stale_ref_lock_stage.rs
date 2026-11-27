//! What:
//!  The disposable repository a control builds so a real Git takes a reference lock in it:
//!  a bare remote with `main` and a branch the remote then loses,
//!  a clone-like repository that fetched and packed both,
//!  and the reftable backend's shape of the same fixture.
//! Why:
//!  The shape of a lock left by a killed Git is a fact about Git,
//!  not about this crate,
//!  so a control produces it with Git in a repository of its own (rule THR).
//!  Every path a control asks about (a reference's lock,
//!  its pid file,
//!  the packed
//!  references,
//!  the reftable list) is named here,
//!  so a control spells no path itself and a change to Git's layout is one edit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stage = gitPolicyCliStage('control-name', true); // { root, repo, gitDir, ready }
//! ```

// The fixture starts real Git and reads its lock files, which only Unix children offer here.
#![cfg(unix)]

use crate::test_support::{
    fixture,
    git,
    remove,
    repository,
};
/// `PathBuf` is an owned filesystem path,
/// and not the borrowed `Path`.
use std::path::PathBuf;

/// The tracking reference the remote no longer has,
/// so `fetch --prune` deletes it and locks it empty.
pub const GONE_REF: &str = "refs/remotes/origin/gone";

/// A repository whose remote lost a branch,
/// with every tracking reference packed.
pub struct Stage {
    /// The fixture directory,
    /// removed at the end.
    pub root: PathBuf,
    /// The working repository.
    pub repo: PathBuf,
    /// Its Git directory.
    pub git_dir: PathBuf,
    /// The file the hook writes its own process number into once the fetch holds its locks.
    pub ready: PathBuf,
}

impl Stage {
    /// The lock file of a reference.
    pub fn lock_of(
        &self,
        reference: &str,
    ) -> PathBuf {
        return self.git_dir.join(format!("{reference}.lock"));
    }

    /// The pid file beside the lock of a reference.
    pub fn pid_file_of(
        &self,
        reference: &str,
    ) -> PathBuf {
        return self.git_dir.join(format!("{reference}~pid.lock"));
    }

    /// The lock of the packed references,
    /// and the staging file Git writes beside it.
    pub fn packed_lock(&self) -> PathBuf {
        return self.git_dir.join("packed-refs.lock");
    }

    /// The reftable backend's lock of its list of tables,
    /// which every writer of a reference takes (`reftable/stack.c`,
    /// `reftable_stack_init_addition`).
    pub fn tables_lock(&self) -> PathBuf {
        return self.git_dir.join("reftable").join("tables.list.lock");
    }

    /// The pid file Git writes beside that lock under `core.lockfilePid`.
    pub fn tables_pid_file(&self) -> PathBuf {
        return self.git_dir.join("reftable").join("tables.list~pid.lock");
    }

    /// The staging file of the packed references.
    pub fn packed_staging(&self) -> PathBuf {
        return self.git_dir.join("packed-refs.new");
    }

    /// The Git directory as the roots a clearing pass searches.
    pub fn roots(&self) -> Vec<PathBuf> {
        return vec![self.git_dir.clone()];
    }

    /// Remove the fixture.
    pub fn finish(self) {
        remove(self.root.as_path());
    }
}

/// Build the stage:
/// a bare remote with `main` and `gone`,
/// a clone-like repository that fetched both and packed them,
/// and
///  then a remote that no longer has `gone`.
/// `lockfile_pid` writes `core.lockfilePid` into the repository's own
///  configuration,
/// as the wrapper turns it on for the Git it starts.
pub fn stage(
    name: &str,
    lockfile_pid: bool,
) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(
        root.as_path(),
        &[
            "init",
            "--quiet",
            "--bare",
            "--initial-branch=main",
            remote.to_str().expect("UTF-8"),
        ],
    );
    git(
        repo.as_path(),
        &["remote", "add", "origin", remote.to_str().expect("UTF-8")],
    );
    git(repo.as_path(), &["branch", "gone"]);
    git(
        repo.as_path(),
        &["push", "--quiet", "origin", "main", "gone"],
    );
    git(repo.as_path(), &["fetch", "--quiet", "origin"]);
    git(repo.as_path(), &["pack-refs", "--all"]);
    git(remote.as_path(), &["branch", "--delete", "--force", "gone"]);
    if lockfile_pid {
        git(repo.as_path(), &["config", "core.lockfilePid", "true"]);
    }
    let git_dir: PathBuf = repo.join(".git");
    let ready: PathBuf = root.join("ready");
    return Stage {
        root,
        repo,
        git_dir,
        ready,
    };
}

/// Build a stage whose references the reftable backend keeps:
/// a repository with one commit and no remote,
/// which is all a held `update-ref --stdin` needs.
/// The reftable backend keeps its references and its lock in the `reftable` directory of the Git directory,
/// so none of the files a files-backend repository locks exists.
pub fn reftable_stage(
    name: &str,
    lockfile_pid: bool,
) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(
        repo.as_path(),
        &[
            "init",
            "--quiet",
            "--ref-format=reftable",
            "--initial-branch=main",
        ],
    );
    git(
        repo.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=initial"],
    );
    if lockfile_pid {
        git(repo.as_path(), &["config", "core.lockfilePid", "true"]);
    }
    let git_dir: PathBuf = repo.join(".git");
    let ready: PathBuf = root.join("ready");
    return Stage {
        root,
        repo,
        git_dir,
        ready,
    };
}
