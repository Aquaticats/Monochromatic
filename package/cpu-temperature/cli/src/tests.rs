//! Exercises the CLI and one running controller using a disposable sysfs tree.

/// Parsing the real command arguments exercises Clap's generated grammar.
use clap::Parser;
/// Standard fixture filesystem operations.
use std::{fs, path::{Path, PathBuf}};
/// A fixture owns its directory and removes it when the test ends.
use tempfile::TempDir;

/// Parse the same command the user runs.
use crate::{Cli, run};
/// Exercise the actual post-stop cleanup entry point's policy code.
use crate::policy::release_all;
/// Parse explicit sensor identities and construct the same watcher as the service.
use crate::{sensor::parse_sensor, service::{Paths, Watcher}};
/// Publish and read real runtime requests.
use crate::target::{Target, read_target};

/// Fixture values in the units exposed by Linux.
const FLOOR: u64 = 2_000_000;
const MAXIMUM: u64 = 6_000_000;
const TARGET: i32 = 80_000;
const MARGIN: u16 = 3;

/// Own the tree supplying the watcher and stage CPUFreq attributes in it.
struct Fixture {
    /// Private directory for every write in this test.
    directory: TempDir,
    /// Hwmon chip selected by driver and label.
    chip: PathBuf,
}

impl Fixture {
    /// Create one policy and one hwmon chip with two temperature inputs.
    fn new() -> Self {
        // What
        // `expect` throws a test failure if the operation fails, adding the provided message.
        // Why
        // Failed fixture setup should report which step failed instead of reaching sysfs.
        // TypeScript pseudocode
        // const directory = createDisposableDirectoryOrThrow(join(packageRoot, "target"));
        let build = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        // Individual fixtures are isolated under the existing build directory.
        fs::create_dir_all(&build).expect("create fixture build directory");
        // Pass the build directory to tempfile so no test state reaches the global temporary directory.
        let directory = TempDir::new_in(build).expect("create private CPU temperature fixture");
        // Build paths inside the disposable directory.
        let chip = directory.path().join("hwmon/hwmon4");
        // Stage names and sensor values using the same attribute spellings as Linux.
        fs::create_dir_all(&chip).expect("create hwmon chip");
        fs::write(chip.join("name"), "demo-chip\n").expect("write chip name");
        fs::write(chip.join("temp1_label"), "Package id 0\n").expect("write package label");
        fs::write(chip.join("temp2_label"), "Chiplet 1\n").expect("write chiplet label");
        // Store both paths after taking an owned copy of the chip's path.
        let result = Self { directory, chip };
        // Use an explicitly lower sample for the package, so selection of the hotter chiplet matters.
        result.sample(TARGET);
        result.policy("policy0");
        return result;
    }

    /// Locate the target file and journal expected by the CLI.
    fn runtime(&self) -> &Path {
        // Return the fixture's owned root as a borrowed path.
        return self.directory.path();
    }

    /// Stage a fresh CPUFreq policy with its own attributes.
    fn policy(&self, name: &str) -> PathBuf {
        // Add a CPUFreq policy without changing any existing one.
        let path = self.runtime().join("cpufreq").join(name);
        // Create the directory tree for the named policy.
        fs::create_dir_all(&path).expect("create CPUFreq policy");
        // Populate the fixture's Linux frequency attributes.
        fs::write(path.join("cpuinfo_max_freq"), MAXIMUM.to_string()).expect("write hardware maximum");
        fs::write(path.join("scaling_min_freq"), FLOOR.to_string()).expect("write frequency floor");
        fs::write(path.join("scaling_max_freq"), MAXIMUM.to_string()).expect("write initial ceiling");
        return path;
    }

    /// Set the hotter chiplet reading in millidegrees.
    fn sample(&self, value: i32) {
        // Give the package a valid lower reading, making the source of the cap observable.
        fs::write(self.chip.join("temp1_input"), (value - i32::from(MARGIN) * 1_000).to_string())
            .expect("write package sample");
        // Write the value the control loop should use.
        fs::write(self.chip.join("temp2_input"), value.to_string()).expect("write chiplet sample");
    }

    /// Submit a request through the command-line parser and dispatcher.
    fn request(&self, value: &str) {
        // Format the existing disposable runtime directory as an argument.
        let runtime = self.runtime().to_str().expect("UTF-8 fixture path");
        // Use the same syntax as the user's command.
        let cli = Cli::try_parse_from(["setMaxCpuTemp", "--runtime-dir", runtime, value])
            .expect("parse live target command");
        // Write the real target protocol into the fixture.
        run(cli).expect("publish live target");
    }

    /// Read a policy's currently requested ceiling.
    fn ceiling(&self, name: &str) -> u64 {
        // Use the fixture path exclusively and parse the written value.
        let path = self.runtime().join("cpufreq").join(name).join("scaling_max_freq");
        let text = fs::read_to_string(path).expect("read test frequency ceiling");
        return text.trim().parse().expect("numeric test frequency ceiling");
    }

    /// Create the real watcher with both channel selectors.
    fn watcher(&self) -> Watcher {
        // Borrow configured text while constructing the owned selectors.
        let package = parse_sensor("demo-chip:Package id 0").expect("parse label selector");
        let chiplet = parse_sensor("demo-chip:temp2_input").expect("parse raw input selector");
        // All three runtime paths point inside the same fixture.
        let paths = Paths {
            runtime: self.runtime().to_path_buf(),
            hwmon: self.runtime().join("hwmon"),
            cpufreq: self.runtime().join("cpufreq"),
        };
        // Transfer ownership of the configuration to the watcher.
        return Watcher::new(paths, vec![package, chiplet], MARGIN);
    }
}

