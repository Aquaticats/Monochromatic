//! What:
//!  One probe of this host's account companion group drop:
//!  the child that makes the companion's own first two calls in the companion's
//!  order,
//!  the parent's bounded wait,
//!  and what the answer means for a person.
//! Why:
//!  `doctor` may not write anything,
//!  may not claim a slot and may not run an add,
//!  so a person learns that the companion's group drop is skipped only from an
//!  add's own line.
//!  The owner asked for `doctor` to say it instead:
//!  "Add the skip and make the doctor tell user what to do."
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "The account companion's group drop on Ubuntu 24.04").
//!  The fact is measured the way an add measures it,
//!  because nothing outside the process can see it:
//!  a child of this probe makes `unshare(CLONE_NEWUSER)`,
//!  opens `/proc/self/setgroups` for write,
//!  writes `deny`,
//!  and answers over a pipe.
//!  The probe touches no mount,
//!  no ZFS state and no file of the person's,
//!  and its child ends on every path.
//!
//!  Why these calls are repeated here rather than called from the companion:
//!  the layout rule (`rust/folder-uses-links`,
//!  `doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`)
//!  finds `account_companion` and `invocation` inside one loop it tolerates
//!  today,
//!  and a tolerated loop may only lose uses,
//!  so a new use between them may not be added.
//!  The two folders' calls and this probe are meant to be one code path:
//!  the untangling round that takes `account_companion` out of that loop
//!  replaces the calls below with the companion's own
//!  (`companion_file_calls.rs`,
//!  `companion_process_calls.rs`) and moves the words onto
//!  `companion_calls.rs`,
//!  which is where they are pinned today beside these copies.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const drop = theGroupDrop(); // { kind: 'written' } | { kind: 'skipped', errno } | { kind: 'answered', call, errno } | { kind: 'unread', reason }
//! ```

use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
use std::time::Duration;

/// What:
///  The file whose write open asks the kernel for a capability.
/// Why:
///  Its open is the call Ubuntu 24.04's default AppArmor profile refuses,
///  so it is the one this probe exists to make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SETGROUPS_PATH = '/proc/self/setgroups';
/// ```
const SETGROUPS_PATH: &std::ffi::CStr = c"/proc/self/setgroups";

/// What:
///  The word the child writes to deny the group drop,
///  which is the word the companion's own child writes.
/// Why:
///  `deny` is the only length `proc_setgroups_write` accepts,
///  so any other word would answer the kernel a different question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DENY = new TextEncoder().encode('deny');
/// ```
const DENY: &[u8] = b"deny";

/// What:
///  How many bytes the child's answer can take:
///  a call's word and a decimal errno.
/// Why:
///  The parent reads one answer once,
///  and a fixed buffer keeps that read off the heap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ANSWER_BYTES = 24;
/// ```
const ANSWER_BYTES: usize = 24;

/// What:
///  How long the parent waits for the child's answer before it kills the child
///  and says the probe could not be read.
/// Why:
///  The owner's rule is that nothing waits forever,
///  and the child makes three calls that answer at once.
///  100 ms is the companion's own measured bound
///  (`account_companion/companion_deadline.rs`,
///  the slowest run of five interleaved rounds plus that deadline's margin),
///  repeated here because this folder may not use that one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PROBE_DEADLINE = 100; // ms
/// ```
const PROBE_DEADLINE: Duration = Duration::from_millis(100);

/// What:
///  Which of the companion's calls answered,
///  as names this probe writes and reads back.
/// Why:
///  The report says which call answered,
///  so the words must survive the pipe.
///  They are the companion's own words (`account_companion/companion_calls.rs`)
///  and the untangling named above moves this list back onto them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProbeCall = 'unshare' | 'open' | 'write';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeCall {
    /// `unshare(CLONE_NEWUSER)`.
    UnshareUserNamespace,
    /// The open of `/proc/self/setgroups` for write.
    OpenSetgroups,
    /// The `deny` write itself.
    WriteSetgroups,
}

/// What:
///  What one probe answered about this host.
/// Why:
///  `doctor` prints a report only for the skip,
///  and the other three answers are told apart so a control can hold each:
///  a host that wrote the `deny` needs no report,
///  a host that answered elsewhere has that answer in the add's own line,
///  and a probe that could not be read must say so rather than pass for a
///  passing host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GroupDrop = { kind: 'written' } | { kind: 'skipped'; errno: number } | { kind: 'answered'; call: ProbeCall; errno: number } | { kind: 'unread'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupDrop {
    /// The `deny` write was made,
    /// so an add's companion passes this step on this host.
    Written,
    /// The open answered `EACCES`,
    /// which is the skip the owner added.
    Skipped {
        /// The number the open answered.
        errno: i32,
    },
    /// The host answered somewhere else,
    /// so its own line is the add's and this probe prints no report.
    Answered {
        /// The call that answered.
        call: ProbeCall,
        /// The number it answered.
        errno: i32,
    },
    /// The probe could not be run or its answer could not be read.
    Unread {
        /// Why.
        reason: String,
    },
}

