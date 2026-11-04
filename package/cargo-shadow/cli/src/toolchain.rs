//! What:
//! The rustup toolchain the repository pins in `toolchain.lock.json`,
//! and the choice between it and an inherited
//! `RUSTUP_TOOLCHAIN`.
//! Why:
//! The update passes `-Zunstable-options`,
//! which only nightly cargo accepts,
//! and every lockfile the catalog tools write
//! should come from one cargo so the format never flips.
//! A systemd user manager on the development host exports
//! `RUSTUP_TOOLCHAIN=1.98.1`,
//! which once made the first real run use stable cargo,
//! so the repository's pin wins over an
//! inherited value;
//! the inherited value applies only when the repository pins none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // toolchain.ts
//! export function rustToolchainFromLock(text: string): string
//! export async function cargoToolchain({ root, explicit }): Promise<string>
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `pinned_toolchain`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports `io` from the standard library so this file can name it unqualified.
/// Why:
/// `io` is used by `pinned_toolchain`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { io } from 'std';
/// ```
use std::io;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `pinned_toolchain`,
/// `cargo_toolchain`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;

/// What:
/// `use crate::constants::TOOLCHAIN_LOCK_NAME;` imports `TOOLCHAIN_LOCK_NAME` from another module of this crate so this file can name it unqualified.
/// Why:
/// `TOOLCHAIN_LOCK_NAME` is used by `pinned_toolchain`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TOOLCHAIN_LOCK_NAME } from 'crate/constants';
/// ```
use crate::constants::TOOLCHAIN_LOCK_NAME;
/// What:
/// `use crate::adapt::text_of;` imports `text_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `text_of` is used by `rust_toolchain_from_lock`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { text_of } from 'crate/adapt';
/// ```
use crate::adapt::text_of;

/// What:
/// `pub fn rust_toolchain_from_lock(text: &str) -> Result<String, String>` reads `.tools.rust.version`.
/// Why:
/// An absent entry is the empty string,
/// which callers read as "the repository pins nothing".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rustToolchainFromLock(text: string): string  // throws SyntaxError
/// ```
pub fn rust_toolchain_from_lock(text: &str) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(text).map_err(text_of)?;
    let version = parsed.pointer("/tools/rust/version");
    return match version.and_then(serde_json::Value::as_str) {
        Some(found) => Ok(found.to_string()),
        None => Ok(String::new()),
    };
}

/// The pinned toolchain of a repository,
/// or empty when there is no lock file or no Rust entry.
pub fn pinned_toolchain(root: &Path) -> Result<String, String> {
    return match fs::read_to_string(root.join(TOOLCHAIN_LOCK_NAME)) {
        Ok(text) => rust_toolchain_from_lock(&text),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.to_string()),
    };
}

/// What:
/// `pub fn cargo_toolchain(root: &Path, explicit: &str) -> Result<(String, Option<String>), String>` picks the toolchain the
/// catalog commands run cargo under,
/// and a warning when it overrides the environment.
/// Why:
/// The pin wins;
/// with no pin an explicit `RUSTUP_TOOLCHAIN` is used;
/// with neither,
/// rustup's default applies (empty string).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function cargoToolchain({ root, explicit }): Promise<{ toolchain: string; warning?: string }>
/// ```
pub fn cargo_toolchain(root: &Path, explicit: &str) -> Result<(String, Option<String>), String> {
    let pinned: String = pinned_toolchain(root)?;
    if !pinned.is_empty() {
        if !explicit.is_empty() && explicit != pinned {
            let warning: String =
                format!("ignoring RUSTUP_TOOLCHAIN={explicit} from the environment; {TOOLCHAIN_LOCK_NAME} pins {pinned}");
            return Ok((pinned, Some(warning)));
        }
        return Ok((pinned, None));
    }
    if !explicit.is_empty() {
        return Ok((explicit.to_string(), None));
    }
    let warning: String = format!(
        "{} pins no Rust toolchain and RUSTUP_TOOLCHAIN is unset; cargo runs under rustup's default",
        root.display()
    );
    return Ok((String::new(), Some(warning)));
}
