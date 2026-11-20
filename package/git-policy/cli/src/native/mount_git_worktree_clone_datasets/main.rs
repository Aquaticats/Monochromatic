//! What:
//!  The thin entry point of the root-run clone-mount handler,
//!  `mount-git-worktree-clone-datasets`:
//!  collect the arguments,
//!  hand them to the library's one command line,
//!  wait for the lines it handed to their own thread,
//!  and end the process with its status.
//! Why:
//!  The program is what ZED runs on a history event and what the boot and shutdown units start,
//!  so its whole process setup is here and every decision is in the library
//!  (`worktree_clone_mount_handler`).
//!  It takes no environment and reads no configuration:
//!  an invocation is a mode word and,
//!  for an adoption,
//!  the replacement's own device and inode.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function main(): never { const code = run(process.argv.slice(2)); writeWhatWaits(WAIT); process.exit(code); }
//! ```

use git_policy_cli::diagnostic_lines_off_the_calling_thread::{WAIT_BEFORE_AN_EXIT, write_what_waits};
use git_policy_cli::service_log_lines_off_the_calling_thread::write_what_waits as write_every_service_log_line_that_waits;
use std::ffi::OsString;

/// What:
///  The program entry point.
/// Why:
///  `args_os` never decodes,
///  so a non-UTF-8 argument reaches the grammar as the argument it was,
///  and the grammar refuses it by its place rather than panicking here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arguments = process.argv.slice(2); process.exit(run(arguments));
/// ```
fn main() {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    let code: i32 = git_policy_cli::worktree_clone_mount_handler::run(arguments.as_slice());
    let _diagnosed: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);
    let _logged: bool = write_every_service_log_line_that_waits(WAIT_BEFORE_AN_EXIT);
    std::process::exit(code);
}
