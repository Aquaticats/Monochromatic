//! What:
//!  The entry point both executables call:
//! a pair of files the approved module's
//!  declaration passes,
//! and a missing base file,
//! a module that is not type information at
//!  all and a module without the command structure each refuse with the file
//! they  read.
//! Why:
//!  The refusal is what the caller and `doctor` print,
//! so the file and the step it
//!  names are part of the gate's contract,
//! not a detail of the words.

use super::approved_module_fixture::{Change, module_with};
use super::blob_fixture::{Blob, check_pair, check_raw, fixture_directory};
use super::{Compatible, LayoutRequest, Refusal, Step, check_module_layout};
use std::path::PathBuf;

#[test]
fn the_approved_pair_reports_what_it_read() {
    let module: Vec<u8> = module_with(Change::None).bytes();
    let base: Vec<u8> = Blob::new().bytes();
    let answer: Result<Compatible, Refusal> = check_pair(
        "mod-approved",
        &Blob::new(),
        &module_with(Change::None),
    );
    let Ok(compatible): Result<Compatible, Refusal> = answer else {
        panic!("the approved pair refused: {}", answer.err().unwrap());
    };
    assert_eq!(compatible.base_bytes, base.len());
    assert_eq!(compatible.module_bytes, module.len());
    assert_eq!(compatible.base_header_bytes, 32);
    assert_eq!(compatible.module_header_bytes, 32);
}

#[test]
fn a_missing_base_file_refuses_at_that_step_and_names_it() {
    let directory: PathBuf = fixture_directory("mod-missing-base");
    let module: PathBuf = directory.join("zfs");
    std::fs::write(&module, module_with(Change::None).bytes()).expect("the module file");
    let absent: PathBuf = directory.join("absent-vmlinux");
    let answer: Result<Compatible, Refusal> = check_module_layout(&LayoutRequest {
        base_btf: absent.as_path(),
        module_btf: module.as_path(),
    });
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("the pair passed without a base file");
    };
    assert_eq!(refusal.step, Step::BaseBtf);
    assert_eq!(refusal.file.as_deref(), Some(absent.as_path()));
}

#[test]
fn a_module_that_is_not_type_information_refuses_whilst_reading_it() {
    let answer: Result<Compatible, Refusal> = check_raw(
        "mod-not-btf",
        &Blob::new().bytes(),
        b"this is no type information",
    );
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("a text file passed as a module's type information");
    };
    assert_eq!(refusal.step, Step::Header);
    let words: String = refusal.to_string();
    assert!(words.contains("header refused whilst reading"), "{words}");
    assert!(words.contains("not 0xeb9f"), "{words}");
}

#[test]
fn a_module_without_the_command_structure_refuses_at_the_type() {
    let answer: Result<Compatible, Refusal> = check_pair(
        "mod-no-command",
        &Blob::new(),
        &Blob::new(),
    );
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("an empty module passed");
    };
    assert_eq!(refusal.step, Step::Type);
    assert!(refusal.to_string().contains("zfs_cmd"), "{}", refusal.to_string());
}

/// The host's own two files pass,
/// when it has them:
/// this is the design's device-free arm that reads copies of the kernel's files
/// and asserts the 32-byte header shape passes.
///
/// A machine without the two files prints one line and returns,
/// so the container gate runs this control as a notice rather than a skip.
#[test]
fn the_hosts_two_files_pass_when_it_has_them() {
    let base: std::path::PathBuf = std::path::PathBuf::from("/sys/kernel/btf/vmlinux");
    let module: std::path::PathBuf = std::path::PathBuf::from("/sys/kernel/btf/zfs");
    if !base.exists() || !module.exists() {
        println!("NOTICE: this machine exposes no /sys/kernel/btf pair, so the host arm did not run");
        return;
    }
    let answer: Result<Compatible, Refusal> = check_module_layout(&LayoutRequest {
        base_btf: base.as_path(),
        module_btf: module.as_path(),
    });
    let Ok(compatible): Result<Compatible, Refusal> = answer else {
        panic!("the host's own files refused: {}", answer.err().unwrap());
    };
    assert_eq!(compatible.base_header_bytes, 32);
    assert_eq!(compatible.module_header_bytes, 32);
    assert!(compatible.module_bytes > compatible.base_header_bytes);
}
