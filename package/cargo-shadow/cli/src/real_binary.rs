//! What:
//! Finds the real `cargo` on PATH while skipping the shadow itself.
//! Why:
//! The shadow is first on PATH under the name `cargo`,
//! so a plain PATH search would find the shadow again and run
//! itself forever.
//! Every candidate that resolves to this executable is skipped,
//! not only the first,
//! because a
//! repository directory and a global directory can both hold a link to it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // real-binary.ts
//! export function findInPath(name: string, pathVariable: string, skip: string): string | undefined { ... }
//! ```

/// Imports the file-system module (resolves symlinks,
/// reads permissions).
use std::fs;

/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports a trait (a named bundle of methods that types opt into,
/// like a
/// TS interface) that makes `.mode()` available on file permissions.
/// Why:
/// The execute bits live in the Unix mode number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: in TS `stats.mode` is always available.
/// ```
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// What:
/// `const EXECUTE_BITS: u32 = 0o111;` is an unsigned 32-bit integer (siblings:
/// `i32`,
/// `u64`).
/// The `0o` prefix writes
/// it in base 8:
/// the owner,
/// group and other execute bits.
/// Why:
/// A file is runnable when any of those bits is set;
/// `u32` because `.mode()` returns that type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EXECUTE_BITS = 0o111;
/// ```
const EXECUTE_BITS: u32 = 0o111;

/// What:
/// `fn is_executable_file(path: &Path) -> bool` is a private function over a borrowed path.
/// Why:
/// The search must ignore directories,
/// missing files and files without an execute bit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExecutableFile(path: string): boolean { return statSync(path, { throwIfNoEntry: false })?.isFile() === true && (statSync(path).mode & 0o111) !== 0; }
/// ```
fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    // `fs::metadata(path)` returns a `Result`; `if let Ok(metadata)` runs the block only on success.
    if let Ok(metadata) = fs::metadata(path) {
        return (metadata.permissions().mode() & EXECUTE_BITS) != 0;
    }
    return false;
}

/// What:
/// `pub fn find_in_path(name: &str, path_variable: &str, skip: &[PathBuf]) -> Option<PathBuf>` takes three borrowed
/// inputs and returns `Some(path)` when found,
/// `None` when not.
/// `Option` is Rust's "value or nothing" (a TS
/// `T | undefined`).
/// `&[PathBuf]` is a borrowed view of a list of owned paths (siblings:
/// `Vec<PathBuf>`,
/// the owned list).
/// Why:
/// `skip` holds resolved paths the loop must never return:
/// the shadow's own executable,
/// and every candidate an earlier
/// shadow instance already picked (`guard.rs`),
/// because a launcher or other link that leads back to the shadow resolves to
/// a path that is not the shadow's own.
/// `PathBuf` (owned) and not `&Path` (borrowed) because the result outlives the
/// borrowed inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findInPath(name: string, pathVariable: string, skip: string[]): string | undefined
/// ```
pub fn find_in_path(name: &str, path_variable: &str, skip: &[PathBuf]) -> Option<PathBuf> {
    return find_in_path_with_image(name, path_variable, skip, running_image_identity());
}

/// What:
/// `pub fn find_in_path_with_image(name, path_variable, skip, image) -> Option<PathBuf>` is `find_in_path` with the identity of the
/// running executable image given.
/// Why:
/// The identity of the image that is running comes from the open image, not from a pathname: an installer may replace the installed
/// path while the shadow runs, after which the path names another file (or reads with a ` (deleted)` suffix) and a hard link of the
/// image still on PATH would not be recognized. A test gives the identity it took before such a replacement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findInPathWithImage(name: string, pathVariable: string, skip: string[], image: [bigint, bigint] | undefined): string | undefined
/// ```
pub fn find_in_path_with_image(name: &str, path_variable: &str, skip: &[PathBuf], image: Option<(u64, u64)>) -> Option<PathBuf> {
    // The skipped executables by file identity: a hard link to the shadow resolves to another pathname than the shadow's own and is
    // still the same file, so the pathname alone cannot tell it (certification, third pass); the running image is one of them.
    let mut skipped: Vec<(u64, u64)> = skip.iter().filter_map(|path| { return identity_of(path); }).collect();
    skipped.extend(image);
    for directory in path_variable.split(':') {
        // An empty PATH entry means the current directory in a shell; the shadow ignores it on purpose.
        if directory.is_empty() {
            continue;
        }
        let candidate: PathBuf = Path::new(directory).join(name);
        if !is_executable_file(&candidate) {
            continue;
        }

        // `fs::canonicalize(&candidate)` resolves every symlink to the final path, so every link to the shadow
        // resolves to the one executable and is recognized under whatever name it was found. The `&&` joins two
        // conditions inside one `if let`.
        if identity_of(&candidate).is_some_and(|found| { return skipped.contains(&found); }) {
            continue;
        }
        if let Ok(resolved) = fs::canonicalize(&candidate)
            && (skip.contains(&resolved) || leads_back_to(&resolved, skip, &skipped))
        {
            continue;
        }

        // `Some(candidate)` wraps the found path in the present case of `Option`: the first real cargo in PATH order wins.
        return Some(candidate);
    }
    return None;
}

