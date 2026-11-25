//! What:
//!  The controls of the canonical decimal spelling:
//!  a value the grammar reads,
//!  and each spelling it refuses.
//! Why:
//!  Two spellings of one GUID would be two records of one object,
//!  so the parse is the guard every number in a plan passes
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "What every reader refuses").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(isCanonicalDecimal('007', 20)).toBe(false);
//! ```

use super::{decimal_digits, is_canonical_decimal, parse_canonical_u64};

/// The digit bound a plan's `uint64` fields take.
const BOUND: usize = 20;

#[test]
fn a_canonical_number_is_read_and_a_repeated_or_signed_one_is_not() {
    assert_eq!(parse_canonical_u64("0", BOUND), Some(0));
    assert_eq!(parse_canonical_u64("1", BOUND), Some(1));
    assert_eq!(parse_canonical_u64("7828944690444299989", BOUND), Some(7_828_944_690_444_299_989));
    assert_eq!(parse_canonical_u64(u64::MAX.to_string().as_str(), BOUND), Some(u64::MAX));
    for refused in ["", "00", "007", "+1", "-1", "1 ", " 1", "1.0", "0x10", "١"] {
        assert!(!is_canonical_decimal(refused, BOUND), "`{refused}` is not a canonical decimal");
        assert_eq!(parse_canonical_u64(refused, BOUND), None, "`{refused}` parses to nothing");
    }
}

#[test]
fn a_number_wider_than_the_grammar_allows_is_refused_before_it_is_parsed() {
    let wide: String = format!("1{}", "0".repeat(BOUND));
    assert!(!is_canonical_decimal(wide.as_str(), BOUND), "21 digits are past the bound");
    assert_eq!(parse_canonical_u64(wide.as_str(), BOUND), None);
}

#[test]
fn the_digit_count_is_one_for_zero_and_grows_by_one_at_each_power_of_ten() {
    assert_eq!(decimal_digits(0), 1);
    assert_eq!(decimal_digits(9), 1);
    assert_eq!(decimal_digits(10), 2);
    assert_eq!(decimal_digits(99), 2);
    assert_eq!(decimal_digits(100), 3);
    assert_eq!(decimal_digits(u64::MAX), BOUND);
}