/// What:
///  Probe this host for the companion's group drop.
/// Why:
///  This is the read `doctor` makes,
///  and the controls hand their own answers instead of forking where they do
///  not need a child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const drop = (() => { const [read, write] = pipe(); if (fork() === 0) child(write); return wait(read); })();
/// ```
pub fn the_group_drop() -> GroupDrop {
    let (reading, writing): (OwnedFd, OwnedFd) = match pipe_pair() {
        Ok(ends) => ends,
        Err(errno) => {
            return GroupDrop::Unread {
                reason: format!("its answer pipe could not be made (errno {errno})"),
            };
        }
    };
    let child: libc::pid_t = match fork_once() {
        Ok(Some(pid)) => pid,
        Ok(None) => {
            // What: the child drops its copy of the reading end, which leaves it the writing end only.
            // Why: a reading end held open in the child is one the parent never sees end.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // reading.close();
            // ```
            drop(reading);
            run_the_probe(writing.as_raw_fd());
        }
        Err(errno) => {
            return GroupDrop::Unread {
                reason: format!("its child could not be started (errno {errno})"),
            };
        }
    };
    // What: the parent drops its copy of the writing end, which leaves the child the only writer.
    // Why: the answer pipe ends when the child ends, and a writing end held here would hide that.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writing.close();
    // ```
    drop(writing);
    return wait_for_the_probe(child, &reading);
}

/// What:
///  Take the companion's first two calls in its order,
///  and write one answer over the pipe.
/// Why:
///  The order is the fact being measured:
///  a host that refuses the open never reaches the write,
///  and a companion that wrote first would ask the kernel a different question.
///
/// Never returns:
///  the child ends with `_exit` after its answer,
/// because nothing after a `fork` may run the parent's destructors.
fn run_the_probe(status: libc::c_int) -> ! {
    // The child asks for a user namespace of its own, which is the companion's first step.
    let unshared: libc::c_int = unsafe { libc::unshare(libc::CLONE_NEWUSER) };
    if unshared != 0 {
        write_answer(status, ProbeCall::UnshareUserNamespace, errno_now());
        unsafe { libc::_exit(1) };
    }
    // The open asks for `CAP_SYS_ADMIN` inside the new namespace, which is where Ubuntu 24.04's profile answers.
    let opened: libc::c_int = unsafe { libc::open(SETGROUPS_PATH.as_ptr(), libc::O_WRONLY) };
    if opened < 0 {
        write_answer(status, ProbeCall::OpenSetgroups, errno_now());
        unsafe { libc::_exit(1) };
    }
    // The write itself asks for no capability, so a refusal here is the companion's own defect and not this host's policy.
    let written: libc::ssize_t = unsafe { libc::write(opened, DENY.as_ptr().cast(), DENY.len()) };
    // The errno is read before anything else can move it.
    let failure: i32 = errno_now();
    let _closed: libc::c_int = unsafe { libc::close(opened) };
    if written != DENY.len() as libc::ssize_t {
        write_answer(status, ProbeCall::WriteSetgroups, failure);
        unsafe { libc::_exit(1) };
    }
    write_bytes(status, b"written");
    unsafe { libc::_exit(0) };
}