/// Map a percentage to the fixture's known policy range.
fn at(percent: u64) -> u64 {
    return FLOOR + (MAXIMUM - FLOOR) * percent / 100;
}

/// A different target and explicit off take effect in the same watcher.
#[test]
fn changes_target_and_releases_live() {
    let fixture = Fixture::new();
    // Add a second policy so a one-policy implementation would fail this test.
    fixture.policy("policy1");
    let mut watcher = fixture.watcher();
    fixture.request("80");
    // Feed the same hot sample at the new target and inspect both ceiling writes.
    watcher.step().expect("apply 80 degree target");
    assert_eq!(fixture.ceiling("policy0"), at(80));
    assert_eq!(fixture.ceiling("policy1"), at(80));
    fixture.request("95");
    // Raising the target gives room for a gradual restoration on the next pass.
    watcher.step().expect("raise target to 95 degrees");
    assert_eq!(fixture.ceiling("policy0"), at(85));
    fixture.request("off");
    // The off token restores every journaled request in one pass.
    watcher.step().expect("release both policies");
    assert_eq!(fixture.ceiling("policy0"), MAXIMUM);
    assert_eq!(fixture.ceiling("policy1"), MAXIMUM);
}

/// A new CPUFreq policy receives the current ceiling even in the hysteresis interval.
#[test]
fn caps_a_policy_that_comes_online() {
    let fixture = Fixture::new();
    let mut watcher = fixture.watcher();
    fixture.request("80");
    watcher.step().expect("cap the first policy");
    // Put the sample between the heat and recovery thresholds.
    fixture.sample(TARGET - i32::from(MARGIN) * 1_500);
    fixture.policy("policy1");
    watcher.step().expect("apply existing ceiling to the new policy");
    assert_eq!(fixture.ceiling("policy1"), fixture.ceiling("policy0"));
}

/// A missing selected input caps at the floor and is re-resolved on recovery.
#[test]
fn holds_the_floor_after_sensor_loss() {
    let fixture = Fixture::new();
    let mut watcher = fixture.watcher();
    fixture.request("80");
    watcher.step().expect("resolve fixture sensors");
    // Simulate a device returning at a different index, with its original identity.
    let new_chip = fixture.runtime().join("hwmon/hwmon9");
    fs::rename(&fixture.chip, &new_chip).expect("re-enumerate chip");
    assert!(watcher.step().is_err());
    assert_eq!(fixture.ceiling("policy0"), FLOOR);
    // A cool reading at the new path allows the controller to raise the ceiling.
    fs::write(new_chip.join("temp2_input"), "50000").expect("write cool chiplet sample");
    fs::write(new_chip.join("temp1_input"), "50000").expect("write cool package sample");
    watcher.step().expect("rediscover the sensor");
    assert_eq!(fixture.ceiling("policy0"), at(5));
}

/// Other policies still receive and release caps when one writer fails.
#[test]
fn retries_failed_policy_release_after_service_exit() {
    let fixture = Fixture::new();
    // Make the second ceiling absent to exercise a failing sysfs write.
    let second = fixture.policy("policy1");
    let second_ceiling = second.join("scaling_max_freq");
    fs::remove_file(&second_ceiling).expect("remove one test ceiling");
    let mut watcher = fixture.watcher();
    fixture.request("80");
    assert!(watcher.step().is_err());
    assert_eq!(fixture.ceiling("policy0"), at(80));
    // Call the same release logic as ExecStopPost without using the watcher's in-memory state.
    let cpufreq = fixture.runtime().join("cpufreq");
    assert!(release_all(&cpufreq, fixture.runtime()).is_err());
    assert_eq!(fixture.ceiling("policy0"), MAXIMUM);
    // Restore access and verify the journal retries only the still-pending policy.
    fs::write(second_ceiling, FLOOR.to_string()).expect("make the failed ceiling writable again");
    release_all(&cpufreq, fixture.runtime()).expect("retry remaining release");
    assert_eq!(fixture.ceiling("policy1"), MAXIMUM);
}

/// Clap rejects malformed targets and accepts the arguments used by the service unit.
#[test]
fn validates_command_line_and_service_configuration() {
    for invalid in ["0", "-1", "80.5", "65536", "disable"] {
        assert!(Cli::try_parse_from(["setMaxCpuTemp", invalid]).is_err());
    }
    assert!(Cli::try_parse_from(["setMaxCpuTemp"]).is_err());
    assert!(Cli::try_parse_from(["setMaxCpuTemp", "watch", "--sensors", "missing-label"]).is_err());
    assert!(Cli::try_parse_from(["setMaxCpuTemp", "watch", "--sensors", "chip:Sensor", "--poll-ms", "0"]).is_err());
    assert!(Cli::try_parse_from([
        "setMaxCpuTemp", "watch", "--sensors", "chip:Package id 0,chip:temp2_input",
    ]).is_ok());
    assert!(Cli::try_parse_from(["setMaxCpuTemp", "release"]).is_ok());
    let fixture = Fixture::new();
    assert_eq!(read_target(fixture.runtime()).expect("default target"), Target::Off);
}
