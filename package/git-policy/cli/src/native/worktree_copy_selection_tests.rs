//! Selection checks against real Git: what the ignored-state copy takes from a source worktree.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// A named FIFO created with the platform's `mkfifo` utility.
fn make_fifo(path: &Path) {
    let status = std::process::Command::new("mkfifo")
        .arg(path)
        .status()
        .expect("mkfifo");
    assert!(status.success(), "mkfifo must create {}", path.display());
}

/// A repository whose `.gitignore` ignores the given patterns.
fn ignoring(parent: &Path, name: &str, patterns: &str) -> PathBuf {
    let root: PathBuf = repository(parent, name);
    std::fs::write(root.join(".gitignore"), patterns.as_bytes()).expect("gitignore");
    return root;
}

/// A component-aware prefix relation between repository paths.
#[test]
fn repository_paths_are_compared_by_component() {
    assert!(repository_path_within("cache", "cache"));
    assert!(repository_path_within("cache/a/b", "cache"));
    assert!(!repository_path_within("cache2", "cache"));
    assert!(!repository_path_within("a", "cache/a"));
}

/// Every unsafe shape Git could never legitimately produce is refused, and ordinary names pass.
#[test]
fn unsafe_repository_paths_are_refused() {
    for unsafe_path in [
        "", "/abs", "a//b", "./a", "a/./b", "a/../b", "..", "a/..", "a/",
    ] {
        let failure: WorktreeCopyError =
            assert_safe_repository_path(unsafe_path).expect_err(unsafe_path);
        assert!(
            failure
                .0
                .starts_with("cli-git: Git returned unsafe ignored path "),
            "{unsafe_path}"
        );
    }
    for safe_path in ["a", "a/b.c", ".hidden", "a/..b", "a b/c"] {
        assert_safe_repository_path(safe_path).expect(safe_path);
    }
}

/// A validated path joins its components under the root.
#[test]
fn filesystem_path_joins_validated_components() {
    let joined: PathBuf = filesystem_path(Path::new("/repo"), "a/b.c").expect("joined");
    assert_eq!(joined, PathBuf::from("/repo/a/b.c"));
    assert!(filesystem_path(Path::new("/repo"), "a/../b").is_err());
}

/// Order matches JavaScript's code-unit order, which differs from byte order for supplementary characters.
#[test]
fn js_order_compares_utf16_code_units() {
    assert_eq!(js_order("\u{FFFD}", "\u{1F600}"), Ordering::Greater);
    assert_eq!(js_order("a", "b"), Ordering::Less);
    assert_eq!(js_order("same", "same"), Ordering::Equal);
}

/// Ignored directories are reported once; files under them are collapsed into the outermost root.
#[test]
fn ignored_directories_collapse_into_outermost_root() {
    let root: PathBuf = fixture("selection-collapse");
    let source: PathBuf = ignoring(root.as_path(), "source", "node_modules/\ncache/\n");
    std::fs::create_dir_all(source.join("node_modules/pkg")).expect("nested");
    std::fs::write(source.join("node_modules/pkg/index.js"), b"x").expect("file");
    std::fs::create_dir_all(source.join("cache/inner")).expect("cache");
    std::fs::write(source.join("cache/inner/data"), b"y").expect("cache file");
    let roots: Vec<String> =
        read_ignored_roots(&git_context(), source.as_path(), &[]).expect("roots");
    assert_eq!(
        roots,
        vec![String::from("cache"), String::from("node_modules")]
    );
    remove(root.as_path());
}

/// Nested `.gitignore`, `info/exclude` and `core.excludesFile` all define what is ignored, and symbolic links are selected.
#[test]
fn every_exclusion_source_and_symbolic_links_are_selected() {
    let root: PathBuf = fixture("selection-sources");
    let source: PathBuf = repository(root.as_path(), "source");
    std::fs::create_dir_all(source.join("sub")).expect("sub");
    std::fs::write(source.join("sub/.gitignore"), b"*.log\n").expect("nested gitignore");
    std::fs::write(source.join("sub/app.log"), b"log").expect("log");
    std::fs::write(source.join(".git/info/exclude"), b"excluded-by-info\n").expect("info exclude");
    std::fs::write(source.join("excluded-by-info"), b"x").expect("info file");
    let global: PathBuf = root.join("global-ignore");
    std::fs::write(global.as_path(), b"from-global\n").expect("global ignore");
    git(
        source.as_path(),
        &[
            "config",
            "core.excludesFile",
            global.to_str().expect("utf8"),
        ],
    );
    std::fs::write(source.join("from-global"), b"g").expect("global file");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("target", source.join("excluded-by-info-link")).expect("link");
        std::fs::write(
            source.join(".git/info/exclude"),
            b"excluded-by-info\nexcluded-by-info-link\n",
        )
        .expect("info exclude with link");
    }
    let roots: Vec<String> =
        read_ignored_roots(&git_context(), source.as_path(), &[]).expect("roots");
    assert!(roots.contains(&String::from("sub/app.log")), "{roots:?}");
    assert!(
        roots.contains(&String::from("excluded-by-info")),
        "{roots:?}"
    );
    assert!(roots.contains(&String::from("from-global")), "{roots:?}");
    #[cfg(unix)]
    assert!(
        roots.contains(&String::from("excluded-by-info-link")),
        "{roots:?}"
    );
    remove(root.as_path());
}

