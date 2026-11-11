//! What:
//! Unit tests of the farm link that makes the running `mpm-shadow` reachable under its own name.
//! Why:
//! Every sentence `mpm-shadow` prints names `mpm-shadow` as the program to run,
//! and a shell outside a checkout reaches only the scope's exposure farm,
//! so the farm must hold that name or the printed command cannot run as printed.

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::os::unix::fs`.
use std::os::unix::fs::symlink;
/// Imports what this module uses from `std::path`.
use std::path::PathBuf;

/// Imports `current_link` from `super::cargo_installs`.
use super::cargo_installs::current_link;
/// Imports `rebuild_exposure`, `expose_running_shadow` from `super::exposure`.
use super::exposure::{expose_running_shadow, rebuild_exposure};
/// Imports what this module uses from `super::fakes`.
use super::fakes::{Rig, write_executable};
/// Imports `parse_record`, `record_path` from `super::launcher`.
use super::launcher::{parse_record, record_path};

/// The id the sample plan gives `mpm-shadow`.
const SHADOW_TOOL_ID: &str = "cargo:monochromatic-mpm-shadow";

/// The crate the sample plan locks for `mpm-shadow`.
const SHADOW_CRATE: &str = "monochromatic-mpm-shadow";

/// The version the fixture has installed,
/// which no lock in the fixture pins.
const INSTALLED: &str = "0.2.0";

/// What:
/// Puts a finished `cargo-binstall` install of `mpm-shadow` at `INSTALLED` in the rig's own toolchain root,
/// and hands back the program inside it,
/// the path a run of that install has as its own.
/// Why:
/// The farm link names an installed program, so the fixture must be one:
/// a version directory with cargo's record,
/// the program below `bin`,
/// and the tool's `current` link naming the version.
fn installed_shadow(rig: &Rig) -> PathBuf {
    let version_dir: PathBuf = rig.scope.toolchain_root.join("cargo").join(SHADOW_CRATE).join(INSTALLED);
    let program: PathBuf = version_dir.join("bin").join("mpm-shadow");
    write_executable(&program);
    fs::write(
        version_dir.join(".crates.toml"),
        format!("[v1]\n\"{SHADOW_CRATE} {INSTALLED} (registry+https://example.invalid/index)\" = [\"mpm-shadow\"]\n"),
    )
    .expect("write cargo's record");
    symlink(INSTALLED, current_link(&rig.scope.toolchain_root, SHADOW_CRATE)).expect("link current");
    return program;
}

/// A program of the same name that belongs to no install:
/// a build folder has no `current` link above it.
fn uninstalled_shadow(rig: &Rig) -> PathBuf {
    let program: PathBuf = rig.scope.root.join("build").join("mpm-shadow");
    write_executable(&program);
    return program;
}

#[test]
fn the_farm_links_the_running_shadow_under_its_own_name() {
    let mut rig = Rig::new();
    rig.lock.tools.retain(|tool| return tool.declaration.id != SHADOW_TOOL_ID);
    let program: PathBuf = installed_shadow(&rig);

    let linked: Option<String> = expose_running_shadow(&rig.context(), &program).expect("the farm is writable");

    assert_eq!(linked, Some("mpm-shadow".to_string()));
    let link: PathBuf = rig.scope.exposure_dir.join("mpm-shadow");
    assert_eq!(
        fs::read_link(&link).expect("a symlink"),
        current_link(&rig.scope.toolchain_root, SHADOW_CRATE).join("bin").join("mpm-shadow"),
        "the link names the program through the tool's `current` link, so a later version move never leaves it dangling"
    );
    assert_eq!(
        fs::canonicalize(&link).expect("the link resolves"),
        fs::canonicalize(&program).expect("the installed program exists"),
        "a shell that types `mpm-shadow` runs the installed program"
    );
}

#[test]
fn a_shadow_that_belongs_to_no_install_links_nothing() {
    let mut rig = Rig::new();
    rig.lock.tools.retain(|tool| return tool.declaration.id != SHADOW_TOOL_ID);

    let linked: Option<String> = expose_running_shadow(&rig.context(), &uninstalled_shadow(&rig)).expect("nothing to do");

    assert_eq!(linked, None, "a build folder is not an install, so it never goes on the machine's PATH");
    assert!(!rig.scope.exposure_dir.join("mpm-shadow").exists(), "the farm holds no link for it");
}

#[test]
fn a_plan_that_declares_the_name_keeps_its_own_link() {
    let rig = Rig::new();
    let program: PathBuf = installed_shadow(&rig);

    let linked: Option<String> = expose_running_shadow(&rig.context(), &program).expect("nothing to do");

    assert_eq!(linked, None, "the plan's own row owns the farm link for the bin it declares");
    let tool = rig.tool(SHADOW_TOOL_ID);
    rebuild_exposure(&rig.context(), &[&tool]).expect("rebuilt");
    assert_eq!(
        fs::read_link(rig.scope.exposure_dir.join("mpm-shadow")).expect("a symlink"),
        rig.scope.launcher_dir.join("mpm-shadow"),
        "a declared bin goes through this tree's launcher, so its record warns about that row"
    );
    let record = parse_record(&fs::read_to_string(record_path(&rig.scope.launcher_dir.join("mpm-shadow"))).expect("a record"))
        .expect("it parses");
    assert_eq!(record.tool_id, SHADOW_TOOL_ID, "the record names the row the farm link belongs to");
}
