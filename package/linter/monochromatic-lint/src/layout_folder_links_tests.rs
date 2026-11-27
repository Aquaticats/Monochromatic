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

use super::{
    check_folder_links,
    counted_work,
};
use crate::{
    layout_finding::LayoutFinding,
    layout_module_uses::collect_uses,
    layout_test_support::{
        package_with,
        tree_of,
    },
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

/// Glob re-exports of two modules on one line are two findings,
///  each with its own key,
///  so a list line for the first never tolerates the second;
///  a glob re-export of the module itself is none.
#[test]
fn layout_two_globs_on_one_line_are_two_findings_and_a_self_glob_is_none() {
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
    assert_eq!(globs.len(), 2, "{found:#?}");
    // The two findings are the two globs the line holds, in source order, and neither is the self glob.
    for (glob, module) in globs.iter().zip(["native", "other"]) {
        assert!(
            glob.starts_with(
                format!("src/lib.rs:3 `pub use {module}::*` re-exports every item").as_str()
            ),
            "{glob}"
        );
    }
}

/// A use written in one group's own file that another group's own file answers closes a loop,
///  so the report names both uses,
///  one finding for each own-file use in it.
#[test]
fn a_loop_that_two_group_own_file_uses_close_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod g1;\nmod g2;\n"),
        ("src/g1/mod.rs", "pub mod own;\npub mod c1;\n"),
        ("src/g1/own.rs", "use crate::g2::c2::X;\n"),
        ("src/g1/c1/mod.rs", "pub struct Y;\n"),
        ("src/g2/mod.rs", "pub mod own;\npub mod c2;\n"),
        ("src/g2/own.rs", "use crate::g1::c1::Y;\n"),
        ("src/g2/c2/mod.rs", "pub struct X;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let loops: Vec<&String> = found
        .iter()
        .filter(|item| return item.contains("use each other in a loop"))
        .collect::<Vec<&String>>();
    assert_eq!(loops.len(), 2, "{found:#?}");
    for part in [
        "src/g1/own.rs:1 These entries use each other in a loop: src/g1 uses src/g2/c2 (`crate::g2::c2::X`); src/g2 uses src/g1 (`crate::g1::c1::Y` at src/g2/own.rs:1).",
        "src/g2/own.rs:1 These entries use each other in a loop: src/g2 uses src/g1/c1 (`crate::g1::c1::Y`); src/g1 uses src/g2 (`crate::g2::c2::X` at src/g1/own.rs:1).",
    ] {
        assert!(
            loops.iter().any(|text| return text.as_str() == part),
            "{part} missing in {loops:#?}"
        );
    }
}

/// A use of a group's own child folders joins no two entries,
///  so the group's own file makes no loop finding for it.
#[test]
fn a_group_own_file_that_uses_only_its_own_child_folders_makes_no_loop_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod g;\n"),
        ("src/g/mod.rs", "pub mod own;\npub mod c1;\n"),
        ("src/g/own.rs", "use crate::g::c1::X;\n"),
        ("src/g/c1/mod.rs", "pub struct X;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found
            .iter()
            .all(|text| return !text.contains("use each other in a loop")),
        "{found:#?}"
    );
}

/// A folder loop already reports the use a group's own file makes inside it,
///  so that use is reported once, by the loop of folders,
///  and not again by the entries it would join.
#[test]
fn a_group_own_file_use_that_a_folder_loop_holds_is_reported_once() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod grp;\n"),
        ("src/a/mod.rs", "use crate::grp::X;\n"),
        ("src/grp/mod.rs", "pub mod own;\npub mod g2;\n"),
        ("src/grp/own.rs", "use crate::a::X;\n"),
        ("src/grp/g2/mod.rs", "pub struct X;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let loops: Vec<&String> = found
        .iter()
        .filter(|text| return text.contains("use each other in a loop"))
        .collect::<Vec<&String>>();
    assert_eq!(loops.len(), 1, "{found:#?}");
    assert!(
        loops[0].contains("These folders use each other in a loop"),
        "{loops:#?}"
    );
}

