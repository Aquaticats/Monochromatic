//! Marks sockets from selected application cgroups so traffic bypasses WireGuard policy routing.
//!
//! `attach <mark> <cgroup-dir>...` transactionally installs four cgroup socket-address links.
//! `detach <cgroup-dir>...` removes only exact links persisted for each canonical cgroup path.
//! Link pins survive loader exit under `/sys/fs/bpf/wg-quicker-exempt/`.

/// Discovers Ghostty,
///  Steam,
///  Helium,
///  Pale Moon,
///  Firefox Nightly,
///  ChatGPT,
///  Interpreter,
///  and Qure cgroup targets.
mod application_targets;
/// Retains links while watching application cgroup lifecycle.
mod application_watch;
/// Starts and stops detached application watcher.
mod application_watch_service;
/// Persists detached application watcher identity.
mod application_watch_state;
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
/// Crash-recoverable descriptor-keeper state.
mod keeper_state;
/// Unit tests for application target discovery.
#[cfg(test)]
mod application_targets_tests;
/// Unit tests for raw BPF instruction encoding.
#[cfg(test)]
mod bpf_tests;
/// Privileged lifecycle and protocol tests.
#[cfg(test)]
mod pin_tests;

/// Standard I/O error type used across command boundaries.
use std::io;
/// Filesystem path view for cgroup arguments.
use std::path::Path;
/// Process exit code returned from `main`.
use std::process::ExitCode;

/// Parses positive nonzero socket mark accepted by policy-routing contract.
fn parse_mark(value: &str) -> io::Result<u32> {
    let mark: u32 = value.parse().map_err(|error| {
        return io::Error::new(io::ErrorKind::InvalidInput, format!("invalid mark {value}: {error}"));
    })?;
    if mark == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "mark must be positive",
        ));
    }
    return Ok(mark);
}

/// Parses numeric user ID selecting app slice to watch.
fn parse_uid(value: &str) -> io::Result<u32> {
    return value.parse().map_err(|error| {
        return io::Error::new(io::ErrorKind::InvalidInput, format!("invalid UID {value}: {error}"));
    });
}

/// Attaches marker transaction to every listed cgroup while global lock is held.
fn attach_many(mark: u32, dirs: &[String]) -> io::Result<()> {
    let _lock = pin::acquire_lifecycle_lock()?;
    for dir in dirs {
        let cgroup = Path::new(dir);
        match pin::attach_cgroup(mark, cgroup) {
            Ok(count) => {
                keeper::detach_keeper(cgroup)?;
                println!("attached mark={mark} to {dir} ({count} links pinned)");
            }
            Err(error) if bpf_error::is_pin_object_invalid(&error) => {
                keeper::replace_keeper(mark, cgroup)?;
                pin::detach_cgroup(cgroup)?;
                println!(
                    "attached mark={mark} to {dir} (4 links held by SELinux-regression fallback)"
                );
            }
            Err(error) => {
                return Err(io::Error::new(error.kind(), format!("attach {dir}: {error}")));
            }
        }
    }
    return Ok(());
}

/// Detaches exact marker links from every listed cgroup while global lock is held.
fn detach_many(dirs: &[String]) -> io::Result<()> {
    let _lock = pin::acquire_lifecycle_lock()?;
    for dir in dirs {
        let cgroup = Path::new(dir);
        keeper::detach_keeper(cgroup).map_err(|error| {
            return io::Error::new(error.kind(), format!("detach fallback {dir}: {error}"));
        })?;
        pin::detach_cgroup(cgroup).map_err(|error| {
            return io::Error::new(error.kind(), format!("detach pins {dir}: {error}"));
        })?;
        println!("detached {dir}");
    }
    return Ok(());
}

/// Prints accepted command forms to stderr.
fn usage() {
    eprintln!("usage: wg-quicker-exempt attach <mark> <cgroup-dir>...");
    eprintln!("       wg-quicker-exempt detach <cgroup-dir>...");
    eprintln!("       wg-quicker-exempt list-targets <uid>");
    eprintln!("       wg-quicker-exempt watch-start <key> <mark> <uid>");
    eprintln!("       wg-quicker-exempt watch-stop <key>");
}

/// Converts command result into conventional process exit code and diagnostic.
fn finish(result: io::Result<()>) -> ExitCode {
    match result {
        Ok(()) => return ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    }
}

/// Program entry point with explicit attach and detach argument contracts.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        usage();
        return ExitCode::from(2);
    };
    if command == "__watch" {
        if args.len() != 4 {
            return ExitCode::from(2);
        }
        let mark = match parse_mark(&args[2]) {
            Ok(value) => value,
            Err(error) => return finish(Err(error)),
        };
        let uid = match parse_uid(&args[3]) {
            Ok(value) => value,
            Err(error) => return finish(Err(error)),
        };
        return finish(application_watch_service::run_application_watch_child(
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
        let mark = match parse_mark(&args[1]) {
            Ok(value) => value,
            Err(error) => return finish(Err(error)),
        };
        return finish(application_watch_service::run_application_watch_child(
            mark,
            0,
            Some(Path::new(&args[2])),
        ));
    }
    if command == "__hold" {
        if args.len() != 3 {
            return ExitCode::from(2);
        }
        let mark = match parse_mark(&args[1]) {
            Ok(value) => value,
            Err(error) => return finish(Err(error)),
        };
        return finish(keeper::run_holder(mark, Path::new(&args[2])));
    }
    if command == "list-targets" {
        if args.len() != 2 {
            usage();
            return ExitCode::from(2);
        }
        let uid = match parse_uid(&args[1]) {
            Ok(value) => value,
            Err(error) => return finish(Err(error)),
        };
        let targets = match application_targets::scan_user_application_targets(uid) {
            Ok(value) => value,
            Err(error) => return finish(Err(error)),
        };
        for target in targets {
            println!("{}", target.display());
        }
        return ExitCode::SUCCESS;
    }
    if command == "watch-start" {
        if args.len() != 4 {
            usage();
            return ExitCode::from(2);
        }
        let mark = match parse_mark(&args[2]) {
            Ok(value) => value,
            Err(error) => return finish(Err(error)),
        };
        let uid = match parse_uid(&args[3]) {
            Ok(value) => value,
            Err(error) => return finish(Err(error)),
        };
        let result = pin::acquire_lifecycle_lock().and_then(|_lock| {
            return application_watch_service::start_application_watch(&args[1], mark, uid);
        });
        return finish(result);
    }
    if command == "watch-stop" {
        if args.len() != 2 {
            usage();
            return ExitCode::from(2);
        }
        let result = pin::acquire_lifecycle_lock().and_then(|_lock| {
            return application_watch_service::stop_application_watch(&args[1]);
        });
        return finish(result);
    }
    if command == "attach" {
        if args.len() < 3 {
            usage();
            return ExitCode::from(2);
        }
        let mark = match parse_mark(&args[1]) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("error: {error}");
                return ExitCode::from(2);
            }
        };
        return finish(attach_many(mark, &args[2..]));
    }
    if command == "detach" {
        if args.len() < 2 {
            usage();
            return ExitCode::from(2);
        }
        return finish(detach_many(&args[1..]));
    }
    usage();
    return ExitCode::from(2);
}
