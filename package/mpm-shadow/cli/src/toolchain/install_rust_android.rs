//! What:
//! The `rustup-toolchain` route (`rustup toolchain install --no-self-update <channel>`,
//! then components and
//!           targets) and the `android-sdk` route (command-line tools from a verified zip,
//! then `android sdk install` packages).
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

/// The rustup channel a tool's toolchain is installed as:
/// the plan's `packageId` when it names one (a `{version}` placeholder is already expanded),
/// else the locked version.
/// Why:
/// `rustup` names a channel that way (`nightly`, `1.90.0`, `nightly-2026-10-30`),
/// and every later `rustup` call about that toolchain must ask for the same channel.
fn rustup_channel(tool: &LockedTool) -> &str {
    return tool.declaration.package_id.as_deref().unwrap_or(&tool.version);
}

/// `rustup toolchain install --no-self-update <package id, or the locked version>`.
pub fn install_rust_toolchain(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let channel: &str = rustup_channel(tool);
    let request: RunRequest = ctx.request("rustup", &["toolchain", "install", "--no-self-update", channel]);
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
        let request: RunRequest = ctx.request("rustup", &["component", "add", &component, "--toolchain", rustup_channel(tool)]);
        run_checked(ctx, &request, &format!("rustup component add ({component})"), &tool.declaration.id)?;
    }
    for target in tool.declaration.options.rust_targets.clone().unwrap_or_default() {
        let request: RunRequest = ctx.request("rustup", &["target", "add", &target, "--toolchain", rustup_channel(tool)]);
        run_checked(ctx, &request, &format!("rustup target add ({target})"), &tool.declaration.id)?;
    }
    return Ok(());
}

