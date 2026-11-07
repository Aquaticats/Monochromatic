//! What:
//! The rule "a shadow never starts a shadow,
//! of any version,
//! as its real program",
//! in three parts:
//! an identity text that every shadow program carries inside its own file,
//! a check that reads that text from a candidate (through launcher records when the candidate is a launcher),
//! and an arrival marker in the environment that stops a shadow which a shadow started anyway.
//! Why:
//! A shadow sits first on `PATH` under the real tool's name.
//! Judging a candidate by its path or by this program's own path cannot tell another copy or another release of a shadow,
//! so two shadows at different paths chose each other as the real tool and replaced themselves with each other without end
//! (measured 2026-11-07,
//! `search-shadow` 0.2.2 against another copy:
//! about 105 starts per second under `strace`).
//! The owner's rule:
//! "We should never invoke ourselves recursively.
//! That's bad design."
//! What a file is does not depend on its path or its version:
//! every shadow carries the same identity text in a section of its own,
//! so a shadow skips such a candidate without starting it.
//! What no test of a file can recognize (a script that runs a shadow,
//! a launcher with no record)
//! is stopped by the start itself:
//! a shadow marks the environment of the program it starts with its own process number and birth identity,
//! and a shadow that finds the mark left for its own process,
//! or for its direct parent,
//! was started by a shadow and stops at once.
//! Anything started further down (a script the real tool runs,
//! which may call the shadow again) has another process in between and passes.
//!
//! This file is one of four byte-identical copies,
//! one per shadow crate (`cargo-shadow`,
//! `mpm-shadow`,
//! `pnpm-shadow`,
//! `search-shadow`),
//! because the four crates share no dependency and a crates.io footprint that stays small is the owner's decision
//! (`doc/planning/one-release-each-batch-2026-11.md`,
//! "What the owner decided").
//! `tests/shadow_identity_copies.rs` of `search-shadow` fails when the copies differ.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-identity.ts
//! export function identityOfFile(file: FileHandle): string | undefined { /* the ELF section ".monochromatic_shadow_identity" */ }
//! export function shadowBehind(candidate: string): string | undefined { /* follow `.launch` records, then identityOfFile */ }
//! export function arrivalAtThisProcess(value: string | undefined): Arrival | undefined { /* pid + birth identity */ }
//! ```

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{FileExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The environment variable that carries the arrival marker from a shadow to the program it starts.
pub const ARRIVAL_VARIABLE: &str = "MONOCHROMATIC_SHADOW_STARTED_BY";

/// The exit status of a shadow that was started by a shadow.
pub const ARRIVAL_EXIT_CODE: i32 = 2;

/// The name of the ELF section that holds the identity text.
pub const IDENTITY_SECTION: &str = ".monochromatic_shadow_identity";

/// The first words of every identity text;
/// what follows is the crate's name and version.
pub const IDENTITY_PREFIX: &str = "MONOCHROMATIC-SHADOW-IDENTITY/1 ";

/// The suffix of the record the toolchain writes beside a launcher (`package/mpm-shadow/cli/README.md`,
/// section "The launcher").
pub const LAUNCH_RECORD_SUFFIX: &str = ".launch";

/// How many launchers in a row are followed before the chain counts as one that never reaches a program.
/// The toolchain makes one launcher per tool and a launcher names a program,
/// so a chain of two is already unusual;
/// a longer one is a loop of records.
/// Delegated pick under the owner's rule that nothing waits without end;
/// not a measurement.
pub const MAX_LAUNCHER_HOPS: usize = 8;

/// The most sections an ELF file may have and still be read for the identity.
/// A release build of each shadow has fewer than one hundred (`readelf --section-headers`,
/// 2026-11-07);
/// this is a bound on memory.
const MAX_SECTIONS: usize = 2048;

/// The most bytes read for the section names and for the identity text.
const MAX_SECTION_BYTES: u64 = 64 * 1024;

/// The identity text this program carries,
/// written in full so that the compiler places it in the section below.
const CARRIED_TEXT: &str = concat!("MONOCHROMATIC-SHADOW-IDENTITY/1 ", env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

/// Copies a text into an array of its own length,
/// which a `static` needs.
const fn copy_into<const LENGTH: usize>(text: &str) -> [u8; LENGTH] {
    let source: &[u8] = text.as_bytes();
    let mut copy: [u8; LENGTH] = [0; LENGTH];
    let mut index: usize = 0;
    while index < LENGTH {
        copy[index] = source[index];
        index += 1;
    }
    return copy;
}

/// What:
/// The identity text as bytes,
/// placed in its own section of the program file.
/// Why:
/// Every shadow carries it,
/// so a candidate on `PATH` that carries it is a shadow whatever its path,
/// name or version.
/// `#[used]` keeps the linker from dropping a value nothing reads.
#[used]
#[unsafe(link_section = ".monochromatic_shadow_identity")]
static CARRIED: [u8; CARRIED_TEXT.len()] = copy_into(CARRIED_TEXT);

/// What:
/// `pub fn own_identity() -> &'static str` is what this program says it is:
/// the crate name,
/// a space and the version.
/// Why:
/// The arrival marker and the messages name the shadow with it.
/// It reads the carried bytes,
/// so the section is referenced and kept in every build.
pub fn own_identity() -> &'static str {
    let carried: &[u8] = std::hint::black_box(&CARRIED);
    return std::str::from_utf8(&carried[IDENTITY_PREFIX.len()..]).unwrap_or("a shadow");
}

/// Reads a little-endian 16-bit number out of a header.
fn u16_at(bytes: &[u8], offset: usize) -> usize {
    return usize::from(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]));
}

