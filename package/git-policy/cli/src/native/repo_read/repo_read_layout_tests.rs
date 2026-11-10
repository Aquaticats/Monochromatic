//! Differential controls for the in-process repository location:
//!  whenever the reader answers,
//! `git rev-parse` prints the same five lines for the same start directory,
//!  and every layout it must not guess is declined.

use super::*;
use crate::repo_read_layout_environment_rules::ENV_CONFIG_ENTRY_LIMIT;
use crate::test_support::{fixture, git, git_output, remove, repository};
use std::os::unix::ffi::OsStringExt;

/// The scope facts that match the hermetic environment `git_output` gives Git:
/// no system file,
/// no global file.
pub(super) fn hermetic_scope(root: &Path) -> ScopeEnvironment {
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
pub(super) fn read_here(root: &Path, directory: &Path) -> Option<RepoLayout> {
    return read_layout_with(directory, &[], &hermetic_scope(root));
}

/// The five lines real Git prints for the location query,
/// or `None` when Git refuses.
pub(super) fn git_lines(directory: &Path) -> Option<Vec<Vec<u8>>> {
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
pub(super) fn spelled(layout: &RepoLayout) -> Vec<Vec<u8>> {
    return vec![
        b"false".to_vec(),
        layout.git_dir.as_os_str().as_encoded_bytes().to_vec(),
        layout.common_dir.as_os_str().as_encoded_bytes().to_vec(),
        layout.worktree_root.as_os_str().as_encoded_bytes().to_vec(),
        layout.prefix.clone(),
    ];
}

/// Require the reader to answer for `directory` and to equal Git;
/// returns the layout.
pub(super) fn answered_equal(root: &Path, directory: &Path, label: &str) -> RepoLayout {
    let Some(layout) = read_here(root, directory) else {
        panic!("{label}: the reader declined a plain layout");
    };
    assert_eq!(Some(spelled(&layout)), git_lines(directory), "{label}");
    return layout;
}

/// Require the reader to decline for `directory`.
pub(super) fn declined(root: &Path, directory: &Path, label: &str) {
    assert!(read_here(root, directory).is_none(), "{label}: the reader answered a layout it must leave to Git");
}

/// A main worktree answers at the top level and from nested directories,
/// each equal to Git.
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

/// A linked worktree answers with its own Git directory under the common one,
/// and declines Git paths it cannot split.
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

/// The name of a directory that ends in a carriage return,
/// which Linux allows and Git's path text never includes.
fn with_carriage_return(parent: &Path, name: &str) -> PathBuf {
    let mut bytes: Vec<u8> = name.as_bytes().to_vec();
    bytes.push(b'\r');
    return parent.join(std::ffi::OsString::from_vec(bytes));
}

/// Git strips every trailing carriage return and line feed from the `gitdir:` line (`read_gitfile_gently` in `setup.c`),
///  so a `.git` file written with CRLF names the directory without the carriage return,
/// even when a second Git
///  directory exists under the name with it.
#[test]
fn a_gitfile_ending_in_crlf_names_the_directory_without_the_carriage_return() {
    let root = fixture("repo-read-gitfile-crlf");
    let repo = root.join("repo");
    let decoy_repo = root.join("decoy-repo");
    std::fs::create_dir(&repo).expect("dir");
    std::fs::create_dir(&decoy_repo).expect("dir");
    let admin = root.join("admin");
    let decoy = with_carriage_return(root.as_path(), "admin");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--separate-git-dir={}", admin.display())]);
    git(decoy_repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--separate-git-dir={}", decoy.display())]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    std::fs::write(repo.join(".git"), format!("gitdir: {}\r\n", admin.display())).expect("gitfile with CRLF");
    let layout = answered_equal(root.as_path(), repo.as_path(), "gitfile with CRLF");
    assert_eq!(layout.git_dir, std::fs::canonicalize(&admin).expect("admin"));
    remove(root.as_path());
}

/// Git strips every trailing carriage return and line feed from `commondir` (`get_common_dir_noenv` in `path.c`),
///  so a linked worktree whose `commondir` ends in CRLF has the directory without the carriage return as its common
///  directory,
/// even when a second Git directory exists under the name with it.
#[test]
fn a_commondir_ending_in_crlf_names_the_directory_without_the_carriage_return() {
    let root = fixture("repo-read-commondir-crlf");
    let repo = repository(root.as_path(), "repo");
    let linked = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked.to_str().expect("UTF-8"), "HEAD"]);
    let common = root.join("common");
    let decoy = with_carriage_return(root.as_path(), "common");
    for target in [&common, &decoy] {
        let copied = std::process::Command::new("cp").arg("--recursive").arg(repo.join(".git")).arg(target).status().expect("cp starts");
        assert!(copied.success(), "the Git directory is copied to {}", target.display());
        std::fs::remove_dir_all(target.join("worktrees")).expect("the copy has no worktrees");
    }
    std::fs::write(repo.join(".git/worktrees/linked/commondir"), format!("{}\r\n", common.display())).expect("commondir with CRLF");
    let layout = answered_equal(root.as_path(), linked.as_path(), "commondir with CRLF");
    assert_eq!(layout.common_dir, std::fs::canonicalize(&common).expect("common"));
    remove(root.as_path());
}

/// A start directory reached through a symbolic link,
/// a non-UTF-8 name and a very long path answer as Git does.
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

/// Every layout the reader must leave to Git is declined:
/// bare,
/// inside `.git`,
/// outside,
/// `core.bare`,
///  `core.worktree`,
/// reftable,
/// unknown extensions,
/// a linked `.git` and unplain environments;
///  the per-worktree configuration has its own controls in `repo_read_layout_worktree_tests.rs`.
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

/// The entry limit bounds the walk,
/// not what plain entries mean:
/// exactly the limit of plain entries is read and found plain,
///  and one entry more declines however plain it is.
#[test]
fn plain_entries_at_exactly_the_limit_are_read_and_one_more_declines() {
    let entries = |count: u64| -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
        let mut variables: Vec<(std::ffi::OsString, std::ffi::OsString)> =
            vec![(std::ffi::OsString::from("GIT_CONFIG_COUNT"), std::ffi::OsString::from(count.to_string()))];
        for index in 0..count {
            variables.push((std::ffi::OsString::from(format!("GIT_CONFIG_KEY_{index}")), std::ffi::OsString::from("core.hooksPath")));
            variables.push((std::ffi::OsString::from(format!("GIT_CONFIG_VALUE_{index}")), std::ffi::OsString::from("/dev/null")));
        }
        return variables;
    };
    let declines_nothing = |_: &str| -> bool { return false; };
    assert!(env_config_avoids(entries(ENV_CONFIG_ENTRY_LIMIT).as_slice(), &declines_nothing), "the limit itself is read");
    assert!(!env_config_avoids(entries(ENV_CONFIG_ENTRY_LIMIT + 1).as_slice(), &declines_nothing), "one entry more declines");
}

