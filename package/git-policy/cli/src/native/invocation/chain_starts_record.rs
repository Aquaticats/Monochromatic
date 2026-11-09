//! What:
//!  The marker the last wrapper start wrote for its Git,
//! carried in a small memory file whose descriptor every child inherits.
//! Why:
//!  The marker in the environment and the environment of the process above are records a launcher can destroy:
//!  `env --ignore-environment` clears the environment it passes on,
//! and a launcher that `exec`s the wrapper in its own process leaves no second process to ask and no environment
//!  of its own to read.
//! One thing outlives both:
//! an open descriptor,
//! which no environment clearing touches and which `exec` keeps unless it is close-on-exec,
//!  so the wrapper's own start is still readable after a launcher has cleared everything.
//! So every start of real Git writes its marker into the record,
//! and a wrapper that finds the record naming its own
//!  process,
//! or the process that started it,
//! continues that chain:
//! a start in its own process is the launcher
//!  that replaced itself with the wrapper,
//! and a marker of a read the wrapper made of itself is the branch the query rule
//!  already refuses.
//! A record naming any other process is a start that has ended,
//! so the count starts again at
//!  one and a long session of one command after another is never refused.
//! The file has no name in any directory:
//! it is memory the kernel owns and it goes away with the last process that
//!  holds it,
//! so nothing needs a sweep,
//! and it never grows past one marker and one path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = openInheritedChainRecord() ?? createChainRecord(); // a memfd every child inherits
//! writeMarker(record, markerOfTheStart); writeChosen(record, realGit); // and a later start reads both
//! ```

/// Text for a record that cannot be made,
/// which never fails a command.
use crate::diagnostic_log::debug;
/// OS text,
/// which a marker and a path both are.
use std::ffi::OsStr;

/// The name the memory file carries,
/// which is how a later start finds it among the descriptors it inherited.
pub const RECORD_NAME: &str = "monochromatic-git-policy-cli-chain-start";

/// The tag of this module's diagnostics.
const TAG: &str = "chainStartsRecord";

/// The most text a marker fills,
/// which the controls of the record use as the room they write into.
pub const MARKER_ROOM: usize = 256;

/// The most text a chosen path fills,
/// which is as long as a path may be,
/// and which the controls of the record use as the room they write into.
pub const CHOSEN_ROOM: usize = 4096 + 256;

/// Where the marker begins in the record.
const MARKER_OFFSET: i32 = 0;

/// Where the chosen path begins in the record,
/// after the marker's room,
/// so one write does not disturb the other.
const CHOSEN_OFFSET: i32 = MARKER_ROOM as i32;

/// The byte a record's text is padded with,
/// which no marker and no path contains.
const PADDING: u8 = 0;

/// What:
///  The record this process inherited,
///  as it was before this start wrote anything.
/// Why:
///   A wrapper reads the start above it and then writes its own,
/// so the two have to be separate steps;
/// the descriptor is
///  kept open for the life of the process so every child inherits it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChainRecord = { fd: number; marker?: string; chosen?: string };
/// ```
pub struct ChainRecord {
    /// The open descriptor,
    /// inherited by every child,
    /// unless a program in between closes it.
    descriptor: i32,
    /// The marker the start above wrote for its Git.
    marker: Option<String>,
    /// The file that start chose as Git.
    chosen: Option<String>,
}

impl ChainRecord {
    /// What:
    ///  The record for this start:
    /// the descriptor inherited from the process above,
    /// or a new one.
    /// Why:
    ///   Nothing is written here:
    /// this start writes its own marker when it starts its Git,
    /// which is the moment
    ///  the record is about,
    /// and a start that refuses before then leaves the one above as it was.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fd = inheritedChainRecord() ?? createChainRecord(); const [marker, chosen] = readParts(fd);
    /// ```
    pub fn for_this_start() -> Option<ChainRecord> {
        return for_this_start_with(inherited_descriptor);
    }

    /// What:
    ///  The marker the start above this one wrote for its Git.
    /// Why:
    ///   It is a full marker:
    /// the process,
    /// its birth,
    /// the way it started its Git and the count of the chain,
    /// so the
    ///  guard reads it exactly as it reads one from an environment.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record.marker
    /// ```
    pub fn marker(&self) -> Option<&str> {
        return self.marker.as_deref();
    }

    /// What:
    ///  The file the start above chose as Git,
    /// when it chose one.
    /// Why:
    ///   A refusal names the file a person can remove from `PATH`,
    /// and the shape a cleared environment comes with has no
    ///  environment left to read it from.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record.chosen
    /// ```
    pub fn chosen(&self) -> Option<&str> {
        return self.chosen.as_deref();
    }

