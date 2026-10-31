//! Unit tests of the lock file (render,
//! parse,
//! digest) and of the plan and lock agreement validator.

use serde_json::Value;

use super::agree::{compare, project, refusal};
use super::error::ErrorKind;
use super::fixtures::{SAMPLE_SHA256, sample_lock, sample_plan};
use super::lock::{Lock, lock_digest, parse_lock, render_lock};
use super::model::{Artifact, Request};

fn rendered_value() -> Value {
    return serde_json::from_str(&render_lock(&sample_lock())).expect("the rendered lock is JSON");
}

#[test]
fn the_rendered_lock_is_strict_json_with_two_space_indentation_and_a_trailing_newline() {
    let text = render_lock(&sample_lock());
    assert!(text.ends_with("}\n"));
    assert!(text.starts_with("{\n  \"schemaVersion\": 1,\n  \"generatedFrom\": \"toolchain.plan.jsonc\",\n  \"tools\": {"));
    assert!(serde_json::from_str::<Value>(&text).is_ok());
}

#[test]
fn the_frozen_surface_is_where_the_jq_readers_look() {
    let value = rendered_value();
    assert_eq!(value["tools"]["node"]["version"], "1.2.3");
    assert_eq!(value["tools"]["node"]["artifacts"]["linux-x64"]["url"], "https://example.invalid/node/download.tar.gz");
    assert_eq!(value["tools"]["node"]["artifacts"]["linux-x64"]["sha256"], SAMPLE_SHA256);
    assert_eq!(value["tools"]["android-sdk"]["options"]["androidPackages"][1], "platform-tools");
    assert_eq!(value["tools"]["rust"]["version"], "nightly-2026-10-30");
}

#[test]
fn a_tool_entry_writes_its_keys_in_the_documented_order_and_omits_absent_ones() {
    let value = rendered_value();
    let keys: Vec<&String> = value["tools"]["cargo:fd-find"].as_object().expect("an object").keys().collect();
    assert_eq!(keys, vec!["request", "version", "route", "manager", "packageId", "bins", "env", "options", "artifacts"]);
    let node_keys: Vec<&String> = value["tools"]["node"].as_object().expect("an object").keys().collect();
    assert_eq!(node_keys, vec!["request", "version", "route", "bins", "env", "options", "artifacts"]);
}

#[test]
fn the_digest_is_the_last_field() {
    let value = rendered_value();
    let last = value.as_object().expect("an object").keys().next_back().expect("has keys").clone();
    assert_eq!(last, "digest");
    assert!(value["digest"].as_str().expect("a string").starts_with("sha256:"));
}

#[test]
fn render_then_parse_gives_the_same_lock_and_the_recorded_digest() {
    let lock = sample_lock();
    let parsed = parse_lock(&render_lock(&lock), "toolchain.lock.json").expect("the rendered lock parses");
    assert_eq!(parsed.lock, lock);
    assert_eq!(parsed.recorded_digest, Some(lock_digest(&lock)));
}

#[test]
fn rendering_twice_gives_identical_bytes() {
    assert_eq!(render_lock(&sample_lock()), render_lock(&sample_lock()));
}

#[test]
fn reformatting_the_file_keeps_the_digest_valid() {
    let lock = sample_lock();
    let value: Value = serde_json::from_str(&render_lock(&lock)).expect("JSON");
    let compact = value.to_string();
    let parsed = parse_lock(&compact, "toolchain.lock.json").expect("compact JSON parses");
    assert!(compare(&sample_plan(), &parsed).is_empty());
}

#[test]
fn a_hand_edited_version_fails_the_digest() {
    let text = render_lock(&sample_lock()).replace("\"version\": \"1.2.3\"", "\"version\": \"9.9.9\"");
    let parsed = parse_lock(&text, "toolchain.lock.json").expect("still parses");
    let verdict = compare(&sample_plan(), &parsed);
    let problem = verdict.digest_problem.expect("the digest no longer matches");
    assert!(problem.contains("edited by hand"), "{problem}");
}

