//! The tests of the toolchain retirement:
//! an unpinned dated nightly goes,
//! and one a lock pins,
//! one a process runs or names,
//! one within the grace period,
//! and every name that is not a dated nightly stay.

use std::{cell::RefCell, collections::HashSet, fs, path::{Path, PathBuf}, process::{Child, Command}, time::Duration};

use crate::lean_identity::Deadline;
use crate::lean_rustup_toolchains::{Installed, ToolchainVerdict, Uninstaller, dated_nightly_name, installed_toolchains, pinned_by, retire_nightlies};
use crate::lean_test_support::{DAY, NOW, age_everything};
use crate::test_scratch::scratch;

/// The grace period of these tests.
const GRACE: i64 = 5 * DAY;

/// The triple rustup adds to a directory name.
const TRIPLE: &str = "x86_64-unknown-linux-gnu";

/// An uninstaller that removes the directory and records the name.
struct Recording<'a> {
    /// The toolchains directory.
    directory: &'a Path,
    /// The names asked for.
    asked: RefCell<Vec<String>>,
}

impl Uninstaller for Recording<'_> {
    fn uninstall(&self, directory_name: &str) -> Result<(), String> {
        self.asked.borrow_mut().push(directory_name.to_string());
        return fs::remove_dir_all(self.directory.join(directory_name)).map_err(|error| { return error.to_string(); });
    }
}

/// A child stopped when the guard drops.
struct Sleeper {
    /// The child.
    child: Child,
}

