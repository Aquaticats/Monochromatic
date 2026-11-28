//! What:
//! The check that every bin name a plan declares fits the longest name the exposure folder's file
//! system holds a link's temporary name under.
//! Why:
//! Every exposure link is written under a dotted temporary name beside the real one and renamed over it
//! (`install_support/the_atomic_symlink`),
//! and that temporary name carries the bin name plus the facts about the install writing it,
//! which came to about seventy bytes.
//! A declared bin that long used to be linked by the release before `0.2.7-alpha.19`;
//! with the longer temporary name the link write fails with `File name too long`,
//! and the failure names the temporary name a person never wrote and cannot find in the plan.
//! So the name is judged where the plan is read,
//! against the limit the file system itself reports for the folder the link goes in,
//! and the refusal names the row, the bin and that limit.
//! The other way this could have been done is a shorter temporary name,
//! which gives up the identity the name carries:
//! which process, which start of it, which process namespace and which boot wrote the link,
//! the facts that let a later rebuild tell a link an install still holds from one an install that
//! ended left behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-bin-names-the-exposure-folder-can-hold.ts
//! export function refuseBinNamesTheExposureFolderCannotHold(plan: Plan, scope: Scope): void;  // throws
//! ```

/// Imports what this module uses from `std::path`.
use std::path::Path;

/// Imports `statvfs` from `rustix`,
/// the one call that asks a file system what it holds,
/// through the safe wrapper this crate already uses for its other calls.
use rustix::fs::statvfs;

/// Imports what this module uses from `super::error`.
use super::error::ToolchainError;
/// Imports `Plan` from `super::plan`.
use super::plan::Plan;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;
/// Imports the purpose text a temporary link's name carries,
/// from the module that builds that name.
use super::install_support::TEMPORARY_PURPOSE;

/// What:
/// The number of bytes the UUID form of a boot identifier takes,
/// which is the length of the field a hidden or temporary name carries for its boot.
/// Why:
/// The kernel writes its boot identifier in that form (`/proc/sys/kernel/random/boot_id`),
/// and `is_boot_id` accepts exactly that shape (`archive_staging`,
/// `the_owner_of_a_hidden_sibling`),
/// so a name this build writes carries at most these bytes for that field.
/// A run that cannot read the identifier writes the shorter name with no boot at all,
/// which is shorter than this reserve and so covered by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_BOOT_IDENTIFIERS_LENGTH = 36;
/// ```
const THE_BOOT_IDENTIFIERS_LENGTH: usize = 36;

/// What:
/// The most bytes a link's temporary name adds to the bin name it stands beside.
/// Why:
/// The refusal has to be checked against the widest name any run can write for that bin,
/// not this run's own:
/// a run whose process number, start time or namespace is longer than this one's writes a longer name
/// for the same bin,
/// so the reserve is the widest each field can be under its own type
/// (a `u32` process number, a `u64` start time and a `u64` namespace,
/// which are the fields `Owner` holds)
/// plus the boot identifier's own length.
/// The shape is `.<bin>.{purpose}.<pid>.<start>.<namespace>.<boot>`
/// (`archive_staging`, `hidden_sibling`),
/// so the leading dot is counted here as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theBytesATemporaryNameAdds(): number {
///   return 1 + 1 + 'tmp'.length + 1 + 10 + 1 + 20 + 1 + 20 + 1 + 36;
/// }
/// ```
pub(crate) fn the_bytes_a_temporary_name_adds() -> usize {
    // What: how many decimal digits each numeric field can hold at its widest.
    // Why: a narrower process number on this machine still leaves a later run free to write the widest one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const digits = (widest: number) => String(widest).length;
    // ```
    let process_number: usize = format!("{}", u32::MAX).len();
    let start_time: usize = format!("{}", u64::MAX).len();
    let process_namespace: usize = format!("{}", u64::MAX).len();
    return 1
        + 1
        + TEMPORARY_PURPOSE.len()
        + 1
        + process_number
        + 1
        + start_time
        + 1
        + process_namespace
        + 1
        + THE_BOOT_IDENTIFIERS_LENGTH;
}

/// What:
/// The longest name the file system holding `folder` accepts,
/// asked of the file system with `statvfs`,
/// or nothing when no directory from `folder` up to the root could be asked.
/// Why:
/// The limit is a property of the file system the link is written on,
/// not a constant: eCryptfs holds 143 bytes where ext4 holds 255,
/// so the question is put to the file system itself.
/// The exposure folder need not exist yet (a scope's first `install` makes it),
/// so the nearest directory above it that does exist answers for it,
/// which is the same file system unless a mount sits exactly there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theLongestNameTheFolderHolds(folder: string): number | undefined {
///   for (let at = folder; ; at = dirname(at)) {
///     try { const facts = statvfs(at); if (facts.f_namemax > 0) return facts.f_namemax; }
///     catch { /* ask the directory above it */ }
///     if (at === dirname(at)) return undefined;
///   }
/// }
/// ```
fn the_longest_name_the_folder_holds(folder: &Path) -> Option<u64> {
    let mut asking: &Path = folder;
    loop {
        if let Ok(facts) = statvfs(asking)
            && facts.f_namemax > 0
        {
            return Some(facts.f_namemax);
        }
        asking = asking.parent()?;
    }
}

/// What:
/// Refuses a plan that declares a bin name the exposure folder cannot hold a link's temporary name for,
/// naming the row, the bin, what the file system answered and the longest bin that would fit.
/// Why:
/// The check runs where the plan is read,
/// so the refusal comes before an install downloads anything or writes a link,
/// and every command of the scope reads the plan the same way.
/// A file system that cannot be asked leaves the question open,
/// and the link write's own error is what a person then meets;
/// nothing is refused on an answer no file system gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function refuseBinNamesTheExposureFolderCannotHold(plan: Plan, scope: Scope): void {
///   const longest = theLongestNameTheFolderHolds(scope.exposureDir);
///   if (longest === undefined) return;
///   const most = longest - theBytesATemporaryNameAdds();
///   for (const tool of plan.tools) for (const bin of tool.bins) {
///     if (Buffer.byteLength(bin) > most) throw new ToolchainError(...);
///   }
/// }
/// ```
pub(crate) fn refuse_bin_names_the_exposure_folder_cannot_hold(
    plan: &Plan,
    scope: &Scope,
) -> Result<(), ToolchainError> {
    let Some(longest) = the_longest_name_the_folder_holds(&scope.exposure_dir) else {
        return Ok(());
    };
    let added: usize = the_bytes_a_temporary_name_adds();
    let most: u64 = longest.saturating_sub(added as u64);
    for tool in &plan.tools {
        for bin in &tool.bins {
            let bytes: u64 = bin.len() as u64;
            if bytes <= most {
                continue;
            }
            return Err(ToolchainError::config(format!(
                "the toolchain plan {} declares the bin {bin:?} of {}, {bytes} bytes long, and the file system of the exposure folder {} holds names of at most {longest} bytes, so the `.{bin}.{TEMPORARY_PURPOSE}.<process id>.<start time>.<namespace>.<boot>` name its link is written under before it is renamed (at most {} bytes for that bin) cannot exist there: a bin name that folder can hold is at most {most} bytes, so declare a shorter exposed name in `bins` and the program's own name in `options.binAliases`, or run this scope from a file system whose names hold more bytes",
                scope.plan_file.display(),
                tool.id,
                scope.exposure_dir.display(),
                bytes + added as u64
            )));
        }
    }
    return Ok(());
}
