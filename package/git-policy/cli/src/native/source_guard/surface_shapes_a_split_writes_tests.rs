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
//!  and `what_this_reading_cannot_see_stays_out_of_the_surface` pins the shapes it still answers nothing for,
//!  so a later change to the reading has to change the reader's own doc with it.
//!  Three shapes the reading answers for are pinned beside them:
//!  a `mod` line it cannot place stops the scan rather than shortening the tree in silence,
//!  a member of a type a `pub use` publishes is read at the facade path,
//!  and a name behind a feature gate is read as a name of the release surface.

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

/// A constant of a type the chain publishes is a name,
/// and a constant of a type nothing publishes is not.
#[test]
fn a_constant_inside_a_published_types_impl_is_a_name() {
    let root = fixture("surface-associated-constants");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "pub struct Published;\n\
                 impl Published {\n\
                 \x20   pub const ALL: &[Published] = &[Published];\n\
                 \x20   pub const NONE: Published = Published;\n\
                 \x20   pub(crate) const KEPT_INSIDE: u8 = 0;\n\
                 \x20   pub fn seen(&self) {}\n\
                 }\n\
                 struct Hidden;\n\
                 impl Hidden {\n\
                 \x20   pub const REACHED_BY_NOTHING: u8 = 0;\n\
                 }\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer const Published::ALL"),
            String::from("outer const Published::NONE"),
            String::from("outer fn Published::seen"),
            String::from("outer struct Published"),
        ]
    );
}

/// A type or an `impl` whose header runs over several lines publishes its members all the same,
/// and one whose header is written on one line is read the same way.
#[test]
fn a_header_written_over_several_lines_still_publishes_its_members() {
    let root = fixture("surface-headers-over-lines");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "pub struct Published<\n\
                 \x20   'a,\n\
                 \x20   T: Clone = u8,\n\
                 > {\n\
                 \x20   pub held: T,\n\
                 \x20   pub stated: &'a str,\n\
                 \x20   pub(crate) kept_inside: u8,\n\
                 }\n\
                 impl<'a> Published<'a, u8>\n\
                 where\n\
                 \x20   u8: Copy,\n\
                 {\n\
                 \x20   pub fn seen(&self) -> u8 {\n\
                 \x20       return self.held;\n\
                 \x20   }\n\
                 }\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer field Published::held"),
            String::from("outer field Published::stated"),
            String::from("outer fn Published::seen"),
            String::from("outer struct Published"),
        ]
    );
}

/// A function is a name whatever count of qualifiers stands before its `fn`,
/// and two such functions of one module are two names.
#[test]
fn a_function_qualified_more_than_once_is_still_a_function() {
    let root = fixture("surface-two-qualifiers");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "pub const unsafe fn unchecked_one() {}\n\
                 pub const unsafe fn unchecked_two() {}\n\
                 pub const extern \"C\" fn called() {}\n\
                 pub unsafe extern \"C\" fn called_too() {}\n\
                 pub const KEPT: u8 = 0;\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer const KEPT"),
            String::from("outer fn called"),
            String::from("outer fn called_too"),
            String::from("outer fn unchecked_one"),
            String::from("outer fn unchecked_two"),
        ]
    );
}

/// An item a gate keeps out of the test build alone is published,
/// and one a gate keeps out of the release build is not,
/// whatever shape the predicate is written in.
#[test]
fn a_gate_that_names_the_test_inside_a_negation_leaves_the_item_published() {
    let root = fixture("surface-negated-test-gates");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "#[cfg(all(not(test), unix))]\n\
                 pub fn released_on_unix() {}\n\
                 #[cfg(any(test, unix))]\n\
                 pub fn released_too() {}\n\
                 #[cfg(all(test, unix))]\n\
                 pub fn only_in_the_test_build() {}\n\
                 #[cfg(test)]\n\
                 pub fn only_for_tests() {}\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer fn released_on_unix"),
            String::from("outer fn released_too"),
        ]
    );
}

/// A `pub use` that gives a name another spelling publishes that spelling,
/// and a re-export that keeps the name publishes nothing the chain does not already reach.
#[test]
fn a_re_export_under_another_name_is_a_name_of_its_own() {
    let root = fixture("surface-renamed-re-exports");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\npub mod reached;\n"),
            (
                "outer.rs",
                "pub use crate::reached::Open as Renamed;\n\
                 pub use crate::reached::Kept;\n",
            ),
            ("reached.rs", "pub struct Open;\npub struct Kept;\n"),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer pub use crate::reached::Renamed"),
            String::from("reached struct Kept"),
            String::from("reached struct Open"),
        ]
    );
}

