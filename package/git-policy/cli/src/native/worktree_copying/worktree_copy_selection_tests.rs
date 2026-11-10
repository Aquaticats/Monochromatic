//! Selection checks against real Git:
//! what the ignored-state copy takes from a source worktree.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::cmp::Ordering;

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
    assert!(repository_path_within(b"cache", b"cache"));
    assert!(repository_path_within(b"cache/a/b", b"cache"));
    assert!(!repository_path_within(b"cache2", b"cache"));
    assert!(!repository_path_within(b"a", b"cache/a"));
}

/// A parent and a child are joined with one separator,
/// and the child keeps every byte it has.
#[test]
fn a_repository_path_is_joined_from_its_parent_and_a_name() {
    assert_eq!(join_repository_path(b"cache", b"a"), b"cache/a".to_vec());
    assert_eq!(join_repository_path(b"cache", b"a\xff"), b"cache/a\xff".to_vec());
    assert_eq!(parent_of_repository_path(b"cache/a/b"), Some(b"cache/a".as_slice()));
    assert_eq!(parent_of_repository_path(b"only"), None);
}

/// Every unsafe shape Git could never legitimately produce is refused,
/// and ordinary names pass,
/// including a name that is not text.
#[test]
fn unsafe_repository_paths_are_refused() {
    for unsafe_path in [
        b"".as_slice(),
        b"/abs",
        b"a//b",
        b"./a",
        b"a/./b",
        b"a/../b",
        b"..",
        b"a/..",
        b"a/",
    ] {
        let failure: WorktreeCopyError =
            assert_safe_repository_path(unsafe_path).expect_err(shown_repository_path(unsafe_path).as_str());
        assert!(
            failure
                .0
                .starts_with("monochromatic-git-policy-cli: Git returned unsafe ignored path "),
            "{unsafe_path:?}"
        );
    }
    for safe_path in [
        b"a".as_slice(),
        b"a/b.c",
        b".hidden",
        b"a/..b",
        b"a b/c",
        b"a/b\xff.bin",
    ] {
        assert!(assert_safe_repository_path(safe_path).is_ok(), "{}", shown_repository_path(safe_path));
    }
}

/// A validated path joins its components under the root,
/// and a name that is not text reaches the filesystem as it is.
#[test]
fn filesystem_path_joins_validated_components() {
    use std::os::unix::ffi::OsStrExt;
    let joined: PathBuf = filesystem_path(Path::new("/repo"), b"a/b.c").expect("joined");
    assert_eq!(joined, PathBuf::from("/repo/a/b.c"));
    assert!(filesystem_path(Path::new("/repo"), b"a/../b").is_err());
    let awkward: PathBuf = filesystem_path(Path::new("/repo"), b"cache/bad\xff.bin").expect("joined");
    assert_eq!(
        awkward.file_name().expect("name").as_bytes(),
        b"bad\xff.bin",
        "the name keeps every byte it has"
    );
}

/// Order matches JavaScript's code-unit order,
/// which differs from byte order for supplementary characters.
#[test]
fn js_order_compares_utf16_code_units() {
    assert_eq!(js_order("\u{FFFD}", "\u{1F600}"), Ordering::Greater);
    assert_eq!(js_order("a", "b"), Ordering::Less);
    assert_eq!(js_order("same", "same"), Ordering::Equal);
}

/// A repository path that is text keeps the order the incumbent gave it,
/// and two names whose text is equal are ordered by their bytes,
/// so a walk of names that are not text is still reproducible.
#[test]
fn repository_path_order_keeps_the_incumbents_order_and_settles_equal_text_by_bytes() {
    assert_eq!(repository_path_order(b"a", b"b"), Ordering::Less);
    // UTF-16 order puts U+FFFD above U+1F600, which byte order does not.
    assert_eq!(
        repository_path_order("\u{FFFD}".as_bytes(), "\u{1F600}".as_bytes()),
        Ordering::Greater
    );
    assert_eq!(repository_path_order(b"a\xff", b"a\xfe"), Ordering::Greater);
    assert_eq!(repository_path_order(b"a\xff", b"a\xff"), Ordering::Equal);
}

/// Ignored directories are reported once;
/// files under them are collapsed into the outermost root.
#[test]
fn ignored_directories_collapse_into_outermost_root() {
    let root: PathBuf = fixture("selection-collapse");
    let source: PathBuf = ignoring(root.as_path(), "source", "node_modules/\ncache/\n");
    std::fs::create_dir_all(source.join("node_modules/pkg")).expect("nested");
    std::fs::write(source.join("node_modules/pkg/index.js"), b"x").expect("file");
    std::fs::create_dir_all(source.join("cache/inner")).expect("cache");
    std::fs::write(source.join("cache/inner/data"), b"y").expect("cache file");
    let roots: Vec<Vec<u8>> =
        read_ignored_roots(&git_context(), source.as_path(), &[]).expect("roots");
    assert_eq!(roots, vec![b"cache".to_vec(), b"node_modules".to_vec()]);
    remove(root.as_path());
}