/// The walk back starts at the entry the used folder belongs to,
///  so a group whose own file uses a folder of another group gets its loop finding
///  even when the two ends are child folders of the two groups.
#[test]
fn a_group_own_file_use_of_another_groups_child_folder_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod g1;\nmod g2;\n"),
        ("src/g1/mod.rs", "pub mod own;\npub mod c1;\n"),
        ("src/g1/own.rs", "use crate::g2::c2::X;\n"),
        ("src/g1/c1/mod.rs", "pub struct Y;\n"),
        ("src/g2/mod.rs", "pub mod c2;\n"),
        ("src/g2/c2/mod.rs", "use crate::g1::c1::Y;\npub struct X;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found.iter().any(|text| return text
            == "src/g1/own.rs:1 These entries use each other in a loop: src/g1 uses src/g2/c2 (`crate::g2::c2::X`); src/g2 uses src/g1 (`crate::g1::c1::Y` at src/g2/c2/mod.rs:1)."),
        "{found:#?}"
    );
}

/// A glob re-export inside an inline module of a file is a finding of its own,
///  so the rule reads one module node per file and set of reference offsets,
///  never one node per file.
#[test]
fn a_glob_inside_an_inline_module_is_a_finding_of_its_own() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\nmod s;\n"),
        ("src/a/mod.rs", "pub mod one;\n"),
        ("src/a/one.rs", ""),
        ("src/b/mod.rs", "pub mod two;\n"),
        ("src/b/two.rs", ""),
        (
            "src/s.rs",
            "pub use crate::a::*;\npub mod inner { pub use crate::b::*; }\n",
        ),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let globs: Vec<&String> = found
        .iter()
        .filter(|text| return text.contains("re-exports every item"))
        .collect::<Vec<&String>>();
    assert_eq!(globs.len(), 2, "{found:#?}");
    for part in ["src/s.rs:1", "src/s.rs:2"] {
        assert!(
            globs.iter().any(|text| return text.starts_with(part)),
            "{part} missing in {globs:#?}"
        );
    }
}

/// A file that two `mod` lines declare is read in two module contexts,
///  and a glob in it can resolve in one and not in the other,
///  so the rule reports the finding whichever line comes first.
#[test]
fn the_glob_dedupe_reports_the_same_findings_whatever_order_the_mod_lines_come_in() {
    // The only difference between the two packages is the order of the two `mod` lines of the root file.
    fn globs_of(lines: &str) -> Vec<String> {
        let fixture = package_with(&[
            ("src/lib.rs", lines),
            ("src/a/mod.rs", ""),
            ("src/m/mod.rs", "#[path = \"../shared.rs\"] mod shared;\n"),
            // From inside the folder, `super::super` is the crate root, which holds `a`;
            // from the crate root itself there is no module above it.
            ("src/shared.rs", "pub use super::super::a::*;\n"),
        ]);
        return findings_of(&fixture.path)
            .into_iter()
            .filter(|text| return text.contains("re-exports every item"))
            .collect::<Vec<String>>();
    }
    let after: Vec<String> = globs_of("mod a;\nmod m;\nmod shared;\n");
    let before: Vec<String> = globs_of("mod a;\nmod shared;\nmod m;\n");
    assert_eq!(after.len(), 1, "{after:#?}");
    assert_eq!(before.len(), 1, "{before:#?}");
}

/// Two globs of one module inside one brace group are two findings,
///  because both paths start where the group's path starts,
///  so only their order apart keeps them from being one.
#[test]
fn two_globs_of_one_module_in_one_brace_group_are_two_findings() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "mod native;\npub use crate::{native::*, native::*};\n",
        ),
        ("src/native/mod.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let globs: Vec<&String> = found
        .iter()
        .filter(|text| return text.contains("re-exports every item"))
        .collect::<Vec<&String>>();
    assert_eq!(globs.len(), 2, "{found:#?}");
}

/// Run the rule with an allow list and render each finding as `file:line message`,
///  with the fixture folder removed.
fn findings_with(
    root: &Path,
    lines: &str,
) -> Vec<String> {
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

/// A package whose folders `a`,
/// `b` and `c` use each other in one loop,
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
        found
            .iter()
            .any(|text| return text.contains("src/b uses the folder src/d")),
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|text| return text.contains("src/d uses the folder src/a")),
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
    assert!(
        found[0].contains("tolerates nothing any more"),
        "{}",
        found[0]
    );
}

