//! What:     Finds cargo's home directory (`~/.cargo` unless `CARGO_HOME` says otherwise).
//! Why:      The install record and the registry cache both live under it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cargo-home.ts
//! export function cargoHome(env: NodeJS.ProcessEnv): string | undefined {
//!   return env.CARGO_HOME || (env.HOME ? path.join(env.HOME, '.cargo') : undefined);
//! }
//! ```

/// What:     `use std::path::PathBuf;` imports the owned, growable file path (sibling `&Path` is the borrowed,
///           read-only one).
/// Why:      The function hands a path back to its caller to keep, so it must own it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS uses plain strings for paths.
/// ```
use std::path::PathBuf;

/// What:     `pub fn cargo_home(cargo_home_variable: Option<String>, home_variable: Option<String>) -> Option<PathBuf>`
///           declares a public function over two `Option<String>` inputs (each `Some(text)` when that environment
///           variable is set, `None` when not) returning `Some(path)` or `None`.
/// Why:      The caller reads the real environment; taking the values as inputs lets the tests pass fixtures without
///           changing the process environment. `String` and not `&str` because the caller passes the owned values
///           `std::env::var(...).ok()` produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function cargoHome(cargoHomeVariable: string | undefined, homeVariable: string | undefined): string | undefined
/// ```
pub fn cargo_home(cargo_home_variable: Option<String>, home_variable: Option<String>) -> Option<PathBuf> {
    // What:     `if let Some(directory) = cargo_home_variable` runs the block only when the variable is set; it also
    //           moves the text out of the `Option` into `directory`. `!directory.is_empty()` rejects an empty text.
    // Why:      `CARGO_HOME` wins over the default, as it does for cargo itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (cargoHomeVariable !== undefined && cargoHomeVariable !== '') return cargoHomeVariable;
    // ```
    if let Some(directory) = cargo_home_variable
        && !directory.is_empty()
    {
        // `Some(PathBuf::from(directory))` wraps the owned path in the present case of `Option`; `PathBuf::from`
        // converts the text into a path.
        return Some(PathBuf::from(directory));
    }

    // `HOME` plus `.cargo` is cargo's default; `home.join(".cargo")` appends a segment and returns a new `PathBuf`.
    if let Some(home) = home_variable
        && !home.is_empty()
    {
        return Some(PathBuf::from(home).join(".cargo"));
    }

    // What:     `None` is the absent case of `Option`.
    // Why:      Neither variable is usable, so the caller skips the prune.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return undefined;
    // ```
    return None;
}
