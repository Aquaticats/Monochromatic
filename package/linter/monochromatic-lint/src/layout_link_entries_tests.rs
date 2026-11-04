//! What:
//!  Controls of `uses` folder reading.
//! Why:
//!  Every kind of wrong entry must be named, and a right one must pass, because a link is how the tree shows
//!  who depends on whom.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a broken link is reported', () => expect(readLinks(folder, sources, base).findings[0].message).toContain('broken'));
//! ```

use super::{LinkRead, link_command, read_links};
use crate::layout_test_support::package_with;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// The source folders of a fixture, keyed by their real paths.
fn sources_of(root: &Path, folders: &[&str]) -> BTreeMap<PathBuf, PathBuf> {
    let mut sources: BTreeMap<PathBuf, PathBuf> = BTreeMap::<PathBuf, PathBuf>::new();
    for folder in folders {
        let path: PathBuf = root.join(folder);
        sources.insert(std::fs::canonicalize(&path).expect("fixture folder"), path);
    }
    return sources;
}

/// The messages of a read, with the fixture folder removed.
fn messages(read: &LinkRead, root: &Path) -> Vec<String> {
    let prefix: String = format!("{}/", root.display());
    return read
        .findings
        .iter()
        .map(|found| return found.message.replace(prefix.as_str(), ""))
        .collect::<Vec<String>>();
}

/// A folder without a `uses` folder links to nothing and has nothing wrong.
#[test]
fn a_folder_without_uses_links_to_nothing() {
    let fixture = package_with(&[("src/a/mod.rs", "")]);
    let sources = sources_of(&fixture.path, &["src/a"]);
    let read: LinkRead = read_links(&fixture.path.join("src/a"), &sources, &fixture.path);
    assert!(read.links.is_empty());
    assert!(read.findings.is_empty());
}

/// A relative link to a source folder, named like it, is a good entry.
#[cfg(unix)]
#[test]
fn a_correct_link_is_a_good_entry() {
    let fixture = package_with(&[("src/a/mod.rs", ""), ("src/b/mod.rs", "")]);
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    let sources = sources_of(&fixture.path, &["src/a", "src/b"]);
    let read: LinkRead = read_links(&fixture.path.join("src/a"), &sources, &fixture.path);
    assert!(read.findings.is_empty(), "{:?}", read.findings);
    assert_eq!(read.links.len(), 1);
    assert_eq!(read.links[0].target, fixture.path.join("src/b"));
    assert_eq!(read.links[0].entry, fixture.path.join("src/a/uses/b"));
}

/// Each kind of wrong entry is named in its own finding and gives no link.
#[cfg(unix)]
#[test]
fn every_kind_of_wrong_entry_is_named() {
    let fixture = package_with(&[
        ("src/a/mod.rs", ""),
        ("src/b/mod.rs", ""),
        ("src/plain/mod.rs", "mod x;"),
        ("outside/mod.rs", ""),
        ("src/a/uses/not_a_link/mod.rs", ""),
        ("src/a/uses/readme.txt", "words"),
        ("src/a/file_target.rs", ""),
    ]);
    crate::layout_test_support::link(&fixture.path, "src/a/uses/broken", "../../missing");
    crate::layout_test_support::link(&fixture.path, "src/a/uses/file_target.rs", "../file_target.rs");
    crate::layout_test_support::link(&fixture.path, "src/a/uses/outside", "../../../outside");
    crate::layout_test_support::link(&fixture.path, "src/a/uses/not_a_source", "../../../src");
    crate::layout_test_support::link(&fixture.path, "src/a/uses/a", "..");
    crate::layout_test_support::link(&fixture.path, "src/a/uses/wrong_name", "../../b");
    let sources = sources_of(&fixture.path, &["src/a", "src/b"]);
    let read: LinkRead = read_links(&fixture.path.join("src/a"), &sources, &fixture.path);
    assert!(read.links.is_empty(), "{:?}", read.links);
    let text: Vec<String> = messages(&read, &fixture.path);
    let expect = |needle: &str| {
        assert!(text.iter().any(|message| return message.contains(needle)), "no finding with {needle:?} in {text:#?}");
    };
    expect("src/a/uses/broken is a broken link");
    expect("src/a/uses/file_target.rs points to ../file_target.rs, which is a file");
    expect("src/a/uses/outside points to ../../../outside, which is not a source folder");
    expect("src/a/uses/not_a_source points to ../../../src, which is not a source folder");
    expect("src/a/uses/a points to the folder it is in");
    expect("src/a/uses/wrong_name points to the folder b, so the link must be named b");
    expect("src/a/uses/not_a_link is not a symbolic link");
    expect("src/a/uses/readme.txt is not a symbolic link");
    assert_eq!(text.len(), 8);
}

/// `uses` as a file, or as a link, is one finding about it and nothing more.
#[cfg(unix)]
#[test]
fn uses_must_be_a_real_folder() {
    let fixture = package_with(&[("src/a/mod.rs", ""), ("src/b/mod.rs", ""), ("src/b/uses", "a file")]);
    crate::layout_test_support::link(&fixture.path, "src/a/uses", "../b");
    let sources = sources_of(&fixture.path, &["src/a", "src/b"]);
    for folder in ["src/a", "src/b"] {
        let read: LinkRead = read_links(&fixture.path.join(folder), &sources, &fixture.path);
        assert_eq!(read.findings.len(), 1, "{folder}: {:?}", read.findings);
        assert!(read.findings[0].message.contains("must be a folder of links"), "{}", read.findings[0].message);
        assert!(read.links.is_empty());
    }
}

/// The command in a finding creates a relative link named after the folder.
#[test]
fn the_link_command_is_relative_and_named_after_the_target() {
    let base: PathBuf = PathBuf::from("/pkg");
    let command: String = link_command(&base.join("src/top"), &base.join("src/middle"), &base);
    assert_eq!(command, "mkdir --parents src/top/uses && ln --symbolic ../../middle src/top/uses/middle");
}