impl Drop for Sleeper {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Makes a toolchain directory aged by `idle` seconds.
fn toolchain(directory: &Path, name: &str, idle: i64) -> PathBuf {
    let path: PathBuf = directory.join(format!("{name}-{TRIPLE}"));
    fs::create_dir_all(path.join("bin")).expect("toolchain");
    fs::write(path.join("bin").join("rustc"), vec![3_u8; 8192]).expect("compiler");
    age_everything(&path, NOW - idle);
    return path;
}

/// Runs the retirement over a toolchains directory with a process table that holds no process,
/// so no real process of this machine (any of which may name a real nightly) decides a test.
fn retire(directory: &Path, pinned: &[&str]) -> (Vec<(String, ToolchainVerdict)>, Vec<String>) {
    return retire_in(directory, pinned, &directory.join("no-processes"));
}

/// Runs the retirement over a toolchains directory against the given process table.
fn retire_in(directory: &Path, pinned: &[&str], proc_root: &Path) -> (Vec<(String, ToolchainVerdict)>, Vec<String>) {
    let installed: Vec<Installed> = installed_toolchains(directory);
    let pinned_set: HashSet<String> = pinned.iter().map(|name| { return (*name).to_string(); }).collect();
    let uninstaller: Recording = Recording { directory, asked: RefCell::new(Vec::new()) };
    let mut log: Vec<String> = Vec::new();
    let verdicts = retire_nightlies(&installed, &pinned_set, proc_root, i32::try_from(std::process::id()).expect("pid"), GRACE, NOW, false, &uninstaller, &Deadline::after(Duration::from_secs(60)), &mut log);
    return (verdicts, uninstaller.asked.into_inner());
}

#[test]
fn only_dated_nightlies_are_candidates() {
    assert_eq!(dated_nightly_name("nightly-2026-11-07-x86_64-unknown-linux-gnu"), Some(String::from("nightly-2026-11-07")));
    assert_eq!(dated_nightly_name("nightly-2026-11-07"), Some(String::from("nightly-2026-11-07")));
    for other in ["nightly-x86_64-unknown-linux-gnu", "stable-x86_64-unknown-linux-gnu", "1.99.0-x86_64-unknown-linux-gnu", "nightly-2026-1-07-x", "nightly-2026-11-077"] {
        assert_eq!(dated_nightly_name(other), None, "{other}");
    }
}

#[test]
fn an_unpinned_old_nightly_is_uninstalled_and_the_names_that_are_not_nightlies_stay() {
    let root = scratch();
    let old: PathBuf = toolchain(root.path(), "nightly-2026-09-11", 40 * DAY);
    let stable: PathBuf = toolchain(root.path(), "stable", 90 * DAY);
    let release: PathBuf = toolchain(root.path(), "1.99.0", 90 * DAY);
    let channel: PathBuf = toolchain(root.path(), "nightly", 90 * DAY);
    let (verdicts, asked) = retire(root.path(), &[]);
    assert_eq!(asked, vec![format!("nightly-2026-09-11-{TRIPLE}")], "{verdicts:?}");
    assert!(!old.exists() && stable.exists() && release.exists() && channel.exists());
}

#[test]
fn a_nightly_a_checkouts_lock_pins_is_kept() {
    let root = scratch();
    let kept: PathBuf = toolchain(root.path(), "nightly-2026-11-01", 30 * DAY);
    let checkout: PathBuf = root.path().join("checkout");
    fs::create_dir_all(&checkout).expect("checkout");
    fs::write(checkout.join("toolchain.lock.json"), "{\"tools\":{\"rust\":{\"version\":\"nightly-2026-11-01\"}}}").expect("lock");
    let pinned: HashSet<String> = pinned_by(&[checkout], &root.path().join("no-settings.toml"));
    assert!(pinned.contains("nightly-2026-11-01"));
    let names: Vec<&str> = pinned.iter().map(String::as_str).collect();
    let (verdicts, asked) = retire(root.path(), &names);
    assert!(asked.is_empty(), "{verdicts:?}");
    assert!(kept.exists());
}

#[test]
fn a_nightly_only_a_checkouts_environment_file_names_is_kept_and_so_are_rustups_default_and_overrides() {
    let root = scratch();
    let checkout: PathBuf = root.path().join("checkout");
    fs::create_dir_all(checkout.join(".cache")).expect("cache");
    fs::write(checkout.join(".cache").join("toolchain.env.json"), "{\"RUSTUP_TOOLCHAIN\":\"nightly-2026-10-31\"}").expect("environment file");
    let settings: PathBuf = root.path().join("settings.toml");
    fs::write(&settings, "default_toolchain = \"nightly-2026-10-12\"\n\n[overrides]\n\"/some/dir\" = \"nightly-2026-10-13\"\n").expect("settings");
    let pinned: HashSet<String> = pinned_by(&[checkout], &settings);
    for name in ["nightly-2026-10-31", "nightly-2026-10-12", "nightly-2026-10-13"] {
        assert!(pinned.contains(name), "{name}");
    }
}

#[test]
fn a_nightly_within_the_grace_period_is_kept() {
    let root = scratch();
    let young: PathBuf = toolchain(root.path(), "nightly-2026-11-07", DAY);
    let (verdicts, asked) = retire(root.path(), &[]);
    assert!(asked.is_empty() && young.exists(), "{verdicts:?}");
}

#[test]
fn a_nightly_a_process_runs_a_program_from_is_kept() {
    let root = scratch();
    let used: PathBuf = toolchain(root.path(), "nightly-2031-01-02", 40 * DAY);
    let program: PathBuf = used.join("bin").join("fake-rustc");
    fs::copy("/usr/bin/sleep", &program).expect("copy a program in");
    let child: Child = Command::new(&program).arg("120").spawn().expect("run it");
    let _holder: Sleeper = Sleeper { child };
    let (verdicts, asked) = retire_in(root.path(), &[], Path::new("/proc"));
    assert!(asked.is_empty(), "{verdicts:?}");
    assert!(used.exists());
}

#[test]
fn a_nightly_a_process_names_in_its_environment_is_kept() {
    let root = scratch();
    let named: PathBuf = toolchain(root.path(), "nightly-2031-01-03", 40 * DAY);
    let child: Child = Command::new("sleep").arg("120").env("RUSTUP_TOOLCHAIN", "nightly-2031-01-03").spawn().expect("run it");
    let _holder: Sleeper = Sleeper { child };
    let (verdicts, asked) = retire_in(root.path(), &[], Path::new("/proc"));
    assert!(asked.is_empty(), "{verdicts:?}");
    assert!(named.exists());
}
