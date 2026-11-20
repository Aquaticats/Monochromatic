//! What:
//!  The device-free proof that a checked name refuses empty text,
//! an interior
//!  NUL and text one byte too long for its field,
//! and that an accepted name is
//!  copied into the field with its terminating NUL.
//! Why:
//!  Truncating instead of refusing is the failure that retargets a write;
//! the
//!  boundary cases (exactly maximum minus one byte,
//! exactly maximum) are where
//!  an off-by-one would hide.

use super::*;
use crate::layout::ZfsCmd;

#[test]
fn an_empty_name_is_refused_with_einval() {
    let error: NameError =
        CheckedName::new(NameField::CommandName, "").expect_err("empty is refused");
    assert_eq!(error.field, NameField::CommandName);
    assert_eq!(error.problem, NameProblem::Empty);
    assert_eq!(error.errno(), libc::EINVAL);
    assert!(error.to_string().contains("zc_name may not be empty"));
}

#[test]
fn an_interior_nul_is_refused_with_its_offset() {
    let error: NameError =
        CheckedName::new(NameField::CommandName, "pool/a\0b").expect_err("a NUL is refused");
    assert_eq!(error.problem, NameProblem::InteriorNul { at: 6 });
    assert_eq!(error.errno(), libc::EINVAL);
    assert!(error.to_string().contains("NUL byte at offset 6"));
}

#[test]
fn exactly_one_byte_too_long_is_refused_with_enametoolong() {
    let too_long: String = "a".repeat(NameField::CommandString.max_bytes());
    let error: NameError = CheckedName::new(NameField::CommandString, &too_long)
        .expect_err("a name as long as the field leaves no room for the NUL");
    assert_eq!(
        error.problem,
        NameProblem::TooLong {
            length: NameField::CommandString.max_bytes(),
            maximum: NameField::CommandString.max_bytes()
        }
    );
    assert_eq!(error.errno(), libc::ENAMETOOLONG);
    assert!(error.to_string().contains("zc_string"));
}

#[test]
fn the_longest_fitting_name_is_accepted() {
    let longest: String = "a".repeat(NameField::CommandString.max_bytes() - 1);
    let checked: CheckedName = CheckedName::new(NameField::CommandString, &longest)
        .expect("maximum minus one fits with its NUL");
    assert_eq!(checked.as_str(), longest);
    assert_eq!(checked.field(), NameField::CommandString);
}

#[test]
fn an_accepted_name_is_copied_with_its_terminating_nul() {
    let checked: CheckedName =
        CheckedName::new(NameField::CommandName, "pool/dataset").expect("a short name fits");
    // SAFETY: an all-zero bit pattern is a valid ZfsCmd.
    let mut command: ZfsCmd = unsafe { std::mem::zeroed() };
    checked.write_into(&mut command);
    assert_eq!(&command.zc_name[..12], b"pool/dataset");
    assert_eq!(command.zc_name[12], 0);
    assert_eq!(command.name(), "pool/dataset");
}

#[test]
fn a_value_name_lands_in_zc_value_and_a_string_in_zc_string() {
    let value: CheckedName = CheckedName::new(NameField::CommandValue, "renamed").expect("fits");
    let string: CheckedName = CheckedName::new(NameField::CommandString, "origin").expect("fits");
    // SAFETY: an all-zero bit pattern is a valid ZfsCmd.
    let mut command: ZfsCmd = unsafe { std::mem::zeroed() };
    value.write_into(&mut command);
    string.write_into(&mut command);
    assert_eq!(command.value(), "renamed");
    assert_eq!(command.string(), "origin");
    assert_eq!(command.name(), "");
}

#[test]
fn a_name_error_is_a_standard_error_with_the_field_and_bound() {
    let error: NameError = CheckedName::new(NameField::CommandString, &"x".repeat(300))
        .expect_err("300 bytes do not fit 256");
    let as_error: &dyn std::error::Error = &error;
    let text: String = as_error.to_string();
    assert!(text.contains("zc_string holds at most 255 bytes"));
    assert!(text.contains("300 bytes"));
}
