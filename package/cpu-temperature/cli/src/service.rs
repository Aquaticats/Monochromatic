//! Reconciles the live target with temperatures, CPUFreq and release records.

/// What
/// Imports the path and synchronization types used by one service process.
/// Why
/// Sample the sensors and apply ceilings in order, with a pause between passes.
/// TypeScript pseudocode
/// ```ts
/// type Applied = Map<string, number>;
/// ```
use std::{collections::BTreeMap, io, path::PathBuf, thread, time::Duration};

/// The pure feedback calculation and its unrestricted endpoint.
use crate::feedback::{FULL_RANGE, adjust_ceiling};
/// Linux policy updates and release.
use crate::policy::{limit_all, release_all};
/// Sensor resolution and sample aggregation.
use crate::sensor::{Sensor, find_inputs, read_highest};
/// Runtime target and Celsius-unit conversion.
use crate::target::{MILLIDEGREES, Target, read_target};

/// Directories used by the service, which can also refer to a disposable test fixture.
pub struct Paths {
    /// Root-only directory for target and release records.
    pub runtime: PathBuf,
    /// Linux hwmon device directory.
    pub hwmon: PathBuf,
    /// Linux CPUFreq policy directory.
    pub cpufreq: PathBuf,
}

/// What
/// A watcher owns the values it carries between one temperature sample and the next.
/// Why
/// A stale sensor or a failed policy write can be retried without restarting the process.
/// TypeScript pseudocode
/// ```ts
/// type Watcher = { paths: Paths; ceiling: number; margin: number;
///   selectors: Sensor[]; inputs: string[]; requested: Map<string, number> };
/// ```
pub struct Watcher {
    /// Configured Linux and runtime directories.
    paths: Paths,
    /// Desired percentage of the available frequency range.
    ceiling: u32,
    /// Approach margin in millidegrees Celsius.
    margin: i32,
    /// User-selected driver and channel pairs.
    selectors: Vec<Sensor>,
    /// Cached real sysfs paths, cleared when a sensor fails.
    inputs: Vec<PathBuf>,
    /// Last successful request per policy.
    requested: BTreeMap<PathBuf, u64>,
}

impl Watcher {
    /// Set up one controller. The first sample begins with the full adjustable range.
    pub fn new(paths: Paths, selectors: Vec<Sensor>, margin_degrees: u16) -> Self {
        // What
        // `Self` names the struct of this method, `Vec::new` starts a list and
        // `BTreeMap::new` starts a map. `i32::from` widens an unsigned short losslessly.
        // Why
        // Build the state without reading sensors until the target becomes active.
        // TypeScript pseudocode
        // return { paths, selectors, inputs: [], requested: new Map(),
        //   ceiling: FULL_RANGE, margin: marginDegrees * MILLIDEGREES };
        return Self {
            paths,
            ceiling: FULL_RANGE,
            margin: i32::from(margin_degrees) * MILLIDEGREES,
            selectors,
            inputs: Vec::new(),
            requested: BTreeMap::new(),
        };
    }

    /// Resolve the inputs when needed, and discard the cache after a read error.
    fn temperature(&mut self) -> io::Result<i32> {
        if self.inputs.is_empty() {
            // What
            // The borrowed paths and selectors stay available for the following sample.
            // `?` passes an error back to the control pass.
            // Why
            // Hwmon device indexes can change while the service is running.
            // TypeScript pseudocode
            // this.inputs = findInputs(this.paths.hwmon, this.selectors);
            self.inputs = find_inputs(&self.paths.hwmon, &self.selectors)?;
        }
        // Borrow the discovered sensor paths for a single sample.
        let sample = read_highest(&self.inputs);
        if sample.is_err() {
            // Rediscover the hwmon paths next time the controller samples.
            self.inputs.clear();
        }
        return sample;
    }

    /// Release the journaled caps and clear cached policy requests.
    fn release(&mut self) -> io::Result<()> {
        self.requested.clear();
        // Read the same durable journal that the systemd stop action uses.
        return release_all(&self.paths.cpufreq, &self.paths.runtime);
    }

    /// Apply the current adjustable frequency percentage to each online policy.
    fn apply(&mut self) -> io::Result<()> {
        // Lend the current values to the policy layer, which records successful writes.
        return limit_all(&self.paths.cpufreq, &self.paths.runtime, self.ceiling, &mut self.requested);
    }

    /// One service pass, using the latest requested temperature or explicit off.
    pub fn step(&mut self) -> io::Result<()> {
        // What
        // `match` opens the active target or selects the off branch. An I/O failure exits this pass.
        // Why
        // Read the target even if a sensor has failed, so off can always request release.
        // TypeScript pseudocode
        // const target = readTarget(this.paths.runtime);
        // if (target === "off") { this.ceiling = FULL_RANGE; return this.release(); }
        let degrees = match read_target(&self.paths.runtime)? {
            Target::Celsius(value) => value,
            Target::Off => {
                self.ceiling = FULL_RANGE;
                return self.release();
            }
        };
        // What
        // Branch on the sample result to control behavior on temperature read failure.
        // Why
        // With an active target and no usable temperature, request every available floor.
        // TypeScript pseudocode
        // try { sample = temperature(); } catch (error) { applyFloors(); throw error; }
        let sample = match self.temperature() {
            Ok(value) => value,
            Err(error) => {
                self.ceiling = 0;
                // Attempt every policy even if the selected sensor has disappeared.
                self.apply()?;
                // Let the service log the missing sensor and retry at the next sample.
                return Err(error);
            }
        };
        // Convert Celsius to the signed units of the sample before comparing.
        let target = i32::from(degrees) * MILLIDEGREES;
        // Move the adjustable ceiling by the amount this sample calls for.
        self.ceiling = adjust_ceiling(self.ceiling, sample, target, self.margin);
        if self.ceiling == FULL_RANGE {
            return self.release();
        }
        // Preserve all policy-specific write failures for the service log.
        self.apply()?;
        if self.ceiling == 0 && sample > target {
            // The setting remains active and will continue sampling at every policy's floor.
            return Err(io::Error::other(format!(
                "CPU temperature exceeds the {degrees} °C target at the CPUFreq floors"
            )));
        }
        // This pass successfully updated the controller.
        return Ok(());
    }
}

/// Run under systemd, which calls release after this process exits.
pub fn run(mut watcher: Watcher, interval: Duration) {
    // What
    // `None` starts an optional last diagnostic with no saved value.
    // Why
    // Polling continues on errors without logging the same failure on every sample.
    // TypeScript pseudocode
    // let previousError: string | undefined;
    let mut previous_error = None;
    // The only long-running loop. The configured interval puts a pause after each pass.
    loop {
        // What
        // Match either a successful step or its owned I/O error.
        // Why
        // Keep the running service available for retries and target changes.
        // TypeScript pseudocode
        // try { watcher.step(); onSuccess(); } catch (error) { onError(error); }
        match watcher.step() {
            Ok(()) => {
                if previous_error.is_some() {
                    // Report when a failing sensor or policy becomes usable again.
                    eprintln!("CPU temperature control recovered");
                    previous_error = None;
                }
            }
            Err(error) => {
                // Retain the text so it can be compared with the next pass.
                let message = error.to_string();
                // Compare by borrowed text and keep ownership for a changed message.
                if previous_error.as_deref() != Some(message.as_str()) {
                    // Send service diagnostics to the systemd journal via stderr.
                    eprintln!("CPU temperature control: {message}");
                    previous_error = Some(message);
                }
            }
        }
        // Block only this service thread between passes.
        thread::sleep(interval);
    }
}