    /// The descriptor this record holds,
    /// for the controls that read the record back.
    #[cfg(test)]
    pub fn descriptor(&self) -> i32 {
        return self.descriptor;
    }
}

/// What:
///  The record this process carries,
/// made once,
/// at the first question asked about the chain.
/// Why:
///   Every child this process starts has to inherit the descriptor,
/// so it is made before anything is started and kept
///  for the life of the process;
/// the question it answers costs no syscall after that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const chain = new Lazy(() => ChainRecord.forThisStart());
/// ```
pub fn this_chain() -> Option<&'static ChainRecord> {
    static RECORD: std::sync::OnceLock<Option<ChainRecord>> = std::sync::OnceLock::new();
    return RECORD.get_or_init(ChainRecord::for_this_start).as_ref();
}

/// What:
///  Writes the marker this start gives its Git into the chain record.
/// Why:
///   This is the whole point of the record:
/// a start that has ended still leaves what it started,
/// and a launcher
///  that clears everything cannot remove it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeAt(chain.fd, MARKER_OFFSET, marker);
/// ```
pub fn remember_marker(marker: &OsStr) {
    let Some(record) = this_chain() else { return };
    let _written = write_part(record.descriptor, MARKER_OFFSET, MARKER_ROOM, marker);
}

/// What:
///  Writes a marker into a record a control made,
///  as the start above this one would have left it.
/// Why:
///  A record is inherited,
/// so a control of what a start reads makes the record itself and hands the guard a process
///  table that already holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMarkerForControls(fd, markerText) // what the wrapper does before it starts its Git
/// ```
#[cfg(test)]
pub fn write_marker_for_controls(descriptor: i32, marker: &str) -> bool {
    return write_part(descriptor, MARKER_OFFSET, MARKER_ROOM, OsStr::new(marker));
}

/// What:
///  Writes the file this start chose as Git into the chain record,
/// beside the marker of this start.
/// Why:
///   The record is the only place a start that a launcher execs the wrapper out of can leave the file it chose,
/// so the
///  refusal a later start writes can name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeAt(chain.fd, CHOSEN_OFFSET, realGit);
/// ```
pub fn remember_chosen(target: &OsStr) {
    let Some(record) = this_chain() else { return };
    let _written = write_part(record.descriptor, CHOSEN_OFFSET, CHOSEN_ROOM, target);
}

#[cfg(unix)]
impl Drop for ChainRecord {
    /// Closes the descriptor this process holds,
    /// which ends the memory file when no child still holds it.
    fn drop(&mut self) {
        // SAFETY: `close` takes one descriptor this process owns and no pointer; the drop runs once because nothing
        //  clones the record.
        let _closed = unsafe { libc::close(self.descriptor) };
    }
}

/// Without a descriptor to close there is nothing to release (this crate runs on Linux only).
#[cfg(not(unix))]
impl Drop for ChainRecord {
    /// Leaves the record to the platform.
    fn drop(&mut self) {}
}

/// What:
///  `ChainRecord::for_this_start` with the way the inherited descriptor is found given,
/// so a control can hand in a descriptor
///  it made itself.
/// Why:
///   The lookup reads this process's own descriptor table,
/// which a control of the record cannot arrange for another
///  process;
/// the rest of the record (reading,
/// writing,
/// the two parts) is the same either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forThisStartWith(inherited: () => number | undefined): ChainRecord | undefined;
/// ```
pub fn for_this_start_with(inherited: impl Fn() -> Option<i32>) -> Option<ChainRecord> {
    let Some(descriptor) = inherited().or_else(create_descriptor) else {
        debug(TAG, "no chain record could be made, so a launcher that clears the environment and execs the wrapper is not counted");
        return None;
    };
    let (marker, chosen): (Option<String>, Option<String>) = read_parts(descriptor);
    return Some(ChainRecord { descriptor, marker, chosen });
}

/// What:
///  The descriptor of the chain record this process inherited,
/// or nothing when it carries none.
/// Why:
///   A `memfd` is found by the name it carries,
/// which `/proc/self/fd` shows as `/memfd:<name>`;
/// the descriptor
///  number is the process's own choice,
/// so it is read from the table and never assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdirSync('/proc/self/fd').find((entry) => readlinkSync(`/proc/self/fd/${entry}`) === `/memfd:${RECORD_NAME}`)
/// ```
pub fn inherited_descriptor() -> Option<i32> {
    return inherited_descriptor_in(std::path::Path::new("/proc/self/fd"));
}

