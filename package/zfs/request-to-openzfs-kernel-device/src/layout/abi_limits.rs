//! What:
//!  The sizes the kernel's `zfs_cmd_t` gives each fixed name field,
//! and the
//!  size the whole struct has on OpenZFS `zfs-2.4.4`.
//! Why:
//!  The names module checks a caller's text against the field it will land in,
//!  and the layout's own size assertions name these constants instead of
//!  repeating a count in each place that reads them.

/// `MAXPATHLEN`:
/// bytes in `zc_name`,
/// terminating NUL included.
pub const MAXPATHLEN: usize = 4096;

/// `MAXNAMELEN`:
/// bytes in `zc_string`,
/// terminating NUL included.
pub const MAXNAMELEN: usize = 256;

/// `MAXPATHLEN * 2`:
/// bytes in `zc_value`,
/// terminating NUL included.
pub const MAX_VALUE_LEN: usize = MAXPATHLEN * 2;

/// `sizeof (zfs_cmd_t)` on OpenZFS `zfs-2.4.4`,
/// from the module's BTF.
pub const COMMAND_STRUCT_SIZE: usize = 13_744;
