//! What:
//! One of the three calls a removal of the exposure farm's rebuild makes,
//! which a build made for controls can plant:
//! another run's removal of the same link,
//! or an `unlink` the kernel refuses.
//! The three calls are the read of the entry's own metadata,
//! the read of the link's target,
//! and the removal itself.
//! Why:
//! A control has to show what a run prints and leaves when the link it came to is already gone,
//! and when a removal fails after earlier ones succeeded;
//! the claim that a link another run took is no failure has to hold at every one of the three calls,
//! because the name can be gone before this run has read its metadata
//! as well as between the read of its target and its removal.
//! No disposable fixture can make either state happen between the rebuild's own reads on demand.
//! A shipped build holds no way to plant one:
//! every read below is compiled out of a build that is not a test,
//! the shape `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` chose,
//! as the link write's own plant does (`install_support`,
//! `a_control_refuses_the_link_move`).
//! Nothing here is read from the environment at all,
//! so no `Environment=` line and no shell can plant it:
//! a control plants one for its own thread and for the one name it is about.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-control-plants-a-removal.ts
//! type What = 'another-run-removes-it-before-the-link-is-read' | 'another-run-removes-it-before-the-target-is-read' | 'another-run-removes-it' | { refused: number };
//! function plantingAtTheRemovalOf(name: string, what: What): RemovalPlantGuard;  // tests only
//! function atTheInstantBeforeTheLinkIsRead(name: string, path: string): void;  // does nothing in a shipped build
//! function atTheInstantBeforeTheTargetIsRead(name: string, path: string): void;  // does nothing in a shipped build
//! function atTheInstantBeforeARemoval(name: string, path: string): Error | undefined;  // always undefined in a shipped build
//! ```

/// Imports the error type a planted refusal answers with.
use std::io;

/// Imports the cell a control's plant lives in.
#[cfg(test)]
use std::cell::Cell;

/// What:
/// What a control plants at one of the calls a removal makes:
/// another run removing the link before this run reads the entry's own metadata,
/// another run removing it before this run reads the link's target,
/// another run removing it in the instant before this run's own removal,
/// or the removal itself refusing.
/// Why:
/// Each of those is a state a removal has to survive without failing the whole rebuild,
/// and a shipped build plants none,
/// so the type only says what a control may ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type What = 'another-run-removes-it-before-the-link-is-read' | 'another-run-removes-it-before-the-target-is-read' | 'another-run-removes-it' | { refused: number };
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum What {
    /// Another run's rebuild removes the link before this run reads the entry's own metadata.
    AnotherRunRemovesItBeforeTheLinkIsRead,
    /// Another run's rebuild removes the link before this run reads the link's target.
    AnotherRunRemovesItBeforeTheTargetIsRead,
    /// Another run's rebuild removes the link,
    /// and names it,
    /// in the instant before this run's own removal.
    AnotherRunRemovesIt,
    /// The removal answers with this operating system's error number.
    Refused {
        /// The operating system's number for the error.
        code: i32,
    },
}

/// What:
/// The plant one control made:
/// the link name it is about and what happens at that call.
/// Why:
/// A rebuild removes several links,
/// and a control is about one of them,
/// so the name is what tells the planted instant from every other call of the same run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plant = { name: string; what: What };
/// ```
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct Plant {
    /// The link name the plant answers for.
    name: String,
    /// What happens at that call.
    what: What,
}

#[cfg(test)]
thread_local! {
    /// The plant this thread's rebuild answers at one call of one removal.
    static THE_PLANT: Cell<Option<Plant>> = const { Cell::new(None) };
}

/// What:
/// Forgets a plant,
/// so a call after a control's own is the real one.
/// Why:
/// A plant that outlived its control would decide another control's call on the same thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forgetThePlant(): void
/// ```
#[cfg(test)]
fn forget_the_plant() {
    THE_PLANT.with(|cell| {
        cell.set(None);
    });
}

/// What:
/// The release of a plant,
/// taken by the guard `planting_at_the_removal_of` answers.
/// Why:
/// A control's failing assertion raises a panic,
/// which skips the line that would forget the plant,
/// and a plant left armed would decide the next run on this thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RemovalPlantGuard { [Symbol.dispose]() { forgetThePlant(); } }
/// ```
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct RemovalPlantGuard;