/// The package's own parameters (the image tag a developer exports to keep parallel runs apart) are named like Git's
///  variables but Git never reads them,
/// so a shell that exports one must not switch every in-process answer off;
///  a name that only resembles the namespace still declines,
/// because Git could define it.
/// An exported tag made eight unit tests fail under every fixture root,
/// which looked like a host layout difference
///  (`doc/troubleshooting/cli-git-unit-tests-ambient-git-variables.md`).
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

/// A scope file that mentions `core.bare`,
/// directly or through any include,
/// makes the reader decline.
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

/// The `GIT_` variables that Git reads only to trace,
/// to word its output,
/// to reach a network or a tool,
/// or to size a limit
///  leave the environment plain:
/// each is checked against Git 2.56.0's reader of it.
#[test]
fn variables_that_cannot_change_a_config_or_attribute_or_layout_answer_are_plain() {
    for name in [
        "GIT_EDITOR", "GIT_TRACE", "GIT_TRACE2", "GIT_TRACE2_REDACT", "GIT_TRACE_CURL", "GIT_MERGE_VERBOSITY", "GIT_FLUSH",
        "GIT_PROGRESS_DELAY", "GIT_PRINT_SHA1_ELLIPSIS", "GIT_ADVICE", "GIT_PAGER_IN_USE", "GIT_CURL_VERBOSE", "GIT_HTTP_USER_AGENT",
        "GIT_HTTP_LOW_SPEED_LIMIT", "GIT_SSL_NO_VERIFY", "GIT_PROXY_COMMAND", "GIT_TCP_KEEPCNT", "GIT_USER_AGENT",
        "GIT_OVERRIDE_VIRTUAL_HOST", "GIT_PROTOCOL_FROM_USER", "GIT_EXTERNAL_DIFF", "GIT_DIFF_OPTS", "GIT_DIFFTOOL_EXTCMD",
        "GIT_MERGETOOL_GUI", "GIT_MAN_VIEWER", "GIT_SHELL_PATH", "GIT_REFLOG_ACTION", "GIT_CHERRY_PICK_HELP", "GIT_ALLOC_LIMIT",
        "GIT_MMAP_LIMIT", "GIT_INDEX_VERSION", "GIT_BASENAME_FACTOR", "GIT_COMMIT_GRAPH_PARANOIA", "GIT_DISABLE_UNTRACKED_CACHE",
        "GIT_FORCE_UNTRACKED_CACHE", "GIT_AUTO_COMMENT_CHAR_CONFIG_WARNING_GIVEN",
    ] {
        let variables: Vec<(OsString, OsString)> = vec![(OsString::from(name), OsString::from("1"))];
        assert!(environment_is_plain(variables.as_slice()), "{name} cannot move an answer");
    }
}

