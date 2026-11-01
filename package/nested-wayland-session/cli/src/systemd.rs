//! systemd-based CPU isolation for the hosted app,
//!  with graceful degradation.
//!
//! To keep a steady 60fps capture even when the hosted app is CPU-greedy,
//!  the app can be
//! launched inside a transient systemd user scope with a `CPUQuota` (a hard cap on total
//! CPU time) and a low `CPUWeight` (so it yields to the compositor under contention).
//!  That
//! reserves headroom for the render thread and encoder pool.
//!  When systemd is unavailable
//! (no `systemd-run`,
//!  or no running user manager) the app is launched directly instead,
//! with a warning:
//!  isolation is a robustness enhancement,
//!  never a hard requirement.

/// What:
///      `use std::process::Command;`.
///  The process-spawn builder.
/// Why:
///       This module builds the `Command` (either `systemd-run ...` or the app directly).
use std::process::Command;

/// What:
///      `use tracing::{info, warn};`.
///  Log macros.
/// Why:
///       Announce the isolation applied,
///  or warn when degrading.
use tracing::{info, warn};

/// CPU-isolation settings for the hosted app.
///
/// What:
///      `pub struct Isolation { pub enabled: bool, pub cpu_quota_percent: Option<u32>,
///           pub cpu_weight: Option<u32> }`.
///  `cpu_quota_percent` is a percent of ONE core
///           (so `800` means eight cores' worth);
///  `cpu_weight` is systemd's 1..=10000
///           relative share.
/// Why:
///       One value describing how the app should be constrained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Isolation = { enabled: boolean; cpuQuotaPercent?: number; cpuWeight?: number };
/// ```
pub struct Isolation {
    /// Whether to launch the app inside a resource-controlled systemd scope.
    pub enabled: bool,
    /// Optional hard CPU cap,
    ///  in percent of a single core (`800` = 8 cores).
    pub cpu_quota_percent: Option<u32>,
    /// Optional relative CPU share (systemd `CPUWeight`,
    ///  1..=10000).
    pub cpu_weight: Option<u32>,
}

/// Cores to leave free for the compositor when computing a default CPU quota.
///
/// What:
///      `const RESERVED_FRACTION: usize = 4;`.
///  The default quota reserves `cores /
///           RESERVED_FRACTION` cores (at least one) for the render thread and encoders.
/// Why:
///       A quarter of the machine is a safe reservation for capture at 60fps.
const RESERVED_FRACTION: usize = 4;

/// Default app `CPUWeight` when isolation is enabled without an explicit weight.
///
/// What:
///      `const DEFAULT_APP_WEIGHT: u32 = 20;`.
///  Well below the default 100,
///  so under
///           contention the app yields CPU to the compositor.
/// Why:
///       Bias the scheduler toward the capture pipeline.
const DEFAULT_APP_WEIGHT: u32 = 20;

/// Percent-per-core multiplier for building a `CPUQuota` string.
///
/// What:
///      `const PERCENT_PER_CORE: usize = 100;`.
///  One core is `100%`.
/// Why:
///       Convert a reserved-core count into a quota percentage.
const PERCENT_PER_CORE: usize = 100;

/// Compute the default app CPU quota (percent) for this machine.
///
/// What:
///      `pub fn default_quota_percent() -> u32`.
///  Reserves `cores / RESERVED_FRACTION`
///           cores for the compositor and returns the rest as a percentage.
/// Why:
///       A sensible cap that leaves capture headroom without an explicit flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function defaultQuotaPercent(): number { ... }
/// ```
pub fn default_quota_percent() -> u32 {
    // What:     `let cores = std::thread::available_parallelism().map(|n| n.get())
    //           .unwrap_or(1);`. Logical core count, or 1 if it cannot be queried.
    // Why:      The basis for the reservation.
    let cores = std::thread::available_parallelism()
        .map(|n| return n.get())
        .unwrap_or(1);

    // What:     `let reserved = (cores / RESERVED_FRACTION).max(1);`. Cores held back for
    //           the compositor, at least one.
    // Why:      Guarantee the capture pipeline some CPU no matter the machine size.
    let reserved = (cores / RESERVED_FRACTION).max(1);

    // What:     `let app_cores = cores.saturating_sub(reserved).max(1);`. Cores the app may
    //           use, at least one. `saturating_sub` avoids underflow.
    // Why:      Never cap the app to zero.
    let app_cores = cores.saturating_sub(reserved).max(1);

    // What:     `(app_cores * PERCENT_PER_CORE) as u32`. Convert to a percentage (tail
    //           expression).
    // Why:      systemd's `CPUQuota` is expressed in percent-of-one-core.
    return (app_cores * PERCENT_PER_CORE) as u32
}

