//! What:
//!  Controls of the module-path reader.
//! Why:
//!  The reader decides which modules a file uses;
//!  a missed path hides a dependency and an invented one blames innocent code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('groups name every leaf', () => expect(referencesOf('use crate::{a, b::{c, d}};')).toEqual([...]));
//! ```

use crate::layout_test_support::{aliases_of, references_of};
use crate::test_deadline::finished_within_deadline;

/// A plain path is one reference with every segment.
#[test]
fn a_plain_crate_path_is_one_reference() {
    assert_eq!(
        references_of("fn f() { crate::a::b::run(); }"),
        ["crate::a::b::run"]
    );
}

/// Groups,
/// nested groups and `self` name every leaf with its whole path.
#[test]
fn groups_name_every_leaf_with_its_whole_path() {
    assert_eq!(
        references_of("use crate::{a, b::{c, d::{self, e}}};"),
        ["crate::a", "crate::b::c", "crate::b::d", "crate::b::d::e"]
    );
}

/// A rename gives a new binding,
/// which is not a module path.
#[test]
fn a_rename_is_not_a_path() {
    assert_eq!(
        references_of("use crate::a::{b as c, d};\nuse super::x as y;"),
        ["crate::a::b", "crate::a::d", "super::x"]
    );
}

/// A re-export keeps the new name after `as`,
/// on a plain path and on a leaf of a group,
/// and the discard name `_` too.
#[test]
fn a_reexport_keeps_its_rename() {
    assert_eq!(
        aliases_of(
            "pub use crate::a::{b as c, d, e::{self as f}};\npub use super::x as y;\npub(crate) use self::z as _;"
        ),
        [
            "pub crate::a::b as c",
            "pub crate::a::e as f",
            "pub super::x as y",
            "pub self::z as _"
        ]
    );
}

/// A rename of a private `use` or a cast of a value is not a re-export name.
#[test]
fn only_a_reexport_keeps_its_rename() {
    assert_eq!(
        aliases_of("use crate::a::b as c;\nfn f() -> usize { return crate::a::LIMIT as usize; }"),
        Vec::<String>::new()
    );
}

/// A glob keeps its prefix and is marked.
#[test]
fn a_glob_keeps_its_prefix_and_is_marked() {
    assert_eq!(
        references_of("use crate::a::*;\nuse super::super::*;"),
        ["crate::a::*", "super::super::*"]
    );
}

/// A `pub use` of any visibility marks its references as re-exports,
/// and a private `use` does not.
#[test]
fn only_public_uses_are_reexports() {
    assert_eq!(
        references_of(
            "pub use crate::a::*;\nuse crate::b::*;\npub(crate) use crate::c::D;\npub(super) use super::e::{F, G};"
        ),
        [
            "pub crate::a::*",
            "crate::b::*",
            "pub crate::c::D",
            "pub super::e::F",
            "pub super::e::G"
        ]
    );
}

/// Comments,
/// doc comments,
/// strings of every kind and character literals hold no code;
/// a double quote character literal must not swallow the code after it.
#[test]
fn comments_strings_and_character_literals_are_not_code() {
    let source: &str = r##"
// crate::in_line_comment::X
/// crate::in_doc_comment::X
/* crate::in_block_comment::X /* crate::nested::X */ */
const A: &str = "crate::in_string::X";
const B: &str = r#"crate::in_raw_string::X "quoted""#;
const C: &[u8] = b"crate::in_byte_string::X";
const D: &[u8] = br#"crate::in_raw_byte_string::X"#;
const E: char = '"';
const F: char = '\'';
fn real() { crate::real::run(); }
const G: &str = "after the quote characters";
#[doc = "crate::in_attribute::X"]
fn documented() {}
"##;
    assert_eq!(references_of(source), ["crate::real::run"]);
}

