//! What:
//!  One element a write request reported as failed,
//! with the name the kernel
//!  keyed it by and its own error number,
//! and the reader that turns a reply
//!  list into those elements.
//! Why:
//!  A batch such as `DESTROY_SNAPS` fails as a whole while succeeding or
//!  failing per snapshot name,
//! and a caller that only saw the ioctl's error
//!  number could not tell which snapshots survived.
//!  The reply is kernel-written and untrusted:
//! a nested list that is not the
//!  `{value, source}` property shape could recurse without bound,
//! so the
//!  reader reads any other nested list in turn,
//! bounded by the codec's depth
//!  limit of 20 for a decoded reply,
//! caps how many names it keeps,
//! and ignores a zero error number.

/// The codec list and value types.
use crate::nvlist::{NvData, NvList};
/// The formatting traits the diagnostics implement.
use std::fmt;

/// Most elements kept from one reply.
///
/// A batch's names come from the caller's own request,
/// so four (the number a
/// diagnostic shows) is too few to report a failed set,
/// while an untrusted
/// reply must not grow the error without bound.
pub const MAXIMUM_ELEMENTS_KEPT: usize = 64;

/// The number of elements a diagnostic prints before summarising the rest.
pub const ELEMENTS_SHOWN: usize = 4;

/// One failed element of a write request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementError {
    /// The snapshot name,
    /// GUID,
    /// or property the kernel keyed the failure by.
    pub name: String,
    /// The kernel's error number for that element.
    pub errno: i32,
}

/// How the value prints to a person.
impl fmt::Display for ElementError {
    /// Prints the element name with its error text.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}: {}", self.name, errno_text(self.errno));
    }
}

/// Read the per-name errors out of a write reply.
///
/// A value under a name is the element's error number (`i32`,
/// `i64` or `u64`).
/// A nested list shaped `{value, source}` is a property error,
/// whose `value` is
/// the number.
/// Any other nested list is read in turn.
/// A zero number means that
/// element succeeded.
pub fn element_errors(list: &NvList) -> Vec<ElementError> {
    let mut elements: Vec<ElementError> = Vec::new();
    collect(list, &mut elements);
    return elements;
}

/// Walk `list` for element errors,
/// stopping at [`MAXIMUM_ELEMENTS_KEPT`].
fn collect(list: &NvList, elements: &mut Vec<ElementError>) {
    for pair in list.iter() {
        if elements.len() >= MAXIMUM_ELEMENTS_KEPT {
            return;
        }
        let errno: i32 = match &pair.data {
            NvData::Int32 { value } => *value,
            NvData::Int64 { value } => *value as i32,
            NvData::Uint64 { value } => *value as i32,
            NvData::List { value: nested } => {
                if let Some(value) = property_error_value(nested) {
                    value
                } else {
                    collect(nested, elements);
                    continue;
                }
            }
            _ => continue,
        };
        if errno != 0 {
            elements.push(ElementError {
                name: pair.name.clone(),
                errno,
            });
        }
    }
}

/// The `value` of a `{value, source}` property error,
/// if `nested` is one.
fn property_error_value(nested: &NvList) -> Option<i32> {
    let is_property_error: bool = nested.iter().all(|pair| {
        return pair.name == "value" || pair.name == "source";
    });
    if !is_property_error {
        return None;
    }
    return nested.get_u64("value").map(|value| {
        return value as i32;
    });
}

/// The text `strerror` gives an error number,
/// or the number itself for ZFS's
/// private range,
/// which `strerror` does not know.
fn errno_text(errno: i32) -> String {
    let error: std::io::Error = std::io::Error::from_raw_os_error(errno);
    let text: String = error.to_string();
    return text;
}

/// The suffix a write error's message adds for its elements,
/// empty when there
/// are none.
pub fn elements_suffix(elements: &[ElementError]) -> String {
    if elements.is_empty() {
        return String::new();
    }
    let mut shown: Vec<String> = elements
        .iter()
        .take(ELEMENTS_SHOWN)
        .map(|element| {
            return element.to_string();
        })
        .collect();
    let more: usize = elements.len().saturating_sub(ELEMENTS_SHOWN);
    if more > 0 {
        shown.push(format!("+{more} more"));
    }
    return format!("; {}", shown.join("; "));
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "element_error_tests.rs"]
mod tests;
