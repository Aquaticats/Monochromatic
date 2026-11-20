//! What:
//!  The device-free proof that each name field's bound is the one the kernel's
//!  struct gives it,
//! and that a diagnostic names the C field.
//! Why:
//!  A bound one byte off would admit a name the kernel truncates,
//! which is the
//!  failure mode that can retarget a write to a different dataset.

use super::*;
use crate::layout::{MAX_VALUE_LEN, MAXNAMELEN, MAXPATHLEN};

#[test]
fn each_field_reports_the_kernel_struct_bound() {
    assert_eq!(NameField::CommandName.max_bytes(), MAXPATHLEN);
    assert_eq!(NameField::CommandValue.max_bytes(), MAX_VALUE_LEN);
    assert_eq!(NameField::CommandString.max_bytes(), MAXNAMELEN);
    assert_eq!(MAX_VALUE_LEN, MAXPATHLEN * 2);
}

#[test]
fn each_field_reports_its_c_name() {
    assert_eq!(NameField::CommandName.as_str(), "zc_name");
    assert_eq!(NameField::CommandValue.as_str(), "zc_value");
    assert_eq!(NameField::CommandString.as_str(), "zc_string");
}

#[test]
fn the_name_maximum_is_the_struct_size_the_design_quotes() {
    assert_eq!(MAXPATHLEN, 4096);
    assert_eq!(MAXNAMELEN, 256);
}
