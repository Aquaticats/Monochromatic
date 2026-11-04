//! What:
//!  Controls of the module-body reader.
//! Why:
//!  The module tree is only as right as the `mod` lines this reader finds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('an inline module is recorded and not entered', () => { /* ... */ });
//! ```

use super::{BodyFacts, collect_body_facts};
use crate::{layout_test_support::describe, rust_source::RustSource};

/// Read the facts of a source text.
fn facts_of(source: &str) -> BodyFacts {
    let parsed: RustSource = RustSource::new(String::from("fixture.rs"), String::from(source));
    return collect_body_facts(parsed.syntax());
}

/// External and inline modules are recorded in order with their path attributes,
/// and the references inside an inline body are left to it.
#[test]
fn an_inline_module_is_recorded_and_not_entered() {
    let facts: BodyFacts = facts_of(
        "mod outer;\n#[path = \"../moved.rs\"]\nmod moved;\nmod inline { use crate::inside::X; mod deeper; }\nuse crate::beside::Y;\n",
    );
    let names: Vec<&str> = facts.declarations.iter().map(|found| return found.name.as_str()).collect::<Vec<&str>>();
    assert_eq!(names, ["outer", "moved", "inline"]);
    assert_eq!(facts.declarations[1].path_attribute.as_deref(), Some("../moved.rs"));
    assert!(facts.declarations[0].body.is_none());
    assert!(facts.declarations[2].body.is_some());
    let shown: Vec<String> = facts.references.iter().map(describe).collect::<Vec<String>>();
    assert_eq!(shown, ["crate::beside::Y"]);
}

/// A raw string path attribute and a raw module name are read.
#[test]
fn raw_forms_of_the_name_and_the_path_attribute_are_read() {
    let facts: BodyFacts = facts_of("#[path = r\"raw/place.rs\"]\nmod r#type;\n");
    assert_eq!(facts.declarations[0].name, "type");
    assert_eq!(facts.declarations[0].path_attribute.as_deref(), Some("raw/place.rs"));
}

/// A module marked for test builds is not recorded,
/// and a `cfg` that is not about tests is.
#[test]
fn test_only_modules_are_not_recorded() {
    let facts: BodyFacts = facts_of(
        "#[cfg(test)]\n#[path = \"x_tests.rs\"]\nmod tests;\n#[cfg(feature = \"test\")]\nmod featured;\n#[cfg(all(test, unix))]\nmod also_tests;\n",
    );
    let names: Vec<&str> = facts.declarations.iter().map(|found| return found.name.as_str()).collect::<Vec<&str>>();
    assert_eq!(names, ["featured"]);
}

/// Lines count from the first token of the path.
#[test]
fn offsets_locate_the_first_token_of_each_path() {
    let source: &str = "fn f() {}\nuse crate::second::Line;\n";
    let facts: BodyFacts = facts_of(source);
    assert_eq!(facts.references[0].offset, source.find("crate").expect("fixture text"));
}
