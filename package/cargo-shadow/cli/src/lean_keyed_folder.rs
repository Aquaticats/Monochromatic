//! What:
//! Mechanism A,
//! toolchain-keyed build folders:
//! for a host cargo run the shadow sets `CARGO_BUILD_BUILD_DIR` to
//! `<base>/toolchain/<toolchain>/<xx>/<14 hex>` for the child cargo only,
//! and leaves a ledger file in it naming the workspace.
//! Why:
//! A nightly update dirties every unit of a build folder in place and Cargo never removes the old ones
//! (`doc/planning/lean-build-dirs.md`,
//! option A),
//! so the first build under a new toolchain loses nothing by starting in a fresh folder,
//! and retiring a toolchain is removing a directory.
//! The environment variable is used verbatim by Cargo,
//! so the workspace key is computed here (SHA-256 of the canonical workspace root,
//! the same shard and hash shape as
//! `{workspace-path-hash}`).
//! The base is the part of the user's `build.build-dir` before `{workspace-path-hash}` in `$CARGO_HOME/config.toml`;
//! with no such setting,
//! with an explicit `CARGO_BUILD_BUILD_DIR`,
//! or with no known toolchain,
//! the shadow sets nothing.
//! Runs that bypass the shadow (an IDE launching Cargo itself) keep the user's setting and are covered by mechanisms B to F.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function keyedBuildDirectory(base: string, toolchain: string, root: string): string
//! ```

/// Arguments,
/// files and paths.
use std::{ffi::OsString, fs, os::unix::ffi::OsStrExt, path::{Path, PathBuf}, process::Command};

/// The digest that keys a workspace.
use sha2::{Digest, Sha256};
/// The reader of `config.toml`.
use toml_edit::DocumentMut;
/// Named reads of paths and text: a ledger is read the way every other name here is.
use crate::lean_text::read_small_text;

/// The Cargo variable the shadow sets for the child only.
pub const BUILD_DIR_VARIABLE: &str = "CARGO_BUILD_BUILD_DIR";

/// The directory under the base that holds the keyed folders.
pub const KEYED_DIRECTORY: &str = "toolchain";

/// The ledger file inside a keyed folder:
/// the workspace root it was made for.
pub const LEDGER_FILE: &str = ".cargo-shadow-workspace";

/// The most bytes of a ledger that are read.
/// Why:
/// A ledger is one line naming one workspace root,
/// and a path on Linux is at most 4096 bytes (`PATH_MAX` file and path names,
/// `limits.h`),
/// so twice that is longer than any ledger this tool writes and small enough that a file of another kind is rejected
/// rather than read.
const LEDGER_BYTES: usize = 8_192;

/// The template token the base is cut at.
const HASH_TOKEN: &str = "{workspace-path-hash}";

/// The number of hexadecimal digits of the shard directory.
const SHARD_LENGTH: usize = 2;

/// The number of hexadecimal digits of the workspace directory.
const KEY_LENGTH: usize = 14;

/// What:
/// `pub fn base_of_template(template: &str) -> Option<PathBuf>` is the directory a `build.build-dir` template starts with,
/// when the template ends with the workspace hash.
/// Why:
/// `/var/mnt/encrypted/cargo-build/{workspace-path-hash}` has the base `/var/mnt/encrypted/cargo-build`.
/// A template with another variable in the base,
/// a relative base,
/// or text after the token is not one the shadow keys.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const baseOfTemplate = (t: string): string | undefined => t.endsWith('/{workspace-path-hash}') && !t.slice(0, -21).includes('{') && t.startsWith('/') ? t.slice(0, -21) : undefined;
/// ```
pub fn base_of_template(template: &str) -> Option<PathBuf> {
    let base: &str = template.strip_suffix(HASH_TOKEN)?.trim_end_matches('/');
    if base.is_empty() || base.contains('{') || !base.starts_with('/') {
        return None;
    }
    return Some(PathBuf::from(base));
}

/// What:
/// `pub fn configured_base(cargo_home: &Path) -> Option<PathBuf>` reads `build.build-dir` from the user's Cargo configuration
/// and returns its base.
/// Why:
/// The base is where the user already keeps build folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function configuredBase(cargoHome: string): string | undefined
/// ```
pub fn configured_base(cargo_home: &Path) -> Option<PathBuf> {
    for name in ["config.toml", "config"] {
        let Ok(text) = fs::read_to_string(cargo_home.join(name)) else { continue };
        let Ok(document) = text.parse::<DocumentMut>() else { return None };
        let template: &str = document.get("build")?.get("build-dir")?.as_str()?;
        return base_of_template(template);
    }
    return None;
}

/// What:
/// `pub fn toolchain_of(arguments: &[OsString], variable: Option<&str>) -> Option<String>` is the toolchain a cargo run uses,
/// from a leading `+toolchain` argument or `RUSTUP_TOOLCHAIN`.
/// Why:
/// Both are how rustup picks the toolchain.
/// A name that is not a single plain path segment is not used as a folder name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function toolchainOf(args: string[], variable?: string): string | undefined
/// ```
pub fn toolchain_of(arguments: &[OsString], variable: Option<&str>) -> Option<String> {
    let mut chosen: Option<String> = variable.map(str::to_string);
    let first: Option<&str> = match arguments.first() {
        Some(word) => word.to_str(),
        None => None,
    };
    if let Some(text) = first
        && let Some(name) = text.strip_prefix('+')
    {
        chosen = Some(name.to_string());
    }
    let name: String = chosen?;
    let mut plain: bool = !name.is_empty() && !name.starts_with('.');
    for character in name.chars() {
        if !(character.is_ascii_alphanumeric() || "-_.".contains(character)) {
            plain = false;
        }
    }
    return if plain { Some(name) } else { None };
}

