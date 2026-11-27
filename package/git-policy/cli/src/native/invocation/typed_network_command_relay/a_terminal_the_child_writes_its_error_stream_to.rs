//! What:
//!  The pseudo-terminal a typed network command's standard error is written to,
//! when the wrapper's own standard error is one.
//! Why:
//!  Git,
//! OpenSSH and `git-lfs` draw only when `isatty(2)` is true,
//! and a pipe answers no;
//!  the terminal is opened without becoming this process's controlling terminal and without a new session,
//!  its output processing is cleared so a captured transcript is plain Git's byte for byte,
//!  and it is given the person's window size for the programs that ask their own standard error's terminal
//!  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`,
//! "What a shipped K,
//! K2 or C must get
//!  right").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // openATerminalPair(): [master, slave]; slave has OPOST cleared and the person's window size
//! ```

// What: `AsFd` is a borrow of one open descriptor, which the flag calls take.
// Why:  The terminal's settings are read and written through its descriptor.
//
// In TS you'd write (pseudocode):
// ```ts
// type AsFd = { fd: number };
// ```
// What: `OsStrExt` adds the constructor that reads raw operating-system bytes as a path; without it `OsStr` has no
//       `from_bytes`.
// Why:  The kernel hands the slave's path back as bytes, and those bytes are the path.
//
// In TS you'd write (pseudocode):
// ```ts
// const path = Buffer.from(bytes);
// ```
use std::os::unix::ffi::OsStrExt;
// What: `OwnedFd` is one open descriptor whose closing is the value's own doing.
// Why:  Both ends of the terminal are opened here, and the caller closes each by handing it on.
//
// In TS you'd write (pseudocode):
// ```ts
// type OwnedFd = number; // closed when the value is dropped
// ```
use std::os::fd::OwnedFd;
// What: `io` is the input and output error type the opens answer with.
use std::io;

/// What:
///  Open the two ends of a pseudo-terminal:
/// the master this process reads and the slave the child writes.
/// Why:
///   K2 gives the child a terminal on standard error only when the wrapper's own standard error is one.
///   The slave is opened without becoming this process's controlling terminal (`NOCTTY`),
///   so the person's terminal keeps signalling Git,
///   and no session is started:
/// the child stays in this process's foreground group
///   (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`,
/// K2,
/// and the proposal's "No `setsid`").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const master = openpt(); grantpt(master); unlockpt(master); const slave = open(ptsname(master), O_RDWR | O_NOCTTY);
/// ```
pub(super) fn open_a_terminal_pair() -> io::Result<(OwnedFd, OwnedFd)> {
    // What: `openpt` asks the kernel for a pseudo-terminal's master; `RDWR` is read and write, `NOCTTY` keeps it from becoming
    //       this process's terminal, and `CLOEXEC` keeps the child from inheriting the end this process reads.
    // Why:  A child that inherited the master would hold the relay's own end open and delay its end of file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const master = rustix.openpt(RDWR | NOCTTY | CLOEXEC);
    // ```
    let master: OwnedFd = rustix::pty::openpt(
        rustix::pty::OpenptFlags::RDWR
            | rustix::pty::OpenptFlags::NOCTTY
            | rustix::pty::OpenptFlags::CLOEXEC,
    )?;
    // On Linux the kernel needs nothing of `grantpt`; on the systems where it does, it decides who may open the slave.
    rustix::pty::grantpt(&master)?;
    rustix::pty::unlockpt(&master)?;
    // What: `CString` is a byte string that ends with a zero byte, which the kernel's path call requires; the sibling `String`
    //       carries no terminator and would reject bytes that are not UTF-8.
    // Why:  The slave's path comes back from the kernel as a terminated string.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const path = rustix.ptsname(master, Buffer.alloc(0));
    // ```
    let path: std::ffi::CString = rustix::pty::ptsname(&master, Vec::new())?;
    // What: `Path::new` reads the kernel's bytes as a path without decoding them.
    // Why:  A path handed to `open` must keep every byte the kernel gave.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const slavePath = os.str(path);
    // ```
    let slave_path: &std::path::Path =
        std::path::Path::new(std::ffi::OsStr::from_bytes(path.as_c_str().to_bytes()));
    let slave: OwnedFd = rustix::fs::open(
        slave_path,
        rustix::fs::OFlags::RDWR | rustix::fs::OFlags::NOCTTY,
        rustix::fs::Mode::empty(),
    )?;
    clear_output_processing(&slave);
    give_the_size_of_the_person_s_terminal(&slave);
    // What: `Ok((..))` is the success variant carrying both ends: master first, slave second.
    // Why:  The read takes one and the child writes to the other.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [master, slave];
    // ```
    return Ok((master, slave));
}

/// What:
///  Clear the terminal's output processing (`OPOST`) on the child's side.
/// Why:
///   Without this,
/// every line feed the child writes becomes a carriage return and a line feed,
///   so a captured transcript gains a byte on every line;
///   with it,
/// a refusal's transcript is plain Git's byte for byte
///   (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`,
/// "What a shipped K,
/// K2 or C must get
///   right").
///   A terminal whose settings cannot be read or written keeps them:
/// the child's stream is worth more than this byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { const t = tcgetattr(slave); t.output_modes &= ~OPOST; tcsetattr(slave, TCSANOW, t); } catch {}
/// ```
fn clear_output_processing(slave: &OwnedFd) {
    // What: `let Ok(..) = .. else { return; }` runs the rest only when the call named a terminal's settings.
    // Why:  A failure here must not stop the command; it only leaves the doubled line ends in place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const settings = tryTcgetattr(slave); if (!settings) return;
    // ```
    let Ok(mut settings) = rustix::termios::tcgetattr(slave) else {
        return;
    };
    // What: `remove` clears one bit of `output_modes`, which is the terminal's `c_oflag`.
    // Why:  `OPOST` is the bit that rewrites each line feed on its way out.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // settings.output_modes &= ~OPOST;
    // ```
    settings
        .output_modes
        .remove(rustix::termios::OutputModes::OPOST);
    // What: `Now` applies the settings at once; the siblings are `Drain`, which lets the output go first, and `Flush`.
    // Why:  The child has written nothing yet, so there is nothing to drain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // tcsetattr(slave, TCSANOW, settings);
    // ```
    let _applied: rustix::io::Result<()> =
        rustix::termios::tcsetattr(slave, rustix::termios::OptionalActions::Now, &settings);
}

/// What:
///  Give the child's terminal the size the person's own terminal has.
/// Why:
///   A program that asks its own standard error's terminal how wide it is gets an answer instead of nothing;
///   Git's progress width comes from its standard output,
/// which this leaves alone,
///   so the size changes no progress line and serves only those programs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { tcsetwinsize(slave, tcgetwinsize(2)); } catch {}
/// ```
fn give_the_size_of_the_person_s_terminal(slave: &OwnedFd) {
    // What: `Winsize` is a terminal window's row and column counts, plus the pixel counts nothing here sets.
    // Why:  Copying the person's window onto the child's terminal is what a program asking its own error stream expects.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const size = tryTcgetwinsize(2); if (size) tryTcsetwinsize(slave, size);
    // ```
    let Ok(size) = rustix::termios::tcgetwinsize(std::io::stderr()) else {
        return;
    };
    let _set: rustix::io::Result<()> = rustix::termios::tcsetwinsize(slave, size);
}
