//! What:
//!  The worker's two requests to the mode for the holder,
//!  and the two scans it compares around that holder:
//!  this process's own namespace,
//!  and the namespaces the scan of `/proc` names.
//! Why:
//!  The mode's holder is the only other process holding the source,
//!  so the scan taken while it runs must name exactly the worker's and the
//!  holder's namespaces,
//!  and the scan after it must name the worker's alone.
//!  The arm reports a leaked or a missing namespace by its name,
//!  rather than a count two mistakes could keep right.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holder = askHolder(requestPipe, replyPipe, HOLDER_START); scanExpecting(source, [ownNamespace(), holder]);
//! ```

use super::probe_namespace_scan::{
    compare_namespaces,
    scan_namespaces,
};
use super::probe_pipe_lines::{
    read_line_blocking,
    write_all,
};
use std::os::fd::RawFd;
use std::path::Path;

/// What:
///  Ask the mode to start or stop the holder,
///  and answer the holder's namespace when it started.
/// Why:
///  Requests are single bytes,
///  and the reply is one line so the first pass can compare the exact set
///  of namespaces with the worker's and the holder's rather than a count.
pub(super) fn ask_holder(
    request: RawFd,
    reply: RawFd,
    request_byte: u8,
) -> Option<String> {
    if write_all(request, &[request_byte]).is_err() {
        return None;
    }
    let line: String = read_line_blocking(reply, 128).ok()?;
    if line == "P" {
        return Some(String::new());
    }
    return line.strip_prefix("P ").map(String::from);
}

/// This process's own mount-namespace identity.
pub(super) fn own_namespace() -> Result<String, ()> {
    return std::fs::read_link("/proc/self/ns/mnt")
        .map(|path| return path.to_string_lossy().into_owned())
        .map_err(|_| ());
}

/// Scan this PID namespace for the source and require exactly the namespaces
///  the pass expects,
///  naming a leaked or missing namespace rather than comparing a count.
pub(super) fn scan_expecting(
    source: &str,
    expected: &[&str],
) -> Result<(), ()> {
    let mentioned: Vec<String> = match scan_namespaces(Path::new("/proc"), source) {
        Ok(mentioned) => mentioned,
        Err(_) => return Err(()),
    };
    return match compare_namespaces(mentioned.as_slice(), expected) {
        None => Ok(()),
        Some(_difference) => Err(()),
    };
}
