//! Controls for the pool calls that need no pool:
//!  a write's per-name failure is reported with its name and error,
//!  and a GUID the pool did not report is refused.

use super::{ObjectIdentity, guid_of, refuse_any_element};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

#[test]
fn a_per_name_failure_is_named_with_its_error() {
    let mut reply: NvList = NvList::new();
    reply.add_i32("tank/clones/topic", 28);
    let refusal: String = refuse_any_element(Some(&reply), "the clone").expect_err("a failed element refuses the write");
    assert!(refusal.contains("tank/clones/topic"), "{refusal}");
    assert!(refusal.contains("No space left on device"), "{refusal}");
    assert!(refusal.contains("Ways forward:"), "{refusal}");
    assert_eq!(refuse_any_element(Some(&NvList::new()), "the clone"), Ok(()), "an empty reply is a success");
    assert_eq!(refuse_any_element(None, "the clone"), Ok(()), "no reply is a success");
}

#[test]
fn a_guid_of_zero_is_refused() {
    let refusal: String = guid_of(0, "tank/clones/topic").expect_err("no live object answers GUID 0");
    assert!(refusal.contains("GUID 0"), "{refusal}");
    assert!(guid_of(7, "tank/clones/topic").is_ok());
    let identity: ObjectIdentity = ObjectIdentity { guid: guid_of(7, "tank/clones/topic").expect("the guid"), createtxg: 9 };
    assert_eq!(identity.guid.file_name(), "7");
    assert_eq!(identity.createtxg, 9);
}
