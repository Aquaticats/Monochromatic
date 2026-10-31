//! What:
//!  Controls of the offline Dylint driver check against stub programs.
//! Why:
//!  The check decides between a skip and a run from three machine facts:
//!  the driver's version,
//!  the cargo cache and the active toolchain.
//!  Each fact is a stub's answer,
//!  so every branch runs without a network,
//!  a nightly toolchain or `cargo-dylint`:
//!  the stub `cargo` answers the offline resolution,
//!  another copy of the stub answers `rustup`,
//!  and a third copy sits in the driver directory as `dylint-driver`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a stale driver and a cold cache is missing', () => expect(driverState(cargo, site, lib, [6, 3, 0], warm)).toMatchObject({ kind: 'missing' }));
//! ```

use super::{
    DriverSite, DriverState, driver_gaps, driver_state, parse_version, probe_manifest, warm_command,
};
use crate::{
    closure_packages::ClosurePackage,
    closure_stub_support::{Scenario, scenario, stub_cargo},
    test_fs::Fixture,
};
use std::path::{Path, PathBuf};

/// The toolchain name that the `rustup` stub reports.
const TOOLCHAIN: &str = "nightly-fixture-toolchain";

/// The version of `cargo-dylint` in the controls.
const NEEDED: (u64, u64, u64) = (6, 3, 0);

/// The stub's answer files that make the offline resolution of the driver crate fail.
const COLD: [(&str, &str); 2] = [
    ("metadata.dylint-driver-probe.status", "101"),
    (
        "metadata.dylint-driver-probe.stderr",
        "     Locking 81 packages to highest compatible versions\nerror: failed to select a version for the requirement `dylint_driver = \"=6.3.0\"`\nmore\n",
    ),
];

/// The machine of one control:
/// a `cargo` stub,
/// a `rustup` stub and the directories below the `cargo` stub's scenario directory.
struct Machine {
    /// The stub that answers `cargo metadata`.
    cargo: Scenario,
    /// The stub that answers `rustup show active-toolchain`.
    rustup: Scenario,
    /// The library directory that `rustup` runs in.
    library: Fixture,
}

impl Machine {
    /// A machine whose `cargo` stub has the given answers and whose `rustup` names the fixture toolchain.
    fn new(cargo_answers: &[(&str, &str)]) -> Machine {
        return Machine {
            cargo: scenario(cargo_answers),
            rustup: scenario(&[("show.stdout", format!("{TOOLCHAIN} (default)\n").as_str())]),
            library: Fixture::new(),
        };
    }

    /// The driver directory.
    fn drivers(&self) -> PathBuf {
        return self.cargo.directory.join("drivers");
    }

    /// The scratch directory for the throwaway crate.
    fn scratch(&self) -> PathBuf {
        return self.cargo.directory.join("scratch");
    }

    /// The machine facts with every directory known.
    fn site(&self) -> DriverSite {
        return DriverSite {
            rustup: self.rustup.cargo.clone(),
            drivers: Some(self.drivers()),
            scratch: Some(self.scratch()),
        };
    }

    /// Install a driver for the fixture toolchain that answers `-V` with the given files.
    fn install_driver(&self, answers: &[(&str, &str)]) {
        let directory: PathBuf = self.drivers().join(TOOLCHAIN);
        std::fs::create_dir_all(&directory).expect("driver directory");
        std::fs::hard_link(stub_cargo(), directory.join("dylint-driver")).expect("driver link");
        for (file, text) in answers {
            std::fs::write(directory.join(file), text).expect("driver answer");
        }
    }

    /// A package of the grouping below the library fixture's directory;
    /// the library directory exists, because `rustup` runs in it.
    fn package(&self, directory: &str, library: &str) -> ClosurePackage {
        std::fs::create_dir_all(self.library.path.join(library)).expect("library directory");
        return ClosurePackage {
            manifest: self.library.path.join(directory).join("Cargo.toml"),
            library: self.library.path.join(library),
            configuration: String::from("monochromatic-lint.config.jsonc"),
        };
    }

