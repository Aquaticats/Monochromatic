//! The unit tests of the workspace root lookup and the manifest walk.

/// Imports the functions under test.
use crate::scan::{find_workspace_root, scan_workspace};
/// Imports the scratch directories.
use crate::test_scratch::scratch;
/// Imports the constants the expectations derive from.
use crate::constants::WORKSPACE_MARKER;

/// Imports the permission helper for the unreadable-file test.
use std::os::unix::fs::PermissionsExt;
/// Imports the path types.
use std::path::{Path, PathBuf};

/// A manifest the rule accepts.
const CLEAN: &str = r#"{ "name": "clean", "dependencies": { "x": "catalog:", "y": "workspace:*" } }"#;

/// A manifest the rule rejects:
/// one literal version.
const LITERAL: &str = r#"{ "name": "literal", "dependencies": { "left-pad": "1.3.0" } }"#;

/// Text that is not JSON at all.
const BROKEN: &str = "{ not json";

/// Writes `content` at `relative` under `root`,
/// creating the directories.
fn write(root: &Path, relative: &str, content: &str) {
    let path: PathBuf = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("a parent directory")).expect("make the directories");
    std::fs::write(&path, content).expect("write the file");
}

/// The paths of a report's findings,
/// in report order.
fn finding_paths(root: &Path) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for finding in scan_workspace(root).findings {
        paths.push(finding.path);
    }
    return paths;
}

#[test]
fn the_root_is_the_nearest_ancestor_that_holds_the_marker() {
    let workspace = scratch();
    write(workspace.path(), WORKSPACE_MARKER, "packages: []\n");
    write(workspace.path(), "a/b/leaf.txt", "x");

    let found = find_workspace_root(&workspace.path().join("a/b"));

    assert_eq!(found, Some(workspace.path().to_path_buf()));
}

#[test]
fn a_directory_tree_with_no_marker_has_no_root() {
    // `/` has no `pnpm-workspace.yaml`, and every ancestor of it is itself.
    assert_eq!(find_workspace_root(Path::new("/")), None);
}

#[test]
fn a_clean_workspace_has_no_findings_and_counts_every_manifest_it_read() {
    let workspace = scratch();
    let readable: [&str; 3] = ["package.json", "a/package.json", ".config/package.json"];
    for relative in readable {
        write(workspace.path(), relative, CLEAN);
    }

    let report = scan_workspace(workspace.path());

    assert!(report.findings.is_empty());
    assert!(report.notes.is_empty());
    assert_eq!(report.manifests_checked, readable.len());
}

#[test]
fn a_literal_version_is_reported_with_its_path_relative_to_the_root() {
    let workspace = scratch();
    write(workspace.path(), "package.json", CLEAN);
    write(workspace.path(), "package/x/package.json", LITERAL);

    assert_eq!(finding_paths(workspace.path()), vec!["package/x/package.json".to_string()]);
}

#[test]
fn unsupported_manifest_names_are_reported_by_name_without_being_read() {
    let workspace = scratch();
    // Bytes that would be an `invalid-manifest` finding if they were read: only the name finding may appear.
    let names: [&str; 3] = ["a/package.json5", "b/package.yaml", "c/package.yml"];
    for name in names {
        write(workspace.path(), name, BROKEN);
    }

    let report = scan_workspace(workspace.path());

    let mut paths: Vec<String> = Vec::new();
    for finding in &report.findings {
        paths.push(finding.path.clone());
        assert_eq!(finding.kind.as_str(), "unsupported-manifest-format");
    }
    let expected: Vec<String> = names.iter().map(|name| { return name.to_string(); }).collect();
    assert_eq!(paths, expected);
    assert_eq!(report.manifests_checked, 0);
}

#[test]
fn an_unparseable_package_json_is_a_finding() {
    let workspace = scratch();
    write(workspace.path(), "package.json", BROKEN);

    let report = scan_workspace(workspace.path());

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].kind.as_str(), "invalid-manifest");
}

#[test]
fn node_modules_git_and_gitignored_directories_are_never_entered() {
    let workspace = scratch();
    write(workspace.path(), ".gitignore", "ignored/\n");
    write(workspace.path(), "package.json", CLEAN);
    write(workspace.path(), "node_modules/dep/package.json", LITERAL);
    write(workspace.path(), ".git/inner/package.json", LITERAL);
    write(workspace.path(), "ignored/package.json", LITERAL);

    assert_eq!(finding_paths(workspace.path()), Vec::<String>::new());
}

#[test]
fn findings_come_in_path_order_whatever_order_the_files_were_made_in() {
    let workspace = scratch();
    let in_creation_order: [&str; 3] = ["c/package.json", "a/package.json", "b/package.json"];
    for relative in in_creation_order {
        write(workspace.path(), relative, LITERAL);
    }
    let mut expected: Vec<String> = in_creation_order.iter().map(|path| { return path.to_string(); }).collect();
    expected.sort();

    assert_eq!(finding_paths(workspace.path()), expected);
}

#[test]
fn an_unreadable_package_json_becomes_a_note_and_not_a_failure() {
    let workspace = scratch();
    write(workspace.path(), "locked/package.json", CLEAN);
    let locked: PathBuf = workspace.path().join("locked/package.json");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).expect("remove all access");
    // A user who can read it anyway (root) cannot run this case; the permission is what the test needs.
    if std::fs::read(&locked).is_ok() {
        return;
    }

    let report = scan_workspace(workspace.path());

    assert_eq!(report.manifests_checked, 0);
    assert_eq!(report.notes.len(), 1);
    assert!(report.notes[0].contains("locked/package.json"));
}
