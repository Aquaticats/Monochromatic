//! What:
//! The name the program was started under:
//! the final segment of the first command-line argument,
//! which decides whether a run is a shadow (`rg` / `grep` / `find` and so on)
//! or the `search-shadow` command-line tool.
//! Why:
//! One binary serves both roles,
//! so this name is the only thing that picks the mode,
//! and it has to answer for every argument list the operating system can hand over,
//! including an empty one an `execve` caller may pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // invoked-name.ts
//! export function invokedName(arguments: readonly string[]): string { ... }
//! ```

/// Imports the owned operating-system text type,
/// which carries bytes that need not be valid UTF-8
/// (its sibling `String` only holds valid UTF-8).
use std::ffi::OsString;

/// Imports `Path`,
/// the borrowed read-only path type
/// (its siblings are `PathBuf`, the owned one, and `&str`, which is text without path rules).
use std::path::Path;

/// What:
/// `pub fn invoked_name(arguments: &[OsString]) -> String` declares a public function that takes a borrowed
/// slice of command-line arguments and returns the name as owned text.
/// Why:
/// The caller needs the name before anything else,
/// and an argument list with no element at all still answers an empty name instead of stopping the program.
/// It takes `&[OsString]` and not `Vec<OsString>` because it only reads,
/// and returns `String` and not `&str` because the name is built from borrowed pieces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function invokedName(args: readonly string[]): string { ... }
/// ```
pub fn invoked_name(arguments: &[OsString]) -> String {
    // What:     `let mut invoked_as: String = String::new();` creates an empty owned text that CAN be reassigned
    // (`mut`); plain `let` is like `const`.
    // Why:      It holds the documented default, so a first argument with no final segment leaves it empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let invokedAs = '';
    // ```
    let mut invoked_as: String = String::new();

    // What:     `arguments.first()` answers `Some(&firstArgument)` or `None` for a list with no element at all, so
    // nothing indexes position zero; `&& arguments.first()` in the `if let` follows the answer twice, once for the
    // present/absent question and once for emptiness, and runs the block only when both hold.
    // Why:      A symlink named `rg` makes the first argument end in `rg`, which selects shadow mode, while a start
    // with no argument at all must still answer instead of stopping the program.
    // Gotcha:   Indexing the list (`&arguments[0]`) looks like TS's `args[0]`, which is `undefined` past the end;
    // Rust indexing of an absent position stops the program instead.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const firstName = args[0];
    // const fileName = firstName === undefined ? undefined : path.basename(firstName);
    // if (fileName !== undefined) invokedAs = fileName;
    // ```
    if let Some(argument) = arguments.first()
        && let Some(file_name) = Path::new(argument).file_name()
    {
        // `to_string_lossy()` turns the segment into text and replaces bytes that are not valid UTF-8 with a
        // replacement character; `.to_string()` copies that text into the owned `String` the function returns.
        invoked_as = file_name.to_string_lossy().to_string();
    }

    // What:     `return invoked_as;` leaves the function with the name.
    // Why:      Hand the caller the name to compare against the shadowed names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return invokedAs;
    // ```
    return invoked_as;
}
