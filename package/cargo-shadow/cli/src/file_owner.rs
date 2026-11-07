//! What:
//! The ownership evidence for a file that a child process (cargo) may have written:
//! a window of time around the child's run,
//! a read of the file through one open descriptor that is checked for stability,
//! and the rule that decides whether the file is this run's output.
//! Why:
//! The reviews of 2026-11-03 showed that reading a file after the child returned and calling that text "ours" is not evidence:
//! another writer may have published first,
//! and an unrelated file may have been recorded as ours.
//! The evidence used here is the inode change time (`ctime`) the kernel stamps on every write,
//! rename and link:
//! a file that differs from its snapshot and whose `ctime` lies inside the window of the child's run was changed while the child ran;
//! one whose `ctime` lies later was changed after the child exited,
//! so it is not the child's output and is never restored over.
//! `ctime` cannot be set by a writer (unlike `mtime`),
//! and it also moves when a file is renamed into place,
//! which is how editors and `git checkout` publish.
//! The remaining blind spot is a writer that publishes within about one scheduler tick after the child's exit,
//! or while the child runs;
//! that interval cannot be separated from the child's own write by any observation after the fact,
//! and the restore protocol
//! (`journal_replace.rs`) keeps every displaced file so the blind spot never destroys data.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // file-owner.ts
//! export type Window = { startNs: bigint; endNs: bigint };
//! export function timed<T>(run: () => T): [T, Window]
//! export function claimOutput(path: string, beforeText: string, window: Window): Claim  // throws on a read error
//! ```

/// What:
/// `use std::fs::File;` imports the open-file type.
/// Why:
/// The file is read through one open descriptor so its identity and change time can be checked before and after the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { openSync } from 'node:fs';
/// ```
use std::fs::File;
/// What:
/// `use std::io::Read;` imports the trait that adds `read_to_end` to an open file.
/// Why:
/// The bytes are read before they are checked to be UTF-8 text,
/// so a decoding failure is reported as an error and not as an empty file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node's readFileSync is a free function; no import is needed.
/// ```
use std::io::Read;
/// What:
/// `use std::io;` imports the file-system error module.
/// Why:
/// Reads report `io::Result`,
/// so a failure is never taken for an empty file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node; nothing is imported
/// ```
use std::io;
/// What:
/// `use std::os::unix::fs::MetadataExt;` imports the trait that adds `ino`,
/// `dev`,
/// `ctime` and `mode` to file metadata.
/// Why:
/// The inode number and the change time are the ownership evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs'; // stat.ino, stat.ctimeNs
/// ```
use std::os::unix::fs::MetadataExt;
/// What:
/// `use std::path::Path;` imports the borrowed path type (sibling:
/// `PathBuf`,
/// the owned one).
/// Why:
/// The paths are lent to the functions here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;
/// What:
/// `use std::time::{SystemTime, UNIX_EPOCH};` imports the wall clock and its zero.
/// Why:
/// The window is measured on the same clock the kernel uses for file change times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() * 1e6
/// ```
use std::time::{SystemTime, UNIX_EPOCH};

/// How long before the child starts a file change time still counts as the child's.
/// File times are stamped from a clock that can lag the wall clock by up to one scheduler tick,
/// so a very fast child's write can carry a time just before the start stamp.
/// Ten milliseconds is several ticks at any common tick rate (a chosen margin,
/// not a measured one).
pub const START_SLACK_NS: i128 = 10_000_000;

/// The interval of wall-clock time in which a child ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Window {
    /// The wall-clock time just before the child was started,
    /// minus `START_SLACK_NS`,
    /// in nanoseconds since the Unix epoch.
    pub start_ns: i128,
    /// The wall-clock time just after the child returned,
    /// in nanoseconds since the Unix epoch.
    pub end_ns: i128,
}

/// What a file looked like when it was read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observed {
    /// The device holding the file.
    pub device: u64,
    /// The inode number of the file.
    pub inode: u64,
    /// The inode change time in nanoseconds since the Unix epoch.
    pub changed_ns: i128,
    /// The file's permission bits.
    pub mode: u32,
    /// The text of the file.
    pub text: String,
}

/// What the evidence says about a file after a child ran.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Claim {
    /// The file does not exist.
    Absent,
    /// The file holds the text it had before the child ran;
    /// the child did not change it (or changed it back).
    Unchanged,
    /// The file differs from its snapshot and was changed while the child ran:
    /// this is the child's output,
    /// with the text read through the stable descriptor.
    Ours(
        /// The text of the file as the child left it.
        String,
    ),
    /// The file differs from its snapshot and was changed outside the child's window:
    /// another writer's file,
    /// which is never restored over.
    Foreign,
}

/// What:
/// `pub fn clock_ns() -> i128` is the wall-clock time in nanoseconds since the Unix epoch.
/// Why:
/// The window's two ends are taken with it,
/// around the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const clockNs = (): bigint => BigInt(Date.now()) * 1_000_000n;
/// ```
pub fn clock_ns() -> i128 {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    return match SystemTime::now().duration_since(UNIX_EPOCH) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     `value as i64` converts a number to another numeric type (siblings: `i32`, `u32`, `u64`, `usize`, `f64`); it truncates silently if the value does not fit.
        // Why:      Time arithmetic and sizes come in different integer widths and must be made equal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Number(value)  // JS has one number type, so no cast is needed
        // ```
        Ok(elapsed) => elapsed.as_nanos() as i128,
        // A clock before 1970 reads as zero, which no file time falls into after a real run.
        Err(_) => 0,
    };
}

