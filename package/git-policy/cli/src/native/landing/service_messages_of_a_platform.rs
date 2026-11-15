//! What:
//!  The sentence a platform without a user service manager leaves a person,
//!  which every failure that needs one ends with.
//! Why:
//!   Windows and the BSD systems have no activation built,
//! and Linux without user services can turn them on,
//! so each names its own way forward and none says the same thing.
//!  They live beside the renderings rather than inside them,
//!  because the file that renders every failure is at the line bound the repository linter enforces
//!  (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! platformRemedy(platform) // the sentence a person reads
//! ```

use crate::service_messages::{LINGER_COMMAND, Platform};

/// What:
///  The remedy sentence for a platform.
/// Why:
///   Windows and the BSD systems have no activation yet;
/// Linux without user services can turn them on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// platformRemedy(platform)
/// ```
pub fn platform_remedy(platform: Platform) -> String {
    return match platform {
        Platform::Windows => String::from(
            "Committing through this wrapper is not supported on Windows yet. Use the Linux or macOS machine for commits \
             to this repository.",
        ),
        Platform::Bsd => String::from(
            "Committing through this wrapper is not supported on this system yet. Use a Linux or macOS machine for \
             commits to this repository.",
        ),
        Platform::LinuxWithoutUserServices => format!(
            "This account has no user service manager. Turn it on with `{LINGER_COMMAND}` (the system must run \
             systemd), or commit from a system that does."
        ),
        Platform::Other => String::from(
            "Committing through this wrapper is not supported on this system. Use a Linux or macOS machine for \
             commits to this repository.",
        ),
    };
}
