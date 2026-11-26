//! What:
//!  The child that enters one mount namespace and runs one piece of the act
//!  there,
//!  and the report it hands back to the handler.
//! Why:
//!  `setns(CLONE_NEWNS)` from a thread of a multithreaded process is refused
//!  `EINVAL`,
//!  and the same call from a single-threaded process succeeds
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  F15,
//!  and "The preflight"),
//!  so the act forks one single-threaded child per namespace,
//!  enters it there,
//!  and reads or changes that namespace's mounts from inside.
//!  A child that outlived its handler would hold a namespace and every mount in
//!  it,
//!  so it asks the kernel to end it with its parent and rechecks that parent
//!  before it acts,
//!  and the handler waits for it on every path.
//!  The report is one bounded text:
//!  what the act came to,
//!  the errno that stopped it,
//!  one line naming the call,
//!  and the mount table the child read after the act.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = runInNamespace(namespace, () => takeTheCopyOut(...));  // the child's own answer
//! ```

use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};

/// The most bytes one report's header takes,
/// its detail line included.
pub const MAXIMUM_HEADER_BYTES: usize = 4096;

/// The most bytes of mount table one report carries,
/// which is the crate's own mount-table bound (`plain_file::MOUNT_TABLE_LIMIT`,
/// four mebibytes).
const MAXIMUM_TABLE_BYTES: usize = 4 << 20;

/// What:
///  How one child's piece of the act ended.
/// Why:
///  A refusal the design names,
///  a call the kernel failed,
///  and an act that finished are three answers the handler reports
///  differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ActOutcome = 'done' | 'refused' | 'failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActOutcome {
    /// The child did what it was asked and its own checks passed.
    Done,
    /// The design refused the act here,
    /// as a covered member or a locked copy is refused.
    Refused,
    /// A call the act made answered an error.
    Failed,
}

impl ActOutcome {
    /// The word the report carries.
    pub fn word(self) -> &'static str {
        return match self {
            ActOutcome::Done => "done",
            ActOutcome::Refused => "refused",
            ActOutcome::Failed => "failed",
        };
    }

    /// The outcome one report word names.
    pub fn parse(word: &str) -> Option<ActOutcome> {
        return match word {
            "done" => Some(ActOutcome::Done),
            "refused" => Some(ActOutcome::Refused),
            "failed" => Some(ActOutcome::Failed),
            _ => None,
        };
    }
}

/// What:
///  One child's whole answer:
///  the outcome,
///  the errno that stopped it,
///  one line naming the call,
///  and the table it read after the act.
/// Why:
///  The handler judges an act by what the child saw inside the namespace,
///  and a line that named only "failed" would leave the report unable to say
///  what refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ActReport = { outcome: ActOutcome; errno?: number; detail: string; table: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActReport {
    /// What the act came to.
    pub outcome: ActOutcome,
    /// The errno the stopping call answered.
    pub errno: Option<i32>,
    /// One line naming the call and what it was asked,
    /// with no newline of its own.
    pub detail: String,
    /// `/proc/self/mountinfo` as the child read it after the act.
    pub table: String,
}

impl ActReport {
    /// An act that finished.
    pub fn done(detail: String, table: String) -> ActReport {
        return ActReport { outcome: ActOutcome::Done, errno: None, detail, table };
    }

    /// An act a call stopped,
    /// with the errno it answered.
    pub fn failed(detail: String, errno: i32, table: String) -> ActReport {
        return ActReport { outcome: ActOutcome::Failed, errno: Some(errno), detail, table };
    }

    /// An act the design refused here.
    pub fn refused(detail: String, table: String) -> ActReport {
        return ActReport { outcome: ActOutcome::Refused, errno: None, detail, table };
    }

