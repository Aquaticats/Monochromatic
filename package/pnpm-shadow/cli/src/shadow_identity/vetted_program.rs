//! What:
//! The program chosen as the real tool,
//! opened once and checked while open,
//! and the command that starts it through that open file.
//! Why:
//! The choice and the start are two steps,
//! and a path can be made to name a shadow between them;
//! starting the checked inode closes that window.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Vetted = { file: FileHandle; spelled: string; throughDescriptor: boolean };
//! function vet(program: string): Vetted { /* refuse a shadow, open O_PATH when unreadable */ }
//! ```
/// Imports the file-system module that opens and reads the chosen program.
use std::fs;
/// Imports the positioned-read trait for the open file.
use std::os::unix::fs::FileExt;
/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the command type that starts the vetted program.
use std::process::Command;

/// Imports the launcher record checks.
use super::launcher_chain::{
    has_launcher_record,
    identity_of_open_file_unless_launcher,
};

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
            /// Imports the trait that adds `custom_flags` to the open options.
            use std::os::unix::fs::OpenOptionsExt;
            let by_path_result: std::io::Result<fs::File> = fs::OpenOptions::new()
                .read(true)
                .custom_flags(OPEN_BY_PATH_ONLY)
                .open(program);
            let by_path: fs::File = match by_path_result {
                Ok(opened) => opened,
                Err(second) => {
                    return Err(format!(
                        "{} cannot be opened ({error}; by path only: {second})",
                        program.display()
                    ));
                }
            };
            (by_path, false)
        }
        Err(error) => return Err(format!("{} cannot be opened ({error})", program.display())),
    };
    if let Some(identity) = identity_of_open_file_unless_launcher(&file, program) {
        return Err(format!(
            "{} is or leads to {identity}, a shadow, so it was not started",
            program.display()
        ));
    }
    let mut magic: [u8; 4] = [0; 4];
    let looks_compiled: bool =
        readable && file.read_exact_at(&mut magic, 0).is_ok() && magic == ELF_MAGIC;
    let has_record: bool = has_launcher_record(program);
    let through_descriptor: bool =
        looks_compiled && !has_record && Path::new("/proc/self/fd").is_dir();
    return Ok(Vetted {
        file,
        spelled: program.to_path_buf(),
        through_descriptor,
    });
}

/// The command that starts the program that was vetted.
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
        /// Imports the trait that gives an open file its descriptor number.
        use std::os::fd::AsRawFd;
        /// Imports the trait that adds `arg0` to a command.
        use std::os::unix::process::CommandExt;
        if self.through_descriptor {
            let mut command: Command =
                Command::new(format!("/proc/self/fd/{}", self.file.as_raw_fd()));
            command.arg0(&self.spelled);
            return command;
        }
        return Command::new(&self.spelled);
    }
}
