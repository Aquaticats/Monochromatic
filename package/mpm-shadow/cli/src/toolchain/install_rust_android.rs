//! What:
//! The `rustup-toolchain` route (`rustup toolchain install --no-self-update <channel>`,
//! then components and
//!           targets) and the `android-sdk` route (command-line tools from a verified zip,
//! then `sdkmanager` packages).
//! Why:
//! Rust channels install directly from the lock:
//! mpm honours a rustup version only because the package id is
//!           the channel,
//! and the prototype showed the direct call produces the same toolchain list.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-rust-android.ts
//! export function installRustToolchain(ctx: Context, tool: LockedTool): void
//! export function installRustComponents(ctx: Context, tool: LockedTool): void
//! export function installAndroidSdk(ctx: Context, tool: LockedTool): void
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `safe_file_name` from `super::install_archive`.
use super::install_archive::safe_file_name;
/// Imports what this module uses from `super::install_support`.
use super::install_support::{artifact_for_host, download_verified, extract_archive, run_checked};
/// Imports `tool_install_dir` from `super::layout`.
use super::layout::tool_install_dir;
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest};

/// `rustup toolchain install --no-self-update <package id, or the locked version>`.
pub fn install_rust_toolchain(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let channel: String = match &tool.declaration.package_id {
        Some(package_id) => package_id.clone(),
        None => tool.version.clone(),
    };
    let request: RunRequest = ctx.request("rustup", &["toolchain", "install", "--no-self-update", &channel]);
    run_checked(ctx, &request, "rustup toolchain install", &tool.declaration.id)?;
    return Ok(());
}

/// What:
/// `rustup component add <name> --toolchain <version>` for each declared component,
/// then `rustup target add
///           <triple> --toolchain <version>` for each declared target.
/// Why:
/// The toolchain install above brings the default profile;
/// the plan names the extras.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installRustComponents(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_rust_components(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    for component in tool.declaration.options.rust_components.clone().unwrap_or_default() {
        let request: RunRequest = ctx.request("rustup", &["component", "add", &component, "--toolchain", &tool.version]);
        run_checked(ctx, &request, &format!("rustup component add ({component})"), &tool.declaration.id)?;
    }
    for target in tool.declaration.options.rust_targets.clone().unwrap_or_default() {
        let request: RunRequest = ctx.request("rustup", &["target", "add", &target, "--toolchain", &tool.version]);
        run_checked(ctx, &request, &format!("rustup target add ({target})"), &tool.declaration.id)?;
    }
    return Ok(());
}

/// What:
/// Which `rustup` subcommand lists and adds one part of a Rust toolchain:
/// a component,
/// or a target.
/// Why:
/// The lock keeps components and targets in two rows and `rustup` answers each row with its own
/// subcommand,
/// so the subcommand word travels with the name instead of being rebuilt at each call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RustPartKind = 'component' | 'target';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustPartKind {
    /// One entry of the lock's `rustComponents` row.
    Component,
    /// One entry of the lock's `rustTargets` row.
    Target,
}

/// The methods of `RustPartKind`.
impl RustPartKind {
    /// What:
    /// The `rustup` subcommand word that lists and adds this kind of part:
    /// `component` or `target`.
    /// Why:
    /// `list --installed` and `add` have the same shape for both kinds,
    /// so this word is the whole difference between the two calls.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function subcommand(kind: RustPartKind): 'component' | 'target'
    /// ```
    fn subcommand(self) -> &'static str {
        // What: `match` picks one of the enum's two variants; the value after `=>` is the word `rustup` takes.
        // Why: the listing and the add of a part both go through here, so the word is written once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return kind === 'component' ? 'component' : 'target';
        // ```
        return match self {
            RustPartKind::Component => "component",
            RustPartKind::Target => "target",
        };
    }
}

/// What:
/// One part of a Rust toolchain the lock declares:
/// a component such as `clippy`,
/// or a target triple such as `aarch64-linux-android`.
/// Why:
/// The installer and the reconciler need the kind (to pick the subcommand) and the name together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RustPart = { kind: RustPartKind; name: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustPart {
    /// What:
    /// Which `rustup` subcommand adds this part.
    /// Why:
    /// The name alone does not say whether `rustup` wants `component add` or `target add`.
    pub kind: RustPartKind,
    /// What:
    /// The name as the lock declares it (`clippy`,
    /// `aarch64-linux-android`).
    /// Why:
    /// This is the text handed to `rustup add`,
    /// so it must stay exactly as the lock spells it.
    pub name: String,
}