/// The suffix of the record that the toolchain writes beside a launcher (`mpm-shadow`, `package/mpm-shadow/cli/README.md`, section "The launcher").
pub const LAUNCH_RECORD_SUFFIX: &str = ".launch";

/// How many launchers in a row are followed before the chain is taken as one that never reaches a program: a record chain longer than
/// this is a loop, and a loop leads back.
const MAX_LAUNCHER_HOPS: usize = 8;

/// What:
/// `pub fn refuse_if_ourselves(program: &Path) -> Result<(), String>` fails when the file a chosen program names is now the running image
/// (or a recorded launcher that leads to it).
/// Why:
/// The choice of the real cargo and its start are two steps, and a path can be made to name the shadow between them (an ordinary
/// installation swapping a link). The check is repeated right before each start, so the window shrinks to the instants between this
/// check and the start itself; the shadow no longer starts itself for a path that already named it when the start was made
/// (fifth certification pass). Round 21 closes the window for a compiled program: `vet` opens the file once and `Vetted::command`
/// starts it through that descriptor. A script, and the cargo of an install step (started by the scope's launcher from its own
/// process), keep the pathname and this repeated check; their remaining window ends at the arrival stop of the second cargo-shadow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function refuseIfOurselves(program: string): void  // throws the reason
/// ```
pub fn refuse_if_ourselves(program: &Path) -> Result<(), String> {
    let Some(image) = running_image_identity() else {
        return Ok(());
    };
    let named_us: bool = identity_of(program) == Some(image) || fs::canonicalize(program).is_ok_and(|resolved| { return leads_back_to(&resolved, &[], &[image]); });
    if named_us {
        return Err(format!("{} names this program itself at the moment of the start, so it was not started", program.display()));
    }
    return Ok(());
}

/// `O_PATH` (`open(2)`): the open file names the inode and is never read or written; the same value on every Linux architecture the
/// shadow is built for (x86-64 and aarch64 use the generic `0o10000000`).
const OPEN_BY_PATH_ONLY: i32 = 0o10_000_000;

/// A program chosen as the real cargo, opened once: every check ran on the open file, and a compiled program is started through it.
pub struct Vetted {
    /// The open file (close-on-exec). It keeps the inode that was checked alive until the start.
    file: fs::File,
    /// The path the program was chosen under, kept as the program's own name (`argv[0]`), so a proxy such as rustup's still picks the
    /// tool by the name it was started under.
    spelled: PathBuf,
    /// Whether the file is a compiled (ELF) program, which can be started through the open file.
    compiled: bool,
}

