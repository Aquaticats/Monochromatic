//! What:
//!  The reads a forced removal makes before it ends anything,
//!  the refusal each unusable answer is,
//!  and the route the reads leave.
//! Why:
//!  "No forced removal may end a process or change a mount before it knows it can
//!  finish with every holder namespace it will meet"
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The preflight").
//!  The preflight is read-only and runs over every holder namespace before the
//!  first structural act,
//!  the first signal and the first unmount,
//!  and its order is fixed:
//!  the caller's protected set first,
//!  a blind read next,
//!  a locked holder before any signal,
//!  and the structural and route reads after them.
//!  This module holds the decision,
//!  not the reads:
//!  a control hands it what the reads answered,
//!  which is how each refusal is shown to be red without its guard.
//!  The route is a value rather than an act:
//!  way 8 where the parent shares with a group of its own and no kept receiver
//!  lies outside the act's namespaces,
//!  way 7 otherwise,
//!  and a refusal where neither fits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planTheForcedRemoval(reads);   // throws a ForcedRefusal
//! plan.route; plan.receivers;
//! ```

use crate::process::a_locked_holders_own_unit::ALockedHoldersUnit;
use crate::worktree_clone_forced_removal::caller_chain_walk::SparedBy;
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::receiver_closure::{
    MountRow,
    NamespaceTable,
    ReceiverSet,
    plan_the_receivers,
};

/// What:
///  Why a forced removal may not go on,
///  in the design's own words.
/// Why:
///  Each cause is separate because each names a different thing to a person,
///  and every one of them refuses before anything is ended or changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForcedRefusal = { kind: 'protected-holds'; holder; sparedBy } | { kind: 'blind'; reads } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForcedRefusal {
    /// A holder is the command,
    /// a task of its session,
    /// or one of its ancestors.
    ProtectedHolds {
        /// The process number of the holder.
        holder: i32,
        /// Which half of the protected set holds it.
        spared_by: SparedBy,
    },
    /// A read the preflight could not make,
    /// or a scan that did not read every namespace it meant to.
    Blind {
        /// The reads that were refused or missing,
        /// one sentence each.
        reads: Vec<String>,
    },
    /// A mount namespace the act cannot enter.
    Unenterable {
        /// The namespace's identity.
        namespace: String,
    },
    /// A holder whose mount namespace is owned by a user namespace of its own,
    /// so the kernel locked its copy,
    /// and which is no service unit of this account's own user manager,
    /// so this removal has no manager to stop it through.
    Locked {
        /// The namespace's identity.
        namespace: String,
        /// The process or unit the report names.
        holder: String,
    },
    /// A copy whose parent mount is not the checkout's own,
    /// so the act cannot tell what an unmount there would reach.
    ParentIsNotTheCheckout {
        /// The namespace's identity.
        namespace: String,
    },
    /// A structure the design refuses before any act.
    Structure {
        /// The namespace's identity.
        namespace: String,
        /// Which structural read refused,
        /// in the design's own words.
        because: &'static str,
    },
    /// A holder the preflight proved cannot be ended.
    Unendable {
        /// The process number.
        holder: i32,
        /// What the proof found.
        because: String,
    },
    /// Way 7's destination cannot be made:
    /// the parent is read-only,
    /// or the dataset has no room for one directory.
    NoDestination {
        /// The namespace's identity.
        namespace: String,
        /// Which read refused.
        because: &'static str,
    },
}

