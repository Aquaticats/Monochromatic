//! What:
//!  Turn a parsed command line into a request to the right server,
//! or run the server itself.
//! Why:
//!  Every command except the hidden `serve` is a client of the per-worktree (or per-user) server,
//! which starts on
//!  demand and exits when idle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function dispatch(cli: Cli, sink: EventSink): Promise<number>
//! ```

/// Imports `Cli`,
/// `Command`,
/// `UserCommand` from `crate::args`.
use crate::args::{Cli, Command, UserCommand};
/// Imports `resolve_repository` from `crate::discover`.
use crate::discover::resolve_repository;
/// Imports `ClientOptions`,
/// `connect` from `monochromatic_file_enforcer::client`.
use monochromatic_file_enforcer::client::{ClientOptions, connect};
/// Imports `ConfigScope` from `monochromatic_file_enforcer::config_decode`.
use monochromatic_file_enforcer::config_decode::ConfigScope;
/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `Endpoint`,
/// `default_user_config_path`,
/// `repository_endpoint`,
/// `user_endpoint` from `monochromatic_file_enforcer::endpoint`.
use monochromatic_file_enforcer::endpoint::{Endpoint, default_user_config_path, repository_endpoint, user_endpoint};
/// Imports `EventSink` from `monochromatic_file_enforcer::event`.
use monochromatic_file_enforcer::event::EventSink;
/// Imports `object` from `monochromatic_file_enforcer::json_build`.
use monochromatic_file_enforcer::json_build::object;
/// Imports `ServerLaunch` from `monochromatic_file_enforcer::launch`.
use monochromatic_file_enforcer::launch::ServerLaunch;
/// Imports `RuleRegistry` from `monochromatic_file_enforcer::registry`.
use monochromatic_file_enforcer::registry::RuleRegistry;
/// Imports `EXIT_DRIFT`,
/// `EXIT_USAGE` from `monochromatic_file_enforcer::run`.
use monochromatic_file_enforcer::run::{EXIT_DRIFT, EXIT_USAGE};
/// Imports `ServerOptions`,
/// `serve` from `monochromatic_file_enforcer::server`.
use monochromatic_file_enforcer::server::{ServerOptions, serve};
/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Duration` from `std::time`.
use std::time::Duration;

/// How long a client keeps trying to reach or start a server.
const CONNECT_DEADLINE: Duration = Duration::from_secs(15);

/// The environment variable that overrides the idle seconds of servers this client starts (CI shutdown control).
pub const IDLE_SECONDS_VARIABLE: &str = "FILE_ENFORCER_IDLE_SECONDS";

/// What:
///  The idle seconds to give a server this client starts.
/// Why:
///  Containers and CI want a short,
/// deterministic shutdown;
/// people want a server that stays warm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function idleSeconds(): number
/// ```
fn idle_seconds() -> u64 {
    return std::env::var(IDLE_SECONDS_VARIABLE).ok().and_then(|text| return text.parse::<u64>().ok()).unwrap_or(120);
}

/// What:
///  Emit one diagnostic as an error event and return the usage exit code.
/// Why:
///  Resolution failures before any server is involved are bad command lines or configs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function usageFailure(sink: EventSink, error: Diagnostic): number
/// ```
fn usage_failure(sink: &mut dyn EventSink, error: &Diagnostic) -> i32 {
    sink.emit(error.to_event("error"));
    return EXIT_USAGE;
}

/// What:
///  Build the client options for an endpoint:
/// how to reach the server and how to start one.
/// Why:
///  The server is started by running this same executable with the hidden `serve` command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clientOptions(endpoint: Endpoint, user: boolean): ClientOptions
/// ```
fn client_options(endpoint: &Endpoint, user: bool) -> Result<ClientOptions, Diagnostic> {
    let program = match std::env::current_exe() {
        Ok(program) => program,
        Err(error) => {
            let message = format!("cannot find this executable to start the server: {error}");
            return Err(Diagnostic::new("server-start-failed", &message));
        }
    };
    let mut args: Vec<OsString> = vec![OsString::from("serve"), OsString::from("--root"), endpoint.root.clone().into_os_string()];
    args.push(OsString::from("--socket"));
    args.push(endpoint.socket.clone().into_os_string());
    args.push(OsString::from("--idle-seconds"));
    args.push(OsString::from(idle_seconds().to_string()));
    if user {
        args.push(OsString::from("--user"));
    }
    let launch = ServerLaunch { program, args };
    return Ok(ClientOptions { root: endpoint.root.clone(), socket: endpoint.socket.clone(), launch, deadline: CONNECT_DEADLINE });
}

