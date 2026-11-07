//! What:
//!  Controls of the group checks of the folder rule.
//! Why:
//!  A group folder holds only folders,
//!  its `uses` folder summarizes the entries its child folders use,
//!  a child links the exact folders it uses and never a group,
//!  and no two entries use each other.
//!  Every finding is shown on a small package,
//!  next to the packages that must make none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a missing summary link is reported', () => expect(check(tree)).toEqual([...]));
//! ```

use crate::{
    layout_finding::LayoutFinding,
    layout_folder_links::check_folder_links,
    layout_module_uses::collect_uses,
    layout_test_support::{link, package_with, tree_of},
};
use std::path::Path;

/// Run the folder rule on a fixture package and render each finding as `file:line message`,
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

/// Two groups:
/// `g` holds `a` and `b`,
/// `h` holds `c`,
/// and `c` uses `a`.
fn two_groups() -> Vec<(&'static str, &'static str)> {
    return vec![
        ("src/lib.rs", "mod g;\nmod h;\n"),
        ("src/g/mod.rs", "//! The group g.\nmod a;\nmod b;\n"),
        ("src/g/a/mod.rs", "pub struct X;\n"),
        ("src/g/b/mod.rs", ""),
        ("src/h/mod.rs", "mod c;\n"),
        ("src/h/c/mod.rs", "use crate::g::a::X;\n"),
    ];
}

/// The exact link of `c` to the folder it uses.
fn link_child_to_a(root: &Path) {
    link(root, "src/h/c/uses/a", "../../../g/a");
}

/// The summary link of `h` to the group that `c` uses a folder of.
fn link_group_h_to_g(root: &Path) {
    link(root, "src/h/uses/g", "../../g");
}

/// A shelf whose child links are exact and whose group links summarize them has no finding.
#[cfg(unix)]
#[test]
fn layout_a_shelf_with_exact_links_and_summaries_has_no_finding() {
    let fixture = package_with(&two_groups());
    link_child_to_a(&fixture.path);
    link_group_h_to_g(&fixture.path);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// A group with no summary link for an entry that its child uses is reported on the child's reference.
#[cfg(unix)]
#[test]
fn layout_a_group_without_the_summary_link_is_reported_with_its_proof() {
    let fixture = package_with(&two_groups());
    link_child_to_a(&fixture.path);
    assert_eq!(
        findings_of(&fixture.path),
        [
            "src/h/c/mod.rs:1 src/h/c uses the folder src/g/a (`crate::g::a::X`), which is in the group src/g, but src/h/uses has no link to src/g."
        ]
    );
}

/// A summary link that no child needs is reported on the link.
#[cfg(unix)]
#[test]
fn layout_a_summary_link_that_no_child_needs_is_reported() {
    let mut files = two_groups();
    files[0].1 = "mod g;\nmod h;\nmod k;\n";
    files.push(("src/k/mod.rs", ""));
    let fixture = package_with(&files);
    link_child_to_a(&fixture.path);
    link_group_h_to_g(&fixture.path);
    link(&fixture.path, "src/h/uses/k", "../../k");
    assert_eq!(
        findings_of(&fixture.path),
        ["src/h/uses/k:0 src/h/uses/k links to src/k, but no folder inside src/h uses it."]
    );
}

/// A summary link that points into another group instead of at the group says to link the group.
#[cfg(unix)]
#[test]
fn layout_a_summary_link_into_another_group_is_reported() {
    let fixture = package_with(&two_groups());
    link_child_to_a(&fixture.path);
    link(&fixture.path, "src/h/uses/a", "../../g/a");
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found.contains(&String::from(
            "src/h/uses/a:0 src/h/uses/a links to src/g/a, which is inside the group src/g, so a group links the group and not a folder inside it."
        )),
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|text| return text.contains("src/h/uses has no link to src/g.")),
        "{found:#?}"
    );
}