/// What:
/// The names `rustup <kind> list --installed --toolchain <version>` prints,
/// one per line with the empty lines dropped.
/// Why:
/// Asking `rustup` what it holds is what lets a sync on a complete machine add nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installedRustNames(ctx: Context, tool: LockedTool, kind: RustPartKind): string[]
/// ```
fn installed_rust_names(ctx: &Context, tool: &LockedTool, kind: RustPartKind) -> Result<Vec<String>, ToolchainError> {
    let subcommand: &str = kind.subcommand();
    let request: RunRequest = ctx.request("rustup", &[subcommand, "list", "--installed", "--toolchain", &tool.version]);
    let what: String = format!("rustup {subcommand} list --installed");
    // What: `run_checked` starts `rustup` and turns a non-zero exit into an error naming this step.
    // Why: a listing that fails cannot prove which parts are installed, so the tool is reported as failed
    // rather than as unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const outcome = runChecked(ctx, request, what, tool.declaration.id);
    // ```
    let outcome: RunOutcome = run_checked(ctx, &request, &what, &tool.declaration.id)?;
    // What: `Vec<String>` is a growable list of owned strings
    // (the siblings a reader might expect are `&[String]`, a borrowed slice, and `[String; N]`, a fixed-size array).
    // Why: `Vec` because the number of printed names is only known at run time; owned `String` because each one
    // outlives the borrowed line it was cut from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const names: string[] = [];
    // ```
    let mut names: Vec<String> = Vec::new();
    for line in outcome.stdout.lines() {
        // What: `.trim()` returns the line without surrounding whitespace as a borrowed `&str`.
        // Why: a trailing space would make a name that is present look different from the lock's spelling.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name = line.trim();
        // ```
        let name: &str = line.trim();
        if !name.is_empty() {
            // What: `.push(name.to_string())` adds an owned copy of the borrowed name to the list.
            // Why: the copy outlives this loop's borrow of `outcome.stdout`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // names.push(name);
            // ```
            names.push(name.to_string());
        }
    }
    // What: `Ok(names)` wraps the list in the success variant of `Result`, the tail expression of this function.
    // Why: the caller gets the names, or the error `?` already returned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return names;
    // ```
    return Ok(names);
}

/// What:
/// Whether `installed` already holds the name `declared`,
/// as the same text or as that text plus a suffix
/// (`clippy` is installed when the listing shows `clippy-x86_64-unknown-linux-gnu`).
/// Why:
/// `rustup component list --installed` spells a component with the host triple it was built for,
/// while the lock declares the bare name,
/// and a target triple is always printed bare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lists(installed: string[], declared: string): boolean
/// ```
fn lists(installed: &[String], declared: &str) -> bool {
    // What: `format!` builds an owned `String` (the siblings are `&str` for a borrowed slice of text and
    // `String::new()` for an empty one) holding the declared name and a hyphen.
    // Why: a listed name that starts with it is the same part built for some triple.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const withHyphen = declared + '-';
    // ```
    let with_hyphen: String = format!("{declared}-");
    for name in installed {
        if name == declared || name.starts_with(&with_hyphen) {
            return true;
        }
    }
    return false;
}

/// What:
/// The declared components and targets `rustup` does not report as installed for this toolchain:
/// components first,
/// each row in the lock's order.
/// Why:
/// This is the list a sync adds,
/// and it is empty on a machine whose toolchain already carries everything the lock declares,
/// which is why such a run adds nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingRustParts(ctx: Context, tool: LockedTool): RustPart[]
/// ```
pub fn missing_rust_parts(ctx: &Context, tool: &LockedTool) -> Result<Vec<RustPart>, ToolchainError> {
    let mut missing: Vec<RustPart> = Vec::new();
    // What: `.clone().unwrap_or_default()` copies the declared list out of the lock entry (`.clone()` duplicates
    // the owned data) and substitutes an empty list when the row is absent (`.unwrap_or_default()`, the sibling
    // of `.unwrap_or(value)` for a fallback that is the type's own default).
    // Why: the loop below moves each name into the result, so it cannot borrow the lock entry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const components: string[] = tool.declaration.options?.rustComponents ?? [];
    // ```
    let components: Vec<String> = tool.declaration.options.rust_components.clone().unwrap_or_default();
    if !components.is_empty() {
        let installed: Vec<String> = installed_rust_names(ctx, tool, RustPartKind::Component)?;
        for name in components {
            if !lists(&installed, &name) {
                // What: `RustPart { kind, name }` builds the record that carries the subcommand and the name.
                // Why: `reconcile_rust_parts` runs `rustup add` from these two fields.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // missing.push({ kind: 'component', name });
                // ```
                missing.push(RustPart { kind: RustPartKind::Component, name });
            }
        }
    }
    let targets: Vec<String> = tool.declaration.options.rust_targets.clone().unwrap_or_default();
    if !targets.is_empty() {
        let installed: Vec<String> = installed_rust_names(ctx, tool, RustPartKind::Target)?;
        for name in targets {
            if !lists(&installed, &name) {
                missing.push(RustPart { kind: RustPartKind::Target, name });
            }
        }
    }
    return Ok(missing);
}

