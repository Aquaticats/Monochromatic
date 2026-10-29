//! Ignored entries are decided from real rule files in disposable folders,
//!  and the global excludes file is always a named fixture,
//!  never the real home folder.

/// The matcher under test,
///  with its sources and the size bound.
use super::{GlobalSource, IgnoreAnswer, IgnoreMatcher, MAX_ENTRIES};
/// Real listings give the same entries,
///  paths,
///  and directory kinds the reader hands the matcher.
use crate::workspace::{DirectoryEntry, Workspace};
/// Disposable project files.
use std::{fs, path::Path};

/// Write a file,
///  creating its folders.
fn put(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().expect("parent folder")).expect("fixture folder");
    fs::write(path, text).expect("fixture file");
}

/// A canonical disposable work tree:
///  `.git` is an empty folder,
///  because the rules are read from files and git itself is never run.
fn work_tree() -> (tempfile::TempDir, Workspace) {
    let directory = tempfile::tempdir().expect("disposable project");
    fs::create_dir(directory.path().join(".git")).expect("git folder");
    let workspace = Workspace::new(directory.path()).expect("open project");
    return (directory, workspace);
}

/// The listing of a folder of the workspace as the reader would produce it.
fn listing(workspace: &Workspace, folder: &str) -> (std::path::PathBuf, Vec<DirectoryEntry>) {
    // An empty name must give the root itself, not the root with a trailing separator.
    let path = if folder.is_empty() {
        workspace.root().to_path_buf()
    } else {
        workspace.root().join(folder)
    };
    let entries = workspace.list(&path).expect("listing");
    return (path, entries);
}

/// The ignored names of one answer as text,
///  sorted.
fn named(answer: &IgnoreAnswer) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for name in answer.names.iter() {
        texts.push(name.to_string_lossy().into_owned());
    }
    return texts;
}

/// The ignored names of one folder,
///  sorted.
fn ignored(matcher: &mut IgnoreMatcher, workspace: &Workspace, folder: &str) -> Vec<String> {
    let (path, entries) = listing(workspace, folder);
    let answer = matcher.evaluate(&path, &entries);
    return named(&answer);
}

/// A matcher with no global excludes file.
fn plain() -> IgnoreMatcher {
    return IgnoreMatcher::new(GlobalSource::File(None));
}

/// A file pattern,
///  a negation,
///  a directory-only pattern,
///  and an anchored pattern decide as git does.
#[test]
fn file_directory_negated_and_anchored_patterns() {
    let (directory, workspace) = work_tree();
    let root = directory.path();
    put(
        root,
        ".gitignore",
        "*.log\n!important.log\nbuild/\n/anchored.txt\n",
    );
    for name in [
        "app.log",
        "important.log",
        "main.rs",
        "anchored.txt",
        "sub/anchored.txt",
        "build/out.o",
    ] {
        put(root, name, "x");
    }
    // A plain file called `build` is not a directory, so the directory-only pattern does not match it.
    put(root, "sub/build", "x");
    let mut matcher = plain();
    assert_eq!(
        ignored(&mut matcher, &workspace, ""),
        vec!["anchored.txt", "app.log", "build"],
        "root: file pattern, anchored pattern and directory pattern ignore; the negation and plain files stay"
    );
    assert_eq!(
        ignored(&mut matcher, &workspace, "sub"),
        Vec::<String>::new(),
        "an anchored pattern does not reach a nested folder, and a directory-only pattern spares a file"
    );
}

/// A nested `.gitignore` applies below its folder only,
///  and a deeper negation beats a shallower pattern.
#[test]
fn nested_gitignore_applies_below_its_folder_and_overrides_above() {
    let (directory, workspace) = work_tree();
    let root = directory.path();
    put(root, ".gitignore", "*.tmp\n*.log\n");
    put(root, "src/.gitignore", "!keep.tmp\n*.cache\n");
    for name in [
        "root.tmp",
        "src/a.tmp",
        "src/keep.tmp",
        "src/a.cache",
        "root.cache",
        "src/a.log",
    ] {
        put(root, name, "x");
    }
    let mut matcher = plain();
    assert_eq!(
        ignored(&mut matcher, &workspace, ""),
        vec!["root.tmp"],
        "the nested folder's own pattern does not reach the root"
    );
    assert_eq!(
        ignored(&mut matcher, &workspace, "src"),
        vec!["a.cache", "a.log", "a.tmp"],
        "the root pattern reaches into src, the nested pattern applies, and the nested negation un-ignores keep.tmp"
    );
}

