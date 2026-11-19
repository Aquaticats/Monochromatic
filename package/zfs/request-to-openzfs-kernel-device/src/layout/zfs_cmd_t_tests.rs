//! What:
//!  The device-free proof that the mirrored `zfs_cmd_t` sits where the loaded
//!  module's BTF says it does on OpenZFS `zfs-2.4.4`,
//! and that the union's
//!  control fields and the fixed name fields read and write the bytes they name.
//! Why:
//!  A field at another offset would turn a destroy into another request's
//!  arguments;
//! the const assertions in the module already fail the build,
//! and
//!  these tests state the numbers the design quotes so a reader can compare
//!  them with `pahole`'s output without reading the assertions.

use super::*;
use std::mem::{align_of, offset_of, size_of};

#[test]
fn struct_size_is_the_module_btf_size() {
    assert_eq!(size_of::<ZfsCmd>(), 13_744);
    assert_eq!(align_of::<ZfsCmd>(), 8);
}

#[test]
fn union_and_trailing_fields_sit_at_their_btf_offsets() {
    assert_eq!(offset_of!(ZfsCmd, zc_inject_overlay), 13_296);
    assert_eq!(size_of::<ZinjectRecord>(), 368);
    assert_eq!(offset_of!(ZfsCmd, zc_cleanup_fd), 13_664);
    assert_eq!(offset_of!(ZfsCmd, zc_zoneid), 13_736);
}

#[test]
fn the_three_control_fields_are_where_the_design_names_them() {
    let base: usize = offset_of!(ZfsCmd, zc_inject_overlay);
    assert_eq!(
        base + offset_of!(ZcDestroyControl, zc_defer_destroy),
        13_648
    );
    assert_eq!(base + offset_of!(ZcDestroyControl, zc_flags), 13_652);
    assert_eq!(
        base + offset_of!(ZcDestroyControl, zc_action_handle),
        13_656
    );
}

#[test]
fn the_defer_mark_writes_only_its_own_bytes() {
    let mut command: ZfsCmd = zeroed();
    command.zc_cleanup_fd = -7;
    command.zc_zoneid = 0xDEAD_BEEF;
    command.set_defer_destroy(true);
    assert_eq!(command.defer_destroy(), 1);
    assert_eq!(command.flags(), 0);
    assert_eq!(command.action_handle(), 0);
    assert_eq!(command.zc_cleanup_fd, -7);
    assert_eq!(command.zc_zoneid, 0xDEAD_BEEF);
    command.set_defer_destroy(false);
    assert_eq!(command.defer_destroy(), 0);
}

#[test]
fn a_name_field_reads_back_up_to_its_nul() {
    let mut command: ZfsCmd = zeroed();
    command.zc_name[..6].copy_from_slice(b"pool/a");
    assert_eq!(command.name(), "pool/a");
}

#[test]
fn a_field_without_a_nul_reads_back_empty_rather_than_panicking() {
    let mut command: ZfsCmd = zeroed();
    command.zc_name.fill(b'x');
    assert_eq!(command.name(), "");
    command.zc_string.fill(b'y');
    assert_eq!(command.string(), "");
}

#[test]
fn invalid_utf8_in_a_field_is_replaced_not_refused() {
    let mut command: ZfsCmd = zeroed();
    command.zc_value[..2].copy_from_slice(&[0xff, 0xfe]);
    command.zc_value[2] = 0;
    assert_eq!(command.value(), "\u{fffd}\u{fffd}");
}

/// A `ZfsCmd` whose every byte is zero,
/// the state the kernel expects.
fn zeroed() -> ZfsCmd {
    // SAFETY: ZfsCmd is a plain-integer struct, so an all-zero bit pattern is
    // a valid value.
    return unsafe { std::mem::zeroed() };
}