// A test swaps what the chosen path names between the check and the start, right where a swap would hurt.
#[cfg(test)]
thread_local! {
    static AFTER_CHECK: std::cell::RefCell<Option<std::rc::Rc<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
/// `pub fn after_check_for_test(hook: Option<Rc<dyn Fn()>>)` sets the hook that runs between the check and the start of a chosen program.
/// Why:
/// The race of the fifth certification pass (group 15, b) is a path made to name the shadow after it was checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function afterCheckForTest(hook?: () => void): void
/// ```
#[cfg(test)]
pub fn after_check_for_test(hook: Option<std::rc::Rc<dyn Fn()>>) {
    AFTER_CHECK.with(|slot| { *slot.borrow_mut() = hook; });
}

/// What:
/// `pub fn vet(program: &Path) -> Result<Vetted, String>` opens the chosen program and checks the open file: it must not be the running
/// image, nor a recorded launcher that leads to it.
/// Why:
/// The check and the start used to be two uses of a pathname, and a path can be made to name the shadow in between: the shadow would
/// start itself (owner rule: "We should never invoke ourselves recursively."). A compiled program is now started through the file that
/// was checked, so the swap cannot reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function vet(program: string): Vetted  // throws the reason
/// ```
pub fn vet(program: &Path) -> Result<Vetted, String> {
    use std::io::Read;
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::MetadataExt;
    // Executing a compiled program needs no read permission, and the PATH search accepts an execute-only file, so a refused read is not
    // a refusal to start: the file is opened by `O_PATH` instead (an identity and a name for `/proc/self/fd/<n>`, no reading). A script
    // that cannot be read cannot run either (its interpreter opens it), so what cannot be read is taken as a compiled program.
    let (file, readable): (fs::File, bool) = match fs::File::open(program) {
        Ok(opened) => (opened, true),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            use std::os::unix::fs::OpenOptionsExt;
            let by_path: fs::File = fs::OpenOptions::new().read(true).custom_flags(OPEN_BY_PATH_ONLY).open(program).map_err(|second| { return format!("{} cannot be opened ({error}; by path only: {second})", program.display()); })?;
            (by_path, false)
        }
        Err(error) => return Err(format!("{} cannot be opened ({error})", program.display())),
    };
    let metadata: fs::Metadata = file.metadata().map_err(|error| { return format!("{} cannot be examined ({error})", program.display()); })?;
    let image: Option<(u64, u64)> = running_image_identity();
    let named_us: bool = image == Some((metadata.dev(), metadata.ino()))
        || image.is_some_and(|found| { return fs::canonicalize(format!("/proc/self/fd/{}", file.as_raw_fd())).is_ok_and(|resolved| { return leads_back_to(&resolved, &[], &[found]); }); });
    if named_us {
        return Err(format!("{} names this program itself at the moment of the start, so it was not started", program.display()));
    }
    let mut magic: [u8; 4] = [0; 4];
    let looks_compiled: bool = if readable { (&file).read_exact(&mut magic).is_ok() && magic == *b"\x7fELF" } else { true };
    let compiled: bool = looks_compiled && Path::new("/proc/self/fd").is_dir();
    return Ok(Vetted { file, spelled: program.to_path_buf(), compiled });
}

/// The methods of the type above.
impl Vetted {
    /// What:
    /// `pub fn command(&self) -> Result<Command, String>` is the command that starts the vetted program.
    /// Why:
    /// A compiled program is started through the open file (`/proc/self/fd/<n>` names the inode that was checked, wherever the path
    /// points now), with its own name as `argv[0]`. A script cannot be: an interpreter opens the path it is given after the descriptor
    /// is closed, so a script keeps its pathname, checked again right here; the instants between this check and the start remain for
    /// scripts, and a hit ends at the arrival stop of the second cargo-shadow (exit 2, nothing runs twice).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// command(): Command  // throws the reason
    /// ```
    pub fn command(&self) -> Result<std::process::Command, String> {
        use std::os::fd::AsRawFd;
        use std::os::unix::process::CommandExt;
        #[cfg(test)]
        if let Some(hook) = AFTER_CHECK.with(|slot| { return slot.borrow().clone(); }) {
            hook();
        }
        if self.compiled {
            let mut command: std::process::Command = std::process::Command::new(format!("/proc/self/fd/{}", self.file.as_raw_fd()));
            command.arg0(&self.spelled);
            return Ok(command);
        }
        return self.command_by_path();
    }

    /// What:
    /// `pub fn command_by_path(&self) -> Result<Command, String>` is the command that starts the program by the path it was chosen
    /// under, after the check is repeated.
    /// Why:
    /// A program started by another process (a scope's launcher takes its path as an argument and starts it itself) cannot be named
    /// by a descriptor of this one, and a script cannot be started through a descriptor at all. The instants between the repeated
    /// check and the start remain for these starts; a hit there reaches the arrival stop of the second cargo-shadow (exit status 2),
    /// so nothing runs twice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// commandByPath(): Command  // throws the reason
    /// ```
    pub fn command_by_path(&self) -> Result<std::process::Command, String> {
        refuse_if_ourselves(&self.spelled)?;
        return Ok(std::process::Command::new(&self.spelled));
    }
}