    /// Ask for the state of the driver with the machine's own facts.
    fn state(&self) -> DriverState {
        return driver_state(
            &self.cargo.cargo,
            &self.site(),
            &self.library.path,
            NEEDED,
            "WARM",
        );
    }
}

/// A plain dotted version is read from the last word,
/// and anything else is not a version.
#[test]
fn versions_are_read_from_the_last_word() {
    assert_eq!(parse_version("cargo-dylint 6.3.0\n"), Some((6, 3, 0)));
    assert_eq!(parse_version("dylint-driver 10.20.30"), Some((10, 20, 30)));
    assert_eq!(parse_version("6.3.0"), Some((6, 3, 0)));
    assert_eq!(parse_version(""), None);
    assert_eq!(parse_version("   \n"), None);
    assert_eq!(parse_version("cargo-dylint"), None);
    assert_eq!(parse_version("cargo-dylint 6.3"), None);
    assert_eq!(parse_version("cargo-dylint 6.3.0.1"), None);
    assert_eq!(parse_version("cargo-dylint 6.3.0-rc.1"), None);
    assert_eq!(parse_version("cargo-dylint 6.x.0"), None);
    assert_eq!(parse_version("cargo-dylint -6.3.0"), None);
}

/// The throwaway crate pins the version exactly,
/// as `cargo-dylint` does,
/// and stays out of any enclosing workspace.
#[test]
fn the_probe_crate_pins_the_driver_exactly() {
    let manifest: String = probe_manifest("6.3.0");
    assert!(
        manifest.contains("dylint_driver = { version = \"=6.3.0\" }"),
        "{manifest}"
    );
    assert!(manifest.contains("\n[workspace]\n"), "{manifest}");
    assert!(manifest.contains("anyhow = \"1.0\""), "{manifest}");
    assert!(manifest.contains("env_logger = \"0.11\""), "{manifest}");
}

/// The warm command unsets the offline variable and checks one package with the library.
#[test]
fn the_warm_command_runs_without_the_offline_variable() {
    assert_eq!(
        warm_command("lint lib", "my pkg/Cargo.toml"),
        "env --unset=CARGO_NET_OFFLINE cargo dylint --path lint lib --manifest-path my pkg/Cargo.toml -- --locked"
    );
}

/// A driver as new as `cargo-dylint` is reused:
/// nothing is resolved,
/// and the toolchain comes from `rustup show active-toolchain` run in the library.
#[test]
fn a_current_driver_is_ready_without_resolving_anything() {
    let machine: Machine = Machine::new(&COLD);
    machine.install_driver(&[("-V.stdout", "dylint-driver 6.3.0\n")]);
    assert_eq!(machine.state(), DriverState::Ready);
    assert!(
        machine.cargo.calls().is_empty(),
        "{:?}",
        machine.cargo.calls()
    );
    assert_eq!(
        machine.rustup.calls(),
        vec![String::from("show|show active-toolchain|")]
    );
}

/// A driver newer than `cargo-dylint` is also reused.
#[test]
fn a_newer_driver_is_ready() {
    let machine: Machine = Machine::new(&COLD);
    machine.install_driver(&[("-V.stdout", "dylint-driver 6.10.0\n")]);
    assert_eq!(machine.state(), DriverState::Ready);
}

