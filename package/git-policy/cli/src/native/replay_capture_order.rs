//! What:
//!  Capture order for one replay:
//!       the landed first-parent changes, the per-path decision between the prepared and the landed captures,
//!       and the parser of the `git diff-tree --stdin` output that lists the changes.
//! Why:
//!  When the prepared commit and every landed commit that changed a path captured it from the same worktree,
//!       the later capture's bytes land, which is what sequential native commits record from the shared disk.
//!       Every other path keeps subsumption and then the three-way merge.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! decideCaptureOrder({ path, captured, history, landed }); // 'prepared' | 'landed' | 'unordered'
//! ```

/// Ordered and keyed collections of paths and captures.
use std::collections::{BTreeMap, BTreeSet};
/// Error type of the parsers.
use std::fmt;

/// What:
///  One capture with the paths its worktree read, Latin-1 decoded.
/// Why:
///   Captures are compared by worktree identity and sequence number only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CaptureView = { worktreeId: string; sequence: number; paths: ReadonlySet<string> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureView {
    /// Identity of the worktree that captured.
    pub worktree_id: String,
    /// Capture sequence number.
    pub sequence: i64,
    /// Paths whose bytes the capture read from the worktree.
    pub paths: BTreeSet<String>,
}

/// What:
///  One landed commit and the paths it changed against its first parent.
/// Why:
///   The decision needs each landed commit's changed paths to know which captures to consult.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandedChange = { commit: string; paths: ReadonlySet<string> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandedChange {
    /// Commit ID.
    pub commit: String,
    /// Changed paths, Latin-1 decoded.
    pub paths: BTreeSet<String>,
}

/// What:
///  The capture-order decision for one shared path.
/// Why:
///   Replay acts on the three answers differently: the prepared entry lands, the landed entry stays,
///       or subsumption and the merge decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CaptureOrderDecision = 'prepared' | 'landed' | 'unordered';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureOrderDecision {
    /// The prepared commit captured later: its entry lands.
    Prepared,
    /// A landed commit captured later: the landed entry stays.
    Landed,
    /// Capture order does not apply: subsumption, then the three-way merge.
    Unordered,
}

/// What:
///  Diff-tree output that does not have the shape `git diff-tree --stdin -r -z --raw` writes.
/// Why:
///   A raw record without a commit before it cannot be attributed, so the replay fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new TypeError('git diff-tree printed a raw record without a commit or a path.');
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureOrderFormatError(pub String);

impl fmt::Display for CaptureOrderFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.0);
    }
}

/// What:
///  Builds a capture view from a capture's stamp and its recorded worktree paths.
/// Why:
///   Records and the transaction's own capture share one comparison shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// captureView({ stamp, worktreePaths });
/// ```
pub fn capture_view(worktree_id: &str, sequence: i64, worktree_paths: &[String]) -> CaptureView {
    return CaptureView {
        worktree_id: String::from(worktree_id),
        sequence,
        paths: worktree_paths.iter().cloned().collect(),
    };
}

/// What:
///  Parses `git diff-tree --stdin -r -z --raw` output:
///       each commit ID NUL-terminated, then its raw records, each a `:` header and a path, both NUL-terminated.
/// Why:
///   One process lists every landed commit's changed paths, so the parser keeps commits in output order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseCommitChanges('abc\0:100644 100644 1 2 M\0a.txt\0'); // [{ commit: 'abc', paths: Set(['a.txt']) }]
/// ```
pub fn parse_commit_changes(output: &str) -> Result<Vec<LandedChange>, CaptureOrderFormatError> {
    // `mut` lets records be attached to the most recent commit as they are read.
    let mut commits: Vec<LandedChange> = Vec::new();
    let tokens: Vec<&str> = output.split('\0').collect();
    let mut cursor: usize = 0;
    while cursor < tokens.len() {
        let token: &str = tokens[cursor];
        cursor += 1;
        if token.starts_with(':') {
            let path: &str = match tokens.get(cursor) {
                Some(found) => found,
                None => {
                    return Err(CaptureOrderFormatError(String::from(
                        "git diff-tree printed a raw record without a commit or a path.",
                    )));
                }
            };
            cursor += 1;
            let owner: &mut LandedChange = match commits.last_mut() {
                Some(found) => found,
                None => {
                    return Err(CaptureOrderFormatError(String::from(
                        "git diff-tree printed a raw record without a commit or a path.",
                    )));
                }
            };
            owner.paths.insert(String::from(path));
        } else if !token.is_empty() {
            commits.push(LandedChange {
                commit: String::from(token),
                paths: BTreeSet::new(),
            });
        }
    }
    return Ok(commits);
}

/// What:
///  Decides one shared path from the prepared capture, the landed history, and the landed captures.
/// Why:
///   The prepared capture must have read the path from the worktree, and every landed commit that changed
///       the path must have a capture from the same worktree that also read it;
///       the later sequence number then decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideCaptureOrder({ path, captured, history, landed });
/// ```
pub fn decide_capture_order(
    path: &str,
    captured: &CaptureView,
    history: &[LandedChange],
    landed: &BTreeMap<String, CaptureView>,
) -> CaptureOrderDecision {
    if !captured.paths.contains(path) {
        return CaptureOrderDecision::Unordered;
    }
    // Landed commits that changed the path.
    let touching: Vec<&LandedChange> = history
        .iter()
        .filter(|change| { return change.paths.contains(path); })
        .collect();
    // Their captures of the path from this worktree; a commit without one leaves the list shorter.
    let ordered: Vec<&CaptureView> = touching
        .iter()
        .filter_map(|change| { return landed.get(&change.commit); })
        .filter(|record| { return record.worktree_id == captured.worktree_id && record.paths.contains(path); })
        .collect();
    if touching.is_empty() || ordered.len() != touching.len() {
        return CaptureOrderDecision::Unordered;
    }
    let latest: i64 = ordered.iter().map(|record| { return record.sequence; }).max().unwrap_or(i64::MIN);
    if captured.sequence > latest {
        return CaptureOrderDecision::Prepared;
    }
    return CaptureOrderDecision::Landed;
}

#[cfg(test)]
#[path = "replay_capture_order_tests.rs"]
mod tests;
