//! Latest-request-wins source opening without replacing the displayed document before a successful read.

/// Opened source reuses the existing revision-aware reader,
///  syntax engine,
///  and read-only project boundary.
use crate::{
    document::{Document, ReadingPosition},
    reload_worker::{ReloadReply, ReloadRequest, ReloadWorker, SyntaxReply},
    workspace::Workspace,
};
/// Protocol inconsistencies remain errors rather than silently installing mismatched source.
use anyhow::{Context, Result, bail};
/// Requested and resolved paths retain native filenames.
use std::path::PathBuf;

/// Ready-to-display source and its matching classifications from one successful background read.
pub struct OpenedFile {
    /// Canonical target returned by project-boundary resolution,
    ///  not a lossy label or symlink alias.
    pub path: PathBuf,
    /// Fresh reading state starts at the beginning rather than mapping the previous file's selection.
    pub document: Document,
    /// Classification may report an independent parser failure without discarding readable text.
    pub syntax: Option<SyntaxReply>,
    /// True for a file outside the project,
    ///  opened read-only from a language target.
    pub outside_project: bool,
}

/// One executing read plus one replaceable desired target;
///  no unbounded file-open queue.
pub struct FileOpener {
    /// Boundary resolution runs on the reader thread for every new target.
    workspace: Workspace,
    /// Reuse the existing bounded read/diff/classification lifecycle.
    worker: ReloadWorker,
    /// Latest requested open identity,
    ///  independent of displayed-file and document revision identities.
    generation: u64,
    /// Only the latest waiting target is retained while an older read finishes.
    pending: Option<PathBuf>,
    /// The waiting target lies outside the project and is read without project resolution.
    pending_outside: bool,
    /// The read submitted for the current generation lies outside the project.
    submitted_outside: bool,
}

/// Turn an accepted empty-base read into a fresh document with a collapsed initial caret.
fn opened(reply: ReloadReply, outside_project: bool) -> Result<OpenedFile> {
    // What: ? propagates the current source-read error before attempting to use a resolved target.
    // Why: Failed opens retain the previously displayed document at the caller boundary.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const update = unwrapResult(reply.result);
    // ```
    let update = reply.result?;
    let path = reply
        .resolved_path
        .context("Successful project open has no resolved source path")?;
    let mut document = Document::new("");
    // What: Some extracts a changed source; an empty file legitimately has no update from the empty base.
    // Why: Empty UTF-8 files remain valid open targets.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (update !== undefined && !document.applyReload(update)) throw new Error('Mismatched base');
    // ```
    // A let-chain combines optional extraction and its boolean rejection condition without another nested block.
    if let Some(reload) = update
        && !document.apply_reload(reload)
    {
        bail!(
            "Cannot install opened source {}: its base revision does not match",
            path.display()
        );
    }
    document.select(ReadingPosition {
        anchor: 0,
        head: 0,
        viewport: 0,
        affinity: crate::document::ReadingAffinity::Downstream,
    });
    if let Some(syntax) = &reply.syntax
        && syntax.revision != document.revision()
    {
        bail!(
            "Cannot install opened source {}: its classification revision does not match",
            path.display()
        );
    }
    // Ok transfers the completed reading state without copying the source into another text representation.
    return Ok(OpenedFile {
        path,
        document,
        syntax: reply.syntax,
        outside_project,
    });
}

/// Unexpected reader termination must release pending UI work after reporting its error.
#[cfg(test)]
#[path = "file_open_tests.rs"]
mod tests;

/// Requests and polling never read filesystem contents on the caller's thread.
impl FileOpener {
    /// Create one reader for new opens while the displayed source can retain its own refresh worker.
    pub fn new(workspace: Workspace) -> Result<Self> {
        // The worker owns its thread; None records that no file has been requested yet.
        return Ok(Self {
            workspace,
            worker: ReloadWorker::new()?,
            generation: 0,
            pending: None,
            pending_outside: false,
            submitted_outside: false,
        });
    }

    /// Invalidate earlier replies even when the new intent is to keep the already displayed file.
    pub fn cancel(&mut self) -> Result<()> {
        // checked_add refuses identity reuse rather than wrapping a u64 back to an old generation.
        self.generation = self
            .generation
            .checked_add(1)
            .context("File-open request identity exhausted; restart the application")?;
        self.pending = None;
        tracing::debug!(
            generation = self.generation,
            "invalidated pending file open"
        );
        return Ok(());
    }

    /// Replace the desired target without accumulating reads while a previous request is executing.
    pub fn request(&mut self, path: PathBuf) -> Result<()> {
        self.cancel()?;
        tracing::debug!(path = %path.display(), generation = self.generation, "requested project source open");
        // Some retains the latest native path until its bounded reader slot becomes available.
        self.pending = Some(path);
        self.pending_outside = false;
        return Ok(());
    }

    /// What:
    ///  Request a file outside the project,
    ///  read without project resolution.
    /// Why:
    ///  A language server can name a standard-library or dependency file as a definition.
    ///      The caller passes only a canonical path the Language module validated as an existing
    ///      regular file outside the root;
    ///  tree and search opens never come here.
    pub fn request_outside(&mut self, path: PathBuf) -> Result<()> {
        self.cancel()?;
        tracing::debug!(path = %path.display(), generation = self.generation, "requested outside-project source open");
        self.pending = Some(path);
        self.pending_outside = true;
        return Ok(());
    }

    /// True includes waiting work and unread replies,
    ///  including a cancelled read still finishing in the background.
    pub fn has_pending(&self) -> bool {
        return self.pending.is_some() || self.worker.is_busy();
    }

    /// Return only the latest successful open;
    ///  current errors propagate while stale failures are logged and discarded.
    pub fn poll(&mut self) -> Result<Option<OpenedFile>> {
        // A stopped transport must release the waiting target as well as the worker's own busy slot.
        let completed = match self.worker.try_take() {
            Ok(reply) => reply,
            Err(error) => {
                self.pending = None;
                return Err(error);
            }
        };
        // Lend no mutable document state to the worker while consuming its completed reply.
        if let Some(reply) = completed {
            if reply.generation == self.generation {
                return Ok(Some(opened(reply, self.submitted_outside)?));
            }
            if let Err(error) = reply.result {
                tracing::debug!(%error, generation = reply.generation, "discarded stale file-open failure");
            } else {
                tracing::debug!(
                    generation = reply.generation,
                    "discarded stale file-open result"
                );
            }
        }
        if self.worker.is_busy() {
            return Ok(None);
        }
        // Borrow the desired path until the reader accepts its owned request.
        let Some(path) = &self.pending else {
            return Ok(None);
        };
        // What: clone copies only path/boundary metadata; the empty document is an immutable initial base.
        // Why: The prior displayed document must not influence another file's initial caret or viewport.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // submit({ workspace, path, snapshot: new Document(''), generation, highlightUnchanged: true });
        // ```
        let request = ReloadRequest {
            path: path.clone(),
            snapshot: Document::new(""),
            generation: self.generation,
            highlight_unchanged: true,
            // Opening shows what is on disk now, as it always has.
            require_quiet: None,
        };
        // An outside target is not resolved against the project, which would refuse it.
        let submitted = if self.pending_outside {
            self.worker.request(request)?
        } else {
            self.worker
                .request_project(self.workspace.clone(), request)?
        };
        if !submitted {
            bail!("File-open reader rejected an idle request; restart the application");
        }
        self.pending = None;
        self.submitted_outside = self.pending_outside;
        return Ok(None);
    }
}