#[test]
fn a_hand_edited_artifact_hash_fails_the_digest() {
    let other = "0".repeat(64);
    let text = render_lock(&sample_lock()).replace(SAMPLE_SHA256, &other);
    let parsed = parse_lock(&text, "toolchain.lock.json").expect("still parses");
    assert!(compare(&sample_plan(), &parsed).digest_problem.is_some());
}

#[test]
fn a_lock_without_a_digest_is_reported_as_not_written_by_mpm_shadow() {
    let mut value = rendered_value();
    value.as_object_mut().expect("an object").remove("digest");
    let parsed = parse_lock(&value.to_string(), "toolchain.lock.json").expect("parses without a digest");
    let problem = compare(&sample_plan(), &parsed).digest_problem.expect("no digest is a problem");
    assert!(problem.contains("no digest"), "{problem}");
}

#[test]
fn a_current_lock_agrees_with_its_plan() {
    let parsed = parse_lock(&render_lock(&sample_lock()), "toolchain.lock.json").expect("parses");
    assert!(compare(&sample_plan(), &parsed).is_empty());
}

#[test]
fn a_tool_added_to_the_plan_needs_update() {
    let mut lock = sample_lock();
    lock.tools.retain(|tool| return tool.declaration.id != "npm:wrangler");
    let parsed = parse_lock(&render_lock(&lock), "toolchain.lock.json").expect("parses");
    let verdict = compare(&sample_plan(), &parsed);
    assert_eq!(verdict.added, vec!["npm:wrangler".to_string()]);
    assert!(verdict.needs_update());
}

#[test]
fn a_tool_dropped_from_the_plan_is_fixed_offline() {
    let mut lock = sample_lock();
    let mut extra = lock.tools[1].clone();
    extra.declaration.id = "gone".to_string();
    extra.declaration.bins = vec!["gone".to_string()];
    lock.tools.push(extra);
    let parsed = parse_lock(&render_lock(&lock), "toolchain.lock.json").expect("parses");
    let verdict = compare(&sample_plan(), &parsed);
    assert_eq!(verdict.removed, vec!["gone".to_string()]);
    assert!(!verdict.needs_update());
    let projected = project(&sample_plan(), &parsed.lock).expect("a removal projects offline");
    assert!(projected.tool("gone").is_none());
}

#[test]
fn a_changed_pin_is_a_resolution_edit_and_offline_lock_refuses_it() {
    let mut lock = sample_lock();
    lock.tools[0].declaration.request = Request::parse("github:marcosnils/bin@0.1.0").expect("parses");
    let parsed = parse_lock(&render_lock(&lock), "toolchain.lock.json").expect("parses");
    let verdict = compare(&sample_plan(), &parsed);
    assert_eq!(verdict.resolution_edits.len(), 1);
    assert_eq!(verdict.resolution_edits[0].0, "bin");
    assert!(verdict.resolution_edits[0].1.contains(&"request".to_string()));
    let error = project(&sample_plan(), &parsed.lock).expect_err("a pin change needs update");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("mpm-shadow update"), "{error}");
}

#[test]
fn every_resolution_affecting_field_is_rejected_offline() {
    let plan = sample_plan();
    let edits: Vec<(&str, fn(&mut Lock))> = vec![
        ("route", |lock| lock.tools[1].declaration.route = super::model::Route::BinSourced),
        ("packageId", |lock| lock.tools[6].declaration.package_id = Some("other".to_string())),
        ("manager", |lock| lock.tools[6].declaration.manager = Some("other".to_string())),
        ("os", |lock| lock.tools[7].declaration.os = None),
        ("options.asset", |lock| lock.tools[1].declaration.options.asset = Some("*.tgz".to_string())),
    ];
    for (field, edit) in edits {
        let mut lock = sample_lock();
        edit(&mut lock);
        let parsed = parse_lock(&render_lock(&lock), "toolchain.lock.json").expect("parses");
        let verdict = compare(&plan, &parsed);
        assert_eq!(verdict.resolution_edits.len(), 1, "{field}");
        assert!(verdict.resolution_edits[0].1.iter().any(|name| return name == field), "{field}");
        assert!(project(&plan, &parsed.lock).is_err(), "{field}");
    }
}