/// What:
///  Wait for the child's answer under the probe's bound,
///  read it,
///  end the child,
///  and judge what it wrote.
/// Why:
///  `doctor` may not wait with no end,
///  and a child the kernel never releases must not hold the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ready = poll(read, PROBE_DEADLINE) ? readAll(read) : (kill(-child), 'unread');
/// ```
fn wait_for_the_probe(
    child: libc::pid_t,
    reading: &OwnedFd,
) -> GroupDrop {
    // What: one `poll` question about the answer pipe, and the millisecond count `poll` takes.
    // Why: the wait ends at the bound whatever the child does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const asked = { fd: read, events: POLLIN, revents: 0 };
    // ```
    let mut asked: libc::pollfd = libc::pollfd {
        fd: reading.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let answered: libc::c_int =
        unsafe { libc::poll(&mut asked, 1, PROBE_DEADLINE.as_millis() as libc::c_int) };
    let mut bytes: [u8; ANSWER_BYTES] = [0; ANSWER_BYTES];
    if answered <= 0 {
        // The child outlasted the bound, so it is killed and its number is collected below.
        let _killed: libc::c_int = unsafe { libc::kill(child, libc::SIGKILL) };
        let _reaped: libc::pid_t = unsafe { libc::waitpid(child, std::ptr::null_mut(), 0) };
        return GroupDrop::Unread {
            reason: format!("it did not answer within {} ms", PROBE_DEADLINE.as_millis()),
        };
    }
    // A read of the answer the child already wrote: the pipe holds it, so this cannot wait.
    let read: libc::ssize_t =
        unsafe { libc::read(reading.as_raw_fd(), bytes.as_mut_ptr().cast(), ANSWER_BYTES) };
    // The child ends on this path too, whether it answered or died before writing.
    let _reaped: libc::pid_t = unsafe { libc::waitpid(child, std::ptr::null_mut(), 0) };
    if read <= 0 {
        return GroupDrop::Unread {
            reason: String::from("its child wrote no answer"),
        };
    }
    return the_answer(&bytes[..read as usize]);
}

/// What:
///  What the bytes the child wrote mean for this host.
/// Why:
///  The judgement is one function of the answer,
///  so every answer is held by a control without a fork,
///  and only the open's `EACCES` is the skip:
///  every other answer is the call's own and prints no report here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = new TextDecoder().decode(bytes);
/// if (text === 'written') return { kind: 'written' };
/// const [word, digits] = text.split(' ');
/// const call = CALLS[word]; const errno = Number(digits);
/// return call === 'open' && errno === 13 ? { kind: 'skipped', errno } : { kind: 'answered', call, errno };
/// ```
fn the_answer(written: &[u8]) -> GroupDrop {
    let text: &str = match std::str::from_utf8(written) {
        Ok(read) => read,
        Err(_not_text) => {
            return GroupDrop::Unread {
                reason: String::from("its child wrote an answer this build cannot read"),
            };
        }
    };
    if text == "written" {
        return GroupDrop::Written;
    }
    let mut words = text.split(' ');
    let (Some(word), Some(digits)) = (words.next(), words.next()) else {
        return GroupDrop::Unread {
            reason: format!("its child wrote `{text}`, which names no call and no number"),
        };
    };
    let call: ProbeCall = match ProbeCall::read_word(word) {
        Some(named) => named,
        None => {
            return GroupDrop::Unread {
                reason: format!(
                    "its child named the call `{word}`, which this build does not know"
                ),
            };
        }
    };
    let Ok(errno) = digits.parse::<i32>() else {
        return GroupDrop::Unread {
            reason: format!("its child answered `{digits}`, which is no number"),
        };
    };
    if call == ProbeCall::OpenSetgroups && errno == libc::EACCES {
        return GroupDrop::Skipped {
            errno,
        };
    }
    return GroupDrop::Answered {
        call,
        errno,
    };
}

impl ProbeCall {
    /// What:
    ///  The word this call is written and read by,
    ///  which is the companion's own name for it.
    /// Why:
    ///  The report and the answer over the pipe name one call,
    ///  so the words have to agree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = { unshare: 'unshare', open: 'open', write: 'write' }[call];
    /// ```
    fn word(self) -> &'static str {
        return match self {
            ProbeCall::UnshareUserNamespace => "unshare",
            ProbeCall::OpenSetgroups => "open",
            ProbeCall::WriteSetgroups => "write",
        };
    }

    /// What:
    ///  The call a word the child wrote names,
    ///  where this build knows it.
    /// Why:
    ///  The parent reads only what this build can act on,
    ///  and an unknown word is said rather than read as a pass.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const call = WORDS[word] ?? undefined;
    /// ```
    fn read_word(word: &str) -> Option<ProbeCall> {
        return match word {
            "unshare" => Some(ProbeCall::UnshareUserNamespace),
            "open" => Some(ProbeCall::OpenSetgroups),
            "write" => Some(ProbeCall::WriteSetgroups),
            _other => None,
        };
    }

    /// What:
    ///  The name a person reads for this call,
    ///  which is the call the companion's own line names.
    /// Why:
    ///  The two lines describe one call,
    ///  so the report must spell it as the add's line does
    ///  (`account_companion/companion_calls.rs`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const shown = { unshare: 'unshare(CLONE_NEWUSER)', open: 'open /proc/self/setgroups', write: 'write /proc/self/setgroups' }[call];
    /// ```
    pub fn name(self) -> &'static str {
        return match self {
            ProbeCall::UnshareUserNamespace => "unshare(CLONE_NEWUSER)",
            ProbeCall::OpenSetgroups => "open /proc/self/setgroups",
            ProbeCall::WriteSetgroups => "write /proc/self/setgroups",
        };
    }
}

