//! What:
//!  Directory-relative file operations that never follow a symbolic link or cross a mount:
//! open a directory,
//! a file or a
//!  link below a held directory handle one plain name at a time (`openat2` with `RESOLVE_BENEATH`,
//! `RESOLVE_NO_SYMLINKS`,
//!  `RESOLVE_NO_XDEV` and `RESOLVE_NO_MAGICLINKS`),
//! create entries there,
//! set times on them,
//! rename them,
//! publish an open
//!  file under a new name without replacing anything,
//! remove a lock file whose identity is the one that was recorded,
//! and take an entry under a name of its own so that only the object whose identity was recorded is removed.
//! Why:
//!  A path string is resolved again by every call that takes it,
//! so a directory swapped for a link between two calls
//!  redirects the second one.
//! A name that was checked may also name another object when it is unlinked,
//! and nothing in the
//!  kernel makes the two one step,
//! so no tree is deleted through this module:
//! what the ladder made is kept and reported.
//!  The clone rung and the snapshot's stub removal work on trees other processes can rename into;
//! holding the handle of each
//!  directory and naming one component at a time keeps every call on the object that was looked at.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const root = await openRoot('/canonical/path'); // throws when an ancestor is a link
//! const dir = await openDirectoryAt(root, 'a'); // throws on a link, a mount, '.' or '..'
//! await removeOwnedAt(dir, 'file', recordedIdentity); // keeps a replacement
//! ```

use std::ffi::{CString, OsStr, OsString};
use std::fs::File;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

/// `RESOLVE_NO_XDEV`:
/// no mount point,
/// bind mounts included,
/// is crossed.
const RESOLVE_NO_XDEV: u64 = 0x01;
/// `RESOLVE_NO_MAGICLINKS`:
/// no `/proc` style link is followed.
const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
/// `RESOLVE_NO_SYMLINKS`:
/// no symbolic link is followed,
/// the last component included.
const RESOLVE_NO_SYMLINKS: u64 = 0x04;
/// `RESOLVE_BENEATH`:
/// the lookup stays below the directory it starts from.
const RESOLVE_BENEATH: u64 = 0x08;

/// The arguments of `openat2`,
/// laid out as the kernel's `struct open_how`.
#[repr(C)]
struct OpenHow {
    /// The `open` flags.
    flags: u64,
    /// The mode of a created file.
    mode: u64,
    /// The `RESOLVE_*` bits.
    resolve: u64,
}

/// The identity of a filesystem object:
/// device,
/// inode and,
/// where the filesystem gives one,
/// the birth time.
///
/// Why:
/// a path or a name proves nothing about which object it names later;
/// this does,
/// up to the reuse of an inode number
/// within one birth-time tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectId {
    /// The device number.
    pub device: u64,
    /// The inode number.
    pub inode: u64,
    /// The birth time as seconds and nanoseconds,
    /// when the filesystem reports it.
    pub birth: Option<(i64, u32)>,
}

impl ObjectId {
    /// The identity as `device:inode:seconds:nanoseconds` (`-` for an unknown birth time).
    pub fn encode(&self) -> String {
        return match self.birth {
            Some((seconds, nanos)) => format!("{}:{}:{seconds}:{nanos}", self.device, self.inode),
            None => format!("{}:{}:-:-", self.device, self.inode),
        };
    }

    /// The identity from its encoding.
    pub fn decode(text: &str) -> Option<ObjectId> {
        let mut parts = text.split(':');
        let device: u64 = parts.next()?.parse().ok()?;
        let inode: u64 = parts.next()?.parse().ok()?;
        let seconds: &str = parts.next()?;
        let nanos: &str = parts.next()?;
        let birth: Option<(i64, u32)> = if seconds == "-" { None } else { Some((seconds.parse().ok()?, nanos.parse().ok()?)) };
        return if parts.next().is_none() { Some(ObjectId { device, inode, birth }) } else { None };
    }
}

/// The components of a relative path,
/// each refused unless it is a plain name.
///
/// Why:
/// `..`,
/// `.`,
/// a root or a prefix would leave the held directory or name it twice.
pub fn plain_components(relative: &Path) -> std::io::Result<Vec<OsString>> {
    let mut names: Vec<OsString> = Vec::new();
    for part in relative.components() {
        match part {
            Component::Normal(name) => names.push(name.to_os_string()),
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} is not a plain relative path", relative.display()))),
        }
    }
    return Ok(names);
}

/// A NUL-terminated copy of `bytes`,
/// or an invalid-input error saying `what` holds a NUL byte.
fn c_string_of(bytes: &[u8], what: &str) -> std::io::Result<CString> {
    return match CString::new(bytes) {
        Ok(text) => Ok(text),
        Err(_error) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{what} holds a NUL byte"))),
    };
}

