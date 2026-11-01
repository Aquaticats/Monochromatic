//! Read-only ripgrep commands stream concurrently and are explicitly reaped on limits,
//!  cancellation,
//!  and failures.

/// Independent stream outcomes retain useful filename matches when content search fails.
use crate::{
    search::{SearchHit, SearchResults},
    search_cancel::SearchCancellation,
    search_collect::{self, Stream},
    search_io,
};
/// Diagnostics identify both the query and the failed subprocess operation.
use anyhow::{Context, Result, bail};
/// Native project paths and stdio configuration never pass through shell interpolation.
use std::{
    path::Path,
    process::{ExitStatus, Stdio},
};
/// Child pipes and process lifetime use Tokio's cancellable I/O rather than blocking the native event loop.
use tokio::{
    io::BufReader,
    process::{Child, Command},
};

/// What:
///  The ripgrep settings every listing of the project shares:
///  no configuration file,
///  no preprocessor,
///       no decompression,
///  one thread,
///  line-buffered output.
///  `[&str; 6]` is a fixed-length array of six texts.
/// Why:
///  The search and the language servers' folder watching list the project with the same ignore rules
///      (`.gitignore`,
///  `.ignore`,
///  `.rgignore`,
///  hidden names),
///  so both agree on what belongs to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RIPGREP_SETTINGS = ['--no-config', '--no-pre', '--no-search-zip', '--threads', '1', '--line-buffered'] as const;
/// ```
pub(crate) const RIPGREP_SETTINGS: [&str; 6] = [
    "--no-config",
    "--no-pre",
    "--no-search-zip",
    "--threads",
    "1",
    "--line-buffered",
];

/// Construct only the required search operation,
///  excluding inherited preprocessing or decompression commands.
fn command(root: &Path, query: &str, stream: Stream) -> Command {
    let mut command = Command::new("rg");
    command.current_dir(root);
    command.env_remove("RIPGREP_CONFIG_PATH");
    command.args(RIPGREP_SETTINGS);
    match stream {
        Stream::Paths => {
            command.args(["--files", "--null", "--"]);
        }
        Stream::Contents => {
            command.args(["--json", "--smart-case", "--max-count", "1", "--", query]);
        }
    }
    command.arg(root);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    // kill_on_drop is a final safeguard; normal completion always explicitly waits for the child.
    command.kill_on_drop(true);
    return command;
}

/// Stop an owned process and reap it before returning a capped or cancelled result.
async fn stop(child: &mut Child) -> Result<ExitStatus> {
    if let Some(status) = child
        .try_wait()
        .context("Cannot inspect ripgrep completion")?
    {
        return Ok(status);
    }
    child.kill().await.context("Cannot stop and reap ripgrep")?;
    // Tokio caches a completed child's status, so this returns the status reaped by kill.
    return child
        .wait()
        .await
        .context("Cannot obtain stopped ripgrep status");
}

/// The prepared command seam lets tests exercise pipe/cancellation behavior without a shell script.
async fn execute(
    mut command: Command,
    root: &Path,
    query: &str,
    stream: Stream,
    cancellation: &SearchCancellation,
) -> Result<Option<Vec<SearchHit>>> {
    let signal = cancellation.subscribe();
    if cancellation.is_cancelled() {
        return Ok(None);
    }
    tracing::debug!(?stream, query, root = %root.display(), "starting ripgrep search stream");
    let child = match command.spawn() {
        Ok(started) => started,
        Err(error) => {
            return Err(anyhow::Error::new(error).context(format!(
                "Cannot start ripgrep for query {query:?} in {}. Install ripgrep and ensure rg is available on PATH",
                root.display()
            )));
        }
    };
    return consume(child, root, query, stream, cancellation, signal).await;
}