/// The number a descriptor table's entry names,
/// or nothing when the entry is not a number.
fn descriptor_number(name: &str) -> Option<i32> {
    return name.parse::<i32>().ok();
}

/// Whether this byte ends a record's text.
fn is_padding(byte: &u8) -> bool {
    return *byte == PADDING;
}

/// What:
///  `inherited_descriptor` with the descriptor table's directory given.
/// Why:
///   A control fabricates the table as links to the memory file's name,
/// which is the only part of the lookup that needs a live
///  process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inheritedDescriptorIn(directory: string): number | undefined;
/// ```
pub fn inherited_descriptor_in(directory: &std::path::Path) -> Option<i32> {
    let listing = std::fs::read_dir(directory).ok()?;
    for entry in listing.flatten() {
        let Some(number) = entry.file_name().to_str().and_then(descriptor_number) else {
            continue;
        };
        let Ok(target) = std::fs::read_link(entry.path()) else { continue };
        if carries_record_name(target.as_path()) {
            return Some(number);
        }
    }
    return None;
}

/// What:
///  Whether this descriptor target is the chain record's memory file.
/// Why:
///   The kernel spells a named memory file as `/memfd:<name> (deleted)`,
/// because a memory file has no name in any
///  directory;
/// a file the person's program holds has a path of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// target === `/memfd:${RECORD_NAME} (deleted)`;
/// ```
fn carries_record_name(target: &std::path::Path) -> bool {
    let named: String = format!("/memfd:{RECORD_NAME} (deleted)");
    return target == std::path::Path::new(named.as_str());
}

/// What:
///  A new chain record:
/// a memory file that is readable and writable by this process and by every child it starts.
/// Why:
///   `memfd_create` with no close-on-exec flag is the whole of it:
/// the file has no directory entry,
/// so nothing can
///  find it in a directory,
/// and it is freed by the kernel when the last descriptor on it closes.
/// A kernel without the call reports it and
///  the command runs as it did before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// syscall(SYS_memfd_create, RECORD_NAME, 0) // no MFD_CLOEXEC: children must inherit it
/// ```
#[cfg(unix)]
pub fn create_descriptor() -> Option<i32> {
    let name: std::ffi::CString = std::ffi::CString::new(RECORD_NAME).ok()?;
    // SAFETY: `memfd_create` takes a pointer to a NUL-terminated name this call keeps alive and a flags word of zero;
    //  it returns a descriptor or -1 and touches no memory of this process.
    let descriptor: libc::c_int = unsafe { libc::syscall(libc::SYS_memfd_create, name.as_ptr(), 0u32) } as libc::c_int;
    if descriptor < 0 {
        debug(TAG, format!("a chain record cannot be made: {}", std::io::Error::last_os_error()).as_str());
        return None;
    }
    return above_the_standard_streams(descriptor);
}

/// The first descriptor a record may take:
/// the three below it are the streams a child reads and writes.
const FIRST_RECORD_DESCRIPTOR: libc::c_int = 3;

/// What:
///  The record's descriptor,
/// moved above the three a child reads as its streams.
/// Why:
///   `memfd_create` takes the lowest free descriptor,
/// which is `0` when standard input is closed and `1` or `2` when
///  one of the others is;
/// the wrapper writes its marker into the record,
/// so a child that inherits that stream reads
///  the marker,
/// its padding and the path of the chosen Git instead of what the person gave it
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 4 of the round-3 review).
/// `F_DUPFD` takes the lowest free descriptor at or above three,
/// and the one the kernel gave is closed afterwards,
/// so the record keeps its
///  contents and the stream stays closed.
/// A move the kernel refuses leaves no record,
/// and the command runs as it did before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const above = (fd) => fd >= 3 ? fd : (() => { const moved = fcntl(fd, F_DUPFD, 3); close(fd); return moved; })();
/// ```
#[cfg(unix)]
fn above_the_standard_streams(descriptor: libc::c_int) -> Option<libc::c_int> {
    if descriptor >= FIRST_RECORD_DESCRIPTOR {
        return Some(descriptor);
    }
    // SAFETY: `fcntl(F_DUPFD, ..)` duplicates this process's descriptor to the lowest free number at or above the one
    //  asked for, touches no memory, and returns the new descriptor or -1.
    let moved: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_DUPFD, FIRST_RECORD_DESCRIPTOR) };
    // The number the kernel gave is closed either way: nothing else holds this memory file, and a stream that was closed
    //  must stay closed for the child.
    // SAFETY: `close` takes a descriptor this process owns and touches no memory.
    let _closed = unsafe { libc::close(descriptor) };
    if moved < FIRST_RECORD_DESCRIPTOR {
        debug(TAG, format!("a chain record cannot be moved above the standard streams: {}", std::io::Error::last_os_error()).as_str());
        return None;
    }
    return Some(moved);
}