/// A listed use whose two folders are no longer in one loop is a finding,
/// because the entry no longer holds a loop out of the report.
#[test]
fn a_listed_use_that_holds_no_loop_is_reported() {
    let fixture = package_with(&a_uses_b());
    crate::layout_test_support::link(&fixture.path, "src/a/uses/b", "../../b");
    let found: Vec<String> = findings_with(&fixture.path, "uses src/a -> src/b\n");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("tolerates nothing any more"),
        "{}",
        found[0]
    );
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
///  and the warning comes before the command it warns about,
///  because a looped crate that copies `Create the link:` first writes one into a hang.
#[test]
fn the_missing_link_help_warns_about_a_looped_crate() {
    let fixture = package_with(&a_uses_b());
    let tree = tree_of(&fixture.path);
    let uses = collect_uses(&tree);
    let found: Vec<LayoutFinding> = check_folder_links(&tree, uses.as_slice(), &fixture.path, None);
    let help: &str = found[0].help.as_deref().expect("a missing link has help");
    assert!(
        help.starts_with("A crate whose folders still use each other in a loop"),
        "{help}"
    );
    assert!(
        help.contains("`cargo package` follows a link into a loop"),
        "{help}"
    );
    assert!(help.contains("untangle that loop first"), "{help}");
    assert!(help.contains("exclude"), "{help}");
    assert!(
        help.contains("over 3038 paths with 109 links"),
        "the warning names what was measured: {help}"
    );
    let hazard: usize = help
        .find("untangle that loop first")
        .expect("the help warns first");
    let command: usize = help
        .find("Then create the link: ")
        .expect("the help gives the command");
    assert!(
        hazard < command,
        "the warning must come before the command: {help}"
    );
}

/// Write the shape the walk cost was measured on:
///  `plain` folders that each use every later one,
///  and `groups` groups whose own file uses a folder that uses a child of the group,
///  so each of those own-file uses closes one loop.
fn package_of_groups_closing_one_loop(
    plain: usize,
    groups: usize,
) -> crate::test_fs::Fixture {
    // What: the module declarations of the root, which every folder below needs.
    // Why: a folder that no `mod` line reaches is no folder of the tree.
    let mut lib: String = String::new();
    let mut files: Vec<(String, String)> = Vec::<(String, String)>::new();
    for index in 0..plain {
        lib.push_str(format!("mod f{index};\n").as_str());
        // What: one plain folder that uses every later plain folder.
        // Why: those uses make the searched graph large without closing any loop of their own.
        let mut body: String = String::new();
        for later in index + 1..plain {
            body.push_str(format!("use crate::f{later}::F;\n").as_str());
        }
        body.push_str("pub struct F;\n");
        files.push((format!("src/f{index}/mod.rs"), body));
    }
    for index in 0..groups {
        lib.push_str(format!("mod g{index};\nmod x{index};\n").as_str());
        files.push((
            format!("src/g{index}/mod.rs"),
            format!("pub mod c{index};\npub mod own;\n"),
        ));
        files.push((
            format!("src/g{index}/c{index}/mod.rs"),
            String::from("pub struct C;\n"),
        ));
        files.push((
            format!("src/g{index}/own.rs"),
            format!("use crate::x{index}::X;\n"),
        ));
        // What: the folder the group's own file uses, which uses a child of the group back.
        // Why: that use is what closes the own-file loop, so the group has a route to walk.
        files.push((
            format!("src/x{index}/mod.rs"),
            format!("use crate::g{index}::c{index}::C;\npub struct X;\n"),
        ));
    }
    let mut listed: Vec<(&str, &str)> = vec![("src/lib.rs", lib.as_str())];
    for (path, body) in &files {
        listed.push((path.as_str(), body.as_str()));
    }
    return package_with(listed.as_slice());
}

/// Every loop a group's own file closes is found, so the controls below measure a walk that ran.
fn own_file_loops_of(root: &Path) -> Vec<String> {
    return findings_of(root)
        .into_iter()
        .filter(|text| return text.contains("These entries use each other in a loop"))
        .collect::<Vec<String>>();
}

/// The reverse index of the searched graph is built once for the package,
///  and each group whose own file closes a loop is walked back once,
///  however many groups close one:
///  a cost of one graph per group would build the index once per group.
#[test]
fn one_reverse_index_answers_every_group_that_closes_a_loop() {
    let fixture = package_of_groups_closing_one_loop(5, 3);
    counted_work::reset();
    let loops: Vec<String> = own_file_loops_of(&fixture.path);
    assert_eq!(loops.len(), 3, "the control found no loop: {loops:#?}");
    assert_eq!(
        counted_work::reverse_indexes(),
        1,
        "the reverse index is one table for the package, not one per group"
    );
    assert_eq!(
        counted_work::walks(),
        3,
        "one walk per group, not one per use and not one per package"
    );
}

