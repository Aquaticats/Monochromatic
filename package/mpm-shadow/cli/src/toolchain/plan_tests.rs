//! Unit tests of the plan reader and the model decoding behind it.

use super::error::ErrorKind;
use super::fixtures::{SAMPLE_PLAN, sample_plan};
use super::model::{Postinstall, Request, Route};
use super::plan::{parse_plan, read_plan};
use crate::test_scratch::scratch;

#[test]
fn every_route_has_a_name_that_parses_back() {
    let routes = [
        Route::BinSourced,
        Route::Archive,
        Route::CargoBinstall,
        Route::CargoGit,
        Route::MpmPnpm,
        Route::MpmUvx,
        Route::GoInstall,
        Route::RustupToolchain,
        Route::RustupComponents,
        Route::AndroidSdk,
    ];
    for route in routes {
        assert_eq!(Route::parse(route.as_str()), Some(route));
        assert_ne!(route.as_str(), "unknown");
    }
}

#[test]
fn the_sample_plan_keeps_the_written_order_and_decodes_every_field() {
    let plan = sample_plan();
    let ids: Vec<&str> = plan.tools.iter().map(|tool| return tool.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "bin",
            "uv",
            "pnpm",
            "node",
            "rust",
            "cargo-binstall",
            "cargo:fd-find",
            "cargo:monochromatic-mpm-shadow",
            "npm:wrangler",
            "pipx:slopo",
            "android-sdk"
        ]
    );
    assert_eq!(plan.tools[0].request, Request::parse("github:marcosnils/bin@0.29.4").expect("parses"));
    let rust = &plan.tools[4];
    assert_eq!(rust.request, Request::parse("rust@nightly").expect("parses"));
    assert_eq!(rust.route, Route::RustupToolchain);
    assert_eq!(rust.package_id.as_deref(), Some("nightly-2026-10-30"));
    assert_eq!(rust.env, vec![("RUSTUP_TOOLCHAIN".to_string(), "nightly-2026-10-30".to_string())]);
    assert_eq!(rust.options.rust_components, Some(vec!["clippy".to_string(), "rust-src".to_string()]));
    let shadow = &plan.tools[7];
    assert!(shadow.os.is_some());
    let Some(Postinstall::Link(link)) = &shadow.options.postinstall else {
        panic!("the shadow entry carries a postinstall link");
    };
    assert_eq!(link.link_name, "mpm");
    assert_eq!(link.link_dir.as_deref(), Some(".cache/mpm-shadow-bin"));
}

#[test]
fn comments_and_trailing_commas_parse_but_strict_json_is_not_required() {
    let source = "{ \"schemaVersion\": 1, \"tools\": { /* c */ \"a\": { \"request\": \"node@latest\", \"route\": \"archive\", }, }, }";
    let plan = parse_plan(source, "inline").expect("JSONC parses");
    assert_eq!(plan.tools.len(), 1);
}

#[test]
fn an_unknown_tool_field_is_rejected_with_its_path() {
    let source = SAMPLE_PLAN.replace("\"route\": \"archive\",\n      \"bins\": [\"uv\"", "\"route\": \"archive\",\n      \"colour\": 1,\n      \"bins\": [\"uv\"");
    let error = parse_plan(&source, "plan.jsonc").expect_err("an unknown field is rejected");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.contains("plan.jsonc"), "{error}");
    assert!(error.message.contains("tools.uv.colour"), "{error}");
}

#[test]
fn a_resolved_field_in_the_plan_is_rejected_because_update_owns_it() {
    let source = SAMPLE_PLAN.replace("\"bins\": [\"node\", \"npm\", \"npx\"],", "\"bins\": [\"node\", \"npm\", \"npx\"], \"version\": \"26.0.0\",");
    let error = parse_plan(&source, "plan.jsonc").expect_err("version is not a plan field");
    assert!(error.message.contains("tools.node.version"), "{error}");
}

#[test]
fn a_tool_id_written_twice_is_rejected() {
    let source = "{ \"schemaVersion\": 1, \"tools\": { \"a\": { \"request\": \"node@latest\", \"route\": \"archive\" }, \"a\": { \"request\": \"node@latest\", \"route\": \"archive\" } } }";
    let error = parse_plan(source, "plan.jsonc").expect_err("duplicate ids are rejected");
    assert!(error.message.contains("twice"), "{error}");
}

#[test]
fn a_binary_name_declared_by_two_tools_is_rejected() {
    let source = SAMPLE_PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"cargo\"]");
    let error = parse_plan(&source, "plan.jsonc").expect_err("duplicate bin names are rejected");
    assert!(error.message.contains("\"cargo\""), "{error}");
    assert!(error.message.contains("rust"), "{error}");
}

#[test]
fn an_unknown_route_names_the_valid_routes() {
    let source = SAMPLE_PLAN.replace("\"route\": \"android-sdk\"", "\"route\": \"carrier-pigeon\"");
    let error = parse_plan(&source, "plan.jsonc").expect_err("an unknown route is rejected");
    assert!(error.message.contains("carrier-pigeon"), "{error}");
    assert!(error.message.contains("bin-sourced"), "{error}");
}

#[test]
fn a_wrong_typed_field_names_the_path_and_what_was_found() {
    let source = SAMPLE_PLAN.replace("\"bins\": [\"adb\", \"sdkmanager\"]", "\"bins\": \"adb\"");
    let error = parse_plan(&source, "plan.jsonc").expect_err("bins must be an array");
    assert!(error.message.contains("tools.android-sdk.bins must be an array of strings, found a string"), "{error}");
}

#[test]
fn text_that_is_not_jsonc_names_the_file() {
    let error = parse_plan("{ \"tools\": ", "toolchain.plan.jsonc").expect_err("truncated input is rejected");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.starts_with("toolchain.plan.jsonc is not valid JSONC"), "{error}");
}

#[test]
fn a_wrong_schema_version_is_rejected() {
    let error = parse_plan("{ \"schemaVersion\": 2, \"tools\": {} }", "plan.jsonc").expect_err("only version 1 exists");
    assert!(error.message.contains("schemaVersion must be 1"), "{error}");
}

#[test]
fn reading_a_missing_plan_file_names_the_path() {
    let directory = scratch();
    let path = directory.path().join("toolchain.plan.jsonc");
    let error = read_plan(&path).expect_err("the file does not exist");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.contains("toolchain.plan.jsonc"), "{error}");
}

#[test]
fn reading_a_plan_file_decodes_it() {
    let directory = scratch();
    let path = directory.path().join("toolchain.plan.jsonc");
    std::fs::write(&path, SAMPLE_PLAN).expect("write the plan");
    let plan = read_plan(&path).expect("the written plan reads");
    assert_eq!(plan.tools.len(), 11);
}
