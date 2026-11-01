//! Controls for staged additions in explicit-path commits: which entries are carried, what the private index then
//! holds, how pathspec files read, and what the ignored-path refusal says.

use super::*;
use crate::capture_order::CaptureIndexes;
use crate::commit_index::{IndexSelection, initialize_commit_index, list_changed_index_paths, write_private_tree};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::TransactionMode;
use std::path::PathBuf;

/// A repository whose `*.js` files are ignored, with `lib/_a.js` and `lib/_b.js` and `other.js` force-staged as
/// additions, as in issue #1807, and the private indexes beside the real index.
fn scenario(name: &str) -> (PathBuf, PathBuf, CaptureIndexes, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), "*.js\n").expect("ignore file");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", ".gitignore", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::create_dir(repo.join("lib")).expect("lib");
    std::fs::write(repo.join("lib").join("_a.js"), "a1\n").expect("_a");
    std::fs::write(repo.join("lib").join("_b.js"), "b1\n").expect("_b");
    std::fs::write(repo.join("other.js"), "o1\n").expect("other");
    git(repo.as_path(), &["add", "--force", "lib/_a.js", "lib/_b.js", "other.js"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("captured.index"),
        commit_index: repo.join(".git").join("commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    return (root, repo, indexes, base);
}

/// Initializes the private index of an explicit-path commit and lists what it changes against the base.
fn selected(
    repo: &Path,
    indexes: &CaptureIndexes,
    base: &str,
    pathspecs: &[&str],
    file: Option<(&Path, bool)>,
) -> Result<Vec<String>, GitFailure> {
    let specs: Vec<OsString> = pathspecs.iter().map(OsString::from).collect();
    let choice: IndexSelection = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: specs.as_slice(),
        pathspec_file: file.map(|(path, _)| { return path; }),
        pathspec_file_nul: file.is_some_and(|(_, nul)| { return nul; }),
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
    initialize_commit_index(&git_context(), repo, repo.join(".git").join("index").as_path(), indexes, &choice)?;
    return Ok(list_changed_index_paths(
        &git_context(),
        repo,
        indexes.commit_index.as_path(),
        base,
        indexes.object_directory.as_path(),
    )
    .expect("changed")
    .iter()
    .map(|path| { return String::from_utf8_lossy(path).into_owned(); })
    .collect());
}

/// The text of a pathspec list as the reader returns it.
fn read(bytes: &[u8], nul: bool) -> Option<Vec<String>> {
    return pathspecs_of_file(bytes, nul)
        .map(|list| { return list.iter().map(|item| { return item.to_string_lossy().into_owned(); }).collect(); });
}

/// A directory pathspec carries the staged additions below it and leaves staged additions outside it.
#[test]
fn a_directory_pathspec_carries_the_staged_additions_below_it() {
    let (root, repo, indexes, base) = scenario("staged-additions-directory");
    assert_eq!(
        selected(&repo, &indexes, &base, &["lib"], None).expect("selects"),
        vec!["lib/_a.js", "lib/_b.js"]
    );
    remove(root.as_path());
}

/// File pathspecs that an ignore rule matches commit when the real index stages them.
#[test]
fn file_pathspecs_commit_staged_additions_that_an_ignore_rule_matches() {
    let (root, repo, indexes, base) = scenario("staged-additions-files");
    assert_eq!(
        selected(&repo, &indexes, &base, &["lib/_a.js", "lib/_b.js"], None).expect("selects"),
        vec!["lib/_a.js", "lib/_b.js"]
    );
    remove(root.as_path());
}

/// A selection of one staged addition leaves the others out of the commit.
#[test]
fn a_staged_addition_outside_the_selection_stays_out() {
    let (root, repo, indexes, base) = scenario("staged-additions-outside");
    assert_eq!(selected(&repo, &indexes, &base, &["lib/_a.js"], None).expect("selects"), vec!["lib/_a.js"]);
    remove(root.as_path());
}

/// The worktree content wins over the staged content, as native `git commit -- <path>` does.
#[test]
fn the_worktree_content_wins_over_the_staged_content() {
    let (root, repo, indexes, base) = scenario("staged-additions-worktree");
    std::fs::write(repo.join("lib").join("_a.js"), "a2\n").expect("edit");
    selected(&repo, &indexes, &base, &["lib/_a.js"], None).expect("selects");
    let tree: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    assert_eq!(git_text(repo.as_path(), &["show", format!("{tree}:lib/_a.js").as_str()]), "a2");
    remove(root.as_path());
}

/// A path staged and then deleted from the worktree leaves the commit empty, as it does for a tracked path.
#[test]
fn a_staged_addition_removed_from_the_worktree_commits_nothing() {
    let (root, repo, indexes, base) = scenario("staged-additions-removed");
    std::fs::remove_file(repo.join("lib").join("_a.js")).expect("remove");
    assert_eq!(selected(&repo, &indexes, &base, &["lib/_a.js"], None).expect("selects"), Vec::<String>::new());
    remove(root.as_path());
}

/// The private tree equals the tree native Git commits for the same selection.
#[test]
fn the_private_tree_equals_the_tree_native_git_commits() {
    let (root, repo, indexes, base) = scenario("staged-additions-oracle-private");
    selected(&repo, &indexes, &base, &["lib"], None).expect("selects");
    let ours: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    let (native_root, native, _, _) = scenario("staged-additions-oracle-native");
    git(native.as_path(), &["commit", "--quiet", "--message=native", "--", "lib"]);
    assert_eq!(ours, git_text(native.as_path(), &["rev-parse", "HEAD^{tree}"]));
    remove(root.as_path());
    remove(native_root.as_path());
}

/// A pathspec file selects staged additions, newline separated, NUL separated and quoted.
#[test]
fn pathspec_files_select_staged_additions() {
    let (root, repo, indexes, base) = scenario("staged-additions-file-list");
    let list: PathBuf = root.join("list");
    std::fs::write(&list, "lib\n").expect("list");
    assert_eq!(
        selected(&repo, &indexes, &base, &[], Some((list.as_path(), false))).expect("newline"),
        vec!["lib/_a.js", "lib/_b.js"]
    );
    std::fs::write(&list, b"lib/_b.js\0").expect("list");
    assert_eq!(
        selected(&repo, &indexes, &base, &[], Some((list.as_path(), true))).expect("nul"),
        vec!["lib/_b.js"]
    );
    std::fs::write(&list, "\"lib/\\137a.js\"\n").expect("list");
    assert_eq!(
        selected(&repo, &indexes, &base, &[], Some((list.as_path(), false))).expect("quoted"),
        vec!["lib/_a.js"]
    );
    remove(root.as_path());
}

/// A path that is ignored and not staged is still refused, and the message names the way forward.
#[test]
fn an_ignored_path_that_is_not_staged_is_refused_with_the_remedy() {
    let (root, repo, indexes, base) = scenario("staged-additions-refusal");
    std::fs::write(repo.join("lib").join("_c.js"), "c1\n").expect("_c");
    let failure: GitFailure =
        selected(&repo, &indexes, &base, &["lib/_c.js"], None).expect_err("an unstaged ignored path is refused");
    assert!(failure.0.contains("ignored by one of your .gitignore files"), "{failure}");
    assert!(failure.0.contains("git add --force -- <path>"), "{failure}");
    remove(root.as_path());
}

/// Another failure keeps its message unchanged.
#[test]
fn another_failure_keeps_its_message() {
    let other: GitFailure = GitFailure(String::from("git add failed: fatal: bad"));
    assert_eq!(explain_ignored_refusal(other.clone()), other);
}

/// The raw record reader takes a record and rejects a malformed one.
#[test]
fn raw_records_are_read_and_malformed_ones_are_rejected() {
    let oid: String = "1".repeat(40);
    let record: String = format!(":000000 100644 {} {oid} A\0lib/_a.js\0", "0".repeat(40));
    assert_eq!(
        parse_raw_additions(record.as_bytes()).expect("reads"),
        vec![StagedAddition { mode: String::from("100644"), oid, path: b"lib/_a.js".to_vec() }]
    );
    assert!(parse_raw_additions(b":000000 100644 0 1 A\0").is_err());
    assert!(parse_raw_additions(b":000000 100644 0 1 M\0path\0").is_err());
    assert!(parse_raw_additions(b"").expect("empty").is_empty());
}

/// No pathspec reads nothing, so an empty selection never starts Git.
#[test]
fn no_pathspec_reads_no_additions() {
    let (root, repo, _, base) = scenario("staged-additions-none");
    let none: Vec<StagedAddition> = staged_additions_selected(
        &git_context(),
        repo.as_path(),
        repo.join(".git").join("index").as_path(),
        base.as_str(),
        &[],
    )
    .expect("reads");
    assert!(none.is_empty());
    remove(root.as_path());
}

/// Pathspec files read as Git reads them, and anything unreadable is declined.
#[test]
fn pathspec_files_read_as_git_reads_them() {
    assert_eq!(read(b"a\nb\n", false), Some(vec![String::from("a"), String::from("b")]));
    assert_eq!(read(b"a\r\nb", false), Some(vec![String::from("a"), String::from("b")]));
    assert_eq!(read(b"a\0b\0", true), Some(vec![String::from("a"), String::from("b")]));
    assert_eq!(read(b"a\nb\n", true), Some(vec![String::from("a\nb\n")]));
    assert_eq!(read(b"\"q\\\"x\"\n", false), Some(vec![String::from("q\"x")]));
    assert_eq!(read(b"\"\"x\n", true), Some(vec![String::from("\"\"x\n")]));
    assert_eq!(read(b"a\n\nb\n", false), None);
    assert_eq!(read(b"a\0\0b\0", true), None);
    assert_eq!(read(b"\"open\n", false), None);
    assert_eq!(read(b"\"x\"y\"\n", false), None);
    assert_eq!(read(b"", false), Some(Vec::new()));
}

/// Every escape Git writes decodes to its byte, and a malformed escape is declined.
#[test]
fn quoted_elements_decode_every_escape() {
    assert_eq!(unquote_c_style(b"\"\\a\\b\\f\\n\\r\\t\\v\\\\\\\"\""), Some(vec![7, 8, 12, b'\n', b'\r', b'\t', 11, b'\\', b'"']));
    assert_eq!(unquote_c_style(b"\"\\303\\251\""), Some(vec![0xC3, 0xA9]));
    assert_eq!(unquote_c_style(b"\"\\400\""), None);
    assert_eq!(unquote_c_style(b"\"\\08x\""), None);
    assert_eq!(unquote_c_style(b"\"\\0\""), None);
    assert_eq!(unquote_c_style(b"\"\\q\""), None);
    assert_eq!(unquote_c_style(b"\"tail\\"), None);
    assert_eq!(unquote_c_style(b"plain"), None);
}

/// A long selection is carried in several starts.
#[test]
fn a_long_selection_is_carried_in_chunks() {
    let (root, repo, indexes, base) = scenario("staged-additions-chunks");
    let count: usize = ENTRIES_PER_START + 5;
    std::fs::create_dir(repo.join("many")).expect("many");
    for number in 0..count {
        std::fs::write(repo.join("many").join(format!("f{number}.js")), format!("{number}\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--force", "many"]);
    assert_eq!(selected(&repo, &indexes, &base, &["many"], None).expect("selects").len(), count);
    remove(root.as_path());
}
