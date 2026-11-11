//! Marks sockets from selected application cgroups so traffic bypasses WireGuard policy routing.
//!
//! `attach <mark> <cgroup-dir>...` transactionally installs four cgroup socket-address links.
//! `detach <cgroup-dir>...` removes only exact links persisted for each canonical cgroup path.
//! Link pins survive loader exit under `/sys/fs/bpf/wg-quicker-exempt/`.

/// Discovers Ghostty,
///  Steam,
///  Helium,
///  Pale Moon,
///  Firefox release,
///  Firefox Nightly,
///  ChatGPT,
///  Interpreter,
///  Qure,
///  and OdyTTY cgroup targets.
mod application_targets;
/// Reads the resource limits a moved process must keep.
mod inherited_limits;
/// Reads facts about one process from a procfs tree.
mod process_info;
/// Runs a command line client as the watched user.
mod user_client;
/// The D-Bus wire format of the calls to the user's manager.
mod dbus_wire;
/// The login every D-Bus socket of this program takes,
/// and the peer check.
mod dbus_login;
/// The user's session bus,
/// where an impostor manager can take the manager's name.
mod session_bus;
/// The guard that removes an impostor manager from that bus.
mod impostor_guard;
/// The user manager's private socket and the check of the process behind it.
mod manager_socket;
/// The words and outcomes of the update traffic policy.
mod update_report;
/// Subscribes to the kernel process-event connector.
mod process_events;
/// Moves processes into the updates slice through the user's service manager.
mod scope_adoption;
/// The lock an origin stop and a move share.
mod adoption_gate;
/// When one pass of the policy may spend another move.
mod pass_gate;
/// Builds the arguments of the calls to the user's service manager.
mod scope_calls;
/// Names the scope of each origin cgroup.
mod scope_naming;
/// Sweeps the scopes the earlier design left behind.
mod leftover_sweep;
/// Stops the origin scopes no process holds,
/// before the first adoption.
mod origin_scope_sweep;
/// Keeps the limits of each scope equal to its origin's.
mod scope_registry;
/// Recognizes update entry points and toolchain install roots.
mod update_entry_points;
/// The toolchain install roots of the watched user.
mod install_roots;
/// The always-on update traffic policy.
mod update_routing;
/// Looks up the watched user's account.
mod user_account;
/// Non-panicking bounded log and escaping.
mod log_line;
/// Clock,
/// stop check,
/// retry wait
/// and manager reach of the policy engine.
mod routing_support;
/// Shutdown signal descriptors.
mod signal_descriptor;
/// Reports the death of the application watcher.
mod watcher_sentinel;
/// Starts the sentinel outside the watcher's cgroup and confirms it runs.
mod sentinel_launch;
/// The marker a running sentinel leaves.
mod sentinel_marker;
/// The real steps of starting a sentinel.
mod sentinel_system_steps;
/// Commands with a time limit and a fixed search path.
mod bounded_command;
/// Whether the whole system is stopping.
mod system_state;
/// Retains links while watching application cgroup lifecycle.
mod application_watch;
/// Lowercase hexadecimal spelling of arbitrary bytes.
mod bytes_as_lowercase_hex;
/// The command line the program accepts and the work behind it.
mod cli_commands;
/// Whether a descriptor has a line to read,
/// within a limit.
mod line_ready;
/// Starts and stops detached application watcher.
mod application_watch_service;
/// Persists detached application watcher identity.
mod application_watch_state;
/// Delivers failures after startup to a person.
mod runtime_report;
/// The cause of a failure line.
mod report_cause;
/// The desktop notification channel of the failure reporter.
mod desktop_notification;
/// Relays the loud lines of a start to the terminal of `up`.
mod startup_relay;
/// Raw `bpf(2)` ABI and socket-marking program loader.
mod bpf;
/// Stable Linux `bpf(2)` UAPI subset.
mod bpf_uapi;
/// Typed BPF syscall errors for compatibility behavior.
mod bpf_error;
/// Bpffs validation and transactional pin lifecycle.
mod pin;
/// Descriptor-keeper fallback for affected SELinux kernels.
mod keeper;
/// Exact Linux process identity validation for descriptor keeper.
mod keeper_process;
/// Unit tests for the process group a stop must end.
#[cfg(test)]
mod keeper_process_tests;
/// Crash-recoverable descriptor-keeper state.
mod keeper_state;
/// Unit tests for application target discovery.
#[cfg(test)]
mod application_targets_tests;
/// Unit tests for the update entry-point list.
#[cfg(test)]
mod update_entry_points_tests;
/// Unit tests for the install roots.
#[cfg(test)]
mod install_roots_tests;
/// Unit tests for the process-event connector.
#[cfg(test)]
mod process_events_tests;
/// Unit tests for the update traffic policy.
#[cfg(test)]
mod update_routing_tests;
/// Unit tests for the words and outcomes of the policy.
#[cfg(test)]
mod update_report_tests;
/// Unit tests for the inherited resource limits.
#[cfg(test)]
mod inherited_limits_tests;
/// Unit tests for the relay of startup failures.
#[cfg(test)]
mod application_watch_service_tests;
/// Unit tests for the watch setup.
#[cfg(test)]
mod application_watch_tests;
/// Unit tests for failure delivery.
#[cfg(test)]
mod runtime_report_tests;
/// Unit tests for the cause of a failure line.
#[cfg(test)]
mod report_cause_tests;
/// Unit tests for the log and escaping.
#[cfg(test)]
mod log_line_tests;
/// Unit tests for the death watch.
#[cfg(test)]
mod watcher_sentinel_tests;
/// Unit tests for the sentinel launch.
#[cfg(test)]
mod sentinel_launch_tests;
/// Unit tests for the sentinel marker.
#[cfg(test)]
mod sentinel_marker_tests;
/// Unit tests for the real steps of starting a sentinel.
#[cfg(test)]
mod sentinel_system_steps_tests;
/// Unit tests for bounded commands.
#[cfg(test)]
mod bounded_command_tests;
/// Unit tests for the stopping system.
#[cfg(test)]
mod system_state_tests;
/// Unit tests for the user client.
#[cfg(test)]
mod user_client_tests;
/// Unit tests for the wire format of the manager calls.
#[cfg(test)]
mod dbus_wire_tests;
/// Unit tests for the manager's private socket.
#[cfg(test)]
mod manager_socket_tests;
/// A live probe of the manager channel,
/// ignored by default.
#[cfg(test)]
mod manager_probe_tests;
/// Unit tests for the guard of the manager's bus name.
#[cfg(test)]
mod impostor_guard_tests;
/// Unit tests for the session bus client.
#[cfg(test)]
mod session_bus_tests;
/// Unit tests for the bounded wait on a line.
#[cfg(test)]
mod line_ready_tests;
/// Unit tests for scope adoption.
#[cfg(test)]
mod scope_adoption_tests;
/// Unit tests for the manager calls the watcher builds.
#[cfg(test)]
mod scope_calls_tests;
/// Unit tests for scope names.
#[cfg(test)]
mod scope_naming_tests;
/// Unit tests for the sweep of leftover scopes.
#[cfg(test)]
mod leftover_sweep_tests;
/// Unit tests for the lock an origin stop and a move share.
#[cfg(test)]
mod adoption_gate_tests;
/// Unit tests for the pass gate,
/// which check each limit on its own.
#[cfg(test)]
mod pass_gate_tests;
/// Unit tests for following the limits of origins.
#[cfg(test)]
mod scope_registry_tests;
/// Unit tests for process facts and the account lookup.
#[cfg(test)]
mod process_info_tests;
/// Unit tests for raw BPF instruction encoding.
#[cfg(test)]
mod bpf_tests;
/// Privileged lifecycle and protocol tests.
#[cfg(test)]
mod pin_tests;

