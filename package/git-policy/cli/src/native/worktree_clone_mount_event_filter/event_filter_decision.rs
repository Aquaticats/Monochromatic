//! What:
//!  What the event filter decided about one delivery,
//!  why it decided it,
//!  and the one line each answer is written as.
//! Why:
//!  The route wants only the events about its clone parent and the datasets
//!  below it,
//!  so every other history event of every pool is dropped before the
//!  compatibility gate and before `/dev/zfs` is opened,
//!  and the drop is one line a person and the journal can read
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The event filter,
//! in brief",
//!  and veto entry V2).
//!  A dropped event proves nothing about a request,
//!  so every reason here names the names it compared and says that no pass ran,
//!  and a request that no event reached is the finding the design already gives
//!  `doctor`
//!  (the same file,
//!  "What doctor reports for a clone whose last event was lost").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const decision = decideAboutTheEvent(event, parent); // { passed: 'below-the-parent' } or { dropped: { ... } }
//! if (decision.dropped) console.log(decision.line);
//! ```

/// What:
///  The filter's answer for one delivery.
/// Why:
///   A pass lets the mode run and a drop ends the invocation,
///  so the two arms carry the reason a person and the journal read,
///  and nothing else about the event is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FilterDecision = { kind: 'passed'; why: PassedWhy } | { kind: 'dropped'; why: DroppedWhy };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FilterDecision {
    /// The event may start a pass.
    Passed(PassedWhy),
    /// The event was dropped and no pass runs.
    Dropped(DroppedWhy),
}

/// What:
///  Why an event may start a pass.
/// Why:
///   Two of the reasons are the route's own scope and three are names the
///  filter could not learn,
///  and a reader has to tell the accidental recovery a missing name gives from
///  the event the route truly wants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PassedWhy = 'the-parents-own-dataset' | 'below-the-parent' | 'the-parent-could-not-be-resolved' | 'the-event-named-no-pool' | 'the-event-named-no-dataset';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassedWhy {
    /// The event is about the clone parent dataset itself.
    TheParentsOwnDataset,
    /// The event is about a dataset below the clone parent.
    BelowTheParent,
    /// The route's parent dataset name could not be read,
    /// so the handler's own checks decide.
    TheParentCouldNotBeResolved,
    /// The delivery carried no pool name,
    /// so nothing proves it is another pool's.
    TheEventNamedNoPool,
    /// The delivery carried no dataset name,
    /// so nothing places it outside the parent.
    TheEventNamedNoDataset,
}

/// What:
///  Why an event was dropped.
/// Why:
///   Each cause is a comparison the filter made on names the delivery carried,
///  and a person reading the line needs both sides of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DroppedWhy = { kind: 'another-pool'; parentPool: string; eventPool: string } | { kind: 'outside-the-parent'; parent: string; dataset: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DroppedWhy {
    /// The event is about another pool than the parent's.
    AnotherPool {
        /// The pool the clone parent lives in.
        parent_pool: String,
        /// The pool the event named.
        event_pool: String,
    },
    /// The event's dataset is neither the parent nor below it.
    OutsideTheParent {
        /// The pinned clone parent dataset.
        parent: String,
        /// The event's dataset,
        /// with a snapshot's `@` suffix removed.
        dataset: String,
    },
}

impl FilterDecision {
    /// What:
    ///  Whether the event may start a pass.
    /// Why:
    ///   The caller decides what a pass costs and what a drop costs,
    ///  and one answer keeps that branch out of the caller's reading of the
    ///  reasons.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const passes = decision.kind === 'passed';
    /// ```
    pub fn passes(&self) -> bool {
        return matches!(self, FilterDecision::Passed(_));
    }

    /// What:
    ///  The one line this answer is written as.
    /// Why:
    ///   A drop is the decision a person has to be able to see,
    ///  because a dropped event is the trigger that did not arrive,
    ///  and the line must say that no pass ran and that nothing was opened;
    ///  a pass says which of the route's own cases it fell in,
    ///  so a verbose run tells the route's events from the accidental recovery
    ///  a missing name gives.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// console.log(decision.line);
    /// ```
    pub fn line(&self) -> String {
        return match self {
            FilterDecision::Passed(why) => passed_line(*why),
            FilterDecision::Dropped(why) => dropped_line(why),
        };
    }
}

/// What:
///  The line a pass is written as.
/// Why:
///  Each case names what the delivery proved,
///  so a reader can tell a drop's cost from a pass that only a nameless
///  delivery made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = passedLine(why);
/// ```
fn passed_line(why: PassedWhy) -> String {
    let because: &str = match why {
        PassedWhy::TheParentsOwnDataset => "it is about the clone parent dataset itself",
        PassedWhy::BelowTheParent => "it is about a dataset below the clone parent",
        PassedWhy::TheParentCouldNotBeResolved => {
            "the clone parent dataset's name could not be read, so the handler's own checks decide this event"
        }
        PassedWhy::TheEventNamedNoPool => {
            "the delivery named no pool, so nothing proves it is another pool's"
        }
        PassedWhy::TheEventNamedNoDataset => {
            "the delivery named no dataset, so nothing places it outside the clone parent"
        }
    };
    return format!("the event filter let this history event through because {because}");
}

/// What:
///  The line a drop is written as.
/// Why:
///  The comparison that dropped the event is printed with both its names,
///  and the sentence says no pass ran and that nothing was opened,
///  which is what makes a drop safe to do without the gate
///  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
///  veto entry V2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = droppedLine({ kind: 'outside-the-parent', parent, dataset });
/// ```
fn dropped_line(why: &DroppedWhy) -> String {
    let because: String = match why {
        DroppedWhy::AnotherPool {
            parent_pool,
            event_pool,
        } => format!(
            "it named pool `{event_pool}` and the clone parent lives in pool `{parent_pool}`"
        ),
        DroppedWhy::OutsideTheParent {
            parent,
            dataset,
        } => format!("its dataset `{dataset}` is neither `{parent}` nor below it"),
    };
    return format!(
        "the event filter dropped this history event because {because}; no pass ran, nothing was mounted or unmounted, and `/dev/zfs` was not opened"
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "event_filter_decision_tests.rs"]
mod tests;
