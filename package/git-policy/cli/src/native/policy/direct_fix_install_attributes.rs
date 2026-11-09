//! What:
//!  The extended attributes a file carries outside its bytes:
//!  reading them from the file a fix replaces,
//!  giving them to
//!       its replacement,
//!  and the value each one holds.
//! Why:
//!  An extended attribute is data of the person's that no commit holds:
//!  Git stores bytes and a
//!       mode,
//!  so a draft kept in `user.draft`,
//!  or the access-control list that decides who may read a
//!       file,
//!  exists nowhere else.
//!  A replacement that did not carry them would delete the only copy,
//!  which is
//!       the loss the owner forbids;
//!  so a replacement either carries exactly the attributes the file lists or the fix
//!       refuses that file and says which attribute it could not keep.
//!  An attribute the replacement carries and the original never
//!       did is taken off for the same reason read the other way:
//!  the corrected file would otherwise
//!       grant an account access the original never granted.
//!
//! What is out of reach:
//!  a namespace this process may not list (another user's `trusted.*`)
//!  is invisible here,
//!  so an
//!       attribute the kernel does not report cannot be carried or counted;
//!  `SPEC.md` names that limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const attributes = await listxattr(path).map((name) => ({ name, value: await getxattr(path, name) }));
//! // for (const { name, value } of attributes) await fsetxattr(file, name, value);
//! ```

/// `CString` is a NUL-terminated name this code builds.
use std::ffi::CString;
/// `File` is the open replacement the attributes are given to.
use std::fs::File;
/// `AsRawFd` lends the descriptor the system calls take.
use std::os::unix::io::AsRawFd;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// How many times a list or a value is asked for before a changing file is refused.
///  Exempt from a citation under `LQM` (class:
/// poll):
/// it sets how often a condition is looked at again,
/// so it adds at
///  most one retry to a read that already exists.
/// Why:
///   The kernel answers a list that grew between the size question and the answer with `ERANGE`,
///  and a
///       second answer is the usual end of it;
///  a file whose attributes keep changing has no list this
///       process can name,
///  so the reading stops rather than loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ATTRIBUTE_ATTEMPTS = 4;
/// ```
const ATTRIBUTE_ATTEMPTS: u32 = 4;

/// What:
///  One extended attribute:
///  its name as the kernel spells it,
///  and the bytes it holds.
/// Why:
///  A name is a property of the file,
///  so it is carried beside its value and never inferred;
///  the order
///       of two attributes is their names' order,
///  so two files' lists compare as one sequence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Attribute = { name: Uint8Array; value: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Attribute {
    /// The attribute's name,
    ///  as `user.draft` or `system.posix_acl_access`.
    pub(crate) name: Vec<u8>,
    /// The bytes the attribute holds.
    pub(crate) value: Vec<u8>,
}

impl Attribute {
    /// What:
    ///  The attribute's name as text a person reads.
    /// Why:
    ///  A refusal names the attribute it could not keep,
    ///  and an attribute name is almost always text;
    ///  a byte that is not text is shown as a replacement character rather than failing the message.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const displayName = (attribute) => Buffer.from(attribute.name).toString('utf8');
    /// ```
    pub(crate) fn display_name(&self) -> String {
        return String::from_utf8_lossy(self.name.as_slice()).into_owned();
    }
}

/// What:
///  Why a replacement could not be made to carry exactly the attributes the original had.
/// Why:
///  The person must be told which attribute the system would not carry,
///  and which one the
///       replacement carries that the original did not;
///  the two are different harms,
///  one deleting their
///       data and the other granting access the original never granted,
///  so they are different answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeRefusal =
///   | { kind: 'not-given'; attribute: Attribute; reason: Error }
///   | { kind: 'not-removed'; attribute: Attribute; reason: Error }
///   | { kind: 'list-failed'; reason: Error };
/// ```
#[derive(Debug)]
pub(crate) enum AttributeRefusal {
    /// The replacement could not be given an attribute the original carries.
    NotGiven {
        /// The attribute the system refused.
        attribute: Attribute,
        /// Why the system refused.
        reason: std::io::Error,
    },
    /// The replacement carries an attribute the original did not,
    ///  and the system would not take it off.
    NotRemoved {
        /// The attribute that stays on the replacement.
        attribute: Attribute,
        /// Why the system refused.
        reason: std::io::Error,
    },
    /// The attributes of the open replacement could not be listed,
    ///  so what it carries beyond the original's cannot be named.
    ListFailed {
        /// Why the system refused.
        reason: std::io::Error,
    },
}

