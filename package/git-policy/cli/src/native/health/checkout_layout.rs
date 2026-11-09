//! What:
//!  How the checkout the doctor's filesystem section reports on is laid out,
//! and the command that converts each
//!  layout into a btrfs subvolume.
//! Why:
//!  The section's advice names a command,
//! and the two layouts need different ones:
//! `migrate-to-subvolume` converts the
//!  repository's main worktree,
//! and `convert-worktree-to-subvolume` a linked one.
//!  Git answers which worktree the command runs in (its Git directory equals the common one there),
//! so the two
//!  commands are chosen from Git's answer and never guessed.
//!  This module names the two layouts on every platform,
//! so the health report speaks of a checkout everywhere,
//!  while the facts and the wording stay with the Linux filesystem section.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CheckoutLayout = 'main' | 'linked';
//! ```

/// What:
///  How the checkout the filesystem section reports on is laid out.
/// Why:
///   The command that converts a checkout depends on it,
/// and printing the other layout's command would name one that
///  refuses this checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutLayout = 'main' | 'linked';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckoutLayout {
    /// The repository's main worktree.
    Main,
    /// A linked worktree.
    Linked,
}

/// What:
///  The command that converts a checkout of this layout into a subvolume.
/// Why:
///   The advice prints the command the person's own checkout needs,
/// and never the one that would refuse it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// layout === 'main' ? 'migrate-to-subvolume' : 'convert-worktree-to-subvolume';
/// ```
pub fn conversion_command(layout: CheckoutLayout) -> &'static str {
    return match layout {
        CheckoutLayout::Main => "migrate-to-subvolume",
        CheckoutLayout::Linked => "convert-worktree-to-subvolume",
    };
}
