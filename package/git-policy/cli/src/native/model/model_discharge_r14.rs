//! What:
//!  Model of rule V's checkpoint cutover and dependency validation at recovery, as a state machine with a power loss and an
//!  independent recovery procedure. The lane writes two requests and their adopt lines (one chain in the old segment), retires the first
//!  with a terminal outcome and removes its request file, writes a checkpoint whose document names the live request, and completes the
//!  checkpoint by removing the old segment. Write-back persists any visible fact in any order; a power loss keeps only what is durable.
//!  `recover` reads only the surviving files (never the lane's history) and chooses a root: the checkpoint when its document's live
//!  dependencies survive, otherwise the old segment's valid chain.
//! Why:
//!  The earlier version of this model made revision 14's acceptance the specification and then compared the two (a tautology), and its
//!  checkpoint rule accepted a checkpoint whatever its live requests (an unsafe rule: a checkpoint replaces log history, not the external
//!  content its document names). Here the properties are checked on crash states of a lane's real schedule: a recovery root exists when
//!  anything durable could be one; every request the recovered root calls live has its content; and a request the lane promised live is
//!  never dropped while its content survives. The unsafe checkpoint rule is a separate variant, red, so that the guard on completion
//!  (the live dependencies are durable first) is shown to be what makes the rule safe. Revision 16 may change the checkpoint rules, so the
//!  rule is a parameter.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const recover = (rules, surviving: number): { root: 'checkpoint' | 'segment' | undefined; live: Set<string> } => ...;
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};
use crate::model_observe::{World, make_visible, power_loss, sync};

/// The request file of R1.
pub const REQ1: u32 = 1;
/// The request file of R2.
pub const REQ2: u32 = 2;
/// R1's terminal outcome line, kind published (its content is gone into the commit; nothing else needs retaining).
pub const OUT1: u32 = 4;
/// R1's adopt line in the old segment.
pub const ADOPT1: u32 = 8;
/// R2's adopt line in the old segment, chained after R1's.
pub const ADOPT2: u32 = 16;
/// The old segment's file (it holds the adopt lines; its own directory entry is part of this fact).
pub const SEG: u32 = 32;
/// The checkpoint file (its document names R2 as live; it also carries its directory entry).
pub const CKPT: u32 = 64;
/// R1's request file was removed (a removal is a fact that persists like any other).
pub const REQ1_GONE: u32 = 128;
/// The old segment was removed.
pub const SEG_GONE: u32 = 256;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 13: an adopt line is valid only against its request file; the checkpoint names its predecessor segment; completion
    /// needs only the checkpoint to be durable.
    Thirteen,
    /// Control: a self-contained checkpoint whose live dependencies are not validated and whose completion needs only the checkpoint.
    UnsafeCheckpoint,
    /// Revision 15: an outcome discharges a retired request's adopt; the checkpoint is self-contained and is a root only if its live
    /// dependencies survive; completion needs the checkpoint and those dependencies durable. Revision 16 may change this rule.
    Fifteen,
}

/// The name of a fact in a step label.
pub fn name_of(fact: u32) -> &'static str {
    return match fact {
        REQ1 => "R1's request file",
        REQ2 => "R2's request file",
        OUT1 => "R1's outcome",
        ADOPT1 => "R1's adopt line",
        ADOPT2 => "R2's adopt line",
        SEG => "the old segment",
        CKPT => "the checkpoint",
        REQ1_GONE => "R1's removal",
        _ => "the old segment's removal",
    };
}

/// The state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Lane {
    /// The rule under test.
    pub rules: Rules,
    /// Visible and durable facts.
    pub world: World,
    /// A power loss happened.
    pub crashed: bool,
}

/// What recovery found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Recovered {
    /// A root was chosen.
    pub root: bool,
    /// R1 is live in the recovered state.
    pub r1_live: bool,
    /// R2 is live in the recovered state.
    pub r2_live: bool,
}

/// The start state: the old segment exists and is durable (it is the recovery root of a lane that has run before).
pub fn initial(rules: Rules) -> Lane {
    return Lane { rules, world: World { visible: SEG, durable: SEG }, crashed: false };
}

fn has(facts: u32, fact: u32) -> bool {
    return facts & fact != 0;
}

/// Recovery over the surviving files only: discover, validate each record against what survives, choose the root, and call live the
/// requests the root names that its rule accepts.
pub fn recover(rules: Rules, surviving: u32) -> Recovered {
    let seg_present: bool = has(surviving, SEG) && !has(surviving, SEG_GONE);
    let req1_present: bool = has(surviving, REQ1) && !has(surviving, REQ1_GONE);
    let req2_present: bool = has(surviving, REQ2);
    let checkpoint_root: bool = match rules {
        Rules::Thirteen => has(surviving, CKPT) && seg_present,
        Rules::UnsafeCheckpoint => has(surviving, CKPT),
        Rules::Fifteen => has(surviving, CKPT) && req2_present,
    };
    if checkpoint_root {
        return Recovered { root: true, r1_live: false, r2_live: true };
    }
    if !seg_present {
        return Recovered { root: false, r1_live: false, r2_live: false };
    }
    // The segment's chain: R1's adopt is valid against its request file, or (revision 15) against its terminal outcome; R2's adopt needs
    // R1's adopt valid or discharged (the hash chain) and its own request file.
    let outcome1: bool = has(surviving, OUT1);
    let adopt1_valid: bool = match rules {
        Rules::Fifteen => has(surviving, ADOPT1) && (req1_present || outcome1),
        Rules::Thirteen | Rules::UnsafeCheckpoint => has(surviving, ADOPT1) && req1_present,
    };
    let chain_ok: bool = adopt1_valid || !has(surviving, ADOPT1) || (rules == Rules::Fifteen && outcome1);
    let adopt2_valid: bool = has(surviving, ADOPT2) && req2_present && chain_ok && (has(surviving, ADOPT1) || rules != Rules::Thirteen);
    let r1_live: bool = adopt1_valid && !outcome1;
    return Recovered { root: true, r1_live, r2_live: adopt2_valid };
}

