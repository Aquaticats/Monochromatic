//! What:
//!  The read that names the user namespace owning a mount namespace,
//!  and the comparison with the handler's own.
//! Why:
//!  The kernel locks every mount copied into a namespace whose user namespace is
//!  new,
//!  and no interface prints whether a mount is locked,
//!  so the preflight asks the kernel for the owner instead:
//!  `NS_GET_USERNS` on the mount namespace's descriptor,
//!  then its inode against `/proc/self/ns/user`'s
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  F59,
//!  F61,
//!  and "The preflight").
//!  A different owner is a locked holder,
//!  and both ways out of the namespace answer `EINVAL` there,
//!  which this round's guest arm measured.
//!  The read is conservative:
//!  a copy a holder mounted itself inside its own user namespace need not be
//!  locked,
//!  and the read refuses that holder too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owner = readTheOwner('/proc/1234/ns/mnt');  // { namespace: 'mnt:[…]', owner, ours, locked }
//! ```

use std::ffi::CString;
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Where this process's own user namespace is named.
/// Why:
///  The handler runs in the initial user namespace,
///  and this is the one path a handler can compare an owner with.
pub const OUR_USER_NAMESPACE: &str = "/proc/self/ns/user";

/// What:
///  A mount namespace's identity and the inode of the user namespace that owns
///  it.
/// Why:
///  The handler compares the owner with its own,
///  and a report has to name which namespace it refused,
///  so the identity and the owner travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceOwner = { namespace: string; owner: number; ours: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceOwner {
    /// The namespace's kernel identity,
    /// such as `mnt:[4026532001]`.
    pub namespace: String,
    /// The inode of the user namespace that owns it.
    pub owner: u64,
    /// The inode of this process's own user namespace.
    pub ours: u64,
}

impl NamespaceOwner {
    /// What:
    ///  Whether the kernel locked every mount this namespace received.
    /// Why:
    ///  A namespace owned by another user namespace has its copied mounts
    ///  locked,
    ///  and the act refuses such a holder before any signal (V10).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const locked = owner.owner !== owner.ours;
    /// ```
    pub fn locked(&self) -> bool {
        return self.owner != self.ours;
    }
}

/// What:
///  Why the owner could not be read.
/// Why:
///  A namespace that cannot be opened,
///  an ioctl the kernel refused,
///  and a descriptor it refused to describe are three different things for a
///  person,
///  and each refusal names the errno the call answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnerRefusal = { calling: 'open' | 'ioctl' | 'stat'; what: string; errno: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerRefusal {
    /// Which call refused.
    pub calling: &'static str,
    /// What it was asked for,
    /// in plain words.
    pub what: String,
    /// The errno it answered.
    pub errno: i32,
}

impl std::fmt::Display for OwnerRefusal {
    /// One sentence naming the call, what it was asked and the errno.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return write!(
            formatter,
            "{} refused for {} with errno {}",
            self.calling, self.what, self.errno
        );
    }
}

/// What:
///  Open one namespace by its `/proc` path,
///  following no link.
/// Why:
///  The path a scan reported may be replaced before the act reads it,
///  and the descriptor is what the ioctl and `setns` must act on.
///
/// # Errors
/// The path could not be opened.
pub fn open_a_namespace(path: &Path) -> Result<std::fs::File, OwnerRefusal> {
    let text: CString = CString::new(path.as_os_str().as_bytes()).map_err(|_nul| {
        return OwnerRefusal {
            calling: "open",
            what: format!("a namespace path holding a NUL byte: {}", path.display()),
            errno: libc::EINVAL,
        };
    })?;
    // SAFETY: `text` is one NUL-terminated path that outlives the call, and the descriptor it answers is owned exactly
    // once, by the `File` below.
    let descriptor: libc::c_int =
        unsafe { libc::open(text.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
    if descriptor < 0 {
        return Err(OwnerRefusal {
            calling: "open",
            what: String::from(path.display().to_string()),
            errno: std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EINVAL),
        });
    }
    // SAFETY: the call answered a descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe {
        OwnedFd::from_raw_fd(descriptor)
    }));
}

/// What:
///  A namespace's kernel identity,
///  as `/proc` spells it.
/// Why:
///  Every journal line and refusal names the namespace it acted on,
///  and the identity is the inode of the namespace object,
///  which names one object across the machine.
///
/// # Errors
/// The descriptor could not be described.
pub fn identity_of(namespace: &std::fs::File) -> Result<String, OwnerRefusal> {
    let facts: std::fs::Metadata = namespace.metadata().map_err(|error| {
        return OwnerRefusal {
            calling: "stat",
            what: String::from("a namespace descriptor"),
            errno: error.raw_os_error().unwrap_or(libc::EINVAL),
        };
    })?;
    return Ok(format!("mnt:[{}]", facts.ino()));
}

/// What:
///  The inode of the user namespace that owns one mount namespace.
/// Why:
///  `NS_GET_USERNS` answers a descriptor to the owner,
///  and the owner's inode is what compares with ours.
///
/// # Errors
/// The ioctl refused, or the descriptor it answered could not be described.
pub fn owner_of(namespace: &std::fs::File) -> Result<u64, OwnerRefusal> {
    // SAFETY: the ioctl takes the descriptor this process holds and answers a new one, which the `OwnedFd` below owns.
    let answered: libc::c_int = unsafe { libc::ioctl(namespace.as_raw_fd(), libc::NS_GET_USERNS) };
    if answered < 0 {
        return Err(OwnerRefusal {
            calling: "ioctl NS_GET_USERNS",
            what: String::from("a mount namespace descriptor"),
            errno: std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EINVAL),
        });
    }
    // SAFETY: the ioctl answered a descriptor that nothing else owns.
    let owner: std::fs::File = std::fs::File::from(unsafe { OwnedFd::from_raw_fd(answered) });
    let facts: std::fs::Metadata = owner.metadata().map_err(|error| {
        return OwnerRefusal {
            calling: "stat",
            what: String::from("the owner of a mount namespace"),
            errno: error.raw_os_error().unwrap_or(libc::EINVAL),
        };
    })?;
    return Ok(facts.ino());
}

/// What:
///  The inode of this process's own user namespace.
/// Why:
///  The handler's own namespace is the one every holder's owner is compared
///  with,
///  and reading it from `/proc/self/ns/user` keeps the answer the kernel's.
///
/// # Errors
/// The path could not be described.
pub fn our_user_namespace() -> Result<u64, OwnerRefusal> {
    let facts: std::fs::Metadata = std::fs::metadata(OUR_USER_NAMESPACE).map_err(|error| {
        return OwnerRefusal {
            calling: "stat",
            what: String::from(OUR_USER_NAMESPACE),
            errno: error.raw_os_error().unwrap_or(libc::EINVAL),
        };
    })?;
    return Ok(facts.ino());
}

/// What:
///  Read one mount namespace's identity and its owner's inode,
///  beside this process's own.
/// Why:
///  A control compares the three values without opening anything itself,
///  and the act asks one function whether the holder's copy is locked.
///
/// # Errors
/// The namespace, its owner or this process's own user namespace could not be
/// read.
pub fn read_the_owner(namespace_path: &Path) -> Result<NamespaceOwner, OwnerRefusal> {
    let namespace: std::fs::File = open_a_namespace(namespace_path)?;
    let identity: String = identity_of(&namespace)?;
    let owner: u64 = owner_of(&namespace)?;
    let ours: u64 = our_user_namespace()?;
    return Ok(NamespaceOwner {
        namespace: identity,
        owner,
        ours,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "namespace_owner_tests.rs"]
mod tests;