/// A group whose own file uses two folders, each of which uses a different child of the group,
///  closes two loops in one group:
///  both findings are reported, and one walk answers both of them.
#[test]
fn one_walk_answers_two_loops_of_one_group() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod g;\nmod x;\nmod y;\n"),
        ("src/g/mod.rs", "pub mod c0;\npub mod c1;\npub mod own;\n"),
        ("src/g/c0/mod.rs", "pub struct C0;\n"),
        ("src/g/c1/mod.rs", "pub struct C1;\n"),
        // What: one own file whose two uses leave the group towards two different entries.
        // Why: both entries reach the group back by their own child, so this one group has two loops.
        ("src/g/own.rs", "use crate::x::X;\nuse crate::y::Y;\n"),
        ("src/x/mod.rs", "use crate::g::c0::C0;\npub struct X;\n"),
        ("src/y/mod.rs", "use crate::g::c1::C1;\npub struct Y;\n"),
    ]);
    counted_work::reset();
    let loops: Vec<String> = own_file_loops_of(&fixture.path);
    assert_eq!(loops.len(), 2, "{loops:#?}");
    assert!(
        loops.iter().any(|text| return text
            .contains("These entries use each other in a loop: src/g uses src/x ")),
        "{loops:#?}"
    );
    assert!(
        loops.iter().any(|text| return text
            .contains("These entries use each other in a loop: src/g uses src/y ")),
        "{loops:#?}"
    );
    assert_eq!(
        counted_work::walks(),
        1,
        "one walk answers every own-file use of one group"
    );
    assert_eq!(
        counted_work::reverse_indexes(),
        1,
        "one group still reads the graph's reverse index once"
    );
}

/// Two shortest routes of one length back to the group are decided by the path names:
///  each step takes the successor that sorts first,
///  so the message spells the route through `a` and never the one through `b`.
#[test]
fn an_equal_length_tie_spells_the_smallest_route() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "mod a;\nmod b;\nmod c;\nmod d;\nmod g;\nmod x;\n",
        ),
        ("src/g/mod.rs", "pub mod c0;\npub mod own;\n"),
        ("src/g/c0/mod.rs", "pub struct G;\n"),
        ("src/g/own.rs", "use crate::x::X;\n"),
        // What: the used entry, which reaches the group by two routes of three uses each.
        // Why: `a` sorts before `b`, so the smallest route is the one through `a`.
        (
            "src/x/mod.rs",
            "use crate::a::A;\nuse crate::b::B;\npub struct X;\n",
        ),
        ("src/a/mod.rs", "use crate::d::D;\npub struct A;\n"),
        ("src/b/mod.rs", "use crate::c::C;\npub struct B;\n"),
        ("src/c/mod.rs", "use crate::g::c0::G;\npub struct C;\n"),
        ("src/d/mod.rs", "use crate::g::c0::G;\npub struct D;\n"),
    ]);
    let loops: Vec<String> = own_file_loops_of(&fixture.path);
    assert_eq!(loops.len(), 1, "{loops:#?}");
    assert!(
        loops[0].contains("src/x uses src/a"),
        "the tie spells the smallest route: {}",
        loops[0]
    );
    assert!(
        !loops[0].contains("src/x uses src/b"),
        "the tie spells one route only: {}",
        loops[0]
    );
}

/// A `uses` line that climbs out of the package with `..` and comes back in
///  names the folder its key names,
///  so it tolerates the use it spells rather than staying alive and tolerating nothing.
#[cfg(unix)]
#[test]
fn a_listed_path_that_climbs_out_and_back_tolerates_the_use_it_names() {
    let fixture = a_loop_of_three();
    let name: String = fixture
        .path
        .file_name()
        .expect("the fixture folder has a name")
        .to_string_lossy()
        .into_owned();
    let listed: String =
        format!("uses ../{name}/src/a -> src/b\nuses src/b -> src/c\nuses src/c -> src/a\n");
    assert_eq!(
        findings_with(&fixture.path, listed.as_str()),
        Vec::<String>::new(),
        "the alias names the same use as `uses src/a -> src/b`"
    );
    // What: the control, an alias that names a use no code makes.
    // Why: it must still be dead, so the reading above does not make every climbing path alive.
    let dead: Vec<String> = findings_with(
        &fixture.path,
        format!("uses ../{name}/src/a -> src/z\n").as_str(),
    );
    assert!(
        dead.iter()
            .any(|text| return text.contains("tolerates nothing any more")),
        "{dead:#?}"
    );
}

