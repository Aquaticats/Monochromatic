//! What:
//! The installed version of a tool whose record is what a program prints:
//! `bin list`,
//! `pnpm list --global --depth=0 --json`,
//! `uv tool list`,
//! and the toolchain `rustup` resolves and that toolchain's own `rustc` reports.
//! Why:
//! A listing proves only what it prints on standard output:
//! a failed listing proves nothing about what is installed,
//! a diagnostic on standard error that looks like a row is not one,
//! and only the compiler of a resolved toolchain can say which release a moving channel holds now.
//! A name alone cannot tell a host triple from a channel that carries a date,
//! so the tool decides which toolchain a channel is,
//! and a channel that is not installed is never taken for one that is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function uvKey(tool: LockedTool): string
//! export function observeBinSourced(ctx: Context, tool: LockedTool): string | undefined
//! export function observeMpmPnpm(ctx: Context, tool: LockedTool): string | undefined
//! export function observeMpmUvx(ctx: Context, tool: LockedTool): string | undefined
//! export function observeRustupToolchain(ctx: Context, tool: LockedTool): string | undefined
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `Context` from `super::super::context`.
use super::super::context::Context;
/// Imports `LockedTool` from `super::super::model`.
use super::super::model::LockedTool;
/// Imports what this module uses from `super::super::parse_output`.
use super::super::parse_output::{
    global_pnpm_versions,
    parse_bin_list,
    parse_uv_tool_list,
    version_matches,
};
/// Imports what this module uses from `super::super::runner`.
use super::super::runner::{
    RunOutcome,
    RunRequest,
};
/// Imports `package_name` from this module's parent.
use super::package_name;

/// What:
/// The package key under `uv tool list`:
/// the package id (or tool id) with `-` turned into `_`.
/// Why:
/// `parse_uv_tool_list` keys its map the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const key = (tool.packageId ?? tool.id).replaceAll('-', '_');
/// ```
fn uv_key(tool: &LockedTool) -> String {
    return package_name(tool).replace('-', "_");
}

/// `bin list`,
/// for the bin-sourced route.
pub(super) fn observe_bin_sourced(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    let cli: PathBuf = ctx.find_bin_of("bin", "bin", &[])?;
    let mut env = ctx.path_env();
    env.insert(
        "BIN_CONFIG".to_string(),
        ctx.scope.bin_config_file.to_string_lossy().to_string(),
    );
    let outcome: RunOutcome =
        ctx.run(&RunRequest::new(&cli.to_string_lossy(), &["list"]).with_env(env));
    // A listing that failed proves nothing about what is installed, and only the rows `bin list` writes on standard
    // output are a listing: a diagnostic on standard error that looks like a row is not one.
    if !outcome.succeeded() {
        return None;
    }
    let versions = parse_bin_list(&outcome.stdout);
    for name in &tool.declaration.bins {
        if let Some(version) = versions.get(name) {
            return Some(version.clone());
        }
    }
    return None;
}

/// `pnpm list --global --depth=0 --json`,
/// for the mpm-pnpm route.
pub(super) fn observe_mpm_pnpm(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    let pnpm: PathBuf = ctx.find_bin_of("pnpm", "pnpm", &[])?;
    let outcome: RunOutcome = ctx.run(&ctx.pnpm_request(
        &pnpm.to_string_lossy(),
        &["list", "--global", "--depth=0", "--json"],
    ));
    let parsed: Value = match serde_json::from_str(&outcome.stdout) {
        Ok(value) => value,
        // Not JSON: pnpm failed or printed something else, so the tool is read as absent.
        Err(_) => return None,
    };
    return global_pnpm_versions(&parsed)
        .get(&package_name(tool))
        .cloned();
}

/// `uv tool list`,
/// for the mpm-uvx route.
pub(super) fn observe_mpm_uvx(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    let uv: PathBuf = ctx.find_bin_of("uv", "uv", &[])?;
    let outcome: RunOutcome = ctx.run(&ctx.request(&uv.to_string_lossy(), &["tool", "list"]));
    return parse_uv_tool_list(&outcome.stdout)
        .get(&uv_key(tool))
        .cloned();
}

