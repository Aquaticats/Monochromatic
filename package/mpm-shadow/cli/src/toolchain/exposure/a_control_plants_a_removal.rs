//! What:
//! The instant before one removal of the exposure farm's rebuild,
//! which a build made for controls can plant:
//! another run's removal of the same link,
//! or an `unlink` the kernel refuses.
//! Why:
//! A control has to show what a run prints and leaves when the link it came to remove is already gone,
//! and when a removal fails after earlier ones succeeded;
//! no disposable fixture can make either state happen between the rebuild's own reads on demand.
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
//! type What = 'another-run-removes-it' | { refused: number };
//! function plantingAtTheRemovalOf(name: string, what: What): RemovalPlantGuard;  // tests only
//! function atTheInstantBeforeARemoval(name: string, path: string): Error | undefined;  // always undefined in a shipped build
//! ```

/// Imports the error type a planted refusal answers with.
use std::io;

/// Imports the cell a control's plant lives in.
#[cfg(test)]
use std::cell::Cell;

/// What:
/// What a control plants at the removal of one link:
/// another run removing it first,
/// or the removal itself refusing.
/// Why:
/// The two are the two states a removal has to survive without failing the whole rebuild,
/// and a shipped build plants neither,
/// so the type only says what a control may ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type What = 'another-run-removes-it' | { refused: number };
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum What {
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
/// the link name it is about and what happens at that removal.
/// Why:
/// A rebuild removes several links,
/// and a control is about one of them,
/// so the name is what tells the planted instant from every other removal of the same run.
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
    /// What happens at that removal.
    what: What,
}

#[cfg(test)]
thread_local! {
    /// The plant this thread's rebuild answers at one removal.
    static THE_PLANT: Cell<Option<Plant>> = const { Cell::new(None) };
}

/// What:
/// Forgets a plant,
/// so a removal after a control's own is the real one.
/// Why:
/// A plant that outlived its control would decide another control's removal on the same thread.
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
    /// and a plant left behind would decide another control's removal.
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
/// Plants `what` for this thread's removal of the link named `name`,
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
///   const plant = plantedFor(name);
///   if (plant === undefined) return undefined;
///   forgetThePlant();
///   if (plant.what === 'another-run-removes-it') { removeFileSync(path); return undefined; }
///   return Object.assign(new Error('planted'), { errno: plant.what.refused });
/// }
/// ```
#[cfg(test)]
pub(super) fn at_the_instant_before_a_removal(
    name: &str,
    path: &std::path::Path,
) -> Option<io::Error> {
    let plant: Plant = THE_PLANT.with(|cell| return cell.take())?;
    if plant.name != name {
        // What: this removal is not the one the control planted, so the plant goes back for the removal it names.
        // Why: one rebuild removes several links, and the plant has to be found at its own name and nowhere else.
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
    forget_the_plant();
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
    };
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
