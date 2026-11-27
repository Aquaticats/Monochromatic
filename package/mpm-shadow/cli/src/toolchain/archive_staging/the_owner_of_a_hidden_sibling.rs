//! What:
//! Which process made a hidden sibling,
//! as the four facts that no reuse of a number can repeat:
//! the boot,
//! the process namespace,
//! the process id,
//! and the moment that process started.
//! Why:
//! A later install asks whether the installer that left a hidden directory still runs,
//! and a process id alone answers wrongly:
//! the system hands those numbers out again,
//! so a dead installer's number can belong to a program that has nothing to do with this tool,
//! and reading that program as the installer leaves a copy of a finished install on disk for ever.
//! The kernel reports a moment at which a process started,
//! and no second process with the same number carries the same moment within one boot,
//! so the pair is the fact a leftover is judged by.
//! That moment is counted from the start of the boot,
//! so the same number and the same moment can come back after a reboot;
//! the boot's own identifier separates one boot from the next.
//! A process id is an answer only inside the process namespace that gave it out,
//! so the namespace's identifier separates one namespace from the next.
//!
//! The kernel documents both facts for a process without privileges:
//! `/proc/sys/kernel/random/boot_id` holds a UUID generated at boot and unvarying after that
//! (`Documentation/admin-guide/sysctl/kernel.rst`, "random"),
//! and the content of `/proc/<pid>/ns/pid` is the process namespace's inode number,
//! which two processes in one namespace share
//! (`namespaces(7)`, "The /proc/pid/ns/ directory").
//! Reading one's own entries needs no capability,
//! and a process's own record and namespace are the only ones this module reads of itself.
//!
//! The answer has three values,
//! and only one of them allows anything to be removed:
//! the installer still runs,
//! it has ended,
//! or the system did not say.
//! A read that fails for any reason but "no such process" is the third,
//! because an unreadable record is not a dead process,
//! and treating it as one would let this tool delete a running install's own directory.
//! A name from another boot names an installer that certainly ended,
//! because no process survives a reboot.
//! A name from another process namespace names an installer this reader cannot see,
//! so it is never judged and never removed.
//! A name that carries no boot and no namespace was written by an earlier build
//! and is not judged either.
//!
//! When `/proc/<pid>/stat` says "no such file",
//! the kernel's own `pidfd_open` still answers whether the number belongs to any process
//! (`pidfd_open(2)`: `ESRCH` means it does not).
//! A process the reader can see but whose record `/proc` hides
//! (a `hidepid` mount) is therefore not read as ended.
//! Nothing is asked of the process itself:
//! only its `/proc/<pid>/stat` record and the existence of its number are read,
//! never its command line,
//! its environment or a descriptor.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-owner-of-a-hidden-sibling.ts
//! type Owner = { pid: number; started: number; boot: string | undefined; namespace: number | undefined };
//! type Liveness = 'running' | 'gone' | 'unknown';
//! function here(): Owner;
//! function liveness(owner: Owner): Liveness;
//! ```

/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the call that asks the kernel whether a process number belongs to any process.
use rustix::process::{
    Pid,
    PidfdFlags,
    pidfd_open,
};

/// What this process can say about the process that made a hidden sibling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Liveness {
    /// The same process is still there:
    /// its number is taken and it started at the recorded moment.
    Running,
    /// The number belongs to no process,
    /// so the installer that made the directory has ended.
    Gone,
    /// The system did not answer,
    /// so nothing may be concluded and nothing may be removed.
    Unknown,
}

/// What:
/// The process that made a hidden sibling.
/// Why:
/// A hidden sibling's name holds these facts,
/// so the record of who made it travels with the directory itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Owner = { pid: number; started: number; boot: string | undefined; namespace: number | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Owner {
    /// The process id the system gave it,
    /// inside the namespace this name records.
    pub(crate) pid: u32,
    /// The moment it started,
    /// in clock ticks since boot,
    /// or `0` where the system reports no such moment.
    pub(crate) started: u64,
    /// The boot's identifier,
    /// or nothing for a name an earlier build wrote.
    pub(crate) boot: Option<String>,
    /// The process namespace's inode number,
    /// or nothing for a name an earlier build wrote.
    pub(crate) namespace: Option<u64>,
}

/// What:
/// The problem file the kernel reports one process's own record at,
/// from the process id alone.
/// Why:
/// The record outlives nothing:
/// it is the kernel's answer for the number as it is now,
/// which is why the moment it holds is the fact this module judges by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const statPath = (pid: number) => `/proc/${pid}/stat`;
/// ```
fn stat_path(pid: u32) -> PathBuf {
    return Path::new("/proc").join(pid.to_string()).join("stat");
}