/// What:
///  Send one request to the endpoint's server and return the exit code.
/// Why:
///  The shared body of `write`,
/// `check`,
/// `explain`,
/// `watch` and the `user` commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function request(endpoint: Endpoint, user: boolean, method: string, params: unknown, sink: EventSink): Promise<number>
/// ```
fn request(endpoint: &Endpoint, user: bool, method: &str, params: Value, sink: &mut dyn EventSink) -> i32 {
    let options = match client_options(endpoint, user) {
        Ok(options) => options,
        Err(error) => return usage_failure(sink, &error),
    };
    let mut connection = match connect(&options) {
        Ok(connection) => connection,
        Err(error) => {
            sink.emit(error.to_event("error"));
            return EXIT_DRIFT;
        }
    };
    return connection.request(method, params, sink);
}

/// What:
///  The `run` request parameters for a mode name and config path.
/// Why:
///  The server rebuilds a `RunRequest` from these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runParams(mode: string, config: string, extra: Record<string, unknown>): unknown
/// ```
fn run_params(mode: &str, config: &Path, extra: &[(&str, Value)]) -> Value {
    let mut pairs: Vec<(&str, Value)> = vec![("mode", Value::from(mode)), ("configPath", Value::from(config.display().to_string()))];
    for pair in extra {
        pairs.push((pair.0, pair.1.clone()));
    }
    return object(&pairs);
}

/// What:
///  Make a possibly relative path absolute against the current directory.
/// Why:
///  The server's current directory is not the client's,
/// so every path crosses the socket absolute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function absolutePath(path: string): string
/// ```
fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    return std::env::current_dir().map(|current| return current.join(path)).unwrap_or_else(|_| return path.to_path_buf());
}

/// What:
///  Run the chosen command and return its exit code.
/// Why:
///  Config discovery failures are usage errors (exit 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dispatch(cli: Cli, sink: EventSink): Promise<number>
/// ```
pub fn dispatch(cli: &Cli, sink: &mut dyn EventSink) -> i32 {
    match &cli.command {
        Command::Serve { root, socket, idle_seconds, user } => return run_server(root, socket, *idle_seconds, *user, sink),
        Command::User(subcommand) => return dispatch_user(cli, subcommand, sink),
        _ => {}
    }
    let (root, config_path) = match resolve_repository(&cli.config) {
        Ok(pair) => pair,
        Err(error) => return usage_failure(sink, &error),
    };
    let endpoint = match repository_endpoint(&root) {
        Ok(endpoint) => endpoint,
        Err(error) => return usage_failure(sink, &error),
    };
    match &cli.command {
        Command::Write => return request(&endpoint, false, "run", run_params("write", &config_path, &[]), sink),
        Command::Check { expected_dir } => {
            let mut extra: Vec<(&str, Value)> = Vec::new();
            if let Some(dir) = expected_dir {
                extra.push(("expectedDir", Value::from(absolute_path(dir).display().to_string())));
            }
            return request(&endpoint, false, "run", run_params("check", &config_path, &extra), sink);
        }
        Command::Explain { path } => {
            let extra = [("path", Value::from(path.as_str()))];
            return request(&endpoint, false, "run", run_params("explain", &config_path, &extra), sink);
        }
        Command::Watch => return request(&endpoint, false, "watch", run_params("write", &config_path, &[]), sink),
        _ => return EXIT_USAGE,
    }
}

/// What:
///  Run a `user` subcommand against the per-user server.
/// Why:
///  The per-user config lives outside any worktree and has its own server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dispatchUser(cli: Cli, command: UserCommand, sink: EventSink): Promise<number>
/// ```
fn dispatch_user(cli: &Cli, subcommand: &UserCommand, sink: &mut dyn EventSink) -> i32 {
    let config_path = match &cli.user_config {
        Some(path) => absolute_path(path),
        None => match default_user_config_path() {
            Ok(path) => path,
            Err(error) => return usage_failure(sink, &error),
        },
    };
    let endpoint = match user_endpoint(&config_path) {
        Ok(endpoint) => endpoint,
        Err(error) => return usage_failure(sink, &error),
    };
    let mode = match subcommand {
        UserCommand::Write => "write",
        UserCommand::Check => "check",
    };
    return request(&endpoint, true, "run", run_params(mode, &config_path, &[]), sink);
}

/// What:
///  Run the server until it stops;
/// the hidden `serve` command.
/// Why:
///  Started detached by the first client;
/// any failure to start is an `error` event on a stream nobody reads,
/// so the
///  client sees it as the server exiting before it listens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runServer(root: string, socket: string, idleSeconds: number, user: boolean): Promise<number>
/// ```
fn run_server(root: &Path, socket: &Path, idle_seconds: u64, user: bool, sink: &mut dyn EventSink) -> i32 {
    let scope = if user { ConfigScope::User } else { ConfigScope::Repository };
    let options = ServerOptions { root: root.to_path_buf(), scope, socket: socket.to_path_buf(), idle: Duration::from_secs(idle_seconds) };
    match serve(&options, RuleRegistry::with_defaults()) {
        Ok(()) => return 0,
        Err(error) => return usage_failure(sink, &error),
    }
}
