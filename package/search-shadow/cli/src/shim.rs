//! What:
//! The shadow mode:
//! when the binary is started under a shadowed name (`rg` / `grep` / `find` and so on),
//! it checks the policy,
//! finds the real tool on PATH,
//! lowers its own disk priority to the lowest best-effort level,
//! and replaces itself with the real tool (which inherits that priority).
//! A lone version or help flag instead runs the real tool as a child
//! and then adds the shadow's own line or section (`announce.rs`).
//! Why:
//! Every search an agent or a script starts goes through here,
//! so a broad search yields the disk to the desktop instead of stalling it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.ts
//! export function runShim(name: string, args: Buffer[]): number { ... }
//! ```

/// What:
/// `use std::env;` imports the module for the process environment
/// (variables / arguments / the working directory).
/// Why:
/// We read PATH and the escape-hatch variable and the working directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.env and process.cwd() are globals in Node.
/// ```
use std::env;

/// Imports the owned operating-system text type (see `launch.rs`).
use std::ffi::OsString;

/// Imports the file-system module for the symlink resolution and the stdin check.
use std::fs;

/// What:
/// `use std::os::unix::fs::FileTypeExt;` imports a trait that adds `.is_fifo()` and `.is_socket()` to file types.
/// Why:
/// A pipe on stdin is a FIFO,
/// and the check needs to tell it from `/dev/null`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // stats.isFIFO() and stats.isSocket() are always available in Node.
/// ```
use std::os::unix::fs::FileTypeExt;

/// Imports the owned path type.
use std::path::PathBuf;

/// Imports the check for a lone version or help flag and the runner that adds the shadow's text after it.
use crate::announce::{announcement_for, run_announced};

/// Imports the escape-hatch variable name and the exit codes the shim returns.
use crate::constants::{ALLOW_WIDE_VARIABLE, EXEC_FAILED_EXIT_CODE, NOT_FOUND_EXIT_CODE, REFUSED_EXIT_CODE};

/// Imports the function that replaces the process with the real tool.
use crate::launch::replace_process;

/// Imports the record every policy reads and the function that asks the policies for a refusal.
use crate::policy::{Invocation, refusal_for};

/// Imports the function that lowers the disk priority.
use crate::priority::lower_disk_priority;

/// Imports the PATH lookup that finds the real tool.
use crate::real_binary::find_in_path;

/// What:
/// `fn own_real_path() -> PathBuf` declares a private function returning an owned path.
/// Why:
/// The PATH search skips every entry that resolves to this executable,
/// so it needs this executable's final resolved path.
/// Falls back to an empty path,
/// which matches nothing,
/// if the system cannot say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownRealPath(): string { return fs.realpathSync(process.execPath); }
/// ```
fn own_real_path() -> PathBuf {
    // What:     `if let Ok(executable) = env::current_exe()` runs the block when the system can name the running
    // executable; `Ok(x)` unwraps the success case of a `Result`.
    // Why:      On Linux this is already the symlink target, never the `rg` link.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let executable; try { executable = process.execPath; } catch {}
    // ```
    if let Ok(executable) = env::current_exe() {
        // `&executable` lends the path; on success the resolved path is returned.
        if let Ok(real) = fs::canonicalize(&executable) {
            return real;
        }
        return executable;
    }
    return PathBuf::new();
}

/// What:
/// `fn stdin_has_data() -> bool` declares a private function returning true or false.
/// Why:
/// With piped input and no path,
/// ripgrep reads the input and walks nothing,
/// so the policy must not refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stdinHasData(): boolean { const s = fs.fstatSync(0); return s.isFile() || s.isFIFO() || s.isSocket(); }
/// ```
fn stdin_has_data() -> bool {
    // What:     `/proc/self/fd/0` is a Linux path that stands for whatever file descriptor 0 (stdin) is attached to;
    // `fs::metadata` follows it to the real object. `if let Ok(metadata)` runs the block on success.
    // Why:      Reading the type avoids touching the stream. A closed stdin simply fails and counts as "no data".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const s = fs.statSync('/proc/self/fd/0', { throwIfNoEntry: false });
    // ```
    if let Ok(metadata) = fs::metadata("/proc/self/fd/0") {
        let file_type: fs::FileType = metadata.file_type();
        return file_type.is_file() || file_type.is_fifo() || file_type.is_socket();
    }
    return false;
}

