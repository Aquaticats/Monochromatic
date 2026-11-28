//! What:
//!  Controls of the file rule.
//! Why:
//!  Every finding about loops and about the top doc list is shown on a small package.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('files that use each other in a loop are one finding', () => expect(check(tree)).toEqual([...]));
//! ```

use super::{
    check_file_order,
    uses_between,
};
use crate::{
    layout_finding::LayoutFinding,
    layout_module_uses::collect_uses,
    layout_test_support::{
        package_with,
        tree_of,
    },
    layout_top_doc::LIST_MARKER,
};
use std::path::Path;

/// Run the rule on a fixture package and render each finding as `file:line message`,
/// with the fixture folder removed.
fn findings_of(root: &Path) -> Vec<String> {
    let tree = tree_of(root);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    let uses = collect_uses(&tree);
    let prefix: String = format!("{}/", root.display());
    return check_file_order(&tree, uses.as_slice(), root, None)
        .iter()
        .map(|item| {
            return format!("{}:{} {}", item.file.display(), item.line, item.message)
                .replace(prefix.as_str(), "");
        })
        .collect::<Vec<String>>();
}

/// A `mod.rs` for the folder `src/a` whose top doc lists the given file names after the marker.
fn index_listing(
    names: &[&str],
    modules: &[&str],
) -> String {
    let mut text: String = format!("//! Words about the folder.\n//!\n//! {LIST_MARKER}\n//!\n");
    for name in names {
        text.push_str(format!("//! - `{name}`: what it does.\n").as_str());
    }
    for module in modules {
        text.push_str(format!("mod {module};\n").as_str());
    }
    return text;
}

/// A top doc entry is read without the whitespace at its end,
///  so an entry written `one.rs ` names the file `one.rs`,
///  and no key of a finding about a doc entry ends in a character a list line trims away.
#[test]
fn a_top_doc_entry_loses_the_whitespace_at_its_end() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            format!(
                "//! Words about the folder.\n//!\n//! {LIST_MARKER}\n//!\n//! - `one.rs `: the entry with a space before its backtick.\nmod one;\n"
            )
            .as_str(),
        ),
        ("src/a/one.rs", ""),
    ]);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// Files listed after the files they use,
/// with no loop,
/// make no finding;
/// uses of the index file,
/// of other folders and of oneself are outside the rule.
#[test]
fn files_listed_after_what_they_use_make_no_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod z;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["base.rs", "top.rs"], &["base", "top"]).as_str(),
        ),
        (
            "src/a/base.rs",
            "use super::Shared;\nuse crate::z::Other;\nfn f() { self::f(); }\n",
        ),
        ("src/a/top.rs", "use super::base::Thing;\n"),
        ("src/z/mod.rs", ""),
    ]);
    // The root folder holds only lib.rs, which is the index, so it lists nothing.
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// Files that use each other,
/// directly or through a third,
/// are one loop finding with one reference per edge.
#[test]
fn files_that_use_each_other_are_one_loop_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(
                &["one.rs", "two.rs", "three.rs", "free.rs"],
                &["one", "two", "three", "free"],
            )
            .as_str(),
        ),
        ("src/a/one.rs", "use super::two::X;\n"),
        ("src/a/two.rs", "use super::three::X;\n"),
        ("src/a/three.rs", "fn f() {}\nuse super::one::X;\n"),
        ("src/a/free.rs", "use super::one::X;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    for part in [
        "one.rs uses two.rs (src/a/one.rs:1 `super::two::X`)",
        "two.rs uses three.rs (src/a/two.rs:1 `super::three::X`)",
        "three.rs uses one.rs (src/a/three.rs:2 `super::one::X`)",
    ] {
        assert!(found[0].contains(part), "{part} missing in {}", found[0]);
    }
    // The free file uses the loop but is not in it, and it is listed last: no order finding.
    assert!(!found[0].contains("free.rs uses"));
}

/// A file that uses a file listed after it is reported on the proving reference.
#[test]
fn a_file_listed_before_what_it_uses_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["top.rs", "base.rs"], &["base", "top"]).as_str(),
        ),
        ("src/a/base.rs", ""),
        ("src/a/top.rs", "\nuse super::base::Thing;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("src/a/top.rs:2 `top.rs` uses `base.rs` (`super::base::Thing`)"),
        "{}",
        found[0]
    );
    assert!(found[0].contains("lists it first"), "{}", found[0]);
}

/// A missing,
/// an extra and a duplicate entry are each reported.
#[test]
fn missing_extra_and_duplicate_entries_are_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["one.rs", "one.rs", "ghost.rs"], &["one", "two"]).as_str(),
        ),
        ("src/a/one.rs", ""),
        ("src/a/two.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let text: String = found.join("\n");
    assert!(text.contains("lists `one.rs` twice"), "{text}");
    assert!(
        text.contains("lists `ghost.rs`, which is not a module file of this folder"),
        "{text}"
    );
    assert!(text.contains("does not list the file `two.rs`"), "{text}");
    assert_eq!(found.len(), 3, "{text}");
}