impl ForcedRefusal {
    /// The word the receipt's failed step and the report's first line carry.
    /// Why:
    ///  A person and `doctor` read the word,
    ///  and the design fixes which word each cause answers.
    pub fn word(&self) -> RefusalWord {
        return match self {
            ForcedRefusal::ProtectedHolds {
                ..
            } => RefusalWord::CallerProtectedHolds,
            ForcedRefusal::Blind {
                ..
            } => RefusalWord::HolderBlind,
            ForcedRefusal::Unenterable {
                ..
            } => RefusalWord::HolderUnendable,
            ForcedRefusal::Locked {
                ..
            } => RefusalWord::HolderLocked,
            ForcedRefusal::ParentIsNotTheCheckout {
                ..
            } => RefusalWord::HolderUnendable,
            ForcedRefusal::Structure {
                ..
            } => RefusalWord::HolderUnendable,
            ForcedRefusal::Unendable {
                ..
            } => RefusalWord::HolderUnendable,
            ForcedRefusal::NoDestination {
                ..
            } => RefusalWord::HolderUnendable,
        };
    }
}

/// What:
///  Which way the act takes for one holder namespace.
/// Why:
///  The route decides what the act changes:
///  way 8 changes nothing and way 7 stops the parent propagating and writes one
///  short-lived directory,
///  and the report has to say which it took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Route = 'in-place-by-the-handle' | 'slave-move-and-unmount';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Route {
    /// Way 8:
    /// unmount the copy in place by the add's recorded handle.
    InPlaceByTheHandle,
    /// Way 7:
    /// make the parent a slave,
    /// move the copy onto a directory in the checkout and unmount it there.
    SlaveMoveAndUnmount,
}

impl Route {
    /// The word a report names the route by.
    pub fn word(self) -> &'static str {
        return match self {
            Route::InPlaceByTheHandle => "in-place-by-the-handle",
            Route::SlaveMoveAndUnmount => "slave-move-and-unmount",
        };
    }
}

/// What:
///  What the reads answered,
///  one value per read the preflight makes.
/// Why:
///  The reads belong to the handler,
///  which has root and may enter namespaces,
///  and the decision belongs here,
///  so a control shows every refusal without root and without a mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreflightReads = { dataset; tables; acting; copy; handled; protected?; ... };
/// ```
pub struct PreflightReads<'a> {
    /// The clone's dataset name,
    /// which a mount of the clone matches exactly.
    pub dataset: &'a str,
    /// Every reachable namespace's table,
    /// the holder's own included.
    pub tables: &'a [NamespaceTable],
    /// The holder namespace whose copy the act is about to take out.
    pub acting: &'a str,
    /// The copy in that namespace.
    pub copy: &'a MountRow,
    /// The namespaces the act handles,
    /// which is every holder namespace the preflight accepted.
    pub handled: &'a [String],
    /// The protected holder the matcher found,
    /// if any,
    /// with which half of the set holds it.
    pub protected: Option<(i32, SparedBy)>,
    /// Every read the preflight could not make,
    /// and every namespace the scan could not read,
    /// one sentence each.
    pub blind: &'a [String],
    /// The namespaces the act cannot enter.
    pub unenterable: &'a [String],
    /// The holder namespaces whose owner is a user namespace of their own,
    /// with the holder the report names.
    pub locked: &'a [(String, String)],
    /// The locked holders that are service units of this account's own user
    /// manager,
    /// which the act stops through that manager before it changes anything else.
    pub locked_units: &'a [ALockedHoldersUnit],
    /// The namespaces whose copy's parent is not the checkout's own mount.
    pub parents_not_the_checkout: &'a [String],
    /// The structural refusals,
    /// each a namespace and the design's own reason.
    pub structural: &'a [(String, &'static str)],
    /// The holders the endability read proved cannot be ended.
    pub unendable: &'a [(i32, String)],
    /// The namespaces with no room for way 7's directory,
    /// or a read-only parent,
    /// each named by the namespace whose way 7 it stops.
    pub no_destination: &'a [(String, &'static str)],
    /// Whether the preflight proved the handle resolves to the copy at the top of the
    /// slot in this namespace,
    /// which way 8 needs and the add's own record alone does not show.
    pub handle_proved: bool,
}