#[test]
fn a_changed_bin_list_is_a_projection_edit_that_offline_lock_repairs_keeping_the_resolution() {
    let mut lock = sample_lock();
    lock.tools[3].declaration.bins = vec!["node".to_string()];
    lock.tools[3].version = "26.13.0".to_string();
    lock.tools[3].artifacts.push((
        "linux-arm64".to_string(),
        Artifact { url: "https://example.invalid/node-arm".to_string(), sha256: SAMPLE_SHA256.to_string() },
    ));
    let parsed = parse_lock(&render_lock(&lock), "toolchain.lock.json").expect("parses");
    let verdict = compare(&sample_plan(), &parsed);
    assert_eq!(verdict.projection_edits, vec![("node".to_string(), vec!["bins".to_string()])]);
    assert!(!verdict.needs_update());
    let projected = project(&sample_plan(), &parsed.lock).expect("projection edits project offline");
    let node = projected.tool("node").expect("node is kept");
    assert_eq!(node.declaration.bins, vec!["node".to_string(), "npm".to_string(), "npx".to_string()]);
    assert_eq!(node.version, "26.13.0");
    assert_eq!(node.artifacts.len(), 2);
}

#[test]
fn a_different_tool_order_is_reported_and_repaired_by_projection() {
    let mut lock = sample_lock();
    lock.tools.swap(0, 1);
    let parsed = parse_lock(&render_lock(&lock), "toolchain.lock.json").expect("parses");
    let verdict = compare(&sample_plan(), &parsed);
    assert!(verdict.order_differs);
    let projected = project(&sample_plan(), &parsed.lock).expect("order projects offline");
    assert_eq!(projected.tools[0].declaration.id, "bin");
}

#[test]
fn the_refusal_names_lock_for_offline_fixes_and_update_for_network_ones() {
    let mut lock = sample_lock();
    lock.tools[3].declaration.bins = vec!["node".to_string()];
    let parsed = parse_lock(&render_lock(&lock), "toolchain.lock.json").expect("parses");
    let offline = refusal(&compare(&sample_plan(), &parsed));
    assert_eq!(offline.kind, ErrorKind::Drift);
    assert!(offline.message.contains("mpm-shadow lock"), "{offline}");
    assert!(offline.message.contains("node differs between the plan and the lock in: bins"), "{offline}");

    let mut missing = sample_lock();
    missing.tools.pop();
    let parsed_missing = parse_lock(&render_lock(&missing), "toolchain.lock.json").expect("parses");
    let online = refusal(&compare(&sample_plan(), &parsed_missing));
    assert!(online.message.contains("mpm-shadow update"), "{online}");
}

#[test]
fn a_digest_problem_points_at_restoring_the_file() {
    let text = render_lock(&sample_lock()).replace("\"version\": \"1.2.3\"", "\"version\": \"9.9.9\"");
    let parsed = parse_lock(&text, "toolchain.lock.json").expect("parses");
    let error = refusal(&compare(&sample_plan(), &parsed));
    assert!(error.message.contains("restore toolchain.lock.json from version control"), "{error}");
}

#[test]
fn an_unknown_lock_field_and_a_bad_hash_are_config_errors_naming_the_path() {
    let mut value = rendered_value();
    value["tools"]["node"]["artifacts"]["linux-x64"]["sha256"] = Value::String("abc".to_string());
    let error = parse_lock(&value.to_string(), "toolchain.lock.json").expect_err("a short hash is rejected");
    assert_eq!(error.kind, ErrorKind::Config);
    assert!(error.message.contains("tools.node.artifacts.linux-x64.sha256"), "{error}");

    let mut extra = rendered_value();
    extra.as_object_mut().expect("an object").insert("surprise".to_string(), Value::Bool(true));
    let unknown = parse_lock(&extra.to_string(), "toolchain.lock.json").expect_err("an unknown top-level field is rejected");
    assert!(unknown.message.contains("$.surprise"), "{unknown}");

    let truncated = parse_lock("{", "toolchain.lock.json").expect_err("not JSON");
    assert!(truncated.message.contains("is not valid JSON"), "{truncated}");
}
