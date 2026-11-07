//! What:
//! The identity of this boot and this machine,
//! read from the kernel's own files:
//! the value that tells a journal's record of a mount view whether the machine was restarted.
//! Why:
//! Split out of `control_group.rs`,
//! which holds the containment of a run in a control group,
//! to keep that file under the code-line limit.
//! The items are re-exported from `control_group`,
//! so their paths there are unchanged.

/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::{ExplainedError};

/// Imports `std::fs` for this file.
use std::fs;


/// Imports `std::io::{Read, Write}` for this file.
use std::io::{Read};

/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;


/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path};







/// Imports `rustix::fs::{AtFlags, CWD, StatxFlags, statx}` for this file.
use rustix::fs::{AtFlags, StatxFlags, statx};



/// Who ran:
/// the machine and the boot of the kernel that held the processes of a run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Identity {
    /// `/proc/sys/kernel/random/boot_id`:
    /// a new value for every boot of a kernel.
    pub(crate) boot: String,
    /// `/etc/machine-id`:
    /// the same value on every boot of one installation.
    pub(crate) machine: String,
}

#[cfg(test)]
thread_local! {
    /// The identity this thread reads instead of the machine's,
    /// so a test can be a recovery after a reboot or on another machine.
    static IDENTITY_OVERRIDE: std::cell::RefCell<Option<Result<Identity, String>>> = const { std::cell::RefCell::new(None) };
}

/// What:
/// `pub(crate) fn use_identity_for_test(identity: Option<Result<Identity, String>>)` makes this thread read `identity` as its own.
/// Why:
/// A recovery after a reboot,
/// on another machine,
/// or with an unreadable identity is made without rebooting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function useIdentityForTest(identity?: Identity | Error): void
/// ```
#[cfg(test)]
pub(crate) fn use_identity_for_test(identity: Option<Result<Identity, String>>) {
    IDENTITY_OVERRIDE.with(|slot| { *slot.borrow_mut() = identity; });
}

/// What:
/// `fn is_hex_id(text: &str, groups: &[usize]) -> bool` says whether `text` is lowercase hexadecimal in groups of exactly the given
/// lengths,
/// separated by single hyphens,
/// and not all zero.
/// Why:
/// The kernel writes the boot id as a lowercase `8-4-4-4-12` UUID,
/// and the machine id is 32 lowercase hexadecimal digits
/// (the same formats systemd reads:
/// `ID128_FORMAT_UUID` and plain);
/// anything else is not an identity,
/// and a text that is not one cannot show that a boot or a machine differs (round 10,
/// 9a D2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isHexId(text: string, groups: number[]): boolean
/// ```
pub(crate) fn is_hex_id(text: &str, groups: &[usize]) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != groups.len() {
        return false;
    }
    let mut shaped: bool = true;
    for (part, length) in parts.iter().zip(groups.iter()) {
        if !(part.len() == *length && part.bytes().all(is_lower_hex_digit)) {
            shaped = false;
            break;
        }
    }
    return shaped && text.bytes().any(is_not_zero_or_dash);
}

/// What:
/// `fn is_lower_hex_digit(byte: u8) -> bool` says whether a byte is `0` to `9` or `a` to `f`.
/// Why:
/// `bytes().all` takes it by path instead of a closure.
pub(crate) fn is_lower_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
}

/// What:
/// `fn is_not_zero_or_dash(byte: u8) -> bool` says whether a byte is neither `0` nor `-`.
/// Why:
/// An identity of only zeros and dashes names nothing,
/// and `bytes().any` takes this by path instead of a closure.
pub(crate) fn is_not_zero_or_dash(byte: u8) -> bool {
    return byte != b'0' && byte != b'-';
}

/// The group lengths of the boot id (`8-4-4-4-12`).
pub(crate) const BOOT_ID_GROUPS: [usize; 5] = [8, 4, 4, 4, 12];

/// The group length of the machine id (one run of 32 digits).
pub(crate) const MACHINE_ID_GROUPS: [usize; 1] = [32];

/// The kernel file that holds the boot id.
pub(crate) const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