/// The findings of a package with an allow list, as the rule made them,
///  so a control can read a finding's help and not only its message.
fn findings_asked_with(
    root: &Path,
    lines: &str,
) -> Vec<LayoutFinding> {
    let tree = tree_of(root);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    let uses = collect_uses(&tree);
    let list = crate::layout_allow_list::parse_allow_list(lines, Path::new("allow-list.txt"))
        .expect("the fixture list is valid");
    return check_folder_links(&tree, uses.as_slice(), root, Some(&list));
}

/// Write the smallest package whose group's own file closes a loop through `x`,
///  where `x` reaches the group both directly and through `a`.
fn write_two_route_min(fixture: &crate::test_fs::Fixture) {
    crate::run_test_support::write(&fixture.path, "src/lib.rs", "mod a;\nmod g;\nmod x;\n");
    crate::run_test_support::write(&fixture.path, "src/g/mod.rs", "pub mod c0;\npub mod own;\n");
    crate::run_test_support::write(&fixture.path, "src/g/c0/mod.rs", "pub struct G;\n");
    crate::run_test_support::write(&fixture.path, "src/g/own.rs", "use crate::x::X;\n");
    crate::run_test_support::write(&fixture.path, "src/x/mod.rs", "pub mod f;\n");
    crate::run_test_support::write(
        &fixture.path,
        "src/x/f.rs",
        "use crate::a::A;\nuse crate::g::c0::G;\npub struct X;\n",
    );
    crate::run_test_support::write(&fixture.path, "src/a/mod.rs", "pub mod f;\n");
    crate::run_test_support::write(
        &fixture.path,
        "src/a/f.rs",
        "use crate::g::c0::G;\npub struct A;\n",
    );
}

/// A use that joins a loop a group's own file closes says how a list written before the change
///  is migrated: delete that loop's `own-file-loop` lines and write the lines the next run prints.
///  Only this help says so, so a run that lost the sentence would send a reader to edit a list
///  that no round may grow.
#[test]
fn a_use_joining_a_tolerated_own_file_loop_says_how_to_migrate_the_list() {
    let fixture = package_with(&[]);
    write_two_route_min(&fixture);
    // What: the list that holds the loop and nothing of its uses, which is what alpha.15 wrote.
    // Why: that is the state the extra sentence is about, and the state a use added later meets.
    let listed: &str = "group-file src/g own.rs\nown-file-loop src/g -> src/x\n";
    let found: Vec<LayoutFinding> = findings_asked_with(&fixture.path, listed);
    let joining: Vec<&LayoutFinding> = found
        .iter()
        .filter(|item| {
            return item
                .message
                .contains("and both sit inside one loop the allow list tolerates today");
        })
        .collect::<Vec<&LayoutFinding>>();
    assert!(!joining.is_empty(), "{found:#?}");
    for item in &joining {
        let help: &str = item.help.as_deref().expect("the finding has help");
        assert!(
            help.contains("If this list was written before alpha.16"),
            "the help names the migration: {help}"
        );
        assert!(
            help.contains(
                "delete those lines, run once with `--debug`, and write every line it prints"
            ),
            "the help says what to do: {help}"
        );
    }
}

/// Write a package with one group whose own file uses `src/x`,
///  where `src/x` uses a child of the group only when `closing` is set,
///  so the group's own file closes a loop in one shape and none in the other.
fn package_of_one_group_that_may_close(closing: bool) -> crate::test_fs::Fixture {
    // What: the used folder's code, which reaches the group back only in the closing shape.
    // Why: that one use is what decides whether the group has a loop and therefore a walk at all.
    let used: &str = if closing {
        "use crate::g::c0::C;\npub struct X;\n"
    } else {
        "pub struct X;\n"
    };
    return package_with(&[
        ("src/lib.rs", "mod g;\nmod x;\n"),
        ("src/g/mod.rs", "pub mod c0;\npub mod own;\n"),
        ("src/g/c0/mod.rs", "pub struct C;\n"),
        ("src/g/own.rs", "use crate::x::X;\n"),
        ("src/x/mod.rs", used),
    ]);
}