/// A top doc without the marker line has no list,
///  and the file the doc never lists is a finding of its own,
///  so a `no-file-list` entry cannot tolerate a file added later.
#[test]
fn a_top_doc_without_the_list_marker_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "//! Words only.\n//! - `one.rs`: bullets are not enough.\nmod one;\n",
        ),
        ("src/a/one.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(
        found[0].starts_with("src/a/mod.rs:1 The top doc of src/a/mod.rs has no file list"),
        "{}",
        found[0]
    );
    assert_eq!(
        found[1],
        "src/a/mod.rs:1 The top doc of src/a/mod.rs does not list the file `one.rs`."
    );
}

/// A folder with module files and no `mod.rs` is a finding of its own,
///  and so is every file that folder holds,
///  so a file added to it later is never tolerated by the one line about the missing doc.
#[test]
fn a_folder_without_mod_rs_has_no_top_doc() {
    let fixture = package_with(&[
        ("src/lib.rs", index_listing(&["a.rs"], &["a"]).as_str()),
        ("src/a.rs", "mod inner;\n"),
        ("src/a/inner.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(
        found[0].contains("The folder src/a holds module files but no mod.rs that declares them"),
        "{}",
        found[0]
    );
    assert!(
        found[1].contains("so the file `inner.rs` is not listed in any top doc"),
        "{}",
        found[1]
    );
}

/// Test files,
/// test-only modules and the other crate roots are not listed,
/// and the crate root folder lists its files in `lib.rs`.
#[test]
fn test_files_and_other_roots_are_never_listed() {
    let fixture = package_with(&[
        ("src/lib.rs", index_listing(&["one.rs"], &["one"]).as_str()),
        ("src/main.rs", "use monochromatic_lint::run;\nmod one;\n"),
        (
            "src/one.rs",
            "#[cfg(test)]\n#[path = \"one_tests.rs\"]\nmod tests;\n",
        ),
        ("src/one_tests.rs", "use super::*;\n"),
    ]);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// A loop that goes through a re-export of the index is found:
/// `one.rs` reaches `two.rs` with `super::Y`,
/// and the index only holds `pub use two::Y;`.
#[test]
fn a_loop_hidden_behind_a_reexport_of_the_index_is_found() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            format!(
                "{}pub use two::Y;\n",
                index_listing(&["one.rs", "two.rs"], &["one", "two"])
            )
            .as_str(),
        ),
        ("src/a/one.rs", "use super::Y;\n"),
        ("src/a/two.rs", "use super::one::X;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    for part in [
        "These files use each other in a loop",
        "one.rs uses two.rs (src/a/one.rs:1 `super::Y`)",
        "two.rs uses one.rs (src/a/two.rs:1 `super::one::X`)",
    ] {
        assert!(found[0].contains(part), "{part} missing in {}", found[0]);
    }
}

/// The order check counts a use through a re-export as a use of the defining file,
/// also under a new name and along a chain of two re-exports.
#[test]
fn the_order_check_follows_renamed_and_chained_reexports() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            format!(
                "{}pub use mid::Thing as Y;\n",
                index_listing(&["top.rs", "mid.rs", "base.rs"], &["top", "mid", "base"])
            )
            .as_str(),
        ),
        ("src/a/top.rs", "use super::Y;\n"),
        ("src/a/mid.rs", "pub use super::base::Thing;\n"),
        ("src/a/base.rs", "pub struct Thing;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let text: String = found.join("\n");
    // `top.rs` really uses `base.rs` (through two hops) and is listed before it.
    assert!(
        text.contains("`top.rs` uses `base.rs` (`super::Y`)"),
        "{text}"
    );
    // `mid.rs` is listed before `base.rs`, which it uses.
    assert!(text.contains("`mid.rs` uses `base.rs`"), "{text}");
    assert_eq!(found.len(), 2, "{text}");
}

/// A folder that holds child folders is a group,
/// and the file rule leaves a group to the folder rule:
/// neither its code in the index nor its missing file list is reported here.
#[test]
fn a_group_folder_is_left_to_the_folder_rule() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        ("src/a/mod.rs", "mod child;\nmod one;\nfn helper() {}\n"),
        ("src/a/one.rs", "use super::child::Thing;\n"),
        (
            "src/a/child/mod.rs",
            index_listing(&["thing.rs"], &["thing"]).as_str(),
        ),
        ("src/a/child/thing.rs", "pub struct Thing;\n"),
    ]);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// A ring of re-exports ends and makes no use between files.