/// An ignored root whose own name is not text is selected exactly,
/// not as the text a conversion would make of it.
#[cfg(unix)]
#[test]
fn an_ignored_root_whose_name_is_not_text_is_selected_byte_for_byte() {
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("selection-non-text-root");
    let source: PathBuf = ignoring(root.as_path(), "source", "*.log\n");
    let mut named: PathBuf = source.clone();
    named.push(std::ffi::OsStr::from_bytes(b"f\x80.log"));
    std::fs::write(named.as_path(), b"log").expect("log file");
    let roots: Vec<Vec<u8>> =
        read_ignored_roots(&git_context(), source.as_path(), &[]).expect("roots");
    assert_eq!(roots, vec![b"f\x80.log".to_vec()]);
    remove(root.as_path());
}

/// Nested `.gitignore`,
/// `info/exclude` and `core.excludesFile` all define what is ignored,
/// and symbolic links are selected.
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
    let roots: Vec<Vec<u8>> =
        read_ignored_roots(&git_context(), source.as_path(), &[]).expect("roots");
    assert!(roots.contains(&b"sub/app.log".to_vec()), "{roots:?}");
    assert!(roots.contains(&b"excluded-by-info".to_vec()), "{roots:?}");
    assert!(roots.contains(&b"from-global".to_vec()), "{roots:?}");
    #[cfg(unix)]
    assert!(
        roots.contains(&b"excluded-by-info-link".to_vec()),
        "{roots:?}"
    );
    remove(root.as_path());
}

/// A FIFO that Git's listing omits is selected by the special-entry walk,
/// and an unignored FIFO is not.
#[cfg(unix)]
#[test]
fn ignored_special_entries_are_selected_only_when_ignored() {
    let root: PathBuf = fixture("selection-special");
    let source: PathBuf = ignoring(root.as_path(), "source", "ignored-pipe\n");
    make_fifo(source.join("ignored-pipe").as_path());
    make_fifo(source.join("plain-pipe").as_path());
    let roots: Vec<Vec<u8>> =
        read_ignored_roots(&git_context(), source.as_path(), &[]).expect("roots");
    assert!(roots.contains(&b"ignored-pipe".to_vec()), "{roots:?}");
    assert!(!roots.contains(&b"plain-pipe".to_vec()), "{roots:?}");
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
    let roots: Vec<Vec<u8>> =
        read_ignored_roots(&git_context(), source.as_path(), excluded.as_slice()).expect("roots");
    assert_eq!(roots, vec![b"nested".to_vec()]);
    remove(root.as_path());
}

/// A directory that is not a repository makes Git fail,
/// and the failure is reported with its cause.
#[test]
fn listing_outside_a_repository_fails_with_cause() {
    let root: PathBuf = fixture("selection-not-repo");
    let failure: WorktreeCopyError =
        read_ignored_roots(&git_context(), root.as_path(), &[]).expect_err("not a repository");
    assert!(failure.0.contains("not a git repository"));
    remove(root.as_path());
}

/// A source with an ignored cache directory that holds the named file next to a sibling,
/// and another ignored root.
fn source_with_named_file(parent: &Path, name: &str) -> PathBuf {
    let source: PathBuf = ignoring(parent, name, ".cache/\nnode_modules/\n");
    std::fs::create_dir_all(source.join(".cache")).expect("cache");
    std::fs::write(source.join(".cache/rules.txt"), b"rule").expect("rules");
    std::fs::write(source.join(".cache/other.txt"), b"other").expect("other");
    std::fs::create_dir_all(source.join("node_modules/pkg")).expect("modules");
    std::fs::write(source.join("node_modules/pkg/index.js"), b"x").expect("module file");
    return source;
}

/// An ignored named file is selected by itself,
/// and nothing else the source ignores comes along.
#[test]
fn named_ignored_file_is_selected_alone() {
    let root: PathBuf = fixture("selection-named");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let roots: Vec<Vec<u8>> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[b".cache/rules.txt".to_vec()],
    )
    .expect("roots");
    assert_eq!(roots, vec![b".cache/rules.txt".to_vec()]);
    remove(root.as_path());
}

