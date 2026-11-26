//! What:
//!  Controls of the one property reader both protocol properties use:
//!  absent,
//!  inherited,
//!  local,
//!  and each unreadable shape.
//! Why:
//!  The local-versus-inherited decision is what keeps an ancestor's intent from
//!  authorizing a mount,
//!  so every shape of the reply has a named answer rather than a guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readLocalProperty(reply, 'pool/clone', 'org...').kind).toBe('inherited');
//! ```

use super::{
    PropertyReading,
    PropertyRefusal,
    read_local_property,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// The property every control reads.
const PROPERTY: &str = "org.monochromatic:git-worktree-clone-mount-desired-state";

/// The dataset the reply describes.
const CLONE: &str = "zpcachyos/nosnap/clones/one";

/// The ancestor a property can be inherited from.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// A reply with the given value and source.
fn reply_with(
    value: Option<&str>,
    source: Option<&str>,
) -> NvList {
    let mut pair: NvList = NvList::new();
    if let Some(word) = value {
        pair.add_str("value", word);
    }
    if let Some(word) = source {
        pair.add_str("source", word);
    }
    let mut reply: NvList = NvList::new();
    reply.add_nvlist(PROPERTY, pair);
    return reply;
}

/// A missing reply and a reply without the property are both absent.
#[test]
fn an_absent_property_is_absent() {
    assert_eq!(
        read_local_property(None, CLONE, PROPERTY),
        PropertyReading::Absent
    );
    assert_eq!(
        read_local_property(Some(&NvList::new()), CLONE, PROPERTY),
        PropertyReading::Absent
    );
}

/// The dataset's own name as source is a local value,
/// and any other source is inherited from that dataset.
#[test]
fn a_local_value_is_told_from_an_inherited_one() {
    let local: NvList = reply_with(Some("v1:00:mount"), Some(CLONE));
    assert_eq!(
        read_local_property(Some(&local), CLONE, PROPERTY),
        PropertyReading::Local {
            value: "v1:00:mount"
        }
    );
    let inherited: NvList = reply_with(Some("v1:00:mount"), Some(PARENT));
    assert_eq!(
        read_local_property(Some(&inherited), CLONE, PROPERTY),
        PropertyReading::Inherited {
            from: String::from(PARENT)
        }
    );
}

/// A source that is not exactly the dataset's name is inherited,
/// never matched by a prefix or a suffix.
#[test]
fn a_source_that_only_resembles_the_dataset_is_inherited() {
    let deeper: String = format!("{CLONE}/child");
    let reply: NvList = reply_with(Some("v1:00:mount"), Some(deeper.as_str()));
    assert!(matches!(
        read_local_property(Some(&reply), CLONE, PROPERTY),
        PropertyReading::Inherited { .. }
    ));
}

/// Each unreadable shape is its own refusal.
#[test]
fn each_unreadable_shape_is_named() {
    let mut bare: NvList = NvList::new();
    bare.add_str(PROPERTY, "v1:00:mount");
    assert_eq!(
        read_local_property(Some(&bare), CLONE, PROPERTY),
        PropertyReading::Malformed(PropertyRefusal::NotAPropertyPair {
            dataset: String::from(CLONE)
        })
    );
    let no_value: NvList = reply_with(None, Some(CLONE));
    assert_eq!(
        read_local_property(Some(&no_value), CLONE, PROPERTY),
        PropertyReading::Malformed(PropertyRefusal::MissingValue)
    );
    let no_source: NvList = reply_with(Some("v1:00:mount"), None);
    assert_eq!(
        read_local_property(Some(&no_source), CLONE, PROPERTY),
        PropertyReading::Malformed(PropertyRefusal::MissingSource)
    );
    let mut number: NvList = NvList::new();
    let mut pair: NvList = NvList::new();
    pair.add_u64("value", 1);
    pair.add_str("source", CLONE);
    number.add_nvlist(PROPERTY, pair);
    assert_eq!(
        read_local_property(Some(&number), CLONE, PROPERTY),
        PropertyReading::Malformed(PropertyRefusal::ValueNotText)
    );
    let mut pair_source: NvList = NvList::new();
    pair_source.add_str("value", "v1:00:mount");
    pair_source.add_u64("source", 1);
    let mut bad_source: NvList = NvList::new();
    bad_source.add_nvlist(PROPERTY, pair_source);
    assert_eq!(
        read_local_property(Some(&bad_source), CLONE, PROPERTY),
        PropertyReading::Malformed(PropertyRefusal::SourceNotText)
    );
}

/// Every refusal names the property and says nothing was done on its behalf.
#[test]
fn every_refusal_names_the_property_and_authorizes_nothing() {
    for refusal in [
        PropertyRefusal::NotAPropertyPair {
            dataset: String::from(CLONE),
        },
        PropertyRefusal::MissingValue,
        PropertyRefusal::MissingSource,
        PropertyRefusal::ValueNotText,
        PropertyRefusal::SourceNotText,
    ] {
        let message: String = refusal.message(PROPERTY);
        assert!(message.contains(PROPERTY), "{message}");
        assert!(message.contains("Nothing was mounted"), "{message}");
    }
}
