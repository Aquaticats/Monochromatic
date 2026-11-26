//! What:
//!  Controls of the dataset GUID name:
//!  the canonical decimal spelling,
//!  its one-name-per-dataset rule,
//!  and each refusal.
//! Why:
//!  The GUID is the receipt's file name,
//!  so a second spelling would leave one dataset with two receipts and a
//!  reader looking at the wrong one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(DatasetGuid.parse('007').kind).toBe('leading-zero');
//! ```

use super::{
    DatasetGuid,
    DatasetGuidRefusal,
};

/// A canonical number reads and writes as itself.
#[test]
fn a_canonical_number_round_trips() {
    for value in [0_u64, 1, 9, 1234567890123456789, u64::MAX] {
        let text: String = value.to_string();
        let guid: DatasetGuid =
            DatasetGuid::parse(text.as_str()).expect("a canonical decimal is a GUID");
        assert_eq!(guid.value(), value);
        assert_eq!(guid.file_name(), text);
    }
    assert_eq!(
        DatasetGuid::parse("0").map(DatasetGuid::file_name),
        Ok(String::from("0"))
    );
}

/// A value that is not decimal digits is refused.
#[test]
fn a_value_that_is_not_digits_is_refused() {
    for value in ["", "-1", "+1", "0x10", "1 2", "1\n", "１２３", "abc"] {
        let refusal: DatasetGuidRefusal = DatasetGuid::parse(value).expect_err("not a number");
        assert!(
            matches!(refusal, DatasetGuidRefusal::NotDigits { .. }),
            "{value}: {refusal:?}"
        );
        assert!(
            refusal.message().contains("Nothing was filed"),
            "{}",
            refusal.message()
        );
    }
}

/// A leading zero is its own refusal,
/// so one dataset has exactly one spelling.
#[test]
fn a_leading_zero_is_refused() {
    assert_eq!(
        DatasetGuid::parse("00"),
        Err(DatasetGuidRefusal::LeadingZero {
            value: String::from("00")
        })
    );
    assert_eq!(
        DatasetGuid::parse("007"),
        Err(DatasetGuidRefusal::LeadingZero {
            value: String::from("007")
        })
    );
}

/// A number too large for a dataset GUID is refused rather than wrapped.
#[test]
fn a_number_beyond_sixty_four_bits_is_refused() {
    let too_large: String = format!("{}0", u64::MAX);
    let refusal: DatasetGuidRefusal =
        DatasetGuid::parse(too_large.as_str()).expect_err("no 64-bit dataset holds it");
    assert_eq!(
        refusal,
        DatasetGuidRefusal::TooLarge {
            value: too_large
        }
    );
    assert!(refusal.message().contains(&u64::MAX.to_string()));
}
