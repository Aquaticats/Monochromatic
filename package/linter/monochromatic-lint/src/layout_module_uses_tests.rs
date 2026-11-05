//! What:
//!  Controls of use collection.
//! Why:
//!  Each use must name the right modules and a line that proves it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a use names both modules and its line', () => expect(collectUses(tree)[0].line).toBe(2));
//! ```

use super::{ModuleUse, collect_uses, folder_of};
use crate::{
    layout_module_tree::ModuleTree,
    layout_test_support::{module_in, package_with, tree_of},
    test_fs::Fixture,
};

/// Where the use that starts in a file and has the given path as written ends up,
///  as a path relative to the fixture.
fn target_of(fixture: &Fixture, tree: &ModuleTree, from_file: &str, text: &str) -> String {
    let from: usize = module_in(tree, &fixture.path, from_file);
    let found: ModuleUse = collect_uses(tree)
        .into_iter()
        .find(|item| return item.from == from && item.text == text)
        .expect("the use exists");
    let file = &tree.modules[found.to].file;
    return file
        .strip_prefix(&fixture.path)
        .expect("inside the fixture")
        .display()
        .to_string();
}

/// A use records the modules,
/// the file,
/// the line and the path as written,
/// and a path to the module itself is no use.
#[test]
fn a_use_names_both_modules_the_line_and_the_path() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        (
            "src/a.rs",
            "fn f() {}\nuse crate::b::Thing;\nuse self::f as g;\n",
        ),
        ("src/b.rs", "pub struct Thing;\npub use super::a::*;\n"),
    ]);
    let tree = tree_of(&fixture.path);
    let uses: Vec<ModuleUse> = collect_uses(&tree);
    let a: usize = module_in(&tree, &fixture.path, "src/a.rs");
    let b: usize = module_in(&tree, &fixture.path, "src/b.rs");
    assert_eq!(uses.len(), 2);
    let from_a: &ModuleUse = uses
        .iter()
        .find(|found| return found.from == a)
        .expect("a uses b");
    let from_b: &ModuleUse = uses
        .iter()
        .find(|found| return found.from == b)
        .expect("b uses a");
    assert_eq!((from_a.to, from_a.line), (b, 2));
    assert_eq!(from_a.text, "crate::b::Thing");
    assert_eq!(from_a.file, fixture.path.join("src/a.rs"));
    assert_eq!((from_b.to, from_b.line), (a, 2));
    assert_eq!(from_b.text, "super::a::*");
}

/// A module belongs to the folder of its file.
#[test]
fn a_module_belongs_to_the_folder_of_its_file() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod inline { }\n"),
        ("src/a/mod.rs", "mod b;\n"),
        ("src/a/b.rs", ""),
    ]);
    let tree = tree_of(&fixture.path);
    let b: usize = module_in(&tree, &fixture.path, "src/a/b.rs");
    let a: usize = module_in(&tree, &fixture.path, "src/a/mod.rs");
    assert_eq!(folder_of(&tree, b), fixture.path.join("src/a"));
    assert_eq!(folder_of(&tree, a), fixture.path.join("src/a"));
    let inline: usize = *tree.modules[tree.roots[0]]
        .children
        .get("inline")
        .expect("inline");
    assert_eq!(folder_of(&tree, inline), fixture.path.join("src"));
}

/// A use that reaches a name through the folder's index belongs to the file the index re-exports it from.
#[test]
fn a_use_through_a_reexport_names_the_file_that_defines_it() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        ("src/a/mod.rs", "mod one;\nmod two;\npub use two::Y;\n"),
        ("src/a/one.rs", "use super::Y;\nuse crate::a::Y as Again;\n"),
        ("src/a/two.rs", "pub struct Y;\n"),
    ]);
    let tree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    assert_eq!(
        target_of(&fixture, &tree, "src/a/one.rs", "super::Y"),
        "src/a/two.rs"
    );
    assert_eq!(
        target_of(&fixture, &tree, "src/a/one.rs", "crate::a::Y"),
        "src/a/two.rs"
    );
    // The index itself names the defining file as well.
    assert_eq!(
        target_of(&fixture, &tree, "src/a/mod.rs", "two::Y"),
        "src/a/two.rs"
    );
}

