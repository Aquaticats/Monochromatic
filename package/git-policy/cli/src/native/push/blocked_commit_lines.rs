//! What:
//!  The lines a command prints about a branch the automatic push stops short of,
//! and the words of a verdict that has none.
//! Why:
//!  The line is the only thing the person reads about a backup that did not happen,
//! so it names the branch,
//! the commit,
//! the check that refused it,
//! and the way forward:
//! their own push sends it,
//! and the automatic push resumes once the commit is off the branch's way out.
//!  A verdict an earlier release wrote holds no words,
//! so it is named with the sentence this release has for that case rather than with an empty clause.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = heldBackLine(branchRef, stopped); const unsure = unansweredLine(branchRef, stopped);
//! ```

use crate::push_held_back_by_a_blocked_commit::{Held, Stopped};


/// What:
///  The reason a verdict with no words of its own is named with.
/// Why:
///   A record an older release wrote holds the word `blocked` alone,
/// and the line must still say why the commit is
///  not published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNRECORDED = 'its post-commit check blocked the automatic backup';
/// ```
pub const UNRECORDED_REASON: &str = "its post-commit check blocked the automatic backup";

/// What:
///  The words a person is given for one verdict.
/// Why:
///   A record with no words of its own is still a refusal,
/// and an empty clause would read as no reason at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namedReason = (reason) => reason.trim() === '' ? UNRECORDED_REASON : reason.trim();
/// ```
pub fn named_reason(reason: &str) -> String {
    let named: &str = reason.trim();
    return match named.is_empty() {
        true => String::from(UNRECORDED_REASON),
        false => named.to_owned(),
    };
}

/// What:
///  The words of one held verdict,
/// for the callers that name only the first.
/// Why:
///   A closure that captures nothing has a named function's shape,
/// and the package's linter asks for the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reasonOf = (held) => held.reason;
/// ```
fn reason_of(held: &Held) -> &str {
    return held.reason.as_str();
}

/// What:
///  The branch a line names,
/// as the person reads it,
/// or the reference itself when it is not a branch.
/// Why:
///   The owner's own wording names `main`,
/// and a reference outside `refs/heads` has no short name to give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shortName = (ref) => ref.startsWith('refs/heads/') ? ref.slice(11) : ref;
/// ```
pub fn branch_name(branch_ref: &str) -> &str {
    return branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
}

/// What:
///  The one line a command prints about a branch the automatic push stops short of.
/// Why:
///   The person must know which commit holds the branch back,
/// what its own check found,
/// and that nothing was lost:
///  the commit is local,
/// a person's own `git push` still sends it,
/// and the push resumes by itself once the commit is off the
///  branch's way out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (branchRef, stopped) => `${name(branchRef)} is held back by the commit ${stopped.commits[0].commit}, because ${reason}. …`;
/// ```
pub fn held_back_line(branch_ref: &str, stopped: &Stopped) -> String {
    let name: &str = branch_name(branch_ref);
    let [only] = stopped.commits.as_slice() else {
        let mut listed: Vec<String> = Vec::new();
        for held in &stopped.commits {
            listed.push(format!("{} ({})", held.commit, held.reason));
        }
        return format!(
            "{name} is held back by these commits, because their own post-commit checks kept them local: {}. The automatic push publishes nothing past them, a person's `git push` still sends them, and the push resumes once they are off {name}'s way out.",
            listed.join(", ")
        );
    };
    return format!(
        "{name} is held back by the commit {}, because {}. The automatic push stops before it, a person's `git push` still sends it, and the push resumes once it is off {name}'s way out.",
        only.commit, only.reason
    );
}

/// What:
///  The line a command prints when Git could not say whether a verdict is still on the branch's way out.
/// Why:
///   Sending nothing is the safe answer,
/// and the person is told why nothing was sent and that the remedy is their own push;
/// a line
///  that named a verdict as holding the branch back would claim more than Git answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unsureLine = (branchRef, stopped) => `${name(branchRef)} may be held back by ${commits}, because ${reason}, and Git could not say whether it is still on the branch's way out; the automatic push sends nothing until it can, and a person's \`git push\` still sends the branch.`;
/// ```
pub fn unanswered_line(branch_ref: &str, stopped: &Stopped) -> String {
    let name: &str = branch_name(branch_ref);
    let mut named: Vec<String> = Vec::new();
    for held in &stopped.commits {
        named.push(held.commit.clone());
    }
    return format!(
        "{name} may be held back by {}, because {}, and Git could not say whether it is still on {name}'s way out; the automatic push sends nothing until it can, and a person's `git push` still sends the branch.",
        named.join(", "),
        named_reason(stopped.commits.first().map_or("", reason_of))
    );
}