/// What:
/// `pub fn running_image_identity() -> Option<(u64, u64)>` is the device and inode of the executable image this process runs.
/// Why:
/// `/proc/self/exe` opened as a file names the open image even when its installed path was replaced or removed; `None` where it
/// cannot be opened, in which case only the pathname-based skips apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runningImageIdentity(): [bigint, bigint] | undefined
/// ```
pub fn running_image_identity() -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    return fs::File::open("/proc/self/exe").and_then(|image| { return image.metadata(); }).ok().map(|found| { return (found.dev(), found.ino()); });
}

/// What:
/// `fn identity_of(path: &Path) -> Option<(u64, u64)>` is the device and inode of the file a path names, through links.
/// Why:
/// Two names of one executable (hard links) have different resolved pathnames and one identity; `None` for a path that cannot be
/// examined, which then matches nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function identityOf(path: string): [bigint, bigint] | undefined { const s = statSync(path, { bigint: true, throwIfNoEntry: false }); return s && [s.dev, s.ino]; }
/// ```
fn identity_of(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    return fs::metadata(path).ok().map(|found| { return (found.dev(), found.ino()); });
}

/// What:
/// `fn leads_back_to(resolved: &Path, skip: &[PathBuf]) -> bool` says whether a PATH candidate is a launcher whose record names a
/// binary that is one of the skipped paths, directly or through further launchers.
/// Why:
/// The toolchain puts a launcher between the link on PATH and the shadow: the launcher is a file of its own, so its resolved path is
/// not the shadow's, yet starting it starts the shadow again. The record beside the launcher (`<launcher>.launch`, a JSON object
/// with the `real` binary) names the program it starts, and reading it starts nothing, so the candidate is skipped before it is run
/// and no second instance of the shadow exists. An unreadable or unparsable record is not a launcher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leadsBackTo(resolved: string, skip: string[]): boolean
/// ```
fn leads_back_to(resolved: &Path, skip: &[PathBuf], skipped: &[(u64, u64)]) -> bool {
    let mut current: PathBuf = resolved.to_path_buf();
    for _hop in 0..MAX_LAUNCHER_HOPS {
        let mut record_name: std::ffi::OsString = current.as_os_str().to_os_string();
        record_name.push(LAUNCH_RECORD_SUFFIX);
        let Ok(text) = fs::read_to_string(PathBuf::from(record_name)) else {
            return false;
        };
        let Some(real) = serde_json::from_str::<serde_json::Value>(&text).ok().and_then(|value| { return value.get("real")?.as_str().map(PathBuf::from); }) else {
            return false;
        };
        let Ok(target) = fs::canonicalize(&real) else {
            return false;
        };
        if skip.contains(&target) || identity_of(&target).is_some_and(|found| { return skipped.contains(&found); }) {
            return true;
        }
        current = target;
    }
    return true;
}

/// What:
/// `pub fn own_real_path() -> PathBuf` is the shadow's own resolved executable path,
/// or an empty path when the system cannot
/// say.
/// Why:
/// The PATH search skips every entry that resolves to it;
/// an empty path matches nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ownRealPath(): string { return fs.realpathSync(process.execPath); }
/// ```
pub fn own_real_path() -> PathBuf {
    if let Ok(executable) = std::env::current_exe() {
        if let Ok(resolved) = fs::canonicalize(&executable) {
            return resolved;
        }
        return executable;
    }
    return PathBuf::new();
}

/// What:
/// `pub fn find_real_cargo() -> Option<PathBuf>` is the first `cargo` on PATH that is not this binary.
/// Why:
/// The install step and the catalog commands run cargo themselves,
/// and they must never run the shadow again.
/// The result keeps
/// the PATH entry's own name (not its resolved path):
/// `~/.cargo/bin/cargo` is a rustup proxy that picks the toolchain from
/// the name it was started under,
/// so only the skip test uses resolved paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findRealCargo(): string | undefined { return findInPath('cargo', process.env.PATH ?? '', [ownRealPath()]); }
/// ```
pub fn find_real_cargo() -> Option<PathBuf> {
    let path_variable: String = std::env::var("PATH").unwrap_or_default();
    return find_in_path(crate::constants::SHADOWED_NAME, &path_variable, &[own_real_path()]);
}
