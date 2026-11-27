//! What:
//!  A repository built from one plan:
//!  its first commit holds the files,
//!  links and submodule entries the plan names,
//!  a second index file makes the plan's edits into a second commit,
//!  and the branch is moved onto it so the main index still holds the first
//!  commit's entries with their real stat data.
//! Why:
//!  The landing's update is compared against `git reset` on copies of one
//!  repository whose branch has moved on by one commit made through a second
//!  index file,
//!  which is what a landed commit leaves behind,
//!  and the two commits it answers with are the pair every comparison is read
//!  against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scene = scene('index-reset-links', { files, links, gitlinks, edits, fresh: false, config, untrackedCache: false, sha256: false });
//! const copy = copyOf(scene);
//! finish(scene);
//! ```

/// Real Git run in the fixture,
/// with the same index file handed to the second commit's steps.
use super::git_starts::git_in_index;
/// What one control asks a scene to hold,
/// and the repository it becomes.
use super::scene_asked_for::{
    Edit,
    Plan,
    Scene,
};
/// The fixture folder,
/// the Git starts and the aging a scene is built with.
use crate::test_support::{
    age,
    fixture,
    git,
    git_text,
    remove,
};
/// The fixture's own root and the repository inside it.
use std::path::PathBuf;
/// The copy of the scene that is compared against.
use std::process::{
    Command,
    Output,
};
pub(crate) fn scene(
    name: &str,
    plan: &Plan<'_>,
) -> Scene {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    let format: &str = if plan.sha256 {
        "--object-format=sha256"
    } else {
        "--object-format=sha1"
    };
    git(
        repo.as_path(),
        &["init", "--quiet", "--initial-branch=main", format],
    );
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
        git(
            repo.as_path(),
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                format!("160000,{id},{path}").as_str(),
            ],
        );
    }
    git(
        repo.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=first"],
    );
    let old: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    // The untracked cache is written by a status that walks the worktree, as the person's own status writes it; every later
    // write of this index keeps it.
    if plan.untracked_cache {
        git(
            repo.as_path(),
            &["status", "--porcelain", "--untracked-files=normal"],
        );
    }
    let temporary: PathBuf = repo.join(".git").join("scene-index");
    git_in_index(
        repo.as_path(),
        temporary.as_path(),
        &["read-tree", "HEAD"],
        None,
    );
    for edit in plan.edits {
        match edit {
            Edit::Put {
                path,
                mode,
                content,
            } => {
                let blob: String = git_in_index(
                    repo.as_path(),
                    temporary.as_path(),
                    &["hash-object", "-w", "--stdin"],
                    Some(content),
                );
                git_in_index(
                    repo.as_path(),
                    temporary.as_path(),
                    &[
                        "update-index",
                        "--add",
                        "--cacheinfo",
                        format!("{mode},{blob},{path}").as_str(),
                    ],
                    None,
                );
            }
            Edit::Gitlink {
                path,
                id,
            } => {
                git_in_index(
                    repo.as_path(),
                    temporary.as_path(),
                    &[
                        "update-index",
                        "--add",
                        "--cacheinfo",
                        format!("160000,{id},{path}").as_str(),
                    ],
                    None,
                );
            }
            Edit::Delete(path) => {
                git_in_index(
                    repo.as_path(),
                    temporary.as_path(),
                    &["update-index", "--force-remove", path],
                    None,
                );
            }
        }
    }
    let tree: String = git_in_index(repo.as_path(), temporary.as_path(), &["write-tree"], None);
    let new: String = git_text(
        repo.as_path(),
        &[
            "commit-tree",
            tree.as_str(),
            "-p",
            old.as_str(),
            "-m",
            "second",
        ],
    );
    git(repo.as_path(), &["update-ref", "HEAD", new.as_str()]);
    std::fs::remove_file(&temporary).expect("scene index removed");
    return Scene {
        root,
        repo,
        old,
        new,
    };
}

/// A copy of the scene's repository beside it,
/// with times and modes kept.
pub(crate) fn copy_of(scene: &Scene) -> PathBuf {
    let copy: PathBuf = scene.root.join("copy");
    let output: Output = Command::new("cp")
        .args(["--archive", "--no-target-directory"])
        .arg(&scene.repo)
        .arg(&copy)
        .output()
        .expect("cp");
    assert!(
        output.status.success(),
        "cp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    return copy;
}

/// End a scene.
pub(crate) fn finish(scene: Scene) {
    remove(scene.root.as_path());
}