#[test]
fn a_ring_of_reexports_makes_no_loop_and_no_hang() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            format!(
                "{}pub use one::Y;\n",
                index_listing(&["one.rs", "top.rs"], &["one", "top"])
            )
            .as_str(),
        ),
        ("src/a/one.rs", "pub use super::Y;\n"),
        ("src/a/top.rs", "use super::Y;\n"),
    ]);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// Code in the index file of a folder with other files is a finding of the file rule,
/// on the line of the item,
/// and it does not hide a loop:
/// the loop between the files is still found.
#[test]
fn code_in_the_index_of_a_folder_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            format!(
                "{}fn helper() {{}}\n",
                index_listing(&["one.rs", "two.rs"], &["one", "two"])
            )
            .as_str(),
        ),
        ("src/a/one.rs", "use super::two::X;\n"),
        ("src/a/two.rs", "use super::one::X;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    let text: String = found.join("\n");
    assert_eq!(found.len(), 2, "{text}");
    assert!(
        text.contains("src/a/mod.rs:9 src/a/mod.rs is the index of its folder"),
        "{text}"
    );
    assert!(text.contains("but it holds a function `helper`."), "{text}");
    assert!(
        text.contains("These files use each other in a loop"),
        "{text}"
    );
}

/// An index that is alone in its folder is checked too:
/// the crate root holds code and only folders.
#[test]
fn code_in_an_index_with_no_other_file_is_reported() {
    let fixture = package_with(&[("src/lib.rs", "mod a;\nfn f() {}\n"), ("src/a/mod.rs", "")]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("src/lib.rs:2 src/lib.rs is the index of its folder"),
        "{}",
        found[0]
    );
}

/// A `main.rs` that is the index of its folder may hold `fn main` and `use` lines beside the index.
#[test]
fn an_entry_point_may_hold_fn_main_and_its_uses() {
    let fixture = package_with(&[
        (
            "src/main.rs",
            "use std::process::ExitCode;\nmod a;\nfn main() -> ExitCode { ExitCode::SUCCESS }\n",
        ),
        ("src/a/mod.rs", ""),
    ]);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// Other code in an entry point is reported.
#[test]
fn other_code_in_an_entry_point_is_reported() {
    let fixture = package_with(&[
        ("src/main.rs", "mod a;\nfn run() {}\nfn main() { run(); }\n"),
        ("src/a/mod.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("src/main.rs:2 src/main.rs is the entry point of its program"),
        "{}",
        found[0]
    );
}

/// A `main.rs` beside a `lib.rs` is another crate root,
/// and it holds the same as an entry point:
/// its top doc,
/// `mod` lines,
/// `use` lines and `fn main`.
#[test]
fn an_entry_point_beside_a_library_root_is_read_as_an_entry_point() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        ("src/main.rs", "fn run() {}\nfn main() { run(); }\n"),
        ("src/a/mod.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("src/main.rs:1 src/main.rs is the entry point of its program"),
        "{}",
        found[0]
    );
    assert!(
        found[0].contains("but it holds a function `run`."),
        "{}",
        found[0]
    );

    let clean = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/main.rs",
            "//! The program.\nuse monochromatic_lint::run;\nfn main() { run(); }\n",
        ),
        ("src/a/mod.rs", ""),
    ]);
    assert_eq!(findings_of(&clean.path), Vec::<String>::new());
}

/// A single-file binary under `src/bin` is read as an entry point too,
/// and the folder `src/bin` needs no `mod.rs` for it.
#[test]
fn a_single_file_binary_is_read_as_an_entry_point() {
    let fixture = package_with(&[
        ("src/lib.rs", ""),
        ("src/bin/tool.rs", "const LIMIT: u8 = 1;\nfn main() {}\n"),
        (
            "src/bin/clean.rs",
            "use std::process::ExitCode;\nfn main() -> ExitCode { ExitCode::SUCCESS }\n",
        ),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("src/bin/tool.rs:1 src/bin/tool.rs is the entry point of its program"),
        "{}",
        found[0]
    );
    assert!(
        found[0].contains("but it holds a constant `LIMIT`."),
        "{}",
        found[0]
    );
}

/// A root that is neither a `main.rs` nor a single file under `src/bin`
/// (here a `[[bin]]` path elsewhere) is outside this check.
#[test]
fn another_root_is_not_read_for_code() {
    let fixture = package_with(&[
        (
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n[[bin]]\nname = \"tool\"\npath = \"src/tool.rs\"\n",
        ),
        ("src/lib.rs", ""),
        ("src/tool.rs", "fn helper() {}\nfn main() {}\n"),
    ]);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// A loop between a file and a file that re-exports from it is found,
/// although the user's path reaches the defining file through the re-export.
#[test]
fn a_loop_through_a_reexporting_file_is_still_found() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["front.rs", "back.rs"], &["front", "back"]).as_str(),
        ),
        ("src/a/front.rs", "pub use super::back::Y;\n"),
        ("src/a/back.rs", "use super::front::Y;\n"),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("front.rs uses back.rs"), "{}", found[0]);
    assert!(found[0].contains("back.rs uses front.rs"), "{}", found[0]);
}