/// What:
///  What the preflight left:
///  the route for this namespace and the receivers the unmount reaches.
/// Why:
///  A route is per namespace,
///  because one holder's parent may share and another's may not,
///  and the receivers are the report's own list of what the act changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForcedPlan = { route; receivers; unitsToStop };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForcedPlan {
    /// The way the act takes here.
    pub route: Route,
    /// The namespaces the unmount reaches and what each holds at the slot.
    pub receivers: ReceiverSet,
    /// The locked holders' own units the act stops first,
    /// in the order they were recorded.
    pub units_to_stop: Vec<ALockedHoldersUnit>,
}

/// What:
///  Decide whether the act may go on for one holder namespace,
///  and by which route.
/// Why:
///  The order is the design's and is checked in this order,
///  so a holder that is the caller's own is named as that rather than as a
///  blind or locked read,
///  and a locked holder is refused before any signal.
///  A route is chosen only after every refusal has passed,
///  because a structural change made before a refusal deeper in the list would
///  be a change the removal cannot justify.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (reads.protected) throw protectedHolds; if (reads.blind.length) throw blind;
/// if (reads.unenterable.length) throw unenterable; if (reads.locked.length) throw locked;
/// ...
/// const plan = planTheReceivers(acting, copy, tables, dataset, handled);
/// const route = plan.wayEight && reads.handleRecorded ? 'in-place-by-the-handle' : 'slave-move-and-unmount';
/// ```
pub fn plan_the_forced_removal(reads: &PreflightReads<'_>) -> Result<ForcedPlan, ForcedRefusal> {
    if let Some((holder, spared_by)) = reads.protected {
        return Err(ForcedRefusal::ProtectedHolds {
            holder,
            spared_by,
        });
    }
    if !reads.blind.is_empty() {
        return Err(ForcedRefusal::Blind {
            reads: reads.blind.to_vec(),
        });
    }
    if let Some(namespace) = reads.unenterable.first() {
        return Err(ForcedRefusal::Unenterable {
            namespace: namespace.clone(),
        });
    }
    if let Some((namespace, holder)) = reads.locked.first() {
        return Err(ForcedRefusal::Locked {
            namespace: namespace.clone(),
            holder: holder.clone(),
        });
    }
    if let Some(namespace) = reads.parents_not_the_checkout.first() {
        return Err(ForcedRefusal::ParentIsNotTheCheckout {
            namespace: namespace.clone(),
        });
    }
    if let Some((namespace, because)) = reads.structural.first() {
        return Err(ForcedRefusal::Structure {
            namespace: namespace.clone(),
            because,
        });
    }
    if let Some((holder, because)) = reads.unendable.first() {
        return Err(ForcedRefusal::Unendable {
            holder: *holder,
            because: because.clone(),
        });
    }
    let Some(acting) = reads
        .tables
        .iter()
        .find(|table| return table.namespace == reads.acting)
    else {
        return Err(ForcedRefusal::Unenterable {
            namespace: String::from(reads.acting),
        });
    };
    let receivers: ReceiverSet = plan_the_receivers(
        acting,
        reads.copy,
        reads.tables,
        reads.dataset,
        reads.handled,
    );
    let in_place: bool = receivers.way_eight && reads.handle_proved;
    if !in_place {
        // Way 7 needs one directory on the parent mount's root, so its reads are the route's own precondition, and
        // only the namespace that takes way 7 is judged by them.
        if let Some((_namespace, because)) = reads
            .no_destination
            .iter()
            .find(|(namespace, _because)| return *namespace == reads.acting)
        {
            return Err(ForcedRefusal::NoDestination {
                namespace: String::from(reads.acting),
                because,
            });
        }
    }
    let route: Route = if in_place {
        Route::InPlaceByTheHandle
    } else {
        Route::SlaveMoveAndUnmount
    };
    return Ok(ForcedPlan {
        route,
        receivers,
        // What: the locked holders' own units, carried to the structural stage, which stops them first.
        // Why:  the owner's answer stops each through the account's own manager before any other namespace is changed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // unitsToStop: reads.lockedUnits,
        // ```
        units_to_stop: reads.locked_units.to_vec(),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_preflight_tests.rs"]
mod tests;