/// Reads a little-endian 32-bit number out of a header.
fn u32_at(bytes: &[u8], offset: usize) -> usize {
    return u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]) as usize;
}

/// Reads a little-endian 64-bit number out of a header.
fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    let mut eight: [u8; 8] = [0; 8];
    eight.copy_from_slice(&bytes[offset..offset + 8]);
    return u64::from_le_bytes(eight);
}

/// What:
/// `pub fn identity_of_open_file(file: &fs::File) -> Option<String>` is the crate name and version of the shadow that the open file is,
/// or `None` when it carries no identity text or is not an ELF file this can read.
/// Why:
/// The check reads the open file,
/// not a path,
/// so a path swapped after the check cannot change what was checked.
/// Only a 64-bit little-endian ELF file is read (every shadow is one);
/// anything else is not a shadow.
/// A few small reads find the section,
/// so a candidate costs a fraction of a millisecond whatever its size.
pub fn identity_of_open_file(file: &fs::File) -> Option<String> {
    let mut header: [u8; 64] = [0; 64];
    file.read_exact_at(&mut header, 0).ok()?;
    let is_elf64_little_endian: bool = header[..4] == *b"\x7fELF" && header[4] == 2 && header[5] == 1;
    if !is_elf64_little_endian {
        return None;
    }
    let table_offset: u64 = u64_at(&header, 0x28);
    let entry_size: usize = u16_at(&header, 0x3A);
    let entry_count: usize = u16_at(&header, 0x3C);
    let names_index: usize = u16_at(&header, 0x3E);
    if entry_size < 64 || entry_count == 0 || entry_count > MAX_SECTIONS || names_index >= entry_count {
        return None;
    }
    let mut table: Vec<u8> = vec![0; entry_size * entry_count];
    file.read_exact_at(&mut table, table_offset).ok()?;
    let entry = |index: usize| -> &[u8] { return &table[index * entry_size..(index + 1) * entry_size]; };
    let names_entry: &[u8] = entry(names_index);
    let names_size: u64 = u64_at(names_entry, 0x20);
    if names_size > MAX_SECTION_BYTES {
        return None;
    }
    let mut names: Vec<u8> = vec![0; usize::try_from(names_size).ok()?];
    file.read_exact_at(&mut names, u64_at(names_entry, 0x18)).ok()?;
    for index in 0..entry_count {
        let candidate: &[u8] = entry(index);
        let name_start: usize = u32_at(candidate, 0);
        let name_bytes: &[u8] = names.get(name_start..)?;
        let name_end: usize = name_bytes.iter().position(|byte| { return *byte == 0; }).unwrap_or(name_bytes.len());
        if &name_bytes[..name_end] != IDENTITY_SECTION.as_bytes() {
            continue;
        }
        let size: u64 = u64_at(candidate, 0x20);
        if size > MAX_SECTION_BYTES {
            return None;
        }
        let mut text: Vec<u8> = vec![0; usize::try_from(size).ok()?];
        file.read_exact_at(&mut text, u64_at(candidate, 0x18)).ok()?;
        let whole: String = String::from_utf8(text).ok()?;
        return whole.trim_end_matches('\0').strip_prefix(IDENTITY_PREFIX).map(str::to_string);
    }
    return None;
}

