//! What:
//!  Controls of module tree building.
//! Why:
//!  A wrong tree puts a module in the wrong folder and every later answer with it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('mod lines load plain and nested files', () => expect(files(tree)).toEqual([...]));
//! ```

use crate::{
    layout_module_tree::ModuleTree,
    layout_test_support::{module_in, package_with, tree_of},
};
use std::path::{Path, PathBuf};

/// The files of every module in the tree,
/// relative to the fixture and sorted.
fn files_of(tree: &ModuleTree, root: &Path) -> Vec<String> {
    let mut files: Vec<String> = tree
        .modules
        .iter()
        .map(|module| {
            return module
                .file
                .strip_prefix(root)
                .expect("inside the fixture")
                .to_string_lossy()
                .into_owned();
        })
        .collect::<Vec<String>>();
    files.sort();
    return files;
}

/// `mod x;` loads `x.rs` or `x/mod.rs`,
/// and a plain file's own modules sit in a folder of its name.
#[test]
fn mod_lines_load_plain_and_nested_files() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a.rs", "mod c;\n"),
        ("src/a/c.rs", ""),
        ("src/b/mod.rs", "mod d;\n"),
        ("src/b/d.rs", ""),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert_eq!(
        files_of(&tree, &fixture.path),
        [
            "src/a.rs",
            "src/a/c.rs",
            "src/b/d.rs",
            "src/b/mod.rs",
            "src/lib.rs"
        ]
    );
}

/// `#[path]` with `..` moves a module to another folder,
/// and the moved file loads its own modules beside itself.
#[test]
fn a_path_attribute_with_parent_components_moves_the_module() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "#[path = \"../shared/moved.rs\"]\nmod moved;\n",
        ),
        ("shared/moved.rs", "mod sibling;\n"),
        ("shared/sibling.rs", ""),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert_eq!(
        files_of(&tree, &fixture.path),
        ["shared/moved.rs", "shared/sibling.rs", "src/lib.rs"]
    );
    let sibling: usize = module_in(&tree, &fixture.path, "shared/sibling.rs");
    assert_eq!(
        tree.modules[sibling]
            .parent
            .map(|parent| return tree.modules[parent].file.clone()),
        Some(fixture.path.join("shared/moved.rs"))
    );
}

/// An inline module is its own node in its host's file,
/// and files it loads sit in a folder named after it.
#[test]
fn inline_modules_extend_the_folder_chain() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "mod inline { #[path = \"other.rs\"] mod inner; mod plain; mod nested { mod deep; } }\n",
        ),
        ("src/inline/other.rs", ""),
        ("src/inline/plain.rs", ""),
        ("src/inline/nested/deep.rs", ""),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    let inline_nodes: usize = tree
        .modules
        .iter()
        .filter(|module| return module.file == fixture.path.join("src/lib.rs"))
        .count();
    // The root and the two inline modules, all in lib.rs.
    assert_eq!(inline_nodes, 3);
    assert!(files_of(&tree, &fixture.path).contains(&String::from("src/inline/nested/deep.rs")));
    assert!(files_of(&tree, &fixture.path).contains(&String::from("src/inline/other.rs")));
}

/// A path attribute on an inline module renames its folder.
#[test]
fn a_path_attribute_on_an_inline_module_renames_its_folder() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "#[path = \"thread_files\"]\nmod thread { #[path = \"tls.rs\"] mod local_data; }\n",
        ),
        ("src/thread_files/tls.rs", ""),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert!(files_of(&tree, &fixture.path).contains(&String::from("src/thread_files/tls.rs")));
}

/// Test-only modules are not read,
/// even when their files do not exist.
#[test]
fn test_only_modules_are_never_read() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "#[cfg(test)]\n#[path = \"missing_tests.rs\"]\nmod tests;\nmod named_tests;\nmod tests;\nmod real;\n",
        ),
        ("src/real.rs", ""),
        ("src/named_tests.rs", "this is not even Rust ::: {"),
        ("src/tests.rs", "mod unread;"),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert_eq!(
        files_of(&tree, &fixture.path),
        ["src/lib.rs", "src/real.rs"]
    );
}

/// A raw identifier names a file without its prefix.
#[test]
fn raw_identifier_modules_load_the_plain_file_name() {
    let fixture = package_with(&[("src/lib.rs", "mod r#type;\n"), ("src/type.rs", "")]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert_eq!(
        files_of(&tree, &fixture.path),
        ["src/lib.rs", "src/type.rs"]
    );
}

/// A module with no file,
/// a module with both files and a module that loads its own container are problems,
/// each naming the file and line of the `mod` line.
#[test]
fn modules_that_cannot_be_followed_are_problems() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "\nmod missing;\nmod both;\n#[path = \"lib.rs\"]\nmod again;\n",
        ),
        ("src/both.rs", ""),
        ("src/both/mod.rs", ""),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    let lines: Vec<usize> = tree
        .problems
        .iter()
        .map(|problem| return problem.line)
        .collect::<Vec<usize>>();
    // The third `mod` item starts at its attribute, on line four.
    assert_eq!(lines, [2, 3, 4]);
    assert!(
        tree.problems[0].message.contains("looked for"),
        "{}",
        tree.problems[0].message
    );
    assert!(
        tree.problems[1].message.contains("both"),
        "{}",
        tree.problems[1].message
    );
    assert!(
        tree.problems[2].message.contains("already contains"),
        "{}",
        tree.problems[2].message
    );
    assert!(
        tree.problems
            .iter()
            .all(|problem| return problem.file == fixture.path.join("src/lib.rs"))
    );
}

/// A library and an executable are two trees in one list.
#[test]
fn each_root_starts_its_own_tree() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        ("src/main.rs", "mod a;\n"),
        ("src/a.rs", ""),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert_eq!(tree.roots.len(), 2);
    let a_nodes: Vec<PathBuf> = tree
        .modules
        .iter()
        .filter(|module| return module.file == fixture.path.join("src/a.rs"))
        .map(|module| return module.file.clone())
        .collect::<Vec<PathBuf>>();
    assert_eq!(a_nodes.len(), 2);
}

/// Folders of links are never walked:
/// a link to a folder that holds modules adds no module.
#[cfg(unix)]
#[test]
fn links_in_uses_folders_add_no_modules() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod top;\nmod middle;\n"),
        ("src/top/mod.rs", ""),
        ("src/middle/mod.rs", ""),
    ]);
    crate::layout_test_support::link(&fixture.path, "src/top/uses/middle", "../../middle");
    crate::layout_test_support::link(&fixture.path, "src/middle/uses/top", "../../top");
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert_eq!(
        files_of(&tree, &fixture.path),
        ["src/lib.rs", "src/middle/mod.rs", "src/top/mod.rs"]
    );
}