/// The release of a plant.
#[cfg(test)]
impl Drop for RemovalPlantGuard {
    /// What:
    /// Forgets the plant this thread made,
    /// which is the one release a control's body must reach however it leaves.
    /// Why:
    /// The drop is the only place that runs on the way out of a panic,
    /// and a plant left behind would decide another control's call.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { forgetThePlant(); }
    /// ```
    fn drop(&mut self) {
        forget_the_plant();
    }
}

/// What:
/// Plants `what` for this thread's call about the link named `name`,
/// and answers the guard that forgets it when the control's body leaves,
/// however it leaves.
/// Why:
/// A control asks for the state it means and nothing else,
/// and the guard means a failed assertion cannot arm a plant for another test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plantingAtTheRemovalOf(name: string, what: What): RemovalPlantGuard {
///   THE_PLANT[threadId] = { name, what };
///   return new RemovalPlantGuard();
/// }
/// ```
#[cfg(test)]
pub(crate) fn planting_at_the_removal_of(
    name: &str,
    what: What,
) -> RemovalPlantGuard {
    THE_PLANT.with(|cell| {
        cell.set(Some(Plant {
            name: name.to_string(),
            what,
        }));
    });
    return RemovalPlantGuard;
}

/// What:
/// Takes the plant this thread armed for the link called `name`,
/// when it is a plant for the call asking now.
/// Why:
/// One removal makes three calls about the same name,
/// so each call has to leave a plant armed for another call where it is;
/// taking it first is what makes a plant answer exactly once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeThePlantFor(name: string, isForThisCall: (what: What) => boolean): Plant | undefined {
///   const plant = THE_PLANT[threadId];
///   THE_PLANT[threadId] = undefined;
///   if (plant === undefined) return undefined;
///   if (plant.name !== name || !isForThisCall(plant.what)) { THE_PLANT[threadId] = plant; return undefined; }
///   return plant;
/// }
/// ```
#[cfg(test)]
fn take_the_plant_for(
    name: &str,
    is_for_this_call: impl FnOnce(&What) -> bool,
) -> Option<Plant> {
    // What: the plant this thread holds, taken out of the cell, or nothing when the thread armed none.
    // Why: taking it first means a plant answers one call, and no call after it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const plant = THE_PLANT[threadId];
    // THE_PLANT[threadId] = undefined;
    // if (plant === undefined) return undefined;
    // ```
    let plant: Plant = THE_PLANT.with(|cell| return cell.take())?;
    if plant.name != name || !is_for_this_call(&plant.what) {
        // What: the plant goes back for the call and the name it was armed for.
        // Why: one removal makes three calls and one rebuild removes several links, and only the planted call
        //  may answer the plant.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // THE_PLANT[threadId] = plant;
        // return undefined;
        // ```
        THE_PLANT.with(|cell| {
            cell.set(Some(plant));
        });
        return None;
    }
    // What: the plant, which this call answered and which no later call is to see.
    // Why: the caller reads what to do from it, and it is already forgotten.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return plant;
    // ```
    return Some(plant);
}

/// What:
/// The instant before this run reads an entry's own metadata,
/// where a control has another run take the link.
/// Why:
/// A rebuild lists the folder first and reads each entry after that,
/// so another install can remove the name between those two steps;
/// the state is what the claim "no failure here" is about, and no fixture can make it happen on demand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function atTheInstantBeforeTheLinkIsRead(name: string, path: string): void {
///   if (!takeThePlantFor(name, (what) => what === 'another-run-removes-it-before-the-link-is-read')) return;
///   removeFileSync(path);
/// }
/// ```
#[cfg(test)]
pub(super) fn at_the_instant_before_the_link_is_read(
    name: &str,
    path: &std::path::Path,
) {
    // What: whether this call is the one the thread's plant is about.
    // Why: every other call and every other name goes on to read the entry as it is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const planted = takeThePlantFor(name, (what) => what === 'another-run-removes-it-before-the-link-is-read') !== undefined;
    // ```
    let planted: bool = take_the_plant_for(name, |what| {
        return matches!(what, What::AnotherRunRemovesItBeforeTheLinkIsRead);
    })
    .is_some();
    if !planted {
        return;
    }
    // What: the other run's removal, which is the whole of what the plant does.
    // Why: the read that follows then answers "no such file", exactly as the kernel does for a name another run took.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // removeFileSync(path);  // its own answer does not matter: the read below is what the control reads
    // ```
    let _ = std::fs::remove_file(path);
}

