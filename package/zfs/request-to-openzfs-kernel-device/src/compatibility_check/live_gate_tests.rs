//! What:
//!  The gate's one live arm in the throwaway guest:
//! the layout checker reads the guest's own
//!  `/sys/kernel/btf/vmlinux` and `/sys/kernel/btf/zfs`,
//! compares the loaded module's
//!  declaration with this crate's,
//! and answers what the arm under test states it must:
//!  a pass on the approved module or the exact refusal a patched one earns.
//! Why:
//!  A pass alone shows nothing,
//! so the arm names what it expects in
//!  `MONOCHROMATIC_ZFS_FIXTURE_GATE_EXPECT`:
//! `compatible` for the module the
//!  recipe built,
//! `member:dds_flags` for a module that lost the flag `zfs-2.4.4`
//!  added,
//! `member:zi_iotype` for one whose two same-size inject-record members were
//!  swapped,
//! and `type:zc_inject_record` for one whose anonymous union is gone.
//!  The driver sets it per arm and reads the `MEASUREMENT gate` line,
//! so a run that
//!  never reached the arm is visible rather than green.
//!  A pass also shows the header length the checker reports is the one the
//! module's own file declares in its own header,
//! read from that file and never
//!  written as a number,
//! and that it is one of the two shapes the header reader knows,
//! so
//!  no accepted module is reported with a length no file stated.
//!  The arm opens no device:
//! it reads two sysfs files,
//! so a machine without
//!  `/dev/zfs` runs it,
//! and only the guest's own files are read.

use super::bounded_read::{
    ReadRequest,
    read_bounded,
};
use super::btf_header::{
    LAYOUT_HEADER_BYTES,
    OLDEST_HEADER_BYTES,
};
use super::refusal::Step;
use super::{
    Compatible,
    LayoutRequest,
    MODULE_BTF_BOUND,
    Refusal,
    check_module_layout,
};
use std::path::Path;

/// Where the guest exposes its two type-information files.
const BASE_BTF: &str = "/sys/kernel/btf/vmlinux";
/// The loaded module's own type information.
const MODULE_BTF: &str = "/sys/kernel/btf/zfs";
/// The loaded module's release string.
const MODULE_VERSION: &str = "/sys/module/zfs/version";

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_running_modules_declaration_answers_what_the_arm_expects() {
    let Some(guest): Option<std::ffi::OsString> =
        std::env::var_os("MONOCHROMATIC_ZFS_FIXTURE_GUEST")
    else {
        println!(
            "SKIP gate_arm: MONOCHROMATIC_ZFS_FIXTURE_GUEST is unset, so this arm was not run"
        );
        return;
    };
    let hostname: String = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .expect("the guest's hostname is readable")
        .trim()
        .to_string();
    assert_eq!(
        hostname,
        guest.to_string_lossy(),
        "this arm runs only in the guest the recipe made"
    );
    let expected: String =
        std::env::var("MONOCHROMATIC_ZFS_FIXTURE_GATE_EXPECT").unwrap_or_default();
    if expected.is_empty() {
        println!(
            "SKIP gate_arm: MONOCHROMATIC_ZFS_FIXTURE_GATE_EXPECT is unset, so this arm names no expectation"
        );
        return;
    }
    let release: String = std::fs::read_to_string(MODULE_VERSION)
        .expect("the guest's module reports a release")
        .trim()
        .to_string();
    let module: std::path::PathBuf = Path::new(MODULE_BTF).to_path_buf();
    let module_bytes: u64 = match std::fs::metadata(&module) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };
    let verdict: Result<Compatible, Refusal> = check_module_layout(&LayoutRequest {
        base_btf: Path::new(BASE_BTF),
        module_btf: module.as_path(),
    });
    let answer: String = match &verdict {
        Ok(compatible) => format!(
            "compatible base={} module={}",
            compatible.base_bytes, compatible.module_bytes
        ),
        Err(refusal) => format!("{} step={:?}", refusal.detail, refusal.step),
    };
    println!(
        "MEASUREMENT gate expected={expected} release={release} module_btf={module_bytes} verdict={answer}"
    );
    match expected.as_str() {
        "compatible" => {
            let Ok(compatible): Result<Compatible, Refusal> = verdict else {
                panic!(
                    "the approved declaration refused: {}",
                    verdict.err().unwrap()
                );
            };
            assert!(compatible.module_bytes > 0);
            // What: the module's own file, read again from the path the
            // checker read, through the crate's one bounded read.
            // Why: the header length the arm compares against must come from
            // the file's own bytes, not from the checker's report of them.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const moduleFile = readBounded({ path: module, bound: MODULE_BTF_BOUND, step: Step.ModuleBtf });
            // ```
            let module_file: Vec<u8> = read_bounded(&ReadRequest {
                path: module.as_path(),
                bound: MODULE_BTF_BOUND,
                step: Step::ModuleBtf,
            })
            .expect("the module's own type information is readable");
            let declared: usize = declared_header_bytes(&module_file);
            println!(
                "MEASUREMENT gate-header declared={declared} reported={} oldest={OLDEST_HEADER_BYTES} layout={LAYOUT_HEADER_BYTES}",
                compatible.module_header_bytes
            );
            assert!(
                declared == OLDEST_HEADER_BYTES || declared == LAYOUT_HEADER_BYTES,
                "the module's own header is {declared} bytes, and this reader knows only the {OLDEST_HEADER_BYTES}-byte and {LAYOUT_HEADER_BYTES}-byte shapes"
            );
            assert_eq!(
                compatible.module_header_bytes, declared,
                "the checker reports the header length the module's own file declares"
            );
        }
        "member:dds_flags" => {
            let Err(refusal): Result<Compatible, Refusal> = verdict else {
                panic!("a module without dds_flags passed the gate");
            };
            assert_eq!(refusal.step, Step::Member);
            assert!(refusal.detail.contains("dds_flags"), "{}", refusal.detail);
            assert!(refusal.detail.contains("2296"), "{}", refusal.detail);
        }
        "member:zi_iotype" => {
            let Err(refusal): Result<Compatible, Refusal> = verdict else {
                panic!("a module with zi_iotype and zi_duration swapped passed the gate");
            };
            assert_eq!(refusal.step, Step::Member);
            assert!(refusal.detail.contains("zi_iotype"), "{}", refusal.detail);
            assert!(refusal.detail.contains("2592"), "{}", refusal.detail);
            assert!(refusal.detail.contains("2560"), "{}", refusal.detail);
        }
        "type:zc_inject_record" => {
            let Err(refusal): Result<Compatible, Refusal> = verdict else {
                panic!("a module without the anonymous union passed the gate");
            };
            assert_eq!(refusal.step, Step::Type);
            assert!(
                refusal.detail.contains("zc_inject_record"),
                "{}",
                refusal.detail
            );
        }
        other => panic!("the driver named an expectation this arm does not know: {other}"),
    }
}

/// The header length a file states in its own first eight bytes,
/// kept to the field's four bytes
/// and parsed here rather than by the checker's own header reader,
/// so the arm compares two readings of one file and not one reading with
/// itself.
///
/// # Panics
/// When the file is shorter than the four bytes the field sits in.
fn declared_header_bytes(bytes: &[u8]) -> usize {
    // What: the field's four bytes, little-endian on this target, widened to
    // `usize` so they compare with the checker's own `usize` report.
    // Why: the type information format counts every section from this length.
    let field: &[u8] = bytes
        .get(4..8)
        .expect("the module's own type information holds its header length");
    return u32::from_le_bytes([field[0], field[1], field[2], field[3]]) as usize;
}