/// The file system type number of procfs (`PROC_SUPER_MAGIC`),
/// from `statfs(2)`.
pub(crate) const PROC_MAGIC: u64 = 0x9fa0;

/// The machine id file.
pub(crate) const MACHINE_ID_PATH: &str = "/etc/machine-id";

/// The methods of the type above.
impl Identity {
    /// What:
    /// `pub(crate) fn is_well_formed(&self) -> bool` says whether both ids have the format of a boot id and a machine id.
    /// Why:
    /// A malformed id differs from every real one and would read as a reboot or as another machine (9a D2);
    /// the check applies to the recorded and to the current identity alike.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isWellFormed(): boolean
    /// ```
    pub(crate) fn is_well_formed(&self) -> bool {
        return is_hex_id(&self.boot, &BOOT_ID_GROUPS) && is_hex_id(&self.machine, &MACHINE_ID_GROUPS);
    }
}

/// What:
/// `fn is_view_line(line: &str) -> bool` says whether a line is exactly `view <mount>:<inode>` with two decimal numbers.
/// Why:
/// The header of the record is part of the format the identity is read from (round 12,
/// 11a);
/// a leading sign or space is not a number
/// of the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isViewLine(line: string): boolean
/// ```
pub(crate) fn is_view_line(line: &str) -> bool {
    let Some((mount_text, inode_text)) = line.strip_prefix("view ").and_then(split_at_colon) else {
        return false;
    };
    return is_decimal_number(mount_text) && is_decimal_number(inode_text);
}

/// What:
/// `fn split_at_colon(rest: &str) -> Option<(&str, &str)>` splits text at its first colon.
/// Why:
/// `Option::and_then` takes it by path instead of a closure.
pub(crate) fn split_at_colon(rest: &str) -> Option<(&str, &str)> {
    return rest.split_once(':');
}

/// What:
/// `fn is_decimal_number(number: &str) -> bool` says whether text is a non-empty run of digits that fits a `u64`.
/// Why:
/// The two numbers of a view line are checked alike.
pub(crate) fn is_decimal_number(number: &str) -> bool {
    return !number.is_empty() && number.bytes().all(is_digit_byte) && number.parse::<u64>().is_ok();
}

/// What:
/// `fn is_digit_byte(byte: u8) -> bool` says whether a byte is `0` to `9`.
/// Why:
/// `bytes().all` takes it by path instead of a closure.
pub(crate) fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
/// `pub(crate) fn read_identity_text(text: &str) -> Option<Identity>` reads the `boot` and `machine` lines of a journal's view record.
/// Why:
/// A missing,
/// repeated,
/// unknown or malformed identity is uncertainty,
/// never a match and never a different boot (round 10,
/// 9a D2):
/// the record is the first line `view <mount>:<inode>`,
/// then exactly one `boot <id>` and one `machine <id>`,
/// each line exactly in
/// that form with a well-formed id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readIdentityText(text: string): Identity | undefined
/// ```
pub(crate) fn read_identity_text(text: &str) -> Option<Identity> {
    let mut boot_text: Option<String> = None;
    let mut machine_text: Option<String> = None;
    let mut lines = text.lines();
    if !is_view_line(lines.next()?) {
        return None;
    }
    for line in lines {
        if let Some(found) = line.strip_prefix("boot ") {
            if boot_text.replace(found.to_string()).is_some() {
                return None;
            }
        } else {
            // Any line that is neither a boot nor a machine line makes the record unusable.
            let found: &str = line.strip_prefix("machine ")?;
            if machine_text.replace(found.to_string()).is_some() {
                return None;
            }
        }
    }
    let identity: Identity = Identity { boot: boot_text?, machine: machine_text? };
    return identity.is_well_formed().then_some(identity);
}

