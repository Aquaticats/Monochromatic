//! Controls for the cleanliness proof of the conversion on real Git repositories:
//! a worktree whose entries stat data cannot
//!  vouch for (a detached or packed `HEAD`,
//! a racily clean entry,
//! a stale stat) converts when its content equals the committed
//!  blob,
//! and refuses,
//! naming the path and the reason,
//! when the content differs or a conversion could make it differ.

use super::*;
use crate::subvolume_test_support::{PlainPlatform, backdate, checkout_with_worktree};
use crate::test_support::{git, remove};
use std::path::{Path, PathBuf};

/// A dry run,
/// which judges the worktree and changes nothing.
const DRY: ConvertOptions = ConvertOptions { dry_run: true, abandon: false, command_line_attributes: false };

/// The verdict of a dry run of the conversion of `worktree`.
fn dry_run(worktree: &Path) -> Verdict {
    return convert_worktree(&PlainPlatform::new(), worktree, DRY, &Interrupt::never(), &mut |_text: &str| {}).expect("no failure");
}

/// The reasons a dry run lists,
/// or a failure of the test when it would convert.
fn refusal_reasons(worktree: &Path) -> String {
    return match dry_run(worktree) {
        Verdict::Skipped(reasons) => reasons.join("\n"),
        other => panic!("expected a skip, got {other:?}"),
    };
}

/// Assert that the dry run would convert the worktree.
fn assert_converts(worktree: &Path) {
    assert_eq!(dry_run(worktree), Verdict::WouldConvert);
}

/// Age every tracked file and refresh the index,
/// so every entry is provably clean by its stat data.
fn settle(worktree: &Path) {
    backdate(worktree);
    git(worktree, &["update-index", "--refresh"]);
}

/// The administrative directory of a worktree.
fn admin_of(worktree: &Path) -> PathBuf {
    return inspect(worktree).expect("registered").admin;
}

/// A checkout with a settled detached linked worktree named `wt`.
fn settled_worktree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical worktree");
    settle(worktree.as_path());
    return (parent, root, worktree);
}

/// Make every entry of the index racy:
/// the index file becomes older than the files it describes.
fn make_every_entry_racy(worktree: &Path) {
    let index = std::fs::OpenOptions::new().write(true).open(admin_of(worktree).join("index")).expect("index");
    index.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(3_600)).expect("old index time");
}

/// Rewrite a file with `bytes`,
/// keeping its modification time,
/// so only the change time and the content can tell.
fn rewrite_keeping_mtime(path: &Path, bytes: &[u8]) {
    let before = std::fs::metadata(path).expect("metadata").modified().expect("mtime");
    std::fs::write(path, bytes).expect("rewrite");
    std::fs::OpenOptions::new().write(true).open(path).expect("open").set_modified(before).expect("restore mtime");
}

/// A detached `HEAD` that holds a commit name converts,
/// also when the repository has a tag whose name has an `@`.
#[test]
fn a_detached_head_naming_a_commit_converts() {
    let (parent, root) = checkout_with_worktree("content-detached");
    git(root.as_path(), &["tag", "@scope/package@1.0.0"]);
    git(root.as_path(), &["pack-refs", "--all"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical worktree");
    assert!(std::fs::read_to_string(admin_of(worktree.as_path()).join("HEAD")).expect("HEAD").len() == 41, "HEAD holds a bare commit name");
    settle(worktree.as_path());
    assert_converts(worktree.as_path());
    remove(parent.as_path());
}

/// A branch that only `packed-refs` holds resolves although another packed ref has a name with an `@`.
#[test]
fn a_packed_branch_resolves_beside_a_tag_with_an_unusual_name() {
    let (parent, root) = checkout_with_worktree("content-packed");
    git(root.as_path(), &["tag", "@scope/package@1.0.0"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "topic"]);
    git(root.as_path(), &["pack-refs", "--all"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("topic")).expect("canonical worktree");
    assert!(!root.join(".git/refs/heads/topic").exists(), "the branch is only in packed-refs");
    settle(worktree.as_path());
    assert_converts(worktree.as_path());
    remove(parent.as_path());
}

/// A `HEAD` that names a commit the repository does not have refuses,
/// saying the commit is unreadable.
#[test]
fn a_head_naming_no_commit_refuses() {
    let (parent, _root, worktree) = settled_worktree("content-nohead");
    std::fs::write(admin_of(worktree.as_path()).join("HEAD"), format!("{}\n", "1".repeat(40))).expect("HEAD");
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("the commit of HEAD is unreadable"), "{reasons}");
    remove(parent.as_path());
}

/// A racily clean entry whose content equals the committed one is accepted.
#[test]
fn a_racy_entry_with_equal_content_converts() {
    let (parent, _root, worktree) = settled_worktree("content-racy-equal");
    make_every_entry_racy(worktree.as_path());
    assert_converts(worktree.as_path());
    remove(parent.as_path());
}

/// A racily clean entry whose content differs is refused,
/// naming the file and saying its content differs.
#[test]
fn a_racy_entry_with_different_content_refuses() {
    let (parent, _root, worktree) = settled_worktree("content-racy-different");
    rewrite_keeping_mtime(worktree.join("README.md").as_path(), b"README\n");
    make_every_entry_racy(worktree.as_path());
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("README.md differs from the committed content"), "{reasons}");
    remove(parent.as_path());
}

/// A file whose stat data is stale (a copy keeps the index's old inode and times) is accepted when its content is the
///  committed one,
/// and refused when it is not.
#[test]
fn a_stale_stat_is_decided_by_content() {
    let (parent, _root, worktree) = settled_worktree("content-stale");
    std::fs::write(worktree.join("README.md"), b"readme\n").expect("same bytes, new times");
    assert_converts(worktree.as_path());
    std::fs::write(worktree.join("README.md"), b"README\n").expect("other bytes of the same size");
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("README.md differs from the committed content"), "{reasons}");
    remove(parent.as_path());
}

