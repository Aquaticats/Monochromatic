//! What:
//! The interpreter a program names:
//! the program after `#!` on a script's first line,
//! or the dynamic linker a compiled program's own file points at.
//! Why:
//! A start that failed with `ETXTBSY` or `ENOENT` names "the interpreter" as one of the two files it needed,
//! and a sentence that calls it "the interpreter named on the program's first line" points a reader
//! at a line a compiled program does not have:
//! a compiled program names its interpreter in its own file,
//! so the sentence sent the person to the wrong file.
//! The kernel's answer cannot say which file it refused,
//! but the program's bytes say which interpreter it names,
//! and whether that file is there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type NamedInterpreter = { kind: 'script'; interpreter: string } | { kind: 'linker'; linker: string } | { kind: 'none' } | { kind: 'unknown' };
//! export function namedInterpreter(program: string): NamedInterpreter { /* '#!/bin/sh' or the ELF PT_INTERP entry */ }
//! type MissingInterpreter = { kind: 'script'; interpreter: string } | { kind: 'linker'; linker: string } | { kind: 'saysNothing' };
//! export function missingInterpreter(program: string): MissingInterpreter
//! ```

/// Imports `File`,
/// the handle this module reads a program's first bytes through
/// (siblings:
/// `fs::read` reads a whole file into memory,
/// which a large program makes wasteful).
use std::fs::File;
/// Imports `Read` (the `read` method),
/// `Seek` and `SeekFrom` (moving to a byte offset inside the open file).
use std::io::{
    Read,
    Seek,
    SeekFrom,
};
/// Imports `Path` (a borrowed path) and `PathBuf` (an owned one,
/// which outlives this module's buffers).
use std::path::{
    Path,
    PathBuf,
};

/// What:
/// `const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];` is the four bytes every compiled program for this
/// system begins with.
/// `[u8; 4]` is a fixed array of four bytes:
/// the siblings a reader might expect are `&[u8; 4]` (a borrow) and `Vec<u8>` (a growable buffer).
/// `b'E'` is the byte for the letter E,
/// and `*b""` spells a byte string's bytes as the array a const needs.
/// Why:
/// The first four bytes are what tells a compiled program from a script,
/// before any other reading is worth doing.
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// What:
/// `const SHEBANG_MAGIC: [u8; 2] = *b"#!";` is the two bytes a script's first line begins with,
/// which the operating system reads to find the program that must start the script.
/// Why:
/// The two bytes are the only thing that distinguishes a script from a text file the kernel refuses.
const SHEBANG_MAGIC: [u8; 2] = *b"#!";

/// What:
/// `const PT_INTERP: u32 = 3;` is the number a compiled program's file uses for the entry that holds the path of
/// its dynamic linker.
/// `u32` is a 32-bit unsigned integer (siblings:
/// `u8`,
/// `u16`,
/// `u64`);
/// the number in the file is that wide.
/// Why:
/// It is the one entry of the program-header table whose payload is the interpreter's path.
const PT_INTERP: u32 = 3;

/// What:
/// `const IDENTITY_SIZE: usize = 16;` is how many bytes the start of a compiled program's header spends saying
/// which system and processor the file is for.
/// `usize` is the unsigned integer wide enough to index memory (siblings:
/// `u32`,
/// `u64`);
/// slicing and lengths are spelled in it.
/// Why:
/// The two bytes inside it that matter here (which class and which byte order) sit past the four magic bytes,
/// and reading the header without them is unsafe to interpret.
const IDENTITY_SIZE: usize = 16;

/// What:
/// `const HEADER_READ: u64 = 64;` is how many bytes this module reads from the start of a program file first:
/// the whole 64-bit header,
/// which also holds the 32-bit one.
/// `u64` is a 64-bit unsigned integer (siblings:
/// `u32`,
/// `usize`);
/// the file-offset arguments here are that wide.
/// Why:
/// One read answers both questions the caller asks:
/// is this a script,
/// and where is the program-header table.
const HEADER_READ: u64 = 64;

/// What:
/// `const PROGRAM_HEADER_READ: u64 = 56;` is how many bytes of one program-header entry this module reads.
/// Why:
/// Every field this module wants is inside the first 40 bytes of either class's entry,
/// and an entry a poisoned file declares as enormous is never read whole.
const PROGRAM_HEADER_READ: u64 = 56;