/// A FIFO that Git's listing omits is selected by the special-entry walk, and an unignored FIFO is not.
#[cfg(unix)]
#[test]
fn ignored_special_entries_are_selected_only_when_ignored() {
    let root: PathBuf = fixture("selection-special");
    let source: PathBuf = ignoring(root.as_path(), "source", "ignored-pipe\n");
    make_fifo(source.join("ignored-pipe").as_path());
    make_fifo(source.join("plain-pipe").as_path());
    let roots: Vec<String> =
        read_ignored_roots(&git_context(), source.as_path(), &[]).expect("roots");
    assert!(roots.contains(&String::from("ignored-pipe")), "{roots:?}");
    assert!(!roots.contains(&String::from("plain-pipe")), "{roots:?}");
    remove(root.as_path());
}

/// Entries under an excluded nested root are never walked for special entries.
#[cfg(unix)]
#[test]
fn excluded_roots_are_not_walked_for_special_entries() {
    let root: PathBuf = fixture("selection-excluded");
    let source: PathBuf = ignoring(root.as_path(), "source", "nested/\n");
    std::fs::create_dir_all(source.join("nested")).expect("nested");
    make_fifo(source.join("nested/pipe").as_path());
    let excluded: Vec<PathBuf> =
        vec![std::fs::canonicalize(source.join("nested")).expect("canonical")];
    let roots: Vec<String> =
        read_ignored_roots(&git_context(), source.as_path(), excluded.as_slice()).expect("roots");
    assert_eq!(roots, vec![String::from("nested")]);
    remove(root.as_path());
}

/// A directory that is not a repository makes Git fail, and the failure is reported with its cause.
#[test]
fn listing_outside_a_repository_fails_with_cause() {
    let root: PathBuf = fixture("selection-not-repo");
    let failure: WorktreeCopyError =
        read_ignored_roots(&git_context(), root.as_path(), &[]).expect_err("not a repository");
    assert!(failure.0.contains("not a git repository"));
    remove(root.as_path());
}

/// A source with an ignored cache directory that holds the named file next to a sibling, and another ignored root.
fn source_with_named_file(parent: &Path, name: &str) -> PathBuf {
    let source: PathBuf = ignoring(parent, name, ".cache/\nnode_modules/\n");
    std::fs::create_dir_all(source.join(".cache")).expect("cache");
    std::fs::write(source.join(".cache/rules.txt"), b"rule").expect("rules");
    std::fs::write(source.join(".cache/other.txt"), b"other").expect("other");
    std::fs::create_dir_all(source.join("node_modules/pkg")).expect("modules");
    std::fs::write(source.join("node_modules/pkg/index.js"), b"x").expect("module file");
    return source;
}

/// An ignored named file is selected by itself, and nothing else the source ignores comes along.
#[test]
fn named_ignored_file_is_selected_alone() {
    let root: PathBuf = fixture("selection-named");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let roots: Vec<String> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[String::from(".cache/rules.txt")],
    )
    .expect("roots");
    assert_eq!(roots, vec![String::from(".cache/rules.txt")]);
    remove(root.as_path());
}

/// A named file the source does not have selects nothing and is not an error.
#[test]
fn absent_named_file_selects_nothing() {
    let root: PathBuf = fixture("selection-named-absent");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let roots: Vec<String> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[String::from(".cache/missing.txt")],
    )
    .expect("roots");
    assert!(roots.is_empty(), "{roots:?}");
    remove(root.as_path());
}

/// A named file that is tracked, or that exists but no ignore rule covers, is never selected.
#[test]
fn named_file_that_git_does_not_ignore_is_not_selected() {
    let root: PathBuf = fixture("selection-named-unignored");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    std::fs::write(source.join("plain.txt"), b"plain").expect("plain");
    std::fs::write(source.join("tracked.txt"), b"tracked").expect("tracked");
    git(source.as_path(), &["add", "tracked.txt"]);
    let roots: Vec<String> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[String::from("plain.txt"), String::from("tracked.txt")],
    )
    .expect("roots");
    assert!(roots.is_empty(), "{roots:?}");
    remove(root.as_path());
}

/// A named directory is not expanded: only a name Git lists exactly is selected.
#[test]
fn named_directory_selects_nothing() {
    let root: PathBuf = fixture("selection-named-directory");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let roots: Vec<String> =
        read_named_ignored_roots(&git_context(), source.as_path(), &[String::from(".cache")])
            .expect("roots");
    assert!(roots.is_empty(), "{roots:?}");
    remove(root.as_path());
}

