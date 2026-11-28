//! What:
//! The one writer of the program files these test binaries start:
//! a program written by a child process,
//! a program copied by a child process,
//! and a compiled program rewritten to name a dynamic linker of its own.
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
//! export function giveALinkerOfItsOwn(compiled: string, program: string): WithALinkerOfItsOwn
//! ```
#![allow(dead_code)] // Each binary includes this file whole and uses the part it needs.

/// Imports the writer that hands a program's bytes to a child through a pipe.
use std::io::Write;
/// Imports the trait that hands a path's bytes to the compiled program's header writer.
use std::os::unix::ffi::OsStrExt;
/// Imports the trait that adds `from_mode` to permissions,
/// which marks a program
/// executable without opening it.
use std::os::unix::fs::PermissionsExt;
/// Imports the borrowed path type both writers take,
/// and the owned one the linker fixture answers with.
use std::path::{
    Path,
    PathBuf,
};
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

/// Imports the reader that says which interpreter a program names,
/// and its answers.
use mpm_shadow::named_interpreter::{
    NamedInterpreter,
    named_interpreter,
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

/// The number a compiled program's file uses for the program-header entry that holds the path of its
/// dynamic linker
/// (`PT_INTERP`,
/// `/usr/include/elf.h`).
const PT_INTERP: u32 = 3;

/// The offsets this file writes and reads in a 64-bit compiled program,
/// little-endian.
/// `TABLE_OFFSET`,
/// `TABLE_ENTRY_SIZE` and `TABLE_COUNT` are the header's `e_phoff`,
/// `e_phentsize` and `e_phnum`;
/// `ENTRY_KIND`,
/// `ENTRY_PAYLOAD` and `ENTRY_LENGTH` are one entry's `p_type`,
/// `p_offset` and `p_filesz`
/// (`/usr/include/elf.h`).
const TABLE_OFFSET: usize = 32;
const TABLE_ENTRY_SIZE: usize = 54;
const TABLE_COUNT: usize = 56;
const ENTRY_KIND: usize = 0;
const ENTRY_PAYLOAD: usize = 8;
const ENTRY_LENGTH: usize = 32;

/// The class and the byte order of the one layout this file rewrites:
/// 64-bit,
/// least significant byte first,
/// which is what this repository's Linux targets build.
const WIDE_CLASS: u8 = 2;
const LEAST_SIGNIFICANT_FIRST: u8 = 1;

/// What:
/// The files a compiled program that names a dynamic linker of its own is made of:
/// the program itself,
/// the linker it now names,
/// and the linker it named before.
/// Why:
/// A test holds the linker open for writing,
/// takes it away and puts it back,
/// so it needs the three paths by name instead of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WithALinkerOfItsOwn = { program: string; linker: string; linkerItNamed: string };
/// ```
pub struct WithALinkerOfItsOwn {
    /// The program file,
    /// which names `linker` and is what a launcher starts.
    pub program: PathBuf,
    /// The linker the program now names,
    /// which a test can hold open for writing and then take away.
    pub linker: PathBuf,
    /// The linker the program named before this file rewrote it,
    /// which a test copies back to make the program startable again.
    pub linker_it_named: PathBuf,
}

/// What:
/// `fn read_u16`,
/// `fn read_u32` and `fn read_u64` read one whole number of that width at `at`,
/// and `fn write_u64` writes one,
/// all of them least significant byte first.
/// Why:
/// A compiled program's header is numbers at fixed offsets,
/// and the layout this file rewrites writes each of them that way.
fn read_u16(
    bytes: &[u8],
    at: usize,
) -> u16 {
    let field: [u8; 2] = bytes[at..at + 2]
        .try_into()
        .expect("two bytes of the header");
    return u16::from_le_bytes(field);
}

fn read_u32(
    bytes: &[u8],
    at: usize,
) -> u32 {
    let field: [u8; 4] = bytes[at..at + 4]
        .try_into()
        .expect("four bytes of an entry");
    return u32::from_le_bytes(field);
}

fn read_u64(
    bytes: &[u8],
    at: usize,
) -> u64 {
    let field: [u8; 8] = bytes[at..at + 8]
        .try_into()
        .expect("eight bytes of the header");
    return u64::from_le_bytes(field);
}

fn write_u64(
    bytes: &mut [u8],
    at: usize,
    value: u64,
) {
    bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
}

/// What:
///     `fn give_a_linker_of_its_own(compiled, program) -> WithALinkerOfItsOwn` writes `program` as a
/// copy of
///     `compiled` whose program-header table points its interpreter entry at a copy of the linker
///     `compiled` names,
///     and copies that linker beside `program`.
/// Why:
///     A start is refused for a file it needs being written or being gone,
///     and the message must name the file that fits;
///     the linker of a program on this machine cannot be held open or taken away,
///     so the fixture needs one of its own.
///     Only the entry's payload offset and length change:
///     the path is appended after the last segment,
///     where no part of the program reads it,
///     and the kernel reads it from the file exactly as it read the original.
///     Measured 2026-11-27 on this machine:
///     the rewritten program starts through its own linker,
///     a start of it answers `Text file busy` while that linker is held open for writing,
///     and `No such file or directory` once it is taken away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function giveALinkerOfItsOwn(compiled: string, program: string): WithALinkerOfItsOwn
/// ```
pub fn give_a_linker_of_its_own(
    compiled: &Path,
    program: &Path,
) -> WithALinkerOfItsOwn {
    let linker_it_named: PathBuf = match named_interpreter(compiled) {
        NamedInterpreter::DynamicLinker {
            linker,
        } => linker,
        other => panic!(
            "{} names no dynamic linker ({other:?}), so this fixture cannot give it one of its own",
            compiled.display()
        ),
    };
    let linker: PathBuf = program
        .parent()
        .expect("a folder for the program")
        .join("linker-of-its-own");
    assert!(
        linker.is_absolute(),
        "the kernel reads the interpreter's path exactly as it is written, and {} is not absolute",
        linker.display()
    );
    let mut bytes: Vec<u8> = std::fs::read(compiled).expect("read the compiled program");
    assert_eq!(
        bytes[..4],
        ELF_MAGIC,
        "{} is not a compiled program, so this fixture cannot rewrite it",
        compiled.display()
    );
    assert_eq!(
        (bytes[4], bytes[5]),
        (WIDE_CLASS, LEAST_SIGNIFICANT_FIRST),
        "this fixture rewrites 64-bit little-endian programs only, and {} is class {} with byte order {}",
        compiled.display(),
        bytes[4],
        bytes[5]
    );
    // What:     `copy_program` copies the linker with a child `cp`, so this process holds no write
    //           descriptor on it.
    // Why:      A descriptor here would be inherited by a child another test thread forks, and that
    //           child's own start would then be refused (issue 6527).
    copy_program(&linker_it_named, &linker);
    let mut text: Vec<u8> = linker.as_os_str().as_bytes().to_vec();
    text.push(0);
    let table: usize = usize::try_from(read_u64(&bytes, TABLE_OFFSET)).expect("an offset");
    let entry_size: usize = usize::from(read_u16(&bytes, TABLE_ENTRY_SIZE));
    let count: usize = usize::from(read_u16(&bytes, TABLE_COUNT));
    let payload_at: u64 = u64::try_from(bytes.len()).expect("a file size");
    let found: Option<usize> = (0..count).find(|index| {
        return read_u32(&bytes, table + index * entry_size + ENTRY_KIND) == PT_INTERP;
    });
    let entry: usize = table
        + found.expect("the program names no interpreter in its program-header table") * entry_size;
    bytes.extend_from_slice(&text);
    write_u64(&mut bytes, entry + ENTRY_PAYLOAD, payload_at);
    write_u64(
        &mut bytes,
        entry + ENTRY_LENGTH,
        u64::try_from(text.len()).expect("a short path"),
    );
    write_program(program, &bytes);
    assert_compiled(program);
    return WithALinkerOfItsOwn {
        program: program.to_path_buf(),
        linker,
        linker_it_named,
    };
}

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