/// An outdated driver with a cold cache is missing,
/// and the reason names the toolchain,
/// both versions,
/// the driver path,
/// the warm command and the first error line of cargo's message only.
#[test]
fn an_outdated_driver_and_a_cold_cache_are_missing() {
    let machine: Machine = Machine::new(&COLD);
    machine.install_driver(&[("-V.stdout", "dylint-driver 6.2.0\n")]);
    let DriverState::Missing(reason) = machine.state() else {
        panic!("the driver should be missing");
    };
    assert!(reason.contains(TOOLCHAIN), "{reason}");
    assert!(reason.contains("reports version 6.2.0"), "{reason}");
    assert!(reason.contains("6.3.0 of cargo-dylint"), "{reason}");
    assert!(reason.contains("`dylint_driver` 6.3.0"), "{reason}");
    assert!(reason.contains("`WARM`"), "{reason}");
    assert!(reason.contains("which this linter does not do"), "{reason}");
    assert!(
        reason.contains(
            machine
                .drivers()
                .join(TOOLCHAIN)
                .join("dylint-driver")
                .to_string_lossy()
                .as_ref()
        ),
        "{reason}"
    );
    assert!(
        reason.contains("failed to select a version for the requirement"),
        "{reason}"
    );
    assert!(
        reason.contains("Cargo's error: failed to select")
            && !reason.contains("more")
            && !reason.contains("Locking"),
        "only the first error line of cargo's message is quoted, not its progress notes: {reason}"
    );
}

/// A refusal without an error line quotes the first line,
/// and one without any output says so.
#[test]
fn a_refusal_without_an_error_line_quotes_what_there_is() {
    let plain: Machine = Machine::new(&[
        ("metadata.dylint-driver-probe.status", "101"),
        ("metadata.dylint-driver-probe.stderr", "plain words\nmore\n"),
    ]);
    let DriverState::Missing(reason) = plain.state() else {
        panic!("the driver should be missing");
    };
    assert!(reason.ends_with("Cargo's error: plain words"), "{reason}");
    let silent: Machine = Machine::new(&[("metadata.dylint-driver-probe.status", "101")]);
    let DriverState::Missing(none) = silent.state() else {
        panic!("the driver should be missing");
    };
    assert!(none.ends_with("Cargo's error: none"), "{none}");
}

/// A missing driver with a cold cache is missing,
/// and the reason says the driver is absent.
#[test]
fn an_absent_driver_and_a_cold_cache_are_missing() {
    let machine: Machine = Machine::new(&COLD);
    let DriverState::Missing(reason) = machine.state() else {
        panic!("the driver should be missing");
    };
    assert!(
        reason.contains("is absent (cargo-dylint is 6.3.0)"),
        "{reason}"
    );
}

/// A driver that fails to report its version counts as outdated,
/// as it does for `cargo-dylint`.
#[test]
fn a_driver_that_fails_or_prints_no_version_is_outdated() {
    let failing: Machine = Machine::new(&COLD);
    failing.install_driver(&[("-V.status", "1"), ("-V.stdout", "dylint-driver 9.9.9\n")]);
    assert!(matches!(failing.state(), DriverState::Missing(_)));
    let silent: Machine = Machine::new(&COLD);
    silent.install_driver(&[]);
    assert!(matches!(silent.state(), DriverState::Missing(_)));
    let garbled: Machine = Machine::new(&COLD);
    garbled.install_driver(&[("-V.stdout", "dylint-driver six\n")]);
    assert!(matches!(garbled.state(), DriverState::Missing(_)));
}

/// An outdated or absent driver is acceptable when the cargo cache resolves the pinned crate,
/// because `cargo dylint` then builds it offline.
/// The resolution runs offline on the scratch crate,
/// which is removed afterwards.
#[test]
fn a_warm_cache_makes_a_stale_driver_ready() {
    let machine: Machine = Machine::new(&[]);
    machine.install_driver(&[("-V.stdout", "dylint-driver 6.2.0\n")]);
    assert_eq!(machine.state(), DriverState::Ready);
    let calls: Vec<String> = machine.cargo.calls();
    assert_eq!(calls.len(), 1, "{calls:?}");
    let expected_prefix: &str = "metadata|metadata --offline --format-version 1 --manifest-path ";
    assert!(calls[0].starts_with(expected_prefix), "{}", calls[0]);
    assert!(
        calls[0].ends_with(
            format!(
                "driver-probe-{}/dylint-driver-probe/Cargo.toml|",
                std::process::id()
            )
            .as_str()
        ),
        "{}",
        calls[0]
    );
    assert!(
        std::fs::read_dir(machine.scratch())
            .expect("the scratch directory exists")
            .next()
            .is_none(),
        "the throwaway crate is removed"
    );
    let absent: Machine = Machine::new(&[]);
    assert_eq!(absent.state(), DriverState::Ready);
}