/// The calls that read the names and the values,
///  and the retry that survives a file changing under them.
#[path = "direct_fix_install_attributes_reading.rs"]
mod reading;

/// What:
///  Whether `name` is a security label:
///  the identity the kernel and its policy give a file,
///  rather than data a person wrote.
/// Why:
///   A container without the label's module answers "operation not supported" for every one of them,
///  and a
///       process may not set or take off a label at all;
///  a correction that refused there would install nothing,
///  and a
///       label is not something a person typed,
///  so a correction carries one when the system allows it and never
///       counts it as a difference.
///  Smack names its labels after the LSM (`security.smack64`,
///  `security.smack64exec`
///       and the rest),
///  so the prefix is what matches them.
///  Every other attribute,
///  `security.capability` and
///       `security.ima` included,
///  decides what the file may do or is data of the person's,
///  and a refusal to
///       carry one still stops the write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isALabel = (name) => name === 'security.selinux' || name === 'security.apparmor' || name.startsWith('security.smack64');
/// ```
pub(crate) fn is_a_label(name: &[u8]) -> bool {
    if name == b"security.selinux" || name == b"security.apparmor" {
        return true;
    }
    return name.starts_with(b"security.smack64");
}

/// Whether an attribute is not a security label,
///  which is what a judgement compares.
/// Why:
///  A label is the identity this system gives a file,
///  so two descriptions that differ only there are the same
///  state of the same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotALabel = (attribute) => !isALabel(attribute.name);
/// ```
fn is_not_a_label(attribute: &&Attribute) -> bool {
    return !is_a_label(attribute.name.as_slice());
}

/// What:
///  Whether two attribute lists carry the same data,
///  apart from the security labels each system gives.
/// Why:
///  A fix carries a label when the system lets it and says so when it cannot,
///  so a replacement that
///       lacks the label the file had is still this fix's own file;
///  comparing labels would keep a journal the fix
///       wrote and would report a correction as the person's save.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameData = (left, right) => left.filter(isNotALabel).equals(right.filter(isNotALabel));
/// ```
pub(crate) fn same_data(left: &[Attribute], right: &[Attribute]) -> bool {
    return left.iter().filter(is_not_a_label).eq(right.iter().filter(is_not_a_label));
}

/// What:
///  Every extended attribute of the regular file at `path`,
///  in name order,
///  read without following a
///       symbolic link.
///  `&str` names the step that failed,
///  for the person's message.
/// Why:
///   A fix replaces a file only when it can carry what the file holds,
///  so the list is read before anything is
///       written,
///  and a value this process may not read refuses the correction instead of dropping a
///       namespace silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributesOf = (path, step) => listxattr(path).sort().map((name) => ({ name, value: getxattr(path, name) }));
/// ```
#[cfg(unix)]
pub(crate) fn attributes_of(path: &Path, step: &str) -> Result<Vec<Attribute>, String> {
    let names: Vec<Vec<u8>> = match reading::names_of_attributes(path) {
        Ok(listed) => listed,
        Err(error) => return Err(unreadable_list(step, &error)),
    };
    let mut found: Vec<Attribute> = Vec::with_capacity(names.len());
    for name in names {
        let Ok(text) = CString::new(name.as_slice()) else {
            return Err(unaddressable_name(step));
        };
        let value: Vec<u8> = match reading::value_of_attribute(path, text.as_c_str()) {
            Ok(read) => read,
            Err(error) => return Err(unreadable_value(step, name.as_slice(), &error)),
        };
        found.push(Attribute { name, value });
    }
    found.sort();
    return Ok(found);
}

/// What:
///  The failure sentence of a list this process could not read.
/// Why:
///   A list that cannot be read means the fix cannot tell what it would delete,
///  so the sentence is a refusal rather than a note.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unreadableList = (step, error) => `${step} failed (${error})`;
/// ```
#[cfg(unix)]
fn unreadable_list(step: &str, error: &std::io::Error) -> String {
    return format!("{step} failed ({error})");
}

