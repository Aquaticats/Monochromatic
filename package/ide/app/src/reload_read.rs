//! Resolve project opens on the reader thread,
//!  then prepare source correspondence and classifications.

/// Existing source reads retain regular-file and UTF-8 validation before producing a prepared change.
use crate::{
    file_reload::{QuietRead, read_reload, read_reload_if_quiet},
    reload_worker::{ReloadReply, ReloadRequest, SyntaxReply},
    source_style::SourceStyles,
    syntax::SyntaxEngine,
    workspace::Workspace,
};
/// Initialization failures remain distinct from unrecognized plain-text source.
use anyhow::Result;
/// Syntax classifies the exact immutable rope used by the prepared source revision.
use helix_core::Rope;
/// Paths stay native rather than becoming labels or shell arguments.
use std::path::Path;

/// Prepare syntax without making a classification failure discard otherwise readable source text.
fn classify(engine: &Result<SyntaxEngine>, path: &Path, text: &Rope, revision: u64) -> SyntaxReply {
    // What: match extracts the initialized engine or wraps its original initialization diagnostic.
    // Why: Missing runtime assets must not masquerade as an unrecognized language.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result = engine.ok ? highlight(engine.value, path, text) : unstarted(engine.error, path, text);
    // ```
    let result = match engine {
        Ok(active) => active.highlight(path, text),
        Err(error) => unstarted(error, path, text),
    };
    return SyntaxReply { revision, result };
}

/// What:
///  The error that tells the reader highlighting could not start for `path`,
///  with the engine's own failure as the reason.
/// Why:
///  Both branches that keep the failure visible build the same message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const initializationFailure = (error: Error, path: string) => new Error(`Cannot initialize highlighting for ${path}: ${error}`);
/// ```
fn initialization_failure(error: &anyhow::Error, path: &Path) -> anyhow::Error {
    return anyhow::anyhow!(
        "Cannot initialize highlighting for {}: {error:#}",
        path.display()
    );
}

/// What:
///  The highlighting result when the engine could not start.
///  `Ok(None)` is plain text;
///  `Err` is
///       the initialization failure,
///  shown and logged as a warning by the caller.
/// Why:
///  A file no language applies to,
///  such as plain text,
///  loses nothing,
///  so it is not a failure;
///      a file some language applies to loses its highlighting,
///  which stays a visible failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unstarted(error: Error, path: string, text: Rope): Styles | undefined // throws for a language file
/// ```
fn unstarted(error: &anyhow::Error, path: &Path, text: &Rope) -> Result<Option<SourceStyles>> {
    // What: `match` on the recognition result: `Ok(false)` is "no language applies", `Ok(true)` is
    //       "a language applies", and `Err` is a recognition failure.
    // Why: Only the first is plain text; the other two keep the initialization failure visible.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { if (!SyntaxEngine.namesALanguage(path, text)) return undefined; } catch (e) { log(e); }
    // throw failure();
    // ```
    match SyntaxEngine::names_a_language(path, text) {
        Ok(false) => {
            tracing::debug!(path = %path.display(), %error, "highlighting cannot start, but no language applies to this file; it stays plain text");
            return Ok(None);
        }
        Ok(true) => {
            return Err(initialization_failure(error, path));
        }
        Err(recognition) => {
            tracing::debug!(path = %path.display(), error = %recognition, "cannot tell whether a language applies to this file");
            return Err(initialization_failure(error, path));
        }
    }
}

/// Project opens resolve within the explicit root;
///  ordinary refreshes retain their already accepted target.
pub(crate) fn prepare(
    request: ReloadRequest,
    workspace: Option<Workspace>,
    syntax: &Result<SyntaxEngine>,
) -> ReloadReply {
    // What: if-let extracts an optional project boundary; clone owns the already accepted path otherwise.
    // Why: New tree/search opens cannot escape the project through parent components or symbolic links.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const target = workspace === undefined ? request.path : workspace.resolve(request.path);
    // ```
    let resolved = if let Some(project) = workspace {
        project.resolve(&request.path)
    } else {
        Ok(request.path.clone())
    };
    let path = match resolved {
        Ok(path) => path,
        Err(error) => {
            tracing::warn!(path = %request.path.display(), %error, "project source resolution failed");
            // None means no accepted target or classification; preserve the resolution failure.
            return ReloadReply {
                generation: request.generation,
                resolved_path: None,
                result: Err(error),
                syntax: None,
                recent_write: false,
            };
        }
    };
    // What: `if let Some(quiet)` runs the quiet read only when the request requires quiet; `match` then
    //       turns a recent write into an early reply and a finished read into the usual result.
    // Why: A read no notification asked for must not show a save in progress.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const outcome = quiet === undefined ? readReload(...) : readReloadIfQuiet(..., quiet);
    // if (outcome.kind === 'recentlyWritten') return { ..., recentWrite: true };
    // ```
    let result = if let Some(quiet) = request.require_quiet {
        match read_reload_if_quiet(&request.snapshot, &path, quiet) {
            Ok(QuietRead::RecentlyWritten) => {
                // Ok(None) installs nothing; the flag tells the UI to wait for the writer.
                return ReloadReply {
                    generation: request.generation,
                    resolved_path: Some(path),
                    result: Ok(None),
                    syntax: None,
                    recent_write: true,
                };
            }
            Ok(QuietRead::Read(update)) => Ok(update),
            Err(error) => Err(error),
        }
    } else {
        // Lend the base snapshot and accepted path without sharing any mutable native state.
        read_reload(&request.snapshot, &path)
    };
    let classified = match &result {
        Ok(Some(reload)) => Some(classify(
            syntax,
            &path,
            reload.text(),
            request.snapshot.revision() + 1,
        )),
        Ok(None) if request.highlight_unchanged => Some(classify(
            syntax,
            &path,
            request.snapshot.text(),
            request.snapshot.revision(),
        )),
        // Failures keep their original result; unchanged accepted classifications need no repeated parse.
        _ => None,
    };
    // Some transfers the accepted path to the consumer so symlink aliases do not become separate open identities.
    return ReloadReply {
        generation: request.generation,
        resolved_path: Some(path),
        result,
        syntax: classified,
        recent_write: false,
    };
}

/// The highlighting result when the engine could not start,
///  for plain text and for a language file.
#[cfg(test)]
#[path = "reload_read_tests.rs"]
mod tests;
