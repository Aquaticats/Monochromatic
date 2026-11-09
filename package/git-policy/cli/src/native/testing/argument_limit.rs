//! What:
//!  The size the platform lets one command-line argument or one environment string reach,
//!  shared by the controls that prove a value cannot travel that way.
//! Why:
//!  Such a control proves nothing unless the value it carries exceeds this size,
//!  and the same control then proves the value reaches Git once a file carries it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const limit = 32 * sysconf(_SC_PAGESIZE);
//! ```

/// What:
///  The size the platform lets one command-line argument or one environment string reach.
/// Why:
///   `MAX_ARG_STRLEN` is 32 pages,
///  so a control that proves a value cannot travel on the command line must exceed this,
///  and the same control proves the value reaches Git once a configuration file carries it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const limit = 32 * sysconf(_SC_PAGESIZE);
/// ```
pub(crate) fn one_argument_limit() -> usize {
    // SAFETY: `sysconf` takes a constant and writes nothing.
    let page_bytes: libc::c_long = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    // The largest page size a Linux machine uses stands in when the size cannot be read.
    let page: usize = usize::try_from(page_bytes).unwrap_or(1 << 16);
    return 32 * page;
}