/// A NUL-terminated copy of one name,
/// refused when it is empty,
/// `.`,
/// `..`,
/// holds a slash or a NUL byte.
fn name_of(name: &OsStr) -> std::io::Result<CString> {
    let bytes: &[u8] = name.as_bytes();
    if bytes.is_empty() || bytes == b"." || bytes == b".." || bytes.contains(&b'/') {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "a name must be one plain component"));
    }
    return c_string_of(bytes, "a name");
}

/// Turn a C result that is `-1` on failure into the thread's error.
fn checked(result: libc::c_int) -> std::io::Result<libc::c_int> {
    return if result < 0 { Err(std::io::Error::last_os_error()) } else { Ok(result) };
}

/// `openat2` of one plain name below `dir`,
/// resolving nothing but the name itself.
fn open_beneath(dir: &File, name: &OsStr, flags: libc::c_int, mode: u32) -> std::io::Result<File> {
    let c_name = name_of(name)?;
    let how: OpenHow = OpenHow {
        flags: (flags | libc::O_CLOEXEC) as u64,
        mode: u64::from(mode),
        resolve: RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_XDEV | RESOLVE_NO_MAGICLINKS,
    };
    // SAFETY: `c_name` is a NUL-terminated string and `how` a live `open_how` of the size passed, both outliving the call;
    // `dir` is an open descriptor.
    let result: libc::c_long = unsafe { libc::syscall(libc::SYS_openat2, dir.as_raw_fd(), c_name.as_ptr(), &how as *const OpenHow, std::mem::size_of::<OpenHow>()) };
    if result < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat2` returned a new descriptor that nothing else owns.
    return Ok(File::from(unsafe { OwnedFd::from_raw_fd(result as libc::c_int) }));
}

/// Open the directory at the canonical absolute path `path` as the root of a walk.
///
/// Why:
/// the ancestors of a path are resolved by the kernel and may be links,
/// so the open directory's own path,
/// read back
/// from the descriptor,
/// must equal the path asked for;
/// a path with a link among its ancestors is refused.
pub fn open_root(path: &Path) -> std::io::Result<File> {
    if !path.is_absolute() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} is not absolute", path.display())));
    }
    let name = c_string_of(path.as_os_str().as_bytes(), "a path")?;
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let descriptor: libc::c_int = checked(unsafe { libc::open(name.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) })?;
    // SAFETY: `open` returned a new descriptor that nothing else owns.
    let held: File = File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
    let real: PathBuf = std::fs::read_link(handle_path(&held))?;
    if real.as_os_str() != path.as_os_str() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} is reached through a link: it is {}", path.display(), real.display())));
    }
    return Ok(held);
}

/// Open the subdirectory `name` of `parent`,
/// refusing a link,
/// a mount point and `.` or `..`.
pub fn open_directory_at(parent: &File, name: &OsStr) -> std::io::Result<File> {
    return open_beneath(parent, name, libc::O_RDONLY | libc::O_DIRECTORY, 0);
}

/// Open the directory at `names` below `root`,
/// one component at a time,
/// refusing a link,
/// a mount point or a directory on
///  another device (a subvolume of a btrfs filesystem has its own).
pub fn open_directory_beneath(root: &File, names: &[OsString]) -> std::io::Result<File> {
    let device: u64 = device_of(root)?;
    let mut current: File = root.try_clone()?;
    for name in names {
        let next: File = open_directory_at(&current, name.as_os_str())?;
        if device_of(&next)? != device {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} is on another device", name.to_string_lossy())));
        }
        current = next;
    }
    return Ok(current);
}

/// The device number of an open file.
pub fn device_of(file: &File) -> std::io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    return Ok(file.metadata()?.dev());
}

/// Open the regular file `name` of `dir` for reading,
/// refusing a link;
/// a fifo or a device is refused later by its type.
pub fn open_file_at(dir: &File, name: &OsStr) -> std::io::Result<File> {
    return open_beneath(dir, name, libc::O_RDONLY | libc::O_NONBLOCK, 0);
}

/// Create the file `name` in `dir` with `mode`,
/// failing when anything is there,
/// a link included.
pub fn create_file_at(dir: &File, name: &OsStr, mode: u32) -> std::io::Result<File> {
    return open_beneath(dir, name, libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL, mode);
}

/// Create the directory `name` in `dir` with `mode`;
/// `Ok(true)` when it was made,
/// `Ok(false)` when a directory was
///  already there (a link or a file in the way is an error).
pub fn make_directory_at(dir: &File, name: &OsStr, mode: u32) -> std::io::Result<bool> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call and `dir` is an open descriptor.
    let result: libc::c_int = unsafe { libc::mkdirat(dir.as_raw_fd(), c_name.as_ptr(), mode as libc::mode_t) };
    if result == 0 {
        return Ok(true);
    }
    let error: std::io::Error = std::io::Error::last_os_error();
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        drop(open_directory_at(dir, name)?);
        return Ok(false);
    }
    return Err(error);
}

/// Create the link `name` in `dir` pointing at `target`.
pub fn symlink_at(target: &Path, dir: &File, name: &OsStr) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    let c_target = c_string_of(target.as_os_str().as_bytes(), "a link target")?;
    // SAFETY: both strings outlive the call and `dir` is an open descriptor.
    checked(unsafe { libc::symlinkat(c_target.as_ptr(), dir.as_raw_fd(), c_name.as_ptr()) })?;
    return Ok(());
}

/// The target of the link `name` in `dir`.
pub fn read_link_at(dir: &File, name: &OsStr) -> std::io::Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let c_name = name_of(name)?;
    let mut buffer: Vec<u8> = vec![0; 4096];
    // SAFETY: `c_name` outlives the call, `buffer` has the length passed, and `dir` is an open descriptor.
    let length: libc::ssize_t = unsafe { libc::readlinkat(dir.as_raw_fd(), c_name.as_ptr(), buffer.as_mut_ptr().cast::<libc::c_char>(), buffer.len()) };
    if length < 0 {
        return Err(std::io::Error::last_os_error());
    }
    buffer.truncate(usize::try_from(length).unwrap_or(0));
    return Ok(PathBuf::from(OsString::from_vec(buffer)));
}

/// The path through which a held directory handle names that directory,
/// for tools that take a path.
pub fn handle_path(directory: &File) -> PathBuf {
    return Path::new("/proc/self/fd").join(directory.as_raw_fd().to_string());
}

/// The metadata of the entry `name` in `dir`,
/// without following a link.
pub fn entry_metadata_at(dir: &File, name: &OsStr) -> std::io::Result<std::fs::Metadata> {
    // The name is one plain component, so the lookup below the descriptor's magic link resolves nothing else.
    name_of(name)?;
    return std::fs::symlink_metadata(handle_path(dir).join(name));
}

/// Set the access and modification times of the entry `name` in `dir`,
/// without following a link.
pub fn set_times_at(dir: &File, name: &OsStr, times: [libc::timespec; 2]) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call, `times` holds the two timestamps the call reads, and `dir` is an open descriptor.
    checked(unsafe { libc::utimensat(dir.as_raw_fd(), c_name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) })?;
    return Ok(());
}

/// Remove the entry `name` from `dir`:
/// a directory when `directory` is set (it must be empty),
/// else a file or a link.
///
/// Why:
/// this removes whatever the name holds now;
/// use `remove_owned_at` for an object that was made earlier.
pub fn remove_at(dir: &File, name: &OsStr, directory: bool) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    let flags: libc::c_int = if directory { libc::AT_REMOVEDIR } else { 0 };
    // SAFETY: `c_name` outlives the call and `dir` is an open descriptor.
    checked(unsafe { libc::unlinkat(dir.as_raw_fd(), c_name.as_ptr(), flags) })?;
    return Ok(());
}

/// Rename `from` of `from_dir` to `to` of `to_dir`;
/// with `replace` false an existing `to` is an error (`RENAME_NOREPLACE`).
pub fn rename_at(from_dir: &File, from: &OsStr, to_dir: &File, to: &OsStr, replace: bool) -> std::io::Result<()> {
    let c_from = name_of(from)?;
    let c_to = name_of(to)?;
    let flags: libc::c_uint = if replace { 0 } else { libc::RENAME_NOREPLACE };
    // SAFETY: both names outlive the call and both directories are open descriptors.
    checked(unsafe { libc::renameat2(from_dir.as_raw_fd(), c_from.as_ptr(), to_dir.as_raw_fd(), c_to.as_ptr(), flags) })?;
    return Ok(());
}

/// Exchange the entries `first` and `second` of `dir` atomically (`RENAME_EXCHANGE`).
pub fn exchange_at(dir: &File, first: &OsStr, second: &OsStr) -> std::io::Result<()> {
    let c_first = name_of(first)?;
    let c_second = name_of(second)?;
    // SAFETY: both names outlive the call and `dir` is an open descriptor.
    checked(unsafe { libc::renameat2(dir.as_raw_fd(), c_first.as_ptr(), dir.as_raw_fd(), c_second.as_ptr(), libc::RENAME_EXCHANGE) })?;
    return Ok(());
}

/// Build an identity from the fields of `statx`.
fn id_of_statx(found: &libc::statx) -> ObjectId {
    let birth: Option<(i64, u32)> = if found.stx_mask & libc::STATX_BTIME != 0 { Some((found.stx_btime.tv_sec, found.stx_btime.tv_nsec)) } else { None };
    return ObjectId { device: libc::makedev(found.stx_dev_major, found.stx_dev_minor), inode: found.stx_ino, birth };
}

/// The identity of the entry `name` in `dir`,
/// without following a link (`name` empty means `dir` itself).
fn identity_statx(dir: &File, name: Option<&OsStr>) -> std::io::Result<ObjectId> {
    let (c_name, flags): (CString, libc::c_int) = match name {
        Some(plain) => (name_of(plain)?, libc::AT_SYMLINK_NOFOLLOW),
        None => (CString::default(), libc::AT_EMPTY_PATH),
    };
    // SAFETY: an all-zero `statx` is a valid value for the call to fill.
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: `c_name` outlives the call, `found` is a live `statx`, and `dir` is an open descriptor.
    checked(unsafe { libc::statx(dir.as_raw_fd(), c_name.as_ptr(), flags, libc::STATX_BASIC_STATS | libc::STATX_BTIME, &mut found) })?;
    return Ok(id_of_statx(&found));
}

/// The identity of an open file or directory.
pub fn identity_of(file: &File) -> std::io::Result<ObjectId> {
    return identity_statx(file, None);
}

/// The identity of the entry `name` in `dir`,
/// a link not followed.
pub fn identity_at(dir: &File, name: &OsStr) -> std::io::Result<ObjectId> {
    return identity_statx(dir, Some(name));
}

/// What a request to remove an entry that the caller made did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Owned {
    /// The entry was removed.
    Removed,
    /// Nothing was at the name.
    Absent,
    /// The name held another object and was left alone,
    /// for this reason.
    Kept(String),
}

/// What:
///  Remove the entry `name` of `dir` if,
/// at the moment of the look,
/// it is the object with identity `expected`.
/// Why:
///   This is the one removal by name the ladder keeps,
/// for the lock file it created and still holds open:
/// it is what Git
///  itself does with its own lock.
/// A name can be taken between the look and the unlink,
/// and nothing in the kernel makes the
///  two one step,
/// so the function is used only for a lock that no other writer can have replaced without breaking the lock
///  protocol,
/// and never to remove a tree:
/// a directory is refused.
/// Everything else the ladder made stays and is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeIfIdentical(dir, 'index.lock', expected);
/// ```
pub fn remove_if_identical(dir: &File, name: &OsStr, expected: ObjectId) -> std::io::Result<Owned> {
    let found: ObjectId = match identity_at(dir, name) {
        Ok(known) => known,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Owned::Absent),
        Err(error) => return Err(error),
    };
    if found != expected {
        return Ok(Owned::Kept(format!("{} is not the object that was made ({} against {})", name.to_string_lossy(), found.encode(), expected.encode())));
    }
    if entry_metadata_at(dir, name)?.is_dir() {
        return Ok(Owned::Kept(format!("{} is a directory and is not removed", name.to_string_lossy())));
    }
    return match remove_at(dir, name, false) {
        Ok(()) => Ok(Owned::Removed),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Owned::Absent),
        Err(error) => Err(error),
    };
}

