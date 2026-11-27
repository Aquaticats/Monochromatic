//! What:
//!  Which owners of a stage claim or of a temporary were found dead,
//!  decided before anything is removed.
//! Why:
//!  The pass that finishes an owed copy and the sweep that settles the claim act on one judgement,
//! taken before anything is changed,
//!  so neither acts on a claim whose owner still runs or cannot be judged;
//! the judgement is a value of its own because both passes carry it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdicts = new StageVerdicts(); // names of the claims and temporaries whose owner is dead
//! verdicts.namesDeadClaim(name);
//! ```

use std::collections::BTreeSet;

/// Which owners were found dead,
/// decided before anything is removed.
#[derive(Debug, Default)]
pub struct StageVerdicts {
    /// The names of the claims whose owner is dead.
    dead_claims: BTreeSet<String>,
    /// The names of the temporaries whose writer is dead.
    dead_temporaries: BTreeSet<String>,
}

impl StageVerdicts {
    /// What:
    ///  Whether the claim of this name has a dead owner.
    /// Why:
    ///   The pass that finishes an owed copy and the sweep that settles the claim act on one judgement,
    /// taken before
    ///  anything is changed,
    /// so neither acts on a claim whose owner still runs or cannot be judged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.namesDeadClaim(name)
    /// ```
    pub fn names_dead_claim(
        &self,
        name: &str,
    ) -> bool {
        return self.dead_claims.contains(name);
    }

    /// What:
    ///  Judges one claim dead:
    /// the owner that no longer runs,
    /// or a claim this release could not read and wrote again,
    /// which names no owner at all.
    /// Why:
    ///   The pass that finishes an owed copy and the sweeps of one pass act on one judgement,
    /// so a claim judged dead here is acted on wherever the verdicts are handed;
    /// a claim this release could not read is settled by the command that found it,
    /// which writes it again from the records beside it,
    /// so the sweeps of the same pass must act on it as they would on any dead owner's claim.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.recordDeadClaim(name)
    /// ```
    pub fn record_dead_claim(
        &mut self,
        name: &str,
    ) {
        let _inserted: bool = self.dead_claims.insert(name.to_string());
    }

    /// What:
    ///  Judges one temporary dead,
    /// because the writer its name carries no longer runs.
    /// Why:
    ///   The sweep that removes the temporaries reads the set this judge fills,
    /// so the judgement is taken once,
    /// before anything is removed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.recordDeadTemporary(name)
    /// ```
    pub fn record_dead_temporary(
        &mut self,
        name: &str,
    ) {
        let _inserted: bool = self.dead_temporaries.insert(name.to_string());
    }

    ///  The names of the temporaries whose writer is dead.
    /// Why:
    ///   The sweep that removes them lives beside this file,
    /// and a dead writer's names are the one thing it needs from here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdicts.deadTemporaryNames() // the names of a dead writer's temporaries
    /// ```
    pub(crate) fn dead_temporary_names(&self) -> &BTreeSet<String> {
        return &self.dead_temporaries;
    }
}
