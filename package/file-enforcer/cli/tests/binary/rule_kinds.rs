//! What:
//!  The binary tests for the rule kinds that need a library the engine does not link.
//! Why:
//! `jetbrains-harper` runs on the per-user config,
//! and `rust-catalog` and `json-keys` run through the binary,
//! so only a run of the built binary proves the wiring.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe("the rule kinds the binary links", () => { it("...", () => { ... }); });
//! ```

use crate::support::{
    CONFIG,
    Harness,
    count_code,
};

#[test]
fn user_commands_use_their_own_server_and_the_per_user_config() {
    let harness = Harness::new();
    let user_config = harness.home.path().join("user.config.jsonc");
    let user_text = r#"{"rules": [{"kind": "jetbrains-harper", "id": "harper", "productPrefixes": ["IdeaIC"],
        "disabledLinters": [], "excludePatterns": [], "scopedServers": []}]}"#;
    std::fs::write(&user_config, user_text).expect("user config");
    let config_argument = user_config.display().to_string();
    let (user_code, user_lines) =
        harness.run(&["--user-config", &config_argument, "user", "check"]);
    assert_eq!(user_code, 0, "{user_lines:?}");
    assert_eq!(
        count_code(&user_lines, "kind-not-wired"),
        0,
        "the XML crate is linked: {user_lines:?}"
    );
    assert_eq!(
        count_code(&user_lines, "jetbrains-harper-skipped"),
        1,
        "the disposable home has no IDE: {user_lines:?}"
    );
    assert_eq!(harness.server_pids().len(), 1);
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (repository_code, _) = harness.run(&["check"]);
    assert_eq!(repository_code, 1);
    assert_eq!(
        harness.server_pids().len(),
        2,
        "the repository and the per-user config have separate servers"
    );
    let (missing_code, missing) =
        harness.run(&["--user-config", "does-not-exist.jsonc", "user", "write"]);
    assert_eq!(missing_code, 2, "{missing:?}");
    assert_eq!(missing[0]["code"], "config-unreadable");
}

#[test]
fn the_rust_catalog_kind_is_wired_into_the_binary() {
    let harness = Harness::new();
    let config = r#"{"rules": [{"kind": "rust-catalog", "id": "catalog", "catalog": "rust-catalog.toml",
        "manifests": ["package/*/*/Cargo.toml"]}]}"#;
    harness.write("file-enforcer.config.jsonc", config);
    harness.write("rust-catalog.toml", "[catalog]\n");
    harness.write("package/cli/a/Cargo.toml", "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nanyhow = \"1\"\n\n[workspace]\n");
    let (check_code, check) = harness.run(&["check"]);
    assert_eq!(
        (
            check_code,
            count_code(&check, "kind-not-wired"),
            count_code(&check, "drift-content")
        ),
        (1, 0, 2),
        "{check:?}"
    );
    let (write_code, write) = harness.run(&["write"]);
    assert_eq!(write_code, 0, "{write:?}");
    assert!(
        harness
            .read("rust-catalog.toml")
            .contains("anyhow = \">=1\"")
    );
    let (clean_code, clean) = harness.run(&["check"]);
    assert_eq!(clean_code, 0, "{clean:?}");
}

#[test]
fn check_fails_on_a_manifest_carrying_the_pre_transfer_url_and_write_converges() {
    let harness = Harness::new();
    let config = r#"{"rules": [{"kind": "json-keys", "id": "package-metadata", "manifests": ["package/*/*/package.json"],
        "edits": [
          {"guard": ["repository"], "path": ["repository", "type"], "value": "git"},
          {"guard": ["repository"], "path": ["repository", "url"], "value": "https://github.com/Aquaticats/Monochromatic.git"},
          {"guard": ["repository"], "path": ["repository", "directory"], "value": {"var": "dir"}},
          {"guard": ["repository"], "path": ["homepage"], "value": {"concat": ["https://github.com/Aquaticats/Monochromatic/tree/main/", {"var": "dir"}]}}
        ]}]}"#;
    harness.write("file-enforcer.config.jsonc", config);
    let pre_transfer = "{\n  \"name\": \"y\",\n  \"repository\": {\n    \"type\": \"git\",\n    \"url\": \"https://github.com/Aquaticat/Monochromatic.git\"\n  }\n}\n";
    harness.write("package/x/y/package.json", pre_transfer);
    let (check_code, check) = harness.run(&["check"]);
    assert_eq!(check_code, 1, "{check:?}");
    assert_eq!(count_code(&check, "drift-content"), 1, "{check:?}");
    assert!(
        harness
            .read("package/x/y/package.json")
            .contains("github.com/Aquaticat/Monochromatic"),
        "check reports the drift and changes nothing"
    );
    let (write_code, write) = harness.run(&["write"]);
    assert_eq!(write_code, 0, "{write:?}");
    let written = harness.read("package/x/y/package.json");
    assert!(
        written.contains("\"url\": \"https://github.com/Aquaticats/Monochromatic.git\""),
        "{written}"
    );
    assert!(
        written.contains("\"directory\": \"package/x/y\""),
        "{written}"
    );
    let (clean_code, clean) = harness.run(&["check"]);
    assert_eq!(
        clean_code, 0,
        "a second check after write is clean: {clean:?}"
    );
    let string_repository =
        "{\n  \"name\": \"w\",\n  \"repository\": \"github:Aquaticat/Monochromatic\"\n}\n";
    harness.write("package/x/w/package.json", string_repository);
    let (shape_code, shape) = harness.run(&["check"]);
    assert_eq!(shape_code, 1, "{shape:?}");
    assert_eq!(count_code(&shape, "manifest-shape"), 1, "{shape:?}");
    let (shape_write_code, shape_write) = harness.run(&["write"]);
    assert_eq!(shape_write_code, 1, "{shape_write:?}");
    assert_eq!(
        harness.read("package/x/w/package.json"),
        string_repository,
        "no homepage lands beside a repository the rule cannot manage"
    );
}
