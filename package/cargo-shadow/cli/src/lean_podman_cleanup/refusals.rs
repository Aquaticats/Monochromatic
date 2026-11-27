//! What:
//! What Podman's refusal of one removal says about the image,
//! the words of the two refusals this cleanup understands,
//! and the reader that tells them apart.
//! Why:
//! Removing an image also removes an ancestor that only it referenced,
//! so a later call of the same run reads "image not known" for an image that is gone,
//! which is the outcome the run wanted and not a failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = 'AlreadyGone' | 'InUse';
//! const ALREADY_GONE = 'image not known';
//! const IN_USE = 'image is in use by a container';
//! export function refusalOf(message: string): Refusal | undefined
//! ```
/// What Podman's refusal of one removal says about the image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// Podman does not know the image any more:
    /// it went with another image of this run.
    AlreadyGone,
    /// A container Podman knows holds the image.
    InUse,
}

/// The words of a Podman refusal that says the image is gone,
/// measured on podman 6.1.3 (2026-11-08).
pub const ALREADY_GONE: &str = "image not known";

/// The words of a Podman refusal that says a container holds the image,
/// measured on podman 6.1.3 (2026-11-08).
pub const IN_USE: &str = "image is in use by a container";

/// What:
/// `pub fn refusal_of(message: &str) -> Option<Refusal>` reads the two refusals this cleanup understands.
/// Why:
/// Removing an image also removes an ancestor that only it referenced,
/// so a later call of the same run reads "image not known" for an image that is gone;
/// that is the outcome the run wanted,
/// not a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function refusalOf(message: string): Refusal | undefined
/// ```
pub fn refusal_of(message: &str) -> Option<Refusal> {
    if message.contains(ALREADY_GONE) {
        return Some(Refusal::AlreadyGone);
    }
    if message.contains(IN_USE) {
        return Some(Refusal::InUse);
    }
    return None;
}
