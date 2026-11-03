//! The unit tests of the lockfile reading,
//! ported from the TypeScript tool's `lockfile-diff` cases.

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

#[test]
fn direct_dependencies_are_the_names_the_sourceless_packages_depend_on() {
    let text: &str = "version = 4\n\n[[package]]\nname = \"member\"\nversion = \"0.1.0\"\ndependencies = [\"dep\", \"twice 1.0.0\"]\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = \"registry+x\"\ndependencies = [\"deep\"]\n";

    let names: Vec<String> = crate::lockfile::direct_dependency_names(text).expect("parses").into_iter().collect();

    assert_eq!(names, vec!["dep".to_string(), "twice".to_string()]);
}

/// Review finding (registry identity): a package from any registry other than crates.io must be refused by name, because
/// every lookup goes to crates.io and would approve an unrelated crate of the same name and version.
#[test]
fn a_package_from_another_registry_is_refused_naming_the_package_and_its_source() {
    let text: String = lock(&[("dep", "1.0.0", "registry+https://registry.example.invalid/index")]);

    let result = parse_registry_versions(&text);

    let message: String = result.expect_err("an unsupported registry must not be read as crates.io");
    assert!(message.contains("dep") && message.contains("registry.example.invalid"), "{message}");
}

/// The sparse spelling of an alternative registry is the same case.
#[test]
fn a_sparse_registry_package_is_refused_too() {
    let text: String = lock(&[("dep", "1.0.0", "sparse+https://registry.example.invalid/index/")]);

    assert!(parse_registry_versions(&text).is_err());
}
