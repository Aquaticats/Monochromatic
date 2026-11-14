//! Unit tests of appending entries to a plan's text:
//! what the plan looks like after the write,
//! and what is refused before one.
//! Each test works on its own text in memory;
//! nothing here reads or writes a real file.

use super::model::{Declaration, Options, Request, Route};
use super::plan::parse_plan;
use super::plan_insert::plan_with_entries;

/// A plan the way people write one:
/// a comment above an entry,
/// a comment after the last one,
/// trailing commas,
/// and an order of its own that no alphabetical sort would give.
const PLAN: &str = r#"// The plan of this fixture: people write these comments.
{
  "schemaVersion": 1,
  "tools": {
    // tokei counts lines; the numbers are compared while it is pinned.
    "tokei": {"request": "cargo:tokei@latest", "route": "cargo-binstall", "bins": ["tokei"]},
    "cargo:fd-find": {
      "request": "cargo:fd-find@latest",
      "route": "cargo-binstall",
      "bins": [
        "fd"
      ]
    },
    // Nothing follows this note; it is here to be kept.
  },
}
"#;

/// One declaration as an argument makes it.
fn declaration(id: &str, request: &str, route: Route, bins: &[&str]) -> Declaration {
    let mut names: Vec<String> = Vec::new();
    for bin in bins {
        names.push((*bin).to_string());
    }
    return Declaration {
        id: id.to_string(),
        request: Request::parse(request).expect("a request"),
        route,
        manager: None,
        package_id: None,
        bins: names,
        os: None,
        env: Vec::new(),
        options: Options::default(),
    };
}

/// `cargo:fastmod` on the cargo-binstall route.
fn fastmod() -> Declaration {
    return declaration("cargo:fastmod", "cargo:fastmod@latest", Route::CargoBinstall, &["fastmod"]);
}

#[test]
fn an_added_entry_keeps_every_comment_trailing_comma_and_the_plans_own_order() {
    let updated = plan_with_entries(PLAN, "plan", &[fastmod()]).expect("the entry is added");
    let expected = r#"// The plan of this fixture: people write these comments.
{
  "schemaVersion": 1,
  "tools": {
    // tokei counts lines; the numbers are compared while it is pinned.
    "tokei": {"request": "cargo:tokei@latest", "route": "cargo-binstall", "bins": ["tokei"]},
    "cargo:fd-find": {
      "request": "cargo:fd-find@latest",
      "route": "cargo-binstall",
      "bins": [
        "fd"
      ]
    },
    "cargo:fastmod": {
      "request": "cargo:fastmod@latest",
      "route": "cargo-binstall",
      "bins": [
        "fastmod"
      ]
    },
    // Nothing follows this note; it is here to be kept.
  },
}
"#;
    assert_eq!(updated, expected);
    let plan = parse_plan(&updated, "plan").expect("the new text parses");
    let ids: Vec<&str> = plan.tools.iter().map(|tool| return tool.id.as_str()).collect();
    assert_eq!(ids, vec!["tokei", "cargo:fd-find", "cargo:fastmod"], "the entry order the person chose is kept, and the new one is last");
    let added = plan.tools.iter().find(|tool| return tool.id == "cargo:fastmod").expect("the new entry");
    assert_eq!(added.request.text(), "cargo:fastmod@latest");
    assert_eq!(added.route, Route::CargoBinstall);
    assert_eq!(added.bins, vec!["fastmod".to_string()]);
}

#[test]
fn two_entries_in_one_call_land_in_the_order_they_were_given() {
    let second = declaration("npm:is-odd", "npm:is-odd@latest", Route::MpmPnpm, &["is-odd"]);
    let updated = plan_with_entries(PLAN, "plan", &[fastmod(), second]).expect("both are added");
    let plan = parse_plan(&updated, "plan").expect("the new text parses");
    let ids: Vec<&str> = plan.tools.iter().map(|tool| return tool.id.as_str()).collect();
    assert_eq!(ids, vec!["tokei", "cargo:fd-find", "cargo:fastmod", "npm:is-odd"]);
}

