//! What:
//!  The request contract:
//! every number this crate can send,
//! and the range's two
//!  endpoints,
//! compared with the loaded module's `enum zfs_ioc`.
//! Why:
//!  A request sent under another number arrives at another handler,
//! and a renumbered
//!  request is the one difference no struct size or member offset shows,
//! so the
//!  numbers are compared even when every layout fact matches.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! compareRequests(zfsIocId); // throws Mismatch at the first difference
//! ```

/// The one difference value this walk returns.
use super::mismatch::Mismatch;
/// The expectation's request table and the enumeration's name and width.
use super::expected_requests::{REQUEST_ENUM, REQUEST_ENUM_BYTES, expected_enumerators};
/// The kinds one enumeration is,
/// and the indexed type entries.
use super::btf_entry::ENUM_KINDS;
/// The indexed type entries.
use super::btf_types::TypeInformation;
/// The step one difference is reported as.
use super::refusal::Step;

/// Compare every request number this crate can send with the module's
/// `enum zfs_ioc`.
///
/// # Errors
/// A mismatch naming the enumerator and both values.
pub(super) fn compare_requests(module: &TypeInformation<'_>) -> Result<(), Mismatch> {
    let found: Option<u32> = module.find(REQUEST_ENUM, &ENUM_KINDS);
    let Some(id): Option<u32> = found else {
        return Err(Mismatch {
            step: Step::Enum,
            detail: format!("the loaded module holds no enumeration named {REQUEST_ENUM}"),
        });
    };
    if module.byte_size(id) != Some(u64::from(REQUEST_ENUM_BYTES)) {
        return Err(Mismatch {
            step: Step::Enum,
            detail: format!(
                "{REQUEST_ENUM} is not {REQUEST_ENUM_BYTES} bytes long, and its range's numbers would be read at another width"
            ),
        });
    }
    let Some(values): Option<Vec<(String, i64)>> = module.enumeration_values(id) else {
        return Err(Mismatch {
            step: Step::Enum,
            detail: format!("{REQUEST_ENUM}'s values could not be read"),
        });
    };
    for wanted in expected_enumerators() {
        let Some(actual): Option<&(String, i64)> = values.iter().find(|(name, _)| {
            return *name == wanted.name;
        }) else {
            return Err(Mismatch {
                step: Step::Enum,
                detail: format!("{} is missing from {REQUEST_ENUM}", wanted.name),
            });
        };
        if actual.1 != wanted.value {
            return Err(Mismatch {
                step: Step::Enum,
                detail: format!(
                    "{} is {} where {} was expected",
                    wanted.name, actual.1, wanted.value
                ),
            });
        }
    }
    return Ok(());
}
