//! What:
//!  The check's order and its refusals:
//! a missing device refuses before any file is
//!  read,
//! an unqualified target refuses the same way,
//! a device with a module whose
//!  declaration differs refuses at the difference,
//! a device with the approved declaration passes
//!  and reports what it read,
//! a module whose two identity reads agree passes,
//! and the retry and the
//!  cache answer only when their own facts move.
//! Why:
//!  A gate that refused everything would pass every arm that expects a refusal,
//! so
//!  the passing arm is the control,
//! and the device arm is what makes the order observable
//!  without this machine's kernel.

use super::gate_paths::GatePaths;
use super::gate_target::current_target;
use super::gate_verdict::{Facts, Verdict, VerdictCache, check, check_with};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    Compatible as LayoutCompatible, CompatibilityStep as Step,
};
use std::cell::Cell;
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

/// The 32-byte header of a type-information file with no sections,
/// which the header checks accept before any type is read.
fn empty_type_information() -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    bytes.extend_from_slice(&0xeb9f_u16.to_le_bytes());
    bytes.push(1);
    bytes.push(0);
    bytes.extend_from_slice(&32_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    return bytes;
}

/// The six paths of one fixture,
/// with the device and the attribute files written.
fn paths(name: &str) -> GatePaths {
    let root: PathBuf = directory(name);
    let device: PathBuf = root.join("zfs-device");
    std::fs::write(&device, b"").expect("a stand-in device file");
    for (file, body) in [
        ("version", "2.4.4-1\n"),
        ("srcversion", "A3553C308C29FD4B6595A72\n"),
        ("build-id", "build one"),
    ] {
        std::fs::write(root.join(file), body).expect("the fixture file");
    }
    std::fs::write(root.join("vmlinux"), empty_type_information()).expect("the base fixture");
    std::fs::write(root.join("zfs"), empty_type_information()).expect("the module fixture");
    return GatePaths {
        device,
        module_version: root.join("version"),
        module_srcversion: root.join("srcversion"),
        module_build_id: root.join("build-id"),
        base_btf: root.join("vmlinux"),
        module_btf: root.join("zfs"),
    };
}

/// What one scripted comparison answers,
/// with the two header lengths and sizes a pass reports.
fn layout() -> LayoutCompatible {
    return LayoutCompatible {
        base_bytes: 64,
        module_bytes: 32,
        base_header_bytes: 32,
        module_header_bytes: 24,
    };
}

#[test]
fn a_missing_device_refuses_before_any_file_is_read() {
    let fixture: GatePaths = paths("verdict-no-device");
    std::fs::remove_file(&fixture.device).expect("the device fixture is removed");
    let verdict: Verdict = check(&fixture);
    let Verdict::Refused(refusal) = verdict else {
        panic!("a missing device passed");
    };
    assert_eq!(refusal.step, Step::Device);
    assert!(refusal.file.is_some());
}

#[test]
fn a_present_device_with_no_type_information_refuses_at_the_header() {
    let fixture: GatePaths = paths("verdict-not-btf");
    std::fs::write(fixture.base_btf.as_path(), b"not type information").expect("the base file");
    let verdict: Verdict = check(&fixture);
    let Verdict::Refused(refusal) = verdict else {
        panic!("a text file passed as type information");
    };
    assert_eq!(refusal.step, Step::Header);
    assert_eq!(refusal.file.as_deref(), Some(fixture.base_btf.as_path()));
}

#[test]
fn a_missing_base_file_refuses_at_that_step_and_names_it() {
    let fixture: GatePaths = paths("verdict-missing-base");
    std::fs::remove_file(&fixture.base_btf).expect("the base fixture is removed");
    let verdict: Verdict = check(&fixture);
    let Verdict::Refused(refusal) = verdict else {
        panic!("a pair passed without a base file");
    };
    assert_eq!(refusal.step, Step::BaseBtf);
    assert_eq!(refusal.file.as_deref(), Some(fixture.base_btf.as_path()));
}