/// Whether `systemd-run` and a user manager are usable here.
///
/// What:
///      `pub fn available() -> bool`.
///  Runs `systemd-run --user --version`;
///  success
///           means both the binary and a reachable user manager exist.
/// Why:
///       Decide whether to isolate or degrade.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function available(): boolean { ... }
/// ```
pub fn available() -> bool {
    // What:     `Command::new("systemd-run").arg("--user").arg("--version").stdout(
    //           std::process::Stdio::null()).stderr(std::process::Stdio::null()).status()
    //           .map(|status| status.success()).unwrap_or(false)`. Try to run it, silencing
    //           output; `.map(...).unwrap_or(false)` treats a spawn error (binary missing)
    //           as unavailable. Tail expression.
    // Why:      A cheap, definitive availability probe.
    return Command::new("systemd-run")
        .arg("--user")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| return status.success())
        .unwrap_or(false)
}

/// Build the `Command` that launches the hosted app,
///  isolated when possible.
///
/// What:
///      `pub fn build_child_command(program: &str, args: &[String], isolation:
///           &Isolation) -> Command`.
///  Returns either a `systemd-run --user --scope ...`
///           command wrapping the app,
///  or the app command directly.
///  The caller sets the
///           Wayland environment on the returned command before spawning.
/// Why:
///       One place that decides between isolated and direct launch,
///  degrading cleanly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildChildCommand(program, args, isolation): Command { ... }
/// ```
///
/// @example
/// ```ts
/// const cmd = buildChildCommand("music-player", ["songs"], { enabled: true, cpuQuotaPercent: 800 });
/// ```
pub fn build_child_command(program: &str, args: &[String], isolation: &Isolation) -> Command {
    // What:     `if !isolation.enabled { return direct(program, args); }`. No isolation
    //           requested: launch the app directly.
    // Why:      The default, un-isolated path.
    if !isolation.enabled {
        return direct(program, args);
    }

    // What:     `if !available() { warn!(...); return direct(program, args); }`. Isolation
    //           was requested but systemd is unavailable: degrade to a direct launch with a
    //           warning.
    // Why:      Isolation is best-effort; a missing systemd must not stop the fixture.
    if !available() {
        warn!("systemd-run unavailable; hosting the app WITHOUT CPU isolation (degraded)");
        return direct(program, args);
    }

    // What:     `let quota = isolation.cpu_quota_percent.unwrap_or_else(default_quota_percent);`.
    //           The hard cap, defaulting to the machine-sized reservation.
    // Why:      Always cap the app when isolating, so the compositor keeps headroom.
    let quota = isolation.cpu_quota_percent.unwrap_or_else(default_quota_percent);

    // What:     `let weight = isolation.cpu_weight.unwrap_or(DEFAULT_APP_WEIGHT);`. The soft
    //           share, defaulting low.
    // Why:      Deprioritise the app under contention.
    let weight = isolation.cpu_weight.unwrap_or(DEFAULT_APP_WEIGHT);

    // What:     `let mut cmd = Command::new("systemd-run");`. Start the wrapper command.
    // Why:      systemd-run places the app in a resource-controlled transient scope.
    let mut cmd = Command::new("systemd-run");

    // What:     `cmd.args(["--user", "--scope", "--collect", "--quiet"]);`. `--user` uses
    //           the caller's session manager; `--scope` runs the app in the caller's context
    //           (so it inherits our environment) under a scope unit; `--collect` garbage-
    //           collects the unit when it exits; `--quiet` suppresses the "Running as unit"
    //           notice.
    // Why:      Run the app in a scope we can constrain, transparently to it.
    cmd.args(["--user", "--scope", "--collect", "--quiet"]);

    // What:     `cmd.arg(format!("--property=CPUQuota={quota}%"));`. Set the hard CPU cap.
    // Why:      Bound the app's total CPU so encoders and the render thread are never starved.
    cmd.arg(format!("--property=CPUQuota={quota}%"));

    // What:     `cmd.arg(format!("--property=CPUWeight={weight}"));`. Set the soft share.
    // Why:      Under contention the scheduler favours the compositor.
    cmd.arg(format!("--property=CPUWeight={weight}"));

    // What:     `cmd.arg("--");`. End of systemd-run options.
    // Why:      Everything after is the app command line.
    cmd.arg("--");

    // What:     `cmd.arg(program); cmd.args(args);`. The app and its arguments.
    // Why:      The command systemd-run runs inside the scope.
    cmd.arg(program);
    cmd.args(args);

    // What:     `info!(...)`. Announce the applied isolation.
    // Why:      Make the constraint visible in logs.
    info!("hosting the app under a systemd scope: CPUQuota={quota}%, CPUWeight={weight}");

    // What:     `cmd`. Return the configured wrapper command (tail expression).
    // Why:      The caller adds Wayland env and spawns it.
    return cmd
}

/// Build a direct (un-isolated) command for the app.
///
/// What:
///      `fn direct(program: &str, args: &[String]) -> Command`.
///  The plain
///           `Command::new(program).args(args)` path.
/// Why:
///       Shared by the disabled and the degraded branches.
fn direct(program: &str, args: &[String]) -> Command {
    // What:     `let mut cmd = Command::new(program);`. Build the app command.
    // Why:      Launch the app itself.
    let mut cmd = Command::new(program);

    // What:     `cmd.args(args);`. Its arguments.
    // Why:      Pass the app its own args.
    cmd.args(args);

    // What:     `cmd`. Return it (tail expression).
    // Why:      Hand it back to the caller.
    return cmd
}