/// A top doc whose marker wraps after its comma lists the files like any other,
/// so a package that follows the Markdown rule makes no finding.
#[test]
fn a_top_doc_with_a_wrapped_marker_lists_its_files() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "//! Words about the folder.\n//!\n//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `base.rs`:\n//!   the base.\n//! - `top.rs`: uses `base.rs`.\nmod base;\nmod top;\n",
        ),
        ("src/a/base.rs", ""),
        ("src/a/top.rs", "use super::base::Thing;\n"),
    ]);
    assert_eq!(findings_of(&fixture.path), Vec::<String>::new());
}

/// The help for a missing list shows the marker wrapped after its comma,
/// so following it does not clash with the Markdown rule.
#[test]
fn the_missing_list_help_shows_the_wrapped_marker() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        ("src/a/mod.rs", "//! Words only.\nmod one;\n"),
        ("src/a/one.rs", ""),
    ]);
    let tree = tree_of(&fixture.path);
    let uses = collect_uses(&tree);
    let found = check_file_order(&tree, uses.as_slice(), &fixture.path, None);
    let help: &str = found[0].help.as_deref().expect("help");
    let (head, tail) = LIST_MARKER.split_once(", ").expect("marker has a comma");
    assert!(
        help.contains(format!("`//! {head},` and `//! {tail}`").as_str()),
        "{help}"
    );
}

/// A top doc that spells the marker nearly is a finding of its own:
///  it names the doc, the line, the words found and the marker expected,
///  so the reader sees the cause and not only the missing list and every file it holds.
#[test]
fn a_near_miss_marker_is_a_finding_of_its_own() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "//! Words about the folder.\n//!\n//! Files in this folder,\n//! each using only the files listed before it:\n//!\n//! - `one.rs`: one.\nmod one;\n",
        ),
        ("src/a/one.rs", ""),
    ]);
    let tree = tree_of(&fixture.path);
    let uses = collect_uses(&tree);
    let found = check_file_order(&tree, uses.as_slice(), &fixture.path, None);
    let near: Vec<&LayoutFinding> = found
        .iter()
        .filter(|item| return item.key.starts_with("near-miss-marker"))
        .collect::<Vec<&LayoutFinding>>();
    assert_eq!(near.len(), 1, "{found:#?}");
    let item = near[0];
    assert_eq!(item.file, fixture.path.join("src/a/mod.rs"));
    assert_eq!(item.line, 3);
    for part in [
        "The top doc of src/a/mod.rs nearly spells the file-list marker at line 3",
        "`Files in this folder, each using only the files listed before it:`",
        format!("where the marker is `{LIST_MARKER}`").as_str(),
    ] {
        assert!(
            item.message.contains(part),
            "{part} missing in {}",
            item.message
        );
    }
    assert_eq!(item.key, "near-miss-marker src/a/mod.rs");
}

/// The near-miss key is a line the list reads back,
///  so a round records the finding and the next run reports nothing.
#[test]
fn the_near_miss_key_is_a_line_the_list_reads_back() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "//! Words about the folder.\n//!\n//! Files in this folder,\n//! each using only the files listed before it:\n//!\n//! - `one.rs`: one.\nmod one;\n",
        ),
        ("src/a/one.rs", ""),
    ]);
    assert_eq!(
        findings_with(
            &fixture.path,
            "no-file-list src/a/mod.rs\nmissing-entry src/a/mod.rs one.rs\nnear-miss-marker src/a/mod.rs\n",
        ),
        Vec::<String>::new(),
    );
}

/// A second near miss in one doc keeps a key of its own,
///  and each finding is reported on the line where its words start.
#[test]
fn a_second_near_miss_in_one_doc_keeps_its_own_key() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "//! Files in this folder, each using only the files listed before it:\n//!\n//! Files in this folder, each using only files listed after it:\n//! - `one.rs`: one.\nmod one;\n",
        ),
        ("src/a/one.rs", ""),
    ]);
    let tree = tree_of(&fixture.path);
    let uses = collect_uses(&tree);
    let found = check_file_order(&tree, uses.as_slice(), &fixture.path, None);
    let near: Vec<(&str, usize)> = found
        .iter()
        .filter(|item| return item.key.starts_with("near-miss-marker"))
        .map(|item| return (item.key.as_str(), item.line))
        .collect::<Vec<(&str, usize)>>();
    assert_eq!(
        near,
        [
            ("near-miss-marker src/a/mod.rs", 1),
            ("near-miss-marker#2 src/a/mod.rs", 3),
        ],
        "{found:#?}"
    );
}

