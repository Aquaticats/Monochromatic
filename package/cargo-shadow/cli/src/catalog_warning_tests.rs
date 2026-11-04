//! The unit tests of the catalog warning:
//! which dependencies it names,
//! and that a clean manifest says nothing.

use crate::catalog_warning::{Literal, literal_dependencies, warn_about_manifest};
use crate::test_scratch::scratch;

fn literal(table: &str, name: &str) -> Literal {
    return Literal { table: table.to_string(), name: name.to_string() };
}

#[test]
fn a_literal_version_and_a_table_without_workspace_are_named() {
    let text: &str = "[package]\nname = \"x\"\n\n[dependencies]\nserde = \"1\"\nclap = { version = \"4\", features = [\"derive\"] }\nok = { workspace = true }\n";

    assert_eq!(literal_dependencies(text), vec![literal("dependencies", "serde"), literal("dependencies", "clap")]);
}

#[test]
fn path_git_and_workspace_dependencies_are_not_named() {
    let text: &str = "[dependencies]\nlocal = { path = \"../local\" }\nremote = { git = \"https://example.invalid/r\" }\ncat = { workspace = true, features = [\"x\"] }\n";

    assert!(literal_dependencies(text).is_empty());
}

#[test]
fn dev_build_and_target_tables_are_read_and_the_workspace_table_is_skipped() {
    let text: &str = "[dev-dependencies]\na = \"1\"\n\n[build-dependencies]\nb = \"2\"\n\n[target.'cfg(unix)'.dependencies]\nc = \"3\"\n\n[workspace.dependencies]\nd = \">=4\"\n";

    assert_eq!(
        literal_dependencies(text),
        vec![literal("dev-dependencies", "a"), literal("build-dependencies", "b"), literal("target.'cfg(unix)'.dependencies", "c")]
    );
}

#[test]
fn invalid_toml_names_nothing() {
    assert!(literal_dependencies("[dependencies\n").is_empty());
}

#[test]
fn the_warning_names_the_manifest_the_dependency_and_the_remedy_and_a_clean_manifest_is_silent() {
    let dir = scratch();
    let dirty = dir.path().join("Cargo.toml");
    std::fs::write(&dirty, "[dependencies]\nserde = \"1\"\n").expect("write");
    let clean = dir.path().join("Clean.toml");
    std::fs::write(&clean, "[dependencies]\nserde = { workspace = true }\n").expect("write");
    let mut dirty_sink: Vec<u8> = Vec::new();
    let mut clean_sink: Vec<u8> = Vec::new();

    warn_about_manifest(&dirty, &mut dirty_sink);
    warn_about_manifest(&clean, &mut clean_sink);

    let text: String = String::from_utf8(dirty_sink).expect("text");
    assert!(text.contains("Cargo.toml"));
    assert!(text.contains("serde"));
    assert!(text.contains("sync:files"));
    assert!(clean_sink.is_empty());
}