/// What:
///  Write one failed call and the number it answered over the pipe.
/// Why:
///  The parent judges an answer,
///  not an exit status,
///  so the failing call and its errno travel as text.
///  Nothing here allocates:
///  after a `fork` only signal-safe work may run,
///  and a heap allocation is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeBytes(fd, `${call.word()} ${errno}`);
/// ```
fn write_answer(
    status: libc::c_int,
    call: ProbeCall,
    errno: i32,
) {
    // What: a stack buffer for the answer, and a cursor into it.
    // Why: the child may not allocate, so the decimal text is built by hand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text: number[] = [];
    // ```
    let mut text: [u8; ANSWER_BYTES] = [0; ANSWER_BYTES];
    let mut at: usize = 0;
    for byte in call.word().as_bytes() {
        text[at] = *byte;
        at += 1;
    }
    text[at] = b' ';
    at += 1;
    at += decimal_into(errno, &mut text[at..]);
    write_bytes(status, &text[..at]);
}

/// What:
///  A number as decimal digits,
///  written into the end of `into`,
///  answering how many bytes it took.
/// Why:
///  The child writes an errno without allocating,
///  and an errno is never negative here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const digits = String(value); into.set(digits); return digits.length;
/// ```
fn decimal_into(
    value: i32,
    into: &mut [u8],
) -> usize {
    // What: the value as an unsigned number, so the division below needs no sign.
    // Why: every errno is a small positive number.
    let mut left: u32 = u32::try_from(value).unwrap_or(0);
    // What: the digits are found from the right and written there, which is the order they read in.
    // Why: dividing by ten gives them right to left, and the buffer is written once.
    let mut digits: [u8; 10] = [0; 10];
    let mut taken: usize = 0;
    loop {
        let next: u32 = left / 10;
        digits[taken] = b'0' + (left % 10) as u8;
        taken += 1;
        left = next;
        if left == 0 {
            break;
        }
    }
    for index in 0..taken {
        into[index] = digits[taken - 1 - index];
    }
    return taken;
}

/// What:
///  Write every byte of `bytes` to `status`.
/// Why:
///  A short write would leave the parent reading half an answer,
///  which it would report as unread rather than as this host's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let at = 0; at < bytes.length; ) { at += write(status, bytes.subarray(at)); }
/// ```
fn write_bytes(
    status: libc::c_int,
    bytes: &[u8],
) {
    let mut at: usize = 0;
    while at < bytes.len() {
        let wrote: libc::ssize_t =
            unsafe { libc::write(status, bytes[at..].as_ptr().cast(), bytes.len() - at) };
        if wrote <= 0 {
            return;
        }
        at += wrote as usize;
    }
}

/// What:
///  The answer pipe,
///  with both ends closed when the run ends and neither inherited by a program
///  the child could start.
/// Why:
///  A pipe is the smallest thing that carries one small answer from a child to
///  its parent,
///  and `O_CLOEXEC` keeps it out of any program either side starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ends = pipe(); return ends instanceof Error ? ends.errno : ends;
/// ```
fn pipe_pair() -> Result<(OwnedFd, OwnedFd), i32> {
    // What: the two ends the kernel fills in, and the call that fills them.
    // Why: `pipe2` takes its flags at creation, so no window holds an inheritable end.
    let mut ends: [libc::c_int; 2] = [0; 2];
    let made: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    if made != 0 {
        return Err(errno_now());
    }
    // What: the reading end, owned from here on, and the writing end beside it.
    // Why: `OwnedFd` closes its number when it is dropped, on every path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [new OwnedFd(ends[0]), new OwnedFd(ends[1])];
    // ```
    let reading: OwnedFd = unsafe { OwnedFd::from_raw_fd(ends[0]) };
    let writing: OwnedFd = unsafe { OwnedFd::from_raw_fd(ends[1]) };
    return Ok((reading, writing));
}

/// What:
///  One `fork`,
///  answering the child's number to the parent,
///  `None` in the child,
///  and the failure number where the system refused it.
/// Why:
///  A user namespace needs a process of its own,
///  because `unshare(CLONE_NEWUSER)` is refused in one with more than one
///  thread and the doctor runs on a runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pid = fork(); if (pid < 0) return { errno: errno() }; return pid === 0 ? undefined : pid;
/// ```
fn fork_once() -> Result<Option<libc::pid_t>, i32> {
    let child: libc::pid_t = unsafe { libc::fork() };
    if child < 0 {
        return Err(errno_now());
    }
    if child == 0 {
        return Ok(None);
    }
    return Ok(Some(child));
}

/// What:
///  The number the call just made answered.
/// Why:
///  It must be read before any other call,
///  because a library function between them can replace it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const errno = theLastCallFailed() ? errnoNumber() : 0;
/// ```
fn errno_now() -> i32 {
    return std::io::Error::last_os_error()
        .raw_os_error()
        .unwrap_or(libc::EIO);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "group_drop_probe_tests.rs"]
mod tests;