#[cfg(test)]
thread_local! {
    /// A control may put another file at the name on this thread while the entry that was taken is being decided,
    /// which is where a process that renames onto the name can slip in.
    pub(crate) static WHILE_THE_ENTRY_IS_DECIDED: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Remove the entry `name` of `dir` only while it is the object with identity `expected`,
///  by taking the name with one rename and removing only what that rename carried,
///  and answer with what happened to it.
/// Why:
///   `remove_if_identical` looks at the name and unlinks the name,
/// and nothing in the kernel makes those one step,
/// so a file another process moves onto the name between them is unlinked without ever being read,
/// which is another process's bytes gone and a work item reported done.
/// Taking the name first is one step that decides which object the removal is about:
/// an object that is not `expected` is put back under its own name,
/// and a file moved onto the name afterwards stays there,
/// while the entry the removal was given is the one it unlinks.
///   This is never how a lock is released:
/// the name of a lock must not be absent while its holder works,
/// or a second writer takes it,
/// so a lock keeps `remove_if_identical` and its own protocol.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await takeThenRemoveIfIdentical(dir, '.manifest', '.manifest.removing', expected);
/// ```
pub fn take_then_remove_if_identical(dir: &File, name: &OsStr, taken: &OsStr, expected: ObjectId) -> std::io::Result<Owned> {
    match rename_at(dir, name, dir, taken, false) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Owned::Absent),
        // Nothing is touched when the name the entry is taken to is already taken: the entry the caller named keeps its
        //  bytes and its name, and the reason says which name was in the way.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Ok(Owned::Kept(format!("the name it is taken to, {}, already holds an entry", taken.to_string_lossy())));
        }
        Err(error) => return Err(error),
    }
    #[cfg(test)]
    WHILE_THE_ENTRY_IS_DECIDED.with(|hook| {
        if let Some(intervene) = hook.borrow().as_ref() {
            intervene();
        }
    });
    let found: ObjectId = match identity_at(dir, taken) {
        Ok(known) => known,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Owned::Kept(format!("{} was removed while it was examined", name.to_string_lossy()))),
        Err(error) => return Err(error),
    };
    let refusal: Option<String> = match entry_metadata_at(dir, taken) {
        Ok(examined) => {
            if found != expected {
                Some(format!("{} is not the object that was made ({} against {})", name.to_string_lossy(), found.encode(), expected.encode()))
            } else if examined.is_dir() {
                Some(format!("{} is a directory and is not removed", name.to_string_lossy()))
            } else {
                None
            }
        }
        Err(error) => Some(format!("{} cannot be examined ({error})", name.to_string_lossy())),
    };
    if let Some(reason) = refusal {
        // What was taken goes back under its own name, so a refusal leaves the entry exactly where it was; one that cannot
        //  be put back stays at the taken name, which the reason says, and is never deleted.
        return Ok(match rename_at(dir, taken, dir, name, false) {
            Ok(()) => Owned::Kept(reason),
            Err(error) => Owned::Kept(format!("{reason}, and it stays at {} because it cannot be put back ({error})", taken.to_string_lossy())),
        });
    }
    return match remove_at(dir, taken, false) {
        Ok(()) => Ok(Owned::Removed),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Owned::Absent),
        Err(error) => Err(error),
    };
}

