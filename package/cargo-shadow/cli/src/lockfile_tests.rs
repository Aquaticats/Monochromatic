//! The unit tests of the lockfile reading, ported from the TypeScript tool's `lockfile-diff` cases.

use crate::lockfile::{LockedVersion, added_versions, parse_registry_versions, registry_versions};

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

fn lock(entries: &[(&str, &str, &str)]) -> String {
    let mut text: String = "version = 4\n".to_string();
    for (name, version, source) in entries {
        text.push_str(&format!("\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\n"));
        if !source.is_empty() {
            text.push_str(&format!("source = \"{source}\"\n"));
        }
    }
    return text;
}

fn version(name: &str, number: &str) -> LockedVersion {
    return LockedVersion { name: name.to_string(), version: number.to_string() };
}

#[test]
fn only_registry_packages_are_read() {
    let text: String = lock(&[("member", "0.1.0", ""), ("dep", "1.0.0", REGISTRY), ("fromgit", "2.0.0", "git+https://example.invalid/x")]);

    assert_eq!(parse_registry_versions(&text), Ok(vec![version("dep", "1.0.0")]));
}

#[test]
fn text_that_is_not_toml_is_an_error_not_an_empty_list() {
    assert!(parse_registry_versions("[[package\n").is_err());
}

#[test]
fn versions_are_deduplicated_across_lockfiles_and_sorted() {
    let first: String = lock(&[("b", "1.0.0", REGISTRY), ("a", "2.0.0", REGISTRY)]);
    let second: String = lock(&[("a", "2.0.0", REGISTRY)]);

    assert_eq!(registry_versions(&[first, second]), Ok(vec![version("a", "2.0.0"), version("b", "1.0.0")]));
}

#[test]
fn a_version_another_lockfile_already_held_is_not_added() {
    let before_one: String = lock(&[("a", "1.0.0", REGISTRY)]);
    let before_two: String = lock(&[("a", "2.0.0", REGISTRY)]);
    let after: String = lock(&[("a", "2.0.0", REGISTRY), ("b", "1.0.0", REGISTRY)]);

    assert_eq!(added_versions(&[before_one, before_two], &[after]), Ok(vec![version("b", "1.0.0")]));
}

#[test]
fn an_absent_base_lockfile_counts_every_version_as_new() {
    let after: String = lock(&[("a", "1.0.0", REGISTRY)]);

    assert_eq!(added_versions(&[String::new()], &[after]), Ok(vec![version("a", "1.0.0")]));
}
