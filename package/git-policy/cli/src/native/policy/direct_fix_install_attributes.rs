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

/// `CStr` is a NUL-terminated name for the system calls;
///  `CString` is one this code builds.
use std::ffi::{CStr, CString};
/// `File` is the open replacement the attributes are given to.
use std::fs::File;
/// `OsStrExt` lends a path's bytes.
use std::os::unix::ffi::OsStrExt;
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

/// What:
///  A pathname as the system calls want it.
/// Why:
///   They take NUL-terminated bytes,
///  and a pathname holding a NUL byte cannot name a file at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (path) => Buffer.concat([path, Buffer.from([0])]);
/// ```
#[cfg(unix)]
fn name_of(path: &Path) -> Result<CString, std::io::Error> {
    let text: Result<CString, std::ffi::NulError> = CString::new(path.as_os_str().as_bytes());
    return match text {
        Ok(built) => Ok(built),
        Err(_invalid) => Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)),
    };
}

/// Whether `byte` is the zero that separates two attribute names in a list.
/// Why:
///   The list is one buffer of names,
///  each one NUL-terminated,
///  so the separator is what splits them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isZero = (byte) => byte === 0;
/// ```
#[cfg(unix)]
fn is_zero(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether a listed name is one,
///  rather than the empty rest after a final separator.
/// Why:
///   The list ends with a zero,
///  which would otherwise read as an attribute with an empty name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = (name) => name.length > 0;
/// ```
#[cfg(unix)]
fn named(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// What:
///  A system call's answer as a length,
///  or the system's own error.
/// Why:
///  Every call here answers a byte count and a negative number as a failure,
///  so the length question and
///       the reading are read the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answerOf = (count) => count < 0 ? { error: lastError() } : { bytes: count };
/// ```
#[cfg(unix)]
fn answer_of(count: libc::ssize_t) -> std::io::Result<usize> {
    if count < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(usize::try_from(count).unwrap_or(usize::MAX));
}

/// What:
///  One two-call reading:
/// how many bytes the answer needs,
///  then the answer itself.
///  `length` asks,
///  `read` copies at most `buffer.len()` bytes and reports how many the answer needs.
/// Why:
///   The values,
/// the names and the descriptor's list are all read this way,
///  so the retry is written once;
///  the state travels as one argument because the calls are plain functions,
///  which keeps them out of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readUntilItFits(state, length, read) { for (let attempt = 0; attempt < ATTEMPTS; attempt++) { const buffer = Buffer.alloc(length(state)); const reported = read(state, buffer); if (reported > buffer.length) continue; return buffer.subarray(0, reported) } throw new Error('kept changing') }
/// ```
#[cfg(unix)]
fn read_until_it_fits<State>(
    state: &mut State,
    changing: &str,
    length: fn(&mut State) -> std::io::Result<usize>,
    read: fn(&mut State, &mut [u8]) -> std::io::Result<usize>,
) -> std::io::Result<Vec<u8>> {
    for _attempt in 0..ATTRIBUTE_ATTEMPTS {
        let wanted: usize = length(state)?;
        let mut buffer: Vec<u8> = vec![0; wanted];
        let reported: usize = match read(state, buffer.as_mut_slice()) {
            Ok(count) => count,
            Err(error) if error.raw_os_error() == Some(libc::ERANGE) => continue,
            Err(error) => return Err(error),
        };
        // What: The kernel reads a zero-sized buffer as another length question, so an answer above the buffer means
        //       nothing was copied and the value or list has grown since the question was asked.
        // Why:  Truncating the empty buffer to that length cannot grow it, and the person's nonempty value would then be
        //       read as empty, which a displaced file compares equal on.
        //
        // In TS you'd write (pseudocode):
        // if (reported > buffer.length) continue;
        if reported > buffer.len() {
            continue;
        }
        buffer.truncate(reported);
        return Ok(buffer);
    }
    return Err(std::io::Error::other(changing));
}

/// What:
///  How many bytes the list of the file at `path` needs.
/// Why:
///  This is the length question of the two-call reading of a pathname's list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lengthOfList = (path) => llistxattr(path, null, 0);
/// ```
#[cfg(unix)]
fn length_of_list_on_path(path: &mut CString) -> std::io::Result<usize> {
    // SAFETY: a null buffer with size 0 asks the system for the length of the list.
    let length: libc::ssize_t = unsafe { libc::llistxattr(path.as_ptr(), std::ptr::null_mut(), 0) };
    return answer_of(length);
}

/// What:
///  Fill `buffer` with the list of the file at `path`,
///  and report how many bytes the list needs.
/// Why:
///  This is the reading itself;
///  a buffer too small for a list that grew answers `ERANGE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fillList = (path, buffer) => llistxattr(path, buffer, buffer.length);
/// ```
#[cfg(unix)]
fn fill_list_on_path(path: &mut CString, buffer: &mut [u8]) -> std::io::Result<usize> {
    // SAFETY: `buffer` holds `buffer.len()` writable bytes, and the pathname outlives the call.
    let filled: libc::ssize_t =
        unsafe { libc::llistxattr(path.as_ptr(), buffer.as_mut_ptr().cast(), buffer.len()) };
    return answer_of(filled);
}

/// What:
///  How many bytes the list of the open file at `descriptor` needs.
/// Why:
///  This is the length question of the two-call reading of the replacement's own list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lengthOfList = (fd) => flistxattr(fd, null, 0);
/// ```
#[cfg(unix)]
fn length_of_list_on_descriptor(descriptor: &mut libc::c_int) -> std::io::Result<usize> {
    // SAFETY: a null buffer with size 0 asks the system for the length of the list.
    let length: libc::ssize_t = unsafe { libc::flistxattr(*descriptor, std::ptr::null_mut(), 0) };
    return answer_of(length);
}

/// What:
///  Fill `buffer` with the list of the open file at `descriptor`,
///  and report how many bytes the list needs.
/// Why:
///  The descriptor cannot be redirected by a swap of the name,
///  so this is the replacement's own list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fillList = (fd, buffer) => flistxattr(fd, buffer, buffer.length);
/// ```
#[cfg(unix)]
fn fill_list_on_descriptor(descriptor: &mut libc::c_int, buffer: &mut [u8]) -> std::io::Result<usize> {
    // SAFETY: `buffer` holds `buffer.len()` writable bytes, and the descriptor is this file's.
    let filled: libc::ssize_t =
        unsafe { libc::flistxattr(*descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
    return answer_of(filled);
}

/// What:
///  The pathname and the name of one extended attribute,
///  as the two calls that read its bytes want them.
/// Why:
///  One reading takes its state as one value,
///  so the pathname and the attribute's name travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ValueOnDisk = { path: Buffer; name: Buffer };
/// ```
#[cfg(unix)]
struct ValueOnDisk {
    /// The pathname,
    ///  NUL-terminated.
    path: CString,
    /// The attribute's name,
    ///  NUL-terminated.
    name: CString,
}

/// What:
///  How many bytes the value of `asked`'s attribute needs.
/// Why:
///  This is the length question of the two-call reading of one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lengthOfValue = ({ path, name }) => lgetxattr(path, name, null, 0);
/// ```
#[cfg(unix)]
fn length_of_value(asked: &mut ValueOnDisk) -> std::io::Result<usize> {
    // SAFETY: a null buffer with size 0 asks the system for the length of the value.
    let length: libc::ssize_t = unsafe {
        libc::lgetxattr(asked.path.as_ptr(), asked.name.as_ptr(), std::ptr::null_mut(), 0)
    };
    return answer_of(length);
}

/// What:
///  Fill `buffer` with the value of `asked`'s attribute,
///  and report how many bytes it needs.
/// Why:
///  A buffer too small for a value that grew answers `ERANGE`,
///  and a zero-sized buffer makes
///       the call a length question again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fillValue = ({ path, name }, buffer) => lgetxattr(path, name, buffer, buffer.length);
/// ```
#[cfg(unix)]
fn fill_value(asked: &mut ValueOnDisk, buffer: &mut [u8]) -> std::io::Result<usize> {
    // SAFETY: `buffer` holds `buffer.len()` writable bytes, and both names outlive the call.
    let read: libc::ssize_t = unsafe {
        libc::lgetxattr(
            asked.path.as_ptr(),
            asked.name.as_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    return answer_of(read);
}

/// What:
///  The names of every extended attribute of the regular file at `path`,
///  in the order the kernel
///       lists them.
/// Why:
///   The list is what the replacement must carry;
///  a namespace the kernel does not report here is one this
///       process cannot see,
///  and `SPEC.md` records that limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (path) => llistxattr(path).split('\0').filter(Boolean);
/// ```
#[cfg(unix)]
fn names_of_attributes(path: &Path) -> std::io::Result<Vec<Vec<u8>>> {
    let mut text: CString = name_of(path)?;
    let listed: Vec<u8> = read_until_it_fits(
        &mut text,
        "the list of its extended attributes kept changing while it was read",
        length_of_list_on_path,
        fill_list_on_path,
    )?;
    return Ok(listed
        .split(is_zero)
        .filter(named)
        .map(<[u8]>::to_vec)
        .collect());
}

/// What:
///  The names of every extended attribute of the open file at `descriptor`,
///  in the order the kernel
///       lists them.
/// Why:
///  The descriptor cannot be redirected by a swap of the name,
///  so the list is the replacement's own,
///  and the names are what a removal addresses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (fd) => flistxattr(fd).split('\0').filter(Boolean);
/// ```
#[cfg(unix)]
fn names_on_descriptor(descriptor: libc::c_int) -> std::io::Result<Vec<Vec<u8>>> {
    let mut asked: libc::c_int = descriptor;
    let listed: Vec<u8> = read_until_it_fits(
        &mut asked,
        "the list of its extended attributes kept changing while it was read",
        length_of_list_on_descriptor,
        fill_list_on_descriptor,
    )?;
    return Ok(listed
        .split(is_zero)
        .filter(named)
        .map(<[u8]>::to_vec)
        .collect());
}

/// What:
///  The bytes the extended attribute `attribute` holds on the file at `path`.
/// Why:
///   The value is the person's data,
///  and a name whose value cannot be read is a refusal rather than an
///       attribute left behind on the replacement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = (path, attribute) => lgetxattr(path, attribute);
/// ```
#[cfg(unix)]
fn value_of_attribute(path: &Path, name: &CStr) -> std::io::Result<Vec<u8>> {
    let mut asked: ValueOnDisk = ValueOnDisk {
        path: name_of(path)?,
        name: CString::from(name),
    };
    return read_until_it_fits(
        &mut asked,
        "its value kept changing while it was read",
        length_of_value,
        fill_value,
    );
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
    let names: Vec<Vec<u8>> = match names_of_attributes(path) {
        Ok(listed) => listed,
        Err(error) => return Err(unreadable_list(step, &error)),
    };
    let mut found: Vec<Attribute> = Vec::with_capacity(names.len());
    for name in names {
        let Ok(text) = CString::new(name.as_slice()) else {
            return Err(unaddressable_name(step));
        };
        let value: Vec<u8> = match value_of_attribute(path, text.as_c_str()) {
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
            return Err(AttributeRefusal::NotGiven {
                attribute: attribute.clone(),
                reason: std::io::Error::last_os_error(),
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
    let names: Vec<Vec<u8>> = match names_on_descriptor(descriptor) {
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
            return Err(AttributeRefusal::NotRemoved {
                attribute: extra,
                reason: std::io::Error::last_os_error(),
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