/// What:
/// `const ENTRY_LIMIT: u64 = 1024;` is the most program-header entries this module walks.
/// Why:
/// A real program has a handful;
/// the bound stops a malformed file from making this loop run for as long as it likes.
const ENTRY_LIMIT: u64 = 1024;

/// What:
/// `const INTERPRETER_PATH_LIMIT: u64 = 4096;` is the longest interpreter path this module reads.
/// Why:
/// A path longer than the system's own limit cannot be one the kernel would use,
/// and the bound keeps a malformed length from asking for a huge buffer.
const INTERPRETER_PATH_LIMIT: u64 = 4096;

/// What:
/// `const FIRST_LINE_LIMIT: u64 = 512;` is how much of a script this module reads to reach the end of its first
/// line.
/// Why:
/// The kernel reads at most 256 bytes of that line,
/// so 512 covers any line it would honour and bounds the read.
const FIRST_LINE_LIMIT: u64 = 512;

/// What:
/// `pub enum NamedInterpreter` is what a program's own file says about the interpreter its start needs:
/// a script's `#!` interpreter,
/// a compiled program's dynamic linker,
/// a compiled program that needs no other file,
/// or nothing this module can read.
/// Why:
/// The message must name the file the system refused,
/// and the four answers need four different sentences.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamedInterpreter = { kind: 'script'; interpreter: string } | { kind: 'linker'; linker: string } | { kind: 'none' } | { kind: 'unknown' };
/// ```
#[derive(Debug)]
pub enum NamedInterpreter {
    /// A script:
    /// the program its first line names after `#!`.
    Script {
        /// The interpreter's path,
        /// as the first line writes it.
        interpreter: PathBuf,
    },
    /// A compiled program for this system:
    /// the dynamic linker its own file points at.
    DynamicLinker {
        /// The linker's path,
        /// as the file writes it.
        linker: PathBuf,
    },
    /// A compiled program that names no interpreter,
    /// so its own file is the only file its start needs.
    NoInterpreter,
    /// A file this module cannot read as a program:
    /// it was gone,
    /// it could not be opened,
    /// or its first bytes are neither a `#!` line nor a program header this module understands.
    Unknown,
}

/// What:
/// The interpreter a program names that a start of it will not find:
/// a script's `#!` program that is not there,
/// a compiled program's dynamic linker that is not there,
/// or nothing the program's own file can say.
/// Why:
/// A message about a start the system refused with "no such file or directory" must name the file
/// that fits,
/// and the two kinds of interpreter are the two sentences a reader can act on;
/// when nothing of the program can be read,
/// the message must name both kinds and claim neither.
/// The answer is chosen here so both messages choose it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MissingInterpreter = { kind: 'script'; interpreter: string } | { kind: 'linker'; linker: string } | { kind: 'saysNothing' };
/// ```
#[derive(Debug)]
pub enum MissingInterpreter {
    /// A script whose interpreter is not there.
    Script {
        /// The interpreter's path,
        /// as the script's first line writes it.
        interpreter: PathBuf,
    },
    /// A compiled program whose dynamic linker is not there.
    DynamicLinker {
        /// The linker's path,
        /// as the program's own file writes it.
        linker: PathBuf,
    },
    /// Nothing the program's own file can say:
    /// it is gone,
    /// it cannot be read,
    /// its first bytes are not a program this module understands,
    /// or the interpreter it names is there.
    SaysNothing,
}

/// What:
/// Where one compiled program keeps its program-header table,
/// and how to read one entry of it.
/// Why:
/// The two classes (32- and 64-bit) and the two byte orders put the same numbers at four different places,
/// and one record keeps those choices together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProgramHeaderTable = { offset: number; entrySize: number; count: number; wide: boolean; bigEndian: boolean };
/// ```
struct ProgramHeaderTable {
    /// The byte offset of the first entry in the file.
    offset: u64,
    /// How many bytes one entry takes.
    entry_size: u64,
    /// How many entries there are.
    count: u64,
    /// Whether the file is a 64-bit program (`true`) or a 32-bit one (`false`).
    wide: bool,
    /// Whether the numbers in the file are written most significant byte first.
    big_endian: bool,
}