/// What:
/// `pub fn workspace_key(root: &Path) -> (String, String)` is the shard and the key of a workspace root.
/// Why:
/// SHA-256 of the root's bytes gives a stable name of the same shape as Cargo's template.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const workspaceKey = (root: string) => { const hex = sha256(root).toString('hex'); return [hex.slice(0, 2), hex.slice(2, 16)]; };
/// ```
pub fn workspace_key(root: &Path) -> (String, String) {
    let digest = Sha256::digest(root.as_os_str().as_bytes());
    let mut hex: String = String::new();
    for byte in digest.iter().take((SHARD_LENGTH + KEY_LENGTH) / 2) {
        hex.push_str(&format!("{byte:02x}"));
    }
    return (hex[..SHARD_LENGTH].to_string(), hex[SHARD_LENGTH..SHARD_LENGTH + KEY_LENGTH].to_string());
}

/// What:
/// `pub fn keyed_build_dir(base: &Path, toolchain: &str, root: &Path) -> PathBuf` is the folder of a workspace under a toolchain.
/// Why:
/// Retiring the toolchain is removing `<base>/toolchain/<toolchain>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const keyedBuildDir = (base: string, toolchain: string, root: string): string => join(base, 'toolchain', toolchain, ...workspaceKey(root));
/// ```
pub fn keyed_build_dir(base: &Path, toolchain: &str, root: &Path) -> PathBuf {
    let (shard, key) = workspace_key(root);
    return base.join(KEYED_DIRECTORY).join(toolchain).join(shard).join(key);
}

/// What:
/// `pub fn leave_ledger(folder: &Path, root: &Path)` writes the ledger file of a keyed folder when it has none.
/// Why:
/// Mechanism C removes a folder only when the workspace it names is gone.
/// The write is skipped when the file exists,
/// so a cargo run costs one `stat`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function leaveLedger(folder: string, root: string): void
/// ```
pub fn leave_ledger(folder: &Path, root: &Path) {
    let ledger: PathBuf = folder.join(LEDGER_FILE);
    if ledger.exists() {
        return;
    }
    if fs::create_dir_all(folder).is_ok() {
        let _ = fs::write(&ledger, format!("{}\n", root.display()));
    }
}

/// What:
/// `pub fn read_ledger(folder: &Path) -> Option<PathBuf>` is the workspace root a keyed folder was made for.
/// Why:
/// The idle-tree collection reads it,
/// and that read must answer at once:
/// a ledger that is a pipe with no writer would otherwise hold the pass until a writer arrived,
/// which nothing promises to do,
/// so the read waits on nothing,
/// takes a regular file only,
/// and stops at the length no ledger exceeds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const readLedger = (folder: string): string | undefined => readSmallText(join(folder, LEDGER_FILE), LEDGER_BYTES, true);
/// ```
pub fn read_ledger(folder: &Path) -> Option<PathBuf> {
    let text: String = read_small_text(&folder.join(LEDGER_FILE), LEDGER_BYTES, true).ok()?;
    let line: &str = text.lines().next()?.trim();
    return if line.is_empty() { None } else { Some(PathBuf::from(line)) };
}

/// What:
/// `pub fn apply_keyed_build_dir(command: &mut Command, arguments: &[OsString], working_directory: &Path)` sets the keyed build
/// folder in the environment of the child cargo when every condition holds,
/// and leaves the ledger.
/// Why:
/// It is the only change the shadow makes to the child's environment for mechanism A.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function applyKeyedBuildDir(command: Command, args: string[], cwd: string): void
/// ```
pub fn apply_keyed_build_dir(command: &mut Command, arguments: &[OsString], working_directory: &Path) {
    if std::env::var_os(BUILD_DIR_VARIABLE).is_some() {
        return;
    }
    let cargo_home: PathBuf = match std::env::var_os("CARGO_HOME") {
        Some(home) => PathBuf::from(home),
        None => match std::env::var_os("HOME") {
            Some(home) => PathBuf::from(home).join(".cargo"),
            None => return,
        },
    };
    let Some(base) = configured_base(&cargo_home) else { return };
    let variable: Option<String> = std::env::var("RUSTUP_TOOLCHAIN").ok();
    let Some(toolchain) = toolchain_of(arguments, variable.as_deref()) else { return };
    let start: PathBuf = crate::shadow_step::start_dir(arguments, working_directory);
    let Some(found) = crate::workspace::find_cargo_root(&start) else { return };
    let Ok(root) = fs::canonicalize(found) else { return };
    let folder: PathBuf = keyed_build_dir(&base, &toolchain, &root);
    leave_ledger(&folder, &root);
    command.env(BUILD_DIR_VARIABLE, &folder);
}
