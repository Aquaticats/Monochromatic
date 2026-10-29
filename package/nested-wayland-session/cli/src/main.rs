//! Parse informational/usage requests before logging or starting Wayland.

/// What:
///  ExitCode is the process status returned from main,
///  unlike ().
/// Why:
///  The compositor must retain the hosted application's exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Equivalent to a final process.exitCode value.
/// ```
use std::process::ExitCode;
/// Context retains application diagnostics without changing the public error type.
use anyhow::{Context, Result};
/// The library owns parsing and the compositor;
///  this binary owns process policy.
use nested_wayland_session::{parse_args, run};
/// Log selection is installed only after command-line parsing succeeds.
use tracing_subscriber::EnvFilter;

/// Handle clap's help/version/usage exit policy while preserving other errors.
///
/// What:
///  downcast_ref borrows a typed clap error from anyhow's error envelope.
/// Why:
///  DisplayHelp exits successfully instead of becoming main's generic failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handleCliError(error: Error): Error {
///   if (error instanceof CliError) error.printAndExit();
///   return error;
/// }
/// ```
fn handle_cli_error(error: anyhow::Error) -> anyhow::Error {
    // Some means this is clap's error; ordinary application errors pass through.
    if let Some(cli_error) = error.downcast_ref::<clap::Error>() {
        // clap prints to the appropriate stream and uses its documented exit code.
        cli_error.exit();
    }
    return error;
}

/// Run application work only after a validated non-informational invocation.
fn main() -> Result<ExitCode> {
    // What: collect creates an owned Vec<String> after dropping argv[0].
    // Why: Keep the existing parse_args interface, which excludes the binary name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const args = process.argv.slice(2);
    // ```
    let args: Vec<String> = std::env::args().skip(1).collect();
    // What: map_err handles clap's control-flow exits; ? propagates other failures.
    // Why: Help and invalid arguments must not connect to Wayland or start a child.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const config = parseArgs(args); // CLI errors print and exit before run()
    // ```
    let config = parse_args(&args)
        .map_err(handle_cli_error)
        .context("parsing command-line arguments")?;

    // What: --list-gpus prints the devices and exits before logging or any Wayland setup.
    // Why: The listing is terminal output for a person, so it must not be mixed with log lines.
    if config.list_gpus {
        println!("{}", nested_wayland_session::gpu_device::list_gpus());
        return Ok(ExitCode::SUCCESS);
    }

    // What: unwrap_or_else supplies a default when the optional log filter is absent.
    // Why: Keep normal invocation's existing logging behavior.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const filter = tryReadLogFilter() ?? new EnvFilter('info');
    // ```
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| return EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    // Run the validated child workflow and propagate compositor startup failures.
    let code = run(config)?;
    // What: Ok wraps success and ExitCode narrows to the operating system's byte status.
    // Why: Preserve the hosted client's exit result instead of always returning zero.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return code & 0xff;
    // ```
    return Ok(ExitCode::from(code as u8));
}
