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
    let found: Vec<LayoutFinding> = check_folder_links(&tree, uses.as_slice(), root, None);
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

/// A folder that holds child folders is a group,
/// so its own code is a finding,
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

/// Glob re-exports of two modules on one line are one finding,
///  and a glob re-export of the module itself is none.
#[test]
fn layout_two_globs_on_one_line_are_one_finding_and_a_self_glob_is_none() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "mod native;\nmod other;\npub use native::*; pub use other::*;\n",
        ),
        ("src/native/mod.rs", "pub use self::*;\n"),
        ("src/other/mod.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let globs: Vec<&String> = found
        .iter()
        .filter(|item| return item.contains("re-exports every item"))
        .collect::<Vec<&String>>();
    assert_eq!(globs.len(), 1, "{found:#?}");
    assert!(
        globs[0].starts_with("src/lib.rs:3 `pub use native::*`"),
        "{}",
        globs[0]
    );
}

/// Run the rule with an allow list and render each finding as `file:line message`,
///  with the fixture folder removed.
fn findings_with(root: &Path, lines: &str) -> Vec<String> {
    let tree = tree_of(root);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    let uses = collect_uses(&tree);
    let list = crate::layout_allow_list::parse_allow_list(lines, Path::new("allow-list.txt"))
        .expect("the fixture list is valid");
    let prefix: String = format!("{}/", root.display());
    return check_folder_links(&tree, uses.as_slice(), root, Some(&list))
        .iter()
        .map(|item| {
            return format!("{}:{} {}", item.file.display(), item.line, item.message)
                .replace(prefix.as_str(), "");
        })
        .collect::<Vec<String>>();
}

/// A package whose folders `a`, `b` and `c` use each other in one loop,
///  with the links the rule asks for.
#[cfg(unix)]
fn a_loop_of_three() -> crate::test_fs::Fixture {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\nmod c;\n"),
        ("src/a/mod.rs", "use crate::b::X;\n"),
        ("src/b/mod.rs", "use crate::c::X;\n"),
        ("src/c/mod.rs", "use crate::a::X;\n"),
    ]);
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    crate::layout_test_support::link(&fixture.path, "src/b/uses/c", "../../c");
    crate::layout_test_support::link(&fixture.path, "src/c/uses/a", "../../a");
    return fixture;
}

/// With every use of the loop on the list the loop is not reported.
#[cfg(unix)]
#[test]
fn listed_uses_take_their_loop_out_of_the_report() {
    let fixture = a_loop_of_three();
    assert_eq!(findings_with(&fixture.path, "").len(), 1);
    let listed: &str = "uses src/a -> src/b\nuses src/b -> src/c\nuses src/c -> src/a\n";
    assert_eq!(findings_with(&fixture.path, listed), Vec::<String>::new());
}

/// A use that joins two folders already inside one listed loop is a finding,
/// because a tolerated loop may only lose uses,
/// and an entry that records a new use would make the list grow.
#[cfg(unix)]
#[test]
fn a_new_use_inside_a_listed_loop_is_a_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\nmod c;\nmod d;\n"),
        ("src/a/mod.rs", "use crate::b::X;\n"),
        ("src/b/mod.rs", "use crate::c::X;\nuse crate::d::Y;\n"),
        ("src/c/mod.rs", "use crate::a::X;\n"),
        ("src/d/mod.rs", "use crate::a::X;\n"),
    ]);
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    crate::layout_test_support::link(&fixture.path, "src/b/uses/c", "../../c");
    crate::layout_test_support::link(&fixture.path, "src/b/uses/d", "../../d");
    crate::layout_test_support::link(&fixture.path, "src/c/uses/a", "../../a");
    crate::layout_test_support::link(&fixture.path, "src/d/uses/a", "../../a");
    let listed: &str = "uses src/a -> src/b\nuses src/b -> src/c\nuses src/c -> src/a\n";
    let found: Vec<String> = findings_with(&fixture.path, listed);
    assert_eq!(found.len(), 2, "{found:#?}");
    for text in &found {
        assert!(
            text.contains("and both sit inside one loop the allow list tolerates today"),
            "{text}"
        );
    }
    assert!(
        found.iter().any(|text| return text.contains("src/b uses the folder src/d")),
        "{found:#?}"
    );
    assert!(
        found.iter().any(|text| return text.contains("src/d uses the folder src/a")),
        "{found:#?}"
    );
}