/// Every variable that can change which configuration,
/// attribute tree,
/// repository,
/// index,
/// objects,
/// refs or pathspec meaning an
///  answer rests on keeps the environment from being plain,
/// and so does a variable this list has never heard of.
#[test]
fn variables_that_can_change_an_answer_are_not_plain() {
    for name in [
        "GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CEILING_DIRECTORIES", "GIT_DISCOVERY_ACROSS_FILESYSTEM", "GIT_NAMESPACE", "GIT_PREFIX", "GIT_IMPLICIT_WORK_TREE",
        "GIT_CONFIG", "GIT_CONFIG_PARAMETERS", "GIT_ATTR_SOURCE", "GIT_ATTR_NOSYSTEM", "GIT_ATTR_GLOBAL", "GIT_ATTR_SYSTEM",
        "GIT_NO_REPLACE_OBJECTS", "GIT_REPLACE_REF_BASE", "GIT_GRAFT_FILE", "GIT_SHALLOW_FILE", "GIT_QUARANTINE_PATH",
        "GIT_REFERENCE_BACKEND", "GIT_REF_PARANOIA", "GIT_LITERAL_PATHSPECS", "GIT_GLOB_PATHSPECS", "GIT_NOGLOB_PATHSPECS",
        "GIT_ICASE_PATHSPECS", "GIT_TEMPLATE_DIR", "GIT_TEXTDOMAINDIR", "GIT_NOTES_REF", "GIT_NO_LAZY_FETCH", "GIT_TEST_SPLIT_INDEX",
        "GIT_DEFAULT_HASH", "GIT_A_VARIABLE_GIT_ADDS_LATER",
    ] {
        let variables: Vec<(OsString, OsString)> = vec![(OsString::from(name), OsString::from("1"))];
        assert!(!environment_is_plain(variables.as_slice()), "{name} can move an answer");
    }
}

/// What:
///  A `.git` file whose `gitdir:` line names a Git directory that is not UTF-8 text names that directory,
///  and every other line of the layout still equals Git's.
/// Why:
///   The pointer was read as UTF-8 text, so the whole layout of such a worktree was declined
///  and every read of it started Git.
#[test]
fn a_gitfile_that_names_a_directory_that_is_not_text_names_it() {
    let root: PathBuf = fixture("repo-read-gitfile-not-text");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    let admin: PathBuf = root.join(std::ffi::OsString::from_vec(b"admin\xff".to_vec()));
    let arguments: Vec<std::ffi::OsString> = vec![
        std::ffi::OsString::from("init"),
        std::ffi::OsString::from("--quiet"),
        std::ffi::OsString::from("--initial-branch=main"),
        std::ffi::OsString::from("--separate-git-dir"),
        admin.as_os_str().to_os_string(),
    ];
    git(repo.as_path(), arguments.as_slice());
    let layout: RepoLayout = answered_equal(root.as_path(), repo.as_path(), "gitfile naming a directory that is not text");
    assert_eq!(layout.git_dir, std::fs::canonicalize(admin.as_path()).expect("the Git directory"));
    remove(root.as_path());
}