/// A pathspec magic or glob in a configured name is matched literally, never expanded.
#[test]
fn named_path_is_matched_literally() {
    let root: PathBuf = fixture("selection-named-literal");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let roots: Vec<String> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[
            String::from(".cache/*.txt"),
            String::from(":(glob).cache/**"),
        ],
    )
    .expect("roots");
    assert!(roots.is_empty(), "{roots:?}");
    remove(root.as_path());
}

/// A file whose name looks like a glob or pathspec magic is found by its literal name.
#[test]
fn named_path_with_glob_characters_is_found_literally() {
    let root: PathBuf = fixture("selection-named-glob-name");
    let source: PathBuf = ignoring(root.as_path(), "source", "\\[x\\].txt\n");
    std::fs::write(source.join("[x].txt"), b"bracketed").expect("bracketed");
    std::fs::write(source.join("x.txt"), b"plain").expect("plain");
    let roots: Vec<String> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[String::from("[x].txt")],
    )
    .expect("roots");
    assert_eq!(roots, vec![String::from("[x].txt")]);
    remove(root.as_path());
}

/// A name that could leave the worktree is refused before Git or the filesystem is asked.
#[test]
fn unsafe_named_path_is_refused() {
    let root: PathBuf = fixture("selection-named-unsafe");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    for unsafe_path in ["../escape", "/abs", ""] {
        let failure: WorktreeCopyError = read_named_ignored_roots(
            &git_context(),
            source.as_path(),
            &[String::from(unsafe_path)],
        )
        .expect_err(unsafe_path);
        assert!(
            failure
                .0
                .starts_with("cli-git: Git returned unsafe ignored path "),
            "{unsafe_path}"
        );
    }
    remove(root.as_path());
}

/// The explicit selection reaches the named files only, while the standard selection reaches every ignored root.
#[test]
fn selection_modes_choose_the_roots() {
    let root: PathBuf = fixture("selection-modes");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let named: Vec<String> = select_roots(
        &git_context(),
        source.as_path(),
        &[],
        &CopySelection::Paths(vec![String::from(".cache/rules.txt")]),
    )
    .expect("named");
    assert_eq!(named, vec![String::from(".cache/rules.txt")]);
    let all: Vec<String> =
        select_roots(&git_context(), source.as_path(), &[], &CopySelection::Ignored)
            .expect("all");
    assert_eq!(
        all,
        vec![String::from(".cache"), String::from("node_modules")]
    );
    remove(root.as_path());
}

/// The selection asks about the source worktree it was given, never about the repository the user's global options pick.
#[test]
fn selection_ignores_the_users_global_git_options() {
    let root: PathBuf = fixture("selection-global-options");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let mut context: GitContext = git_context();
    context.global_prefix = vec![
        "--git-dir".into(),
        root.join("elsewhere").into_os_string(),
    ];
    let named: Vec<String> = read_named_ignored_roots(
        &context,
        source.as_path(),
        &[String::from(".cache/rules.txt")],
    )
    .expect("named roots");
    assert_eq!(named, vec![String::from(".cache/rules.txt")]);
    let all: Vec<String> =
        read_ignored_roots(&context, source.as_path(), &[]).expect("ignored roots");
    assert_eq!(
        all,
        vec![String::from(".cache"), String::from("node_modules")]
    );
    remove(root.as_path());
}

/// The ignore question for a special entry is asked about the source worktree, whatever global options the user passed.
#[cfg(unix)]
#[test]
fn special_entry_check_ignores_the_users_global_git_options() {
    let root: PathBuf = fixture("selection-global-options-special");
    let source: PathBuf = ignoring(root.as_path(), "source", "ignored-pipe\n");
    make_fifo(source.join("ignored-pipe").as_path());
    let mut context: GitContext = git_context();
    context.global_prefix = vec![
        "--git-dir".into(),
        root.join("elsewhere").into_os_string(),
    ];
    let roots: Vec<String> = read_ignored_roots(&context, source.as_path(), &[]).expect("roots");
    assert!(roots.contains(&String::from("ignored-pipe")), "{roots:?}");
    remove(root.as_path());
}

/// A special entry inside an excluded nested root is skipped even when the outer ignore rules would cover it.
#[cfg(unix)]
#[test]
fn special_entries_under_an_excluded_root_are_not_selected() {
    let root: PathBuf = fixture("selection-excluded-pipe");
    let source: PathBuf = ignoring(root.as_path(), "source", "pipe\n");
    std::fs::create_dir_all(source.join("nested")).expect("nested");
    make_fifo(source.join("nested/pipe").as_path());
    make_fifo(source.join("pipe").as_path());
    let excluded: Vec<PathBuf> =
        vec![std::fs::canonicalize(source.join("nested")).expect("canonical")];
    let roots: Vec<String> =
        read_ignored_roots(&git_context(), source.as_path(), excluded.as_slice()).expect("roots");
    assert_eq!(roots, vec![String::from("pipe")]);
    remove(root.as_path());
}
