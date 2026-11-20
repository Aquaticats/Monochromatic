//! What:
//!  The readers the live tests use on a kernel answer:
//! one property's value and
//!  source out of an identity reply,
//! one bookmark's identity,
//! every name a
//!  listing returns,
//! and the dataset name a `zfs get` source stands for.
//! Why:
//!  The reply shapes are the kernel's,
//! not this crate's,
//! so a test that read them
//!  inline would spread the same assumptions through every arm;
//! one reader with
//!  the shape's source beside it keeps them in one place.

// The live failure type the listing reader passes through.
use super::device_error::DeviceError;
// One step of a listing.
use super::listed_object::ListedObject;
// The decoded reply list.
use crate::nvlist::NvList;

/// The numeric value of `property` in an `OBJSET_STATS` reply.
///
/// The kernel nests each property as `{value, source}`
/// (`dsl_prop_get_all_impl`,
/// `module/zfs/dsl_prop.c` of `zfs-2.4.4`),
/// and `dmu_objset_stats` adds its
/// own numeric entries the same way.
///
/// # Panics
///
/// When the reply has no such property or it is not a number.
pub(crate) fn property_u64(reply: &NvList, property: &str) -> u64 {
    let entry: &NvList = reply.get_list(property).unwrap_or_else(|| {
        panic!("the reply carries no '{property}'");
    });
    return entry.get_u64("value").unwrap_or_else(|| {
        panic!("'{property}' is not a number: {entry:?}");
    });
}

/// The string value of `property` in an `OBJSET_STATS` reply.
///
/// # Panics
///
/// When the reply has no such property or it is not a string.
pub(crate) fn property_str<'a>(reply: &'a NvList, property: &str) -> &'a str {
    let entry: &NvList = reply.get_list(property).unwrap_or_else(|| {
        panic!("the reply carries no '{property}'");
    });
    return entry.get_str("value").unwrap_or_else(|| {
        panic!("'{property}' is not a string");
    });
}

/// The source string `OBJSET_STATS` reports for `property`.
///
/// The kernel writes the dataset the value was set on,
/// `received`,
/// or a
/// source keyword;
/// `zfs get` then prints it as `local`,
/// `inherited from <dataset>`
/// or the keyword.
///
/// # Panics
///
/// When the reply has no such property.
pub(crate) fn property_source<'a>(reply: &'a NvList, property: &str) -> &'a str {
    let entry: &NvList = reply.get_list(property).unwrap_or_else(|| {
        panic!("the reply carries no '{property}'");
    });
    return entry.get_str("source").unwrap_or_else(|| {
        panic!("'{property}' has no source");
    });
}

/// The dataset the kernel's source string stands for,
/// read from what `zfs get`
/// prints for the same property.
///
/// `local` is the dataset the property was read from.
///
/// # Panics
///
/// When `zfs get` printed a source this reader does not know.
pub(crate) fn expected_source(cli_source: &str, local: &str) -> String {
    if cli_source == "local" {
        return local.to_string();
    }
    if let Some(dataset) = cli_source.strip_prefix("inherited from ") {
        return dataset.to_string();
    }
    assert!(
        cli_source == "received" || cli_source == "default" || cli_source == "-",
        "zfs get printed an unknown source '{cli_source}'"
    );
    return cli_source.to_string();
}

/// Every name a listing returns,
/// asking each time with the cursor the kernel wrote.
///
/// # Panics
///
/// When a step fails.
pub(crate) fn listing_names<F>(mut step: F) -> Vec<String>
where
    F: FnMut(u64) -> Result<Option<ListedObject>, DeviceError>,
{
    let mut names: Vec<String> = Vec::new();
    let mut cursor: u64 = 0;
    let mut steps: usize = 0;
    loop {
        let listed: Option<ListedObject> = step(cursor).expect("the listing step is answered");
        let Some(object) = listed else {
            return names;
        };
        cursor = object.cursor;
        names.push(object.name);
        steps += 1;
        assert!(steps < 10_000, "the listing never reached its end");
    }
}

/// The identity a bookmark reply reports for `name`,
/// as the GUID and
/// creation transaction group the kernel keeps.
///
/// # Panics
///
/// When the reply has no such bookmark.
pub(crate) fn bookmark_identity(reply: &NvList, name: &str) -> (u64, u64) {
    let entry: &NvList = reply.get_list(name).unwrap_or_else(|| {
        panic!("the reply carries no bookmark '{name}'");
    });
    // `dsl_prop_nvlist_add_uint64` nests each property as `{value}`
    // (`module/zfs/dsl_prop.c` of `zfs-2.4.4`), the same shape `dsl_prop_get_all`
    // uses for a dataset's properties.
    let guid: u64 = entry
        .get_list("guid")
        .and_then(|value| {
            return value.get_u64("value");
        })
        .expect("a bookmark has a GUID");
    let txg: u64 = entry
        .get_list("createtxg")
        .and_then(|value| {
            return value.get_u64("value");
        })
        .expect("a bookmark has a creation transaction group");
    return (guid, txg);
}

/// The exact value `zfs get` prints for `property` of `dataset`.
///
/// # Panics
///
/// When `zfs get` printed no such property or its value is not a number.
pub(crate) fn cli_value(fixture: &super::fixture::Fixture, dataset: &str, property: &str) -> u64 {
    return cli_value_and_source(fixture, dataset, property).0;
}

/// The exact value and the source `zfs get` prints for `property` of
/// `dataset`.
///
/// # Panics
///
/// When `zfs get` printed no such property or its value is not a number.
pub(crate) fn cli_value_and_source(
    fixture: &super::fixture::Fixture,
    dataset: &str,
    property: &str,
) -> (u64, String) {
    let rows = fixture.property_rows(dataset, property);
    let Some((_, value, source)) = rows.first() else {
        panic!("zfs get printed no '{property}' for {dataset}");
    };
    let number: u64 = value.parse().unwrap_or_else(|_| {
        panic!("zfs get printed '{value}', which is not a number, for '{property}'");
    });
    return (number, source.clone());
}