/// What:
/// `pub fn run_shim(name: &str, arguments: Vec<OsString>) -> i32` declares a public function.
/// `Vec<OsString>` is an owned growable list of operating-system texts,
/// and the function takes ownership of it (the sibling `&[OsString]` would only borrow).
/// It returns `i32`,
/// the exit code.
/// Why:
/// On success it never returns (the process is replaced by the real tool),
/// so a returned value is always a refusal or a failure.
/// `Vec` and not a borrow because the arguments are moved into the launch list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runShim(name: string, args: Buffer[]): number { ... }
/// ```
pub fn run_shim(name: &str, arguments: Vec<OsString>) -> i32 {
    // What:     `env::var("PATH").unwrap_or_default()` reads the PATH variable. `env::var` returns a `Result`;
    // `.unwrap_or_default()` takes the success value or, on error, the empty text (the default for `String`).
    // Why:      A missing PATH simply means nothing will be found.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pathVariable = process.env.PATH ?? '';
    // ```
    let path_variable: String = env::var("PATH").unwrap_or_default();
    let own_path: PathBuf = own_real_path();

    // The policy sees lossy text copies of the arguments; the launch below uses the original bytes.
    let mut readable_arguments: Vec<String> = Vec::new();
    for argument in &arguments {
        // `argument.to_string_lossy().to_string()` converts to text, replacing invalid UTF-8 with a placeholder.
        readable_arguments.push(argument.to_string_lossy().to_string());
    }

    // What:     `Invocation { ... }` builds the record every policy reads. `.to_string()` makes an owned copy of the tool name;
    // `env::var("HOME").unwrap_or_default()` reads HOME as text (empty when unset) and `PathBuf::from(...)` turns
    // text into an owned path.
    // Why:      One value carries the facts the policies need: the arguments, the escape hatch, the stdin kind, and where
    // the tool will run.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const invocation = { name, arguments: readableArguments, allowWide, stdinHasData: stdinHasData(), workingDirectory: process.cwd(), home: process.env.HOME ?? '' };
    // ```
    let invocation: Invocation = Invocation {
        name: name.to_string(),
        arguments: readable_arguments,
        allow_wide: env::var(ALLOW_WIDE_VARIABLE).unwrap_or_default() == "1",
        stdin_has_data: stdin_has_data(),
        working_directory: env::current_dir().unwrap_or_default(),
        home: PathBuf::from(env::var("HOME").unwrap_or_default()),
    };

    // What:     `if let Some(message) = refusal_for(&invocation)` runs the block when a policy returned a refusal message.
    // `&invocation` lends the record; `eprintln!` prints a line to standard error (a macro, hence `!`).
    // Why:      Tell the caller why the tool did not run, then stop with the refusal exit code.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const message = refusalFor(invocation);
    // if (message !== undefined) { console.error(message); return 2; }
    // ```
    if let Some(message) = refusal_for(&invocation) {
        eprintln!("{message}");
        return REFUSED_EXIT_CODE;
    }

    // What:     `find_in_path(...)` returns `Option<PathBuf>`; `&own_path` lends the shadow's own path to skip.
    // `.is_none()` asks whether it is the absent case; `.unwrap_or_default()` then takes the path out, and
    // substitutes an empty path only in the absent case, which the check above already returned from.
    // Why:      Report a missing real tool once, clearly, before going on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = findInPath(name, pathVariable, ownPath);
    // if (found === undefined) { console.error('...'); return 127; }
    // const real = found;
    // ```
    let found: Option<PathBuf> = find_in_path(name, &path_variable, &own_path);
    if found.is_none() {
        eprintln!(
            "search-shadow: no real `{name}` found on PATH (the shadow links themselves are skipped). Install `{name}` or fix PATH."
        );
        return NOT_FOUND_EXIT_CODE;
    }
    let real: PathBuf = found.unwrap_or_default();

    // What:     `lower_disk_priority();` lowers this process's disk priority and ignores whether the kernel accepted it.
    // Why:      The real tool inherits the priority through the process replacement below. If the kernel refuses (an old
    // kernel, a sandbox), the search simply runs at normal priority, which is still correct.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lowerDiskPriority(); // best effort, result ignored
    // ```
    lower_disk_priority();

    // What:     `announcement_for(name, &arguments)` answers with `Some(kind)` only when the whole argument list is one
    // version or help flag of this tool.
    // Why:      Such a run starts the real tool as a child and then adds the shadow's own line or section (see
    // `announce.rs`); every other run falls through to the process replacement below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kind = announcementFor(name, args);
    // if (kind !== undefined) return runAnnounced(name, real, args, kind);
    // ```
    if let Some(announcement) = announcement_for(name, &arguments) {
        return run_announced(name, &real, &arguments, announcement);
    }

    // A returned value here means the launch failed; success replaced the process. `&real` and `&arguments` lend the
    // path and the argument list without giving them away.
    let error = replace_process(&real, &arguments);
    eprintln!("search-shadow: could not start {}: {error}", real.display());
    return EXEC_FAILED_EXIT_CODE;
}
