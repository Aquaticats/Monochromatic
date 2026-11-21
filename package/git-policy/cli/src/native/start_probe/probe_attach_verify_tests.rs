//! What:
//!  Controls of the read-back's calls on a machine that mounts nothing:
//!  a mount the table does not list is named,
//!  and a path that is not there is refused with the open's errno.
//! Why:
//!  The arm must refuse a mount it did not make rather than reading whatever
//!  is at the path,
//!  and its refusals carry the call's errno to the record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(verifyAfterMove('/no/such/path', 1, 'x').errno).toBe(ENOENT);
//! ```

use super::verify_after_move;
use crate::testing::test_support::fixture;
use std::path::{Path, PathBuf};

/// A mount ID the worker's own table does not list is refused as unlisted,
/// never as a pass.
#[test]
fn an_unlisted_mount_id_is_refused() {
    let reading = verify_after_move(Path::new("/no/such/slot"), u64::MAX, "pool/ds@ready").expect_err("the mount is not listed");
    let said: String = reading.reason.clone();
    assert!(said.contains("no line for it"), "{said}");
}

/// A listed mount whose path is not there refuses with the open's errno.
#[test]
fn a_mount_whose_path_is_missing_is_refused_with_its_errno() {
    let root: PathBuf = fixture("start-probe-readback");
    let missing: PathBuf = root.join("no-such-mount-point");
    let refused = verify_after_move(missing.as_path(), 1, "pool/ds@ready");
    assert!(refused.is_err(), "a path that is not there is refused");
}