/// A scratch directory that cannot be written is a check that could not be made,
/// not a skip,
/// and a missing scratch location is the same when the driver is not current.
#[test]
fn an_unwritable_or_unknown_scratch_is_unknown() {
    let machine: Machine = Machine::new(&COLD);
    std::fs::write(machine.scratch(), "a file, not a directory").expect("scratch is a file");
    let DriverState::Unknown(note) = machine.state() else {
        panic!("the check could not be made");
    };
    assert!(note.contains("cannot be written"), "{note}");
    assert!(machine.cargo.calls().is_empty());
    let none: Machine = Machine::new(&COLD);
    let site: DriverSite = DriverSite {
        scratch: None,
        ..none.site()
    };
    let DriverState::Unknown(unknown) =
        driver_state(&none.cargo.cargo, &site, &none.library.path, NEEDED, "WARM")
    else {
        panic!("the check could not be made");
    };
    assert!(unknown.contains("no cache directory"), "{unknown}");
    let current: Machine = Machine::new(&COLD);
    current.install_driver(&[("-V.stdout", "dylint-driver 6.3.0\n")]);
    let ready: DriverState = driver_state(
        &current.cargo.cargo,
        &DriverSite {
            scratch: None,
            ..current.site()
        },
        &current.library.path,
        NEEDED,
        "WARM",
    );
    assert_eq!(
        ready,
        DriverState::Ready,
        "a current driver needs no scratch"
    );
}

/// Without a known driver directory,
/// with a failing `rustup`,
/// with a `rustup` that names nothing,
/// with a missing `rustup`,
/// and with a missing `cargo` for the resolution,
/// the check could not be made.
#[test]
fn facts_that_cannot_be_read_make_the_check_unknown() {
    let machine: Machine = Machine::new(&COLD);
    let unlocated: DriverSite = DriverSite {
        drivers: None,
        ..machine.site()
    };
    let DriverState::Unknown(drivers) = driver_state(
        &machine.cargo.cargo,
        &unlocated,
        &machine.library.path,
        NEEDED,
        "WARM",
    ) else {
        panic!("unknown");
    };
    assert!(drivers.contains("no driver directory"), "{drivers}");
    let failing: Scenario = scenario(&[("show.status", "1")]);
    let failing_site: DriverSite = DriverSite {
        rustup: failing.cargo.clone(),
        ..machine.site()
    };
    let DriverState::Unknown(failed) = driver_state(
        &machine.cargo.cargo,
        &failing_site,
        &machine.library.path,
        NEEDED,
        "WARM",
    ) else {
        panic!("unknown");
    };
    assert!(failed.contains("rustup show active-toolchain"), "{failed}");
    let silent: Scenario = scenario(&[]);
    let silent_site: DriverSite = DriverSite {
        rustup: silent.cargo.clone(),
        ..machine.site()
    };
    let DriverState::Unknown(nothing) = driver_state(
        &machine.cargo.cargo,
        &silent_site,
        &machine.library.path,
        NEEDED,
        "WARM",
    ) else {
        panic!("unknown");
    };
    assert!(nothing.contains("printed no toolchain"), "{nothing}");
    let absent_site: DriverSite = DriverSite {
        rustup: PathBuf::from("/nonexistent/monochromatic-lint/rustup"),
        ..machine.site()
    };
    let DriverState::Unknown(missing) = driver_state(
        &machine.cargo.cargo,
        &absent_site,
        &machine.library.path,
        NEEDED,
        "WARM",
    ) else {
        panic!("unknown");
    };
    assert!(missing.contains("rustup does not run"), "{missing}");
    let DriverState::Unknown(no_cargo) = driver_state(
        Path::new("/nonexistent/monochromatic-lint/cargo"),
        &machine.site(),
        &machine.library.path,
        NEEDED,
        "WARM",
    ) else {
        panic!("unknown");
    };
    assert!(no_cargo.contains("cargo does not run"), "{no_cargo}");
}

