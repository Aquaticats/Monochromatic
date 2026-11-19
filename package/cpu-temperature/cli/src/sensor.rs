//! Resolves configured Linux hwmon channels and reports their highest sample.

/// What
/// Imports I/O, paths and the filesystem. `Path` borrows a path and `PathBuf` owns one.
/// Why
/// Borrow configured directories when scanning and own the resolved input paths across samples.
/// TypeScript pseudocode
/// ```ts
/// import * as fs from "node:fs";
/// type Path = string;
/// ```
use std::{fs, io, path::{Path, PathBuf}};

/// What
/// A struct groups two owned strings. `String` owns bytes that would only be borrowed by `&str`.
/// Why
/// Keep the chip and channel names after the argument parser drops its input.
/// TypeScript pseudocode
/// ```ts
/// type Sensor = { driver: string; channel: string };
/// ```
#[derive(Clone)]
pub struct Sensor {
    /// Chip names such as the value of a hwmon device's `name` file.
    driver: String,
    /// A temperature label or a channel's `tempN_input` filename.
    channel: String,
}

/// Parse the chip and channel names without interpreting either as a path.
pub fn parse_sensor(value: &str) -> Result<Sensor, String> {
    // What
    // `let Some((driver, channel))` unwraps a pair from an optional split.
    // Why
    // A selector identifies the driver even when a label exists on several devices.
    // TypeScript pseudocode
    // const pair = splitOnce(value, ":"); if (pair === undefined) throw new Error("missing :");
    let Some((driver, channel)) = value.split_once(':') else {
        // What
        // `Err` carries failure, and `to_owned` copies the diagnostic into an owned string.
        // Why
        // Clap can show how to correct the malformed selector.
        // TypeScript pseudocode
        // throw new Error("expected DRIVER:LABEL or DRIVER:tempN_input");
        return Err("expected DRIVER:LABEL or DRIVER:tempN_input".to_owned());
    };
    if driver.is_empty() || channel.is_empty() {
        // Report empty halves as invalid before opening sysfs.
        return Err("both the sensor driver and channel are required".to_owned());
    }
    // What
    // `Ok` wraps a struct literal in a successful result.
    // Why
    // Both pieces of the selector can outlive the parser input.
    // TypeScript pseudocode
    // return { driver, channel };
    return Ok(Sensor { driver: driver.to_owned(), channel: channel.to_owned() });
}

/// Recognize the Linux temperature-channel filename.
fn temperature_channel(input: &str) -> Option<&str> {
    // What
    // `?` on an option returns the absent case when a suffix is missing.
    // Why
    // Attributes for labels, other sensors, and thresholds cannot be sampled as temperatures.
    // TypeScript pseudocode
    // const channel = stripSuffix(input, "_input"); if (channel === undefined) return undefined;
    let channel = input.strip_suffix("_input")?;
    // Require the temperature prefix before checking the channel number.
    let number = channel.strip_prefix("temp")?;
    // What
    // `is_ok` tests successful numeric parsing. An unsigned value accepts sysfs channel numbers.
    // Why
    // Match the numbered attribute family without relying on a particular channel number.
    // TypeScript pseudocode
    // if (isUnsignedInteger(number)) return channel;
    if number.parse::<u32>().is_ok() {
        // Keep the channel prefix for the corresponding label lookup.
        return Some(channel);
    }
    // The input filename belongs to another attribute family.
    return None;
}

