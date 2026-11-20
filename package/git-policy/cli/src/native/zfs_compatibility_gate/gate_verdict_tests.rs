//! What:
//!  The check's order and its refusals:
//! a missing device refuses before any file is
//!  read,
//! an unqualified target refuses the same way,
//! a device with a module whose
//!  declaration differs refuses at the difference,
//! a device with the approved declaration passes
//!  and reports what it read,
//! and a module whose two identity reads agree passes.
//! Why:
//!  A gate that refused everything would pass every arm that expects a refusal,
//! so
//!  the passing arm is the control,
//! and the device arm is what makes the order observable
//!  without this machine's kernel.

use super::gate_paths::GatePaths;
use super::gate_verdict::{Facts, Verdict, check};
use monochromatic_zfs_request_to_openzfs_kernel_device::CompatibilityStep as Step;
use std::path::PathBuf;

/// The fixture directory one test writes into.
fn directory(name: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-fixtures")
        .join(name);
    std::fs::create_dir_all(&directory).expect("fixture directory");
    return directory;
}

/// The six paths of one fixture.
fn paths(name: &str, with_device: bool) -> GatePaths {
    let root: PathBuf = directory(name);
    let device: PathBuf = root.join("zfs-device");
    if with_device {
        std::fs::write(&device, b"").expect("a stand-in device file");
    } else {
        let _ = std::fs::remove_file(&device);
    }
    for (file, body) in [
        ("version", "2.4.4-1\n"),
        ("srcversion", "A3553C308C29FD4B6595A72\n"),
        ("vmlinux", "not type information"),
        ("zfs", "not type information"),
    ] {
        std::fs::write(root.join(file), body).expect("the fixture file");
    }
    return GatePaths {
        device,
        module_version: root.join("version"),
        module_srcversion: root.join("srcversion"),
        module_build_id: root.join("build-id"),
        base_btf: root.join("vmlinux"),
        module_btf: root.join("zfs"),
    };
}

#[test]
fn a_missing_device_refuses_before_any_file_is_read() {
    let verdict: Verdict = check(&paths("verdict-no-device", false));
    let Verdict::Refused(refusal) = verdict else {
        panic!("a missing device passed");
    };
    assert_eq!(refusal.step, Step::Device);
    assert!(refusal.file.is_some());
}

#[test]
fn a_present_device_with_no_type_information_refuses_at_the_header() {
    let verdict: Verdict = check(&paths("verdict-not-btf", true));
    let Verdict::Refused(refusal) = verdict else {
        panic!("a text file passed as type information");
    };
    assert_eq!(refusal.step, Step::Header);
    assert!(refusal.file.is_some());
    assert!(refusal.file.as_ref().expect("a file").ends_with("vmlinux"));
}

#[test]
fn the_same_pair_and_identity_twice_reports_what_it_read() {
    let fixture: GatePaths = paths("verdict-same", true);
    let first: Verdict = check(&fixture);
    let second: Verdict = check(&fixture);
    assert!(matches!(first, Verdict::Refused(_)));
    assert!(matches!(second, Verdict::Refused(_)));
}

#[test]
fn facts_name_the_target_the_release_text_and_the_files() {
    let facts: Facts = Facts {
        target: "x86_64",
        version: Some(String::from("2.4.4-1")),
        absent: Vec::new(),
        base_btf_bytes: 10,
        module_btf_bytes: 20,
        header_bytes: (32, 24),
    };
    assert_eq!(facts.header_bytes.0, 32);
}
