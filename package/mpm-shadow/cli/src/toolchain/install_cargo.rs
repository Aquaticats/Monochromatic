//! What:
//! The two cargo routes:
//! `cargo-binstall` (`cargo binstall --no-confirm --root <version directory> --version <locked> <crate>`) and
//!           `cargo-git` (`cargo install --git <url> --rev <rev> --root <install dir> <package>`).
//! Why:
//! The version always comes from the lock,
//! so a crate is installed at exactly the locked version and never
//!           through mpm,
//! which drops pinned versions for cargo.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-cargo.ts
//! export function installCargoBinstall(ctx: Context, tool: LockedTool): void
//! export function installCargoGit(ctx: Context, tool: LockedTool): void
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports what this module uses from `super::cargo_installs`.
use super::cargo_installs::{
    is_repository_published, point_current_at_newest, remove_older_than_newest, version_directory, version_names,
};
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `no_archive_failure` from `super::binstall_no_archive`.
use super::binstall_no_archive::no_archive_failure;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::install_support`.
use super::install_support::{failed_step, run_checked};
/// Imports `tool_install_dir` from `super::layout`.
use super::layout::tool_install_dir;
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Origin};
/// Imports `package_name` from `super::observe`.
use super::observe::package_name;
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest};
/// Imports the keeps every removal asks from `super::store_keeps`.
use super::store_keeps::InstallKeeps;

/// What:
/// Installs one crate with `cargo binstall --root <version directory>` into the crate's own version-keyed cargo
/// root,
/// passing the GitHub token when one is available.
/// A crate this repository publishes also gets `--disable-strategies compile,quick-install`
/// (owner decision 2026-11-04,
/// "A:
/// one flag,
/// our crates"):
/// cargo-binstall resolves the release archive from the crate's own published metadata (no URL is passed here),
/// and the install fails with a message naming what to do when none fits;
/// only a third-party crate may fall back to compiling or to the third-party quick-install host.
/// For a repository-published crate it then points `current` at the newest installed version and deletes every
/// older one that no lock names and no process of this user runs.
/// Why:
/// `--root` (unlike `--install-path`) also covers the build-from-source fallback:
/// cargo-binstall 1.26.1 passes `--root` on to `cargo install`,
/// and with `--install-path` it refuses to build instead
/// (`crates/binstalk/src/ops/resolve/resolution.rs`,
/// `CargoInstallDoesNotSupportInstallPath`).
/// Tracking stays on,
/// so the root's `.crates.toml` is the record `list` reads and cargo-binstall's own lock covers two trees
/// installing the same version at once.
/// Nothing is written to `$CARGO_HOME/bin` or `$CARGO_HOME/.crates.toml`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installCargoBinstall(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_cargo_binstall(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let crate_name: String = package_name(tool);
    let root: PathBuf = version_directory(&ctx.scope.toolchain_root, &crate_name, &tool.version);
    if let Err(error) = fs::create_dir_all(&root) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", root.display())));
    }
    let mut args: Vec<String> = vec![
        "binstall".to_string(),
        "--no-confirm".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--version".to_string(),
        tool.version.clone(),
    ];
    // A crate this repository publishes installs from its own release archive or fails loudly: never from source,
    // never from the third-party quick-install host.
    let repository_crate: bool = is_repository_published(tool);
    if repository_crate {
        args.push("--disable-strategies".to_string());
        args.push("compile,quick-install".to_string());
    }
    for feature in tool.declaration.options.cargo_features.clone().unwrap_or_default() {
        args.push(format!("--features={feature}"));
    }
    args.push(crate_name.clone());
    let mut env = ctx.path_env();
    if let Some(token) = ctx.github_token() {
        env.insert("GITHUB_TOKEN".to_string(), token);
    }
    let request: RunRequest = RunRequest { command: "cargo".to_string(), args, env: Some(env), cwd: None };
    let outcome: RunOutcome = ctx.run(&request);
    if !outcome.succeeded() {
        if repository_crate && let Some(error) = no_archive_failure(&tool.declaration.id, &crate_name, &tool.version, &outcome) {
            return Err(error);
        }
        return Err(failed_step("cargo binstall", &tool.declaration.id, &outcome));
    }
    if repository_crate {
        point_current_at_newest(&ctx.scope.toolchain_root, &crate_name)?;
        // What: the keeps are read only when the crate's directory holds a version besides the one this run wrote.
        // Why: reading every worktree's lock and the process table costs about a second,
        // and a first install of the crate has nothing older to judge.
        let older: bool = version_names(&ctx.scope.toolchain_root, &crate_name).iter().any(|name| return name != &tool.version);
        if older {
            // What: the locks and the process table are read here, before anything goes.
            // Why: a version another worktree's lock pins, or a service runs, outranks the newest-only rule.
            let keeps: InstallKeeps = InstallKeeps::of(ctx);
            remove_older_than_newest(&ctx.scope.toolchain_root, &crate_name, &keeps, &tool.version)?;
        }
    }
    return Ok(());
}

/// What:
/// Installs one crate from a git source at a pinned revision into the tool's own install directory.
/// Why:
/// The revision is the request's range with a leading `ref:` removed, or the range as it stands.
/// A request that pins no revision carries the range `latest`,
/// which is passed to `--rev` unchanged and that cargo refuses,
/// so a git entry must pin the revision it wants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installCargoGit(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_cargo_git(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let Origin::Git(url) = tool.declaration.request.origin() else {
        return Err(ToolchainError::failure(format!(
            "toolchain plan entry {} installs from git but its request is not git:<url>@<revision>",
            tool.declaration.id
        )));
    };
    let revision: String = tool.declaration.request.range.strip_prefix("ref:").unwrap_or(&tool.declaration.request.range).to_string();
    let install_dir: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version)?;
    if let Err(error) = fs::create_dir_all(&install_dir) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", install_dir.display())));
    }
    let install_text: String = install_dir.to_string_lossy().to_string();
    let package: String = package_name(tool);
    let request: RunRequest = ctx.request("cargo", &["install", "--git", &url, "--rev", &revision, "--root", &install_text, &package]);
    run_checked(ctx, &request, "cargo install --git", &tool.declaration.id)?;
    return Ok(());
}