/// Own and reap an already spawned child;
///  tests can observe its PID without introducing a production debug callback.
async fn consume(
    mut child: Child,
    root: &Path,
    query: &str,
    stream: Stream,
    cancellation: &SearchCancellation,
    mut signal: tokio::sync::watch::Receiver<bool>,
) -> Result<Option<Vec<SearchHit>>> {
    let stdout = child
        .stdout
        .take()
        .context("Ripgrep stdout pipe was not created")?;
    let stderr_pipe = child
        .stderr
        .take()
        .context("Ripgrep stderr pipe was not created")?;
    // What: spawn drains stderr concurrently while select waits for stdout or cancellation.
    // Why: A full stderr pipe must not deadlock a child whose stdout is being parsed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const diagnostics = drainBounded(child.stderr);
    // const output = await race(collect(child.stdout), signal.aborted);
    // ```
    let diagnostics = tokio::spawn(search_io::diagnostic(stderr_pipe));
    let output = tokio::select! {
        result = search_collect::collect(BufReader::new(stdout), root, query, stream) => result.map(Some),
        closed = signal.changed() => {
            if let Err(error) = closed { tracing::debug!(%error, "search cancellation channel closed"); }
            Ok(None)
        }
    };
    let early = match &output {
        Ok(Some(collected)) => collected.capped,
        Ok(None) | Err(_) => true,
    };
    let waited_status = if early {
        stop(&mut child).await
    } else {
        // Drop the waiting borrow before issuing a kill after cancellation.
        let waited = tokio::select! {
            status = child.wait() => Some(status),
            closed = signal.changed() => {
                if let Err(error) = closed { tracing::debug!(%error, "search cancellation channel closed during wait"); }
                None
            }
        };
        if let Some(status) = waited {
            status.context("Cannot wait for ripgrep completion")
        } else {
            stop(&mut child).await
        }
    };
    let status = match waited_status {
        Ok(status) => status,
        Err(error) => {
            diagnostics.abort();
            if let Err(join_error) = diagnostics.await {
                tracing::debug!(%join_error, "stopped diagnostic drain after process cleanup failure");
            }
            return Err(error);
        }
    };
    let stderr = diagnostics
        .await
        .context("Ripgrep diagnostic reader stopped unexpectedly")??;
    if cancellation.is_cancelled() {
        tracing::debug!(
            ?stream,
            "discarding cancelled search stream after process cleanup"
        );
        return Ok(None);
    }
    let read = match output {
        Ok(found) => found,
        Err(error) => {
            return Err(error.context(format!(
                "Cannot read {stream:?} search results for query {query:?}"
            )));
        }
    };
    let Some(collected) = read else {
        return Ok(None);
    };
    if !collected.capped && !status.success() && status.code() != Some(1) {
        bail!(
            "Ripgrep {stream:?} search for query {query:?} in {} exited with {status}: {}",
            root.display(),
            stderr.trim()
        );
    }
    if !stderr.is_empty() {
        tracing::debug!(?stream, diagnostic = stderr, "ripgrep diagnostic output");
    }
    tracing::debug!(
        ?stream,
        results = collected.hits.len(),
        capped = collected.capped,
        "completed ripgrep search stream"
    );
    return Ok(Some(collected.hits));
}

/// Real child pipes exercise cancellation,
///  reaping,
///  diagnostics,
///  and inherited-config exclusion.
#[cfg(test)]
#[path = "search_process_tests.rs"]
mod tests;

/// Execute both bounded streams concurrently;
///  a cancelled query never becomes an empty successful reply.
pub(crate) async fn search(
    root: &Path,
    query: &str,
    cancellation: &SearchCancellation,
) -> Option<SearchResults> {
    let (path_output, content_output) = tokio::join!(
        execute(
            command(root, query, Stream::Paths),
            root,
            query,
            Stream::Paths,
            cancellation
        ),
        execute(
            command(root, query, Stream::Contents),
            root,
            query,
            Stream::Contents,
            cancellation
        )
    );
    if cancellation.is_cancelled() {
        return None;
    }
    let paths = match path_output {
        Ok(Some(hits)) => Ok(hits),
        Ok(None) => {
            return None;
        }
        Err(error) => Err(error),
    };
    let contents = match content_output {
        Ok(Some(hits)) => Ok(hits),
        Ok(None) => {
            return None;
        }
        Err(error) => Err(error),
    };
    return Some(SearchResults { paths, contents });
}