/// A symbolic link is compared by its target:
/// recreated with the same target it is accepted,
/// retargeted it is refused.
#[test]
fn a_symbolic_link_is_compared_by_its_target() {
    let (parent, _root, worktree) = settled_worktree("content-link");
    std::fs::remove_file(worktree.join("link")).expect("remove link");
    std::os::unix::fs::symlink("src/main.rs", worktree.join("link")).expect("same target");
    assert_converts(worktree.as_path());
    std::fs::remove_file(worktree.join("link")).expect("remove link");
    std::os::unix::fs::symlink("README.md", worktree.join("link")).expect("other target");
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("link differs from the committed content"), "{reasons}");
    remove(parent.as_path());
}

/// A checkout whose tree also holds `.gitattributes` lines and the files they name,
/// with a settled detached worktree `attr`.
fn attributed_worktree(name: &str, attributes: &str) -> (PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    // The files are committed before the attributes are written, so the blob of `crlf.txt` keeps its carriage returns.
    std::fs::write(root.join("data.dat"), b"data\n").expect("data");
    std::fs::write(root.join("crlf.txt"), b"one\r\ntwo\r\n").expect("crlf");
    git(root.as_path(), &["add", "data.dat", "crlf.txt"]);
    git(root.as_path(), &["commit", "--quiet", "--message=files"]);
    std::fs::write(root.join(".gitattributes"), attributes).expect("attributes");
    git(root.as_path(), &["add", ".gitattributes"]);
    git(root.as_path(), &["commit", "--quiet", "--message=attributes"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "attr"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("attr")).expect("canonical worktree");
    settle(worktree.as_path());
    return (parent, worktree);
}

/// A path to which a filter attribute applies is refused when stat data cannot vouch for it,
/// naming the path and the
///  attribute;
/// a path the pattern does not match is still judged by content.
#[test]
fn a_filtered_path_refuses_and_an_unfiltered_one_converts() {
    let (parent, worktree) = attributed_worktree("content-filter", "*.dat filter=scrambler\n");
    std::fs::write(worktree.join("README.md"), b"readme\n").expect("same bytes, new times");
    assert_converts(worktree.as_path());
    std::fs::write(worktree.join("data.dat"), b"data\n").expect("same bytes, new times");
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("data.dat") && reasons.contains("filter") && reasons.contains(".gitattributes"), "{reasons}");
    remove(parent.as_path());
}

/// The other attributes that can make worktree bytes differ from the blob (`ident`,
/// `working-tree-encoding`) refuse too,
/// and
///  so does a macro that sets one.
#[test]
fn every_attribute_that_can_change_bytes_refuses() {
    for (label, attributes) in [("ident", "*.dat ident\n"), ("encoding", "*.dat working-tree-encoding=ISO-8859-1\n"), ("macro", "[attr]scrambled filter=scrambler\n*.dat scrambled\n")] {
        let (parent, worktree) = attributed_worktree(format!("content-attribute-{label}").as_str(), attributes);
        std::fs::write(worktree.join("data.dat"), b"data\n").expect("same bytes, new times");
        let reasons: String = refusal_reasons(worktree.as_path());
        assert!(reasons.contains("data.dat") && reasons.contains(".gitattributes"), "{label}: {reasons}");
        remove(parent.as_path());
    }
}

/// A file with a carriage return is refused when stat data cannot vouch for it,
/// because a line-ending conversion may apply.
#[test]
fn a_carriage_return_refuses() {
    let (parent, worktree) = attributed_worktree("content-cr", "* text=auto eol=lf\n");
    std::fs::write(worktree.join("crlf.txt"), b"one\r\ntwo\r\n").expect("same bytes, new times");
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("crlf.txt") && reasons.contains("carriage return"), "{reasons}");
    remove(parent.as_path());
}

