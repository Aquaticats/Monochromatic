//! Streaming collectors apply result limits without buffering the complete ripgrep output.

/// Record decoders preserve native paths and distinguish protocol metadata from matches.
use crate::{
    search::{MAX_CONTENT_RESULTS, MAX_PATH_RESULTS, SearchHit, SearchKind},
    search_io, search_protocol,
    search_query::PathQuery,
};
/// A malformed record stops its own stream with an actionable error.
use anyhow::{Context, Result};
/// The canonical project root bounds both stream types.
use std::path::Path;
/// Buffered async child pipes feed one bounded record at a time.
use tokio::io::AsyncBufRead;

/// The two read-only ripgrep operations have different record delimiters and matching rules.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Stream {
    /// NUL-delimited filenames are filtered as project-relative literal substrings.
    Paths,
    /// JSON Lines carry smart-case regex matches from file contents.
    Contents,
}

/// Collection ends either at EOF or at the explicit result cap.
pub(crate) struct Collected {
    /// Owned bounded results retain the subprocess's discovery order.
    pub hits: Vec<SearchHit>,
    /// Reaching the result cap requests immediate process termination and reaping.
    pub capped: bool,
}

/// Consume bounded records until EOF or the stream's result cap,
///  without spawning or owning a child process.
pub(crate) async fn collect<R: AsyncBufRead + Unpin>(
    mut reader: R,
    root: &Path,
    query: &str,
    stream: Stream,
) -> Result<Collected> {
    let (delimiter, maximum) = match stream {
        Stream::Paths => (0, MAX_PATH_RESULTS),
        Stream::Contents => (b'\n', MAX_CONTENT_RESULTS),
    };
    let matcher = PathQuery::new(query);
    let mut hits = Vec::new();
    loop {
        // What: Some owns one complete record; None means clean EOF rather than a parsing failure.
        // Why: A final unterminated record is still processed, but corrupt output is never an empty success.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const record = await nextRecord(reader); if (record === undefined) return { hits, capped: false };
        // ```
        let Some(bytes) = search_io::record(&mut reader, delimiter).await? else {
            return Ok(Collected {
                hits,
                capped: false,
            });
        };
        let hit = match stream {
            Stream::Paths => {
                let path = search_protocol::filename(root, &bytes)?;
                let relative = path
                    .strip_prefix(root)
                    .context("Validated search path lost its project prefix")?;
                if matcher.matches(relative) {
                    Some(SearchHit {
                        path,
                        kind: SearchKind::Path,
                    })
                } else {
                    None
                }
            }
            Stream::Contents => search_protocol::content(root, &bytes)?,
        };
        if let Some(found) = hit {
            hits.push(found);
            if hits.len() == maximum {
                return Ok(Collected { hits, capped: true });
            }
        }
    }
}
