//! What:
//! The `bin-sourced` route:
//! install a tool from a GitHub release with the `bin` CLI,
//! after seeding `bin`'s
//!           configuration so it downloads into the toolchain's own bin directory.
//! Why:
//! Most single-binary tools (ripgrep,
//! dprint,
//! caddy,
//! zizmor) come this way;
//! the release tag is derived from
//!           the locked artifact URL,
//! so the lock stays the only record of what is installed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-bin.ts
//! export function releaseTagUrl(artifactUrl: string): string
//! export function seedBinConfig(downloadDir: string, configPath: string): void
//! export function installBinSourced(ctx: Context, tool: LockedTool): void
//! ```

/// Imports what this module uses from `serde_json`.
use serde_json::{Map, Value};
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::install_support`.
use super::install_support::{artifact_for_host, failed_step};
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest};

/// The prefix of every artifact URL this route accepts.
const GITHUB_PREFIX: &str = "https://github.com/";

/// The error for an artifact URL that is not a GitHub release download.
fn release_refusal(artifact_url: &str) -> ToolchainError {
    return ToolchainError::failure(format!(
        "cannot derive a release-tag URL from {artifact_url}: bin-sourced artifacts must be \
         https://github.com/<owner>/<repo>/releases/download/<tag>/<asset>"
    ));
}

/// What:
/// The release page URL (`https://github.com/<owner>/<repo>/releases/tag/<tag>`) for a locked asset URL of
///           the form `https://github.com/<owner>/<repo>/releases/download/<tag>/<asset>`.
/// Why:
/// `bin install` takes the release page,
/// not the asset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function releaseTagUrl(artifactUrl: string): string  // throws ReleaseTagUrlError
/// ```
pub fn release_tag_url(artifact_url: &str) -> Result<String, ToolchainError> {
    let Some(rest) = artifact_url.strip_prefix(GITHUB_PREFIX) else {
        return Err(release_refusal(artifact_url));
    };
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.len() < 6 || segments[2] != "releases" || segments[3] != "download" {
        return Err(release_refusal(artifact_url));
    }
    let (owner, repo, tag) = (segments[0], segments[1], segments[4]);
    if owner.is_empty() || repo.is_empty() || tag.is_empty() {
        return Err(release_refusal(artifact_url));
    }
    return Ok(format!("{GITHUB_PREFIX}{owner}/{repo}/releases/tag/{}", percent_decode(tag)));
}

/// What:
/// Decodes `%XX` escapes (`%2B` to `+`);
/// a malformed escape is kept as written.
/// Why:
/// A tag such as `jdk-21.0.13+9` appears escaped in download URLs and plain on its release page.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeURIComponent(tag)
/// ```
pub fn percent_decode(text: &str) -> String {
    let bytes: &[u8] = text.as_bytes();
    let mut decoded: Vec<u8> = Vec::new();
    let mut index: usize = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) = (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
        {
            decoded.push(high * 16 + low);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    return String::from_utf8_lossy(&decoded).to_string();
}

/// The value of one hex digit,
/// or `None`.
fn hex_value(byte: u8) -> Option<u8> {
    return match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    };
}

/// What:
/// Writes `bin`'s configuration file so downloads land in `download_dir`,
/// keeping every other setting the
///           file already holds (a malformed file is replaced).
/// Why:
/// `bin` reads `BIN_CONFIG`;
/// pointing its `default_path` at the toolchain's bin directory keeps its downloads
///           out of the user's own `bin` setup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function seedBinConfig(downloadDir: string, configPath: string): void
/// ```
pub fn seed_bin_config(download_dir: &Path, config_path: &Path) -> Result<(), ToolchainError> {
    let mut document: Map<String, Value> = match fs::read_to_string(config_path) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(existing)) => existing,
            // Not an object, or not JSON at all: start again; the old content could not be used.
            _ => Map::new(),
        },
        Err(_) => Map::new(),
    };
    document.insert("default_path".to_string(), Value::String(download_dir.to_string_lossy().to_string()));
    if !document.contains_key("bins") {
        document.insert("bins".to_string(), Value::Object(Map::new()));
    }
    if let Some(parent) = config_path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", parent.display())));
    }
    let mut text: String = match serde_json::to_string_pretty(&Value::Object(document)) {
        Ok(rendered) => rendered,
        Err(error) => return Err(ToolchainError::failure(format!("cannot render the bin configuration: {error}"))),
    };
    text.push('\n');
    if let Err(error) = fs::write(config_path, text) {
        return Err(ToolchainError::failure(format!("cannot write {}: {error}", config_path.display())));
    }
    return Ok(());
}