    /// What:
    ///  The report as one bounded text.
    /// Why:
    ///  A pipe carries bytes and the parent must find the fields without
    ///  guessing,
    ///  so the header is three known lines and the table follows one length.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = `outcome ${outcome}\nerrno ${errno}\ndetail ${escaped}\ntable ${bytes}\n${table}`;
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        let errno: String = match self.errno {
            Some(number) => number.to_string(),
            None => String::from("none"),
        };
        let mut escaped: String = String::with_capacity(self.detail.len());
        for character in self.detail.chars() {
            if character == '\\' {
                escaped.push_str("\\\\");
            } else if character == '\n' {
                escaped.push_str("\\n");
            } else {
                escaped.push(character);
            }
        }
        let header: String = format!("outcome {}\nerrno {errno}\ndetail {escaped}\ntable {}\n", self.outcome.word(), self.table.len());
        let mut bytes: Vec<u8> = Vec::with_capacity(header.len() + self.table.len());
        bytes.extend_from_slice(header.as_bytes());
        bytes.extend_from_slice(self.table.as_bytes());
        return bytes;
    }

    /// What:
    ///  Read one report,
    ///  or say which field is not one this build writes.
    /// Why:
    ///  A truncated or rewritten report must be refused by name rather than
    ///  read as an act that finished.
    ///
    /// # Errors
    /// A line is absent,
    /// the outcome is not a word this build writes,
    /// the table's length is not a number,
    /// or the bytes do not carry the table the length names.
    pub fn decode(bytes: &[u8]) -> Result<ActReport, String> {
        let mut at: usize = 0;
        let mut outcome: Option<ActOutcome> = None;
        let mut errno: Option<i32> = None;
        let mut detail: Option<String> = None;
        let mut table_bytes: Option<usize> = None;
        for _line in 0..4 {
            let rest: &[u8] = &bytes[at..];
            let Some(end) = rest.iter().position(|byte| return *byte == b'\n') else {
                return Err(String::from("the report's header ended before its four lines did"));
            };
            let Ok(line) = std::str::from_utf8(&rest[..end]) else {
                return Err(String::from("a header line is not text"));
            };
            at += end + 1;
            if let Some(word) = line.strip_prefix("outcome ") {
                outcome = ActOutcome::parse(word);
                if outcome.is_none() {
                    return Err(format!("the outcome `{word}` is not one this build writes"));
                }
            } else if let Some(number) = line.strip_prefix("errno ") {
                if number == "none" {
                    errno = None;
                } else {
                    errno = Some(number.parse::<i32>().map_err(|_error| return String::from("the errno is not a number"))?);
                }
            } else if let Some(text) = line.strip_prefix("detail ") {
                let mut plain: String = String::with_capacity(text.len());
                let mut characters = text.chars();
                while let Some(character) = characters.next() {
                    if character != '\\' {
                        plain.push(character);
                        continue;
                    }
                    match characters.next() {
                        Some('n') => plain.push('\n'),
                        Some('\\') => plain.push('\\'),
                        _ => return Err(String::from("the detail line's escape is not one this build writes")),
                    }
                }
                detail = Some(plain);
            } else if let Some(number) = line.strip_prefix("table ") {
                table_bytes = Some(number.parse::<usize>().map_err(|_error| return String::from("the table's length is not a number"))?);
            } else {
                return Err(format!("the header line `{line}` is not one this build writes"));
            }
        }
        let (Some(answered), Some(named), Some(wanted)) = (outcome, detail, table_bytes) else {
            return Err(String::from("the report's header lacks one of its four lines"));
        };
        if wanted > MAXIMUM_TABLE_BYTES {
            return Err(String::from("the table is longer than the crate's own bound"));
        }
        let table: &[u8] = &bytes[at..];
        if table.len() != wanted {
            return Err(format!("the report carries {} table bytes and its length says {wanted}", table.len()));
        }
        let Ok(text) = std::str::from_utf8(table) else {
            return Err(String::from("the table is not text"));
        };
        return Ok(ActReport { outcome: answered, errno, detail: named, table: String::from(text) });
    }
}