/// The named spec is the wrapper's own,
/// so two glob modes that disagree in the caller's environment must not make
/// the listing refuse it and the copy fail.
#[test]
fn a_named_ignored_file_is_selected_under_the_callers_conflicting_glob_modes() {
    let root: PathBuf = fixture("selection-named-glob-modes");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let mut inherited: crate::transaction_git::GitContext = git_context();
    inherited.overlay.push((std::ffi::OsString::from("GIT_GLOB_PATHSPECS"), std::ffi::OsString::from("1")));
    inherited.overlay.push((std::ffi::OsString::from("GIT_NOGLOB_PATHSPECS"), std::ffi::OsString::from("1")));
    let roots: Vec<Vec<u8>> = read_named_ignored_roots(&inherited, source.as_path(), &[b".cache/rules.txt".to_vec()])
        .expect("the listing answers despite the caller's modes");
    assert_eq!(roots, vec![b".cache/rules.txt".to_vec()]);
    remove(root.as_path());
}

/// A named file the source does not have selects nothing and is not an error.
#[test]
fn absent_named_file_selects_nothing() {
    let root: PathBuf = fixture("selection-named-absent");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let roots: Vec<Vec<u8>> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[b".cache/missing.txt".to_vec()],
    )
    .expect("roots");
    assert!(roots.is_empty(), "{roots:?}");
    remove(root.as_path());
}

/// A named file that is tracked,
/// or that exists but no ignore rule covers,
/// is never selected.
#[test]
fn named_file_that_git_does_not_ignore_is_not_selected() {
    let root: PathBuf = fixture("selection-named-unignored");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    std::fs::write(source.join("plain.txt"), b"plain").expect("plain");
    std::fs::write(source.join("tracked.txt"), b"tracked").expect("tracked");
    git(source.as_path(), &["add", "tracked.txt"]);
    let roots: Vec<Vec<u8>> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[b"plain.txt".to_vec(), b"tracked.txt".to_vec()],
    )
    .expect("roots");
    assert!(roots.is_empty(), "{roots:?}");
    remove(root.as_path());
}

/// A named directory is not expanded:
/// only a name Git lists exactly is selected.
#[test]
fn named_directory_selects_nothing() {
    let root: PathBuf = fixture("selection-named-directory");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let roots: Vec<Vec<u8>> =
        read_named_ignored_roots(&git_context(), source.as_path(), &[b".cache".to_vec()])
            .expect("roots");
    assert!(roots.is_empty(), "{roots:?}");
    remove(root.as_path());
}

/// A pathspec magic or glob in a configured name is matched literally,
/// never expanded.
#[test]
fn named_path_is_matched_literally() {
    let root: PathBuf = fixture("selection-named-literal");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let roots: Vec<Vec<u8>> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[b".cache/*.txt".to_vec(), b":(glob).cache/**".to_vec()],
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
    let roots: Vec<Vec<u8>> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[b"[x].txt".to_vec()],
    )
    .expect("roots");
    assert_eq!(roots, vec![b"[x].txt".to_vec()]);
    remove(root.as_path());
}

/// A named ignored file whose name is not text is found by those bytes.
#[cfg(unix)]
#[test]
fn a_named_ignored_file_whose_name_is_not_text_is_found_by_its_bytes() {
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("selection-named-non-text");
    let source: PathBuf = ignoring(root.as_path(), "source", "*.env\n");
    let mut named: PathBuf = source.clone();
    named.push(std::ffi::OsStr::from_bytes(b"bad\xff.env"));
    std::fs::write(named.as_path(), b"SECRET=1\n").expect("env");
    let roots: Vec<Vec<u8>> = read_named_ignored_roots(
        &git_context(),
        source.as_path(),
        &[b"bad\xff.env".to_vec()],
    )
    .expect("roots");
    assert_eq!(roots, vec![b"bad\xff.env".to_vec()]);
    remove(root.as_path());
}

/// A name that could leave the worktree is refused before Git or the filesystem is asked.
#[test]
fn unsafe_named_path_is_refused() {
    let root: PathBuf = fixture("selection-named-unsafe");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    for unsafe_path in [b"../escape".to_vec(), b"/abs".to_vec(), Vec::new()] {
        let failure: WorktreeCopyError = read_named_ignored_roots(
            &git_context(),
            source.as_path(),
            std::slice::from_ref(&unsafe_path),
        )
        .expect_err(shown_repository_path(unsafe_path.as_slice()).as_str());
        assert!(
            failure
                .0
                .starts_with("monochromatic-git-policy-cli: Git returned unsafe ignored path "),
            "{unsafe_path:?}"
        );
    }
    remove(root.as_path());
}

/// The context of a caller whose environment turns pathspec magic off,
/// as `GIT_LITERAL_PATHSPECS=1` in a person's shell does for every Git the copy starts.
fn literal_pathspec_context() -> crate::transaction_git::GitContext {
    let mut context = git_context();
    context.overlay.push((std::ffi::OsString::from("GIT_LITERAL_PATHSPECS"), std::ffi::OsString::from("1")));
    return context;
}

