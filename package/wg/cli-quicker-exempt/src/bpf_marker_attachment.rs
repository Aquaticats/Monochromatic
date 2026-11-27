//! Attaching the socket-marking program to a cgroup:
//! the hooks every marker covers,
//! the links that keep it attached,
//! and the rollback of a half-finished attach.

/// The syscalls each link is made of.
use super::bpf_syscalls::{
    attach_link,
    create_mark_map,
    load_prog,
    pin_obj,
    pin_path_contains_nul,
};
/// Stable Linux command constants.
use crate::bpf_uapi::{
    BPF_CGROUP_INET4_CONNECT,
    BPF_CGROUP_INET6_CONNECT,
    BPF_CGROUP_UDP4_SENDMSG,
    BPF_CGROUP_UDP6_SENDMSG,
};
/// Standard I/O error type used for syscall failures.
use std::io;
/// Owned descriptor that closes automatically when its value leaves scope.
use std::os::fd::OwnedFd;

/// All hooks the marker installs,
///  covering TCP connect and UDP sendmsg,
///  v4 and v6.
pub const HOOKS: [u32; 4] = [
    BPF_CGROUP_INET4_CONNECT,
    BPF_CGROUP_INET6_CONNECT,
    BPF_CGROUP_UDP4_SENDMSG,
    BPF_CGROUP_UDP6_SENDMSG,
];

/// Hook names used for pinned-link file names,
///  parallel to [`HOOKS`].
pub const HOOK_NAMES: [&str; 4] = ["connect4", "connect6", "udp4_sendmsg", "udp6_sendmsg"];

/// Owns unpinned links so dropping this value atomically detaches its marker.
pub struct MarkerLinks {
    /// Link descriptors retain programs and their shared mark map.
    _links: Vec<OwnedFd>,
}

/// Loads and attaches one hook,
///  returning link descriptor that owns attachment.
fn create_link(
    map_fd: &OwnedFd,
    cgroup_fd: i32,
    hook: u32,
) -> io::Result<OwnedFd> {
    let prog_fd = load_prog(map_fd, hook)?;
    return attach_link(&prog_fd, cgroup_fd, hook);
}

/// Loads,
///  attaches,
///  and pins one hook while descriptors remain automatically owned.
fn attach_one(
    map_fd: &OwnedFd,
    cgroup_fd: i32,
    hook: u32,
    hook_name: &str,
    pin_dir: &str,
) -> io::Result<String> {
    let link_fd = create_link(map_fd, cgroup_fd, hook)?;
    let path = format!("{pin_dir}/{hook_name}");
    let c_path = std::ffi::CString::new(path.as_str()).map_err(pin_path_contains_nul)?;
    pin_obj(&link_fd, &c_path)?;
    return Ok(path);
}

/// Removes only link pins created by an interrupted four-hook transaction.
fn rollback_pins(pinned: &[String]) -> io::Result<()> {
    let mut first_error: Option<io::Error> = None;
    for path in pinned {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }
    if let Some(error) = first_error {
        return Err(error);
    }
    return Ok(());
}

/// Attaches links and optionally injects test failure after specified completed count.
fn attach_marker_unpinned_until(
    cgroup_fd: i32,
    mark: u32,
    fail_after: Option<usize>,
) -> io::Result<MarkerLinks> {
    let map_fd = create_mark_map(mark)?;
    let mut links = Vec::with_capacity(HOOKS.len());
    for hook in HOOKS {
        links.push(create_link(&map_fd, cgroup_fd, hook)?);
        if fail_after == Some(links.len()) {
            return Err(io::Error::other("injected partial-link failure"));
        }
    }
    return Ok(MarkerLinks {
        _links: links,
    });
}

/// Attaches all marker hooks without persistence for privileged protocol verification.
pub fn attach_marker_unpinned(
    cgroup_fd: i32,
    mark: u32,
) -> io::Result<MarkerLinks> {
    return attach_marker_unpinned_until(cgroup_fd, mark, None);
}

/// Injects partial attach failure so descriptor rollback can be observed.
#[cfg(test)]
pub fn attach_marker_unpinned_failing_after(
    cgroup_fd: i32,
    mark: u32,
    link_count: usize,
) -> io::Result<MarkerLinks> {
    return attach_marker_unpinned_until(cgroup_fd, mark, Some(link_count));
}

/// Attaches the socket-marking program to one cgroup for every hook and pins each
/// resulting link under an empty staging directory.
///
/// A failed hook removes every earlier pin,
///  so no partial attachment survives.
pub fn attach_marker(
    cgroup_fd: i32,
    mark: u32,
    pin_dir: &str,
) -> io::Result<Vec<String>> {
    let map_fd = create_mark_map(mark)?;
    let mut pinned = Vec::with_capacity(HOOKS.len());
    for (index, hook) in HOOKS.iter().enumerate() {
        let result = attach_one(&map_fd, cgroup_fd, *hook, HOOK_NAMES[index], pin_dir);
        match result {
            Ok(path) => pinned.push(path),
            Err(error) => {
                if let Err(rollback_error) = rollback_pins(&pinned) {
                    return Err(io::Error::new(
                        error.kind(),
                        format!("{error}; partial-pin rollback also failed: {rollback_error}"),
                    ));
                }
                return Err(error);
            }
        }
    }
    return Ok(pinned);
}
