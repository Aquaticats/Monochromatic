//! What:     The "say that this is a shadow" feature: when a shadowed tool is started with exactly one argument that
//!           asks for its version or its help, the real tool runs as usual and the shadow then adds one line (version) or
//!           one short section (help) that names the shadow, its version and the real binary's path.
//! Why:      A shadow is invisible by design, so `rg --version` printed only ripgrep's version and nobody could tell that
//!           `rg` was a shadow. The convention follows the `hub` wrapper, which prints git's version line and then its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // announce.ts
//! export function announcementFor(name: string, args: string[]): 'version' | 'help' | undefined { ... }
//! export function runAnnounced(name: string, real: string, args: string[], kind: 'version' | 'help'): number { ... }
//! ```

/// Imports the owned operating-system text type, the shape of an argument that may not be valid UTF-8.
use std::ffi::OsString;

/// Imports `Write` (the trait that adds `write_all` and `flush` to standard output).
use std::io::{self, Write};

/// Imports the Unix-only trait that adds `.signal()` to an exit status, so a tool killed by a signal gets the shell's
/// `128 + signal` exit code.
use std::os::unix::process::ExitStatusExt;

/// Imports `Path` (a borrowed read-only path).
use std::path::{self, Path};

/// Imports `Command` (the builder that starts the real tool) and `ExitStatus` (how it ended).
use std::process::{Command, ExitStatus};

use crate::constants::{ALLOW_WIDE_VARIABLE, EXEC_FAILED_EXIT_CODE};

/// What:     `pub enum Announcement { Version, Help }` declares a type whose value is exactly one of the two listed
///           variants (a TS union of string literals).
/// Why:      The two requests add different text, so the caller says which one it saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Announcement = 'version' | 'help';
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Announcement {
    /// The tool was asked for its version; the shadow adds one line.
    Version,
    /// The tool was asked for its help; the shadow adds one section.
    Help,
}

/// What:     `pub struct AnnouncedFlags { ... }` declares a record of two lists of borrowed texts that live for the whole
///           program (`&'static`).
/// Why:      Each tool spells its version and help flags differently: `-h` is help for `rg` but "human-readable" for
///           `du`, and `-V` is version for `rg` but an error for `find`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface AnnouncedFlags { version: readonly string[]; help: readonly string[] }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnnouncedFlags {
    /// The single arguments that make the tool print its version.
    pub version: &'static [&'static str],
    /// The single arguments that make the tool print its help.
    pub help: &'static [&'static str],
}

/// What:     `pub fn announced_flags(name: &str) -> Option<AnnouncedFlags>` declares a public function that returns the
///           flags of one shadowed tool, or nothing for a name that is not shadowed.
/// Why:      Each row was read off the real tool's own `--version`, `--help` and short flags (ripgrep, GNU grep, ugrep, ag,
///           ack, GNU find, fd, bfs, tree, GNU du, ncdu, tokei, cloc), so a short flag that means something else in one
///           tool (`du -h`, `tree -h`, `grep -h`, `ack -h`) is never taken for help.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function announcedFlags(name: string): AnnouncedFlags | undefined { switch (name) { ... } }
/// ```
pub fn announced_flags(name: &str) -> Option<AnnouncedFlags> {
    // What:     `match name { "rg" | "fd" => ..., _ => ... }` picks the branch whose pattern equals `name`; `|` joins
    //           alternatives and `_` is the catch-all.
    // Why:      One row per group of tools that share a spelling.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (name) { case 'rg': case 'fd': return {...}; default: return undefined; }
    // ```
    return match name {
        "rg" | "fd" | "ag" | "tokei" => Some(AnnouncedFlags {
            version: &["--version", "-V"],
            help: &["--help", "-h"],
        }),
        "ncdu" => Some(AnnouncedFlags {
            version: &["--version", "-V"],
            help: &["--help", "-h", "-?"],
        }),
        "grep" | "egrep" | "fgrep" => Some(AnnouncedFlags {
            version: &["--version", "-V"],
            help: &["--help"],
        }),
        "ugrep" | "ug" => Some(AnnouncedFlags {
            version: &["--version", "-V"],
            help: &["--help", "-?"],
        }),
        "find" | "bfs" => Some(AnnouncedFlags {
            version: &["--version", "-version"],
            help: &["--help", "-help"],
        }),
        "ack" | "tree" | "du" | "cloc" => Some(AnnouncedFlags {
            version: &["--version"],
            help: &["--help"],
        }),
        _ => None,
    };
}

/// What:     `pub fn announcement_for(name: &str, arguments: &[OsString]) -> Option<Announcement>` declares a public
///           function that decides whether this run asks for a version or a help.
/// Why:      Only a top-level request counts: the whole argument list is exactly one of the tool's version or help
///           flags. Anything else (`rg --version foo`, `grep -h x`, a pattern, no arguments) runs untouched, because only
///           the lone flag is certainly a request for the tool itself and not for a sub-feature of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function announcementFor(name: string, args: Buffer[]): Announcement | undefined {
///   const flags = announcedFlags(name);
///   if (flags === undefined || args.length !== 1) return undefined;
///   ...
/// }
/// ```
pub fn announcement_for(name: &str, arguments: &[OsString]) -> Option<Announcement> {
    let flags: AnnouncedFlags = announced_flags(name)?;
    if arguments.len() != 1 {
        return None;
    }
    // `arguments[0] == *flag` compares an operating-system text with a `&str` byte for byte, so an argument that is
    // not valid UTF-8 never equals a flag.
    let only: &OsString = &arguments[0];
    for flag in flags.version {
        if *only == **flag {
            return Some(Announcement::Version);
        }
    }
    for flag in flags.help {
        if *only == **flag {
            return Some(Announcement::Help);
        }
    }
    return None;
}

