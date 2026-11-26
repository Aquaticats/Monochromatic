//! What:
//!  The one line reader both halves of a plan's parse share:
//!  one line of a region,
//!  without its newline,
//!  and the region after it.
//! Why:
//!  A record that stops before a line it declares is a refusal at every line,
//!  and one reader means the header and the attempt lines refuse an unfinished record the same way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [line, rest] = takeLine(region, 'ends-early');
//! ```

use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};

/// One line of the region,
/// without its newline,
/// and the region after it.
pub fn take_line<'a>(
    region: &mut &'a str,
    cause: PlanRefusalCause,
) -> Result<&'a str, PlanRefusal> {
    let Some(end) = region.find('\n') else {
        return Err(PlanRefusal::of(cause, *region));
    };
    let line: &'a str = &region[..end];
    *region = &region[end + 1..];
    return Ok(line);
}