/// What:
///  The marker and the chosen path a record holds,
/// or two nothings when it holds none.
/// Why:
///   One read of the whole record:
/// every holder shares one offset,
/// so the parts are read by position and
///  never by advancing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = readAt(fd, 0, MARKER_ROOM + CHOSEN_ROOM); const [marker, chosen] = splitOnNul(text);
/// ```
#[cfg(unix)]
pub fn read_parts(descriptor: i32) -> (Option<String>, Option<String>) {
    // `mut` allows the buffer to be filled in place.
    let mut bytes: Vec<u8> = vec![0; MARKER_ROOM + CHOSEN_ROOM];
    // SAFETY: `pread` reads into the buffer this call owns for its whole length, at an offset that exists or not, and
    //  returns how much it read or -1.
    let read: isize = unsafe { libc::pread(descriptor, bytes.as_mut_ptr().cast(), bytes.len(), 0) };
    let Ok(read_bytes) = usize::try_from(read) else { return (None, None) };
    let marker: Option<String> = text_of(&bytes, 0, MARKER_ROOM.min(read_bytes));
    let chosen: Option<String> = text_of(&bytes, MARKER_ROOM.min(read_bytes), read_bytes);
    return (marker, chosen);
}

/// What:
///  The text between two positions of a record,
/// or nothing when it is empty.
/// Why:
///   A record's text runs to its NUL padding,
/// and a part this start never wrote reads as nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = buffer.subarray(from, to).toString('utf8').replace(/\0.*$/s, ''); return text || undefined;
/// ```
#[cfg(unix)]
fn text_of(bytes: &[u8], from: usize, to: usize) -> Option<String> {
    let part: &[u8] = bytes.get(from..to)?;
    let end: usize = part.iter().position(is_padding).unwrap_or(part.len());
    let text: &[u8] = &part[..end];
    if text.is_empty() {
        return None;
    }
    return Some(String::from_utf8_lossy(text).into_owned());
}

/// What:
///  Writes one part of the record:
/// its text and the NUL that ends it.
/// Why:
///   The two parts are written by different moments of one start,
/// so each owns its room and neither disturbs the other;
/// the
///  NUL ends the text,
/// so bytes of a longer text that stood there before are past it and read as nothing;
/// a
///  text longer than its room is written short and read back truncated,
/// which the guard treats as an unknown
///  marker.
/// Only the text is written and not the room,
/// which is what a forwarded command notices.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeAt(fd, offset, Buffer.concat([Buffer.from(text.slice(0, room - 1)), Buffer.alloc(1)]));
/// ```
#[cfg(unix)]
fn write_part(descriptor: i32, offset: i32, room: usize, text: &OsStr) -> bool {
    use std::os::unix::ffi::OsStrExt;
    // `mut` allows the ending NUL to be added after the text.
    let mut bytes: Vec<u8> = text.as_bytes().to_vec();
    bytes.truncate(room.saturating_sub(1));
    bytes.push(PADDING);
    // SAFETY: `pwrite` writes the bytes this call owns at an offset that exists or not, and returns how much it wrote
    //  or -1.
    let written: isize = unsafe { libc::pwrite(descriptor, bytes.as_ptr().cast(), bytes.len(), offset.into()) };
    return written == isize::try_from(bytes.len()).unwrap_or(-1);
}

/// Without a memory file there is no record to make (this crate runs on Linux only).
#[cfg(not(unix))]
pub fn create_descriptor() -> Option<i32> {
    return None;
}

/// Without a descriptor there is nothing to read.
#[cfg(not(unix))]
pub fn read_parts(_descriptor: i32) -> (Option<String>, Option<String>) {
    return (None, None);
}

/// Without a descriptor there is nothing to write.
#[cfg(not(unix))]
fn write_part(_descriptor: i32, _offset: i32, _room: usize, _text: &OsStr) -> bool {
    return false;
}

/// Records and the guard they feed stay out of the release executable's controls.
#[cfg(test)]
#[path = "chain_starts_record_tests.rs"]
mod tests;
