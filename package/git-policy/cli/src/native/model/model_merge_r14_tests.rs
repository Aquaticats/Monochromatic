//! What:
//!  The controls of MG2's plain staged commit during the lag, MG2's stat condition and MG3's overlapping cases.
//! Why:
//!  Controls 15, 16 and 17 of the thirteenth review; each is the review's schedule, then the exhaustive comparison over every
//!  combination of the versions, so a loss that the schedule does not name is found too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a plain commit of an edit to another line keeps the merge change', ...);
//! ```

use crate::model_merge_r14::{CONFLICT, Rules, install, losses, merge3, plain_commit};

/// Control 15: `O: a=0 b=0`, the merge made `M: a=1`, the file lags, the person edits and stages `b=1`, a plain commit. Revision 13
/// captures the staged version whole and reverses `a` (red); revision 14 replays the staged edit on `M`, keeping both.
#[test]
fn a_plain_commit_of_an_old_based_edit_keeps_the_merge_change() {
    let (o, s, m): ((u8, u8), (u8, u8), (u8, u8)) = ((0, 0), (0, 1), (1, 0));
    let thirteen: (u8, u8) = (plain_commit(Rules::Thirteen, o.0, s.0, m.0), plain_commit(Rules::Thirteen, o.1, s.1, m.1));
    assert_eq!(thirteen, (0, 1), "line a is reversed");
    let fourteen: (u8, u8) = (plain_commit(Rules::Fourteen, o.0, s.0, m.0), plain_commit(Rules::Fourteen, o.1, s.1, m.1));
    assert_eq!(fourteen, (1, 1));
}

/// Control 16: `E` names `B`, `O` names `A`, and the file's stat data matches `E`. Revision 13 reads the stat match as "the file is `O`"
/// and overwrites the file with the merge, losing `B` (red); revision 14 reads it as "the file is `E`".
#[test]
fn a_stat_match_with_e_is_not_a_proof_that_the_file_is_o() {
    let (o, e, m): (u8, u8, u8) = (0, 1, 2);
    let thirteen = install(Rules::Thirteen, o, e, e, e, m, true);
    assert_eq!(thirteen.file, m, "the file B was replaced by the merge");
    assert!(losses(Rules::Thirteen).iter().any(|entry| { return entry.5 && entry.0 == o && entry.1 == e; }));
    let fourteen = install(Rules::Fourteen, o, e, e, e, m, true);
    assert_eq!(fourteen.file, merge3(o, e, m));
}

/// Control 17: `S = E != O`, `W != S`, `W != O`. Revision 13 saves `S` only when it differs from `E`, replaces the entry with `M`, and
/// merges the file: `S` is nowhere (red). Revision 14 preserves it first. Over every combination, revision 14 loses nothing.
#[test]
fn overlapping_staged_and_worktree_versions_are_all_kept() {
    let (o, e, w, m): (u8, u8, u8, u8) = (0, 1, 2, 3);
    let s: u8 = e;
    let thirteen = install(Rules::Thirteen, o, e, s, w, m, false);
    assert!(thirteen.saved & (1 << s) == 0 && thirteen.index == m && thirteen.file != s, "S is in neither the index, the file nor a saved copy");
    assert!(losses(Rules::Thirteen).contains(&(o, e, s, w, m, false)));
    let fourteen = install(Rules::Fourteen, o, e, s, w, m, false);
    assert!(fourteen.saved & (1 << s) != 0);
    assert!(losses(Rules::Fourteen).is_empty(), "{:?}", losses(Rules::Fourteen).first());
    assert_eq!(merge3(1, 2, 3), CONFLICT);
}
