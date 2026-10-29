//! Resolve each selected search directory inside the fixed project before starting child processes.

/// The worker owns request identity and immutable reply publication.
use super::{Request, SearchReply};
/// Search scopes do not create another project;
///  they narrow the existing read-only boundary.
use crate::{search::SearchResults, search_process, workspace::Workspace};
/// Resolution and directory-kind failures retain their affected input.
use anyhow::{Result, bail};
/// Native scope paths retain byte identity across relative and absolute input forms.
use std::{fs, path::PathBuf};

/// Validate a selected directory on the background thread,
///  before ripgrep can enumerate it.
fn scope(workspace: &Workspace, request: &Request) -> Result<PathBuf> {
    let path = if let Some(selected) = &request.scope {
        workspace.resolve(selected)?
    } else {
        workspace.root().to_path_buf()
    };
    let metadata = match fs::metadata(&path) {
        Ok(found) => found,
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "Cannot inspect search directory {}",
                path.display()
            )));
        }
    };
    if !metadata.is_dir() {
        bail!("Cannot search {}: it is not a directory", path.display());
    }
    return Ok(path);
}

/// A scope failure belongs to the whole query;
///  both streams report the same operation-specific diagnostic.
pub(super) async fn run(workspace: &Workspace, request: &Request) -> Option<SearchReply> {
    if request.cancellation.is_cancelled() {
        return None;
    }
    let (root, results) = match scope(workspace, request) {
        Ok(root) => {
            // What: `?` on Option returns None when cancellation produced no result.
            // Why: Cancelled work must not be published as an empty successful search.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const results = await search(root, request);
            // if (results === undefined) return undefined;
            // ```
            let results =
                search_process::search(&root, &request.query, &request.cancellation).await?;
            (root, results)
        }
        Err(error) => {
            tracing::warn!(%error, query = request.query, "project search scope is unavailable");
            let root = if let Some(selected) = &request.scope {
                workspace.root().join(selected)
            } else {
                workspace.root().to_path_buf()
            };
            let diagnostic = format!("{error:#}");
            (
                root,
                SearchResults {
                    paths: Err(anyhow::anyhow!(diagnostic.clone())),
                    contents: Err(anyhow::anyhow!(diagnostic)),
                },
            )
        }
    };
    if request.cancellation.is_cancelled() {
        return None;
    }
    return Some(SearchReply {
        generation: request.generation,
        query: request.query.clone(),
        scope: root,
        results,
    });
}
