//! What:
//!  The kernel acts one propagation step makes,
//!  behind one seam:
//!  open the checkout's path,
//!  read the mount a descriptor is on,
//!  set that mount private through the descriptor,
//!  and read this process's own mount table.
//! Why:
//!  Every one of them is a call no control on a person's machine may make
//!  answer:
//!  a mount is changed and a table is read,
//!  so the release implementation makes the calls and a control hands in its
//!  own answers
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The tests a build must have",
//!  whose every test states which answer its double gives).
//!  The set is a `mount_setattr(2)` on the descriptor and never a path:
//!  the project refused a path fallback, because a path would act on whatever
//!  stands there at that moment (choice V4).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface PropagationCalls {
//!   open(path): Descriptor; mountOf(descriptor): DescriptorMount; makePrivate(descriptor): void; readTable(): MountFacts[];
//! }
//! ```

use crate::checkout_propagation::mount_descriptor_read::{
    DescriptorMount,
    MountReadFailure,
    read_the_mount,
};
use crate::plain_file::{
    MOUNT_TABLE_LIMIT,
    read_plain_file,
};
use crate::subvolume::fs_mount_options::{
    MOUNTINFO_PATH,
    MountFacts,
    parse_mount_table,
};
use std::ffi::CString;
use std::os::fd::{
    AsRawFd,
    FromRawFd,
};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// The empty path the set takes,
/// which `AT_EMPTY_PATH` tells the kernel to read as the descriptor itself.
const NO_PATH: [libc::c_char; 1] = [0];

/// What:
///  Why one call of the step could not be made.
/// Why:
///  The step names the act that refused in its record,
///  so each answer is the act's own error rather than one sentence a caller
///  has to parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CallFailure = { kind: 'open' | 'read' | 'set' | 'table'; errno?: number; why?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallFailure {
    /// The path could not be opened.
    Open {
        /// The error number the call answered.
        errno: i32,
    },
    /// The descriptor's mount could not be read.
    Read(MountReadFailure),
    /// The kernel refused the set.
    Set {
        /// The error number the call answered.
        errno: i32,
    },
    /// The mount table could not be read.
    Table {
        /// The error number the call answered.
        errno: i32,
    },
}

impl CallFailure {
    /// The error number the answer carries,
    /// where a call answered one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const errno = (failure: CallFailure): number | undefined => ...;
    /// ```
    pub fn errno(&self) -> Option<i32> {
        return match self {
            CallFailure::Open {
                errno,
            }
            | CallFailure::Set {
                errno,
            }
            | CallFailure::Table {
                errno,
            } => Some(*errno),
            CallFailure::Read(failure) => failure.errno(),
        };
    }

    /// The sentence a person reads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const message = (failure: CallFailure): string => ...;
    /// ```
    pub fn message(&self) -> String {
        return match self {
            CallFailure::Open {
                errno,
            } => format!("the checkout's path could not be opened (error {errno})"),
            CallFailure::Read(failure) => failure.message(),
            CallFailure::Set {
                errno,
            } => format!("the kernel refused the set (error {errno})"),
            CallFailure::Table {
                errno,
            } => format!("this process's mount table could not be read (error {errno})"),
        };
    }
}

/// What:
///  The acts one propagation step makes,
///  as one seam a control can answer.
/// Why:
///  The step's own order and its decisions are what the controls test,
///  so the calls are named here once and the release answers them from the
///  kernel while a control answers them from its fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface PropagationCalls { open(path); mountOf(descriptor); makePrivate(descriptor); readTable(); }
/// ```
pub trait PropagationCalls {
    /// Open the checkout's path with the flags the set needs,
    /// or the error the call answered.
    fn open_the_checkout(
        &self,
        path: &Path,
    ) -> Result<std::fs::File, CallFailure>;

    /// Read the mount the descriptor is on.
    fn mount_of(
        &self,
        descriptor: &std::fs::File,
    ) -> Result<DescriptorMount, CallFailure>;

    /// Set the descriptor's mount private,
    /// and nothing beneath it.
    fn make_private(
        &self,
        descriptor: &std::fs::File,
    ) -> Result<(), CallFailure>;

    /// Read this process's own mount table.
    fn read_the_table(&self) -> Result<Vec<MountFacts>, CallFailure>;
}

/// What:
///  The kernel's own answers,
///  which are the calls the release makes.
/// Why:
///  Root is the only account that may change a mount,
///  and the handler runs as root in the host's namespace,
///  so this is the one implementation a run uses and a control never does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const calls: PropagationCalls = theKernelsCalls;
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct TheKernelsCalls;

impl PropagationCalls for TheKernelsCalls {
    fn open_the_checkout(
        &self,
        path: &Path,
    ) -> Result<std::fs::File, CallFailure> {
        let text: CString = CString::new(path.as_os_str().as_bytes()).map_err(|_nul| {
            return CallFailure::Open {
                errno: libc::EINVAL,
            };
        })?;
        // SAFETY: the path is a NUL-terminated string that outlives the call, and the call either answers a descriptor
        // this function then owns or answers negative, in which case no descriptor was made.
        let opened: libc::c_int = unsafe {
            libc::open(
                text.as_ptr(),
                libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC,
            )
        };
        if opened < 0 {
            return Err(CallFailure::Open {
                errno: std::io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EIO),
            });
        }
        // SAFETY: the call answered an open descriptor and nothing else holds it, so this file is its one owner.
        return Ok(unsafe { std::fs::File::from_raw_fd(opened) });
    }

    fn mount_of(
        &self,
        descriptor: &std::fs::File,
    ) -> Result<DescriptorMount, CallFailure> {
        return read_the_mount(descriptor).map_err(CallFailure::Read);
    }

    fn make_private(
        &self,
        descriptor: &std::fs::File,
    ) -> Result<(), CallFailure> {
        let attributes: libc::mount_attr = libc::mount_attr {
            attr_set: 0,
            attr_clr: 0,
            propagation: libc::MS_PRIVATE,
            userns_fd: 0,
        };
        // SAFETY: the descriptor is the caller's and open, the path is the empty string `AT_EMPTY_PATH` makes the call
        // ignore, and the attribute structure outlives the call. No `AT_RECURSIVE` is passed, so no mount beneath the
        // checkout is touched.
        let answered: libc::c_int = unsafe {
            libc::syscall(
                libc::SYS_mount_setattr,
                descriptor.as_raw_fd(),
                NO_PATH.as_ptr(),
                libc::AT_EMPTY_PATH,
                (&attributes as *const libc::mount_attr).cast::<libc::c_void>(),
                std::mem::size_of::<libc::mount_attr>(),
            ) as libc::c_int
        };
        if answered != 0 {
            return Err(CallFailure::Set {
                errno: std::io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or(libc::EIO),
            });
        }
        return Ok(());
    }

    fn read_the_table(&self) -> Result<Vec<MountFacts>, CallFailure> {
        let text: Vec<u8> =
            read_plain_file(Path::new(MOUNTINFO_PATH), MOUNT_TABLE_LIMIT).map_err(|error| {
                return CallFailure::Table {
                    errno: error.raw_os_error().unwrap_or(libc::EIO),
                };
            })?;
        return Ok(parse_mount_table(text.as_slice()));
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "propagation_calls_tests.rs"]
mod tests;
