//! What:
//!  What a merge that would fast-forward owes the paths it writes,
//!  and whether the whole command is native Git's.
//! Why:
//!  A fast-forward is native Git's own,
//!  handed over at once,
//!  and Git's update writes exactly the paths the incoming commits change.
//!  A named pipe,
//!  a socket or a device standing at one of them holds no bytes a copy can keep,
//!  and Git removes it without a copy,
//!  so the refusal a merge that creates a commit makes has to run before the whole command leaves the wrapper
//!  (`merge_isolation_a_special_file_the_merge_changes`,
//!  `SPEC.md`,
//!  "A named pipe,
//! a socket or a device at a path the merge writes or deletes").
//!  `--ff-only` states on the command line the promise `merge.ff=only` states in the settings,
//!  and a merge either forbids is native Git's own refusal,
//! with nothing written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function whatTheFastForwardOwes({ shape, config, fastForwardable, reference, root, written }): Bail | undefined;
//! ```

use crate::merge_isolation_a_special_file_the_merge_changes::{a_special_file_the_merge_changes, the_line};
use crate::merge_isolation_run_types::{Bail, MergeConfig, refused};
use crate::merge_options::{FastForwardChoice, MergeShape};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The answer a merge that would move the branch by a fast-forward owes:
///  none when the merge is not one,
///  native Git's hand-over when the paths hold none of the entries its update would remove without a copy,
///  and the refusal that names them when they do.
/// Why:
///   One place decides both the refusal and the hand-over,
///  so a fast-forward under `--ff-only` or `merge.ff=only` is asked about those paths
///  exactly as a plain one is,
///  and a merge that only a fast-forward may make but cannot is native Git's refusal rather than a merge made here.
///  `--ff-only` reads as a branch that may move only by a fast-forward,
///  and `--no-ff` as one that may not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wantsCommit = shape.fastForward === 'never' || (shape.fastForward === 'default' && config.fastForward === false);
/// if (!onlyTyped && !onlySet && !(fastForwardable && !wantsCommit)) return undefined;
/// if (!fastForwardable) return { kind: 'forward' };
/// return special.length ? { kind: 'finished', code: 1, stderr: line(reference, special) } : { kind: 'forward' };
/// ```
pub fn what_the_fast_forward_owes(
    shape: &MergeShape,
    config: &MergeConfig,
    fast_forwardable: bool,
    reference: &str,
    root: &Path,
    written: &[Vec<u8>],
) -> Option<Bail> {
    let merge_commit_wanted: bool = match shape.fast_forward {
        FastForwardChoice::Never => true,
        FastForwardChoice::Allowed => false,
        // `--ff-only` never makes a commit: a merge that cannot fast-forward is native Git's own refusal, and the branch
        //  moves only by a fast-forward when it moves here at all.
        FastForwardChoice::Only => false,
        FastForwardChoice::Default => config.fast_forward == Some(false),
    };
    let only_typed: bool = shape.fast_forward == FastForwardChoice::Only;
    let only_set: bool = config.fast_forward_only && shape.fast_forward == FastForwardChoice::Default;
    if !only_typed && !only_set && !(fast_forwardable && !merge_commit_wanted) {
        return None;
    }
    // A merge the option or the setting allows only as a fast-forward, and which cannot fast-forward, is native Git's
    //  own refusal (`fatal: Not possible to fast-forward, aborting.`); nothing is written, so nothing is asked.
    if !fast_forwardable {
        return Some(Bail::Forward(None));
    }
    let special: Vec<Vec<u8>> = a_special_file_the_merge_changes(root, written);
    if special.is_empty() {
        return Some(Bail::Forward(None));
    }
    return Some(refused(1, the_line(reference, special.as_slice())));
}