/// Match a configured sensor against one chip and append the resolved input paths.
fn find_on_chip(chip: &Path, sensor: &Sensor, inputs: &mut Vec<PathBuf>) -> io::Result<()> {
    // What
    // Read the chip name from a file. `?` propagates an I/O failure to the sampler.
    // Why
    // The numbered hwmon directory can change across reboots and hotplug events.
    // TypeScript pseudocode
    // const chipName = readFileSync(join(chip, "name"), "utf8");
    let chip_name = match fs::read_to_string(chip.join("name")) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if chip_name.trim() != sensor.driver {
        // This chip cannot contain the configured driver.
        return Ok(());
    }
    // Walk only this directory and propagate an enumeration failure.
    for entry_result in fs::read_dir(chip)? {
        // Retain the input's full path independently of its filename.
        let entry = entry_result?;
        // Keep the temporary filename alive while borrowing its text.
        let name = entry.file_name();
        // Ignore a filename that cannot belong to Linux's ASCII sensor ABI.
        let Some(input) = name.to_str() else {
            continue;
        };
        // Extract a numbered temperature channel before considering its label.
        let Some(channel) = temperature_channel(input) else {
            continue;
        };
        if input == sensor.channel {
            // Resolve the chip symlink so reusing a hwmon number cannot silently switch chips.
            inputs.push(fs::canonicalize(entry.path())?);
            continue;
        }
        // What
        // `format!` fills a template and produces owned text. The path belongs to this chip.
        // Why
        // Labels are optional and correspond to the same channel number.
        // TypeScript pseudocode
        // const labelPath = join(chip, `${channel}_label`);
        let label_path = chip.join(format!("{channel}_label"));
        // Skip channels that only provide an input filename.
        let label = match fs::read_to_string(label_path) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if label.trim() == sensor.channel {
            // Preserve the matched chip identity across samples.
            inputs.push(fs::canonicalize(entry.path())?);
        }
    }
    // All temperature inputs on this chip have been considered.
    return Ok(());
}

/// Resolve every configured selector. Multiple matches contribute to the highest reading.
pub fn find_inputs(root: &Path, selectors: &[Sensor]) -> io::Result<Vec<PathBuf>> {
    // What
    // `Vec::new` creates a growable owned list. A slice borrows the selectors from the service.
    // Why
    // The number of hwmon devices comes from Linux and is unknown at compile time.
    // TypeScript pseudocode
    // const inputs: Path[] = [];
    let mut inputs = Vec::new();
    for selector in selectors {
        // Comparing lengths detects a missing selector even when other selectors match.
        let preceding_count = inputs.len();
        // Enumerate the current names without descending into any chip.
        for chip_result in fs::read_dir(root)? {
            // Keep a usable path without borrowing from the directory iterator.
            let chip = chip_result?;
            // Let this selector add matches to the one combined input list.
            find_on_chip(&chip.path(), selector, &mut inputs)?;
        }
        if inputs.len() == preceding_count {
            // Name the exact sensor the operator needs to configure or reconnect.
            return Err(io::Error::new(io::ErrorKind::NotFound, format!(
                "CPU sensor {}:{} was not found in {}", selector.driver, selector.channel, root.display()
            )));
        }
    }
    // Transfer the resolved paths to the service's read cache.
    return Ok(inputs);
}

/// Sample the hottest of the selected inputs in signed millidegrees Celsius.
pub fn read_highest(inputs: &[PathBuf]) -> io::Result<i32> {
    if inputs.is_empty() {
        // Reading an empty set cannot establish a valid control input.
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "configure at least one CPU sensor"));
    }
    // Start with the smallest signed value so below-freezing readings also compare correctly.
    let mut highest = i32::MIN;
    for input in inputs {
        // Add the affected path so a failed read can be diagnosed.
        let text = fs::read_to_string(input).map_err(|error| {
            // Preserve the system's error kind and show the failed channel.
            return io::Error::new(error.kind(), format!("read {}: {error}", input.display()));
        })?;
        // Use a signed parser since hwmon temperatures are signed.
        let sample = text.trim().parse::<i32>().map_err(|error| {
            // Name the channel that cannot be interpreted in millidegrees.
            return io::Error::new(io::ErrorKind::InvalidData, format!("parse {}: {error}", input.display()));
        })?;
        highest = highest.max(sample);
    }
    // Return the sample shared by all frequency policies.
    return Ok(highest);
}
