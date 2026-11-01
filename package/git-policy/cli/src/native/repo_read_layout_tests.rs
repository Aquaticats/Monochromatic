//! Differential controls for the in-process repository location:
//!  whenever the reader answers, `git rev-parse` prints the same five lines for the same start directory,
//!  and every layout it must not guess is declined.

use super::*;
use crate::test_support::{fixture, git, git_output, remove, repository};
use std::os::unix::ffi::OsStringExt;

/// The scope facts that match the hermetic environment `git_output` gives Git: no system file, no global file.
fn hermetic_scope(root: &Path) -> ScopeEnvironment {
    return ScopeEnvironment {
        home: Some(root.join("no-home")),
        xdg: None,
        no_system: true,
        system: None,
        system_default: None,
        global: Some(PathBuf::from("/nonexistent-global-config")),
        unresolvable: false,
    };
}

/// The reader's answer for `directory` in the hermetic environment.
fn read_here(root: &Path, directory: &Path) -> Option<RepoLayout> {
    return read_layout_with(directory, &[], &hermetic_scope(root));
}

/// The five lines real Git prints for the location query, or `None` when Git refuses.
fn git_lines(directory: &Path) -> Option<Vec<Vec<u8>>> {
    let output = git_output(
        directory,
        &["rev-parse", "--path-format=absolute", "--is-bare-repository", "--git-dir", "--git-common-dir", "--show-toplevel", "--show-prefix"],
    );
    if !output.status.success() {
        return None;
    }
    let body: &[u8] = output.stdout.strip_suffix(b"\n").unwrap_or(output.stdout.as_slice());
    return Some(body.split(|byte| { return *byte == b'\n'; }).map(<[u8]>::to_vec).collect());
}

/// The layout spelled as the five lines.
fn spelled(layout: &RepoLayout) -> Vec<Vec<u8>> {
    return vec![
        b"false".to_vec(),
        layout.git_dir.as_os_str().as_encoded_bytes().to_vec(),
        layout.common_dir.as_os_str().as_encoded_bytes().to_vec(),
        layout.worktree_root.as_os_str().as_encoded_bytes().to_vec(),
        layout.prefix.clone(),
    ];
}

/// Require the reader to answer for `directory` and to equal Git; returns the layout.
fn answered_equal(root: &Path, directory: &Path, label: &str) -> RepoLayout {
    let Some(layout) = read_here(root, directory) else {
        panic!("{label}: the reader declined a plain layout");
    };
    assert_eq!(Some(spelled(&layout)), git_lines(directory), "{label}");
    return layout;
}

/// Require the reader to decline for `directory`.
fn declined(root: &Path, directory: &Path, label: &str) {
    assert!(read_here(root, directory).is_none(), "{label}: the reader answered a layout it must leave to Git");
}

/// A main worktree answers at the top level and from nested directories, each equal to Git.
#[test]
fn a_main_worktree_is_read_from_every_depth() {
    let root = fixture("repo-read-main");
    let repo = repository(root.as_path(), "repo");
    let nested = repo.join("a/b/c");
    std::fs::create_dir_all(&nested).expect("nested");
    let top = answered_equal(root.as_path(), repo.as_path(), "top");
    assert!(top.prefix.is_empty(), "the top level has an empty prefix");
    let deep = answered_equal(root.as_path(), nested.as_path(), "nested");
    assert_eq!(deep.prefix, b"a/b/c/".to_vec());
    assert_eq!(deep.git_dir, top.git_dir);
    remove(root.as_path());
}

/// A SHA-256 repository answers with its own object format.
#[test]
fn a_sha256_repository_reports_its_format() {
    let root = fixture("repo-read-sha256");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--object-format=sha256", "--initial-branch=main"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let layout = answered_equal(root.as_path(), repo.as_path(), "sha256");
    assert_eq!(layout.object_format, ObjectFormat::Sha256);
    let named = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "--show-object-format"]).stdout).expect("UTF-8");
    assert_eq!(named.trim(), layout.object_format.name());
    remove(root.as_path());
}

/// A linked worktree answers with its own Git directory under the common one, and declines Git paths it cannot split.
#[test]
fn a_linked_worktree_is_read_and_its_git_paths_are_left_to_git() {
    let root = fixture("repo-read-linked");
    let repo = repository(root.as_path(), "repo");
    let linked = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked.to_str().expect("UTF-8"), "HEAD"]);
    std::fs::create_dir(linked.join("sub")).expect("sub");
    let layout = answered_equal(root.as_path(), linked.join("sub").as_path(), "linked");
    assert_ne!(layout.git_dir, layout.common_dir);
    assert!(git_path(&layout, "index").is_none());
    remove(root.as_path());
}