/// What:
/// `pub(crate) fn read_authoritative_boot_id(path: &Path) -> Result<String, String>` opens the boot id file once and returns its text
/// only if the opened object is the kernel's own file and not one put over it.
/// Why:
/// A bind mount can replace `/proc/sys/kernel/random/boot_id` without a reboot (measured in a disposable user and mount namespace on
/// 2026-11-04T08:36:57-05:00),
/// and `systemd-nspawn` does it on purpose to give each container its own id
/// (`src/nspawn/nspawn.c`,
/// `setup_boot_id`).
/// The kernel serves the file from procfs;
/// a file of another file system,
/// or a file that is itself the root of a mount
/// (`STATX_ATTR_MOUNT_ROOT`),
/// is a replacement.
/// A kernel that does not say whether the file is a mount root is an error,
/// not a pass (round 10,
/// 9a section 4).
/// Round 12 (11a):
/// the file is opened without following a final symbolic link (a link to a genuine procfs file is a replacement that
/// the checks of the target would pass),
/// and the checks and the read are made on the one descriptor,
/// so a mount that appears after
/// the checks cannot change what is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readAuthoritativeBootId(path: string): string  // throws the reason
/// ```
pub(crate) fn read_authoritative_boot_id(path: &Path) -> Result<String, String> {
    let opened: OwnedFd = rustix::fs::open(path, rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
        .explained(format_args!("{} cannot be opened without following a link", path.display()))?;
    let found = rustix::fs::fstatfs(&opened).explained(format_args!("{} cannot be examined", path.display()))?;
    if u64::try_from(found.f_type).unwrap_or(0) != PROC_MAGIC {
        return Err(format!("{} is not served by procfs, so it may be a file put over the kernel's", path.display()));
    }
    let attributes = statx(&opened, "", AtFlags::EMPTY_PATH, StatxFlags::TYPE).explained(format_args!("{} cannot be examined", path.display()))?;
    if !attributes.stx_attributes_mask.contains(rustix::fs::StatxAttributes::MOUNT_ROOT) {
        return Err("the kernel does not say whether the boot id file is the root of a mount".to_string());
    }
    if attributes.stx_attributes.contains(rustix::fs::StatxAttributes::MOUNT_ROOT) {
        return Err(format!("{} is the root of a mount of its own, so it was put over the kernel's file", path.display()));
    }
    let mut text: String = String::new();
    fs::File::from(opened).read_to_string(&mut text).explained(format_args!("the boot id cannot be read"))?;
    return Ok(text);
}

/// What:
/// `fn single_line(text: String) -> String` drops one trailing newline.
/// Why:
/// The boot id and the machine id files end with a newline that is not part of the identity.
pub(crate) fn single_line(text: String) -> String {
    return text.strip_suffix('\n').unwrap_or(&text).to_string();
}

/// What:
/// `pub(crate) fn read_current_identity(boot_path: &Path, machine_path: &Path) -> Result<Identity, String>` reads and validates the
/// boot id and the machine id from the given files.
/// Why:
/// The boot file must pass the authority check (made on the descriptor that is read) and both ids must have their format;
/// the paths are parameters so a test can show
/// each refusal on a file it controls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readCurrentIdentity(bootPath: string, machinePath: string): Identity  // throws the reason
/// ```
pub(crate) fn read_current_identity(boot_path: &Path, machine_path: &Path) -> Result<Identity, String> {
    let boot: String = single_line(read_authoritative_boot_id(boot_path)?);
    let machine: String = single_line(fs::read_to_string(machine_path).explained(format_args!("the machine id cannot be read"))?);
    let identity: Identity = Identity { boot, machine };
    if !identity.is_well_formed() {
        return Err("the boot id or the machine id is empty or not in the format of an id".to_string());
    }
    return Ok(identity);
}

/// What:
/// `pub(crate) fn current_identity() -> Result<Identity, String>` reads the boot id and the machine id of this system.
/// Why:
/// Both are recorded with a journal,
/// so a recovery can tell a reboot of the same machine from another machine.
/// Either unreadable,
/// malformed,
/// or not the kernel's own (`read_authoritative_boot_id`) is an error,
/// and the caller keeps the refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function currentIdentity(): Identity  // throws the reason
/// ```
pub(crate) fn current_identity() -> Result<Identity, String> {
    #[cfg(test)]
    if let Some(chosen) = IDENTITY_OVERRIDE.with(|slot| { return slot.borrow().clone(); }) {
        return chosen;
    }
    return read_current_identity(Path::new(BOOT_ID_PATH), Path::new(MACHINE_ID_PATH));
}
