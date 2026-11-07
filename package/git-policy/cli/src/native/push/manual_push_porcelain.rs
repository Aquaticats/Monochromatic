//! What:
//!  The parser of `git push --dry-run --porcelain` output,
//! which tells the manual-push probe what a push would send.
//! Why:
//!  The probe used to learn the update lines from a recording hook written as shell text.
//! Git's own dry-run report
//!  carries the same facts without any program of ours:
//! for each destination a `To <url>` line,
//! then one tab-separated
//!  line per ref (`<flag>\t<from>:<to>\t<summary>`),
//! then `Done`.
//! With `core.abbrev=no` the summary of an update holds
//!  the full old and new object names (`old..new`,
//! or `old...new` when forced);
//! a new ref,
//! a deletion and a refusal
//!  hold a bracketed word instead.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const blocks = parsePorcelain(stdout); // [{ shownLocation, updates: [{ from, to, before, after }] }]
//! ```

/// What the remote held for a ref before the push,
/// as far as the report says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoteBefore = { kind: 'absent' } | { kind: 'known'; oid: string } | { kind: 'unknown' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteBefore {
    /// A new branch or tag:
    /// the remote has no such ref.
    Absent,
    /// An update:
    /// the remote held this object name.
    Known(String),
    /// A deletion or a refusal:
    /// the report does not say,
    /// so the confirmation asks the remote.
    Unknown,
}

/// One reported ref update.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PorcelainUpdate = { from: string; to: string; before: RemoteBefore; after?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PorcelainUpdate {
    /// The local ref or expression pushed from;
    /// empty for a deletion.
    pub from: String,
    /// The remote ref updated.
    pub to: String,
    /// What the remote held before.
    pub before: RemoteBefore,
    /// The object the update sends,
    /// when the report names it.
    pub after: Option<String>,
}

/// The updates reported for one destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PorcelainBlock = { updates: PorcelainUpdate[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PorcelainBlock {
    /// The location Git printed after `To`,
    /// with any credentials removed.
    pub shown: String,
    /// The reported updates of this destination,
    /// up-to-date refs left out.
    pub updates: Vec<PorcelainUpdate>,
}

/// The start of the line Git adds to the report when the push would set an upstream.
const SET_UPSTREAM_PREFIX: &str = "Would set upstream of ";

/// What:
///  Split an update's summary into what the remote held and what is sent.
/// Why:
///   `old..new` and `old...new` name both;
/// `[new branch]` and `[new tag]` mean the remote has nothing;
/// any other
///  bracketed word (a deletion,
/// a refusal) leaves the remote's value to the confirmation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseSummary(summary: string): { before: RemoteBefore; after?: string };
/// ```
fn parse_summary(summary: &str) -> (RemoteBefore, Option<String>) {
    let token: &str = summary.split(' ').next().unwrap_or_default();
    if token == "[new" {
        return (RemoteBefore::Absent, None);
    }
    let range: Option<(&str, &str)> = match token.split_once("...") {
        Some(halves) => Some(halves),
        None => token.split_once(".."),
    };
    return match range {
        Some((old, new)) if !old.is_empty() && !new.is_empty() => {
            (RemoteBefore::Known(old.to_owned()), Some(new.to_owned()))
        }
        Some(_) | None => (RemoteBefore::Unknown, None),
    };
}

/// What:
///  Parse the standard output of a porcelain dry-run push into one block per destination.
///  An error names the line that is not part of the format.
/// Why:
///   An unrecognised line is a probe failure,
/// never a guess about what would be pushed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePorcelain(output: string): PorcelainBlock[]; // throws
/// ```
pub fn parse_porcelain(output: &str) -> Result<Vec<PorcelainBlock>, String> {
    let mut blocks: Vec<PorcelainBlock> = Vec::new();
    for line in output.lines() {
        if let Some(shown) = line.strip_prefix("To ") {
            blocks.push(PorcelainBlock { shown: shown.to_owned(), updates: Vec::new() });
            continue;
        }
        // `-u`, `--set-upstream` and `push.autoSetupRemote` make Git add this line to the report (measured on 2.56.0).
        if line == "Done" || line.is_empty() || line.starts_with(SET_UPSTREAM_PREFIX) {
            continue;
        }
        let mut fields = line.splitn(3, '\t');
        let (Some(flag), Some(spec), summary) = (fields.next(), fields.next(), fields.next().unwrap_or_default()) else {
            return Err(format!("Malformed porcelain line: {line}"));
        };
        // Git splits a refspec at its last colon, so a source such as `:/message` keeps its own.
        let Some((from, to)) = spec.rsplit_once(':') else {
            return Err(format!("Malformed porcelain ref pair: {line}"));
        };
        let Some(block) = blocks.last_mut() else {
            return Err(format!("Porcelain line before any destination: {line}"));
        };
        if flag == "=" {
            continue;
        }
        let (before, after) = parse_summary(summary);
        block.updates.push(PorcelainUpdate { from: from.to_owned(), to: to.to_owned(), before, after });
    }
    return Ok(blocks);
}

/// The parser is controlled in the test module.
#[cfg(test)]
#[path = "manual_push_porcelain_tests.rs"]
mod tests;