/// Of several uses between the same two files,
///  the smallest proof is kept whatever order they were found in.
#[test]
fn layout_the_smallest_proof_of_a_use_between_two_files_wins_in_any_order() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a.rs", ""),
        (
            "src/b.rs",
            "use crate::a::First;\nuse crate::a::Second;\nuse crate::a::Third;\n",
        ),
    ]);
    let tree = tree_of(&fixture.path);
    let uses = collect_uses(&tree);
    let listed: Vec<std::path::PathBuf> =
        vec![fixture.path.join("src/a.rs"), fixture.path.join("src/b.rs")];
    let mut backwards = uses.clone();
    backwards.reverse();
    let between: usize = uses
        .iter()
        .filter(|found| {
            return tree.modules[found.from].file == listed[1]
                && tree.modules[found.to].file == listed[0];
        })
        .count();
    assert!(between >= 2, "the control needs several uses: {uses:?}");
    for order in [uses, backwards] {
        let edges = uses_between(&tree, order.as_slice(), listed.as_slice());
        assert_eq!(edges[&(1, 0)].line, 1, "{order:?}");
    }
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
    let list = crate::layout_allow_list::parse_allow_list(lines, Path::new("allow-list.txt"), root)
        .expect("the fixture list is valid");
    let prefix: String = format!("{}/", root.display());
    return check_file_order(&tree, uses.as_slice(), root, Some(&list))
        .iter()
        .map(|item| {
            return format!("{}:{} {}", item.file.display(), item.line, item.message)
                .replace(prefix.as_str(), "");
        })
        .collect::<Vec<String>>();
}

/// With both uses of a two-file loop on the list the loop is not reported,
///  and the top doc still lists both files.
#[test]
fn listed_file_uses_take_their_loop_out_of_the_report() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["one.rs", "two.rs"], &["one", "two"]).as_str(),
        ),
        ("src/a/one.rs", "use super::two::X;\n"),
        ("src/a/two.rs", "use super::one::Y;\n"),
    ]);
    assert_eq!(findings_with(&fixture.path, "").len(), 1);
    let listed: &str = "file-uses src/a one.rs -> two.rs\nfile-uses src/a two.rs -> one.rs\n";
    assert_eq!(findings_with(&fixture.path, listed), Vec::<String>::new());
}

/// A listed file use that names no use is a finding on its own line.
#[test]
fn a_listed_file_use_that_names_no_use_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["one.rs"], &["one"]).as_str(),
        ),
        ("src/a/one.rs", ""),
    ]);
    let found: Vec<String> = findings_with(&fixture.path, "file-uses src/a one.rs -> two.rs\n");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with(
            "allow-list.txt:1 The allow-list entry `file-uses src/a one.rs -> two.rs`"
        ),
        "{}",
        found[0]
    );
}

/// A listed key takes exactly the doc finding it names out of the report,
///  and a second file the doc omits stays a finding,
///  which a fixture with one finding could not show.
#[test]
fn a_listed_doc_key_takes_its_finding_out_of_the_report() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["one.rs"], &["one", "two", "three"]).as_str(),
        ),
        ("src/a/one.rs", ""),
        ("src/a/two.rs", ""),
        ("src/a/three.rs", ""),
    ]);
    assert_eq!(findings_with(&fixture.path, "").len(), 2);
    let listed: &str = "missing-entry src/a/mod.rs two.rs\n";
    let found: Vec<String> = findings_with(&fixture.path, listed);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("does not list the file `three.rs`"),
        "{}",
        found[0]
    );
}

/// Every copy of a name the doc lists twice or more has its own key,
///  so one line tolerates one extra copy and a third copy is reported.
#[test]
fn every_extra_copy_of_a_listed_name_has_its_own_key() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `one.rs`: one.\n//! - `one.rs`: again.\n//! - `one.rs`: a third time.\nmod one;\n",
        ),
        ("src/a/one.rs", ""),
    ]);
    let tree = tree_of(&fixture.path);
    let uses = collect_uses(&tree);
    let copies: Vec<crate::layout_finding::LayoutFinding> =
        check_file_order(&tree, uses.as_slice(), &fixture.path, None)
            .into_iter()
            .filter(|item| return item.key.starts_with("listed-twice"))
            .collect::<Vec<crate::layout_finding::LayoutFinding>>();
    assert_eq!(copies.len(), 2, "{copies:#?}");
    assert_eq!(copies[0].key, "listed-twice src/a/mod.rs one.rs");
    // The second extra copy gains a number in its first word, so a line for the first names one finding
    // and no doc entry named `one.rs#2` can collide with it.
    assert_eq!(copies[1].key, "listed-twice#2 src/a/mod.rs one.rs");
    let list = crate::layout_allow_list::parse_allow_list(
        "listed-twice src/a/mod.rs one.rs\n",
        Path::new("allow-list.txt"),
        &fixture.path,
    )
    .expect("the fixture list is valid");
    let kept: Vec<String> = check_file_order(&tree, uses.as_slice(), &fixture.path, Some(&list))
        .iter()
        .map(|item| return item.key.clone())
        .collect::<Vec<String>>();
    assert_eq!(kept, ["listed-twice#2 src/a/mod.rs one.rs"], "{kept:#?}");
}