/// Line-ending attributes alone do not refuse a file without a carriage return.
#[test]
fn line_ending_attributes_do_not_refuse_a_file_without_a_carriage_return() {
    let (parent, worktree) = attributed_worktree("content-eol", "* text=auto eol=lf\n");
    std::fs::write(worktree.join("data.dat"), b"data\n").expect("same bytes, new times");
    assert_converts(worktree.as_path());
    remove(parent.as_path());
}

/// Attributes set outside the tracked root file refuse as well:
/// `info/attributes`,
/// the file `core.attributesFile` names,
/// and a
///  `.gitattributes` in the directory of the file;
/// a relative `core.attributesFile` cannot be ruled out and refuses.
#[test]
fn attributes_from_every_other_source_refuse() {
    let (parent, worktree) = attributed_worktree("content-sources", "# nothing converts here\n");
    let root: PathBuf = std::fs::canonicalize(parent.join("checkout")).expect("canonical root");
    std::fs::write(worktree.join("data.dat"), b"data\n").expect("same bytes, new times");
    assert_converts(worktree.as_path());
    // The repository's own file.
    std::fs::create_dir_all(root.join(".git/info")).expect("info");
    std::fs::write(root.join(".git/info/attributes"), "*.dat filter=scrambler\n").expect("info attributes");
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("data.dat") && reasons.contains("info/attributes"), "{reasons}");
    std::fs::remove_file(root.join(".git/info/attributes")).expect("remove");
    // A global file named by an absolute path.
    let global: PathBuf = parent.join("global-attributes");
    std::fs::write(&global, "data.dat ident\n").expect("global attributes");
    git(root.as_path(), &["config", "core.attributesFile", global.to_str().expect("UTF-8")]);
    let from_global: String = refusal_reasons(worktree.as_path());
    assert!(from_global.contains("data.dat") && from_global.contains(global.to_str().expect("UTF-8")), "{from_global}");
    // A relative path cannot be ruled out.
    git(root.as_path(), &["config", "core.attributesFile", "relative-attributes"]);
    let from_relative: String = refusal_reasons(worktree.as_path());
    assert!(from_relative.contains("core.attributesFile") && from_relative.contains("not an absolute path"), "{from_relative}");
    git(root.as_path(), &["config", "--unset", "core.attributesFile"]);
    assert_converts(worktree.as_path());
    remove(parent.as_path());
}

/// A `.gitattributes` in a subdirectory applies to the files below it only.
#[test]
fn a_nested_attributes_file_applies_below_its_directory() {
    let (parent, root) = checkout_with_worktree("content-nested");
    std::fs::write(root.join("src/.gitattributes"), "*.rs filter=scrambler\n").expect("nested attributes");
    git(root.as_path(), &["add", "src/.gitattributes"]);
    git(root.as_path(), &["commit", "--quiet", "--message=nested attributes"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "nested"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("nested")).expect("canonical worktree");
    settle(worktree.as_path());
    std::fs::write(worktree.join("README.md"), b"readme\n").expect("same bytes, new times");
    assert_converts(worktree.as_path());
    std::fs::write(worktree.join("src/main.rs"), b"fn main() {}\n").expect("same bytes, new times");
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("src/main.rs") && reasons.contains("src/.gitattributes"), "{reasons}");
    remove(parent.as_path());
}

/// A file larger than the limit is refused with that reason instead of being read into memory.
#[test]
fn a_file_larger_than_the_limit_refuses() {
    let (parent, _root, worktree) = settled_worktree("content-large");
    let large = std::fs::OpenOptions::new().write(true).open(worktree.join("README.md")).expect("open");
    large.set_len(crate::worktree_clean_content::CONTENT_LIMIT + 1).expect("sparse growth");
    drop(large);
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("README.md is larger than"), "{reasons}");
    remove(parent.as_path());
}

/// A change to the mode or the type is refused whatever the content,
/// also when the entry is racy.
#[test]
fn a_mode_or_type_change_is_not_settled_by_content() {
    let (parent, _root, worktree) = settled_worktree("content-mode");
    make_every_entry_racy(worktree.as_path());
    std::fs::set_permissions(worktree.join("README.md"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
    let reasons: String = refusal_reasons(worktree.as_path());
    assert!(reasons.contains("README.md changed its executable bit"), "{reasons}");
    std::fs::set_permissions(worktree.join("README.md"), std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("chmod back");
    std::fs::remove_file(worktree.join("link")).expect("remove link");
    std::fs::write(worktree.join("link"), b"src/main.rs").expect("a file in place of the link");
    let type_reasons: String = refusal_reasons(worktree.as_path());
    assert!(type_reasons.contains("link changed its type"), "{type_reasons}");
    remove(parent.as_path());
}