/// What:
/// `fn record_target(program: &Path) -> Option<PathBuf>` is the program that the launcher record beside `program` names,
/// when there is one.
/// Why:
/// The toolchain puts a launcher (a copy of the `mpm-shadow` program,
/// which carries the identity text) between a link and a tool;
/// the record beside it,
/// a JSON object with a `real` field,
/// names what it starts.
/// A launcher is judged by where it leads,
/// not by what it is.
/// An unreadable or unparsable record is not a launcher record.
fn record_target(program: &Path) -> Option<PathBuf> {
    let mut record_name: OsString = program.as_os_str().to_os_string();
    record_name.push(LAUNCH_RECORD_SUFFIX);
    let text: String = fs::read_to_string(PathBuf::from(record_name)).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    return value.get("real")?.as_str().map(PathBuf::from);
}

/// What:
/// `pub fn shadow_behind(candidate: &Path) -> Option<String>` is the identity of the shadow that starting `candidate` would start,
/// when it would:
/// the candidate itself,
/// or the program a launcher record leads to,
/// through further launchers up to `MAX_LAUNCHER_HOPS`.
/// Why:
/// A path or a version says nothing about what a file is.
/// A candidate that leads to a shadow is skipped without being started.
/// A chain of records that never ends counts as leading to a shadow (it would start launchers without end).
/// A file that cannot be opened or a record that names a missing file is "cannot tell":
/// the arrival marker is the backstop for what this cannot see.
pub fn shadow_behind(candidate: &Path) -> Option<String> {
    let mut current: PathBuf = candidate.to_path_buf();
    for _hop in 0..=MAX_LAUNCHER_HOPS {
        let resolved: PathBuf = fs::canonicalize(&current).ok()?;
        if let Some(target) = record_target(&resolved) {
            current = target;
            continue;
        }
        let file: fs::File = fs::File::open(&resolved).ok()?;
        return identity_of_open_file(&file);
    }
    return Some(format!("a chain of more than {MAX_LAUNCHER_HOPS} launchers that never reaches a program"));
}

/// What:
/// `pub fn identity_of_open_file_unless_launcher(file: &fs::File, program: &Path) -> Option<String>` is the identity of the shadow that
/// starting `program` would start,
/// judged on the open file when no launcher record stands beside the path and by where the record leads when one does.
/// Why:
/// A launcher is a copy of the `mpm-shadow` program,
/// which carries the identity:
/// its own bytes say nothing about the tool it starts.
/// Without a record the open file is the whole answer,
/// and it is the file that is about to be started.
pub fn identity_of_open_file_unless_launcher(file: &fs::File, program: &Path) -> Option<String> {
    let has_record: bool = fs::canonicalize(program).ok().is_some_and(|resolved| { return record_target(&resolved).is_some(); });
    if has_record {
        return shadow_behind(program);
    }
    return identity_of_open_file(file);
}

/// The first bytes of an ELF file.
const ELF_MAGIC: [u8; 4] = *b"\x7fELF";

/// `O_PATH` (`open(2)`):
/// the open file names the inode and is never read or written;
/// the same value on every Linux architecture built.
const OPEN_BY_PATH_ONLY: i32 = 0o10_000_000;

/// What:
/// `pub struct Vetted { ... }` is a program chosen as the real tool,
/// opened once and checked while open.
/// Why:
/// The choice of the real tool and its start are two steps,
/// and a path can be made to name a shadow between them.
/// A compiled program is started through the file that was checked,
/// so the swap cannot reach it.
pub struct Vetted {
    /// The open file (close-on-exec),
    /// which keeps the checked inode alive until the start.
    file: fs::File,
    /// The path the program was chosen under,
    /// kept as its own name (`argv[0]`):
    /// some programs pick what to do by the name they were started under.
    spelled: PathBuf,
    /// Whether the file can be started through the open file:
    /// a readable compiled program that no launcher record stands beside.
    through_descriptor: bool,
}

