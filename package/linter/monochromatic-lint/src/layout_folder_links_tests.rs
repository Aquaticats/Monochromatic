//! What:
//!  Controls of the folder rule.
//! Why:
//!  Every finding of the rule,
//! and the inputs that must not make one,
//! are shown on small packages.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a used folder without a link is reported', () => expect(check(tree)).toEqual([...]));
//! ```

use super::check_folder_links;
use crate::{
    layout_finding::LayoutFinding,
    layout_module_uses::collect_uses,
    layout_test_support::{package_with, tree_of},
};
use std::path::Path;

/// Run the rule on a fixture package and render each finding as `file:line message`,
/// with the fixture folder removed.
fn findings_of(root: &Path) -> Vec<String> {
    let tree = tree_of(root);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    let uses = collect_uses(&tree);
    let found: Vec<LayoutFinding> = check_folder_links(&tree, uses.as_slice(), root);
    let prefix: String = format!("{}/", root.display());
    return found
        .iter()
        .map(|item| {
            return format!("{}:{} {}", item.file.display(), item.line, item.message)
                .replace(prefix.as_str(), "");
        })
        .collect::<Vec<String>>();
}

/// Packages where a folder `a` uses a folder `b`.
fn a_uses_b() -> Vec<(&'static str, &'static str)> {
    return vec![
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a/mod.rs", "use crate::b::Thing;\n"),
        ("src/b/mod.rs", "pub struct Thing;\n"),
    ];
}

/// A package whose code uses no other folder needs no links and has no finding.
#[test]
fn folders_that_use_nothing_have_no_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a/mod.rs", ""),
        ("src/b/mod.rs", ""),
    ]);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// A used folder with no link is reported on the proving reference,
/// naming both folders.
#[test]
fn a_used_folder_without_a_link_is_reported_with_its_proof() {
    let fixture = package_with(&a_uses_b());
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found.iter().any(|text| return text.starts_with("src/a/mod.rs:1 src/a uses the folder src/b (`crate::b::Thing`) but src/a/uses has no link to it.")),
        "{found:#?}"
    );
}

/// With the link in place the package is clean (the root folder declares `a` and `b` but names neither by path).
#[cfg(unix)]
#[test]
fn a_matching_link_satisfies_the_rule() {
    let fixture = package_with(&a_uses_b());
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// A link to a folder the code does not use is reported on the link.
#[cfg(unix)]
#[test]
fn a_link_to_an_unused_folder_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a/mod.rs", ""),
        ("src/b/mod.rs", ""),
    ]);
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    assert_eq!(
        findings_of(&fixture.path),
        ["src/a/uses/b:0 src/a/uses/b links to the folder src/b, but no code in src/a uses it."]
    );
}

/// Folders that use each other,
/// directly or through a third,
/// are one finding naming every use.
#[cfg(unix)]
#[test]
fn folders_that_use_each_other_are_one_loop_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\nmod c;\n"),
        ("src/a/mod.rs", "use crate::b::X;\n"),
        ("src/b/mod.rs", "use crate::c::X;\n"),
        ("src/c/mod.rs", "use crate::a::X;\n"),
    ]);
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    crate::layout_test_support::link(&fixture.path, "src/b/uses/c", "../../c");
    crate::layout_test_support::link(&fixture.path, "src/c/uses/a", "../../a");
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    for part in [
        "src/a uses src/b (src/a/mod.rs:1 `crate::b::X`)",
        "src/b uses src/c (src/b/mod.rs:1 `crate::c::X`)",
        "src/c uses src/a (src/c/mod.rs:1 `crate::a::X`)",
    ] {
        assert!(found[0].contains(part), "{part} missing in {}", found[0]);
    }
    assert!(found[0].starts_with("src/a/mod.rs:1 These folders use each other in a loop"));
}

/// A folder that holds child folders is a group, so its own code is a finding,
/// and a child that reaches into it names the group itself,
/// which a child never does (the group rules are in `layout_group_folders_tests.rs`).
#[test]
fn a_parent_folder_with_code_is_a_group_that_holds_none() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "mod b;\npub fn shared() {}\nfn go() { self::b::run(); }\n",
        ),
        ("src/a/b/mod.rs", "fn run() { super::shared(); }\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let text: String = found.join("\n");
    for part in [
        "src/a/mod.rs is the index of the group folder src/a, so it may hold only its top doc and `mod` lines, but it holds a function `shared`",
        "src/a/mod.rs is the index of the group folder src/a, so it may hold only its top doc and `mod` lines, but it holds a function `go`",
        "src/a/b uses the group folder src/a itself (`super::shared`)",
    ] {
        assert!(text.contains(part), "{part} missing in {text}");
    }
}