/// A version line that is not readable is a note and no check runs.
#[test]
fn an_unreadable_cargo_dylint_version_is_a_note() {
    let machine: Machine = Machine::new(&COLD);
    let found = driver_gaps(
        &[machine.package("a", "lib")],
        &machine.cargo.cargo,
        &machine.site(),
        "\n",
        &machine.library.path,
    );
    assert!(found.gaps.is_empty());
    assert_eq!(found.notes.len(), 1);
    assert!(
        found.notes[0].contains("printed no readable version"),
        "{:?}",
        found.notes
    );
    assert!(machine.rustup.calls().is_empty());
}

/// Packages that share a library are checked once and skipped together,
/// and the message names them as all packages.
#[test]
fn packages_of_one_library_share_one_gap() {
    let machine: Machine = Machine::new(&COLD);
    let found = driver_gaps(
        &[machine.package("a", "lib"), machine.package("b", "lib")],
        &machine.cargo.cargo,
        &machine.site(),
        "cargo-dylint 6.3.0\n",
        &machine.library.path,
    );
    assert_eq!(machine.rustup.calls().len(), 1, "one library, one check");
    assert_eq!(found.gaps.len(), 1);
    assert_eq!(found.gaps[0].library, machine.library.path.join("lib"));
    assert_eq!(found.gaps[0].subject, "all 2 package(s)");
    assert!(
        found.gaps[0]
            .reason
            .contains("`env --unset=CARGO_NET_OFFLINE cargo dylint --path lib --manifest-path a/Cargo.toml -- --locked`"),
        "{}",
        found.gaps[0].reason
    );
}

/// Libraries are checked one by one,
/// a ready one is not a gap,
/// and a gap that covers only some packages names them and the library.
#[test]
fn each_library_has_its_own_verdict() {
    let machine: Machine = Machine::new(&COLD);
    let found = driver_gaps(
        &[
            machine.package("a", "lib"),
            machine.package("b", "other"),
            machine.package("c", "other"),
        ],
        &machine.cargo.cargo,
        &machine.site(),
        "cargo-dylint 6.3.0\n",
        &machine.library.path,
    );
    assert_eq!(machine.rustup.calls().len(), 2);
    assert_eq!(found.gaps.len(), 2);
    assert_eq!(
        found.gaps[0].subject,
        "1 package(s) checked with the library lib"
    );
    assert_eq!(
        found.gaps[1].subject,
        "2 package(s) checked with the library other"
    );
    let ready: Machine = Machine::new(&COLD);
    ready.install_driver(&[("-V.stdout", "dylint-driver 6.3.0\n")]);
    let none = driver_gaps(
        &[ready.package("a", "lib")],
        &ready.cargo.cargo,
        &ready.site(),
        "cargo-dylint 6.3.0\n",
        &ready.library.path,
    );
    assert!(none.gaps.is_empty() && none.notes.is_empty());
}

/// A check that could not be made is a note that names the library.
#[test]
fn an_unknown_state_is_a_note_that_names_the_library() {
    let machine: Machine = Machine::new(&COLD);
    let unlocated: DriverSite = DriverSite {
        drivers: None,
        ..machine.site()
    };
    let found = driver_gaps(
        &[machine.package("a", "lib")],
        &machine.cargo.cargo,
        &unlocated,
        "cargo-dylint 6.3.0\n",
        &machine.library.path,
    );
    assert!(found.gaps.is_empty());
    assert!(
        found.notes[0].contains("for the library lib did not run: no driver directory"),
        "{:?}",
        found.notes
    );
}