/// What:
/// `pub fn named_interpreter(program: &Path) -> NamedInterpreter` reads `program`'s first bytes and answers
/// which interpreter the program names.
/// `&Path` borrows the path read-only;
/// the answer owns its own paths because the caller keeps it after this function returns.
/// Why:
/// Both messages about a refused start need this,
/// and both must not be able to fail:
/// a file that cannot be read is one more case,
/// not an error that stops the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedInterpreter(program: string): NamedInterpreter
/// ```
pub fn named_interpreter(program: &Path) -> NamedInterpreter {
    // `File::open` returns a `Result`; `let Ok(...) else` takes the handle on success and returns early otherwise.
    let Ok(mut file) = File::open(program) else {
        return NamedInterpreter::Unknown;
    };
    let Some(header) = read_at(&mut file, 0, HEADER_READ) else {
        return NamedInterpreter::Unknown;
    };
    if header.starts_with(&ELF_MAGIC) {
        return compiled(&mut file, &header);
    }
    if header.starts_with(&SHEBANG_MAGIC) {
        return script(&mut file);
    }
    return NamedInterpreter::Unknown;
}

/// What:
/// `pub fn missing_interpreter(program: &Path) -> MissingInterpreter` answers which interpreter `program`
/// names that is not there,
/// or that its own file says nothing the message can use.
/// `&Path` borrows the path read-only;
/// the answer owns its own paths because the caller keeps it after this function returns.
/// Why:
/// Both messages about a start that found nothing need this,
/// and both must not be able to fail:
/// a file that cannot be read is one more answer,
/// not an error that stops the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingInterpreter(program: string): MissingInterpreter
/// ```
pub fn missing_interpreter(program: &Path) -> MissingInterpreter {
    return match named_interpreter(program) {
        NamedInterpreter::Script {
            interpreter,
        } if !interpreter.exists() => MissingInterpreter::Script {
            interpreter,
        },
        NamedInterpreter::DynamicLinker {
            linker,
        } if !linker.exists() => MissingInterpreter::DynamicLinker {
            linker,
        },
        _ => MissingInterpreter::SaysNothing,
    };
}

/// What:
/// `fn read_at(file: &mut File, offset: u64, length: u64) -> Option<Vec<u8>>` reads at most `length` bytes from
/// `offset` and answers them,
/// or `None` when the file cannot be read at all.
/// `&mut File` lends the open file for changing (the read position moves);
/// `Option<Vec<u8>>` is "a growable byte buffer,
/// or nothing to read" (siblings:
/// `Result<Vec<u8>, Error>`,
/// which would make every caller handle a file that merely ends early).
/// Why:
/// A file shorter than asked for is normal here (a small script),
/// so a short read is an answer,
/// not a failure.
fn read_at(
    file: &mut File,
    offset: u64,
    length: u64,
) -> Option<Vec<u8>> {
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut buffer: Vec<u8> = vec![0; usize::try_from(length).ok()?];
    let mut filled: usize = 0;
    while filled < buffer.len() {
        let read: usize = file.read(&mut buffer[filled..]).ok()?;
        if read == 0 {
            break;
        }
        filled += read;
    }
    buffer.truncate(filled);
    return Some(buffer);
}

/// What:
/// `fn number(bytes: &[u8], offset: usize, width: usize, big_endian: bool) -> Option<u64>` reads a whole number
/// of `width` bytes from `bytes` at `offset` and answers it as a `u64`,
/// or `None` when those bytes are not there.
/// Why:
/// Every field this module reads is a number at a fixed offset,
/// and a file cut short must answer "no" rather than panic.
fn number(
    bytes: &[u8],
    offset: usize,
    width: usize,
    big_endian: bool,
) -> Option<u64> {
    let end: usize = offset.checked_add(width)?;
    let field: &[u8] = bytes.get(offset..end)?;
    let mut value: u64 = 0;
    if big_endian {
        for byte in field {
            value = (value << 8) | u64::from(*byte);
        }
    } else {
        for byte in field.iter().rev() {
            value = (value << 8) | u64::from(*byte);
        }
    }
    return Some(value);
}

