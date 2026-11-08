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

/// Imports what this module uses from `super::archive_staging`.
use super::archive_staging::{discard_path, publish_staged, staging_dir_of, write_complete_marker};
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::install_support`.
use super::install_support::{
    ARCHIVE_SUFFIXES, artifact_for_host, atomic_symlink, download_verified, extract_archive, extract_conda, make_executable, restore_exec_bits,
};
/// Imports what this module uses from `super::layout`.
use super::layout::{find_bin_under, install_dir_name, tool_downloads_dir, tool_install_dir};
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

/// The program an alias names:
/// the second half of a `binAliases` entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function primaryOfAlias(entry: [string, string]): string
/// ```
fn primary_of_alias(entry: &(String, String)) -> String {
    return entry.1.clone();
}

/// What:
/// Every program name the plan declares for `tool`:
/// its `bins` and the program side of every `binAliases` entry,
/// sorted and without repeats.
/// Why:
/// An alias names another program of the same archive,
/// so a stable link is made once per program,
/// never for the alias's own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function programNames(tool: LockedTool): string[]
/// ```
fn program_names(tool: &LockedTool) -> Vec<String> {
    let mut names: Vec<String> = tool.declaration.bins.clone();
    if let Some(aliases) = &tool.declaration.options.bin_aliases {
        names.extend(aliases.iter().map(primary_of_alias));
    }
    names.sort();
    names.dedup();
    return names;
}

/// The target a stable link gets:
/// the program's path relative to the version directory,
/// so the link stays right when the whole toolchain root moves.
#[cfg(unix)]
fn stable_link_target(version_dir: &Path, program: &Path) -> PathBuf {
    return match program.strip_prefix(version_dir) {
        Ok(relative) => relative.to_path_buf(),
        Err(_) => program.to_path_buf(),
    };
}

/// A platform without symbolic links makes a hard link,
/// whose relative target would resolve against the working directory.
#[cfg(not(unix))]
fn stable_link_target(_version_dir: &Path, program: &Path) -> PathBuf {
    return program.to_path_buf();
}