/// A listed use that no code makes any more is a finding on its own line.
#[test]
fn a_listed_use_that_no_code_makes_is_reported() {
    let fixture = package_with(&a_uses_b());
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    let found: Vec<String> = findings_with(&fixture.path, "uses src/a -> src/z\n");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("allow-list.txt:1 The allow-list entry `uses src/a -> src/z`"),
        "{}",
        found[0]
    );
    assert!(found[0].contains("tolerates nothing any more"), "{}", found[0]);
}

/// A listed use whose two folders are no longer in one loop is a finding,
/// because the entry no longer holds a loop out of the report.
#[test]
fn a_listed_use_that_holds_no_loop_is_reported() {
    let fixture = package_with(&a_uses_b());
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    let found: Vec<String> = findings_with(&fixture.path, "uses src/a -> src/b\n");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("tolerates nothing any more"), "{}", found[0]);
}

/// A listed key takes exactly the finding it names out of the report,
///  and the second glob re-export of the same module stays a finding,
///  which a fixture with one finding could not show.
#[test]
fn a_listed_key_takes_one_finding_out_of_the_report() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "mod native;\npub use native::*;\npub use native::*;\n",
        ),
        ("src/native/mod.rs", "pub mod saving;\n"),
        ("src/native/saving.rs", ""),
    ]);
    // The two glob re-exports and the use of the globbed folder.
    assert_eq!(findings_with(&fixture.path, "").len(), 3);
    let listed: &str = "missing-link src -> src/native\nglob src/lib.rs native\n";
    let found: Vec<String> = findings_with(&fixture.path, listed);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("re-exports every item"), "{}", found[0]);
    // The second of the two, on its own line.
    assert!(found[0].starts_with("src/lib.rs:3"), "{}", found[0]);
}

/// A listed key that names no finding is a finding of its own.
#[test]
fn a_listed_key_that_names_no_finding_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod native;\npub use native::*;\n"),
        ("src/native/mod.rs", "pub mod saving;\n"),
        ("src/native/saving.rs", ""),
    ]);
    let found: Vec<String> = findings_with(&fixture.path, "glob src/lib.rs elsewhere\n");
    assert_eq!(found.len(), 3, "{found:#?}");
    assert!(
        found
            .iter()
            .any(|text| return text.contains("tolerates nothing any more")),
        "{found:#?}"
    );
}

/// A new use that closes a loop of its own is reported once,
///  by the loop finding,
///  and never once per old use the new loop happens to join.
#[cfg(unix)]
#[test]
fn a_new_mutual_use_inside_a_tolerated_loop_is_one_loop_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\nmod c;\n"),
        ("src/a/mod.rs", "use crate::b::X;\nuse crate::c::Y;\n"),
        ("src/b/mod.rs", "use crate::a::X;\n"),
        ("src/c/mod.rs", "use crate::a::Z;\n"),
    ]);
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    crate::layout_test_support::link(&fixture.path, "src/a/uses/c", "../../c");
    crate::layout_test_support::link(&fixture.path, "src/b/uses/a", "../../a");
    crate::layout_test_support::link(&fixture.path, "src/c/uses/a", "../../a");
    let listed: &str = "uses src/a -> src/b\nuses src/b -> src/a\n";
    let found: Vec<String> = findings_with(&fixture.path, listed);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("src/a/mod.rs:2 These folders use each other in a loop"),
        "{}",
        found[0]
    );
    assert!(found[0].contains("src/a uses src/c"), "{}", found[0]);
    assert!(found[0].contains("src/c uses src/a"), "{}", found[0]);
}

/// The help for a missing link warns what a link into a loop does to `cargo package`,
///  because a looped crate that only reads `Create the link:` writes one into a hang.
#[test]
fn the_missing_link_help_warns_about_a_looped_crate() {
    let fixture = package_with(&a_uses_b());
    let tree = tree_of(&fixture.path);
    let uses = collect_uses(&tree);
    let found: Vec<LayoutFinding> = check_folder_links(&tree, uses.as_slice(), &fixture.path, None);
    let help: &str = found[0].help.as_deref().expect("a missing link has help");
    assert!(help.starts_with("Create the link: "), "{help}");
    assert!(help.contains("`cargo package` follows a link into a loop"), "{help}");
    assert!(help.contains("untangle that loop first"), "{help}");
    assert!(help.contains("exclude"), "{help}");
}