/// Nothing below an ignored directory can be re-included,
///  and a file pattern's negation still works in a folder that is not ignored.
#[test]
fn a_file_below_an_ignored_directory_cannot_be_re_included() {
    let (directory, workspace) = work_tree();
    let root = directory.path();
    put(
        root,
        ".gitignore",
        "build/\n!build/keep.txt\n*.log\n!important.log\n",
    );
    for name in [
        "build/keep.txt",
        "build/out.o",
        "build/nested/deep.o",
        "important.log",
        "app.log",
    ] {
        put(root, name, "x");
    }
    let mut matcher = plain();
    assert_eq!(
        ignored(&mut matcher, &workspace, "build"),
        vec!["keep.txt", "nested", "out.o"],
        "the negation of build/keep.txt does not help: its parent is ignored"
    );
    assert_eq!(
        ignored(&mut matcher, &workspace, "build/nested"),
        vec!["deep.o"],
        "an ignored directory ignores everything inside it at any depth"
    );
    assert_eq!(
        ignored(&mut matcher, &workspace, ""),
        vec!["app.log", "build"],
        "important.log is un-ignored by its negation"
    );
}

/// `info/exclude` and the global excludes file apply,
///  and a `.gitignore` negation wins over both.
#[test]
fn exclude_file_and_global_file_apply_below_gitignore() {
    let (directory, workspace) = work_tree();
    let root = directory.path();
    put(root, ".git/info/exclude", "private.txt\nkeep.bak\n");
    put(root, ".gitignore", "!keep.bak\n");
    let global = directory.path().join(".git").join("global-ignore");
    fs::write(&global, "*.bak\n").expect("global file");
    for name in ["private.txt", "old.bak", "keep.bak", "plain.txt"] {
        put(root, name, "x");
    }
    let mut matcher = IgnoreMatcher::new(GlobalSource::File(Some(global)));
    assert_eq!(
        ignored(&mut matcher, &workspace, ""),
        vec!["old.bak", "private.txt"],
        "exclude and global rules apply; the .gitignore negation re-includes keep.bak above both"
    );
}

/// A folder outside any work tree shows nothing ignored,
///  and says so with default stamps.
#[test]
fn outside_a_work_tree_nothing_is_ignored() {
    let directory = tempfile::tempdir().expect("disposable folder");
    put(directory.path(), ".gitignore", "*.log\n");
    put(directory.path(), "app.log", "x");
    let workspace = Workspace::new(directory.path()).expect("open folder");
    let (path, entries) = listing(&workspace, "");
    let mut matcher = plain();
    let answer = matcher.evaluate(&path, &entries);
    assert_eq!(answer, IgnoreAnswer::default());
}

/// A `.gitignore` above the project folder applies,
///  and one above a nested repository does not.
#[test]
fn rules_apply_from_the_top_of_the_work_tree_and_stop_at_a_nested_repository() {
    let (directory, _workspace) = work_tree();
    let root = directory.path();
    put(root, ".gitignore", "*.log\n");
    put(root, "project/app.log", "x");
    put(root, "project/main.rs", "x");
    // A nested repository starts its own rules.
    put(root, "vendor/.git/HEAD", "ref: refs/heads/main\n");
    put(root, "vendor/lib.log", "x");
    let project = Workspace::new(&root.join("project")).expect("open nested project");
    let vendor = Workspace::new(&root.join("vendor")).expect("open vendored repository");
    let mut matcher = plain();
    assert_eq!(
        ignored(&mut matcher, &project, ""),
        vec!["app.log"],
        "the project folder is below the work tree's top, whose .gitignore applies"
    );
    assert_eq!(
        ignored(&mut matcher, &vendor, ""),
        Vec::<String>::new(),
        "the outer .gitignore does not apply inside a nested repository"
    );
}

/// A linked work tree's `.git` file names its git directory,
///  whose `commondir` holds the shared `info/exclude`.
#[test]
fn a_git_file_finds_the_common_directory_of_a_linked_work_tree() {
    let directory = tempfile::tempdir().expect("disposable folder");
    let root = directory.path();
    put(root, "main/.git/info/exclude", "shared.txt\n");
    put(root, "main/.git/worktrees/linked/commondir", "../..\n");
    put(
        root,
        "linked/.git",
        &format!(
            "gitdir: {}\n",
            root.join("main/.git/worktrees/linked").display()
        ),
    );
    put(root, "linked/shared.txt", "x");
    put(root, "linked/other.txt", "x");
    let workspace = Workspace::new(&root.join("linked")).expect("open linked work tree");
    let mut matcher = plain();
    assert_eq!(ignored(&mut matcher, &workspace, ""), vec!["shared.txt"]);
}

