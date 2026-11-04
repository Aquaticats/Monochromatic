//! What:
//!  Controls of module path resolution.
//! Why:
//!  The folder a path points at is the folder of the deepest module it names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a path stops at the first item', () => expect(resolvePath(tree, from, ['crate', 'a', 'Item'], true)).toBe(a));
//! ```

use super::resolve_path;
use crate::{
    layout_module_tree::ModuleTree,
    layout_test_support::{module_in, package_with, tree_of},
};

/// Split a written path into segments.
fn segments(path: &str) -> Vec<String> {
    return path.split("::").map(String::from).collect::<Vec<String>>();
}

/// A fixture with nested modules, a plain-name neighbour and an inline module.
fn nested() -> (crate::test_fs::Fixture, ModuleTree) {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod top { mod inner {} }\n"),
        ("src/a/mod.rs", "pub mod b;\nmod c;\n"),
        ("src/a/b.rs", ""),
        ("src/a/c.rs", ""),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    return (fixture, tree);
}

/// A crate path reaches the deepest module and stops at the first name that is an item.
#[test]
fn a_crate_path_stops_at_the_first_item() {
    let (fixture, tree) = nested();
    let from: usize = module_in(&tree, &fixture.path, "src/lib.rs");
    let b: usize = module_in(&tree, &fixture.path, "src/a/b.rs");
    let a: usize = module_in(&tree, &fixture.path, "src/a/mod.rs");
    assert_eq!(resolve_path(&tree, from, &segments("crate::a::b::Item::Variant"), true), Some(b));
    assert_eq!(resolve_path(&tree, from, &segments("crate::a::Thing::b"), true), Some(a));
    assert_eq!(resolve_path(&tree, from, &segments("crate::Item"), true), Some(from));
}

/// `super` climbs, also in a chain, `self` stays,
/// and climbing past the root names nothing.
#[test]
fn super_and_self_navigate_the_tree() {
    let (fixture, tree) = nested();
    let lib: usize = module_in(&tree, &fixture.path, "src/lib.rs");
    let a: usize = module_in(&tree, &fixture.path, "src/a/mod.rs");
    let b: usize = module_in(&tree, &fixture.path, "src/a/b.rs");
    let c: usize = module_in(&tree, &fixture.path, "src/a/c.rs");
    assert_eq!(resolve_path(&tree, b, &segments("super::c::run"), true), Some(c));
    assert_eq!(resolve_path(&tree, b, &segments("super::super::a::b"), true), Some(b));
    assert_eq!(resolve_path(&tree, b, &segments("super::super::Item"), true), Some(lib));
    assert_eq!(resolve_path(&tree, b, &segments("self::Item"), true), Some(b));
    assert_eq!(resolve_path(&tree, a, &segments("self::b::f"), true), Some(b));
    assert_eq!(resolve_path(&tree, lib, &segments("super::x"), true), None);
}

/// `super` inside an inline module names the module around it.
#[test]
fn super_inside_an_inline_module_names_its_host() {
    let (fixture, tree) = nested();
    let lib: usize = module_in(&tree, &fixture.path, "src/lib.rs");
    let top: usize = *tree.modules[lib].children.get("top").expect("inline module");
    let inner: usize = *tree.modules[top].children.get("inner").expect("nested inline module");
    assert_eq!(resolve_path(&tree, inner, &segments("super::Item"), true), Some(top));
    assert_eq!(resolve_path(&tree, inner, &segments("super::super::Item"), true), Some(lib));
    assert_eq!(resolve_path(&tree, inner, &segments("crate::top::inner::f"), true), Some(inner));
}

/// A leading plain name counts only when it is a module declared beside the code.
#[test]
fn a_plain_name_counts_only_for_a_declared_module() {
    let (fixture, tree) = nested();
    let a: usize = module_in(&tree, &fixture.path, "src/a/mod.rs");
    let b: usize = module_in(&tree, &fixture.path, "src/a/b.rs");
    assert_eq!(resolve_path(&tree, a, &segments("b::run"), true), Some(b));
    assert_eq!(resolve_path(&tree, a, &segments("std::mem::swap"), true), None);
    assert_eq!(resolve_path(&tree, a, &segments("Vec"), true), None);
}

/// A module name is found through a `pub use folder::*` of the crate root,
/// but not when globs are not followed, and a private glob does not re-export.
#[test]
fn glob_reexports_make_module_names_visible() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod native;\npub use native::*;\nmod hidden;\nuse hidden::*;\n"),
        ("src/native/mod.rs", "pub mod saving;\n"),
        ("src/native/saving.rs", ""),
        ("src/hidden/mod.rs", "pub mod secret;\n"),
        ("src/hidden/secret.rs", ""),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    let lib: usize = module_in(&tree, &fixture.path, "src/lib.rs");
    let saving: usize = module_in(&tree, &fixture.path, "src/native/saving.rs");
    assert_eq!(resolve_path(&tree, lib, &segments("crate::saving::run"), true), Some(saving));
    assert_eq!(resolve_path(&tree, lib, &segments("crate::saving::run"), false), Some(lib));
    assert_eq!(resolve_path(&tree, lib, &segments("crate::secret::run"), true), Some(lib));
}

/// Globs that re-export each other end the search.
#[test]
fn a_ring_of_globs_ends() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a.rs", "pub use super::b::*;\n"),
        ("src/b.rs", "pub use super::a::*;\n"),
    ]);
    let tree: ModuleTree = tree_of(&fixture.path);
    let a: usize = module_in(&tree, &fixture.path, "src/a.rs");
    assert_eq!(resolve_path(&tree, a, &segments("self::unknown::run"), true), Some(a));
}