/// A child that uses a one-folder entry needs the entry in its group's summary,
/// and with the link in place the package is clean.
#[cfg(unix)]
#[test]
fn layout_a_one_folder_entry_is_summarized_by_its_own_name() {
    let mut files = two_groups();
    files[0].1 = "mod g;\nmod h;\nmod x;\n";
    files[5].1 = "use crate::x::Y;\n";
    files.push(("src/x/mod.rs", "pub struct Y;\n"));
    let fixture = package_with(&files);
    link(&fixture.path, "src/h/c/uses/x", "../../../x");
    assert_eq!(
        findings_of(&fixture.path),
        [
            "src/h/c/mod.rs:1 src/h/c uses the folder src/x (`crate::x::Y`), but src/h/uses has no link to src/x."
        ]
    );
    link(&fixture.path, "src/h/uses/x", "../../x");
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// Folders of one group that use each other need exact links and no summary.
#[cfg(unix)]
#[test]
fn layout_uses_inside_one_group_need_no_summary() {
    let mut files = two_groups();
    files[3].1 = "use crate::g::a::X;\n";
    let fixture = package_with(&files);
    link(&fixture.path, "src/g/b/uses/a", "../../a");
    link_child_to_a(&fixture.path);
    link_group_h_to_g(&fixture.path);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// A module file other than `mod.rs` in a group folder is reported.
#[test]
fn layout_a_code_file_in_a_group_is_reported() {
    let mut files = two_groups();
    files[1].1 = "mod a;\nmod b;\nmod helper;\n";
    files.push(("src/g/helper.rs", "pub fn go() {}\n"));
    let fixture = package_with(&files);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found.contains(&String::from(
            "src/g/helper.rs:0 src/g holds the file helper.rs, but a group folder holds only its child folders and its mod.rs."
        )),
        "{found:#?}"
    );
}

/// An item in the index of a group is reported on its line.
#[cfg(unix)]
#[test]
fn layout_code_in_the_index_of_a_group_is_reported() {
    let mut files = two_groups();
    files[1].1 = "mod a;\nmod b;\nfn helper() {}\n";
    let fixture = package_with(&files);
    link_child_to_a(&fixture.path);
    link_group_h_to_g(&fixture.path);
    assert_eq!(
        findings_of(&fixture.path),
        [
            "src/g/mod.rs:3 src/g/mod.rs is the index of the group folder src/g, so it may hold only its top doc and `mod` lines, but it holds a function `helper`."
        ]
    );
}

/// A re-export in the index of a group is reported,
/// because it would let a path name the group instead of the folder that defines the item.
#[test]
fn layout_a_reexport_in_the_index_of_a_group_is_reported() {
    let mut files = two_groups();
    files[1].1 = "mod a;\nmod b;\npub use a::X;\n";
    let fixture = package_with(&files);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found.contains(&String::from(
            "src/g/mod.rs:3 src/g/mod.rs is the index of the group folder src/g, so it may hold only its top doc and `mod` lines, but it holds a `use` line."
        )),
        "{found:#?}"
    );
}

/// A child that links its group,
/// and not the folder inside it that it uses,
/// is told the group is never linked.
#[cfg(unix)]
#[test]
fn layout_a_child_that_links_a_group_is_reported() {
    let fixture = package_with(&two_groups());
    link(&fixture.path, "src/h/c/uses/g", "../../../g");
    link_group_h_to_g(&fixture.path);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found.contains(&String::from(
            "src/h/c/uses/g:0 src/h/c/uses/g links to the group folder src/g, but a folder links the folders inside a group that it uses, never the group itself."
        )),
        "{found:#?}"
    );
    assert!(
        found.iter().any(|text| return text.contains(
            "src/h/c uses the folder src/g/a (`crate::g::a::X`) but src/h/c/uses has no link to it."
        )),
        "{found:#?}"
    );
}

/// A child whose code names a group itself is told to name a folder inside it.
#[test]
fn layout_a_child_that_names_a_group_is_reported() {
    let mut files = two_groups();
    files[5].1 = "use crate::g;\n";
    let fixture = package_with(&files);
    assert_eq!(
        findings_of(&fixture.path),
        [
            "src/h/c/mod.rs:1 src/h/c uses the group folder src/g itself (`crate::g`), but a folder names the folders inside a group that it uses, never the group itself."
        ]
    );
}

/// Two groups whose child folders use each other across the groups are a loop between the groups,
/// although no two child folders form a loop.
#[cfg(unix)]
#[test]
fn layout_two_groups_that_use_each_other_are_a_loop() {
    let mut files = two_groups();
    files[2].1 = "use crate::h::c::Y;\n";
    files[3].1 = "pub struct X;\n";
    files[4].1 = "mod c;\nmod d;\n";
    files[5].1 = "pub struct Y;\n";
    files.push(("src/h/d/mod.rs", "use crate::g::b::X;\n"));
    let fixture = package_with(&files);
    link(&fixture.path, "src/g/a/uses/c", "../../../h/c");
    link(&fixture.path, "src/h/d/uses/b", "../../../g/b");
    link(&fixture.path, "src/g/uses/h", "../../h");
    link(&fixture.path, "src/h/uses/g", "../../g");
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    for part in [
        "src/g uses src/h (src/g/a/mod.rs:1 `crate::h::c::Y`)",
        "src/h uses src/g (src/h/d/mod.rs:1 `crate::g::b::X`)",
    ] {
        assert!(found[0].contains(part), "{part} missing in {}", found[0]);
    }
    assert!(
        found[0].starts_with("src/g/a/mod.rs:1 These groups and folders use each other in a loop"),
        "{}",
        found[0]
    );
}

