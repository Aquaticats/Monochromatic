//! What:
//!  Controls of the file rule.
//! Why:
//!  Every finding about loops and about the top doc list is shown on a small package.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('files that use each other in a loop are one finding', () => expect(check(tree)).toEqual([...]));
//! ```

use super::check_file_order;
use crate::{
    layout_module_uses::collect_uses,
    layout_test_support::{package_with, tree_of},
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
    return check_file_order(&tree, uses.as_slice(), root)
        .iter()
        .map(|item| {
            return format!("{}:{} {}", item.file.display(), item.line, item.message)
                .replace(prefix.as_str(), "");
        })
        .collect::<Vec<String>>();
}

/// A `mod.rs` for the folder `src/a` whose top doc lists the given file names after the marker.
fn index_listing(names: &[&str], modules: &[&str]) -> String {
    let mut text: String = format!("//! Words about the folder.\n//!\n//! {LIST_MARKER}\n//!\n");
    for name in names {
        text.push_str(format!("//! - `{name}`: what it does.\n").as_str());
    }
    for module in modules {
        text.push_str(format!("mod {module};\n").as_str());
    }
    return text;
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

/// A top doc without the marker line has no list.
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
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].starts_with("src/a/mod.rs:1 The top doc of src/a/mod.rs has no file list"),
        "{}",
        found[0]
    );
}

/// A folder with module files and no `mod.rs` is a finding;
/// one with only an index and folders is not.
#[test]
fn a_folder_without_mod_rs_has_no_top_doc() {
    let fixture = package_with(&[
        ("src/lib.rs", index_listing(&["a.rs"], &["a"]).as_str()),
        ("src/a.rs", "mod inner;\n"),
        ("src/a/inner.rs", ""),
    ]);
    let found: Vec<String> = findings_of(&fixture.path);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("The folder src/a holds module files but no mod.rs"),
        "{}",
        found[0]
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

/// A use through a re-export from a child folder is a use of the child folder,
/// so it is no use between the files of this folder.
#[test]
fn a_reexport_from_a_child_folder_is_no_use_between_sibling_files() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        (
            "src/a/mod.rs",
            format!(
                "{}mod child;\npub use child::Thing;\n",
                index_listing(&["one.rs", "two.rs"], &["one", "two"])
            )
            .as_str(),
        ),
        ("src/a/one.rs", "use super::Thing;\n"),
        ("src/a/two.rs", "use super::one::X;\n"),
        (
            "src/a/child/mod.rs",
            format!(
                "{}pub use thing::Thing;\n",
                index_listing(&["thing.rs"], &["thing"])
            )
            .as_str(),
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

/// A `main.rs` beside a `lib.rs` is another crate root and is not read.
#[test]
fn an_entry_point_beside_a_library_root_is_not_read() {
    let fixture = package_with(&[
        ("src/lib.rs", "mod a;\n"),
        ("src/main.rs", "fn run() {}\nfn main() { run(); }\n"),
        ("src/a/mod.rs", ""),
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
