//! What:
//! The message of a cargo-shadow that was started by another cargo-shadow's own start of the real `cargo`.
//! Why:
//! A program on PATH named `cargo` that leads back to cargo-shadow (a script or launcher that has no `.launch` record beside it)
//! would make cargo-shadow start itself again without end. The second start stops at once, before doing any work, with exit status 2,
//! and tells a reader who knows nothing about cargo-shadow what happened and what to type, step by step (owner decision of
//! 2026-11-04: option A, "make the help message descriptive enough that even a monkey can understand and execute it").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function arrivalMessage(input: ArrivalInput): string
//! ```

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::guard::{Guard, GuardKind};
use crate::real_binary::{LAUNCH_RECORD_SUFFIX, running_image_identity};

/// The exit status of a cargo-shadow that was started by cargo-shadow itself.
pub const ARRIVAL_EXIT_CODE: i32 = 2;

/// What:
/// `pub fn shell_quote(text: &str) -> String` quotes text for a POSIX shell inside single quotes.
/// Why:
/// Every command in the message works as printed, whatever the paths contain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const shellQuote = (text: string): string => `'${text.replaceAll("'", "'\\''")}'`;
/// ```
pub fn shell_quote(text: &str) -> String {
    return format!("'{}'", text.replace('\'', "'\\''"));
}

/// One `cargo` found on PATH, in PATH order, and what it is.
struct Found {
    /// The path as PATH spells it.
    spelled: PathBuf,
    /// What the message says about it.
    note: &'static str,
    /// Whether it is a program that may be the real cargo.
    candidate: bool,
}

/// The `cargo` files of a PATH, in order, each classified.
fn cargo_files(path_variable: &str, chosen: Option<&Path>) -> Vec<Found> {
    let image: Option<(u64, u64)> = running_image_identity();
    let mut found: Vec<Found> = Vec::new();
    for directory in path_variable.split(':').filter(|entry| { return !entry.is_empty(); }) {
        let file: PathBuf = Path::new(directory).join("cargo");
        let Ok(metadata) = fs::metadata(&file) else {
            continue;
        };
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            continue;
        }
        let resolved: Option<PathBuf> = fs::canonicalize(&file).ok();
        let is_us: bool = image == Some((metadata.dev(), metadata.ino()));
        let is_chosen: bool = chosen.is_some_and(|picked| { return resolved.as_deref() == Some(picked); });
        let recorded: bool = resolved.as_ref().is_some_and(|path| {
            let mut record: std::ffi::OsString = path.as_os_str().to_os_string();
            record.push(LAUNCH_RECORD_SUFFIX);
            return Path::new(&record).is_file();
        });
        let (note, candidate) = if is_us {
            ("this is cargo-shadow itself", false)
        } else if is_chosen {
            ("cargo-shadow chose this file as the real cargo, and it started cargo-shadow again", false)
        } else if recorded {
            ("a launcher with a `.launch` record, which cargo-shadow skips on its own", false)
        } else {
            ("not cargo-shadow as far as can be seen without running it", true)
        };
        found.push(Found { spelled: file, note, candidate });
    }
    return found;
}

/// What:
/// `fn launch_record_text(own_executable: &Path) -> String` is the whole text of the `.launch` record that names this program.
/// Why:
/// The printed command must write a record that parses whatever the path holds (quotation marks, backslashes, control characters), so
/// the JSON is built by the serializer, never by filling a template (round 21, sixth pass, group 14).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const launchRecordText = (ownExecutable: string): string => JSON.stringify({ toolId: 'cargo-shadow', lockFile: '/dev/null', real: ownExecutable, scope: 'repo' });
/// ```
fn launch_record_text(own_executable: &Path) -> String {
    let mut fields = serde_json::Map::new();
    for (name, value) in [("toolId", "cargo-shadow"), ("lockFile", "/dev/null"), ("real", &own_executable.to_string_lossy()), ("scope", "repo")] {
        fields.insert(name.to_string(), serde_json::Value::String(value.to_string()));
    }
    return serde_json::Value::Object(fields).to_string();
}