/// What:
/// The two kernel facts that place this reader: the boot and the process namespace.
/// Why:
/// Both are read once per question,
/// so the boot and the namespace a name is compared against cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reader = { boot: string | undefined; namespace: number | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct Reader {
    /// The boot's identifier, or nothing when it cannot be read.
    boot: Option<String>,
    /// The process namespace's inode number, or nothing when it cannot be read.
    namespace: Option<u64>,
}

/// What:
/// Whether the kernel says a process number belongs to any process.
/// Why:
/// `ESRCH` is the kernel saying the number is free;
/// every other answer is either "it is there" or "I could not say".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Existence = 'there' | 'free' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Existence {
    /// The number belongs to a process.
    There,
    /// The number belongs to no process.
    Free,
    /// The kernel did not answer.
    Unknown,
}

/// What:
/// What the read of a process's record answered.
/// Why:
/// "No such process" and "the record could not be read" are different answers,
/// and only the first says the installer ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartTime = { at: bigint } | 'gone' | 'unreadable';
/// ```
enum StartTime {
    /// The record says when the process started.
    At {
        /// The moment the record names,
        /// in clock ticks since boot.
        moment: u64,
    },
    /// The number belongs to no process.
    Gone,
    /// The record could not be read or could not be understood.
    Unreadable,
}

/// What:
/// The moment one `/proc/<pid>/stat` record reports as the process's start,
/// in clock ticks since boot,
/// or nothing when the bytes are not that record.
/// Why:
/// The second field is the program's name in brackets,
/// which may hold spaces,
/// brackets and bytes that are not text,
/// so the record is read as bytes and every field is counted from after the last closing bracket.
/// A record cut short is no record at all,
/// and answering a moment from one would judge a running installer by a made-up number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function momentIn(record: Buffer): bigint | undefined;
/// ```
pub(crate) fn moment_in(record: &[u8]) -> Option<u64> {
    let closing: usize = record.iter().rposition(|byte| return *byte == b')')?;
    let tail: &[u8] = record.get(closing + 1..)?;
    // The record's third field is the state, so the twenty-second is the nineteenth after the name.
    let field: &[u8] = tail
        .split(|byte| return byte.is_ascii_whitespace())
        .filter(|part| return !part.is_empty())
        .nth(19)?;
    return std::str::from_utf8(field).ok()?.parse().ok();
}

/// What:
/// Whether `text` is a boot identifier in the UUID form the kernel writes.
/// Why:
/// The boot field of a name is compared as text with the kernel's own answer,
/// so a name whose field is not that form is not one this scheme wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isBootId(text: string): boolean;
/// ```
pub(crate) fn is_boot_id(text: &str) -> bool {
    let bytes: &[u8] = text.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        let separator: bool = matches!(index, 8 | 13 | 18 | 23);
        if separator {
            if *byte != b'-' {
                return false;
            }
            continue;
        }
        if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    return true;
}

/// What:
/// The kernel's answer for this boot's identifier.
/// Why:
/// A name whose boot is another one names a process that no reboot could keep alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thisBootId(): string | undefined;
/// ```
fn this_boot_id() -> Option<String> {
    #[cfg(test)]
    if let Some(forced) = read_the_forcing_view(|view| return view.boot.clone()) {
        return forced;
    }
    let text: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").ok()?;
    let trimmed: &str = text.trim();
    if !is_boot_id(trimmed) {
        return None;
    }
    return Some(trimmed.to_string());
}

/// What:
/// The kernel's answer for this process namespace's inode number.
/// Why:
/// A process id means one thing inside the namespace that gave it out,
/// so a name from another namespace cannot be judged by this reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thisNamespace(): bigint | undefined;
/// ```
fn this_namespace() -> Option<u64> {
    #[cfg(test)]
    if let Some(forced) = read_the_forcing_view(|view| return view.namespace) {
        return forced;
    }
    let target: PathBuf = std::fs::read_link("/proc/self/ns/pid").ok()?;
    let text: &str = target.to_str()?;
    let inside: &str = text.strip_prefix("pid:[")?.strip_suffix(']')?;
    return inside.parse().ok();
}

/// What:
/// Both kernel facts that place this reader, read once.
/// Why:
/// A question compares a name against one boot and one namespace,
/// never two reads that could fall on either side of a change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thisReader(): Reader;
/// ```
fn this_reader() -> Reader {
    return Reader {
        boot: this_boot_id(),
        namespace: this_namespace(),
    };
}