/// Filesystem path view for cgroup arguments.
use std::path::Path;
/// Process exit code returned from `main`.
use std::process::ExitCode;

/// Program entry point with explicit attach and detach argument contracts.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        cli_commands::usage();
        return ExitCode::from(2);
    };
    if command == "__watch" {
        if args.len() != 4 {
            return ExitCode::from(2);
        }
        let mark = match cli_commands::parse_mark(&args[2]) {
            Ok(value) => value,
            Err(error) => return cli_commands::finish(Err(error)),
        };
        let uid = match cli_commands::parse_uid(&args[3]) {
            Ok(value) => value,
            Err(error) => return cli_commands::finish(Err(error)),
        };
        return cli_commands::finish(application_watch_service::run_application_watch_child(
            mark,
            uid,
            None,
        ));
    }
    #[cfg(debug_assertions)]
    if command == "__watch-test" {
        if args.len() != 3 {
            return ExitCode::from(2);
        }
        let mark = match cli_commands::parse_mark(&args[1]) {
            Ok(value) => value,
            Err(error) => return cli_commands::finish(Err(error)),
        };
        return cli_commands::finish(application_watch_service::run_application_watch_child(
            mark,
            0,
            Some(Path::new(&args[2])),
        ));
    }
    if command == watcher_sentinel::SENTINEL_COMMAND {
        if args.len() != 6 {
            return ExitCode::from(2);
        }
        let (Ok(pid), Ok(start_time), Ok(uid), Ok(gid)) = (args[2].parse::<i32>(), args[3].parse::<u64>(), args[4].parse::<u32>(), args[5].parse::<u32>()) else {
            return ExitCode::from(2);
        };
        return cli_commands::finish(watcher_sentinel::run_sentinel_command(&args[1], pid, start_time, uid, gid));
    }
    if command == "__hold" {
        if args.len() != 3 {
            return ExitCode::from(2);
        }
        let mark = match cli_commands::parse_mark(&args[1]) {
            Ok(value) => value,
            Err(error) => return cli_commands::finish(Err(error)),
        };
        return cli_commands::finish(keeper::run_holder(mark, Path::new(&args[2])));
    }
    if command == "list-targets" {
        if args.len() != 2 {
            cli_commands::usage();
            return ExitCode::from(2);
        }
        let uid = match cli_commands::parse_uid(&args[1]) {
            Ok(value) => value,
            Err(error) => return cli_commands::finish(Err(error)),
        };
        let targets = match application_targets::scan_user_application_targets(uid) {
            Ok(value) => value,
            Err(error) => return cli_commands::finish(Err(error)),
        };
        for target in targets.targets {
            println!("{}", target.display());
        }
        for cgroup in targets.without_application {
            println!(
                "not marked, no program of the application its name claims is in it: {}",
                cgroup.display()
            );
        }
        return ExitCode::SUCCESS;
    }
    if command == impostor_guard::GUARD_COMMAND {
        if args.len() != 4 {
            return ExitCode::from(2);
        }
        let (Ok(uid), Ok(poll_millis), Ok(reexec_millis)) = (args[1].parse::<u32>(), args[2].parse::<u64>(), args[3].parse::<u64>()) else {
            return ExitCode::from(2);
        };
        let setup = impostor_guard::GuardSetup {
            uid,
            poll_interval: std::time::Duration::from_millis(poll_millis),
            reexec_gap: std::time::Duration::from_millis(reexec_millis),
        };
        let mut steps = impostor_guard::RealSteps::new(std::time::Duration::from_secs(5));
        return cli_commands::finish(impostor_guard::run_child(&setup, &mut steps));
    }
    if command == "watch-start" {
        if args.len() != 4 {
            cli_commands::usage();
            return ExitCode::from(2);
        }
        let mark = match cli_commands::parse_mark(&args[2]) {
            Ok(value) => value,
            Err(error) => return cli_commands::finish(Err(error)),
        };
        let uid = match cli_commands::parse_uid(&args[3]) {
            Ok(value) => value,
            Err(error) => return cli_commands::finish(Err(error)),
        };
        let result = match pin::acquire_lifecycle_lock() {
            Ok(_lock) => application_watch_service::start_application_watch(&args[1], mark, uid),
            Err(error) => Err(error),
        };
        return cli_commands::finish(result);
    }
    if command == "watch-stop" {
        if args.len() != 2 {
            cli_commands::usage();
            return ExitCode::from(2);
        }
        let result = match pin::acquire_lifecycle_lock() {
            Ok(_lock) => application_watch_service::stop_application_watch(&args[1]),
            Err(error) => Err(error),
        };
        return cli_commands::finish(result);
    }
    if command == "attach" {
        if args.len() < 3 {
            cli_commands::usage();
            return ExitCode::from(2);
        }
        let mark = match cli_commands::parse_mark(&args[1]) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("error: {error}");
                return ExitCode::from(2);
            }
        };
        return cli_commands::finish(cli_commands::attach_many(mark, &args[2..]));
    }
    if command == "detach" {
        if args.len() < 2 {
            cli_commands::usage();
            return ExitCode::from(2);
        }
        return cli_commands::finish(cli_commands::detach_many(&args[1..]));
    }
    cli_commands::usage();
    return ExitCode::from(2);
}

/// Registers the unit-test scratch root,
///  placed in the build directory instead of `/tmp`.
#[cfg(test)]
mod test_scratch;
