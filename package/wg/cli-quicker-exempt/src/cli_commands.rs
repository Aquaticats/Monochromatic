//! The commands the program accepts and the work each does:
//! the socket mark's parsing,
//! the user's,
//! attaching and detaching the marking links,
//! and the words of the usage text.
//! Kept apart from the dispatcher so the entry point reads as the list of commands it offers.

/// Raw `bpf(2)` errors that decide between the pin path and the fallback.
use crate::bpf_error;
/// The descriptor keeper that holds the links when pinning is refused.
use crate::keeper;
/// The marking links and the lifecycle lock.
use crate::pin;
/// Standard I/O error type used across command boundaries.
use std::io;
/// Filesystem path view for cgroup arguments.
use std::path::Path;
/// Process exit code returned from `main`.
use std::process::ExitCode;

/// Parses positive nonzero socket mark accepted by policy-routing contract.
pub fn parse_mark(value: &str) -> io::Result<u32> {
    let mark: u32 = match value.parse() {
        Ok(parsed) => parsed,
        Err(error) => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("invalid mark {value}: {error}")));
        }
    };
    if mark == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "mark must be positive",
        ));
    }
    return Ok(mark);
}

/// Parses numeric user ID selecting app slice to watch.
pub fn parse_uid(value: &str) -> io::Result<u32> {
    return match value.parse() {
        Ok(uid) => Ok(uid),
        Err(error) => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("invalid UID {value}: {error}"))),
    };
}

/// Attaches marker transaction to every listed cgroup while global lock is held.
pub fn attach_many(mark: u32, dirs: &[String]) -> io::Result<()> {
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
pub fn detach_many(dirs: &[String]) -> io::Result<()> {
    let _lock = pin::acquire_lifecycle_lock()?;
    for dir in dirs {
        let cgroup = Path::new(dir);
        if let Err(error) = keeper::detach_keeper(cgroup) {
            return Err(io::Error::new(error.kind(), format!("detach fallback {dir}: {error}")));
        }
        if let Err(error) = pin::detach_cgroup(cgroup) {
            return Err(io::Error::new(error.kind(), format!("detach pins {dir}: {error}")));
        }
        println!("detached {dir}");
    }
    return Ok(());
}

/// Prints accepted command forms to stderr.
pub fn usage() {
    eprintln!("usage: wg-quicker-exempt attach <mark> <cgroup-dir>...");
    eprintln!("       wg-quicker-exempt detach <cgroup-dir>...");
    eprintln!("       wg-quicker-exempt list-targets <uid>");
    eprintln!("       wg-quicker-exempt watch-start <key> <mark> <uid>");
    eprintln!("       wg-quicker-exempt watch-stop <key>");
}

/// Converts command result into conventional process exit code and diagnostic.
pub fn finish(result: io::Result<()>) -> ExitCode {
    match result {
        Ok(()) => return ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    }
}
