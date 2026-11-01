//! Controls for shadow requests and removal.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::transaction_git::{GitOutput, run_git_checked};

/// The shadow path derives from the common directory and the transaction ID.
#[test]
fn shadow_paths_derive_from_the_id() {
    assert_eq!(
        shadow_repository_path(Path::new("/repo/.git"), "0b6c"),
        PathBuf::from("/repo/.git/cli-git/shadow/0b6c")
    );
}

/// Shadow requests select the shadow,
///  disable hooks and remove redirecting variables.
#[test]
fn shadow_requests_select_only_the_shadow() {
    let request: GitRequest =
        shadow_request(Path::new("/s"), Path::new("/cwd"), &["rev-parse", "HEAD"]);
    assert_eq!(
        request.arguments,
        vec![
            OsString::from("--git-dir=/s"),
            OsString::from("-c"),
            OsString::from("core.hooksPath=/s/no-hooks"),
            OsString::from("rev-parse"),
            OsString::from("HEAD"),
        ]
    );
    assert!(request.without_prefix);
    assert_eq!(request.cwd, PathBuf::from("/cwd"));
    assert_eq!(
        request.unset,
        vec![
            OsString::from("GIT_DIR"),
            OsString::from("GIT_WORK_TREE"),
            OsString::from("GIT_COMMON_DIR"),
            OsString::from("GIT_OBJECT_DIRECTORY"),
            OsString::from("GIT_INDEX_FILE"),
        ]
    );
}

/// A redirecting variable in the overlay does not reach a shadow command.
#[test]
fn shadow_commands_ignore_redirects() {
    let root: PathBuf = fixture("shadow-git-redirect");
    let real: PathBuf = repository(root.as_path(), "real");
    let shadow_parent: PathBuf = repository(root.as_path(), "shadow-work");
    git(
        shadow_parent.as_path(),
        &[
            "commit",
            "--quiet",
            "--allow-empty",
            "--message=shadow only",
        ],
    );
    let shadow: PathBuf = shadow_parent.join(".git");
    let mut context: crate::transaction_git::GitContext = git_context();
    context.overlay.push((
        OsString::from("GIT_DIR"),
        real.join(".git").into_os_string(),
    ));
    let output: GitOutput = run_git_checked(
        &context,
        &shadow_request(
            shadow.as_path(),
            shadow.as_path(),
            &["log", "-1", "--format=%s"],
        ),
    )
    .expect("shadow log");
    assert_eq!(
        String::from_utf8_lossy(output.stdout.as_slice()),
        "shadow only\n"
    );
    remove(root.as_path());
}

/// Removal deletes the whole shadow and tolerates an absent one.
#[test]
fn shadows_are_removed_whole() {
    let root: PathBuf = fixture("shadow-git-remove");
    let shadow: PathBuf = root.join("cli-git").join("shadow").join("0b6c");
    std::fs::create_dir_all(shadow.join("objects")).expect("shadow");
    std::fs::write(shadow.join("objects").join("x"), b"x").expect("object");
    remove_shadow_repository(shadow.as_path()).expect("removed");
    assert!(!shadow.exists());
    remove_shadow_repository(shadow.as_path()).expect("absent");
    let file: PathBuf = root.join("file");
    std::fs::write(&file, b"x").expect("file");
    assert!(
        remove_shadow_repository(file.join("child").as_path())
            .expect_err("under a file")
            .0
            .starts_with("removing the shadow repository ")
    );
    remove(root.as_path());
}