/// What:
/// `pub trait RecordedWork<T>` is a piece of work whose files are attributed by evidence:
/// `run(self)` does it once and returns its result.
/// Why:
/// `timed`,
/// `run_recording` and `run_recording_manifests` take the work as a value instead of a closure.
/// Every `FnOnce() -> T` is a `RecordedWork<T>` too,
/// so a test passes a closure and the commands pass a named type that holds what the work needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RecordedWork<T> { run(): T; }
/// ```
pub trait RecordedWork<T> {
    /// What:
    /// `run(self)` does the work once.
    /// Why:
    /// The caller measures the window the work ran in around this call.
    fn run(self) -> T;
}

/// What:
/// The implementation of `RecordedWork` for every function that takes nothing and returns the result.
/// Why:
/// Tests and callers that already hold a closure need no wrapper.
impl<T, F: FnOnce() -> T> RecordedWork<T> for F {
    /// What:
    /// Calls the function.
    /// Why:
    /// The function is the work.
    fn run(self) -> T {
        return self();
    }
}

/// What:
/// `pub fn timed<T>(work: impl RecordedWork<T>) -> (T, Window)` runs a child and returns its result with the window it ran in.
/// Why:
/// Every call that may write a file the transaction restores goes through it,
/// so the evidence exists for each one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function timed<T>(work: RecordedWork<T>): [T, Window]
/// ```
pub fn timed<T>(work: impl RecordedWork<T>) -> (T, Window) {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let start_ns: i128 = clock_ns() - START_SLACK_NS;
    // Typed binding: `result: T`.
    let result: T = work.run();
    // Typed binding: `end_ns: i128`.
    let end_ns: i128 = clock_ns();
    return (result, Window { start_ns, end_ns });
}

/// What:
/// `fn identity_of(file: &File) -> io::Result<(u64, u64, i128, u32)>` reads the inode,
/// device,
/// change time and mode of an open file.
/// Why:
/// The same descriptor is asked before and after the read,
/// so a write during the read is noticed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identityOf = (fd: number): [bigint, bigint, bigint, number] => { const s = fstatSync(fd, { bigint: true }); return [s.dev, s.ino, s.ctimeNs, s.mode]; };
/// ```
fn identity_of(file: &File) -> io::Result<(u64, u64, i128, u32)> {
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    let metadata = file.metadata()?;
    // Typed binding: `changed_ns: i128`.
    // Cast `as i128`: converts between numeric types.
    let changed_ns: i128 = (metadata.ctime() as i128) * 1_000_000_000 + (metadata.ctime_nsec() as i128);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok((metadata.dev(), metadata.ino(), changed_ns, metadata.mode()));
}

/// What:
/// `pub fn observe(path: &Path) -> io::Result<Option<Observed>>` reads a file's text and identity through one open descriptor.
/// `Ok(None)` means the file does not exist;
/// every other failure (permission,
/// invalid UTF-8,
/// a file that kept changing) is an `Err`.
/// Why:
/// The reviews showed that treating an unreadable file as an empty one authorizes deletion of someone's edit;
/// absence and failure are different answers here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function observe(path: string): Observed | undefined  // undefined: absent; throws on any other failure
/// ```
pub fn observe(path: &Path) -> io::Result<Option<Observed>> {
    // A file written continuously is read again a few times before the read is given up.
    for _attempt in 0..3 {
        // `match`: picks the arm whose pattern fits the value.
        // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
        // Why:      The value is built up or replaced step by step below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let x = ...;   // plain Rust `let` is TS `const`
        // ```
        // Typed binding: `file: File`.
        let mut file: File = match File::open(path) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(opened) => opened,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => return Err(error),
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        let before = identity_of(&file)?;
        // `let mut`: a binding that may change.
        // Typed binding: `bytes: Vec<u8>`.
        // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
        // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const list: string[] = [];   // or new Set() / new Map()
        // ```
        // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
        // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: T[] / Set / Map / number
        // ```
        // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
        // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: number
        // ```
        let mut bytes: Vec<u8> = Vec::new();
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&mut bytes`: lends the value without copying or giving it away.
        file.read_to_end(&mut bytes)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&file`: lends the value without copying or giving it away.
        let after = identity_of(&file)?;
        if before != after {
            continue;
        }
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `text: String`.
        // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
        // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let text: String = match String::from_utf8(bytes) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
            // Why:      Text building and printing are written as one call instead of string concatenation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // `..${x}..`   // template literal
            // console.error(text);   // eprintln!
            // [a, b]            // vec![a, b]
            // ```
            Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("{} is not UTF-8 text: {error}", path.display()))),
        };
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        return Ok(Some(Observed { device: before.0, inode: before.1, changed_ns: before.2, mode: before.3, text }));
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Err(io::Error::other(format!("{} kept changing while it was read", path.display())));
}

/// What:
/// `pub fn claim_output(path: &Path, before_text: &str, window: Window) -> io::Result<Claim>` says whether a file that a child may
/// have written is the child's output.
/// Why:
/// This is the single place that attributes a file to this run.
/// A file equal to its snapshot is unchanged;
/// a different file whose change time is inside the child's window is ours;
/// a different file changed at any other time is another writer's.
/// A read failure is an `Err`,
/// never a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function claimOutput(path: string, beforeText: string, window: Window): Claim  // throws on a read error
/// ```
pub fn claim_output(path: &Path, before_text: &str, window: Window) -> io::Result<Claim> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `seen: Observed`.
    let seen: Observed = match observe(path)? {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        None => return Ok(Claim::Absent),
    };
    if seen.text == before_text {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(Claim::Unchanged);
    }
    // A window that ends before it starts means the clock stepped back during the call: it proves nothing, so the changed file is
    // treated as the call's output. That is the safe direction, because a restore keeps the file it displaces.
    if window.end_ns < window.start_ns || (seen.changed_ns >= window.start_ns && seen.changed_ns <= window.end_ns) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(Claim::Ours(seen.text));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(Claim::Foreign);
}