/// Every step enabled.
pub fn steps(state: &Lane) -> Vec<(String, Lane)> {
    let mut found: Vec<(String, Lane)> = Vec::new();
    if state.crashed {
        return found;
    }
    let visible = state.world.visible;
    let durable = state.world.durable;
    let write = |fact: u32, label: &str, list: &mut Vec<(String, Lane)>| {
        if !has(visible, fact) {
            let mut after: Lane = state.clone();
            after.world = make_visible(state.world, fact);
            push_step(list, label, after);
        }
    };
    write(REQ1, "lane: writes the request file of R1", &mut found);
    write(REQ2, "lane: writes the request file of R2", &mut found);
    if has(visible, REQ1) {
        write(ADOPT1, "lane: appends the adopt line of R1 to the segment", &mut found);
    }
    if has(visible, REQ2) && has(visible, ADOPT1) {
        write(ADOPT2, "lane: appends the adopt line of R2 to the segment", &mut found);
    }
    if has(visible, ADOPT1) {
        write(OUT1, "lane: appends R1's terminal outcome (published)", &mut found);
    }
    // The reclamation barrier: R1's request file is removed only after its outcome is durable (revision 15); revision 13 removed it on a
    // visible outcome.
    let outcome_ok: bool = if state.rules == Rules::Fifteen { has(durable, OUT1) } else { has(visible, OUT1) };
    if outcome_ok && !has(visible, REQ1_GONE) {
        write(REQ1_GONE, "lane: removes R1's request file", &mut found);
    }
    if has(visible, ADOPT2) {
        write(CKPT, "lane: writes the checkpoint whose document names R2 as live", &mut found);
    }
    // Completion removes the old segment: it needs the checkpoint durable and, revision 15, the live dependency durable first.
    let completes: bool = match state.rules {
        Rules::Fifteen => has(durable, CKPT) && has(durable, REQ2),
        Rules::Thirteen | Rules::UnsafeCheckpoint => has(durable, CKPT),
    };
    if completes && !has(visible, SEG_GONE) {
        write(SEG_GONE, "lane: completes the checkpoint and removes the old segment", &mut found);
    }
    let mut bit: u32 = 1;
    while bit <= SEG_GONE {
        if has(visible, bit) && !has(durable, bit) {
            let mut forced: Lane = state.clone();
            forced.world = sync(state.world, bit);
            push_step(&mut found, &format!("lane: forces {}", name_of(bit)), forced.clone());
            push_step(&mut found, &format!("environment: write-back persists {}", name_of(bit)), forced);
        }
        bit <<= 1;
    }
    let mut lost: Lane = state.clone();
    lost.world = power_loss(state.world);
    lost.crashed = true;
    push_step(&mut found, "environment: power is lost", lost);
    return found;
}

/// The properties of a crash state, judged on what recovery returns from the surviving files.
pub fn invariant(state: &Lane) -> Option<String> {
    if !state.crashed {
        return None;
    }
    let surviving: u32 = state.world.visible;
    let found: Recovered = recover(state.rules, surviving);
    let something_to_recover_from: bool = has(surviving, CKPT) || (has(surviving, SEG) && !has(surviving, SEG_GONE));
    if something_to_recover_from && !found.root {
        return Some("a checkpoint or a segment survived and recovery found no root".to_string());
    }
    if found.r2_live && !has(surviving, REQ2) {
        return Some("recovery calls R2 live and its request file is gone".to_string());
    }
    if found.r1_live && (!has(surviving, REQ1) || has(surviving, REQ1_GONE)) {
        return Some("recovery calls R1 live and its request file is gone".to_string());
    }
    // A request that a surviving record names and whose file survives is not dropped: the adopt of R2 (or the checkpoint) is durable.
    let promised_r2: bool = (has(surviving, ADOPT2) && has(surviving, SEG) && !has(surviving, SEG_GONE)) || has(surviving, CKPT);
    // A record after a record whose own dependency is gone is dropped with it (the chain); that is the design, so the property is stated for
    // a chain whose earlier dependencies are satisfied: R1's request file survives, or R1's outcome does.
    let chain_satisfied: bool = !has(surviving, ADOPT1) || (has(surviving, REQ1) && !has(surviving, REQ1_GONE)) || has(surviving, OUT1);
    if promised_r2 && has(surviving, REQ2) && chain_satisfied && !found.r2_live {
        return Some("R2 is named by a surviving record, its request file survives, and recovery dropped it".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 500_000, max_depth: 40 };
}

/// Explores every schedule of the lane with write-back and one power loss.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, |_| { return None; }, limits());
}