/// What:
///  The failure sentence of an attribute name this process cannot address.
/// Why:
///   The system calls take NUL-terminated names,
///  and a name holding a NUL cannot be one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unaddressableName = (step) => `${step} failed (the system named an extended attribute this process cannot address)`;
/// ```
#[cfg(unix)]
fn unaddressable_name(step: &str) -> String {
    return format!(
        "{step} failed (the system named an extended attribute this process cannot address)"
    );
}

/// What:
///  The failure sentence of a value this process could not read,
///  naming the attribute.
/// Why:
///  The person must know which of their attributes is the one the fix cannot carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unreadableValue = (step, name, error) => `${step} failed (its extended attribute ${name} could not be read: ${error})`;
/// ```
#[cfg(unix)]
fn unreadable_value(step: &str, name: &[u8], error: &std::io::Error) -> String {
    return format!(
        "{step} failed (its extended attribute {} could not be read: {error})",
        String::from_utf8_lossy(name)
    );
}

/// What:
///  Every extended attribute of the file at `path`;
///  this system has none.
/// Why:
///   The attributes are data of a Unix file,
///  and a system without them has nothing for a replacement to carry.
/// UNTESTED ON WINDOWS:
///  no gate compiles or runs this branch (the crate does not build for Windows;
///  README,
///  Requirements).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attributesOf = () => [];
/// ```
#[cfg(not(unix))]
pub(crate) fn attributes_of(path: &Path, step: &str) -> Result<Vec<Attribute>, String> {
    let _ = (path, step);
    return Ok(Vec::new());
}

/// What:
///  Give the open replacement at `file` each of `attributes`,
///  and nothing else.
///  `Result<(), AttributeRefusal>` is the attribute the system refused and why,
///  or the
///       attribute the system would not take off.
/// Why:
///   The descriptor cannot be redirected by a swap of the name,
///  so the attributes land on the file that will be
///  swapped in;
///  the first refusal stops the write,
///  because a replacement missing one of the
///       person's attributes must not be installed at all.
///  An attribute already present with the value the person had is set
///       the same way:
///  a system label that the file was created with is then replaced by the label the
///       person's file carried,
///  and a refusal to replace it is a refusal to fix.
///  Everything the replacement carries that `attributes` does not name is then taken
///       off,
///  because a directory's default access-control list is copied onto every file created
///       inside it,
///  and the replacement is created inside the Git directory:
///  a corrected file that kept that
///       list would grant an account access the original never granted (`take_off_every_extra`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const { name, value } of attributes) { try { fsetxattr(file, name, value) } catch (error) { return { kind: 'not-given', attribute: name, error } } }
/// takeOffEveryExtra(file, attributes);
/// ```
#[cfg(unix)]
pub(crate) fn give_to(file: &File, attributes: &[Attribute]) -> Result<(), AttributeRefusal> {
    let descriptor: libc::c_int = file.as_raw_fd();
    for attribute in attributes {
        let Ok(text) = CString::new(attribute.name.as_slice()) else {
            return Err(AttributeRefusal::NotGiven {
                attribute: attribute.clone(),
                reason: std::io::Error::from(std::io::ErrorKind::InvalidInput),
            });
        };
        // SAFETY: the names are NUL-terminated strings, and `value` holds `value.len()` initialised bytes.
        let result: libc::c_int = unsafe {
            libc::fsetxattr(
                descriptor,
                text.as_ptr(),
                attribute.value.as_ptr().cast(),
                attribute.value.len(),
                0,
            )
        };
        if result != 0 {
            let reason: std::io::Error = std::io::Error::last_os_error();
            // What: A security label is the system's identity for the file, so a system that will not let this
            //       process set one gets a warning and the correction goes ahead.
            // Why:  A container without the label's module answers "operation not supported" for every label, and a
            //       correction that refused there would install nothing at all.
            //
            // In TS you'd write (pseudocode):
            // if (isALabel(attribute.name)) { l.warn(labelKeptSentence(attribute, reason)); continue; }
            if is_a_label(attribute.name.as_slice()) {
                continue;
            }
            return Err(AttributeRefusal::NotGiven {
                attribute: attribute.clone(),
                reason,
            });
        }
    }
    return take_off_every_extra(file, attributes);
}