/// A group and a one-folder entry that use each other are a loop,
/// although the folder uses form none.
#[cfg(unix)]
#[test]
fn layout_a_group_and_a_one_folder_entry_that_use_each_other_are_a_loop() {
    let files = vec![
        ("src/lib.rs", "mod g;\nmod t;\n"),
        ("src/g/mod.rs", "mod a;\nmod b;\n"),
        ("src/g/a/mod.rs", "pub struct X;\n"),
        ("src/g/b/mod.rs", "use crate::t::Z;\n"),
        ("src/t/mod.rs", "pub struct Z;\nuse crate::g::a::X;\n"),
    ];
    let fixture = package_with(&files);
    link(&fixture.path, "src/t/uses/a", "../../g/a");
    link(&fixture.path, "src/g/b/uses/t", "../../../t");
    link(&fixture.path, "src/g/uses/t", "../../t");
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("src/g uses src/t (src/g/b/mod.rs:1 `crate::t::Z`)")
            && found[0].contains("src/t uses src/g (src/t/mod.rs:2 `crate::g::a::X`)"),
        "{}",
        found[0]
    );
}

/// A group inside a group is reported,
/// because the rule is one level of groups.
#[test]
fn layout_a_group_inside_a_group_is_reported() {
    let mut files = two_groups();
    files[2].1 = "mod deep;\n";
    files.push(("src/g/a/deep/mod.rs", ""));
    let fixture = package_with(&files);
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found.contains(&String::from(
            "src/g/a/mod.rs:0 src/g/a holds folders, so it is a group folder, but it sits inside the group folder src/g, and groups go one level deep."
        )),
        "{found:#?}"
    );
}

/// Two child folders that use the same entry make one missing-summary finding,
///  on the reference of the smaller proof,
///  whichever child is read last.
#[cfg(unix)]
#[test]
fn layout_the_missing_summary_is_reported_on_the_smallest_proof_of_two_children() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod g;\nmod h;\n"),
        ("src/g/mod.rs", "//! The group g.\nmod a;\nmod b;\n"),
        ("src/g/a/mod.rs", "pub struct X;\n"),
        ("src/g/b/mod.rs", ""),
        ("src/h/mod.rs", "mod c;\nmod d;\n"),
        ("src/h/c/mod.rs", "use crate::g::a::X;\n"),
        ("src/h/d/mod.rs", "use crate::g::a::X;\n"),
    ]);
    link(&fixture.path, "src/h/c/uses/a", "../../../g/a");
    link(&fixture.path, "src/h/d/uses/a", "../../../g/a");
    let missing: Vec<String> = findings_of(&fixture.path)
        .into_iter()
        .filter(|item| return item.contains("src/h/uses has no link to src/g"))
        .collect::<Vec<String>>();
    assert_eq!(missing.len(), 1, "{missing:#?}");
    assert!(
        missing[0].starts_with("src/h/c/mod.rs:1 "),
        "{}",
        missing[0]
    );
}

/// A summary link to a folder inside its own group is a link that nothing needs,
///  and is not described as a link into another group.
#[cfg(unix)]
#[test]
fn layout_a_summary_link_to_a_folder_of_its_own_group_is_an_unneeded_link() {
    let fixture = package_with(&two_groups());
    link_child_to_a(&fixture.path);
    link_group_h_to_g(&fixture.path);
    link(&fixture.path, "src/h/uses/c", "../c");
    let extra: Vec<String> = findings_of(&fixture.path)
        .into_iter()
        .filter(|item| return item.contains("src/h/uses/c links to"))
        .collect::<Vec<String>>();
    assert_eq!(extra.len(), 1, "{extra:#?}");
    assert!(
        extra[0].contains("src/h/uses/c links to src/h/c, but no folder inside src/h uses it."),
        "{}",
        extra[0]
    );
}