/// What:
/// `pub fn vet(program: &Path) -> Result<Vetted, String>` opens the chosen program and refuses it when it leads to a shadow.
/// Why:
/// The lookup skipped every shadow it could see;
/// this is the check at the moment of the start,
/// on the open file.
/// A file that cannot be read (execute-only) is opened by `O_PATH` and started by its path.
pub fn vet(program: &Path) -> Result<Vetted, String> {
    let (file, readable): (fs::File, bool) = match fs::File::open(program) {
        Ok(opened) => (opened, true),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            use std::os::unix::fs::OpenOptionsExt;
            let by_path: fs::File = fs::OpenOptions::new()
                .read(true)
                .custom_flags(OPEN_BY_PATH_ONLY)
                .open(program)
                .map_err(|second| { return format!("{} cannot be opened ({error}; by path only: {second})", program.display()); })?;
            (by_path, false)
        }
        Err(error) => return Err(format!("{} cannot be opened ({error})", program.display())),
    };
    if let Some(identity) = identity_of_open_file_unless_launcher(&file, program) {
        return Err(format!("{} is or leads to {identity}, a shadow, so it was not started", program.display()));
    }
    let mut magic: [u8; 4] = [0; 4];
    let looks_compiled: bool = readable && file.read_exact_at(&mut magic, 0).is_ok() && magic == ELF_MAGIC;
    let has_record: bool = fs::canonicalize(program).ok().is_some_and(|resolved| { return record_target(&resolved).is_some(); });
    let through_descriptor: bool = looks_compiled && !has_record && Path::new("/proc/self/fd").is_dir();
    return Ok(Vetted { file, spelled: program.to_path_buf(), through_descriptor });
}

impl Vetted {
    /// What:
    /// `pub fn command(&self) -> Command` is the command that starts the vetted program.
    /// Why:
    /// A compiled program is started through the open file (`/proc/self/fd/<n>` names the inode that was checked,
    /// wherever the path points now),
    /// with its own name as `argv[0]`.
    /// A script,
    /// an unreadable program and a launcher keep their path
    /// (an interpreter opens the path after the descriptor is closed,
    /// and a launcher finds its record by its own path);
    /// a swap in the instants before their start ends at the arrival stop of the shadow it reaches.
    pub fn command(&self) -> Command {
        use std::os::fd::AsRawFd;
        use std::os::unix::process::CommandExt;
        if self.through_descriptor {
            let mut command: Command = Command::new(format!("/proc/self/fd/{}", self.file.as_raw_fd()));
            command.arg0(&self.spelled);
            return command;
        }
        return Command::new(&self.spelled);
    }
}

/// How a shadow starts the program it chose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartMode {
    /// The shadow replaces itself with the program:
    /// the process number stays.
    Exec,
    /// The shadow runs the program as a child and waits:
    /// the child's parent is the shadow.
    Spawn,
}

/// What an arrival marker says,
/// taken apart.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Arrival {
    /// How the starting shadow started the program.
    pub mode: StartMode,
    /// The process number of the starting shadow.
    pub pid: u32,
    /// The birth identity of that process,
    /// which changes when its number is reused.
    pub birth: String,
    /// The crate name and version of the starting shadow.
    pub starter: String,
}

/// What:
/// `pub fn birth_in(stat_text: &str) -> String` is the start time in clock ticks since boot that a `/proc/<pid>/stat` text names (field 22),
/// or `?` when the text has no such field.
/// Why:
/// The name field of that file may hold spaces and brackets,
/// so the fields are counted after its last closing bracket.
pub fn birth_in(stat_text: &str) -> String {
    let Some((_, after_name)) = stat_text.rsplit_once(')') else {
        return "?".to_string();
    };
    return after_name.split_whitespace().nth(19).unwrap_or("?").to_string();
}

/// What:
/// `pub fn process_birth(pid: u32) -> String` is the birth identity of a process,
/// or `?` when it cannot be read.
/// Why:
/// A process number alone can name a later process once the first has ended;
/// the start time cannot.
pub fn process_birth(pid: u32) -> String {
    return fs::read_to_string(format!("/proc/{pid}/stat")).map_or_else(|_| { return "?".to_string(); }, |text| { return birth_in(&text); });
}