/// A separate Git directory (a `.git` file) answers with the directory the file names.
#[test]
fn a_gitfile_repository_is_read() {
    let root = fixture("repo-read-gitfile");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    let separate = root.join("separate.git");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--separate-git-dir={}", separate.display())]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let layout = answered_equal(root.as_path(), repo.as_path(), "gitfile");
    assert_eq!(layout.git_dir, std::fs::canonicalize(&separate).expect("separate"));
    remove(root.as_path());
}

/// A start directory reached through a symbolic link, a non-UTF-8 name and a very long path answer as Git does.
#[test]
fn awkward_paths_are_read_as_git_reads_them() {
    let root = fixture("repo-read-paths");
    let repo = repository(root.as_path(), "repo");
    let raw_name = std::ffi::OsString::from_vec(vec![b'r', 0xff, 0xfe, b'x']);
    let raw = repo.join(raw_name);
    std::fs::create_dir(&raw).expect("raw name");
    let mut long = repo.clone();
    for _ in 0..8 {
        long.push("d".repeat(200));
    }
    std::fs::create_dir_all(&long).expect("long path");
    let link = root.join("link");
    std::os::unix::fs::symlink(&raw, &link).expect("link");
    answered_equal(root.as_path(), raw.as_path(), "non-UTF-8");
    answered_equal(root.as_path(), long.as_path(), "long");
    let through_link = answered_equal(root.as_path(), link.as_path(), "symlinked start");
    assert!(through_link.prefix.starts_with(b"r"), "the prefix is the real path below the top level");
    remove(root.as_path());
}

/// Every layout the reader must leave to Git is declined: bare, inside `.git`, outside, `core.bare`,
///  `core.worktree`, reftable, unknown extensions, a linked `.git`, a worktree config and unplain environments.
#[test]
fn layouts_that_need_git_are_declined() {
    let root = fixture("repo-read-declined");
    let repo = repository(root.as_path(), "repo");
    let bare = root.join("bare.git");
    git(root.as_path(), &["init", "--quiet", "--bare", bare.to_str().expect("UTF-8")]);
    declined(root.as_path(), bare.as_path(), "bare");
    declined(root.as_path(), repo.join(".git").as_path(), "inside .git");
    declined(root.as_path(), repo.join(".git/refs").as_path(), "inside .git/refs");
    let outside = root.join("outside");
    std::fs::create_dir(&outside).expect("outside");
    declined(root.as_path(), outside.as_path(), "outside any repository");
    let reftable = root.join("reftable");
    std::fs::create_dir(&reftable).expect("dir");
    git(reftable.as_path(), &["init", "--quiet", "--ref-format=reftable"]);
    declined(root.as_path(), reftable.as_path(), "reftable");
    let variable = |name: &str, value: &str| -> (std::ffi::OsString, std::ffi::OsString) {
        return (std::ffi::OsString::from(name), std::ffi::OsString::from(value));
    };
    let scope = hermetic_scope(root.as_path());
    let moved = [variable("GIT_DIR", "/elsewhere")];
    assert!(read_layout_with(repo.as_path(), &moved, &scope).is_none(), "GIT_DIR in the environment");
    let plain = [variable("GIT_AUTHOR_NAME", "x"), variable("PATH", "/usr/bin")];
    assert!(read_layout_with(repo.as_path(), &plain, &scope).is_some(), "plain names must not decline");
    // The wrapper injects configuration entries for its children; entries that leave discovery alone are plain.
    let injected = [variable("GIT_CONFIG_COUNT", "1"), variable("GIT_CONFIG_KEY_0", "core.hooksPath"), variable("GIT_CONFIG_VALUE_0", "/dev/null")];
    assert!(read_layout_with(repo.as_path(), &injected, &scope).is_some(), "an injected hooks path is plain");
    let injected_bare = [variable("GIT_CONFIG_COUNT", "1"), variable("GIT_CONFIG_KEY_0", "Core.Bare"), variable("GIT_CONFIG_VALUE_0", "true")];
    assert!(read_layout_with(repo.as_path(), &injected_bare, &scope).is_none(), "an injected core.bare");
    let missing_key = [variable("GIT_CONFIG_COUNT", "1")];
    assert!(read_layout_with(repo.as_path(), &missing_key, &scope).is_none(), "a count with no key");
    for (label, key, value) in [
        ("core.bare", "core.bare", "true"),
        ("core.worktree", "core.worktree", "/elsewhere"),
        ("extensions.worktreeConfig", "extensions.worktreeConfig", "true"),
    ] {
        let case = repository(root.as_path(), format!("case-{label}").as_str());
        git(case.as_path(), &["config", "--local", "core.repositoryformatversion", "1"]);
        git(case.as_path(), &["config", "--local", key, value]);
        declined(root.as_path(), case.as_path(), label);
    }
    let unknown = repository(root.as_path(), "unknown-extension");
    git(unknown.as_path(), &["config", "--local", "core.repositoryformatversion", "1"]);
    git(unknown.as_path(), &["config", "--local", "extensions.somethingnew", "1"]);
    declined(root.as_path(), unknown.as_path(), "unknown extension");
    let linked_dot_git = repository(root.as_path(), "linked-dot-git");
    std::fs::rename(linked_dot_git.join(".git"), root.join("moved.git")).expect("move");
    std::os::unix::fs::symlink(root.join("moved.git"), linked_dot_git.join(".git")).expect("link");
    declined(root.as_path(), linked_dot_git.as_path(), ".git is a link");
    remove(root.as_path());
}

