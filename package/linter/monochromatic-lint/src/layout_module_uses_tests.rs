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
use crate::layout_test_support::{module_in, package_with, tree_of};

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
