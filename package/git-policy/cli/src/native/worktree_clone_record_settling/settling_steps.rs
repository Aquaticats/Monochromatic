//! What:
//!  The two orders the settling command removes records in,
//!  as data a control reads without watching three folders.
//! Why:
//!  The order is one sentence a sibling design quotes
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command" and "What a record whose clone is gone means"):
//!  a lost lock written again,
//!  then the registration,
//!  then the plan,
//!  then the two inventories,
//!  then the binding,
//!  then the claim.
//!  The plan goes before the inventories so the binding stays the witness a
//!  replayed run reads,
//!  and the claim goes last because it is what keeps a second command out.
//!  With a checkout begun only the records go:
//!  the registration and the slot stay for `git worktree remove`,
//!  and no lock is written there,
//!  because an ordinary worktree is never locked and a lock would make that
//!  removal refuse
//!  (U19 of the seventh add-side read).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const steps = theSteps({ checkoutBegun: false, planStands: true });
//! ```

/// What:
///  One kind of removal the settling command makes.
/// Why:
///  A control asserts the two orders,
///  and the runner executes the steps it was given,
///  so neither can drift from the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = 'the-lost-lock' | 'the-registration-recipe' | 'the-plan' | 'the-origin-inventory' | 'the-clone-inventory' | 'the-binding' | 'the-claim';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettlingStep {
    /// The registration's lock written again when it is gone or cut short.
    TheLostLock,
    /// The recipe's eight removals on the recorded registration.
    TheRegistrationRecipe,
    /// The destruction plan a cancelled removal kept,
    /// after its lost names are printed.
    ThePlan,
    /// `<slot>.origin-inventory`.
    TheOriginInventory,
    /// `<slot>.clone-inventory`.
    TheCloneInventory,
    /// `<slot>.binding`, the last witness.
    TheBinding,
    /// The slot's claim,
    /// released after every record went.
    TheClaim,
}

impl SettlingStep {
    /// The words a line names this step with.
    pub fn words(self) -> &'static str {
        return match self {
            SettlingStep::TheLostLock => "the registration's lost lock, written again with this add's sentence",
            SettlingStep::TheRegistrationRecipe => "the registration's own files, by the recipe's eight steps, and its directory",
            SettlingStep::ThePlan => "the destruction plan, after the names it recorded as destroyed are printed",
            SettlingStep::TheOriginInventory => "the origin-side inventory",
            SettlingStep::TheCloneInventory => "the clone-side inventory",
            SettlingStep::TheBinding => "the binding record",
            SettlingStep::TheClaim => "the slot's claim, released",
        };
    }
}

/// The order a slot with no checkout begun takes,
/// which is the design's own list.
pub const WHEN_NO_CHECKOUT_HAS_BEGUN: [SettlingStep; 7] = [
    SettlingStep::TheLostLock,
    SettlingStep::TheRegistrationRecipe,
    SettlingStep::ThePlan,
    SettlingStep::TheOriginInventory,
    SettlingStep::TheCloneInventory,
    SettlingStep::TheBinding,
    SettlingStep::TheClaim,
];

/// The order a slot with a checkout begun takes:
/// the records alone,
/// with the registration and the slot left for `git worktree remove`.
pub const WHEN_A_CHECKOUT_HAS_BEGUN: [SettlingStep; 5] = [
    SettlingStep::ThePlan,
    SettlingStep::TheOriginInventory,
    SettlingStep::TheCloneInventory,
    SettlingStep::TheBinding,
    SettlingStep::TheClaim,
];

/// What:
///  The steps this run makes,
///  by the checkout's answer and whether a plan stands.
/// Why:
///  A plan that no cancelled removal left has nothing to remove,
///  so the step is left out rather than made a no-op,
///  and the order of what remains is the const's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const steps = theSteps({ checkoutBegun, planStands });
/// ```
pub fn the_steps(TheOrderFacts { checkout_begun, plan_stands }: TheOrderFacts) -> Vec<SettlingStep> {
    let whole: &[SettlingStep] = match checkout_begun {
        true => WHEN_A_CHECKOUT_HAS_BEGUN.as_slice(),
        false => WHEN_NO_CHECKOUT_HAS_BEGUN.as_slice(),
    };
    return whole
        .iter()
        .copied()
        .filter(|step| return plan_stands || *step != SettlingStep::ThePlan)
        .collect();
}

/// What:
///  The two answers the order is chosen by.
/// Why:
///  Both come from reads this command made,
///  and the runner takes the same pair,
///  so the steps a control asserts are the steps a run makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OrderFacts = { checkoutBegun: boolean; planStands: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TheOrderFacts {
    /// Whether the slot holds an entry besides `.git` with nothing mounted there.
    pub checkout_begun: bool,
    /// Whether a destruction plan naming this clone stands.
    pub plan_stands: bool,
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_steps_tests.rs"]
mod tests;