/// What:
///  Why no child's report came back.
/// Why:
///  A fork the system refused,
///  a pipe,
///  a wait and a report that is not one this build writes are four different
///  failures,
///  and the act must never read a missing report as a finished one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildRefusal = { calling: string; errno?: number; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildRefusal {
    /// Which call refused.
    pub calling: &'static str,
    /// The errno the call answered,
    /// where it answered one.
    pub errno: Option<i32>,
    /// What it said,
    /// in plain words.
    pub said: String,
}

impl std::fmt::Display for ChildRefusal {
    /// One sentence naming the call, the errno and what it said.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self.errno {
            Some(errno) => write!(formatter, "{} answered errno {errno}: {}", self.calling, self.said),
            None => write!(formatter, "{} could not be made: {}", self.calling, self.said),
        };
    }
}

/// What:
///  Enter one mount namespace in a child of this process and run one piece of
///  the act there.
/// Why:
///  The child is single-threaded,
///  which is the only state the kernel lets `setns` change the mount namespace
///  in,
///  and the handler waits for it on every path so no namespace outlives the
///  act.
///
/// # Errors
/// The pipe or the fork was refused,
/// the child's wait failed,
/// the child died before it reported,
/// or its report is not one this build writes.
pub fn run_in<Work>(namespace: &std::fs::File, work: Work) -> Result<ActReport, ChildRefusal>
where
    Work: Fn() -> ActReport,
{
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: `ends` is a live two-element array the call fills with the two descriptors it makes.
    let piped: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    if piped != 0 {
        return Err(ChildRefusal { calling: "pipe2", errno: std::io::Error::last_os_error().raw_os_error(), said: String::from("a report pipe") });
    }
    let read_end: RawFd = ends[0];
    let write_end: RawFd = ends[1];
    // SAFETY: `fork` takes no argument and this call makes the child below run on its own.
    let child: libc::pid_t = unsafe { libc::fork() };
    if child < 0 {
        close(read_end);
        close(write_end);
        return Err(ChildRefusal { calling: "fork", errno: std::io::Error::last_os_error().raw_os_error(), said: String::from("one child per namespace") });
    }
    if child == 0 {
        close(read_end);
        enters_the_namespace(namespace.as_raw_fd(), write_end);
        let report: ActReport = work();
        let bytes: Vec<u8> = report.encode();
        write_all(write_end, bytes.as_slice());
        close(write_end);
        end_now(0);
    }
    close(write_end);
    let mut read: Vec<u8> = Vec::new();
    let bound: u64 = u64::try_from(MAXIMUM_HEADER_BYTES + MAXIMUM_TABLE_BYTES + 1).unwrap_or(u64::MAX);
    // SAFETY: the descriptor is this call's own read end and nothing else reads it.
    let mut pipe: std::fs::File = unsafe { std::fs::File::from_raw_fd(read_end) };
    let read_result: std::io::Result<usize> = Read::by_ref(&mut pipe).take(bound).read_to_end(&mut read);
    drop(pipe);
    let waited: Result<(), ChildRefusal> = wait_for(child);
    read_result.map_err(|error| return ChildRefusal {
        calling: "read",
        errno: error.raw_os_error(),
        said: String::from("the child's report"),
    })?;
    waited?;
    if read.len() > MAXIMUM_HEADER_BYTES + MAXIMUM_TABLE_BYTES {
        return Err(ChildRefusal { calling: "read", errno: None, said: String::from("the child's report is longer than this build writes") });
    }
    return ActReport::decode(read.as_slice()).map_err(|said| return ChildRefusal { calling: "report", errno: None, said });
}