/// What:
/// The names `rustup <listing subcommand> list --installed --toolchain <version>` prints,
/// one per line with the empty lines dropped,
/// for the `component` row or the `target` row of a toolchain.
/// Why:
/// Asking `rustup` what it holds is what lets a sync on a complete machine add nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installedNames(ctx: Context, tool: LockedTool, listing: 'component' | 'target'): string[]
/// ```
fn installed_names(ctx: &Context, tool: &LockedTool, listing: &str) -> Result<Vec<String>, ToolchainError> {
    let request: RunRequest = ctx.request("rustup", &[listing, "list", "--installed", "--toolchain", rustup_channel(tool)]);
    let what: String = format!("rustup {listing} list --installed");
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
/// Whether `installed` holds `name` as that exact text.
/// Why:
/// `rustup target list --installed` prints one bare triple per line,
/// so a longer name that merely starts with a declared one
/// (`wasm32-wasip1-threads` for a declared `wasm32-wasip1`)
/// is a different standard library and must not count as the declared target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listsExactly(installed: string[], name: string): boolean
/// ```
fn lists_exactly(installed: &[String], name: &str) -> bool {
    for listed in installed {
        if listed == name {
            return true;
        }
    }
    return false;
}

/// What:
/// Whether `installed` holds the component `name`,
/// either as that bare name (`rust-src`)
/// or as that name plus one of the toolchain's target triples
/// (`clippy` is installed when the listing shows `clippy-x86_64-unknown-linux-gnu`),
/// and in either spelling:
/// a run that already added the bare name covers a declaration that names the triple.
/// Why:
/// `rustup component list --installed` spells a component with the target triple it was built for,
/// while the lock declares the bare name,
/// and only a suffix that is one of the toolchain's own targets names the same component:
/// `rustc-dev-x86_64-unknown-linux-gnu` is not the component `rustc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listsComponent(installed: string[], name: string, targets: string[]): boolean
/// ```
fn lists_component(installed: &[String], name: &str, targets: &[String]) -> bool {
    for listed in installed {
        if listed == name {
            return true;
        }
        if spelled_with_one_of_the_targets(listed, name, targets) || spelled_with_one_of_the_targets(name, listed, targets) {
            return true;
        }
    }
    return false;
}

/// What:
/// Whether `long` is `short` plus a hyphen and one of `targets`.
/// Why:
/// It is the one test both directions of `lists_component` need:
/// the listing's spelling of the declared name,
/// and the declared name's spelling of the listing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spelledWithOneOfTheTargets(long: string, short: string, targets: string[]): boolean
/// ```
fn spelled_with_one_of_the_targets(long: &str, short: &str, targets: &[String]) -> bool {
    // What: `let Some(leftover) = ... else { return false }` binds the text after the prefix and leaves the
    // function with `false` when `long` does not start with it (the sibling is a plain `match`).
    // Why: `long` need not carry the prefix at all, which is the common case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!long.startsWith(short + '-')) return false;
    // ```
    let Some(leftover) = long.strip_prefix(&format!("{short}-")) else {
        return false;
    };
    return lists_exactly(targets, leftover);
}

/// What:
/// The names one option row of the lock declares,
/// or an empty list when the row is absent.
/// Why:
/// Both loops below treat "absent" and "empty" the same,
/// which is the row the sync has nothing to ask `rustup` about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declaredNames(row: string[] | undefined): string[]
/// ```
fn declared_names(row: &Option<Vec<String>>) -> Vec<String> {
    // What: `.clone()` copies the declaration's list out of the lock entry; `.unwrap_or_default()` then
    // substitutes an empty list when the row is absent (its sibling `.unwrap_or(value)` takes a value).
    // Why: the loops move each name out of the list, so they cannot borrow the lock entry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [...(row ?? [])];
    // ```
    return row.clone().unwrap_or_default();
}

/// What:
/// Runs `rustup <listing> add <name> --toolchain <channel>` and records the sentence the report prints.
/// Why:
/// Both rows add a part the same way,
/// and the sentence has to name the part and the listing that missed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addPart(ctx: Context, tool: LockedTool, listing: 'component' | 'target', name: string, added: string[]): void
/// ```
fn add_part(ctx: &Context, tool: &LockedTool, listing: &str, name: &str, added: &mut Vec<String>) -> Result<(), ToolchainError> {
    let request: RunRequest = ctx.request("rustup", &[listing, "add", name, "--toolchain", rustup_channel(tool)]);
    let what: String = format!("rustup {listing} add ({name})");
    run_checked(ctx, &request, &what, &tool.declaration.id)?;
    // What: `format!` builds the sentence the report prints in place of counting the tool unchanged.
    // Why: the reader has to learn which part was added to a toolchain that was already at its version.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // added.push(`${id}: ${version} was already installed; added the declared ${listing} ${name}`);
    // ```
    added.push(format!(
        "{}: {} was already installed; added the declared {listing} {name}, which `rustup {listing} list --installed` did not show",
        tool.declaration.id, tool.version
    ));
    return Ok(());
}

/// What:
/// Adds every declared component and target `rustup` does not have for this toolchain,
/// one sentence per part added into `added`,
/// and returns the error of the first listing or add that failed.
/// Why:
/// A toolchain another program installed at the locked version never reaches `install_tool`,
/// so the lock's `rustComponents` and `rustTargets` rows need a second place that applies them;
/// asking `rustup` first keeps a sync on a complete machine free of any add.
/// The sentences stay with the caller when a later add fails,
/// so parts this run did add are still reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reconcileRustParts(ctx: Context, tool: LockedTool, added: string[]): void  // throws
/// ```
pub fn reconcile_rust_parts(ctx: &Context, tool: &LockedTool, added: &mut Vec<String>) -> Result<(), ToolchainError> {
    let components: Vec<String> = declared_names(&tool.declaration.options.rust_components);
    let targets: Vec<String> = declared_names(&tool.declaration.options.rust_targets);
    if components.is_empty() && targets.is_empty() {
        return Ok(());
    }
    // What: `?` returns early with the error when the listing fails.
    // Why: a toolchain whose listings cannot be read is a failed tool, not an unchanged one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let installedTargets = installedNames(ctx, tool, 'target');  // throws
    // ```
    let mut installed_targets: Vec<String> = installed_names(ctx, tool, "target")?;
    // The components come second only because the check for them needs the toolchain's targets.
    if !components.is_empty() {
        let mut installed_components: Vec<String> = installed_names(ctx, tool, "component")?;
        let mut added_component: bool = false;
        for name in components {
            if lists_component(&installed_components, &name, &installed_targets) {
                continue;
            }
            add_part(ctx, tool, "component", &name, added)?;
            // A successful add leaves that component installed, so a later declaration naming the same
            // component (the same text, or the same part under another spelling) is not added again.
            installed_components.push(name.clone());
            added_component = true;
        }
        // A component add can bring a target's standard library with it (`rust-std-<triple>`), so the target
        // listing is read again before any target is judged missing.
        if added_component && !targets.is_empty() {
            installed_targets = installed_names(ctx, tool, "target")?;
        }
    }
    for name in targets {
        if lists_exactly(&installed_targets, &name) {
            continue;
        }
        add_part(ctx, tool, "target", &name, added)?;
        // Same reason as the components above: the triple is installed now, and a duplicate declaration of it
        // must not add it twice.
        installed_targets.push(name.clone());
    }
    return Ok(());
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
