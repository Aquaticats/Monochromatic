//! What:
//! What a failed download says when a whole finished copy of the version is already on disk
//! under a hidden name this build never publishes.
//! Why:
//! An offline machine whose last install was killed under an earlier build keeps the version's bytes
//! and loses the version's name,
//! and the failure it reads is the network's,
//! so nothing tells the person the copy is there.
//! It sits in a file of its own because the install step next to it is longer than a person reads at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-failed-download-names-the-copy-it-cannot-publish.ts
//! export function addingTheCopyThisBuildCannotPublish(error: ToolchainError, tool: LockedTool, versionDir: string): ToolchainError;
//! ```

/// Imports the hidden names a killed install leaves,
/// and what one name's installer answers.
use super::super::archive_staging::{
    Liveness,
    hidden_siblings,
};
/// Imports `ToolchainError` from `super::error`.
use super::super::error::ToolchainError;
/// Imports the locked tool whose version the download was for.
use super::super::model::LockedTool;
/// Imports the question that tells a finished install from an unfinished one.
use super::super::published_archive::is_finished;
/// Imports the text a version directory's own name gives.
use super::super::util::file_name_text;
/// Imports what this file uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// What:
/// The hidden name of a finished copy of this version whose installer this build cannot judge,
/// or nothing when there is none.
/// Why:
/// A name from another process namespace,
/// a record `/proc` hides,
/// and a name an earlier build wrote
/// are never judged ended,
/// so no run of this build claims,
/// publishes or removes the copy they hold;
/// a person offline then needs the path that copy sits at,
/// which nothing else prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aFinishedCopyThisBuildCannotPublish(versionDir: string, firstBin: string): string | undefined {
///   const siblings = hiddenSiblings(dirname(versionDir));
///   return siblings.find((s) => s.version === basename(versionDir) && s.liveness() === 'unknown' && isFinished(s.path, firstBin))?.path;
/// }
/// ```
fn a_finished_copy_this_build_cannot_publish(
    version_dir: &Path,
    first_bin: &str,
) -> Option<PathBuf> {
    // What: the tool directory the version's own name sits in, which holds every hidden sibling of every version.
    // Why: the hidden names sit beside the version directory, never inside it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const directory = dirname(versionDir);
    // ```
    let directory: &Path = version_dir.parent()?;
    let version: String = file_name_text(version_dir);
    // What: a listing that fails answers nothing here, because the download's own error is what the person acts on.
    // Why: a second failure must not replace the failure that stopped the install.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let siblings; try { siblings = hiddenSiblings(directory); } catch { return undefined; }
    // ```
    let Ok(siblings) = hidden_siblings(directory) else {
        return None;
    };
    for sibling in siblings {
        if sibling.version != version {
            continue;
        }
        if sibling.liveness() != Liveness::Unknown {
            continue;
        }
        if !is_finished(&sibling.path, first_bin) {
            continue;
        }
        return Some(sibling.path);
    }
    return None;
}

/// What:
/// The download's failure,
/// with a sentence naming a finished copy of the version this build never publishes.
/// Why:
/// An install that fails offline leaves the version's name empty and a whole finished copy two names away,
/// and a person who reads only "download failed" has no way to know that;
/// the sentence names the copy,
/// why no run publishes it,
/// and the two ways an offline machine gets the version back,
/// each worded for what this run read:
/// a download brings the version back only when its bytes match the lock and the install can keep them,
/// so a failure whose network was there is not told a network would repair it,
/// and the step by hand copies the copy's contents,
/// which reaches a version's name that holds nothing and one that already holds a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addingTheCopyThisBuildCannotPublish(error: ToolchainError, tool: LockedTool, versionDir: string): ToolchainError {
///   const copy = aFinishedCopyThisBuildCannotPublish(versionDir, requiredProgram(tool) ?? '');
///   if (copy === undefined) return error;
///   return { kind: error.kind, message: `${error.message}; a finished copy of ${tool.id} ${tool.version} sits under the hidden name ${copy}, whose installer this build cannot judge, so no install of this build publishes it; the version comes back from a download that matches this lock's SHA-256 and that the install can keep, or by copying that copy's contents into ${versionDir} by hand (\`cp --archive ${copy}/. ${versionDir}\`), which checks none of its bytes against the lock` };
/// }
/// ```
pub(super) fn adding_the_copy_this_build_cannot_publish(
    error: ToolchainError,
    tool: &LockedTool,
    version_dir: &Path,
) -> ToolchainError {
    let first_bin: &str = super::required_program(tool).unwrap_or("");
    let Some(copy) = a_finished_copy_this_build_cannot_publish(version_dir, first_bin) else {
        return error;
    };
    return ToolchainError {
        kind: error.kind,
        message: format!(
            "{}; a finished copy of {} {} sits under the hidden name {}, whose installer this build cannot judge, so no install of this build publishes it; the version comes back from a download that matches this lock's SHA-256 and that the install can keep, or by copying that copy's contents into {} by hand (`cp --archive {}/. {}`), which checks none of its bytes against the lock",
            error.message,
            tool.declaration.id,
            tool.version,
            copy.display(),
            version_dir.display(),
            copy.display(),
            version_dir.display()
        ),
    };
}
