//! Target verdicts against a disposable project and a disposable outside directory.

use super::{Classified, TargetRefusal, classify};
use helix_lsp::lsp;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

/// A project with one source file,
///  and an outside directory with one file,
///  both canonical.
struct Fixture {
    /// Keeps the project directory alive for the test.
    _project: TempDir,
    /// Keeps the outside directory alive for the test.
    _elsewhere: TempDir,
    /// Canonical project root.
    root: PathBuf,
    /// Canonical file inside the project.
    inside: PathBuf,
    /// Canonical file outside the project.
    outside: PathBuf,
}

fn fixture() -> Fixture {
    let project = tempfile::tempdir().expect("project directory");
    let elsewhere = tempfile::tempdir().expect("outside directory");
    let root = project.path().canonicalize().expect("canonical project");
    let outside_root = elsewhere.path().canonicalize().expect("canonical outside");
    fs::create_dir_all(root.join("src/@scope")).expect("nested directory");
    let inside = root.join("src/@scope/main.rs");
    fs::write(&inside, "fn main() {}\n").expect("inside file");
    let outside = outside_root.join("library.rs");
    fs::write(&outside, "pub fn library() {}\n").expect("outside file");
    return Fixture {
        _project: project,
        _elsewhere: elsewhere,
        root,
        inside,
        outside,
    };
}

fn file_url(path: &std::path::Path) -> lsp::Url {
    return lsp::Url::from_file_path(path).expect("absolute path");
}

#[test]
fn project_file_is_inside() {
    let fixture = fixture();
    assert_eq!(
        classify(&file_url(&fixture.inside), &fixture.root),
        Classified::InsideProject(fixture.inside.clone())
    );
}

#[test]
fn existing_file_elsewhere_is_outside_and_stays_openable() {
    let fixture = fixture();
    assert_eq!(
        classify(&file_url(&fixture.outside), &fixture.root),
        Classified::OutsideProject(fixture.outside.clone()),
        "a file outside the project root must be flagged outside, not inside or refused"
    );
}

#[test]
fn link_inside_the_project_that_leaves_the_root_is_outside() {
    let fixture = fixture();
    let link = fixture.root.join("src/linked.rs");
    std::os::unix::fs::symlink(&fixture.outside, &link).expect("symbolic link");
    assert_eq!(
        classify(&file_url(&link), &fixture.root),
        Classified::OutsideProject(fixture.outside.clone()),
        "a link leaving the project root was treated as a project file"
    );
}

#[test]
fn sibling_directory_sharing_the_root_prefix_is_outside() {
    let fixture = fixture();
    let mut sibling_name = fixture.root.file_name().expect("root name").to_os_string();
    sibling_name.push("-other");
    let sibling = fixture.root.with_file_name(sibling_name);
    fs::create_dir(&sibling).expect("sibling directory");
    let file = sibling.join("main.rs");
    fs::write(&file, "").expect("sibling file");
    let verdict = classify(&file_url(&file), &fixture.root);
    fs::remove_dir_all(&sibling).expect("remove sibling directory");
    assert_eq!(verdict, Classified::OutsideProject(file));
}

#[test]
fn schemes_other_than_file_are_refused() {
    let fixture = fixture();
    for (text, scheme) in [
        ("untitled:Untitled-1", "untitled"),
        ("jdt://contents/java.base/java/lang/String.class", "jdt"),
        ("https://example.invalid/main.rs", "https"),
    ] {
        let url = lsp::Url::parse(text).expect("parseable address");
        assert_eq!(
            classify(&url, &fixture.root),
            Classified::Refused(TargetRefusal::UnsupportedScheme(scheme.to_string())),
            "a non-file address was not refused"
        );
    }
}

#[test]
fn missing_path_is_refused() {
    let fixture = fixture();
    let missing = fixture.root.join("src/absent.rs");
    assert_eq!(
        classify(&file_url(&missing), &fixture.root),
        Classified::Refused(TargetRefusal::Missing),
        "a missing target was not refused"
    );
}

#[test]
fn directory_is_refused() {
    let fixture = fixture();
    assert_eq!(
        classify(&file_url(&fixture.root.join("src")), &fixture.root),
        Classified::Refused(TargetRefusal::NotAFile)
    );
}

#[test]
fn percent_encoded_address_resolves_to_the_same_file() {
    let fixture = fixture();
    let plain = file_url(&fixture.inside);
    let encoded_text = plain.as_str().replace('@', "%40");
    assert_ne!(
        encoded_text,
        plain.as_str(),
        "the fixture path must contain an encodable character"
    );
    let encoded = lsp::Url::parse(&encoded_text).expect("encoded address");
    assert_eq!(
        classify(&encoded, &fixture.root),
        classify(&plain, &fixture.root),
        "two spellings of one path produced different targets"
    );
    assert_eq!(
        classify(&encoded, &fixture.root),
        Classified::InsideProject(fixture.inside.clone())
    );
}

#[test]
fn refusals_explain_themselves() {
    assert_eq!(
        TargetRefusal::UnsupportedScheme("jdt".to_string()).to_string(),
        "the language server named a 'jdt' address, and only local files can be opened"
    );
    assert_eq!(
        TargetRefusal::Missing.to_string(),
        "the file does not exist"
    );
}