/// What:
/// Whether the kernel says the process number belongs to any process,
/// asked without `/proc`.
/// Why:
/// `pidfd_open` answers for a number that `hidepid` hides from `/proc`,
/// so a hidden live installer is not read as ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function existenceOf(pid: number): Existence;
/// ```
fn existence_of(pid: u32) -> Existence {
    let Ok(number) = i32::try_from(pid) else {
        return Existence::Unknown;
    };
    let Some(process) = Pid::from_raw(number) else {
        return Existence::Unknown;
    };
    return match pidfd_open(process, PidfdFlags::empty()) {
        // What: the number belongs to a process; the descriptor says so and is closed at once.
        // Why: the caller only needs the answer, not a handle to the process.
        Ok(_) => Existence::There,
        // What: the kernel's `ESRCH` is "no process has this number".
        // Why: that is the one answer that lets a directory be removed.
        Err(rustix::io::Errno::SRCH) => Existence::Free,
        // What: an old kernel without the call is the same answer as `/proc` alone can give.
        // Why: no better question can be asked there, and the boot and namespace still bound the reading.
        Err(rustix::io::Errno::NOSYS) => Existence::Free,
        // What: any other failure is the kernel not saying.
        // Why: nothing may be concluded, so nothing is removed.
        Err(_) => Existence::Unknown,
    };
}

/// What:
/// The moment `/proc/<pid>/stat` reports as the process's start,
/// in clock ticks since boot.
/// Why:
/// "No such file" is the system saying the process ended,
/// which is an answer;
/// any other failure is the system saying nothing,
/// which is not.
/// A number whose record `/proc` hides is still a process,
/// which `pidfd_open` can tell apart from a free number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startTimeOf(pid: number): bigint | 'gone' | 'unreadable';
/// ```
fn start_time_of(pid: u32) -> StartTime {
    #[cfg(test)]
    if let Some(forced) = read_the_forcing_view(|view| return view.start) {
        return match forced {
            ForcedStart::At {
                moment,
            } => StartTime::At {
                moment,
            },
            ForcedStart::Gone => StartTime::Gone,
            ForcedStart::Unreadable => StartTime::Unreadable,
        };
    }
    let record: Vec<u8> = match std::fs::read(stat_path(pid)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // What: the record is absent; the kernel's own answer for the number decides whether that means ended.
            // Why: a `hidepid` mount hides a live process's record, and reading that as ended would remove its work.
            return match existence_of(pid) {
                Existence::There => StartTime::Unreadable,
                Existence::Free => StartTime::Gone,
                Existence::Unknown => StartTime::Unreadable,
            };
        }
        Err(_) => return StartTime::Unreadable,
    };
    return match moment_in(&record) {
        Some(moment) => StartTime::At {
            moment,
        },
        None => StartTime::Unreadable,
    };
}

/// What:
/// The moment a process the system reports started at,
/// or nothing when it reports none,
/// for a control that builds the pair of a process it can see.
/// Why:
/// A control has no use for the difference between a number no process holds and a record it cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startTimeOfAProcess(pid: number): bigint | undefined;
/// ```
#[cfg(test)]
pub(crate) fn start_time_of_a_process(pid: u32) -> Option<u64> {
    return match start_time_of(pid) {
        StartTime::At {
            moment,
        } => Some(moment),
        StartTime::Gone | StartTime::Unreadable => None,
    };
}

/// What:
/// An answer a control makes the reader give instead of asking the kernel.
/// Why:
/// The environments this module guards against are not reproducible on this host
/// (another boot, another process namespace, a `hidepid` mount),
/// so a control needs a seam of the code's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForcedStart = { at: bigint } | 'gone' | 'unreadable';
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ForcedStart {
    /// The record reports this moment.
    At {
        /// The moment, in clock ticks since boot.
        moment: u64,
    },
    /// The record is absent.
    Gone,
    /// The record could not be read or understood.
    Unreadable,
}

/// What:
/// The kernel facts and the record answer a control forces, field by field.
/// Why:
/// A test can put the reader in another boot or namespace,
/// or hide a record,
/// without changing anything on this machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReaderView = { boot?: string | undefined; namespace?: bigint | undefined; start?: ForcedStart };
/// ```
#[cfg(test)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ReaderView {
    /// When present, the boot answer the reader gives,
    /// including "no answer" as an inner `None`.
    pub(crate) boot: Option<Option<String>>,
    /// When present, the namespace answer the reader gives,
    /// including "no answer" as an inner `None`.
    pub(crate) namespace: Option<Option<u64>>,
    /// When present, the answer for every process record.
    pub(crate) start: Option<ForcedStart>,
}

// What:
// The one forcing view a test thread holds.
// Why:
// Tests in one binary may run at the same time,
// so each thread gets its own answer and no test changes another's.
//
// In TS you'd write (pseudocode):
// ```ts
// const readerView: ReaderView = {};
// ```
#[cfg(test)]
thread_local! {
    // The forcing view of the thread reading now.
    static THE_FORCING_VIEW: std::cell::RefCell<ReaderView> =
        const { std::cell::RefCell::new(ReaderView { boot: None, namespace: None, start: None }) };
}

