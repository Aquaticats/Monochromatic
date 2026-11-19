//! Provides the command line and systemd entry points.
//!
//! The independent modules are `feedback`, `sensor`, and `target`. `policy` uses
//! `feedback`, `service` uses all four, and this file connects them to Clap.

/// Adjust a ceiling using one temperature sample.
mod feedback;
/// Read and release CPUFreq policy ceilings.
mod policy;
/// Find and read explicitly selected hwmon channels.
mod sensor;
/// Execute one pass or run the service loop.
mod service;
/// Validate and exchange the desired Celsius target.
mod target;

/// What
/// `Parser` and `Subcommand` provide Clap's command-line parser and subcommand declarations.
/// Why
/// Generate the entry points and help from the argument structure.
/// TypeScript pseudocode
/// ```ts
/// import { parseArgs } from "node:util";
/// ```
use clap::{Parser, Subcommand};
/// Filesystem paths, durations, process status and validated nonzero numbers.
use std::{
    io,
    num::{NonZeroU16, NonZeroU64},
    path::PathBuf,
    process::ExitCode,
    time::Duration,
};

/// System-wide CPUFreq directory with one entry per kernel policy.
const CPUFREQ: &str = "/sys/devices/system/cpu/cpufreq";
/// Kernel hwmon class directory.
const HWMON: &str = "/sys/class/hwmon";

/// Change the CPU temperature target of the running Linux service.
#[derive(Parser)]
#[command(name = "setMaxCpuTemp", version, subcommand_negates_reqs = true)]
struct Cli {
    /// Requested temperature in whole degrees Celsius, or off.
    #[arg(value_parser = target::parse_target, required = true, value_name = "CELSIUS|off")]
    target: Option<target::Target>,
    /// Directory in which the service exchanges target and release records.
    #[arg(long, global = true, default_value = "/run/set-max-cpu-temp")]
    runtime_dir: PathBuf,
    /// Entry points used by the service unit.
    #[command(subcommand)]
    service: Option<Service>,
}

/// What
/// This enum holds either the watch configuration or the release operation.
/// Why
/// A running watcher and its post-stop cleanup have distinct entry points.
/// TypeScript pseudocode
/// ```ts
/// type Service = { kind: "watch"; sensors: Sensor[]; pollMs: number; margin: number }
///   | { kind: "release" };
/// ```
#[derive(Subcommand)]
enum Service {
    /// Sample the selected sensors and follow the live target.
    Watch {
        /// Comma-separated driver:label or driver:tempN_input selectors.
        #[arg(long, value_delimiter = ',', value_parser = sensor::parse_sensor, required = true)]
        sensors: Vec<sensor::Sensor>,
        /// Delay between passes in milliseconds.
        #[arg(long, default_value = "500")]
        poll_ms: NonZeroU64,
        /// Degrees below the target at which to begin lowering the ceiling.
        #[arg(long, default_value = "3")]
        margin: NonZeroU16,
    },
    /// Release journaled ceilings after the watcher exits.
    Release,
}

/// Dispatch the parsed operation without changing the target on a service stop.
fn run(cli: Cli) -> io::Result<()> {
    if cli.target.is_some() && cli.service.is_some() {
        // A target update must not accidentally dispatch a systemd entry point.
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "supply a target or a service command"));
    }
    // What
    // `match` selects the entry point and binds the values carried by that case.
    // Why
    // Only the normal command publishes a target. The service owns the CPUFreq writes.
    // TypeScript pseudocode
    // switch (cli.service?.kind) { case "watch": return watch(); case "release": return release(); }
    match cli.service {
        Some(Service::Watch { sensors, poll_ms, margin }) => {
            // `PathBuf::from` creates owned paths that stay with the watcher.
            let paths = service::Paths {
                runtime: cli.runtime_dir,
                hwmon: PathBuf::from(HWMON),
                cpufreq: PathBuf::from(CPUFREQ),
            };
            // Take the positive numeric values out of Clap's validated wrappers.
            let watcher = service::Watcher::new(paths, sensors, margin.get());
            // What
            // Construct a duration from a count of milliseconds and give it to the watcher.
            // Why
            // Polling takes place only in the service, with a delay after every pass.
            // TypeScript pseudocode
            // watch(watcher, pollMs);
            service::run(watcher, Duration::from_millis(poll_ms.get()));
            // Successful exit lets systemd proceed to its post-stop action.
            return Ok(());
        }
        Some(Service::Release) => {
            // Borrow the trusted kernel root and the service's journal directory.
            return policy::release_all(std::path::Path::new(CPUFREQ), &cli.runtime_dir);
        }
        None => {
            // Clap requires the positional target when no service action was chosen.
            let Some(target) = cli.target else {
                // Report an inconsistent dispatch while keeping runtime state intact.
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "supply a Celsius target or off"));
            };
            // Publish only to the service's existing directory and contextualize any failure.
            return target::save_target(&cli.runtime_dir, target).map_err(|error| {
                // Show which path must be available before the command is usable.
                return io::Error::new(error.kind(), format!(
                    "set target in {}: {error}. Start set-max-cpu-temp.service and run the command as root",
                    cli.runtime_dir.display()
                ));
            });
        }
    }
}

/// Parse the CLI once and report I/O failure with a failing process status.
fn main() -> ExitCode {
    // What
    // `if let Err` opens an error and binds its details. `ExitCode::FAILURE`
    // returns an unsuccessful process status to the caller.
    // Why
    // An unsuccessful target update or release must be visible to the shell or systemd.
    // TypeScript pseudocode
    // try { run(parseArgs()); } catch (error) { console.error(error); process.exitCode = 1; }
    if let Err(error) = run(Cli::parse()) {
        // Keep ordinary output free and print the failed operation on stderr.
        eprintln!("setMaxCpuTemp: {error}");
        return ExitCode::FAILURE;
    }
    // Report that the selected action succeeded.
    return ExitCode::SUCCESS;
}

/// Fixture tests exercise the same watcher and request functions as the installed binary.
#[cfg(test)]
mod tests;