/// Write the shape one directly answered own-file use is measured on:
///  one group whose own file uses `src/p0`,
///  `src/p0` uses a child folder of the group,
///  and each later `src/pN` uses the folder before it,
///  so `plain` folders reach the one the group's own file uses.
fn package_of_a_chain_behind_one_direct_use(plain: usize) -> crate::test_fs::Fixture {
    // What: the module declarations of the root, which every folder below needs.
    // Why: a folder that no `mod` line reaches is no folder of the tree.
    let mut lib: String = String::from("mod g;\n");
    let mut files: Vec<(String, String)> = Vec::<(String, String)>::new();
    files.push((
        String::from("src/g/mod.rs"),
        String::from("pub mod c0;\npub mod own;\n"),
    ));
    files.push((
        String::from("src/g/c0/mod.rs"),
        String::from("pub struct C;\n"),
    ));
    // What: the own file's use of the first folder of the chain, which is the use the finding reports.
    // Why: that folder reaches the group directly, so its route is one use long however long the chain behind it is.
    files.push((
        String::from("src/g/own.rs"),
        String::from("use crate::p0::P;\n"),
    ));
    for index in 0..plain {
        lib.push_str(format!("mod p{index};\n").as_str());
        // What: the use that puts this folder one step further from the group.
        // Why: the first folder uses the group itself, and every later one uses the folder before it.
        let used: String = if index == 0 {
            String::from("use crate::g::c0::C;\n")
        } else {
            format!("use crate::p{}::P;\n", index - 1)
        };
        files.push((
            format!("src/p{index}/mod.rs"),
            format!("{used}pub struct P;\n"),
        ));
    }
    let mut listed: Vec<(&str, &str)> = vec![("src/lib.rs", lib.as_str())];
    for (path, body) in &files {
        listed.push((path.as_str(), body.as_str()));
    }
    return package_with(listed.as_slice());
}

/// A group whose own file uses an entry that reaches the group directly
///  walks that entry and no other,
///  however many entries reach the one it uses:
///  the walk stops once every use of the group has its route.
#[test]
fn a_directly_answered_use_reaches_no_entry_behind_it() {
    // What: how many folders reach the one the group's own file uses, named so the count is no magic number.
    // Why: the graph behind the group is as large as this, so a walk that reads it costs that much for one use.
    const BEHIND: usize = 200;
    // What: the entries one answered use needs, which are the group and the entry the own file uses.
    // Why: that is the whole route the finding spells, so anything more is a walk that read entries no use asked for.
    const ENTRIES_OF_ONE_ANSWERED_USE: usize = 2;
    let fixture = package_of_a_chain_behind_one_direct_use(BEHIND);
    counted_work::reset();
    let loops: Vec<String> = own_file_loops_of(&fixture.path);
    assert_eq!(loops.len(), 1, "{loops:#?}");
    assert!(
        loops[0].contains("src/p0 uses src/g"),
        "the finding spells the one use that leads back: {}",
        loops[0]
    );
    // What: how many entries the walk reached to answer that one use.
    // Why: the use reaches the group directly, so the chain that reaches the used entry is no part of its route.
    let reached: usize = counted_work::reached();
    assert!(
        reached <= ENTRIES_OF_ONE_ANSWERED_USE,
        "one use that reaches the group directly reached {reached} entries over {BEHIND} entries behind it"
    );
}

/// A package whose groups close no loop builds no reverse index and walks nothing,
///  which is what the README says such a package pays.
#[test]
fn a_package_whose_groups_close_no_loop_builds_no_reverse_index() {
    let open: crate::test_fs::Fixture = package_of_one_group_that_may_close(false);
    counted_work::reset();
    assert_eq!(own_file_loops_of(&open.path).len(), 0);
    assert_eq!(
        counted_work::reverse_indexes(),
        0,
        "no own file closes a loop, so the package builds no reverse index"
    );
    assert_eq!(
        counted_work::walks(),
        0,
        "no own file closes a loop, so nothing is walked"
    );
    assert_eq!(
        counted_work::reached(),
        0,
        "no own file closes a loop, so no entry is reached"
    );
    // What: the control, the same package whose used folder reaches the group back.
    // Why: a count of zero is only an answer if this package's shape can reach one.
    let closing: crate::test_fs::Fixture = package_of_one_group_that_may_close(true);
    counted_work::reset();
    assert_eq!(own_file_loops_of(&closing.path).len(), 1);
    assert_eq!(
        counted_work::reverse_indexes(),
        1,
        "the control package builds the reverse index once"
    );
    assert_eq!(counted_work::walks(), 1, "the control package walks once");
}