/// Editing a rule file changes the answer and the stamps;
///  an unchanged file keeps both.
#[test]
fn an_edited_rule_file_is_compiled_again_and_changes_the_stamps() {
    let (directory, workspace) = work_tree();
    let root = directory.path();
    put(root, ".gitignore", "*.log\n");
    put(root, "app.log", "x");
    put(root, "app.txt", "x");
    let (path, entries) = listing(&workspace, "");
    let mut matcher = plain();
    let first = matcher.evaluate(&path, &entries);
    let again = matcher.evaluate(&path, &entries);
    assert_eq!(
        first, again,
        "an unchanged rule file gives an identical answer"
    );
    assert_eq!(named(&first), vec!["app.log"]);
    put(root, ".gitignore", "*.txt\n# a longer file\n");
    let changed = matcher.evaluate(&path, &entries);
    assert_ne!(
        changed.stamps, first.stamps,
        "the stamp of .gitignore moved"
    );
    assert_eq!(named(&changed), vec!["app.txt"]);
}

/// A directory with more entries than the bound shows nothing ignored,
///  but still reports its stamps.
#[test]
fn a_directory_over_the_entry_bound_dims_nothing() {
    let (directory, workspace) = work_tree();
    let root = directory.path();
    put(root, ".gitignore", "*\n");
    let (path, mut entries) = listing(&workspace, "");
    let template = DirectoryEntry {
        name: "x".into(),
        path: path.join("x"),
        is_directory: false,
    };
    entries.resize(MAX_ENTRIES + 1, template);
    let mut matcher = plain();
    let answer = matcher.evaluate(&path, &entries);
    assert!(answer.names.is_empty());
    assert!(answer.stamps.own.exists(), "the stamps are still reported");
}

/// A tracked file that matches a pattern is reported ignored:
///  git's index is not read,
///  so the matcher cannot know it is tracked.
/// This is the documented difference from `git check-ignore`,
///  pinned here so it stays deliberate.
#[test]
fn a_file_that_matches_a_pattern_is_ignored_whether_or_not_git_tracks_it() {
    let (directory, workspace) = work_tree();
    let root = directory.path();
    put(root, ".gitignore", "*.log\n");
    put(root, "tracked.log", "x");
    // The index holds a stand-in: it is never read.
    put(root, ".git/index", "DIRC");
    let mut matcher = plain();
    assert_eq!(ignored(&mut matcher, &workspace, ""), vec!["tracked.log"]);
}

/// What:
///  Times the first and a repeated decision for a directory at the entry bound,
///  three levels below the work tree's top,
///  with a dozen patterns per rule file;
///  prints microseconds.
///  `#[ignore]` keeps the timing out of the default run.
/// Why:
///  `MAX_ENTRIES` and the claim that matching is cheap next to a read are measured numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test.skip('matching cost at the entry bound', () => { time(evaluate(directory, entries)); });
/// ```
#[test]
#[ignore = "timing measurement; run with --ignored --nocapture"]
fn measure_matching_cost_at_the_entry_bound() {
    let (directory, workspace) = work_tree();
    let root = directory.path();
    let patterns = "*.log\n*.tmp\nbuild/\ndist/\n/target\nnode_modules/\n*.o\n*.swp\n.cache/\ncoverage/\n!keep.log\n*.pyc\n";
    put(root, ".gitignore", patterns);
    put(root, "a/.gitignore", patterns);
    put(root, "a/b/.gitignore", patterns);
    put(root, "a/b/c/.gitignore", patterns);
    let folder = workspace.root().join("a/b/c");
    let mut entries = Vec::new();
    for index in 0..MAX_ENTRIES {
        let name = match index % 5 {
            0 => format!("file{index}.rs"),
            1 => format!("file{index}.log"),
            2 => format!("file{index}.o"),
            3 => format!("dir{index}"),
            _ => format!("file{index}.md"),
        };
        entries.push(DirectoryEntry {
            path: folder.join(&name),
            name: name.into(),
            is_directory: index % 5 == 3,
        });
    }
    let mut matcher = plain();
    let cold_start = std::time::Instant::now();
    let first = matcher.evaluate(&folder, &entries);
    let cold = cold_start.elapsed();
    let warm_start = std::time::Instant::now();
    let second = matcher.evaluate(&folder, &entries);
    let warm = warm_start.elapsed();
    println!(
        "ignore matching at {MAX_ENTRIES} entries, 4 levels of 12 patterns: first call {} us, repeated call {} us, {} ignored",
        cold.as_micros(),
        warm.as_micros(),
        first.names.len()
    );
    assert_eq!(first, second);
}