/// The `--name` patterns tried in order for one binary:
/// nested in the asset,
/// directly in it,
/// then the asset with the
/// binary as the path.
fn name_attempts(asset: &str, name: &str) -> Vec<Vec<String>> {
    return vec![
        vec!["--name".to_string(), format!("{asset}/**/{name}")],
        vec!["--name".to_string(), format!("{asset}/{name}")],
        vec!["--name".to_string(), asset.to_string(), name.to_string()],
    ];
}

/// Whether `name` is one of the alias names
/// (so it is linked,
/// not downloaded).
fn is_alias(aliases: &[(String, String)], name: &str) -> bool {
    for (alias, _) in aliases {
        if alias == name {
            return true;
        }
    }
    return false;
}

/// What:
/// Installs one bin-sourced tool:
/// for each declared bin that is not an alias,
/// `bin install --force <release
///           page> --name <pattern>`,
/// trying the three patterns in order.
/// Why:
/// Aliases (`bunx` for `bun`) are links,
/// not downloads,
/// so only primaries are installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installBinSourced(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_bin_sourced(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let artifact = artifact_for_host(tool, &ctx.host.platform)?;
    let tag_url: String = release_tag_url(&artifact.url)?;
    let asset: String = artifact.url.rsplit('/').next().unwrap_or("").to_string();
    let aliases: Vec<(String, String)> = tool.declaration.options.bin_aliases.clone().unwrap_or_default();
    seed_bin_config(&ctx.scope.toolchain_bin_dir, &ctx.scope.bin_config_file)?;
    let Some(cli) = ctx.find_bin_of("bin", "bin", &[]) else {
        return Err(ToolchainError::failure(format!(
            "the bin CLI is not installed; expected it under {}",
            super::layout::tool_install_dir(&ctx.scope.toolchain_root, "bin", "").display()
        )));
    };
    let mut env = ctx.env.all().clone();
    env.insert("HOME".to_string(), ctx.scope.home.to_string_lossy().to_string());
    env.insert("BIN_CONFIG".to_string(), ctx.scope.bin_config_file.to_string_lossy().to_string());
    if let Some(token) = ctx.github_token() {
        env.insert("GITHUB_TOKEN".to_string(), token);
    }
    for name in &tool.declaration.bins {
        if is_alias(&aliases, name) {
            continue;
        }
        let mut last: Option<RunOutcome> = None;
        let mut installed: bool = false;
        for attempt in name_attempts(&asset, name) {
            let mut args: Vec<String> = vec!["install".to_string(), "--force".to_string(), tag_url.clone()];
            args.extend(attempt);
            let request: RunRequest = RunRequest {
                command: cli.to_string_lossy().to_string(),
                args,
                env: Some(env.clone()),
                cwd: None,
            };
            let outcome: RunOutcome = ctx.run(&request);
            if outcome.succeeded() {
                installed = true;
                break;
            }
            last = Some(outcome);
        }
        if !installed {
            let failure: RunOutcome = last.unwrap_or(RunOutcome { stdout: String::new(), stderr: "no attempt ran".to_string(), code: 1 });
            return Err(failed_step(&format!("bin install ({name})"), &tool.declaration.id, &failure));
        }
    }
    let bin_dir: PathBuf = ctx.scope.toolchain_bin_dir.clone();
    if bin_dir.exists() && !bin_dir.is_dir() {
        return Err(ToolchainError::failure(format!(
            "bin install for {} replaced the toolchain bin directory {} with a file; restore it from its '.bin.old' sibling",
            tool.declaration.id,
            bin_dir.display()
        )));
    }
    return Ok(());
}