/// A re-export under a new name is found by the new name only.
#[test]
fn a_renamed_reexport_is_found_by_its_new_name() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "mod one;\nmod two;\npub use two::Thing as Y;\n",
        ),
        ("src/a/one.rs", "use super::Y;\nuse super::Thing;\n"),
        ("src/a/two.rs", "pub struct Thing;\n"),
    ]);
    let tree = tree_of(&fixture.path);
    assert_eq!(
        target_of(&fixture, &tree, "src/a/one.rs", "super::Y"),
        "src/a/two.rs"
    );
    // `Thing` is not a name the index exports, so the use stays on the index.
    assert_eq!(
        target_of(&fixture, &tree, "src/a/one.rs", "super::Thing"),
        "src/a/mod.rs"
    );
}

/// A chain of two re-exports ends at the file that defines the name.
#[test]
fn a_chain_of_reexports_ends_at_the_defining_file() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "mod base;\nmod mid;\nmod top;\npub use mid::Y;\n",
        ),
        ("src/a/mid.rs", "pub use super::base::Y;\n"),
        ("src/a/base.rs", "pub struct Y;\n"),
        ("src/a/top.rs", "use super::Y;\n"),
    ]);
    let tree = tree_of(&fixture.path);
    assert_eq!(
        target_of(&fixture, &tree, "src/a/top.rs", "super::Y"),
        "src/a/base.rs"
    );
}

/// A ring of re-exports has no defining file;
/// the walk ends and the use stays on the index.
#[test]
fn a_ring_of_reexports_ends() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        ("src/a/mod.rs", "mod one;\nmod top;\npub use one::Y;\n"),
        ("src/a/one.rs", "pub use super::Y;\n"),
        ("src/a/top.rs", "use super::Y;\n"),
    ]);
    let tree = tree_of(&fixture.path);
    assert_eq!(
        target_of(&fixture, &tree, "src/a/top.rs", "super::Y"),
        "src/a/mod.rs"
    );
}

/// A re-export from a child folder sends the use to the child folder.
#[test]
fn a_reexport_from_a_child_folder_names_the_child_folder() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "mod child;\nmod user;\npub use child::Thing;\n",
        ),
        ("src/a/user.rs", "use super::Thing;\n"),
        ("src/a/child/mod.rs", "mod thing;\n"),
        ("src/a/child/thing.rs", "pub struct Thing;\n"),
    ]);
    let tree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    let child: usize = module_in(&tree, &fixture.path, "src/a/child/mod.rs");
    let from: usize = module_in(&tree, &fixture.path, "src/a/user.rs");
    let found: ModuleUse = collect_uses(&tree)
        .into_iter()
        .find(|item| return item.from == from)
        .expect("the use exists");
    assert_eq!(found.to, child);
    assert_eq!(folder_of(&tree, found.to), fixture.path.join("src/a/child"));
}

/// Once the child's index re-exports the name from its own file,
/// the use goes on to that file.
#[test]
fn a_reexport_from_a_child_folder_goes_on_to_the_childs_own_file() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "mod child;\nmod user;\npub use child::Thing;\n",
        ),
        ("src/a/user.rs", "use super::Thing;\n"),
        ("src/a/child/mod.rs", "mod thing;\npub use thing::Thing;\n"),
        ("src/a/child/thing.rs", "pub struct Thing;\n"),
    ]);
    let tree = tree_of(&fixture.path);
    assert_eq!(
        target_of(&fixture, &tree, "src/a/user.rs", "super::Thing"),
        "src/a/child/thing.rs"
    );
}

/// A glob re-export,
/// a private `use`
/// and a re-export of something outside the package
/// leave the use on the index.
#[test]
fn only_a_named_reexport_of_the_package_is_followed() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "mod one;\nmod two;\nmod three;\npub use two::*;\nuse three::Q;\npub use std::fmt::Display as D;\n",
        ),
        (
            "src/a/one.rs",
            "use super::Y;\nuse super::Q;\nuse super::D;\n",
        ),
        ("src/a/two.rs", "pub struct Y;\n"),
        ("src/a/three.rs", "pub struct Q;\n"),
    ]);
    let tree = tree_of(&fixture.path);
    for text in ["super::Y", "super::Q", "super::D"] {
        assert_eq!(
            target_of(&fixture, &tree, "src/a/one.rs", text),
            "src/a/mod.rs",
            "{text}"
        );
    }
}
