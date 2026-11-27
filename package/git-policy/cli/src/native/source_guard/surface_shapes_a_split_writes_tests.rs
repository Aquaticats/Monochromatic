//! The shapes a split writes that publish a name,
//! and the shapes this reading still cannot see:
//! a control of `surface_of` on a fixture tree for each one.
//! Why:
//!  The first version of the surface reader read one shape,
//!  a bare `pub` followed by a kind at the first column,
//!  and a split that wrote any other shape published a name with no line in the list
//!  (review M1:
//! a qualified function,
//! a name another module already published,
//! a field,
//! a method,
//!  and a `pub use` over several lines were all invisible).
//!  Each control here names one shape and shows what the reader answers for it,
//!  and the last control pins the shapes it still answers nothing for,
//!  so a later change to the reading has to change the reader's own doc with it.

use super::nameable_items_of_a_source_tree_tests::surface_of;
use crate::source_scan_tests::write_tree;
use crate::test_support::fixture;
use std::path::Path;

/// Every entry the reader answers for the fixture tree at `root`,
/// sorted:
/// the published items and the carried names alike.
fn keys_of(root: &Path) -> Vec<String> {
    let surface = surface_of(root).expect("a readable fixture tree");
    let mut keys: Vec<String> = surface
        .items
        .iter()
        .map(|item| return item.key.clone())
        .collect();
    keys.extend(surface.carried.iter().map(|site| return site.key.clone()));
    keys.sort();
    return keys;
}

/// A function is a name whatever qualifies its `fn`,
/// and the crate's own reach publishes nothing.
#[test]
fn a_qualified_function_is_seen_whatever_qualifies_it() {
    let root = fixture("surface-qualified-functions");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "pub async fn awaited() {}\n\
                 pub const fn constant() -> u8 {\n    0\n}\n\
                 pub unsafe fn unchecked() {}\n\
                 pub extern \"C\" fn called() {}\n\
                 pub(crate) async fn kept_inside() {}\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer fn awaited"),
            String::from("outer fn called"),
            String::from("outer fn constant"),
            String::from("outer fn unchecked"),
        ]
    );
}

/// Two modules that publish one kind and one name publish two names,
/// because a caller reaches each of them by its own path.
#[test]
fn a_name_another_module_holds_is_still_a_name_of_its_own() {
    let root = fixture("surface-one-name-two-modules");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod first;\npub mod second;\n"),
            ("first.rs", "pub fn subject_of() {}\n"),
            ("second.rs", "pub fn subject_of() {}\n"),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("first fn subject_of"),
            String::from("second fn subject_of"),
        ]
    );
}

/// A field of a type the chain publishes is a name,
/// and a field the crate alone reads,
/// one the test build alone holds,
/// or one of a type nothing publishes is not.
#[test]
fn a_field_is_seen_only_through_a_published_type() {
    let root = fixture("surface-fields");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "pub struct Published {\n\
                 \x20   pub seen: u8,\n\
                 \x20   pub(crate) kept_inside: u8,\n\
                 \x20   #[cfg(test)]\n\
                 \x20   pub only_in_the_test_build: u8,\n\
                 }\n\
                 struct Hidden {\n\
                 \x20   pub reached_by_nothing: u8,\n\
                 }\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer field Published::seen"),
            String::from("outer struct Published"),
        ]
    );
}

/// A method of a type the chain publishes is a name,
/// and a method of a type nothing publishes is not.
#[test]
fn a_method_is_seen_only_through_a_published_type() {
    let root = fixture("surface-methods");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "pub struct Published;\n\
                 impl Published {\n\
                 \x20   pub fn seen(&self) {}\n\
                 \x20   pub async fn seen_awaiting(&self) {}\n\
                 \x20   pub(crate) fn kept_inside(&self) {}\n\
                 }\n\
                 struct Hidden;\n\
                 impl Hidden {\n\
                 \x20   pub fn reached_by_nothing(&self) {}\n\
                 }\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer fn Published::seen"),
            String::from("outer fn Published::seen_awaiting"),
            String::from("outer struct Published"),
        ]
    );
}

/// A `pub use` carries one name per name it lists,
/// whether the statement is written on one line or over several,
/// and only out of a module the `pub mod` chain does not reach.
#[test]
fn a_re_export_carries_each_of_its_names() {
    let root = fixture("surface-carried-names");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\npub mod reached;\n"),
            (
                "outer.rs",
                "mod private;\n\
                 pub use self::private::{\n\
                 \x20   One,\n\
                 \x20   Two as Renamed,\n\
                 };\n\
                 pub use self::private::Three;\n\
                 pub use crate::reached::Open;\n\
                 pub use self::globbed::*;\n",
            ),
            (
                "outer/private.rs",
                "pub struct One;\npub struct Two;\npub struct Three;\n",
            ),
            ("outer/globbed.rs", "pub struct FromAGlob;\n"),
            ("reached.rs", "pub struct Open;\n"),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer pub use self::globbed::*"),
            String::from("outer pub use self::private::One"),
            String::from("outer pub use self::private::Renamed"),
            String::from("outer pub use self::private::Three"),
            String::from("reached struct Open"),
        ]
    );
}

/// A gate that holds only when testing keeps a name out of the surface,
/// whether it stands above an item or beside a member.
#[test]
fn a_test_gate_keeps_a_name_out_of_the_surface() {
    let root = fixture("surface-test-gates");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n#[cfg(test)]\npub mod support;\n"),
            (
                "outer.rs",
                "pub fn released() {}\n\
                 #[cfg(test)]\n\
                 pub const ONLY_IN_TESTS: u8 = 0;\n",
            ),
            ("support.rs", "pub fn reached_only_by_the_tests() {}\n"),
        ],
    );
    assert_eq!(keys_of(&root), vec![String::from("outer fn released")]);
}

/// What this reading cannot see stays out of the surface,
/// and each shape below is named in the reader's own doc with the reason it is out:
/// a trait's method,
/// an enum's variant,
/// a field written in a tuple-like struct's parentheses,
/// a member of a type published only through a `pub use` facade,
/// and the names a glob stands for.
#[test]
fn what_this_reading_cannot_see_stays_out_of_the_surface() {
    let root = fixture("surface-shapes-not-read");
    write_tree(
        &root,
        &[
            (
                "lib.rs",
                "pub mod outer;\nmod facade;\npub use self::facade::Carried;\n",
            ),
            (
                "outer.rs",
                "pub trait Asked {\n\
                 \x20   fn ask(&self);\n\
                 }\n\
                 pub enum Chosen {\n\
                 \x20   First,\n\
                 }\n\
                 pub struct Wrapped(pub u8);\n\
                 pub fn seen() {}\n",
            ),
            ("facade.rs", "pub struct Carried {\n    pub field: u8,\n}\n"),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("crate pub use self::facade::Carried"),
            String::from("outer enum Chosen"),
            String::from("outer fn seen"),
            String::from("outer struct Wrapped"),
            String::from("outer trait Asked"),
        ]
    );
}
