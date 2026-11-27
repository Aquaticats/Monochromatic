//! What:
//! The one writer of the program files these test binaries start:
//! a program written by a child process,
//! and a program copied by a child process.
//! Why:
//! A test binary runs its tests as threads of one process,
//! so another test may `fork` at any moment.
//! A `fork` copies the whole descriptor table,
//! so a program file this process holds open for writing is inherited by that child
//! and stays open in it until the child reaches its own start,
//! and `execve` refuses a file that any process holds open for writing:
//! `Text file busy` (issue 6527,
//! measured 2026-11-27).
//! A file only ever opened for writing by `tee` or `cp` cannot be inherited by another
//! thread's child.
//! The file lives in a folder of its own because several test binaries include it and it
//! is not a test binary itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // tests/written-programs/through-a-child-process.ts
//! export function writeProgram(path: string, content: Uint8Array): void
//! export function copyProgram(from: string, to: string): void
//! ```
#![allow(dead_code)] // Each binary includes this file whole and uses the part it needs.

/// Imports the writer that hands a program's bytes to a child through a pipe.
use std::io::Write;
/// Imports the trait that adds `from_mode` to permissions,
/// which marks a program
/// executable without opening it.
use std::os::unix::fs::PermissionsExt;
/// Imports the borrowed path type both writers take.
use std::path::Path;
/// Imports the types one writer needs:
/// the running child,
/// the pipe into it,
/// the command that starts it and the standard streams it gets.
use std::process::{
    Child,
    ChildStdin,
    Command,
    Stdio,
};

/// What:
///     `fn write_program(path, content)` creates the file at `path`,
/// writes `content` into
///     it and marks it executable,
///     and the writing is done by a child `tee` process.
/// Why:
///     The descriptor that writes the program belongs to `tee`,
/// not to this process,
/// so no
///     child another thread forks can inherit it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync('tee', [path], { input: content, stdio: ['pipe', 'ignore', 'inherit'] });
/// chmodSync(path, 0o755);
/// ```
pub fn write_program(
    path: &Path,
    content: &[u8],
) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("make the directory");
    // What:     `Stdio::piped()` connects this process to the child's standard input, and
    //           `Stdio::null()` throws its copy of the content away.
    // Why:      `tee` writes what it reads to each output, and only the file matters.
    let mut writer: Child = Command::new("tee")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("start the program writer");
    // What:     `.take()` moves the pipe out of the child record, so the pipe can be
    //           closed below.
    // Why:      Closing it is how `tee` learns the content ended.
    let mut input: ChildStdin = writer.stdin.take().expect("the program writer's input");
    input.write_all(content).expect("write the program");
    // What:     `drop(input)` closes the pipe now.
    // Why:      Without the close, `tee` would wait for more content forever.
    drop(input);
    assert!(
        writer.wait().expect("the program writer's exit").success(),
        "the program writer failed for {}",
        path.display()
    );
    mark_executable(path);
}

/// What:
///     `fn copy_program(from, to)` copies the program at `from` to `to` and marks the copy
///     executable,
///     and the copy is made by a child `cp` process.
/// Why:
///     A copy this process made would be open for writing here,
///     with the window
///     `write_program` describes,
///     and the tests copy a built binary that they then start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync('cp', ['--reflink=never', '--', from, to]);
/// chmodSync(to, 0o755);
/// ```
pub fn copy_program(
    from: &Path,
    to: &Path,
) {
    std::fs::create_dir_all(to.parent().expect("a parent")).expect("make the directory");
    // What:     `--reflink=never` asks for a real data copy, and `--` ends the options so
    //           a path may start with a dash.
    // Why:      On btrfs, where this build directory lives, the default copy clones the
    //           extents and the clone has no pages in the page cache,
    //           so the first start of the copy reads most of it from disk.
    let copied: bool = Command::new("cp")
        .arg("--reflink=never")
        .arg("--")
        .arg(from)
        .arg(to)
        .status()
        .expect("start the program copy")
        .success();
    assert!(
        copied,
        "the program copy failed: {} to {}",
        from.display(),
        to.display()
    );
    mark_executable(to);
}

/// What:
///     `fn mark_executable(path)` sets the file's mode to `0o755`.
/// Why:
///     `chmod` names the file by path and never opens it,
/// so it cannot leave a write
///     descriptor behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync(path, 0o755);
/// ```
fn mark_executable(path: &Path) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("mark the program executable");
}

/// The four bytes every compiled ELF program starts with (the "magic number").
const ELF_MAGIC: [u8; 4] = *b"\x7fELF";

/// What:
///     `fn assert_compiled(path)` fails unless the file at `path` starts with `ELF_MAGIC`,
///     which every compiled program does and no text file does.
/// Why:
///     A control that holds a program open for writing needs a compiled program:
///     a script's first line names an interpreter,
///     so the message names the interpreter instead of the program,
///     and a compiled program is started through the open file rather than by its path.
///     A missing or unreadable file fails here rather than later,
///     where its absence would read as the refusal the control is looking for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = readFileSync(path).subarray(0, 4).toString('latin1');
/// if (head !== '\x7fELF') throw new Error(`${path} is not a compiled program`);
/// ```
pub fn assert_compiled(path: &Path) {
    /// Imports `Read`,
    /// the trait that adds `read_exact` to an open file.
    use std::io::Read;
    // What:     `fs::File::open(path)` gives an open file for reading only.
    // Why:      A write descriptor this process held would be inherited by a child another
    //           test thread forks, and its start would then be refused (issue 6527).
    let mut file: std::fs::File = std::fs::File::open(path).expect("open the compiled program");
    // What:     A four-byte array of zeroes, filled by `read_exact`.
    // Why:      Only the magic number is needed, so the program is never read whole.
    let mut head: [u8; 4] = [0; 4];
    file.read_exact(&mut head)
        .expect("read the compiled program's first bytes");
    assert_eq!(
        head,
        ELF_MAGIC,
        "{} is not a compiled program, so this control cannot use it",
        path.display()
    );
}