#[test]
fn a_missing_module_file_refuses_at_that_step_and_names_it() {
    let fixture: GatePaths = paths("verdict-missing-module");
    std::fs::remove_file(&fixture.module_btf).expect("the module fixture is removed");
    let verdict: Verdict = check(&fixture);
    let Verdict::Refused(refusal) = verdict else {
        panic!("a pair passed without a module file");
    };
    assert_eq!(refusal.step, Step::ModuleBtf);
    assert_eq!(refusal.file.as_deref(), Some(fixture.module_btf.as_path()));
}

#[test]
fn a_scripted_pass_builds_the_facts_from_what_the_comparison_read() {
    let fixture: GatePaths = paths("verdict-scripted-pass");
    let verdict: Verdict = check_with(&fixture, &|_paths: &GatePaths| {
        return Ok(layout());
    });
    let Verdict::Compatible(facts) = verdict else {
        panic!("a scripted pass refused");
    };
    let expected: Facts = Facts {
        target: current_target(),
        version: Some(String::from("2.4.4-1")),
        absent: Vec::new(),
        base_btf_bytes: 64,
        module_btf_bytes: 32,
        header_bytes: (32, 24),
    };
    assert_eq!(facts.target, expected.target);
    assert_eq!(facts.version, expected.version);
    assert_eq!(facts.absent, expected.absent);
    assert_eq!(facts.base_btf_bytes, expected.base_btf_bytes);
    assert_eq!(facts.module_btf_bytes, expected.module_btf_bytes);
    assert_eq!(facts.header_bytes, expected.header_bytes);
}

#[test]
fn a_comparison_refusal_is_the_verdict_and_is_not_retried() {
    let fixture: GatePaths = paths("verdict-comparison-refusal");
    let calls: Cell<usize> = Cell::new(0);
    let verdict: Verdict = check_with(&fixture, &|_paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        return Err(super::module_identity::ModuleIdentity::changed_under_the_check());
    });
    assert!(matches!(verdict, Verdict::Refused(_)));
    assert_eq!(calls.get(), 1);
}

#[test]
fn a_stable_identity_is_compared_once() {
    let fixture: GatePaths = paths("verdict-stable");
    let calls: Cell<usize> = Cell::new(0);
    let verdict: Verdict = check_with(&fixture, &|_paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        return Ok(layout());
    });
    assert!(matches!(verdict, Verdict::Compatible(_)));
    assert_eq!(calls.get(), 1);
}

#[test]
fn a_module_that_moves_under_the_first_read_passes_after_one_retry() {
    let fixture: GatePaths = paths("verdict-one-retry");
    let rewritten: Cell<bool> = Cell::new(false);
    let calls: Cell<usize> = Cell::new(0);
    let verdict: Verdict = check_with(&fixture, &|paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        if !rewritten.get() {
            std::fs::write(paths.module_version.as_path(), "2.4.4-2\n").expect("the moved version");
            rewritten.set(true);
        }
        return Ok(layout());
    });
    let Verdict::Compatible(facts) = verdict else {
        panic!("the retry did not pass");
    };
    assert_eq!(calls.get(), 2);
    assert_eq!(facts.version.as_deref(), Some("2.4.4-2"));
}

#[test]
fn a_module_that_moves_under_both_attempts_refuses_at_the_identity() {
    let fixture: GatePaths = paths("verdict-both-attempts");
    let moved: Cell<bool> = Cell::new(false);
    let calls: Cell<usize> = Cell::new(0);
    let verdict: Verdict = check_with(&fixture, &|paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        let text: &str = if moved.get() { "2.4.4-1\n" } else { "2.4.4-2\n" };
        std::fs::write(paths.module_version.as_path(), text).expect("the moved version");
        moved.set(!moved.get());
        return Ok(layout());
    });
    let Verdict::Refused(refusal) = verdict else {
        panic!("a module that moved under both attempts passed");
    };
    assert_eq!(refusal.step, Step::Identity);
    assert_eq!(calls.get(), 2);
}