#[test]
fn an_empty_tools_object_gets_the_entry_inside_it_and_keeps_its_indentation() {
    let empty = "{\n  \"schemaVersion\": 1,\n  \"tools\": {}\n}\n";
    let updated = plan_with_entries(empty, "plan", &[fastmod()]).expect("the entry is added");
    let expected = "{\n  \"schemaVersion\": 1,\n  \"tools\": {\n    \"cargo:fastmod\": {\n      \"request\": \"cargo:fastmod@latest\",\n      \"route\": \"cargo-binstall\",\n      \"bins\": [\n        \"fastmod\"\n      ]\n    }\n  }\n}\n";
    assert_eq!(updated, expected);
    assert_eq!(parse_plan(&updated, "plan").expect("parses").tools.len(), 1);
}

#[test]
fn an_empty_tools_object_whose_braces_are_on_their_own_lines_keeps_them_there() {
    let empty = "{\n  \"schemaVersion\": 1,\n  \"tools\": {\n  }\n}\n";
    let updated = plan_with_entries(empty, "plan", &[fastmod()]).expect("the entry is added");
    let expected = "{\n  \"schemaVersion\": 1,\n  \"tools\": {\n    \"cargo:fastmod\": {\n      \"request\": \"cargo:fastmod@latest\",\n      \"route\": \"cargo-binstall\",\n      \"bins\": [\n        \"fastmod\"\n      ]\n    }\n  }\n}\n";
    assert_eq!(updated, expected);
    assert_eq!(parse_plan(&updated, "plan").expect("parses").tools.len(), 1);
}

#[test]
fn a_plan_whose_last_entry_pins_a_version_keeps_the_comma_the_person_wrote() {
    let plan = "{\n  \"schemaVersion\": 1,\n  \"tools\": {\n    \"cargo:tokei\": {\"request\": \"cargo:tokei@1.0.0\", \"route\": \"cargo-binstall\", \"bins\": [\"tokei\"]}\n  }\n}\n";
    let updated = plan_with_entries(plan, "plan", &[fastmod()]).expect("the entry is added");
    assert!(updated.contains("{\"request\": \"cargo:tokei@1.0.0\", \"route\": \"cargo-binstall\", \"bins\": [\"tokei\"]},\n    \"cargo:fastmod\""), "{updated}");
    assert_eq!(updated.matches("\"cargo:fastmod\"").count(), 1, "the entry appears once: {updated}");
    assert_eq!(parse_plan(&updated, "plan").expect("parses").tools.len(), 2);
}

#[test]
fn an_entry_whose_binary_name_is_taken_is_refused_and_no_text_comes_back() {
    let collides = declaration("cargo:another-tokei", "cargo:another-tokei@latest", Route::CargoBinstall, &["tokei"]);
    let error = plan_with_entries(PLAN, "plan", &[collides]).expect_err("the binary name is taken");
    assert!(error.message.contains("tokei"), "{error}");
    assert!(error.message.contains("declared by both"), "{error}");
}

#[test]
fn an_entry_whose_id_is_taken_is_refused_and_no_text_comes_back() {
    let twice = declaration("tokei", "cargo:tokei2@latest", Route::CargoBinstall, &["tokei2"]);
    let error = plan_with_entries(PLAN, "plan", &[twice]).expect_err("the id is taken");
    assert!(error.message.contains("\"tokei\""), "{error}");
}

#[test]
fn a_plan_without_a_tools_object_is_refused_by_name() {
    let error = plan_with_entries("{\n  \"schemaVersion\": 1\n}\n", "plan", &[fastmod()]).expect_err("no tools");
    assert!(error.message.contains("no \"tools\" object"), "{error}");
}

#[test]
fn a_plan_that_does_not_parse_is_refused_before_anything_is_added() {
    let error = plan_with_entries("{\n  \"schemaVersion\": 1,\n  \"tools\": {\n", "plan", &[fastmod()]).expect_err("broken");
    assert!(error.message.contains("is not valid JSONC"), "{error}");
}

#[test]
fn no_declarations_leave_the_text_exactly_as_it_was() {
    let updated = plan_with_entries(PLAN, "plan", &[]).expect("nothing to add");
    assert_eq!(updated, PLAN);
}