/// Paths inside a macro call's tokens are read,
/// and `$crate` of a macro definition is not a path of the file.
#[test]
fn paths_inside_macro_calls_count_and_macro_variables_do_not() {
    let source: &str = "
fn f() { println!(\"{}\", crate::inside::value()); assert!(super::other::ready()); }
macro_rules! m { () => { $crate::defined::x() }; }
";
    assert_eq!(
        references_of(source),
        ["crate::inside::value", "super::other::ready"]
    );
}

/// A raw identifier names the module without its prefix.
#[test]
fn raw_identifiers_lose_their_prefix() {
    assert_eq!(
        references_of("fn f() { crate::r#type::run(); }"),
        ["crate::type::run"]
    );
}

/// A visibility path,
/// a method turbofish and a field access are not module paths.
#[test]
fn visibility_restrictions_and_member_accesses_are_not_paths() {
    let source: &str = "
pub(in crate::restricted) fn f() {}
fn g(value: Wrapper) { value.process::<u8>(); value.field.nested::<u8>(); }
";
    assert_eq!(references_of(source), Vec::<String>::new());
}

/// Code marked for test builds is skipped with everything inside it,
/// and the negated form is production code.
#[test]
fn test_only_code_is_skipped() {
    let source: &str = "
#[cfg(test)]
mod tests { use crate::in_inline_test_module::X; }
#[cfg(test)]
use crate::in_test_use::Y;
#[test]
fn a_test() { crate::in_test_function::run(); }
#[cfg(all(test, unix))]
fn helper() { crate::in_cfg_all::run(); }
#[cfg(not(test))]
use crate::production::Z;
";
    assert_eq!(references_of(source), ["crate::production::Z"]);
}

/// A plain name followed by `::` is read as a possible module,
/// and a path continued from an earlier `::` is not a second path.
#[test]
fn a_leading_name_is_kept_and_a_continued_path_is_not_split() {
    assert_eq!(
        references_of(
            "use child::Thing;\nfn f() { std::mem::swap(a, b); <T as crate::tr::Tr>::run(); }"
        ),
        ["child::Thing", "std::mem::swap", "crate::tr::Tr"]
    );
}

/// An entry that follows a nested group is still read,
///  and an entry before it is not lost.
#[test]
fn layout_entries_after_a_nested_group_are_still_read() {
    assert_eq!(
        references_of("use crate::{a, b::{c, d}, e};"),
        ["crate::a", "crate::b::c", "crate::b::d", "crate::e"]
    );
}

/// A group that the source never closes ends with the source,
///  and reading it must not stand still.
#[test]
fn layout_a_group_that_is_never_closed_ends_with_the_source() {
    let found: Vec<String> =
        finished_within_deadline(|| return references_of("use crate::{a, b::{c, d"));
    assert_eq!(found, ["crate::a", "crate::b::c", "crate::b::d"]);
}

/// Group entries of several segments and group entries that end in a glob are read whole,
///  and a glob entry adds no glob of the prefix.
#[test]
fn layout_group_entries_of_several_segments_and_globs_are_read_whole() {
    assert_eq!(
        references_of("use crate::{a::b::c, d::e, f::*, g::{h::*}};"),
        [
            "crate::a::b::c",
            "crate::d::e",
            "crate::f::*",
            "crate::g::h::*"
        ]
    );
}

/// A path that starts with `self` and continues keeps its `self` segment inside a group,
///  where a lone `self` names the prefix itself.
#[test]
fn layout_a_self_path_inside_a_group_keeps_its_self_segment() {
    assert_eq!(references_of("use crate::{self::b};"), ["crate::self::b"]);
}

/// A path that starts exactly where a public `use` item ends is not part of that item,
///  so it is not a re-export.
#[test]
fn layout_a_path_right_after_a_public_use_is_not_a_reexport() {
    assert_eq!(
        references_of("pub use a::B;crate::x::y!();"),
        ["pub a::B", "crate::x::y"]
    );
}