/// Paths in strings,
/// comments,
/// character literals and test-only code are not uses;
/// paths in macro calls are.
#[test]
fn only_code_outside_tests_makes_a_use() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\nmod c;\n"),
        (
            "src/a/mod.rs",
            "// crate::b::X\nconst S: &str = \"crate::b::X\";\nconst C: char = '\"';\nconst T: &str = r#\"crate::b::X\"#;\n#[cfg(test)]\n#[path = \"mod_tests.rs\"]\nmod tests;\n",
        ),
        ("src/a/mod_tests.rs", "use crate::b::X;\n"),
        ("src/b/mod.rs", ""),
        (
            "src/c/mod.rs",
            "fn f() { println!(\"{}\", crate::b::X); }\n",
        ),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("src/c/mod.rs:1 src/c uses the folder src/b (`crate::b::X`)"),
        "{}",
        found[0]
    );
}

/// A `#[path]` with `..` puts a module in the folder where its file is.
#[test]
fn the_folder_of_a_moved_module_is_the_folder_of_its_file() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "mod a;\n#[path = \"../shared/x.rs\"]\nmod x;\n",
        ),
        ("src/a/mod.rs", "use crate::x::Thing;\n"),
        ("shared/x.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found
            .iter()
            .any(|text| return text.contains("src/a uses the folder shared (`crate::x::Thing`)")),
        "{found:#?}"
    );
}

/// A path through `super::super::` and one through an inline module reach the folders their modules are in.
#[test]
fn super_chains_and_inline_modules_reach_the_right_folders() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a/mod.rs", "mod deep;\n"),
        (
            "src/a/deep/mod.rs",
            "mod inline { fn f() { super::super::super::b::go(); } }\n",
        ),
        ("src/b/mod.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found.iter().any(|text| return text
            .contains("src/a/deep uses the folder src/b (`super::super::super::b::go`)")),
        "{found:#?}"
    );
}

/// A glob re-export of a module is a finding of its own,
/// next to the use of the child folder that its bare path makes.
#[test]
fn a_glob_reexport_is_a_finding_of_its_own() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod native;\npub use native::*;\n"),
        ("src/native/mod.rs", "pub mod saving;\n"),
        ("src/native/saving.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(
        found[0].contains("src uses the folder src/native (`native::*`)"),
        "{}",
        found[0]
    );
    assert!(
        found[1].starts_with(
            "src/lib.rs:2 `pub use native::*` re-exports every item of the module in src/native"
        ),
        "{}",
        found[1]
    );
}

/// A module file below a folder named `uses` is a finding.
#[test]
fn a_module_below_a_uses_folder_is_a_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "#[path = \"uses/x.rs\"]\nmod x;\n"),
        ("src/uses/x.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found
            .iter()
            .any(|text| return text.contains("is below a folder named uses")),
        "{found:#?}"
    );
}

/// A link that points outside the package is reported,
/// and so is a missing link for a used folder.
#[cfg(unix)]
#[test]
fn a_link_outside_the_package_is_reported() {
    let fixture = package_with(&a_uses_b());
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../../elsewhere");
    std::fs::create_dir_all(fixture.path.join("elsewhere")).expect("elsewhere");
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found
            .iter()
            .any(|text| return text.contains("is not a source folder of this package")),
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|text| return text.contains("has no link to it")),
        "{found:#?}"
    );
}

/// A folder that reaches an item through another folder's index uses that folder and the folder that defines the item,
/// which is the child folder when the index re-exports it from there;
/// an index that holds child folders is a group,
/// so the re-export is a finding of its own and the use of the group itself is one too.
#[test]
fn a_use_through_a_reexport_needs_the_link_to_the_defining_folder_too() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a/mod.rs", "use crate::b::Thing;\n"),
        ("src/b/mod.rs", "mod inner;\npub use inner::Thing;\n"),
        ("src/b/inner/mod.rs", "pub struct Thing;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    for part in [
        "src/a uses the group folder src/b itself (`crate::b::Thing`)",
        "src/a uses the folder src/b/inner (`crate::b::Thing`) but src/a/uses has no link to it.",
        "src/b/mod.rs:2 src/b/mod.rs is the index of the group folder src/b, so it may hold only its top doc and `mod` lines, but it holds a `use` line.",
    ] {
        assert!(
            found.iter().any(|text| return text.contains(part)),
            "{part} missing in {found:#?}"
        );
    }
}