/// What:
///  Whether `kept` names the attribute `name`.
/// Why:
///  A name the original carried is left as it is;
///  a name it never had is one the replacement may not keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesIt = (kept, name) => kept.some((attribute) => attribute.name === name);
/// ```
#[cfg(unix)]
fn names_it(kept: &[Attribute], name: &[u8]) -> bool {
    for attribute in kept {
        if attribute.name.as_slice() == name {
            return true;
        }
    }
    return false;
}

/// What:
///  Take off the open replacement at `file` every extended attribute `kept` does not name.
/// Why:
///  A default access-control list belongs to the directory a file is created in,
///  and the journal
///       directory the replacement is written inside inherits one from the Git directory;
///  the list is
///       then copied onto the replacement,
///  which after the swap decides who may read the person's file.
///  A
///       corrected file that kept it would grant an account the original never named read access,
///  and
///       comparing the displaced original cannot see it,
///  because the unexpected list belongs to the
///       replacement.
///  The list is read from the descriptor,
///  so no other process can move the name
///       between the listing and the removal,
///  and a removal the system refuses stops the write rather than leaving
///       a grant behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of flistxattr(file)) if (!kept.some((attribute) => attribute.name === name)) fremovexattr(file, name);
/// ```
#[cfg(unix)]
fn take_off_every_extra(file: &File, kept: &[Attribute]) -> Result<(), AttributeRefusal> {
    let descriptor: libc::c_int = file.as_raw_fd();
    let names: Vec<Vec<u8>> = match reading::names_on_descriptor(descriptor) {
        Ok(listed) => listed,
        Err(reason) => return Err(AttributeRefusal::ListFailed { reason }),
    };
    for name in names {
        if names_it(kept, name.as_slice()) {
            continue;
        }
        let extra: Attribute = Attribute {
            name: name.clone(),
            value: Vec::new(),
        };
        let Ok(text) = CString::new(name.as_slice()) else {
            return Err(AttributeRefusal::NotRemoved {
                attribute: extra,
                reason: std::io::Error::from(std::io::ErrorKind::InvalidInput),
            });
        };
        // SAFETY: the name is a NUL-terminated string the kernel listed, and the descriptor is this file's.
        let result: libc::c_int = unsafe { libc::fremovexattr(descriptor, text.as_ptr()) };
        if result != 0 {
            let reason: std::io::Error = std::io::Error::last_os_error();
            // What: A label this process may not take off stays, with a warning, and the correction goes ahead.
            // Why:  The label is the identity this system gave the new file rather than data a person wrote, and a
            //       process may not be allowed to remove one at all.
            //
            // In TS you'd write (pseudocode):
            // if (isALabel(extra.name)) { l.warn(labelKeptSentence(extra, reason)); continue; }
            if is_a_label(extra.name.as_slice()) {
                continue;
            }
            return Err(AttributeRefusal::NotRemoved {
                attribute: extra,
                reason,
            });
        }
    }
    return Ok(());
}

/// What:
///  Give the open replacement at `file` each of `attributes`;
///  this system has none to give.
/// Why:
///   An attribute list is empty everywhere outside Unix,
///  and one that is not is a state this system cannot produce.
/// UNTESTED ON WINDOWS:
///  no gate compiles or runs this branch (the crate does not build for Windows;
///  README,
///  Requirements).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const giveTo = (file, attributes) => attributes.length === 0 ? undefined : { kind: 'not-given', attribute: attributes[0], error: unsupported };
/// ```
#[cfg(not(unix))]
pub(crate) fn give_to(file: &File, attributes: &[Attribute]) -> Result<(), AttributeRefusal> {
    let _ = file;
    return match attributes.first() {
        Some(attribute) => Err(AttributeRefusal::NotGiven {
            attribute: attribute.clone(),
            reason: std::io::Error::from(std::io::ErrorKind::Unsupported),
        }),
        None => Ok(()),
    };
}

/// Attribute controls stay out of the release executable.
#[cfg(test)]
#[path = "direct_fix_install_attributes_tests.rs"]
mod tests;
