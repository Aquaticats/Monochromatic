//! A private session bus with the recording notification service, and no compositor.
//!
//! Runs one command with `DBUS_SESSION_BUS_ADDRESS` pointing at a disposable `dbus-daemon` that has no service
//!  directories, so a real `notify-send` (or any client of `org.freedesktop.Notifications`) lands on the same
//!  recording service the nested session serves,
//!  never on the owner's desktop.
//! When the command ends,
//!  the notification log is written to `--log PATH`,
//!  one record per notification,
//!  in the format of the `notification-log` control verb.
//! Nothing here opens a window,
//!  needs a Wayland display,
//!  or starts a compositor.

/// Imports the owned path for the log file and the child process types.
use std::{
    path::PathBuf,
    process::{Command, ExitCode},
    sync::Arc,
    time::Instant,
};

/// Imports the error channel.
use anyhow::{Context, Result};

/// Imports the library pieces the nested session already uses.
use nested_wayland_session::{
    control_notification,
    notification_mode::NotificationMode,
    plasma_services::PlasmaServices,
    private_bus::PrivateBus,
    protocol::{NotificationCommand, Response},
};

/// The grammar,
///  printed with every usage error.
const USAGE: &str = "usage: monochromatic-nested-wayland-session-notification-recorder --log PATH \
                     [--notifications freedesktop|kde] -- COMMAND [ARG...]";

/// One validated invocation.
struct Invocation {
    /// Where the notification log is written when the command ends.
    log: PathBuf,
    /// Which services the bus serves.
    mode: NotificationMode,
    /// The command and its arguments.
    command: Vec<String>,
}

/// Reads the arguments after the program name.
///
/// What:
///      `fn parse(args: &[String]) -> Result<Invocation>`.
///  `--log` is required,
///  `--notifications` defaults to `freedesktop`,
///  and everything after `--` is the command.
fn parse(args: &[String]) -> Result<Invocation> {
    let mut log = None;
    let mut mode = NotificationMode::Freedesktop;
    let mut position = 0;
    while position < args.len() {
        match args[position].as_str() {
            "--log" => {
                position += 1;
                log = Some(PathBuf::from(args.get(position).context(USAGE)?));
            }
            "--notifications" => {
                position += 1;
                mode = NotificationMode::parse(args.get(position).context(USAGE)?)?;
                anyhow::ensure!(mode.serves_notifications(), "--notifications none serves nothing to record; {USAGE}");
            }
            "--" => {
                let command = args[position + 1..].to_vec();
                anyhow::ensure!(!command.is_empty(), "the command is missing; {USAGE}");
                return Ok(Invocation { log: log.context(USAGE)?, mode, command });
            }
            other => anyhow::bail!("unknown argument {other}; {USAGE}"),
        }
        position += 1;
    }
    anyhow::bail!("the `--` before the command is missing; {USAGE}");
}

/// Serves the bus and the recording service,
///  runs the command,
///  and writes the log.
///
/// What:
///      `fn record(invocation: Invocation) -> Result<i32>`.
///  The hosted command sees no runtime directory,
///  so a client that ignores the bus address cannot find the host's bus by its default path.
fn record(invocation: Invocation) -> Result<i32> {
    let bus = Arc::new(PrivateBus::start().context("starting the private session bus")?);
    let services = PlasmaServices::start(invocation.mode, Arc::clone(&bus), Instant::now())
        .context("starting the recording notification service")?;
    let (program, arguments) = invocation.command.split_first().context("the command is missing")?;
    let status = Command::new(program)
        .args(arguments)
        .env("DBUS_SESSION_BUS_ADDRESS", bus.address())
        .env_remove("XDG_RUNTIME_DIR")
        .status()
        .with_context(|| return format!("running {program}"))?;
    let text = match control_notification::run(services.as_ref(), &NotificationCommand::Log { count: usize::MAX }) {
        Response::OkWith(records) => records,
        Response::Ok => String::new(),
        Response::Err(message) => anyhow::bail!("reading the notification log: {message}"),
    };
    std::fs::write(&invocation.log, text).with_context(|| return format!("writing {}", invocation.log.display()))?;
    return Ok(status.code().unwrap_or(1));
}

/// Parses the arguments,
///  records,
///  and returns the command's own exit status.
fn main() -> Result<ExitCode> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = record(parse(&args)?)?;
    return Ok(ExitCode::from(u8::try_from(code & 0xff).unwrap_or(1)));
}