/// What:
/// `pub fn arrival_message(guard: &Guard, own_executable: &Path, path_variable: &str) -> String` is the whole text printed when
/// this cargo-shadow was started by another one.
/// Why:
/// It states in one sentence what happened, shows the chain with full paths, the `PATH` this program saw, why it stops, and numbered
/// steps with commands that work as printed, then how to check the fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function arrivalMessage(guard: Guard, ownExecutable: string, pathVariable: string): string
/// ```
pub fn arrival_message(guard: &Guard, own_executable: &Path, path_variable: &str) -> String {
    let chosen: Option<&Path> = guard.skipped.last().map(PathBuf::as_path);
    let chosen_text: String = chosen.map_or_else(|| { return "(a file that is not known any more)".to_string(); }, |path| { return path.display().to_string(); });
    let starter: String = match guard.kind {
        GuardKind::Exec => format!("an earlier cargo-shadow (process {}) replaced itself with that file", guard.pid),
        GuardKind::Spawn => {
            let program: String = fs::read_link(format!("/proc/{}/exe", guard.pid)).map_or_else(|_| { return "a program that has ended".to_string(); }, |path| { return path.display().to_string(); });
            format!("an earlier cargo-shadow (process {}, the program {program}) started that file as a child", guard.pid)
        }
    };
    let files: Vec<Found> = cargo_files(path_variable, chosen);
    let listing: String = if files.is_empty() {
        "    (no file named `cargo` was found on that PATH)\n".to_string()
    } else {
        files.iter().enumerate().map(|(index, one)| { return format!("    {}. {}  ({})\n", index + 1, one.spelled.display(), one.note); }).collect()
    };
    let steps_for_the_path: String = match files.iter().find(|one| { return one.candidate; }) {
        Some(real) => {
            let directory: String = real.spelled.parent().map_or_else(String::new, |parent| { return parent.display().to_string(); });
            format!(
                "  Fix A, put the real cargo first (this changes only the terminal you type it in):\n\
                 \x20   1. Type this line exactly and press Enter:\n\
                 \x20        export PATH={}:\"$PATH\"\n\
                 \x20   2. Check that it worked: type `cargo --version` and press Enter. A version line such as `cargo 1.x.y` means it works.\n",
                shell_quote(&directory)
            )
        }
        None => "  Fix A, put the real cargo first: no other `cargo` was found on that PATH. Install Rust (https://rustup.rs) so that a real\n    `cargo` exists, then follow Fix A again.\n".to_string(),
    };
    let steps_for_the_record: String = match chosen {
        Some(path) => format!(
            "  Fix B, tell cargo-shadow that the file it chose is a launcher of cargo-shadow (the permanent fix, done once):\n\
             \x20   1. Type this line exactly and press Enter (it writes the file {record}):\n\
             \x20        printf %s {record_text} > {record_quoted}\n\
             \x20   2. Check that it worked: type `cargo --version` and press Enter.\n",
            record = format_args!("{}{}", path.display(), LAUNCH_RECORD_SUFFIX),
            record_text = shell_quote(&launch_record_text(own_executable)),
            record_quoted = shell_quote(&format!("{}{}", path.display(), LAUNCH_RECORD_SUFFIX)),
        ),
        None => String::new(),
    };
    return format!(
        "cargo-shadow: stopped at once, before doing anything, because cargo-shadow was started by cargo-shadow.\n\
         \n\
         What happened, in one sentence: the program named `cargo` that was run started cargo-shadow, cargo-shadow looked for the real\n\
         cargo and chose {chosen_text}, and that file started cargo-shadow again instead of cargo.\n\
         \n\
         The chain, from the first program to this one:\n\
         \x20 1. a program named `cargo` (what you typed, or a script that ran it) started cargo-shadow;\n\
         \x20 2. {starter};\n\
         \x20 3. that file is the one that started this cargo-shadow, the program {own}.\n\
         \n\
         The PATH this program saw (the list of folders searched for `cargo`, in order):\n\
         \x20 {path_variable}\n\
         \n\
         The files named `cargo` on that PATH, in the order they are searched:\n\
         {listing}\
         \n\
         Why it stops: if it went on, cargo-shadow would start itself again and again and never run cargo.\n\
         \n\
         What to do (either fix works; copy the lines into the terminal where you saw this):\n\
         {steps_for_the_path}\
         {steps_for_the_record}\
         \n\
         Exit status 2 means this stop.\n",
        own = own_executable.display(),
    );
}
