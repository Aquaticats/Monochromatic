//! What:
//!  The line `git monochromatic-git-policy-cli doctor` prints about the repository's link to the 860 drive:
//! the same plain message the shared check prints.
//! Why:
//!  Plan item W15:
//! nothing watches the link,
//! and a real folder can replace it,
//! so the doctor says what the check says (ready,
//! the drive is not mounted,
//! a real folder replaced the link,
//! and so on).
//! The line is informational and comes with the doctor's other sections:
//! it never changes the doctor's exit status.
//! A machine with no identity file (no move was recorded) gets no line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = (await readIdentity(path)).found ? `repository link: ${(await check()).message}\n` : '';
//! ```

use crate::repository_link_check_walk::{DEFAULT_DEADLINE, check_repository_link_within};
use crate::repository_link_identity::{IdentityRead, read_identity};
use crate::repository_link_probe::{LinkProbe, RealLinkProbe};
use std::path::Path;
use std::sync::Arc;

/// What:
///  The doctor's link line for the identity below `home`,
/// asked of `probe`.
/// Why:
///  Tests give a probe over a world;
/// the doctor gives the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// doctorLinkLineWithProbe({ probe, home })
/// ```
pub fn doctor_link_line_with_probe(probe: Arc<dyn LinkProbe + Send + Sync>, home: Option<&Path>) -> String {
    let Some(directory) = home else {
        return String::new();
    };
    return match read_identity(directory) {
        IdentityRead::Absent => String::new(),
        IdentityRead::Unreadable(why) => format!("repository link: the identity file cannot be used ({why}), so the link to the 860 is not checked\n"),
        IdentityRead::Found(identity) => format!("repository link: {}\n", check_repository_link_within(&identity, probe, DEFAULT_DEADLINE).message),
    };
}

/// What:
///  The doctor's link line for the real system and the home directory of the environment.
/// Why:
///  The doctor's host section appends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// doctorLinkLine(home)
/// ```
pub fn doctor_link_line(home: Option<&Path>) -> String {
    return doctor_link_line_with_probe(Arc::new(RealLinkProbe::default()), home);
}

/// Doctor line controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_link_doctor_line_tests.rs"]
mod tests;