/// What:
/// Gives every declared program of a release archive of this repository a path that names no version:
/// `<version directory>/<program name>`,
/// linked to the program inside the folder the archive unpacks into,
/// so the program is `<tool directory>/current/<program name>`.
/// Why:
/// A release archive unpacks into a folder whose name carries the version,
/// so the program below `current` is `current/<version folder>/<program>`,
/// and every launcher record that named it went stale the moment any tree installed a newer version.
/// A launcher does repair such a record by following `current`,
/// but only once it runs;
/// a link of the program's own name inside the version directory makes the program reachable at
/// `current/<program name>`,
/// the shape a `cargo-binstall` tool already has,
/// so a record written through `current` names a program that is there after every version move.
/// A directory an earlier release unpacked in place carries no such link,
/// and its program is still found below `current`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function linkStablePrograms(tool: LockedTool, versionDir: string): void  // throws
/// ```
pub fn link_stable_programs(tool: &LockedTool, version_dir: &Path) -> Result<(), ToolchainError> {
    if !is_published_archive(tool) {
        return Ok(());
    }
    for name in program_names(tool) {
        let link: PathBuf = version_dir.join(&name);
        let Some(program) = find_bin_under(version_dir, &name) else {
            continue;
        };
        if program == link {
            continue;
        }
        atomic_symlink(&link, &stable_link_target(version_dir, &program))?;
    }
    return Ok(());
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

/// The program the install must hold for it to count as finished:
/// the first declared bin of a tool this repository publishes,
/// or `None` for any other tool (a program missing from a third-party archive is reported by the sync as a missing
/// bin,
/// as it always was,
/// and never fails the install).
fn required_program(tool: &LockedTool) -> Option<&str> {
    if !is_published_archive(tool) {
        return None;
    }
    let first_bin: &str = first_bin_of_tool(tool);
    if first_bin.is_empty() {
        return None;
    }
    return Some(first_bin);
}

/// What:
/// Unpacks the verified download of one archive-route tool into `into`,
/// then checks that a tool this repository publishes holds its first program as an executable file.
/// Why:
/// A URL ending in an archive suffix is unpacked and its launchers get their execute bits back;
/// any other
///           URL is a single binary (gunzipped when it ends in `.gz`) saved under the tool's first bin name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unpackInto(ctx: Context, tool: LockedTool, url: string, download: string, into: string): void
/// ```
fn unpack_into(ctx: &Context, tool: &LockedTool, url: &str, download_path: &Path, into: &Path) -> Result<(), ToolchainError> {
    let is_archive: bool = ends_with_any(url, &ARCHIVE_SUFFIXES);
    if is_archive || url.ends_with(".conda") {
        if url.ends_with(".conda") {
            extract_conda(ctx, download_path, into)?;
        } else {
            extract_archive(ctx, download_path, into)?;
        }
        let mut names: Vec<String> = tool.declaration.bins.clone();
        if let Some(aliases) = &tool.declaration.options.bin_aliases {
            for (alias, _) in aliases {
                names.push(alias.clone());
            }
        }
        restore_exec_bits(into, &names);
    } else {
        let binary_name: String = match tool.declaration.bins.first() {
            Some(first) => first.clone(),
            None => safe_file_name(url),
        };
        let target: PathBuf = into.join(binary_name);
        if url.ends_with(".gz") {
            gunzip_to(download_path, &target)?;
        } else if let Err(error) = fs::copy(download_path, &target) {
            return Err(ToolchainError::failure(format!("cannot copy {} to {}: {error}", download_path.display(), target.display())));
        }
        make_executable(&target)?;
    }
    if let Some(program) = required_program(tool)
        && find_bin_under(into, program).is_none()
    {
        return Err(ToolchainError::failure(format!(
            "the archive {url} for {} {} holds no executable file named {program}, so nothing was installed from it; \
             the plan's bins for {} must name a program the release contains",
            tool.declaration.id, tool.version, tool.declaration.id
        )));
    }
    return Ok(());
}

/// Unpacks the verified download into `staging`,
/// gives a release archive of this repository its version-independent program paths,
/// and writes the completion marker last.
fn unpack_and_mark(ctx: &Context, tool: &LockedTool, url: &str, download_path: &Path, staging: &Path) -> Result<(), ToolchainError> {
    unpack_into(ctx, tool, url, download_path, staging)?;
    link_stable_programs(tool, staging)?;
    return write_complete_marker(staging, &tool.declaration.id, &tool.version);
}

/// What:
/// Downloads,
/// verifies and unpacks the archive of one archive-route tool into its version directory.
/// Why:
/// The archive is unpacked into a hidden staging directory,
/// checked,
/// marked complete and only then renamed to the version's name (`archive_staging.rs`),
/// so a download that fails to unpack,
/// or a release that lacks its program,
/// leaves no directory at the version's name and touches no copy that was already there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installArchiveFiles(ctx: Context, tool: LockedTool): void
/// ```
fn install_archive_files(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let artifact = artifact_for_host(tool, &ctx.host.platform)?;
    let file_name: String = safe_file_name(&artifact.url);
    let download_path: PathBuf = tool_downloads_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version)?.join(&file_name);
    download_verified(ctx, &artifact.url, &download_path, &artifact.sha256)?;
    let install_dir: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version)?;
    let staging: PathBuf = staging_dir_of(&install_dir);
    // A leftover of a killed run that had this process id would be unpacked over.
    discard_path(&staging);
    if let Err(error) = fs::create_dir_all(&staging) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", staging.display())));
    }
    if let Err(error) = unpack_and_mark(ctx, tool, &artifact.url, &download_path, &staging) {
        discard_path(&staging);
        return Err(error);
    }
    return publish_staged(&staging, &install_dir, required_program(tool).unwrap_or(""));
}