/// What:
/// The instant before this run reads the link's target,
/// where a control has another run take the link.
/// Why:
/// The metadata read and the target read are two calls,
/// so a name that was there for the first can be gone for the second,
/// and that gap is the second window the claim has to cover.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function atTheInstantBeforeTheTargetIsRead(name: string, path: string): void {
///   if (!takeThePlantFor(name, (what) => what === 'another-run-removes-it-before-the-target-is-read')) return;
///   removeFileSync(path);
/// }
/// ```
#[cfg(test)]
pub(super) fn at_the_instant_before_the_target_is_read(
    name: &str,
    path: &std::path::Path,
) {
    // What: whether this call is the one the thread's plant is about.
    // Why: the target of every other link is read as it is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const planted = takeThePlantFor(name, (what) => what === 'another-run-removes-it-before-the-target-is-read') !== undefined;
    // ```
    let planted: bool = take_the_plant_for(name, |what| {
        return matches!(what, What::AnotherRunRemovesItBeforeTheTargetIsRead);
    })
    .is_some();
    if !planted {
        return;
    }
    // What: the other run's removal.
    // Why: the target read that follows then answers "no such file", as it does for any name another run took.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // removeFileSync(path);
    // ```
    let _ = std::fs::remove_file(path);
}

/// What:
/// The error a control planted for the removal of `name`,
/// or nothing when this removal is to be made.
/// Why:
/// A control shows what a run leaves when another run took the link,
/// or when the unlink is refused,
/// and this is the one place that answers whether its own plant stands here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function atTheInstantBeforeARemoval(name: string, path: string): Error | undefined {
///   const plant = takeThePlantFor(name, (what) => what === 'another-run-removes-it' || typeof what === 'object');
///   if (plant === undefined) return undefined;
///   if (plant.what === 'another-run-removes-it') { removeFileSync(path); return undefined; }
///   return Object.assign(new Error('planted'), { errno: plant.what.refused });
/// }
/// ```
#[cfg(test)]
pub(super) fn at_the_instant_before_a_removal(
    name: &str,
    path: &std::path::Path,
) -> Option<io::Error> {
    // What: the plant this thread armed for the removal of this name, or nothing.
    // Why: the two calls above a removal leave their own plants armed, and this one reads only its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const plant = takeThePlantFor(name, (what) => what === 'another-run-removes-it' || typeof what === 'object');
    // if (plant === undefined) return undefined;
    // ```
    let plant: Plant = take_the_plant_for(name, |what| {
        return matches!(what, What::AnotherRunRemovesIt | What::Refused { .. });
    })?;
    // What: what the plant asks for at this removal:
    // another run taking the link first, so the real `unlink` below answers "no such file",
    // or the error a refusal carries.
    // Why: both are states the removal has to survive without failing the rebuild,
    // and the caller reads them from this one answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return plant.what === 'another-run-removes-it' ? (removeFileSync(path), undefined) : new Error('planted', plant.what.refused);
    // ```
    return match plant.what {
        What::AnotherRunRemovesIt => {
            let _ = std::fs::remove_file(path);
            None
        }
        What::Refused {
            code,
        } => Some(io::Error::from_raw_os_error(code)),
        What::AnotherRunRemovesItBeforeTheLinkIsRead
        | What::AnotherRunRemovesItBeforeTheTargetIsRead => None,
    };
}

/// The answer a shipped build gives:
/// every entry is read and every removal is made,
/// because nothing can plant an instant at one of them.
#[cfg(not(test))]
pub(super) fn at_the_instant_before_the_link_is_read(
    _name: &str,
    _path: &std::path::Path,
) {
    return;
}

/// The answer a shipped build gives for the read of the link's target:
/// the target is read,
/// because nothing can plant an instant at that call.
#[cfg(not(test))]
pub(super) fn at_the_instant_before_the_target_is_read(
    _name: &str,
    _path: &std::path::Path,
) {
    return;
}

/// The answer a shipped build gives:
/// every removal is made,
/// because nothing can plant an instant at it.
#[cfg(not(test))]
pub(super) fn at_the_instant_before_a_removal(
    _name: &str,
    _path: &std::path::Path,
) -> Option<io::Error> {
    return None;
}