/// A bare `pub use` is read from the module that writes it,
/// so a private child of that module is not a public module of another folder.
#[test]
fn a_bare_re_export_is_read_from_the_module_that_writes_it() {
    let root = fixture("surface-bare-re-export");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\npub mod elsewhere;\n"),
            ("outer.rs", "mod subject;\npub use subject::Held;\n"),
            ("outer/subject.rs", "pub struct Held;\n"),
            ("elsewhere.rs", "pub mod subject;\n"),
            ("elsewhere/subject.rs", "pub struct Other;\n"),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("elsewhere::subject struct Other"),
            String::from("outer pub use subject::Held"),
        ]
    );
}

/// What this reading cannot see stays out of the surface,
/// and each shape below is named in the reader's own doc with the reason it is out:
/// a trait's method,
/// an enum's variant,
/// and a field written in a tuple-like struct's parentheses.
#[test]
fn what_this_reading_cannot_see_stays_out_of_the_surface() {
    let root = fixture("surface-shapes-not-read");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
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
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer enum Chosen"),
            String::from("outer fn seen"),
            String::from("outer struct Wrapped"),
            String::from("outer trait Asked"),
        ]
    );
}

/// A `mod` line this reading cannot place stops the scan,
/// rather than leaving its module out of the tree in silence.
/// Why:
///  a skipped declaration shortens the surface by every name its module publishes,
///  and the reading's own doc says such a line is an error;
///  these are the three shapes no reader of this file places,
///  and the tree holds none of them on 2026-11-27.
#[test]
fn a_mod_line_the_reading_cannot_place_stops_the_scan() {
    for (name, declaration) in [
        (
            "surface-unreadable-mod-restricted",
            "pub(in crate::outer) mod inner;",
        ),
        ("surface-unreadable-mod-self", "pub(self) mod inner;"),
        (
            "surface-unreadable-mod-comment",
            "mod inner; // the module a split made",
        ),
    ] {
        let root = fixture(name);
        let outer: String = format!("{declaration}\npub fn seen() {{}}\n");
        write_tree(
            &root,
            &[
                ("lib.rs", "pub mod outer;\n"),
                ("outer.rs", outer.as_str()),
                ("outer/inner.rs", "pub fn kept_inside() {}\n"),
            ],
        );
        match surface_of(&root) {
            Ok(surface) => panic!(
                "`{declaration}` is no declaration this reading places, so the scan must stop; it answered {} item(s)",
                surface.items.len()
            ),
            Err(error) => assert!(
                error.contains(declaration),
                "the error names the line it could not place: {error}"
            ),
        }
    }
}

/// A `pub` member of a type a `pub use` publishes is a name a consumer reaches,
/// so the reader keys it by the path the facade gives the type.
#[test]
fn a_member_of_a_type_a_facade_publishes_is_read_at_the_facade_path() {
    let root = fixture("surface-members-of-carried-types");
    write_tree(
        &root,
        &[
            (
                "lib.rs",
                "pub mod outer;\nmod facade;\npub use self::facade::Carried;\n",
            ),
            ("outer.rs", "pub enum Reached {\n    First,\n}\n"),
            (
                "facade.rs",
                "pub struct Carried {\n\
                 \x20   pub field: u8,\n\
                 }\n\
                 impl Carried {\n\
                 \x20   pub fn method(&self) -> u8 {\n\
                 \x20       return self.field;\n\
                 \x20   }\n\
                 }\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("crate field Carried::field"),
            String::from("crate fn Carried::method"),
            String::from("crate pub use self::facade::Carried"),
            String::from("outer enum Reached"),
        ]
    );
}

/// A facade may name its module through the glob the crate root re-exports,
/// which is how `crate::the_guarded_commands` names `command::the_guarded_commands`.
#[test]
fn a_member_of_a_type_a_facade_reaches_through_the_roots_glob_is_read() {
    let root = fixture("surface-facade-through-the-roots-glob");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod command;\npub use command::*;\n"),
            ("command/mod.rs", "pub mod facade;\npub(crate) mod inner;\n"),
            ("command/facade.rs", "pub use crate::inner::Thing;\n"),
            (
                "command/inner.rs",
                "pub struct Thing {\n    pub field: u8,\n}\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("command::facade field Thing::field"),
            String::from("command::facade pub use crate::inner::Thing"),
        ]
    );
}

