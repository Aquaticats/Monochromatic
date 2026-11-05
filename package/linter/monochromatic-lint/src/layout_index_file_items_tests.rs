//! What:
//!  Controls of the index file check.
//! Why:
//!  Each kind of item that an index file must not hold is shown with the line it is reported on,
//!  and the lines an index file may hold are shown to make no finding.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a function in mod.rs is reported on its line', () => expect(check('fn f() {}')).toEqual(['1 ...']));
//! ```

use super::{IndexRole, check_index_items};
use crate::layout_test_support::package_with;

/// Check the text as `src/a/mod.rs` in the given role,
/// and render each finding as `line message`.
fn findings_of(text: &str, role: IndexRole) -> Vec<String> {
    let fixture = package_with(&[("src/lib.rs", "mod a;\n"), ("src/a/mod.rs", text)]);
    return check_index_items(&fixture.path.join("src/a/mod.rs"), role, &fixture.path)
        .iter()
        .map(|item| {
            return format!(
                "{} {} | {}",
                item.line,
                item.message,
                item.help.clone().unwrap_or_default()
            );
        })
        .collect::<Vec<String>>();
}

/// Every kind of item other than a `mod` line and a `pub use` is reported once,
/// on the line of its first word after attributes and docs,
/// and the message names the kind and the name.
#[test]
fn every_kind_of_code_in_an_index_is_reported_with_its_kind() {
    let kinds: [(&str, &str); 17] = [
        ("fn f() {}", "a function `f`"),
        ("pub async fn f() {}", "a function `f`"),
        ("struct S;", "a struct `S`"),
        ("enum E { A }", "an enum `E`"),
        ("union U { a: u8 }", "a union `U`"),
        ("impl S {}", "an `impl` block"),
        ("trait T {}", "a trait `T`"),
        ("const C: u8 = 1;", "a constant `C`"),
        ("static V: u8 = 1;", "a static `V`"),
        ("type A = u8;", "a type alias `A`"),
        (
            "macro_rules! m { () => {} }",
            "a `macro_rules!` definition `m`",
        ),
        (
            "thread_local! { static X: u8 = 1; }",
            "a call of the macro `thread_local!`",
        ),
        ("use std::fmt::Display;", "a `use` that is not public"),
        ("pub(self) fn g() {}", "a function `g`"),
        ("extern crate alloc;", "an `extern crate` line"),
        ("extern \"C\" { fn g(); }", "an `extern` block"),
        ("mod inner { fn g() {} }", "an inline module `inner`"),
    ];
    for (source, what) in kinds {
        let found: Vec<String> = findings_of(source, IndexRole::Index);
        assert_eq!(found.len(), 1, "{source}: {found:#?}");
        assert!(
            found[0].starts_with("1 src/a/mod.rs is the index of its folder"),
            "{source}: {}",
            found[0]
        );
        assert!(
            found[0].contains(format!("but it holds {what}.").as_str()),
            "{source}: {}",
            found[0]
        );
        assert!(
            found[0].contains("Move it into a file of this folder"),
            "{source}: {}",
            found[0]
        );
    }
}

/// The line of a finding is the line of the item,
/// not the line of the attribute or doc above it.
#[test]
fn the_line_is_the_items_and_not_its_attributes() {
    let found: Vec<String> = findings_of(
        "//! Words.\n\n/// Documented.\n#[derive(Debug)]\npub struct S;\n",
        IndexRole::Index,
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].starts_with("5 "), "{}", found[0]);
}

/// Each item is its own finding.
#[test]
fn every_item_is_its_own_finding() {
    let found: Vec<String> = findings_of(
        "fn a() {}\nstruct B;\nmod ok;\nenum C { D }\n",
        IndexRole::Index,
    );
    assert_eq!(found.len(), 3, "{found:#?}");
}

/// The top doc,
/// inner attributes,
/// comments,
/// `mod` lines with every kind of attribute and doc,
/// and re-exports of every visibility are the whole of an index.
#[test]
fn the_top_doc_mod_lines_and_reexports_make_no_finding() {
    let text: &str = "//! Words about the folder.\n\
#![allow(clippy::module_inception)]\n\
// An ordinary comment.\n\
mod plain;\n\
pub mod public;\n\
pub(crate) mod restricted;\n\
/// A doc on a mod line.\n\
mod documented;\n\
#[path = \"moved_file.rs\"]\n\
mod moved;\n\
#[cfg(test)]\n\
#[path = \"controls.rs\"]\n\
mod tests;\n\
#[macro_use]\n\
mod macros;\n\
#[cfg(unix)]\n\
mod unix_only;\n\
pub use plain::Thing;\n\
pub(crate) use public::{Other, Third as Renamed};\n\
pub(super) use restricted::*;\n\
pub use documented::Item as _;\n";
    assert_eq!(findings_of(text, IndexRole::Index), Vec::<String>::new());
}

/// Code marked for test builds is outside the check,
/// and the negated form is production code.
#[test]
fn code_for_test_builds_is_not_checked() {
    let text: &str = "#[cfg(test)]\nfn helper() {}\n\
#[cfg(test)]\nuse std::fmt::Display;\n\
#[cfg(test)]\nmod tests { fn t() {} }\n\
#[test]\nfn a_test() {}\n\
#[cfg(not(test))]\nfn real() {}\n";
    let found: Vec<String> = findings_of(text, IndexRole::Index);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].starts_with("10 "), "{}", found[0]);
    assert!(found[0].contains("a function `real`"), "{}", found[0]);
}

/// An entry point may hold `fn main` and the `use` lines it needs beside the index,
/// and `fn main` under any attribute or `async`.
#[test]
fn an_entry_point_may_hold_fn_main_and_its_uses() {
    let text: &str = "//! The program.\n\
use std::process::ExitCode;\n\
mod run;\n\
pub use run::Thing;\n\
#[tokio::main]\n\
async fn main() -> ExitCode { run::run().await }\n";
    assert_eq!(
        findings_of(text, IndexRole::EntryPoint),
        Vec::<String>::new()
    );
}

/// Anything else in an entry point is reported in the entry point's own words.
#[test]
fn other_code_in_an_entry_point_is_reported() {
    let found: Vec<String> = findings_of(
        "fn run() {}\nfn main() { run(); }\nstruct S;\nconst C: u8 = 1;\nmod inner { }\n",
        IndexRole::EntryPoint,
    );
    assert_eq!(found.len(), 4, "{found:#?}");
    assert!(
        found[0].starts_with("1 src/a/mod.rs is the entry point of its program"),
        "{}",
        found[0]
    );
    assert!(
        found[0].contains("`mod` lines, `use` lines and `fn main`"),
        "{}",
        found[0]
    );
    assert!(
        found[0].contains("but it holds a function `run`."),
        "{}",
        found[0]
    );
    assert!(found[0].contains("call it from `fn main`"), "{}", found[0]);
}

/// `fn main` is code like any other in an index that is not an entry point.
#[test]
fn fn_main_in_an_ordinary_index_is_reported() {
    let found: Vec<String> = findings_of("fn main() {}\n", IndexRole::Index);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("a function `main`"), "{}", found[0]);
}

/// A file that cannot be read makes no finding here,
/// because the module tree already failed the run for it.
#[test]
fn an_unreadable_file_makes_no_finding() {
    let fixture = package_with(&[("src/lib.rs", "")]);
    let found = check_index_items(
        &fixture.path.join("src/missing.rs"),
        IndexRole::Index,
        &fixture.path,
    );
    assert!(found.is_empty());
}