/// What:
/// Reads one field of the forcing view, or nothing when no view is armed.
/// Why:
/// Every kernel read goes through this,
/// so a control's answer reaches exactly the question it was made for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTheForcingView<T>(pick: (view: ReaderView) => T | undefined): T | undefined;
/// ```
#[cfg(test)]
fn read_the_forcing_view<T>(pick: impl FnOnce(&ReaderView) -> Option<T>) -> Option<T> {
    return THE_FORCING_VIEW.with(|view| return pick(&view.borrow()));
}

/// What:
/// Puts this test thread under a forcing view until the guard drops.
/// Why:
/// A test that changes the boot, namespace or record answer
/// must put the reader back even when an assertion fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAs(view: ReaderView): { [Symbol.dispose](): void };
/// ```
#[cfg(test)]
pub(crate) struct ReaderViewGuard;

/// Clears the forcing view when the guard leaves.
#[cfg(test)]
impl Drop for ReaderViewGuard {
    fn drop(&mut self) {
        THE_FORCING_VIEW.with(|view| {
            *view.borrow_mut() = ReaderView {
                boot: None,
                namespace: None,
                start: None,
            };
        });
    }
}

/// What:
/// Arms the forcing view for this test thread.
/// Why:
/// The control describes what the reader sees,
/// and the guard takes the description away when the test ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAs(view: ReaderView): ReaderViewGuard;
/// ```
#[cfg(test)]
pub(crate) fn read_as(view: ReaderView) -> ReaderViewGuard {
    THE_FORCING_VIEW.with(|held| {
        *held.borrow_mut() = view;
    });
    return ReaderViewGuard;
}

/// The methods of `Owner`.
impl Owner {
    /// What:
    /// The owner of what this process is making now.
    /// Why:
    /// Every hidden sibling this process writes carries these facts,
    /// so a later install can tell whether that process is still there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function here(): Owner { return { pid: process.pid, started: startTimeOf(process.pid) ?? 0, boot: thisBootId(), namespace: thisNamespace() }; }
    /// ```
    pub(crate) fn here() -> Owner {
        let pid: u32 = std::process::id();
        let reader: Reader = this_reader();
        let started: u64 = match start_time_of(pid) {
            StartTime::At {
                moment,
            } => moment,
            StartTime::Gone | StartTime::Unreadable => 0,
        };
        return Owner {
            pid,
            started,
            boot: reader.boot,
            namespace: reader.namespace,
        };
    }

    /// What:
    /// What this process can say about the process that made a hidden sibling.
    /// Why:
    /// Only a process the system reports as ended may lose its directory:
    /// a number now held by another program carries a different start and is that program,
    /// while a record that could not be read says nothing at all.
    /// A name from another boot is an installer that reboot certainly ended.
    /// A name from another namespace,
    /// or one an earlier build wrote without these facts,
    /// is never judged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function liveness(owner: Owner): Liveness {
    ///   if (owner.started === 0) return 'unknown';
    ///   const reader = thisReader();
    ///   if (reader.boot === undefined || reader.namespace === undefined) return 'unknown';
    ///   if (owner.boot === undefined || owner.namespace === undefined) return 'unknown';
    ///   if (owner.boot !== reader.boot) return 'gone';
    ///   if (owner.namespace !== reader.namespace) return 'unknown';
    ///   const seen = startTimeOf(owner.pid);
    ///   if (seen === 'gone') return 'gone';
    ///   if (seen === 'unreadable') return 'unknown';
    ///   return seen === owner.started ? 'running' : 'gone';
    /// }
    /// ```
    pub(crate) fn liveness(&self) -> Liveness {
        if self.started == 0 {
            return Liveness::Unknown;
        }
        let reader: Reader = this_reader();
        let (Some(this_boot), Some(this_namespace)) = (reader.boot, reader.namespace) else {
            return Liveness::Unknown;
        };
        let (Some(named_boot), Some(named_namespace)) = (&self.boot, self.namespace) else {
            return Liveness::Unknown;
        };
        if *named_boot != this_boot {
            return Liveness::Gone;
        }
        if named_namespace != this_namespace {
            return Liveness::Unknown;
        }
        return match start_time_of(self.pid) {
            StartTime::At {
                moment,
            } if moment == self.started => Liveness::Running,
            StartTime::At {
                moment: _,
            } => Liveness::Gone,
            StartTime::Gone => Liveness::Gone,
            StartTime::Unreadable => Liveness::Unknown,
        };
    }
}