/// What:     `fn absolute(real: &Path) -> String` declares a private function returning the real binary's path as text,
///           made absolute without following symlinks.
/// Why:      A relative PATH entry would otherwise print a path that means something else from another directory. The
///           symlinks stay unresolved so the line shows the PATH entry that won, which is what `command -v` would show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const absolute = (real: string) => path.resolve(real);
/// ```
fn absolute(real: &Path) -> String {
    let resolved = path::absolute(real).unwrap_or_else(|_| real.to_path_buf());
    return resolved.display().to_string();
}

/// What:     `pub fn version_line(name: &str, real: &Path) -> String` declares a public function returning the one line
///           the shadow adds after the real tool's version output, newline included.
/// Why:      The exact text is part of the contract (`search-shadow <version>, shadowing <tool> at <path>`), so tests and
///           the README can quote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const versionLine = (name: string, real: string) => `search-shadow ${version}, shadowing ${name} at ${real}\n`;
/// ```
pub fn version_line(name: &str, real: &Path) -> String {
    return format!(
        "search-shadow {}, shadowing {name} at {}\n",
        env!("CARGO_PKG_VERSION"),
        absolute(real),
    );
}

/// What:     `fn behaviour_lines(name: &str) -> String` declares a private function returning the bullet lines that say
///           what the shadow adds for this tool.
/// Why:      The help section must be honest per tool: a `tree` user is not told about `rg -r`. The groups mirror
///           `refusal_for` in `policy.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function behaviourLines(name: string): string { ... }
/// ```
fn behaviour_lines(name: &str) -> String {
    let escape: String = format!("unless {ALLOW_WIDE_VARIABLE}=1 is set for one deliberate walk");
    return match name {
        "rg" => format!(
            "- refuses the short flag -r anywhere (in ripgrep it is --replace, not grep's recursive flag)\n\
             - refuses --hidden, --no-ignore*, -u or -. with no search path and no piped input from a git repository root, {escape}\n",
        ),
        "grep" | "egrep" | "fgrep" => format!(
            "- refuses a recursive search with no path or a wide path (a git repository root, the home directory or /), {escape}\n",
        ),
        "find" | "bfs" => format!(
            "- refuses a start at a wide path (a git repository root, the home directory or /) without -maxdepth 3 or less, {escape}\n",
        ),
        _ => String::from("- never refuses a run; it only lowers the disk priority\n"),
    };
}

/// What:     `pub fn help_section(name: &str, real: &Path) -> String` declares a public function returning the section
///           the shadow adds after the real tool's help output: a blank separator line, a header line in the same shape as
///           the version line, and the bullet lines.
/// Why:      A reader of `rg --help` learns that the tool is a shadow, which version, where the real tool is and what the
///           shadow adds, without leaving the page.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function helpSection(name: string, real: string): string { ... }
/// ```
pub fn help_section(name: &str, real: &Path) -> String {
    return format!(
        "\n{}The help above is the real {name}'s own. search-shadow starts it at best-effort disk priority 7 and:\n{}Run `search-shadow --help` for the shadow itself.\n",
        version_line(name, real),
        behaviour_lines(name),
    );
}

/// What:     `fn exit_code_of(status: ExitStatus) -> i32` declares a private function turning how a process ended into
///           the shell's exit code.
/// Why:      The shadow must report the real tool's status: its own code, or `128 + signal` when a signal ended it (the
///           shell convention, and what a shell reports for the same process).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exitCodeOf = (r: SpawnSyncReturns<Buffer>) => r.status ?? 128 + signalNumber(r.signal);
/// ```
fn exit_code_of(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    if let Some(signal) = status.signal() {
        return 128 + signal;
    }
    return 1;
}

/// What:     `pub fn run_announced(...) -> i32` declares a public function that runs the real tool as a child with the
///           shadow's standard streams, waits for it, then appends the version line or the help section to standard output.
/// Why:      Replacing the process (as the normal path does) leaves no code to run afterward, so these two rare requests
///           pay for one extra process to get the last word. The child's output reaches the same descriptors untouched, so
///           the standard output is byte for byte the real tool's plus the added text. The added text is written whatever
///           the status, since some tools exit non-zero on a plain help request (`ugrep --help` exits 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runAnnounced(name, real, args, kind): number {
///   const result = spawnSync(real, args, { stdio: 'inherit' });
///   process.stdout.write(kind === 'version' ? versionLine(name, real) : helpSection(name, real));
///   return result.status ?? 128 + signal;
/// }
/// ```
pub fn run_announced(name: &str, real: &Path, arguments: &[OsString], announcement: Announcement) -> i32 {
    // What:     `Command::new(real).args(arguments).status()` starts the child (standard streams are inherited by
    //           default for `.status()`), waits and returns a `Result` of how it ended. `match` picks the branch.
    // Why:      On a start failure the message and exit code are the same as for the normal path's failed launch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { status = spawnSync(real, args, { stdio: 'inherit' }); } catch (error) { console.error(...); return 126; }
    // ```
    let status: ExitStatus = match Command::new(real).args(arguments).status() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("search-shadow: could not start {}: {error}", real.display());
            return EXEC_FAILED_EXIT_CODE;
        }
    };

    let added: String = match announcement {
        Announcement::Version => version_line(name, real),
        Announcement::Help => help_section(name, real),
    };

    // What:     `let _ = ...` deliberately drops the `Result` of each write.
    // Why:      A reader that closed the pipe early (`rg --help | head`) must not turn a finished run into a failure or a
    //           panic; the exit code stays the real tool's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { process.stdout.write(added); } catch {}
    // ```
    let mut standard_output = io::stdout().lock();
    let _ = standard_output.write_all(added.as_bytes());
    let _ = standard_output.flush();
    return exit_code_of(status);
}