/// What:
/// `pub fn arrival_value(mode: StartMode) -> String` is the marker a shadow puts in the environment of the program it starts.
/// Why:
/// The program,
/// if it is a shadow,
/// finds that it was started by this process,
/// and stops.
pub fn arrival_value(mode: StartMode) -> String {
    let word: &str = match mode {
        StartMode::Exec => "exec",
        StartMode::Spawn => "spawn",
    };
    let pid: u32 = std::process::id();
    return format!("{word} {pid} {} {}", process_birth(pid), own_identity());
}

/// What:
/// `pub fn parse_arrival(text: &str) -> Option<Arrival>` reads what `arrival_value` wrote;
/// any other text is `None`.
/// Why:
/// A value left by anything else,
/// or a damaged one,
/// is never believed:
/// the shadow behaves as if started fresh.
pub fn parse_arrival(text: &str) -> Option<Arrival> {
    let mut fields = text.splitn(4, ' ');
    let mode: StartMode = match fields.next()? {
        "exec" => StartMode::Exec,
        "spawn" => StartMode::Spawn,
        _ => return None,
    };
    let pid: u32 = fields.next()?.parse().ok()?;
    let birth: String = fields.next()?.to_string();
    let starter: String = fields.next()?.to_string();
    return Some(Arrival { mode, pid, birth, starter });
}

/// What:
/// `pub fn is_meant_for(arrival: &Arrival, own: (u32, &str), parent: (u32, &str)) -> bool` says whether this process is the one the marker was left for.
/// Why:
/// A shadow that replaced itself is the very process the marker names (same number,
/// same birth);
/// a shadow that started a child is that child's parent.
/// Anyone else,
/// such as a script the real tool runs,
/// passes.
pub fn is_meant_for(arrival: &Arrival, own: (u32, &str), parent: (u32, &str)) -> bool {
    let (pid, birth): (u32, &str) = match arrival.mode {
        StartMode::Exec => own,
        StartMode::Spawn => parent,
    };
    return arrival.pid == pid && arrival.birth == birth;
}

/// What:
/// `pub fn arrival_at_this_process() -> Option<Arrival>` is the marker left for this process,
/// when there is one.
/// Why:
/// It reads the environment and the two process identities once;
/// the shim stops when it answers `Some`.
pub fn arrival_at_this_process() -> Option<Arrival> {
    let text: String = std::env::var(ARRIVAL_VARIABLE).ok()?;
    let arrival: Arrival = parse_arrival(&text)?;
    let own_pid: u32 = std::process::id();
    let parent_pid: u32 = std::os::unix::process::parent_id();
    let meant: bool = is_meant_for(&arrival, (own_pid, &process_birth(own_pid)), (parent_pid, &process_birth(parent_pid)));
    return if meant { Some(arrival) } else { None };
}

/// What:
/// `pub fn arrival_line(tool: &str, arrival: &Arrival) -> String` is the one line printed when a shadow was started by a shadow.
/// Why:
/// It names the stopped shadow,
/// the shadow that started it,
/// the tool and what to do,
/// so a reader who knows nothing about shadows can act.
pub fn arrival_line(tool: &str, arrival: &Arrival) -> String {
    return format!(
        "{own}: stopped at once, before doing anything, because it was started by {starter} (process {pid}) as the real `{tool}`; a shadow never starts a shadow. \
         Put the real `{tool}` before the shadows on PATH, or remove the program on PATH that leads back to a shadow. Exit status {ARRIVAL_EXIT_CODE} means this stop.",
        own = own_identity(),
        starter = arrival.starter,
        pid = arrival.pid,
    );
}

/// What:
/// `pub fn mark_start(command: &mut Command, mode: StartMode)` adds the arrival marker to the environment of a command that starts the real program.
/// Why:
/// Every start of a real program carries it,
/// so a shadow among the programs reached (a launcher with no record,
/// a script) stops on arrival.
pub fn mark_start(command: &mut Command, mode: StartMode) {
    command.env(ARRIVAL_VARIABLE, arrival_value(mode));
}

/// What:
/// `pub fn is_same_file(first: &Path, second: &Path) -> bool` says whether two paths name one file (the same device and inode).
/// Why:
/// Two names of one program (hard links) have different resolved paths and one identity.
pub fn is_same_file(first: &Path, second: &Path) -> bool {
    return match (fs::metadata(first), fs::metadata(second)) {
        (Ok(left), Ok(right)) => left.dev() == right.dev() && left.ino() == right.ino(),
        _ => false,
    };
}