/// What:
/// Adds every declared component and target `rustup` does not have for this toolchain,
/// and returns one sentence per part it added.
/// Why:
/// A toolchain another program installed at the locked version never reaches `install_tool`,
/// so the lock's `rustComponents` and `rustTargets` rows need a second place that applies them;
/// asking `rustup` first keeps a sync on a complete machine free of any add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reconcileRustParts(ctx: Context, tool: LockedTool): string[]
/// ```
pub fn reconcile_rust_parts(ctx: &Context, tool: &LockedTool) -> Result<Vec<String>, ToolchainError> {
    let mut added: Vec<String> = Vec::new();
    // What: `?` returns early with the error when the missing-part lookup fails.
    // Why: a toolchain whose listing cannot be read is a failed tool, not an unchanged one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const missing = missingRustParts(ctx, tool);  // throws
    // ```
    for part in missing_rust_parts(ctx, tool)? {
        let subcommand: &str = part.kind.subcommand();
        let request: RunRequest = ctx.request("rustup", &[subcommand, "add", &part.name, "--toolchain", &tool.version]);
        let what: String = format!("rustup {subcommand} add ({})", part.name);
        run_checked(ctx, &request, &what, &tool.declaration.id)?;
        // What: `format!` builds the sentence the report prints in place of counting the tool unchanged.
        // Why: the reader has to learn which part was added to a toolchain that was already at its version.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // added.push(`${id}: ${version} was already installed; added the declared ${subcommand} ${name}`);
        // ```
        added.push(format!(
            "{}: {} was already installed; added the declared {subcommand} {}, which `rustup {subcommand} list --installed` did not show",
            tool.declaration.id, tool.version, part.name
        ));
    }
    return Ok(added);
}

/// What:
/// Installs the Android command-line tools from the locked zip into `<install dir>/cmdline-tools/<version>`
///           and runs `android sdk install <packages>` with `ANDROID_HOME` and `ANDROID_SDK_ROOT` set.
/// Why:
/// The zip unpacks a top-level `cmdline-tools` directory;
/// `sdkmanager` insists on finding itself at
///           `cmdline-tools/<version>/bin`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installAndroidSdk(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_android_sdk(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let artifact = artifact_for_host(tool, &ctx.host.platform)?;
    let download_path: PathBuf = ctx
        .scope
        .toolchain_root
        .join("downloads")
        .join(format!("android-sdk-{}-{}", tool.version, safe_file_name(&artifact.url)));
    download_verified(ctx, &artifact.url, &download_path, &artifact.sha256)?;
    let sdk_root: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version)?;
    let staging: PathBuf = sdk_root.join(".cmdline-tools-staging");
    let destination: PathBuf = sdk_root.join("cmdline-tools").join(&tool.version);
    // Best effort: a staging directory left by a killed run is cleared; absence is the normal case.
    let _ = fs::remove_dir_all(&staging);
    extract_archive(ctx, &download_path, &staging)?;
    if let Err(error) = fs::create_dir_all(sdk_root.join("cmdline-tools")) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", sdk_root.join("cmdline-tools").display())));
    }
    let _ = fs::remove_dir_all(&destination);
    if let Err(error) = fs::rename(staging.join("cmdline-tools"), &destination) {
        return Err(ToolchainError::failure(format!(
            "the Android command-line tools archive did not hold a cmdline-tools directory to move into {}: {error}",
            destination.display()
        )));
    }
    let _ = fs::remove_dir_all(&staging);
    let android_cli: PathBuf = destination.join("bin").join("android");
    let mut args: Vec<String> = vec!["sdk".to_string(), "install".to_string()];
    args.extend(tool.declaration.options.android_packages.clone().unwrap_or_default());
    let mut env = ctx.path_env();
    env.insert("ANDROID_HOME".to_string(), sdk_root.to_string_lossy().to_string());
    env.insert("ANDROID_SDK_ROOT".to_string(), sdk_root.to_string_lossy().to_string());
    let request: RunRequest = RunRequest {
        command: android_cli.to_string_lossy().to_string(),
        args,
        env: Some(env),
        cwd: None,
    };
    run_checked(ctx, &request, "android sdk install", &tool.declaration.id)?;
    return Ok(());
}