/// A listed file use whose two files are no longer in one loop is a finding,
///  because the entry no longer holds a loop out of the report.
#[test]
fn a_listed_file_use_that_holds_no_loop_is_reported() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["one.rs", "two.rs"], &["one", "two"]).as_str(),
        ),
        ("src/a/one.rs", "use super::two::X;\n"),
        ("src/a/two.rs", ""),
    ]);
    // The use is made, and nothing uses `one.rs` back, so its two ends are in no loop.
    let found: Vec<String> = findings_with(&fixture.path, "file-uses src/a one.rs -> two.rs\n");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("tolerates nothing any more"),
        "{}",
        found[0]
    );
}

/// A dead `file-uses` line names the folder the rule read,
///  so a reader whose line is mistyped or hand-aligned is told which name the tree lacks
///  rather than only to delete the line.
#[test]
fn a_dead_file_use_line_names_the_folder_the_rule_read() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(&["one.rs", "two.rs"], &["one", "two"]).as_str(),
        ),
        ("src/a/one.rs", ""),
        ("src/a/two.rs", ""),
    ]);
    let found: Vec<String> = findings_with(&fixture.path, "file-uses src/gone one.rs -> two.rs\n");
    assert_eq!(found.len(), 1, "{found:#?}");
    for part in [
        "names the folder `src/gone`",
        "No source folder of this package has that name",
        "tolerates nothing any more",
    ] {
        assert!(found[0].contains(part), "{part} missing in {}", found[0]);
    }
    // The control: a line naming a folder this package holds names no folder, and is told only to be deleted.
    // Why: without the folders of the tree the check would call every dead line a misspelling.
    let held: Vec<String> = findings_with(&fixture.path, "file-uses src/a one.rs -> two.rs\n");
    assert_eq!(held.len(), 1, "{held:#?}");
    assert!(
        held[0].contains("tolerates nothing any more. Delete the line"),
        "{}",
        held[0]
    );
    assert!(
        !held[0].contains("No source folder of this package has that name"),
        "{}",
        held[0]
    );
}

/// A `file-uses` line reads its folder the way a `uses` line reads one:
///  a path that climbs out with `..` and comes back in names the folder every key names,
///  so the alias tolerates the use it spells.
#[test]
fn a_listed_file_use_reads_its_folder_back_against_the_package_folder() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod ab;\n"),
        (
            "src/ab/mod.rs",
            index_listing(&["one.rs", "two.rs"], &["one", "two"]).as_str(),
        ),
        ("src/ab/one.rs", "use super::two::T;\n"),
        ("src/ab/two.rs", "use super::one::O;\n"),
    ]);
    let name: String = fixture
        .path
        .file_name()
        .expect("the fixture folder has a name")
        .to_string_lossy()
        .into_owned();
    let listed: String = format!(
        "file-uses ../{name}/src/ab one.rs -> two.rs\nfile-uses ../{name}/src/ab two.rs -> one.rs\n"
    );
    assert_eq!(
        findings_with(&fixture.path, listed.as_str()),
        Vec::<String>::new(),
        "the alias names the same folder as `file-uses src/ab`"
    );
}

