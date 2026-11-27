//! Minimal Linux `bpf(2)` UAPI bindings and the socket-marking program builder.
//!
//! The `libc` crate does not expose the BPF syscall's `bpf_attr` union or its
//! constants,
//!  so the small stable-ABI subset used here is defined directly.
//! The program marks sockets with `SO_MARK` so a policy-routing rule can send
//! their traffic outside the WireGuard tunnel.
//!
//! The `dst_src` instruction byte follows Linux's little-endian bitfield layout.
//! Big-endian targets fail at compile time instead of loading a misencoded program.

/// The `bpf(2)` calls one attach is made of.
#[path = "bpf_syscalls.rs"]
mod bpf_syscalls;
/// The socket-marking program itself.
#[path = "bpf_mark_program.rs"]
mod mark_program;
/// Attaching the program to a cgroup.
#[path = "bpf_marker_attachment.rs"]
mod marker_attachment;

/// Returns stable opcode,
///  offset,
///  and immediate triples for unit verification.
#[cfg(test)]
pub use mark_program::mark_program_snapshot;
/// Hook names used for pinned-link file names,
///  parallel to [`HOOKS`].
pub use marker_attachment::HOOK_NAMES;
/// Owns unpinned links so dropping this value atomically detaches its marker.
pub use marker_attachment::MarkerLinks;
/// Attaches the socket-marking program to one cgroup for every hook and pins each
/// resulting link under an empty staging directory.
pub use marker_attachment::attach_marker;
/// Attaches all marker hooks without persistence for privileged protocol verification.
pub use marker_attachment::attach_marker_unpinned;
/// Injects partial attach failure so descriptor rollback can be observed.
#[cfg(test)]
pub use marker_attachment::attach_marker_unpinned_failing_after;

#[cfg(not(target_endian = "little"))]
compile_error!("wg-quicker-exempt currently supports little-endian Linux targets only");
