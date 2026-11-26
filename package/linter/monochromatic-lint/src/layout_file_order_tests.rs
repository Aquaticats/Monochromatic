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
        found[0].contains("The folder src/a holds module files but no mod.rs"),
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
    let list = crate::layout_allow_list::parse_allow_list(lines, Path::new("allow-list.txt"))
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
    // The second extra copy gains a number, so a line for the first one names one finding.
    assert_eq!(copies[1].key, "listed-twice src/a/mod.rs one.rs#2");
    let list = crate::layout_allow_list::parse_allow_list(
        "listed-twice src/a/mod.rs one.rs\n",
        Path::new("allow-list.txt"),
    )
    .expect("the fixture list is valid");
    let kept: Vec<String> = check_file_order(&tree, uses.as_slice(), &fixture.path, Some(&list))
        .iter()
        .map(|item| return item.key.clone())
        .collect::<Vec<String>>();
    assert_eq!(kept, ["listed-twice src/a/mod.rs one.rs#2"], "{kept:#?}");
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