/// A use added between two files of a tolerated loop is a finding,
///  because a tolerated loop may only lose uses.
#[test]
fn a_new_use_inside_a_tolerated_file_loop_is_a_finding() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            index_listing(
                &["alpha.rs", "beta.rs", "gamma.rs"],
                &["alpha", "beta", "gamma"],
            )
            .as_str(),
        ),
        ("src/a/alpha.rs", "use super::beta::X;\n"),
        ("src/a/beta.rs", "use super::gamma::Y;\n"),
        ("src/a/gamma.rs", "use super::alpha::Z;\n"),
    ]);
    // The control: without the list the loop of three files is one finding.
    let open: Vec<String> = findings_with(&fixture.path, "");
    assert_eq!(open.len(), 1, "{open:#?}");
    // Two of the loop's three uses are recorded, so the third is a use added inside a tolerated loop.
    let listed: &str = "file-uses src/a alpha.rs -> beta.rs\nfile-uses src/a beta.rs -> gamma.rs\n";
    let found: Vec<String> = findings_with(&fixture.path, listed);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("and both files sit inside one loop the allow list tolerates today"),
        "{}",
        found[0]
    );
    assert!(
        found[0].contains("`gamma.rs` uses `alpha.rs`"),
        "{}",
        found[0]
    );
}
/// A `mod.rs` on disk that no `mod` line declares is no part of the package,
///  so a folder whose files are declared from elsewhere keeps no index
///  and owes a top doc of its own.
#[test]
fn a_readable_index_that_no_module_line_declares_is_not_the_folders_index() {
    let fixture = package_with(&[
        (
            "src/lib.rs",
            "#[path = \"a/b.rs\"] mod b;\n#[path = \"a/c.rs\"] mod c;\n",
        ),
        ("src/a/b.rs", "pub struct B;\n"),
        ("src/a/c.rs", "pub struct C;\n"),
        // The stray file lists both files, and no `mod` line reaches it.
        (
            "src/a/mod.rs",
            "//! Files here.\n//!\n//! - `b.rs`: b.\n//! - `c.rs`: c.\nmod b;\nmod c;\n",
        ),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    // The folder keeps no index, so it owes a top doc of its own and lists neither file.
    assert!(
        found.iter().any(|text| return text.contains(
            "The folder src/a holds module files but no mod.rs that declares them, so it has no top doc to list them in. The mod.rs on disk is not part of the package, because no mod line declares it."
        )),
        "{found:#?}"
    );
    for name in ["b.rs", "c.rs"] {
        assert!(
            found.iter().any(|text| return text
                .contains(format!("the file `{name}` is not listed in any top doc").as_str())),
            "{name} missing in {found:#?}"
        );
    }
    assert!(
        found
            .iter()
            .all(|text| return !text.contains("src/a/mod.rs")),
        "{found:#?}"
    );
}

/// An index that cannot be read is taken only when the folder holds no index the tree reached,
///  so the crate root stays the index of a package that holds an unreadable `src/mod.rs`.
#[test]
fn an_unreadable_index_beside_a_crate_root_is_not_the_folders_index() {
    let fixture = package_with(&[("src/lib.rs", "mod a;\n"), ("src/a.rs", "")]);
    std::fs::write(fixture.path.join("src/mod.rs"), [0x2f, 0x2f, 0xff, b'\n'])
        .expect("fixture stray index");
    let found: Vec<String> = findings_of(&fixture.path);
    assert!(
        found.iter().any(|text| return text.contains(
            "The top doc of src/lib.rs has no file list, so its files are not listed in order."
        )),
        "{found:#?}"
    );
    assert!(
        found.iter().all(|text| return !text.contains("src/mod.rs")),
        "{found:#?}"
    );
}

/// The lines one allow list earns on a fixture package,
///  read from a file of that package so a message quotes its name and shows it as a person sees it.
fn findings_with_list(
    root: &Path,
    lines: &str,
) -> Vec<String> {
    let tree = tree_of(root);
    assert!(tree.problems.is_empty(), "{:?}", tree.problems);
    let uses = collect_uses(&tree);
    let source: std::path::PathBuf = root.join("allow-list.txt");
    std::fs::write(&source, lines).expect("write the fixture list");
    let list = crate::layout_allow_list::parse_allow_list(lines, source.as_path(), root)
        .expect("the fixture list reads");
    let prefix: String = format!("{}/", root.display());
    return check_file_order(&tree, uses.as_slice(), root, Some(&list))
        .iter()
        .map(|item| {
            return format!("{}:{} {}", item.file.display(), item.line, item.message)
                .replace(prefix.as_str(), "");
        })
        .collect::<Vec<String>>();
}

/// A dead `file-uses` line is told to take a character out only when that character stands in a file name
/// the folder's modules do not hold,
///  because a line that spells a real module file's own name is dead for another reason,
///  and the file name is what the finding prints.
#[test]
fn a_dead_file_use_line_takes_a_character_out_only_when_no_module_of_the_folder_holds_its_file_name()
 {
    // What: one folder whose module file really is named with the character, and its line, spelled as the file is.
    // Why: the use the line records sits in no loop, so the line is dead although its name is the file's own.
    let genuine = package_with(&[
        (
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n\n[lib]\npath = \"lib.rs\"\n",
        ),
        (
            "lib.rs",
            "//! Index.\n#[path = \"src/f/mod.rs\"]\npub mod m0;\n",
        ),
        (
            "src/f/mod.rs",
            "//! Folder.\n//!\n//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `two.rs`: two.\n//! - `o\u{200b}ne.rs`: one.\n#[path = \"o\u{200b}ne.rs\"]\nmod one;\nmod two;\n",
        ),
        ("src/f/two.rs", "/// Two.\npub struct Two;\n"),
        (
            "src/f/o\u{200b}ne.rs",
            "use super::two::Two;\n/// One.\npub struct One(pub Two);\n",
        ),
    ]);
    let message: String = findings_with_list(
        genuine.path.as_path(),
        "file-uses src/f o\u{200b}ne.rs -> two.rs\n",
    )
    .join("\n");
    assert!(
        message.contains("tolerates nothing any more. Delete the line"),
        "the name is the file's own, so the use is what is gone: {message}"
    );
    assert!(
        !message.contains("A file name is read exactly as the line writes it"),
        "the file name here is the real one: {message}"
    );
    // What: the control, a file name the folder does not hold, which is the case the advice is for.
    // Why: an advice that never fired would pass the lines above and leave a mistyped name unreported.
    let mistyped: String = findings_with_list(
        genuine.path.as_path(),
        "file-uses src/f one\u{200b}.rs -> two.rs\n",
    )
    .join("\n");
    assert!(
        mistyped.contains("Take that character out of the line"),
        "no module of the folder is called `one\u{200b}.rs`: {mistyped}"
    );
    // What: a character in the folder's own name, which is no file name and so gets no file-name advice.
    // Why: the folder is real, so the line is dead because its use is gone, and taking the character out would name another folder.
    let folder_name = package_with(&[
        (
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n\n[lib]\npath = \"lib.rs\"\n",
        ),
        (
            "lib.rs",
            "//! Index.\n#[path = \"src/e\u{200c}f/mod.rs\"]\npub mod m0;\n",
        ),
        (
            "src/e\u{200c}f/mod.rs",
            "//! Folder.\n//!\n//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `two.rs`: two.\n//! - `three.rs`: three.\nmod three;\nmod two;\n",
        ),
        ("src/e\u{200c}f/two.rs", "/// Two.\npub struct Two;\n"),
        ("src/e\u{200c}f/three.rs", "/// Three.\npub struct Three;\n"),
    ]);
    let in_folder: String = findings_with_list(
        folder_name.path.as_path(),
        "file-uses src/e\u{200c}f three.rs -> two.rs\n",
    )
    .join("\n");
    assert!(
        !in_folder.contains("A file name is read exactly as the line writes it"),
        "the character stands in the folder's name, not a file's: {in_folder}"
    );
    assert!(
        in_folder.contains("tolerates nothing any more. Delete the line"),
        "the use is gone, so the line is deleted: {in_folder}"
    );
}

/// A dead `file-uses` line for a folder a `#[path]` names outside the package is judged by the file the walk reached,
///  with the walk the line writes read back first,
///  because the tree spells that file from the package folder and the line spells it from its own:
///  a reader that joined the line's folder to the package folder without reading the `..` away
///  told a line that spells a real module file's own name to take that character out.
#[test]
fn a_dead_file_use_line_for_a_folder_outside_the_package_names_the_file_the_walk_reached() {
    // What: a package whose one folder is a sibling of it, reached with `..`, holding a real module file named with the character.
    // Why: the tree spells that file `../outside/o\u{200b}ne.rs`, and the line writes the same walk.
    let outside = package_with(&[
        (
            "pkg/Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n\n[lib]\npath = \"lib.rs\"\n",
        ),
        (
            "pkg/lib.rs",
            "//! Index.\n#[path = \"../outside/mod.rs\"]\npub mod m0;\n",
        ),
        (
            "outside/mod.rs",
            "//! Folder.\n//!\n//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `two.rs`: two.\n//! - `o\u{200b}ne.rs`: one.\n#[path = \"o\u{200b}ne.rs\"]\nmod one;\nmod two;\n",
        ),
        ("outside/two.rs", "/// Two.\npub struct Two;\n"),
        (
            "outside/o\u{200b}ne.rs",
            "use super::two::Two;\n/// One.\npub struct One(pub Two);\n",
        ),
    ]);
    let root: std::path::PathBuf = outside.path.join("pkg");
    let genuine: String = findings_with_list(
        root.as_path(),
        "file-uses ../outside o\u{200b}ne.rs -> two.rs\n",
    )
    .join("\n");
    assert!(
        genuine.contains("tolerates nothing any more. Delete the line"),
        "the name is the file the walk reached, so the use is what is gone: {genuine}"
    );
    assert!(
        !genuine.contains("no module of this folder is called the name it holds"),
        "the folder's module is called that name: {genuine}"
    );
    // What: the control, a file name the same folder does not hold, which is the case the advice is for.
    // Why: an advice that never fired for a folder reached through `..` would pass the lines above.
    let mistyped: String = findings_with_list(
        root.as_path(),
        "file-uses ../outside one\u{200b}.rs -> two.rs\n",
    )
    .join("\n");
    assert!(
        mistyped.contains("no module of this folder is called the name it holds"),
        "no module of the folder is called `one\u{200b}.rs`: {mistyped}"
    );
}
