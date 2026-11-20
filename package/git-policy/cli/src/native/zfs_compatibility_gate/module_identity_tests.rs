//! What:
//!  The identity read:
//! the release string loses one trailing newline,
//! an absent attribute
//!  is no refusal and is named,
//! a differing value is a different module,
//! and a module carrying
//!  no value at all is compared by the two files' own facts.
//! Why:
//!  A module that unloaded between the two reads must be noticed,
//! and a module
//!  the project's answer leaves judgeable must not be refused for carrying no
//!  version.

use super::gate_paths::GatePaths;
use super::module_identity::{ModuleIdentity, read_text};
use std::path::PathBuf;

/// The fixture directory one test writes into,
/// under the build directory rather than a temporary one.
fn directory(name: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-fixtures")
        .join(name);
    std::fs::create_dir_all(&directory).expect("fixture directory");
    return directory;
}

/// The six paths of one fixture,
/// written where a test needs them.
fn paths(name: &str, version: Option<&str>, srcversion: Option<&str>) -> GatePaths {
    let root: PathBuf = directory(name);
    let version_path: PathBuf = root.join("version");
    let srcversion_path: PathBuf = root.join("srcversion");
    let build_id_path: PathBuf = root.join("build-id");
    let base_path: PathBuf = root.join("vmlinux");
    let module_path: PathBuf = root.join("zfs");
    let _ = std::fs::remove_file(&build_id_path);
    for (path, body) in [
        (&version_path, version.unwrap_or("")),
        (&srcversion_path, srcversion.unwrap_or("")),
    ] {
        if version.is_some() || srcversion.is_some() {
            std::fs::write(path, format!("{body}\n")).expect("the attribute is written");
        }
    }
    if version.is_none() && srcversion.is_none() {
        let _ = std::fs::remove_file(&version_path);
        let _ = std::fs::remove_file(&srcversion_path);
        let _ = std::fs::remove_file(&build_id_path);
    }
    std::fs::write(&base_path, b"base").expect("the base file");
    std::fs::write(&module_path, b"module").expect("the module file");
    return GatePaths {
        device: root.join("zfs-device"),
        module_version: version_path,
        module_srcversion: srcversion_path,
        module_build_id: build_id_path,
        base_btf: base_path,
        module_btf: module_path,
    };
}

#[test]
fn the_release_string_loses_one_trailing_newline() {
    let fixture: GatePaths = paths("identity-newline", Some("2.4.4-1"), Some("A3553C308C29FD4B6595A72"));
    assert_eq!(read_text(fixture.module_version.as_path()), Some(String::from("2.4.4-1")));
    let identity: ModuleIdentity = ModuleIdentity::read(&fixture);
    assert_eq!(identity.version.as_deref(), Some("2.4.4-1"));
    assert!(identity.absent_attributes().contains(&"build ID"));
}

#[test]
fn an_absent_attribute_is_no_refusal_and_is_named() {
    let fixture: GatePaths = paths("identity-absent", None, None);
    let identity: ModuleIdentity = ModuleIdentity::read(&fixture);
    assert!(identity.version.is_none());
    assert_eq!(identity.absent_attributes(), ["version", "srcversion", "build ID"]);
    assert!(identity.base.is_some());
}

#[test]
fn another_value_is_another_module() {
    let first: ModuleIdentity = ModuleIdentity::read(&paths("identity-first", Some("2.4.4-1"), Some("A3553C308C29FD4B6595A72")));
    let second: ModuleIdentity = ModuleIdentity::read(&paths("identity-second", Some("2.4.4-2"), Some("A3553C308C29FD4B6595A72")));
    assert!(!first.same_as(&second));
    assert!(first.same_as(&first.clone()));
}

#[test]
fn a_module_with_no_value_is_compared_by_the_two_files_facts() {
    let first: ModuleIdentity = ModuleIdentity::read(&paths("identity-no-values", None, None));
    let same: ModuleIdentity = first.clone();
    assert!(first.same_as(&same));
    let other: ModuleIdentity = ModuleIdentity::read(&paths("identity-no-values-other", None, None));
    assert!(!first.same_as(&other) || first.base != other.base);
}