/// Members are matched by a type's bare name,
/// so a private type that shares a published type's name has its `pub` members read as published.
/// Why:
///  the reader keys a type by the name its declaration writes,
///  so this is a limit of the reading and not a name a consumer reaches.
#[test]
fn a_private_type_sharing_a_published_types_name_has_its_members_read() {
    let root = fixture("surface-same-name-in-two-modules");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "pub struct Shared {\n    pub seen: u8,\n}\nmod inside;\n",
            ),
            (
                "outer/inside.rs",
                "pub struct Shared {\n    pub reached_by_no_one: u8,\n}\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer field Shared::seen"),
            String::from("outer struct Shared"),
            String::from("outer::inside field Shared::reached_by_no_one"),
        ]
    );
}

/// A macro the crate publishes with `#[macro_export(local_inner_macros)]` is a name,
/// whatever stands inside the attribute's parentheses.
/// Why:
///  the attribute was read as the exact text `#[macro_export]`,
///  so this form published a macro with no line in the list beside this reader
///  (the alpha.52 read's L1).
#[test]
fn a_macro_export_with_parentheses_is_a_name() {
    let root = fixture("surface-macro-export-with-parentheses");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "#[macro_export]\n\
                 macro_rules! plain {\n\
                 \x20   () => {};\n\
                 }\n\
                 #[macro_export(local_inner_macros)]\n\
                 macro_rules! local {\n\
                 \x20   () => { plain!() };\n\
                 }\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer macro_rules local"),
            String::from("outer macro_rules plain"),
        ]
    );
}

/// An attribute and a `pub` item written on one line publish the item,
/// and a test gate on that line still keeps it out of the surface.
/// Why:
///  a line that started with `#[` was taken for an attribute and never read as a declaration,
///  so the item it carried published a name with no line
///  (the alpha.52 read's L1).
#[test]
fn an_item_on_its_attributes_line_is_a_name() {
    let root = fixture("surface-item-on-its-attribute-line");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "#[inline] pub fn inlined() {}\n\
                 #[allow(dead_code)] pub struct Held;\n\
                 #[macro_export] macro_rules! exported {\n\
                 \x20   () => {};\n\
                 }\n\
                 #[cfg(test)] pub fn only_in_the_test_build() {}\n\
                 pub fn plain() {}\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer fn inlined"),
            String::from("outer fn plain"),
            String::from("outer macro_rules exported"),
            String::from("outer struct Held"),
        ]
    );
}

/// An `impl` whose own generics run over several lines publishes the members of its type,
/// which is the header `rustfmt` writes for a long generic list.
/// Why:
///  the type was read from the `impl` line alone,
///  so a header with no `>` on it named no type and its body was not read
///  (the alpha.52 read's L1).
#[test]
fn an_impl_whose_generics_run_over_several_lines_publishes_its_members() {
    let root = fixture("surface-impl-generics-over-lines");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "pub struct Published<\n\
                 \x20   'a,\n\
                 \x20   T: Clone,\n\
                 > {\n\
                 \x20   pub held: &'a T,\n\
                 }\n\
                 impl<\n\
                 \x20   'a,\n\
                 \x20   T: Clone,\n\
                 > Published<'a, T> {\n\
                 \x20   pub fn seen(&self) -> &'a T {\n\
                 \x20       return self.held;\n\
                 \x20   }\n\
                 }\n\
                 struct Hidden<T> {\n\
                 \x20   held: T,\n\
                 }\n\
                 impl<\n\
                 \x20   T: Clone,\n\
                 > Hidden<T> {\n\
                 \x20   pub fn reached_by_nothing(&self) {}\n\
                 }\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![
            String::from("outer field Published::held"),
            String::from("outer fn Published::seen"),
            String::from("outer struct Published"),
        ]
    );
}

/// A name behind a feature gate is published by the build that turns the feature on,
/// and a gate keeps a name out only where it holds only when testing.
#[test]
fn a_name_behind_a_feature_gate_is_published() {
    let root = fixture("surface-feature-gated-names");
    write_tree(
        &root,
        &[
            ("lib.rs", "pub mod outer;\n"),
            (
                "outer.rs",
                "#[cfg(feature = \"a-build-that-turns-it-on\")]\npub fn only_with_the_feature() {}\n\
                 #[cfg(test)]\npub fn only_in_the_test_build() {}\n",
            ),
        ],
    );
    assert_eq!(
        keys_of(&root),
        vec![String::from("outer fn only_with_the_feature")]
    );
}
