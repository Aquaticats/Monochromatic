//! Unit tests of `bin-paths`: the output shape the IDE launcher pins, built from a disposable scope.

use serde_json::Value;
use std::fs;

use super::bin_paths::{bin_paths_output, render_bin_paths};
use super::fixtures::{SAMPLE_PLAN, sample_plan};
use super::plan::parse_plan;
use super::scope::{EnvMap, global_scope, repo_scope};
use crate::test_scratch::scratch;

#[test]
fn the_output_has_the_three_keys_the_ide_parser_requires_and_lists_only_existing_links() {
    let directory = scratch();
    let home = directory.path().join("home");
    let root = directory.path().join("work");
    fs::create_dir_all(&home).expect("make home");
    let env = EnvMap::from_pairs(&[("HOME", home.to_str().expect("utf-8"))]);
    let scope = repo_scope(&root, &env).expect("a scope");
    fs::create_dir_all(&scope.exposure_dir).expect("make the exposure directory");
    fs::write(scope.exposure_dir.join("node"), "").expect("write a link stand-in");
    fs::write(scope.exposure_dir.join("uv"), "").expect("write a link stand-in");

    let document = bin_paths_output(&sample_plan(), &scope, &env).expect("an output");
    let object = document.as_object().expect("an object");
    assert_eq!(object.keys().collect::<Vec<_>>(), vec!["exposureDir", "pathDirs", "bins"]);
    assert_eq!(document["exposureDir"], ".cache/toolchain-bin");
    let bins = document["bins"].as_object().expect("an object");
    assert_eq!(bins.keys().collect::<Vec<_>>(), vec!["uv", "node"]);
    assert_eq!(bins["node"], scope.exposure_dir.join("node").to_str().expect("utf-8"));
    let dirs = document["pathDirs"].as_array().expect("an array");
    assert_eq!(dirs.len(), 1, "only the exposure directory exists");
    assert_eq!(dirs[0], scope.exposure_dir.to_str().expect("utf-8"));
    for entry in dirs {
        assert!(std::path::Path::new(entry.as_str().expect("text")).is_absolute());
    }
}

#[test]
fn a_binary_declared_twice_is_refused() {
    let source = SAMPLE_PLAN.replace("\"bins\": [\"fd\"]", "\"bins\": [\"slopo\"]");
    assert!(parse_plan(&source, "plan").is_err(), "the plan reader already refuses the duplicate");
    let mut plan = sample_plan();
    plan.tools[6].bins = vec!["slopo".to_string()];
    let env = EnvMap::from_pairs(&[("HOME", "/h")]);
    let scope = repo_scope(std::path::Path::new("/w"), &env).expect("a scope");
    let error = bin_paths_output(&plan, &scope, &env).expect_err("a duplicate name");
    assert!(error.message.contains("\"slopo\""), "{error}");
}

#[test]
fn the_rendered_text_is_pretty_json_with_a_trailing_newline() {
    let directory = scratch();
    let env = EnvMap::from_pairs(&[("HOME", directory.path().to_str().expect("utf-8"))]);
    let scope = global_scope(&env).expect("a scope");
    let document = bin_paths_output(&sample_plan(), &scope, &env).expect("an output");
    let text = render_bin_paths(&document);
    assert!(text.starts_with("{\n  \"exposureDir\": \"exposure\",\n  \"pathDirs\": ["));
    assert!(text.ends_with("}\n"));
    let parsed: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(parsed, document);
}
