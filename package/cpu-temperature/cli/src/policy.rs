//! Applies ceilings to Linux CPUFreq policies and records which ceilings must be released.

/// What
/// Imports the filesystem types and an ordered map from the standard library.
/// Why
/// Keep the last successful request for each policy and retry writes that failed.
/// TypeScript pseudocode
/// ```ts
/// import * as fs from "node:fs";
/// type Requests = Map<string, number>;
/// ```
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Import the percentage that means the full range is available.
use crate::feedback::FULL_RANGE;

/// Runtime subdirectory of policies that may still carry one of our requests.
const LIMITED: &str = "limited";

/// CPUFreq attribute for the live userspace ceiling.
const CEILING: &str = "scaling_max_freq";

/// CPUFreq attribute for the hardware maximum, in kHz.
const HARDWARE_MAX: &str = "cpuinfo_max_freq";

/// Recognize kernel CPUFreq policy names before using them to join trusted directories.
fn is_policy(name: &str) -> bool {
    // What
    // `let Some` takes the suffix out of an optional value.
    // Why
    // The kernel's root directory includes files such as `boost` alongside the policies.
    // TypeScript pseudocode
    // const number = stripPrefix(name, "policy"); if (number === undefined) return false;
    let Some(number) = name.strip_prefix("policy") else {
        return false;
    };
    // Require decimal digits so a journal filename can only name a policy.
    return !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit());
}

/// Read a nonnegative frequency in kHz with the source path in any diagnostic.
fn read_frequency(path: &Path) -> io::Result<u64> {
    // What
    // `map_err` modifies the failure case and `?` returns that error early.
    // Why
    // Operators need the policy and attribute that failed.
    // TypeScript pseudocode
    // const text = readTextOrThrowWithPath(path);
    let text = fs::read_to_string(path).map_err(|error| {
        // Attach context while preserving the original operating-system error kind.
        return io::Error::new(error.kind(), format!("read {}: {error}", path.display()));
    })?;
    // What
    // `u64` holds an unsigned 64-bit integer. `u32` is narrower, `i64` is signed,
    // and `usize` follows the machine's pointer width.
    // Why
    // Frequencies and intermediate percentage calculations have a fixed nonnegative range.
    // TypeScript pseudocode
    // return parseNonnegativeInteger(text.trim());
    return text.trim().parse::<u64>().map_err(|error| {
        // Report a malformed kernel value with its source.
        return io::Error::new(io::ErrorKind::InvalidData, format!("parse {}: {error}", path.display()));
    });
}

/// Write an existing ceiling attribute, preserving write failures for the caller.
fn write_ceiling(policy: &Path, ceiling: u64) -> io::Result<()> {
    // Borrow the computed path when opening it so it can also appear in the diagnostic.
    let path = policy.join(CEILING);
    // Open the existing attribute. A disappeared policy cannot be recreated from here.
    let mut file = OpenOptions::new().write(true).truncate(true).open(&path).map_err(|error| {
        // Keep the operating-system error and identify the policy.
        return io::Error::new(error.kind(), format!("open {}: {error}", path.display()));
    })?;
    // What
    // `write!` writes formatted text to the existing file and returns its I/O result.
    // Why
    // The kernel expects a single numeric CPUFreq request.
    // TypeScript pseudocode
    // writeToOpenFile(file, String(ceiling));
    return write!(file, "{ceiling}").map_err(|error| {
        // Report which ceiling the kernel rejected.
        return io::Error::new(error.kind(), format!("write {ceiling} to {}: {error}", path.display()));
    });
}

/// Apply one policy's share of the available range, journaling before the sysfs write.
fn limit_policy(
    policy: &Path,
    marker: &Path,
    percent: u32,
    requested: &mut BTreeMap<PathBuf, u64>,
) -> io::Result<()> {
    // Use the live floor so this controller does not need to own minimum frequency.
    let floor = read_frequency(&policy.join("scaling_min_freq"))?;
    // Keep each policy's own hardware maximum on mixed-core systems.
    let maximum = read_frequency(&policy.join(HARDWARE_MAX))?;
    if floor > maximum {
        // Refuse inconsistent bounds in this policy and let others proceed.
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
            "{} has a frequency floor above its hardware maximum", policy.display()
        )));
    }
    // Widen the percentage to the frequency type before interpolation.
    let span = maximum - floor;
    // Split the integer product so even a malformed large bound cannot overflow.
    let scale = u64::from(percent);
    let divisor = u64::from(FULL_RANGE);
    let ceiling = floor + (span / divisor) * scale + (span % divisor) * scale / divisor;
    // What
    // `Some(&ceiling)` means the saved request exists and equals this ceiling.
    // Why
    // Unchanged requests need no write. Check the effective maximum for a recreated policy.
    // TypeScript pseudocode
    // if (requested.get(policy) === ceiling && readFrequency(join(policy, CEILING)) <= ceiling) return;
    if requested.get(policy) == Some(&ceiling) && read_frequency(&policy.join(CEILING))? <= ceiling {
        // The prior request still keeps this policy within the desired range.
        return Ok(());
    }
    // A marker created before the write survives termination between these operations.
    OpenOptions::new().write(true).create(true).truncate(false).open(marker)?;
    // Keep the marker even if sysfs accepts only part of the multi-policy operation.
    write_ceiling(policy, ceiling)?;
    // What
    // `to_path_buf` makes an owned copy for the map, and `insert` replaces the old value.
    // Why
    // A failed write must not count as a successful request on the next sample.
    // TypeScript pseudocode
    // requested.set(policy, ceiling);
    requested.insert(policy.to_path_buf(), ceiling);
    // The policy accepted its ceiling.
    return Ok(());
}

