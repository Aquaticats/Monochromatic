//! What:
//!  Model of MG2's capture rule for a plain staged commit during a merge-install lag, and of MG3's three independent steps over the
//!  versions of one path: the original `O`, the entry `E` recorded at publication, the person's staged version `S`, the worktree file
//!  `W` (whose stat data may match `E`), and the merge result `M`. A version is a small number; a conflict keeps both sides.
//! Why:
//!  Revision 13 captured the staged version whole, so a plain `git commit` of an edit to another line reversed the merge's change on
//!  the lagging lines (the `-a` case was covered, the plain one was not), took a stat match with `E` for "the file is `O`" when `E` and
//!  `O` differ, and handled the overlapping case `S = E != O`, `W != S`, `W != O` in a way that lost a version. Revision 14 replays the
//!  staged edit on `M` by a three-way merge of `O`, `S` and `M`, treats the stat match as evidence about `E` only, and runs MG3 as three
//!  steps (the later-staging test, the preservation of a staged-only version, the three-way file merge). The specification is that no
//!  version the person held is lost: each of `S` and `W` is `O`, the merge result, in the file, in the index, or saved aside.
//!  Question 5 (which side wins for the saved copy) is not decided: both saved copies are kept, so the check holds for either answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const merge3 = (o, w, m) => w === o ? m : m === o ? w : w === m ? w : CONFLICT;
//! ```

/// A conflict cell: it keeps both sides' text, so it represents both `w` and `m`.
pub const CONFLICT: u8 = 9;

/// The three-way merge of one cell: the side that changed from `o` wins, equal changes agree, different changes conflict.
pub fn merge3(o: u8, w: u8, m: u8) -> u8 {
    if w == o {
        return m;
    }
    if m == o || w == m {
        return w;
    }
    return CONFLICT;
}

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 13: the staged version is captured whole; a stat match with `E` stands for "the file is `O`"; `S` is saved only when it
    /// differs from `E`.
    Thirteen,
    /// Revision 14: the staged edit is replayed on `M`; the stat match means `W == E`; `S` is preserved whenever the result holds it nowhere.
    Fourteen,
}

/// The tree a plain commit makes from the staged cell `s` (staged against `o`) while the merge changed the cell to `m`.
pub fn plain_commit(rules: Rules, o: u8, s: u8, m: u8) -> u8 {
    return match rules {
        Rules::Thirteen => s,
        Rules::Fourteen => merge3(o, s, m),
    };
}

/// What MG3 leaves: the file cell, the index cell and the versions saved aside (a bit set over cells 0 to 9).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Installed {
    /// The worktree file's cell.
    pub file: u8,
    /// The index entry's cell.
    pub index: u8,
    /// The saved cells, one bit per value.
    pub saved: u16,
}

/// MG3 over one path. `stat_matches` means the worktree file's stat data equals `E`'s, which lets the wrapper skip reading it.
pub fn install(rules: Rules, o: u8, e: u8, s: u8, w: u8, m: u8, stat_matches: bool) -> Installed {
    // Step one, the later-staging test: an entry that differs from `E` is the person's later staging and is left alone.
    let later_staging: bool = s != e;
    // The cell MG3 believes the file holds.
    let believed_w: u8 = match rules {
        Rules::Thirteen => {
            if stat_matches {
                o
            } else {
                w
            }
        }
        Rules::Fourteen => {
            if stat_matches {
                e
            } else {
                w
            }
        }
    };
    let file: u8 = merge3(o, believed_w, m);
    let index: u8 = if later_staging { s } else { m };
    let mut saved: u16 = 0;
    // Step two, preservation: a staged-only version that the result holds nowhere is saved before the entry is replaced.
    let result_holds_s: bool = s == o || s == m || s == file || s == index;
    match rules {
        Rules::Thirteen => {
            if later_staging && !result_holds_s {
                saved |= 1 << s;
            }
        }
        Rules::Fourteen => {
            if !result_holds_s {
                saved |= 1 << s;
            }
        }
    }
    return Installed { file, index, saved };
}

/// Whether a version `v` the person held is still somewhere: it is the original, the merge result, in the file, in the index, or saved.
/// A conflict in the file keeps the worktree side and the merge side (`worktree_side` says `v` is the worktree version), never the
/// staged version.
pub fn kept(v: u8, worktree_side: bool, o: u8, m: u8, result: Installed) -> bool {
    return v == o || v == m || v == result.file || v == result.index || (worktree_side && result.file == CONFLICT) || result.saved & (1 << v) != 0;
}

/// Every combination of cells (and of the stat match, which forces `W == E`) where MG3 loses a version the person held.
pub fn losses(rules: Rules) -> Vec<(u8, u8, u8, u8, u8, bool)> {
    let mut found: Vec<(u8, u8, u8, u8, u8, bool)> = Vec::new();
    for o in 0..4_u8 {
        for e in 0..4_u8 {
            for s in 0..4_u8 {
                for w in 0..4_u8 {
                    for m in 0..4_u8 {
                        for stat_matches in [false, true] {
                            if stat_matches && w != e {
                                continue;
                            }
                            let result: Installed = install(rules, o, e, s, w, m, stat_matches);
                            if !kept(s, false, o, m, result) || !kept(w, true, o, m, result) {
                                found.push((o, e, s, w, m, stat_matches));
                            }
                        }
                    }
                }
            }
        }
    }
    return found;
}