/// What:
/// `fn compiled(file: &mut File, header: &[u8]) -> NamedInterpreter` reads the program-header table of a
/// compiled program and answers the dynamic linker it names.
/// Why:
/// A compiled program keeps its interpreter's path in one entry of that table,
/// so the answer comes from the file itself and not from a guess about the machine.
fn compiled(
    file: &mut File,
    header: &[u8],
) -> NamedInterpreter {
    if header.len() < IDENTITY_SIZE {
        return NamedInterpreter::Unknown;
    }
    let wide: bool = match header[4] {
        1 => false,
        2 => true,
        _ => return NamedInterpreter::Unknown,
    };
    let big_endian: bool = match header[5] {
        1 => false,
        2 => true,
        _ => return NamedInterpreter::Unknown,
    };
    let Some(table) = program_header_table(header, wide, big_endian) else {
        return NamedInterpreter::Unknown;
    };
    for index in 0..table.count {
        let start: u64 = table.offset + index * table.entry_size;
        let Some(entry) = read_at(file, start, table.entry_size.min(PROGRAM_HEADER_READ)) else {
            return NamedInterpreter::Unknown;
        };
        if number(&entry, 0, 4, table.big_endian) != Some(u64::from(PT_INTERP)) {
            continue;
        }
        return match interpreter_path(file, &entry, &table) {
            Some(linker) => NamedInterpreter::DynamicLinker {
                linker,
            },
            None => NamedInterpreter::Unknown,
        };
    }
    return NamedInterpreter::NoInterpreter;
}

/// What:
/// `fn program_header_table(header: &[u8], wide: bool, big_endian: bool) -> Option<ProgramHeaderTable>` finds
/// where the table starts and how its entries are sized.
/// Why:
/// The 32-bit and 64-bit headers place the same three numbers at different offsets,
/// and this is the one place that knows both layouts.
fn program_header_table(
    header: &[u8],
    wide: bool,
    big_endian: bool,
) -> Option<ProgramHeaderTable> {
    let (offset_field, size_field, count_field): (usize, usize, usize) =
        if wide { (32, 54, 56) } else { (28, 42, 44) };
    let offset_width: usize = if wide { 8 } else { 4 };
    let offset: u64 = number(header, offset_field, offset_width, big_endian)?;
    let entry_size: u64 = number(header, size_field, 2, big_endian)?;
    let count: u64 = number(header, count_field, 2, big_endian)?;
    let shortest: u64 = if wide { 56 } else { 32 };
    if entry_size < shortest || count > ENTRY_LIMIT {
        return None;
    }
    return Some(ProgramHeaderTable {
        offset,
        entry_size,
        count,
        wide,
        big_endian,
    });
}

/// What:
/// `fn interpreter_path(file: &mut File, entry: &[u8], table: &ProgramHeaderTable) -> Option<PathBuf>` reads the
/// path out of the one entry that holds the dynamic linker.
/// Why:
/// The entry names where its text sits in the file by an offset and a length,
/// so the path is read where the file puts it rather than where a table of offsets would guess.
fn interpreter_path(
    file: &mut File,
    entry: &[u8],
    table: &ProgramHeaderTable,
) -> Option<PathBuf> {
    let (offset_field, size_field): (usize, usize) = if table.wide { (8, 32) } else { (4, 16) };
    let offset_width: usize = if table.wide { 8 } else { 4 };
    let offset: u64 = number(entry, offset_field, offset_width, table.big_endian)?;
    let size: u64 = number(entry, size_field, offset_width, table.big_endian)?;
    if size == 0 || size > INTERPRETER_PATH_LIMIT {
        return None;
    }
    let bytes: Vec<u8> = read_at(file, offset, size)?;
    let end: usize = bytes
        .iter()
        .position(|byte| return *byte == 0)
        .unwrap_or(bytes.len());
    let text: String = String::from_utf8_lossy(&bytes[..end]).to_string();
    if text.is_empty() {
        return None;
    }
    // `PathBuf::from` wraps the text as an owned path; the `Some` below is the present case of `Option`.
    return Some(PathBuf::from(text));
}

/// What:
/// `fn script(file: &mut File) -> NamedInterpreter` reads the program a script's first line names.
/// Why:
/// The kernel starts the program after `#!`,
/// and the rest of the line is its arguments,
/// so only the first word is the interpreter.
fn script(file: &mut File) -> NamedInterpreter {
    let Some(bytes) = read_at(file, 0, FIRST_LINE_LIMIT) else {
        return NamedInterpreter::Unknown;
    };
    let end: usize = bytes
        .iter()
        .position(|byte| return *byte == b'\n')
        .unwrap_or(bytes.len());
    let line: String = String::from_utf8_lossy(&bytes[SHEBANG_MAGIC.len()..end]).to_string();
    let Some(first_word) = line.split_whitespace().next() else {
        return NamedInterpreter::Unknown;
    };
    return NamedInterpreter::Script {
        interpreter: PathBuf::from(first_word),
    };
}
