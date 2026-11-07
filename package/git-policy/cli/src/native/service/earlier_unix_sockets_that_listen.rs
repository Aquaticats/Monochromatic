//! What:
//!  Whether a socket of an earlier release's unit listens, read from the table of unix sockets (`/proc/net/unix`).
//! Why:
//!  Stub until the controls for it are written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! earlierSocketOfInstanceListens(procRoot, instance)
//! ```

use std::path::Path;

/// Whether the earlier release's socket of an instance listens; a table that cannot be read counts as "yes".
pub fn earlier_socket_of_instance_listens(proc_root: &Path, instance: &str) -> bool {
    let _unused = (proc_root, instance);
    return false;
}
