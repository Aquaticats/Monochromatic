//! What:
//! The `archive` route:
//! download the artifact of the host's platform,
//! verify its SHA-256 against the lock,
//!           and unpack it (or copy the single file) into the tool's install directory.
//! Why:
//! Node,
//! uv,
//! cmake,
//! zig,
//! the JDK and `bin` itself come this way;
//! the hash check is what makes the bootstrap
//!           of a prebuilt `mpm-shadow` archive safe to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-archive.ts
//! export function installArchive(ctx: Context, tool: LockedTool): void
//! ```

/// Imports `GzDecoder` from `flate2::read`.
use flate2::read::GzDecoder;
/// Imports what this module uses from `std::fs`.
use std::fs::{self, File};
/// Imports `io` from `std`.
use std::io;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::install_support`.
use super::install_support::{
    ARCHIVE_SUFFIXES, artifact_for_host, download_verified, extract_archive, extract_conda, make_executable, restore_exec_bits,
};
/// Imports what this module uses from `super::layout`.
use super::layout::{install_dir_name, tool_install_dir};
/// Imports the suffix check.
use super::util::ends_with_any;
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports what this module uses from `super::published_archive`.
use super::published_archive::{first_bin_of_tool, is_published_archive, point_current_at_newest_archive, remove_older_archives};

/// What:
/// The last path segment of a URL (query and fragment removed) made safe as a file name:
/// characters outside
///           letters,
/// digits,
/// `.`,
/// `_`,
/// `-` are replaced,
/// and an empty segment,
/// `.` or `..` becomes `download`.
/// Why:
/// The name is joined onto a local directory,
/// so a hostile URL must not be able to name a parent directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function safeFileName(url: string): string
/// ```
pub fn safe_file_name(url: &str) -> String {
    let without_query: &str = url.split(['?', '#']).next().unwrap_or(url);
    let segment: &str = without_query.rsplit('/').next().unwrap_or("");
    let cleaned: String = install_dir_name(segment);
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        return "download".to_string();
    }
    return cleaned;
}

/// Writes the gunzipped contents of `source` to `target`.
fn gunzip_to(source: &Path, target: &Path) -> Result<(), ToolchainError> {
    let input: File = match File::open(source) {
        Ok(opened) => opened,
        Err(error) => return Err(ToolchainError::failure(format!("cannot open {}: {error}", source.display()))),
    };
    let mut output: File = match File::create(target) {
        Ok(created) => created,
        Err(error) => return Err(ToolchainError::failure(format!("cannot create {}: {error}", target.display()))),
    };
    let mut decoder: GzDecoder<File> = GzDecoder::new(input);
    if let Err(error) = io::copy(&mut decoder, &mut output) {
        return Err(ToolchainError::failure(format!("cannot decompress {}: {error}", source.display())));
    }
    return Ok(());
}

/// What:
/// Installs one archive-route tool,
/// and for a tool this repository publishes points its `current` link at the newest finished version
/// and deletes every older one.
/// Why:
/// Older repository-published tools are never kept once a newer one is installed,
/// and every tree's exposed link goes through `current`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installArchive(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_archive(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    install_archive_files(ctx, tool)?;
    if is_published_archive(tool) {
        let first_bin: &str = first_bin_of_tool(tool);
        point_current_at_newest_archive(&ctx.scope.toolchain_root, &tool.declaration.id, first_bin)?;
        remove_older_archives(&ctx.scope.toolchain_root, &tool.declaration.id, first_bin)?;
    }
    return Ok(());
}

/// What:
/// Downloads,
/// verifies and unpacks the archive of one archive-route tool into its version directory.
/// Why:
/// A URL ending in an archive suffix is unpacked and its launchers get their execute bits back;
/// any other
///           URL is a single binary (gunzipped when it ends in `.gz`) saved under the tool's first bin name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installArchiveFiles(ctx: Context, tool: LockedTool): void
/// ```
fn install_archive_files(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let artifact = artifact_for_host(tool, &ctx.host.platform)?;
    let file_name: String = safe_file_name(&artifact.url);
    let download_path: PathBuf = ctx.scope.toolchain_root.join("downloads").join(format!(
        "{}-{}-{}",
        install_dir_name(&tool.declaration.id),
        install_dir_name(&tool.version),
        file_name
    ));
    download_verified(ctx, &artifact.url, &download_path, &artifact.sha256)?;
    let install_dir: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version);
    if let Err(error) = fs::create_dir_all(&install_dir) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", install_dir.display())));
    }
    let is_archive: bool = ends_with_any(&artifact.url, &ARCHIVE_SUFFIXES);
    if is_archive || artifact.url.ends_with(".conda") {
        if artifact.url.ends_with(".conda") {
            extract_conda(ctx, &download_path, &install_dir)?;
        } else {
            extract_archive(ctx, &download_path, &install_dir)?;
        }
        let mut names: Vec<String> = tool.declaration.bins.clone();
        if let Some(aliases) = &tool.declaration.options.bin_aliases {
            for (alias, _) in aliases {
            names.push(alias.clone());
        }
        }
        restore_exec_bits(&install_dir, &names);
        return Ok(());
    }
    let binary_name: String = match tool.declaration.bins.first() {
        Some(first) => first.clone(),
        None => file_name,
    };
    let target: PathBuf = install_dir.join(binary_name);
    if artifact.url.ends_with(".gz") {
        gunzip_to(&download_path, &target)?;
    } else if let Err(error) = fs::copy(&download_path, &target) {
        return Err(ToolchainError::failure(format!("cannot copy {} to {}: {error}", download_path.display(), target.display())));
    }
    return make_executable(&target);
}
