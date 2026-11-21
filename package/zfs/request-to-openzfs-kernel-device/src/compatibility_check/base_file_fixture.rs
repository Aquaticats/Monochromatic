//! What:
//!  The one module file a control builds over a base file the module does not
//! own:
//!  its `zc_simple` member is the base file's own integer,
//! written under the base file's own
//!  name,
//! and the pair is the gate's two files.
//! Why:
//!  A split module numbers its ids and its name offsets past its base's,
//! and
//!  `btf-rs` answers a handle from either file,
//! so an adapter that read the module's own
//!  section alone would refuse this pair.
//!  The base file's integer and the name are one entry there,
//!  which is why one call makes both.

/// The writer of the two files.
use super::blob_fixture::Blob;
/// The approved declaration over the base file,
/// with one member moved into it.
use super::approved_module_fixture::module_over_a_member;
/// The base file's integer width,
/// which is the approved `zc_simple`'s.
use super::approved_module_member_fixture::BYTE_BYTES;

/// The byte member's name,
/// which the base file holds and the module's member points at.
const MEMBER_NAME: &str = "zc_simple";

/// The base file and the approved module whose `zc_simple` member is the base
/// file's own integer,
/// named by the base file's own string.
pub fn module_typed_and_named_from_a_base_file() -> (Blob, Blob) {
    let mut base: Blob = Blob::new();
    let byte: u32 = base.int(MEMBER_NAME, BYTE_BYTES);
    let name_offset: u32 = base.name_offset(MEMBER_NAME).expect("the base's own name");
    let module: Blob = module_over_a_member(&base, MEMBER_NAME, byte, name_offset);
    return (base, module);
}