/// What:
///  Make this forked child tied to its parent's life,
///  and enter the namespace,
///  reporting the refusal through the pipe and ending when the kernel refuses.
/// Why:
///  A child that survived its handler would hold the namespace it entered and
///  every mount in there,
///  so the kernel ends it with its parent and the recheck catches a parent that
///  died before the request was made,
///  and a `setns` the kernel refuses must reach the handler as a report rather
///  than as a silent child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prctl(PR_SET_PDEATHSIG, SIGKILL); if (getppid() === 1) _exit(1);
/// if (setns(namespaceFd, CLONE_NEWNS) !== 0) { write(report, failure(errno)); _exit(1); }
/// ```
fn enters_the_namespace(namespace: RawFd, report: RawFd) {
    // SAFETY: `prctl` reads its four words and touches no memory of ours.
    let _set: libc::c_int = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) };
    // SAFETY: `getppid` takes no argument and reads one number.
    if unsafe { libc::getppid() } == 1 {
        end_now(1);
    }
    // SAFETY: the descriptor is inherited from the handler and stays open in this child; no memory of ours is read.
    let entered: libc::c_int = unsafe { libc::setns(namespace, libc::CLONE_NEWNS) };
    if entered == 0 {
        return;
    }
    let errno: i32 = std::io::Error::last_os_error().raw_os_error().unwrap_or(libc::EINVAL);
    let refused: ActReport = ActReport::failed(format!("setns into the mount namespace answered errno {errno}"), errno, String::new());
    let bytes: Vec<u8> = refused.encode();
    write_all(report, bytes.as_slice());
    // The report is the answer, so the child ends as one that finished; a child that dies without a report is what
    // the handler reads as a child that never answered.
    end_now(0);
}

/// What:
///  Close one descriptor this call owns.
/// Why:
///  Each end of the pipe is closed on the side that does not read or write it,
///  so the reader sees the writer's end of file.
fn close(descriptor: RawFd) {
    // SAFETY: the descriptor is this call's own and is closed exactly once.
    let _closed: libc::c_int = unsafe { libc::close(descriptor) };
}

/// What:
///  Write every byte of one report to the pipe,
///  and say nothing when the reader has gone.
/// Why:
///  A handler that died closes its read end,
///  and the child's own end is then the one thing it must not wait on.
fn write_all(descriptor: RawFd, mut bytes: &[u8]) {
    while !bytes.is_empty() {
        // SAFETY: the bytes are this call's own and the descriptor is the child's write end.
        let written: libc::ssize_t = unsafe { libc::write(descriptor, bytes.as_ptr().cast::<libc::c_void>(), bytes.len()) };
        if written <= 0 {
            return;
        }
        bytes = &bytes[usize::try_from(written).unwrap_or(0)..];
    }
}

/// What:
///  End this child at once,
///  never through the handler's frames.
/// Why:
///  A child that returned would run a second copy of the handler,
///  and `_exit` makes no call the child may not make after a fork.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exit(status) // without running any cleanup of the parent's
/// ```
fn end_now(status: libc::c_int) -> ! {
    // SAFETY: `_exit` ends this process at once and takes no argument but its status.
    unsafe { libc::_exit(status) }
}

/// What:
///  Wait for the child and refuse when it did not end cleanly.
/// Why:
///  A child killed by a signal or exiting nonzero has left a mount half changed,
///  and its report must not be read as an act that finished.
fn wait_for(child: libc::pid_t) -> Result<(), ChildRefusal> {
    let mut status: libc::c_int = 0;
    loop {
        // SAFETY: the status word is this call's own and `waitpid` writes it.
        let waited: libc::pid_t = unsafe { libc::waitpid(child, &mut status, 0) };
        if waited < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(ChildRefusal { calling: "waitpid", errno: error.raw_os_error(), said: String::from("the child that entered a namespace") });
        }
        break;
    }
    let exited: bool = libc::WIFEXITED(status);
    let status_of_exit: libc::c_int = libc::WEXITSTATUS(status);
    if exited && status_of_exit == 0 {
        return Ok(());
    }
    return Err(ChildRefusal {
        calling: "waitpid",
        errno: None,
        said: format!("the child that entered a namespace ended with status {status} (exited {exited}, status {status_of_exit})"),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_namespace_child_tests.rs"]
mod tests;