/// The package's own parameters (the image tag a developer exports to keep parallel runs apart) are named like Git's
///  variables but Git never reads them, so a shell that exports one must not switch every in-process answer off;
///  a name that only resembles the namespace still declines, because Git could define it.
/// This control keeps the host layout free of ambient state: it failed at the same eight places under every fixture root
///  (`doc/troubleshooting/git-policy-unit-tests-and-ambient-git-variables.md`).
#[test]
fn the_packages_own_variables_do_not_move_discovery() {
    let root = fixture("repo-read-own-variables");
    let repo = repository(root.as_path(), "repo");
    let scope = hermetic_scope(root.as_path());
    let named = |suffix: &str| -> [(std::ffi::OsString, std::ffi::OsString); 1] {
        return [(std::ffi::OsString::from(format!("{WRAPPER_ENVIRONMENT_PREFIX}{suffix}")), std::ffi::OsString::from("tag"))];
    };
    for suffix in ["NATIVE_IMAGE_TAG", "E2E_BASE_TAG"] {
        assert!(
            read_layout_with(repo.as_path(), &named(suffix), &scope).is_some(),
            "{WRAPPER_ENVIRONMENT_PREFIX}{suffix} is the package's own and Git never reads it"
        );
    }
    let lookalike: [(std::ffi::OsString, std::ffi::OsString); 1] = [(
        std::ffi::OsString::from(WRAPPER_ENVIRONMENT_PREFIX.trim_end_matches('_').to_owned() + "X"),
        std::ffi::OsString::from("tag"),
    )];
    assert!(read_layout_with(repo.as_path(), &lookalike, &scope).is_none(), "a name outside the namespace can be Git's");
    remove(root.as_path());
}

/// A scope file that mentions `core.bare`, directly or through any include, makes the reader decline.
#[test]
fn scope_files_that_mention_discovery_keys_are_declined() {
    let root = fixture("repo-read-scopes");
    let repo = repository(root.as_path(), "repo");
    let home = root.join("home");
    std::fs::create_dir(&home).expect("home");
    let mut scope = hermetic_scope(root.as_path());
    scope.home = Some(home.clone());
    scope.global = None;
    assert!(read_layout_with(repo.as_path(), &[], &scope).is_some(), "an absent global file is an empty file");
    std::fs::write(home.join(".gitconfig"), "[user]\n\tname = x\n[includeIf \"gitdir:/nowhere/\"]\n\tpath = extra\n").expect("global");
    std::fs::write(home.join("extra"), "[core]\n\tbare = false\n").expect("included");
    assert!(read_layout_with(repo.as_path(), &[], &scope).is_none(), "a conditional include that mentions core.bare");
    std::fs::write(home.join("extra"), "[core]\n\teditor = x\n").expect("included");
    let layout = read_layout_with(repo.as_path(), &[], &scope).expect("an include without the keys is plain");
    assert_eq!(Some(spelled(&layout)), git_lines(repo.as_path()));
    std::fs::write(home.join(".gitconfig"), "[core\n").expect("broken");
    assert!(read_layout_with(repo.as_path(), &[], &scope).is_none(), "a file the reader cannot parse");
    remove(root.as_path());
}

/// `git_path` answers the names it knows for a main worktree exactly as `rev-parse --git-path` does.
#[test]
fn git_paths_equal_rev_parse() {
    let root = fixture("repo-read-gitpath");
    let repo = repository(root.as_path(), "repo");
    let layout = answered_equal(root.as_path(), repo.as_path(), "main");
    for name in ["index", "objects", "HEAD", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "cli-git-transactions", "packed-refs", "config"] {
        let output = git_output(repo.as_path(), &["rev-parse", "--path-format=absolute", "--git-path", name]);
        let expected: String = String::from_utf8(output.stdout).expect("UTF-8").trim_end().to_string();
        assert_eq!(git_path(&layout, name).map(|path| { return path.display().to_string(); }), Some(expected), "{name}");
    }
    assert!(git_path(&layout, "hooks").is_none(), "hooks follows core.hooksPath and is left to Git");
    // Git prints the target of a linked name, so the reader leaves a linked name to Git.
    std::os::unix::fs::symlink("nowhere", repo.join(".git/MERGE_HEAD")).expect("link");
    assert!(git_path(&layout, "MERGE_HEAD").is_none());
    remove(root.as_path());
}