/// Finish a policy scan with all of its diagnostics.
fn finish(errors: &[String]) -> io::Result<()> {
    if errors.is_empty() {
        // Every policy in the scan accepted its operation.
        return Ok(());
    }
    // Combine errors so one failing policy cannot hide another.
    return Err(io::Error::other(errors.join("\n")));
}

/// Limit every policy currently visible to Linux, continuing after individual failures.
pub fn limit_all(
    root: &Path,
    runtime: &Path,
    percent: u32,
    requested: &mut BTreeMap<PathBuf, u64>,
) -> io::Result<()> {
    // Keep recovery data separate from the desired target token.
    let journal = runtime.join(LIMITED);
    // The root-only directory already exists on subsequent samples.
    fs::create_dir_all(&journal)?;
    // Accumulate policy-specific errors while continuing to apply other ceilings.
    let mut errors = Vec::new();
    let mut found = false;
    // Rescan so newly online policies join the running controller.
    for entry_result in fs::read_dir(root)? {
        // Propagate a failed directory entry read.
        let entry = entry_result?;
        // Preserve the owned filename while interpreting its text.
        let name = entry.file_name();
        // Linux policy names are ASCII.
        let Some(name_text) = name.to_str() else {
            continue;
        };
        if !is_policy(name_text) {
            continue;
        }
        found = true;
        // Report the failure and move on to the next policy.
        if let Err(error) = limit_policy(&entry.path(), &journal.join(&name), percent, requested) {
            // Include the policy even if the failure occurred in the marker directory.
            errors.push(format!("limit {name_text}: {error}"));
        }
    }
    if !found {
        // A successful command must have at least one policy it can control.
        return Err(io::Error::new(io::ErrorKind::NotFound, format!(
            "no CPUFreq policies found in {}", root.display()
        )));
    }
    // Propagate all failed policy operations to the daemon's diagnostics.
    return finish(&errors);
}

/// Release the userspace request for one policy and retire its marker on success.
fn release_policy(policy: &Path, marker: &Path) -> io::Result<()> {
    // What
    // `try_exists` distinguishes absence from a filesystem error.
    // Why
    // A removed kernel policy has lost its request. Other failures need a retry.
    // TypeScript pseudocode
    // if (existsOrThrow(policy)) restoreCeiling(policy);
    if policy.try_exists()? {
        // The hardware maximum releases this writer's contribution to the effective cap.
        let maximum = read_frequency(&policy.join(HARDWARE_MAX))?;
        // Keep the marker until the release write succeeds.
        write_ceiling(policy, maximum)?;
    }
    // Only retire a marker after release or policy removal.
    return fs::remove_file(marker);
}

/// Release only journaled policies, leaving failed releases pending for a retry.
pub fn release_all(root: &Path, runtime: &Path) -> io::Result<()> {
    // An untouched runtime has no policy markers to release.
    let entries = match fs::read_dir(runtime.join(LIMITED)) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    // Keep processing markers after the first failure.
    let mut errors = Vec::new();
    for entry_result in entries {
        // Read policy names from the root-owned journal.
        let entry = entry_result?;
        // Keep the filename available to build the sysfs path.
        let name = entry.file_name();
        // Reject malformed marker names without using them as paths.
        let Some(name_text) = name.to_str() else {
            errors.push("invalid filename in the CPUFreq release journal".to_owned());
            continue;
        };
        if !is_policy(name_text) {
            // Leave the unexpected entry visible for inspection.
            errors.push(format!("invalid CPUFreq policy marker: {name_text}"));
            continue;
        }
        // Retain failed markers, which the next pass or the stop action can retry.
        if let Err(error) = release_policy(&root.join(&name), &entry.path()) {
            // Identify the policy whose request is still present.
            errors.push(format!("release {name_text}: {error}"));
        }
    }
    // Report whether every journaled restriction was released.
    return finish(&errors);
}