/// What:
/// The toolchain this row names,
/// for the rustup-toolchain route:
/// the row's channel when `rustup` resolves it to an installed toolchain,
/// else `None`.
/// Why:
/// `rustup toolchain list` prints a channel followed by the host triple it was installed for,
/// and a name alone cannot tell that triple from a channel that carries a date:
/// `nightly-x86_64-unknown-linux-gnu` and `nightly-2026-10-30-x86_64-unknown-linux-gnu` both start with `nightly-`.
/// `rustup which --toolchain <channel>` is the tool deciding which installed toolchain that channel is,
/// so no name is guessed and a toolchain for another host is never taken for this one.
/// `rustup_query` keeps the question from installing a toolchain that is missing,
/// and a query that failed or printed no path proves nothing.
pub(super) fn observe_rustup_toolchain(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    let channel: &str = tool.rustup_channel();
    let rustc: PathBuf = resolved_rustc(ctx, channel)?;
    // What: a row whose channel text differs from its version also asks that toolchain's compiler which release
    // it holds.
    // Why: `which` proves the channel is installed, not that it still holds the release the lock records;
    // a moving channel (`stable`) can hold a newer release, and a lock pinning `1.90.0` must not read `ok` then.
    // A locked text that names no release (a dated channel such as `nightly-2026-10-30`) is not compared: the
    // compiler of such a channel reports `1.9x.0-nightly`, which names no channel date, and the dated channel
    // text is the identity.
    if channel != tool.version
        && names_a_release(&tool.version)
        && !compiler_reports(ctx, &rustc, &tool.version)
    {
        return None;
    }
    return Some(channel.to_string());
}

/// What:
/// The `rustc` a channel resolves to,
/// from `rustup which --toolchain <channel> rustc`,
/// or `None` when rustup says that channel is not installed.
/// Why:
/// The path is what the version question runs,
/// and its absence is the answer that the channel is not installed.
fn resolved_rustc(
    ctx: &Context,
    channel: &str,
) -> Option<PathBuf> {
    let outcome: RunOutcome =
        ctx.run(&ctx.rustup_query(&["which", "--toolchain", channel, "rustc"]));
    if !outcome.succeeded() {
        return None;
    }
    let path: &str = outcome.stdout.trim();
    if path.is_empty() {
        return None;
    }
    return Some(PathBuf::from(path));
}

/// What:
/// Whether the compiler at `rustc` reports the release `locked` names (`rustc 1.90.0 (hash date)`).
/// Why:
/// The installed compiler is the only witness of which release a moving channel holds now.
fn compiler_reports(
    ctx: &Context,
    rustc: &Path,
    locked: &str,
) -> bool {
    let outcome: RunOutcome = ctx.run(&ctx.request(&rustc.to_string_lossy(), &["--version"]));
    if !outcome.succeeded() {
        return false;
    }
    // `rustc <release> (<hash> <date>)`: the release is the second word.
    let released: Option<&str> = outcome.stdout.split_whitespace().nth(1);
    return released.is_some_and(|reported| return version_matches(reported, locked));
}

/// What:
/// Whether `text` names a release (`1.90.0`,
/// `v1.90.0`,
/// `1.90.0-beta.1`) rather than a channel
/// (`stable`,
/// `nightly`,
/// `nightly-2026-10-30`).
/// Why:
/// Only a release text is something a compiler can confirm or contradict;
/// a channel text is the toolchain's identity,
/// which `rustup which` already resolved.
fn names_a_release(text: &str) -> bool {
    let without_prefix: &str = text
        .strip_prefix('v')
        .filter(|rest| return !rest.is_empty())
        .unwrap_or(text);
    return without_prefix.starts_with(|first: char| return first.is_ascii_digit());
}
