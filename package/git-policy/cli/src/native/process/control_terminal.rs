//! What:
//!  Whether this process has a controlling terminal,
//! answered by the kernel through `/dev/tty`.
//! Why:
//!  Only a command with a controlling terminal may have a person at it,
//! and only such a command keeps Git's own prompting.
//! A process without one,
//! like the daemon that started the push of
//!  2026-11-10 (`ps -o tty=` printed `?`),
//! has nobody to answer a login question,
//! and Git's and git-lfs's prompt programs
//!  open a desktop window for nobody instead
//! (`doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`).
//!  A terminal this process merely inherits as a descriptor is not asked about:
//! the incident's wrapper had its standard input at `/dev/null` and still
//!  inherited `DISPLAY`,
//! so "is standard input a terminal" answers the wrong question.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const terminal = fs.openSync('/dev/tty', 'r+') // throws ENXIO when this process has none
//! ```

/// What:
///  The two answers,
/// named so a call site reads as a decision:
///  a person may be able to answer (`Present`),
/// or nobody can (`Absent`).
///  The siblings of an enum here are a `bool`,
/// which says nothing about what it measured,
///  and an `Option<Path>` of the terminal,
/// which no caller needs.
/// Why:
///   The overlay of a command the caller typed differs by this fact alone,
/// so it is carried as a value a control can hand in
///  without a terminal of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Terminal = 'present' | 'absent';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Terminal {
    /// The process has a controlling terminal,
    /// so a person may be at it.
    Present,
    /// The process has none,
    /// so no question reaches a person.
    Absent,
}

/// What:
///  Whether this process has a controlling terminal,
/// read from the kernel's own `/dev/tty`.
/// Why:
///   Opening `/dev/tty` opens the caller's controlling terminal and fails with `ENXIO` when the caller has none,
///  which is the definition itself;
/// no descriptor of the process is examined,
/// so a command whose input was redirected is
///  still read as typed at a terminal.
/// The descriptor is closed again by the drop of the file,
/// and an open that finds no terminal changes nothing of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasControllingTerminal(): boolean { try { fs.closeSync(fs.openSync('/dev/tty', 'r+')); return true } catch { return false } }
/// ```
pub fn has_controlling_terminal() -> bool {
    // `open` with both access modes is the call Git 2.56.0 makes in `git_terminal_prompt` (`prompt.c`,
    //  `open("/dev/tty", O_RDWR)`), so this answers whether Git could ask on this terminal at all: one it cannot open for
    //  writing is one Git's own prompt cannot use either. The file closes at once, and no byte is read or written, so nothing
    //  of a person's terminal changes.
    match std::fs::OpenOptions::new().read(true).write(true).open("/dev/tty") {
        // `Ok(_file)` is the "present" variant: the kernel named a controlling terminal.
        Ok(_file) => return true,
        // `Err(_)` is the "absent" variant: there is none, or this system has no `/dev/tty` at all.
        Err(_) => return false,
    }
}

/// What:
///  The same question as a value a caller can pass on.
/// Why:
///   One read of the kernel's answer per invocation,
/// and every Git child of that command is decided from the one reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const terminal: Terminal = hasControllingTerminal() ? 'present' : 'absent';
/// ```
pub fn control_terminal() -> Terminal {
    if has_controlling_terminal() {
        // The kernel named a controlling terminal.
        return Terminal::Present;
    }
    // No controlling terminal, so nothing can reach a person.
    return Terminal::Absent;
}

/// The probe is controlled in the test module.
#[cfg(test)]
#[path = "control_terminal_tests.rs"]
mod tests;
