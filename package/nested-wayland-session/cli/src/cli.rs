//! Display-independent clap parser for the nested Wayland session.

/// Validated configuration retains the library's existing public interface.
#[path = "cli_config.rs"]
mod config;
/// Option grammar and generated help are separate from result conversion.
#[path = "cli_command.rs"]
mod command;
/// Existing size validation remains the single dimension parser.
#[path = "cli_size.rs"]
mod size;

/// Reexport the same configuration record consumed by the compositor.
pub use config::Config;
/// Own optional socket paths beyond the parser call.
use std::path::PathBuf;
/// Preserve the public anyhow error interface while retaining clap's error type.
use anyhow::{Context, Result};
/// Existing private-portal values are unchanged.
use crate::{
    accent_color::AccentColor, appearance_portal::ColorSchemePreference, notification_mode::NotificationMode,
};
/// Validated output scale produced by the `--scale` value parser.
use crate::screen_geometry::OutputScale;
/// The device choice produced by the `--gpu` value parser.
use crate::gpu_choice::GpuChoice;

/// Parse arguments excluding the executable name without starting application work.
///
/// What:
///  The borrowed string slice stays owned by the caller.
///  Clap errors are
/// retained inside anyhow,
///  so main can honor help's success exit status.
/// Why:
///  Library tests can inspect help without terminating their own process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseArgs(args: readonly string[]): Config { return parser.parse(args); }
/// ```
pub fn parse_args(args: &[String]) -> Result<Config> {
    // What: Vec owns the argument list; &str entries borrow the caller's strings.
    // Why: clap expects argv[0], but the existing library interface omits it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const argv = ['monochromatic-nested-wayland-session', ...args];
    // ```
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push("monochromatic-nested-wayland-session");
    for arg in args {
        argv.push(arg.as_str());
    }

    // What: ? propagates a clap error without losing its typed exit semantics.
    // Why: DisplayHelp is not an application failure or a request to start Wayland.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const matches = parser.tryParse(argv);
    // ```
    let matches = command::command().try_get_matches_from(argv)?;
    let dimensions = matches.get_one::<(i32, i32)>("size").context("validated size is missing")?;
    // copied takes the small Copy value out of clap's matches; the option has a default.
    let scale = matches.get_one::<OutputScale>("scale").copied().context("validated scale is missing")?;
    // What: --list-gpus replaces the client command, so the command may be absent only then.
    // Why: A listing needs no client; every other start still requires one.
    let list_gpus = matches.get_flag("list-gpus");
    let mut child_command = Vec::new();
    if let Some(child_values) = matches.get_many::<String>("command") {
        for value in child_values {
            // clone creates owned argv values that outlive clap's matches.
            child_command.push(value.clone());
        }
    } else if !list_gpus {
        anyhow::bail!("validated client command is missing");
    }

    // What: --gpu is read as given; absence means auto, the behavior headless had before the option.
    // Why: The nested window renders on its parent's GPU, so naming a GPU there could only be ignored.
    //      Rejecting it says so instead.
    let gpu_given = matches.get_one::<GpuChoice>("gpu").cloned();
    let headless = matches.get_flag("headless");
    if gpu_given.is_some() && !headless {
        anyhow::bail!(
            "--gpu is only valid together with --headless: the nested window renders on the GPU of its parent \
             compositor, which this tool cannot choose. Add --headless to render without a display on a GPU you \
             pick (run --list-gpus to see them), or drop --gpu to use the parent's GPU"
        );
    }
    let gpu = gpu_given.unwrap_or_default();

    // What: None denotes an omitted optional setting; Some owns a present value.
    // Why: Absence must keep the previous inherited-session behavior.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let controlSocket: string | undefined;
    // ```
    let mut control_socket = None;
    if let Some(value) = matches.get_one::<String>("socket") {
        control_socket = Some(PathBuf::from(value));
    }
    let mut color_scheme = None;
    if let Some(value) = matches.get_one::<String>("color-scheme") {
        color_scheme = Some(ColorSchemePreference::parse(value)?);
    }

    let mut accent_color = None;
    if let Some(value) = matches.get_one::<String>("accent-color") {
        match AccentColor::parse(value) {
            Ok(color) => accent_color = Some(color),
            Err(message) => anyhow::bail!("--accent-color {message}"),
        }
    }
    let notifications = match matches.get_one::<String>("notifications") {
        Some(value) => NotificationMode::parse(value)?,
        None => NotificationMode::None,
    };

    // copied extracts owned integer options rather than references into matches.
    let app_cpu_quota = matches.get_one::<u32>("app-cpu-quota").copied();
    let app_cpu_weight = matches.get_one::<u32>("app-cpu-weight").copied();
    let repeat_rate = matches.get_one::<i32>("repeat-rate").copied().context("validated repeat rate is missing")?;
    let repeat_delay = matches.get_one::<i32>("repeat-delay").copied().context("validated repeat delay is missing")?;
    // What: Ok wraps the validated configuration as the successful Result.
    // Why: The caller gets no partially parsed or display-dependent state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { childCommand, controlSocket, ... };
    // ```
    return Ok(Config {
        child_command,
        control_socket,
        width: dimensions.0,
        height: dimensions.1,
        scale,
        color_scheme,
        accent_color,
        notifications,
        isolate: matches.get_flag("isolate"),
        launcher_log: matches.get_flag("launcher-log"),
        app_cpu_quota,
        app_cpu_weight,
        accessibility: matches.get_flag("accessibility"),
        repeat_rate,
        repeat_delay,
        headless,
        gpu,
        list_gpus,
    });
}

/// Parser behavior is tested without an available compositor.
#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
