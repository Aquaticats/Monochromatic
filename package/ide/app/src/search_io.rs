//! Bounded search-protocol records and stderr capture keep child output from growing the UI's memory indefinitely.

/// Search record limits are shared with user-facing stream diagnostics.
use crate::search::MAX_SEARCH_RECORD;
/// I/O failures retain their context instead of becoming an empty result list.
use anyhow::{Context, Result, bail};
/// Tokio's buffered reads can be cancelled while retaining already consumed bytes.
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt};

/// Bound diagnostic storage while continuing to drain the pipe so a child cannot block behind stderr.
const MAX_DIAGNOSTIC_BYTES: usize = 64 * 1024;

/// Read one NUL-delimited filename or newline-delimited JSON record with a hard allocation bound.
pub(crate) async fn record<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    delimiter: u8,
) -> Result<Option<Vec<u8>>> {
    // What: Vec owns one variable-length bounded record; take wraps the borrowed reader with a byte ceiling.
    // Why: Checking length only after an unlimited read_until would already have allocated the oversized record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const record = await readDelimitedWithByteLimit(reader, delimiter, MAX_SEARCH_RECORD + 2);
    // ```
    let mut bytes = Vec::new();
    let mut bounded = reader.take((MAX_SEARCH_RECORD + 2) as u64);
    let count = bounded
        .read_until(delimiter, &mut bytes)
        .await
        .context("Cannot read ripgrep output")?;
    if count == 0 {
        return Ok(None);
    }
    if bytes.last() == Some(&delimiter) {
        bytes.pop();
    }
    if bytes.len() > MAX_SEARCH_RECORD {
        bail!(
            "One ripgrep output record exceeds the {MAX_SEARCH_RECORD}-byte search limit. Filename results may still be available; open the file from the tree or narrow the query to avoid this matching line"
        );
    }
    return Ok(Some(bytes));
}

/// Consume all stderr but retain only a bounded diagnostic prefix,
///  including an explicit truncation marker.
pub(crate) async fn diagnostic<R: AsyncRead + Unpin>(mut reader: R) -> Result<String> {
    let mut retained = Vec::new();
    let mut truncated = false;
    // A fixed stack buffer avoids allocating another record for each diagnostic fragment.
    let mut chunk = [0; 4096];
    loop {
        let count = reader
            .read(&mut chunk)
            .await
            .context("Cannot read ripgrep diagnostic output")?;
        if count == 0 {
            break;
        }
        let accepted = count.min(MAX_DIAGNOSTIC_BYTES - retained.len());
        retained.extend_from_slice(&chunk[..accepted]);
        truncated = truncated || accepted < count;
    }
    // Invalid diagnostic bytes are display text only, never file identities or source positions.
    let mut text = String::from_utf8_lossy(&retained).into_owned();
    if truncated {
        text.push_str("\nRipgrep diagnostic output was truncated after 65536 bytes.");
    }
    return Ok(text);
}

/// In-memory streams verify the same byte limits and EOF behavior as real child pipes.
#[cfg(test)]
#[path = "search_io_tests.rs"]
mod tests;