/// What:
///  Make the open file `source` another name `name` of `dir`,
/// failing if the name exists.
/// Why:
///   The object published is the one the descriptor holds,
/// not whatever a separately checked name refers to at the moment of a
///  rename,
/// and an existing entry is never replaced (`linkat` fails with `EEXIST`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkOpenFileAt(file, dir, 'index');
/// ```
pub fn link_open_file_at(source: &File, dir: &File, name: &OsStr) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    let through = c_string_of(handle_path(source).as_os_str().as_bytes(), "a path")?;
    // SAFETY: both strings outlive the call, `dir` is an open descriptor, and `AT_SYMLINK_FOLLOW` makes the link to the file
    // the descriptor's magic link names.
    checked(unsafe { libc::linkat(libc::AT_FDCWD, through.as_ptr(), dir.as_raw_fd(), c_name.as_ptr(), libc::AT_SYMLINK_FOLLOW) })?;
    return Ok(());
}

/// Give the owner read,
/// write and search permission on the open directory,
/// through its descriptor.
///
/// Why:
/// a directory of mode 555 or 000 cannot be emptied until its owner can write and search it;
/// the change lands on the
/// directory that is held,
/// whatever its path becomes.
/// A directory the user does not own keeps its mode and the later
/// removal reports the operating system's refusal.
pub fn widen_directory(directory: &File) -> std::io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let mode: u32 = directory.metadata()?.mode();
    if mode & 0o700 == 0o700 {
        return Ok(());
    }
    return directory.set_permissions(std::fs::Permissions::from_mode(mode | 0o700));
}

/// Open the subdirectory `name` of `parent` as `open_directory_at` does,
/// giving the owner access to a directory whose mode
///  forbids opening it.
///
/// Why:
/// the retry names the very directory the first open found (a handle that only names it),
/// so a swapped name cannot
/// redirect the change of mode.
pub fn open_directory_widening(parent: &File, name: &OsStr) -> std::io::Result<File> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    match open_directory_at(parent, name) {
        Err(error) if error.raw_os_error() == Some(libc::EACCES) => {
            let handle: File = open_beneath(parent, name, libc::O_PATH | libc::O_DIRECTORY, 0)?;
            let mode: u32 = handle.metadata()?.mode();
            if std::fs::set_permissions(handle_path(&handle), std::fs::Permissions::from_mode(mode | 0o700)).is_err() {
                return Err(error);
            }
            return open_directory_at(parent, name);
        }
        other => return other,
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_beneath_tests.rs"]
mod tests;