/// A configured ignored file is still found when the caller's environment turns pathspec magic off:
/// the listing's own `:(top,literal)` pathspec is magic,
/// so a copy that honoured the caller's mode would leave the rules file or a `.env` behind in the source.
#[test]
fn named_ignored_file_is_selected_under_literal_pathspec_mode() {
    let root: PathBuf = fixture("selection-named-literal-mode");
    let source: PathBuf = ignoring(root.as_path(), "source", ".env\n");
    std::fs::write(source.join(".env"), b"SECRET=1\n").expect("env");
    let roots: Vec<Vec<u8>> = read_named_ignored_roots(&literal_pathspec_context(), source.as_path(), &[b".env".to_vec()]).expect("roots");
    assert_eq!(roots, vec![b".env".to_vec()], "a configured ignored file must be copied under the caller's literal pathspec mode");
    remove(root.as_path());
}

/// The explicit selection reaches the named files only,
/// while the standard selection reaches every ignored root.
#[test]
fn selection_modes_choose_the_roots() {
    let root: PathBuf = fixture("selection-modes");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let named: Vec<Vec<u8>> = select_roots(
        &git_context(),
        source.as_path(),
        &[],
        &CopySelection::Paths(vec![b".cache/rules.txt".to_vec()]),
    )
    .expect("named");
    assert_eq!(named, vec![b".cache/rules.txt".to_vec()]);
    let all: Vec<Vec<u8>> =
        select_roots(&git_context(), source.as_path(), &[], &CopySelection::Ignored)
            .expect("all");
    assert_eq!(all, vec![b".cache".to_vec(), b"node_modules".to_vec()]);
    remove(root.as_path());
}

/// The selection asks about the source worktree it was given,
/// never about the repository the user's global options pick.
#[test]
fn selection_ignores_the_users_global_git_options() {
    let root: PathBuf = fixture("selection-global-options");
    let source: PathBuf = source_with_named_file(root.as_path(), "source");
    let mut context: GitContext = git_context();
    context.global_prefix = vec![
        "--git-dir".into(),
        root.join("elsewhere").into_os_string(),
    ];
    let named: Vec<Vec<u8>> = read_named_ignored_roots(
        &context,
        source.as_path(),
        &[b".cache/rules.txt".to_vec()],
    )
    .expect("named roots");
    assert_eq!(named, vec![b".cache/rules.txt".to_vec()]);
    let all: Vec<Vec<u8>> =
        read_ignored_roots(&context, source.as_path(), &[]).expect("ignored roots");
    assert_eq!(all, vec![b".cache".to_vec(), b"node_modules".to_vec()]);
    remove(root.as_path());
}

/// The ignore question for a special entry is asked about the source worktree,
/// whatever global options the user passed.
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
    let roots: Vec<Vec<u8>> = read_ignored_roots(&context, source.as_path(), &[]).expect("roots");
    assert!(roots.contains(&b"ignored-pipe".to_vec()), "{roots:?}");
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
    let roots: Vec<Vec<u8>> =
        read_ignored_roots(&git_context(), source.as_path(), excluded.as_slice()).expect("roots");
    assert_eq!(roots, vec![b"pipe".to_vec()]);
    remove(root.as_path());
}

/// What:
///  The `check-ignore` start that decides whether a special entry (a FIFO,
/// a socket) is ignored.
/// Why:
///   The owner's rule is that no Git start of the wrapper gets `--quiet` or `-q`.
/// The exit status is the answer either way
///  (0 for an ignored path,
/// 1 for any other;
/// Git 2.56.0 `builtin/check-ignore.c` gates only the printing on the option),
/// so
///  the start keeps its answer without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await isIgnoredSpecialPath(recordingGit, repo, 'ignored.fifo')).toBe(true); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_ignore_question_about_a_special_entry_starts_check_ignore_without_quiet() {
    let root: PathBuf = fixture("selection-check-ignore-no-quiet");
    let repo: PathBuf = ignoring(root.as_path(), "repo", "ignored.fifo\n");
    let (context, log) = crate::recording_git::recording_context(root.as_path());
    assert!(is_ignored_special_path(&context, repo.as_path(), b"ignored.fifo").expect("answer"), "an ignored path answers yes");
    assert!(!is_ignored_special_path(&context, repo.as_path(), b"kept.fifo").expect("answer"), "any other path answers no");
    let asks: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.contains("check-ignore"); }).collect();
    assert_eq!(asks.len(), 2, "one start for each question: {asks:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(root.as_path());
}
