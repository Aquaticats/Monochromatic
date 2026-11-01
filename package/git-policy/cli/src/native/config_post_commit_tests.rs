//! Controls for the `postCommit` key: what loads, and every refusal with the key path it names.

use super::*;
use crate::config_parse::parse_config;
use crate::config_schema::CliGitConfig;

/// Parse a configuration document and return its actions, or the error text.
fn actions(document: &str) -> Result<Vec<PostCommitAction>, String> {
    return parse_config(document).map(|config: CliGitConfig| { return config.post_commit_actions; }).map_err(|error| { return error.to_string(); });
}

/// A declared action loads with its templates and the documented default bound.
#[test]
fn an_action_loads_with_the_default_bound() {
    let loaded = actions(r#"{ "postCommit": { "actions": [{ "name": "review", "argv": ["run", "--unit=u-{shortHead}", "{env:HOME}"] }] } }"#)
        .expect("loads");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "review");
    assert_eq!(loaded[0].argv.len(), 3);
    assert_eq!(loaded[0].timeout_seconds, DEFAULT_ACTION_TIMEOUT_SECONDS);
    let bounded = actions(r#"{ "postCommit": { "actions": [{ "name": "a", "argv": ["x"], "timeoutSeconds": 5 }] } }"#).expect("loads");
    assert_eq!(bounded[0].timeout_seconds, 5);
}

/// No `postCommit` key, an empty object and an empty list all declare nothing.
#[test]
fn nothing_declared_means_no_actions() {
    assert_eq!(actions("{}"), Ok(Vec::new()));
    assert_eq!(actions(r#"{ "postCommit": {} }"#), Ok(Vec::new()));
    assert_eq!(actions(r#"{ "postCommit": { "actions": [] } }"#), Ok(Vec::new()));
}

/// Each refusal names the key path the author must change.
#[test]
fn refusals_name_the_key_path() {
    let cases: [(&str, &str); 9] = [
        (r#"{ "postCommit": 1 }"#, "postCommit"),
        (r#"{ "postCommit": { "run": [] } }"#, "postCommit.run"),
        (r#"{ "postCommit": { "actions": {} } }"#, "postCommit.actions"),
        (r#"{ "postCommit": { "actions": [1] } }"#, "postCommit.actions[0]"),
        (r#"{ "postCommit": { "actions": [{ "argv": ["x"] }] } }"#, "needs a name"),
        (r#"{ "postCommit": { "actions": [{ "name": "a" }] } }"#, "needs an argv array"),
        (r#"{ "postCommit": { "actions": [{ "name": "a", "argv": [] }] } }"#, "postCommit.actions[0].argv"),
        (r#"{ "postCommit": { "actions": [{ "name": "a", "argv": ["x"], "shell": true }] } }"#, "postCommit.actions[0].shell"),
        (r#"{ "postCommit": { "actions": [{ "name": "a", "argv": ["x"], "timeoutSeconds": 0 }] } }"#, "timeoutSeconds"),
    ];
    for (document, expected) in cases {
        let message: String = actions(document).expect_err(document);
        assert!(message.contains(expected), "{document}: {message}");
    }
}

/// An argument with an unknown field or a stray brace is refused with the field list, and names its index.
#[test]
fn an_argument_outside_the_field_set_is_refused() {
    let message: String = actions(r#"{ "postCommit": { "actions": [{ "name": "a", "argv": ["x", "{commit}"] }] } }"#).expect_err("refused");
    assert!(message.contains("postCommit.actions[0].argv[1]") && message.contains("{head}"), "{message}");
    assert!(actions(r#"{ "postCommit": { "actions": [{ "name": "a", "argv": ["x", "$(git rev-parse HEAD)"] }] } }"#).is_ok());
}

/// Two actions may not share a name.
#[test]
fn action_names_are_unique() {
    let message: String = actions(
        r#"{ "postCommit": { "actions": [{ "name": "a", "argv": ["x"] }, { "name": "a", "argv": ["y"] }] } }"#,
    )
    .expect_err("refused");
    assert!(message.contains("postCommit.actions[1].name") && message.contains("unique"), "{message}");
}

/// More actions than the cap are refused, and exactly the cap loads.
#[test]
fn the_number_of_actions_is_capped() {
    let one = |index: usize| { return format!(r#"{{ "name": "a{index}", "argv": ["x"] }}"#); };
    let build = |count: usize| {
        let list: Vec<String> = (0..count).map(one).collect();
        return format!(r#"{{ "postCommit": {{ "actions": [{}] }} }}"#, list.join(","));
    };
    assert_eq!(actions(build(MAXIMUM_ACTIONS).as_str()).expect("loads").len(), MAXIMUM_ACTIONS);
    assert!(actions(build(MAXIMUM_ACTIONS + 1).as_str()).expect_err("refused").contains("at most"));
}

/// The retired `hooks` key still loads and is ignored; the file's value is checked but never used.
#[test]
fn the_retired_hooks_key_still_loads() {
    let config: CliGitConfig = parse_config(r#"{ "hooks": { "concurrentCommits": true } }"#).expect("loads");
    assert!(config.concurrency.hooks.present);
    assert!(parse_config(r#"{ "hooks": { "concurrentCommits": "yes" } }"#).is_err());
    assert!(!parse_config("{}").expect("loads").concurrency.hooks.present);
}
