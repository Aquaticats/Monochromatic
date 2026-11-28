//! What:
//!  This boot's identifier,
//!  which every record of the handler's and every receipt is judged against.
//! Why:
//!  A readiness record and a receipt name the boot they were published in,
//!  and a record from an earlier boot admits nothing and answers no request
//!  (`crate::worktree_clone_mount_protocol::clone_mount_readiness`;
//!  plan step 18,
//!  "the latest readiness and unresolved requests").
//!  The kernel's answer is the only source,
//!  so it is read here once and both checks compare against the one string.
//!  A boot identifier that could not be read leaves nothing to compare
//!  against,
//!  and a comparison against an empty string would report every record of this
//!  boot as another boot's,
//!  so the two checks answer unknown and the report says why.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const boot = theLiveBoot({ places, unread }); // 'a0b1…' or ''
//! ```

/// The places this boot's identifier is named by.
use super::the_route_places::TheRoutePlaces;
/// The unknown a read that failed answers.
use super::the_unknown_a_failed_read_answers::the_unknowns_of;
/// One check's stable name,
/// and the report line a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};

/// What:
///  The two checks that compare a record against this boot.
/// Why:
///  A readiness record and a receipt each name a boot,
///  and neither comparison holds without one,
///  so both answer unknown when the kernel names no boot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_CHECKS_OF_THE_BOOT = ['readiness', 'unresolved-requests'];
/// ```
const THE_CHECKS_OF_THE_BOOT: &[Check] = &[Check::Readiness, Check::UnresolvedRequests];

/// The longest identifier the kernel's answer may hold,
/// which is one UUID line and its terminator.
const BOOT_IDENTIFIER_LIMIT: u64 = 128;

/// What:
///  This boot's identifier as the kernel writes it,
///  without its terminator,
///  or nothing when the answer is not an identifier or could not be read.
/// Why:
///  Every record the report judges is compared with this one string,
///  so a whitespace or a terminator left in it would report this boot's own
///  records as another boot's.
///  The read is bounded because it is a kernel file a boot cannot outgrow,
///  and a read that gave more is not an answer this route knows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const boot = theLiveBoot({ places, unread }); // the identifier, or '' with two unknowns pushed
/// ```
pub(crate) fn the_live_boot(
    places: &TheRoutePlaces,
    unread: &mut Vec<CheckReport>,
) -> String {
    let path: &std::path::Path = places.boot_identity.as_path();
    let read: Result<String, String> = read_one_line(path);
    return match read {
        Ok(identifier) => identifier,
        Err(why) => {
            // What:     `extend` appends one unknown per check that compares a record against this boot.
            // Why:      An empty identifier would make every record of this boot look like another boot's.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // unread.push(...theUnknownsOf(THE_CHECKS_OF_THE_BOOT, why));
            // ```
            unread.extend(the_unknowns_of(THE_CHECKS_OF_THE_BOOT, why.as_str()));
            String::new()
        }
    };
}

/// What:
///  One bounded read of a file the kernel writes,
///  with the terminator removed.
/// Why:
///  A boot identifier is one line,
///  and a file that grew past a UUID or holds no line is not one this route
///  knows,
///  so the answer is a refusal naming the file rather than a truncated
///  identifier two records would be compared against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identifier = readOneLine('/proc/sys/kernel/random/boot_id'); // 'a0b1…' | throws
/// ```
fn read_one_line(path: &std::path::Path) -> Result<String, String> {
    let bytes: Vec<u8> = crate::foundation::plain_file::read_plain_file(path, BOOT_IDENTIFIER_LIMIT)
        .map_err(|error| {
            return format!(
                "this boot's identifier could not be read from {} ({error}), so no record can be compared with this boot",
                path.display()
            );
        })?;
    let text: &str = std::str::from_utf8(bytes.as_slice()).map_err(|error| {
        return format!(
            "this boot's identifier in {} is not text ({error}), so no record can be compared with this boot",
            path.display()
        );
    })?;
    let identifier: &str = text.trim();
    if identifier.is_empty() {
        return Err(format!(
            "this boot's identifier in {} holds no text, so no record can be compared with this boot",
            path.display()
        ));
    }
    return Ok(String::from(identifier));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_boot_identity_read_tests.rs"]
mod tests;