#[test]
fn the_cache_reuses_a_verdict_while_the_three_values_and_the_files_hold() {
    let fixture: GatePaths = paths("verdict-cache-reuse");
    let cache: VerdictCache = VerdictCache::new();
    let calls: Cell<usize> = Cell::new(0);
    let compare = |_paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        return Ok(layout());
    };
    assert!(matches!(cache.answer(&fixture, &compare), Verdict::Compatible(_)));
    assert!(matches!(cache.answer(&fixture, &compare), Verdict::Compatible(_)));
    assert_eq!(calls.get(), 1);
}

#[test]
fn the_cache_reruns_when_the_version_text_moves() {
    let fixture: GatePaths = paths("verdict-cache-version");
    let cache: VerdictCache = VerdictCache::new();
    let calls: Cell<usize> = Cell::new(0);
    let compare = |_paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        return Ok(layout());
    };
    let _ = cache.answer(&fixture, &compare);
    std::fs::write(fixture.module_version.as_path(), "2.4.4-2\n").expect("the moved version");
    let _ = cache.answer(&fixture, &compare);
    assert_eq!(calls.get(), 2);
}

#[test]
fn the_cache_reruns_when_the_source_version_moves() {
    let fixture: GatePaths = paths("verdict-cache-srcversion");
    let cache: VerdictCache = VerdictCache::new();
    let calls: Cell<usize> = Cell::new(0);
    let compare = |_paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        return Ok(layout());
    };
    let _ = cache.answer(&fixture, &compare);
    std::fs::write(fixture.module_srcversion.as_path(), "00000000000000000000000\n").expect("the moved source version");
    let _ = cache.answer(&fixture, &compare);
    assert_eq!(calls.get(), 2);
}

#[test]
fn the_cache_reruns_when_the_build_id_moves() {
    let fixture: GatePaths = paths("verdict-cache-build-id");
    let cache: VerdictCache = VerdictCache::new();
    let calls: Cell<usize> = Cell::new(0);
    let compare = |_paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        return Ok(layout());
    };
    let _ = cache.answer(&fixture, &compare);
    std::fs::write(fixture.module_build_id.as_path(), "build two").expect("the moved build ID");
    let _ = cache.answer(&fixture, &compare);
    assert_eq!(calls.get(), 2);
}

#[test]
fn the_cache_reruns_when_the_module_file_vanishes_and_returns() {
    let fixture: GatePaths = paths("verdict-cache-module-file");
    let cache: VerdictCache = VerdictCache::new();
    let calls: Cell<usize> = Cell::new(0);
    let compare = |_paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        return Ok(layout());
    };
    let _ = cache.answer(&fixture, &compare);
    std::fs::remove_file(fixture.module_btf.as_path()).expect("the module fixture is removed");
    let _ = cache.answer(&fixture, &compare);
    std::fs::write(fixture.module_btf.as_path(), empty_type_information()).expect("the module returns");
    let _ = cache.answer(&fixture, &compare);
    assert_eq!(calls.get(), 3);
}

#[test]
fn the_cache_reruns_when_a_file_fact_moves_with_no_identity_value() {
    let fixture: GatePaths = paths("verdict-cache-no-values");
    for file in [&fixture.module_version, &fixture.module_srcversion, &fixture.module_build_id] {
        std::fs::remove_file(file).expect("the identity file is removed");
    }
    let cache: VerdictCache = VerdictCache::new();
    let calls: Cell<usize> = Cell::new(0);
    let compare = |_paths: &GatePaths| {
        calls.set(calls.get().saturating_add(1));
        return Ok(layout());
    };
    let _ = cache.answer(&fixture, &compare);
    std::fs::write(fixture.module_btf.as_path(), b"a longer module file").expect("the module moves");
    let _ = cache.answer(&fixture, &compare);
    assert_eq!(calls.get(), 2);
}
