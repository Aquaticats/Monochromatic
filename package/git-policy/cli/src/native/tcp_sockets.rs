//! What:
//!  The TCP sockets of this machine that are connected, with the bytes each has sent and had acknowledged and the bytes it has
//!  received, and the sockets a process holds.
//! Why:
//!  `rchar` and `wchar` of `/proc/<pid>/io` count `read` and `write`, not `send` and `recv`, so a transfer done through `libcurl`
//!  (`git-remote-http`) moves no byte the push meter can see, and `/proc/net/tcp` has queue lengths and no running count. The
//!  kernel's own socket diagnostics (`NETLINK_SOCK_DIAG`, `man 7 sock_diag`, the interface `ss` reads) return, for every
//!  socket, its state, its inode and a `struct tcp_info` with `tcpi_bytes_acked` and `tcpi_bytes_received`; a socket a process
//!  holds is the `socket:[<inode>]` link of its `/proc/<pid>/fd` (`man 5 proc_pid_fd`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sockets = establishedSockets(); const mine = socketInodesOf(pid); sockets.filter((socket) => mine.includes(socket.inode));
//! ```

/// The state number of a connected socket, `TCP_ESTABLISHED` in `include/net/tcp_states.h`.
pub const ESTABLISHED: u8 = 1;

/// One connected TCP socket as the kernel reports it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TcpSocket {
    /// The socket's inode, the number in `socket:[<inode>]`.
    pub inode: u64,
    /// The state number (`ESTABLISHED`).
    pub state: u8,
    /// Bytes sent that the peer has acknowledged (`tcpi_bytes_acked`).
    pub bytes_acked: u64,
    /// Bytes received (`tcpi_bytes_received`).
    pub bytes_received: u64,
}

/// Every connected TCP socket of the network namespace of this process, or nothing where the kernel cannot say.
// RED SCAFFOLD: answers nothing until the diagnostics are read.
pub fn established_sockets() -> Option<Vec<TcpSocket>> {
    return None;
}

/// The inodes of the sockets a process holds open, or nothing where its descriptors cannot be read.
// RED SCAFFOLD: answers nothing until the descriptors are read.
pub fn socket_inodes_of(_pid: i32) -> Vec<u64> {
    return Vec::new();
}

/// Controls for what the kernel reports.
#[cfg(test)]
#[path = "tcp_sockets_tests.rs"]
mod tests;
