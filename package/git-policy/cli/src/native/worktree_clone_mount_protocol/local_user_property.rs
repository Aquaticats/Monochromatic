//! What:
//!  One local user property of a dataset,
//!  read from the reply OpenZFS gives for a property:
//!  a nested list with `value` and `source`,
//!  where a local property's source is the dataset's own name.
//! Why:
//!  Both the durable intent and the sweep nonce live in local user properties,
//!  and the design says to ignore inherited intent and to reject a malformed
//!  one before any filesystem operation on its behalf
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  OpenZFS writes each property in that shape
//!  (`module/zfs/dsl_prop.c:1182` to `:1193`,
//! OpenZFS `zfs-2.4.4`),
//!  and a source equal to the dataset's name is the local one.
//!  One reader serves both properties,
//!  so the three-way distinction of local,
//!  inherited and unreadable is made once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reading = readLocalProperty(reply, cloneName, 'org.monochromatic:...');
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_property_names::{PROPERTY_SOURCE_KEY, PROPERTY_VALUE_KEY};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// What:
///  Why one property could not be read as local,
///  inherited or absent.
/// Why:
///   A reply that does not hold the pair OpenZFS writes cannot prove the
///  property is absent,
///  and guessing either way would authorize the wrong action,
///  so each unreadable shape has its own named cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropertyRefusal = { kind: 'not-a-pair' | 'missing-value' | 'missing-source' | 'value-not-text' | 'source-not-text'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyRefusal {
    /// The reply's entry is not the nested list OpenZFS writes.
    NotAPropertyPair {
        /// The dataset the reply describes.
        dataset: String,
    },
    /// The pair has no `value` key.
    MissingValue,
    /// The pair has no `source` key.
    MissingSource,
    /// The `value` is not text.
    ValueNotText,
    /// The `source` is not text.
    SourceNotText,
}

impl PropertyRefusal {
    /// The refusal in words,
    /// naming the property and the shape that was found,
    /// and saying that nothing was done on its behalf.
    pub fn message(&self, property: &str) -> String {
        let reason: String = match self {
            PropertyRefusal::NotAPropertyPair { dataset } => format!(
                "the value of `{property}` on `{dataset}` is not the pair of `{PROPERTY_VALUE_KEY}` and `{PROPERTY_SOURCE_KEY}` that OpenZFS returns for a property, so this route cannot tell a local value from an inherited one"
            ),
            PropertyRefusal::MissingValue => format!("the value of `{property}` has no `{PROPERTY_VALUE_KEY}` key"),
            PropertyRefusal::MissingSource => format!(
                "the value of `{property}` has no `{PROPERTY_SOURCE_KEY}` key, so this route cannot tell whether the clone set it or inherited it"
            ),
            PropertyRefusal::ValueNotText => format!("the `{PROPERTY_VALUE_KEY}` of `{property}` is not text"),
            PropertyRefusal::SourceNotText => format!("the `{PROPERTY_SOURCE_KEY}` of `{property}` is not text"),
        };
        return format!(
            "{reason}. Nothing was mounted, unmounted, attached or recorded on its behalf. Ways forward: read the property again, as `zfs get -H -o value,source {property} <clone>` prints it, or unset and set it again in the designed form."
        );
    }
}

/// What:
///  What one property says about the dataset that carries it.
/// Why:
///   The three cases the design names are separate values,
///  and a malformed reply is its own case rather than an absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropertyReading = { kind: 'absent' } | { kind: 'inherited'; from } | { kind: 'local'; value } | { kind: 'malformed'; refusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyReading<'a> {
    /// The dataset carries no such property at all.
    Absent,
    /// The property's source is another dataset.
    Inherited {
        /// The dataset the property is set on.
        from: String,
    },
    /// The property's source is the dataset itself.
    Local {
        /// The value exactly as the kernel wrote it.
        value: &'a str,
    },
    /// The reply could not prove which of the three the property is.
    Malformed(PropertyRefusal),
}

/// What:
///  Read one property of a dataset from a property reply.
/// Why:
///   The two properties this protocol reads share this one decision,
///  and a local source is compared with the dataset's own full name,
///  which is what OpenZFS returns as `source` for a value set on that
///  dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readLocalProperty(properties, 'pool/data', 'org...state');
/// ```
pub fn read_local_property<'a>(properties: Option<&'a NvList>, dataset: &str, property: &str) -> PropertyReading<'a> {
    let Some(reply) = properties else {
        return PropertyReading::Absent;
    };
    if reply.get(property).is_none() {
        return PropertyReading::Absent;
    }
    let Some(pair) = reply.get_list(property) else {
        return PropertyReading::Malformed(PropertyRefusal::NotAPropertyPair { dataset: String::from(dataset) });
    };
    let Some(value) = pair.get_str(PROPERTY_VALUE_KEY) else {
        return PropertyReading::Malformed(if pair.get(PROPERTY_VALUE_KEY).is_some() {
            PropertyRefusal::ValueNotText
        } else {
            PropertyRefusal::MissingValue
        });
    };
    let Some(source) = pair.get_str(PROPERTY_SOURCE_KEY) else {
        return PropertyReading::Malformed(if pair.get(PROPERTY_SOURCE_KEY).is_some() {
            PropertyRefusal::SourceNotText
        } else {
            PropertyRefusal::MissingSource
        });
    };
    if source == dataset {
        return PropertyReading::Local { value };
    }
    return PropertyReading::Inherited { from: String::from(source) };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "local_user_property_tests.rs"]
mod tests;
