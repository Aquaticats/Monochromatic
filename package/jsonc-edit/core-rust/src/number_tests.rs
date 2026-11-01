//! What:
//!      Exact-number identity tests:
//!  equal spellings,
//!  distinct values,
//!  invalid tokens,
//!  huge
//!           exponents,
//!  hashing and an independently computed rational oracle.
//! Why:
//!       The identity is what makes `1`,
//!  `1.0` and `1e0` compare equal while
//!           `9007199254740993` stays distinct,
//!  so its edges need explicit coverage.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('JsoncNumberIdentity', () => { /* equality, rejection, oracle */ });
//! ```

/// What:
///      Import the identity under test.
/// Why:
///       These tests exercise the same public constructor a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncNumberIdentity } from './number';
/// ```
use crate::number::JsoncNumberIdentity;

/// Check equivalent JSON spellings,
///  huge exponents,
///  and signed zero.
#[test]
fn equal_values() {
    for (left, right) in [
        ("1", "1.0"), ("1e0", "1"), ("100", "1e2"),
        ("0.00100", "1e-3"), ("0.1e+1", "1"), ("1e-00000", "1"),
        ("-0e+123456789012345678901", "0.000"),
        ("1e999999999999999999999999", "10e999999999999999999999998"),
        ("1e-999999999999999999999999", "10e-1000000000000000000000000"),
        ("1.2000e+2", "120"), ("1E+0000000000000000000", "1"),
    ] {
        let left_value = JsoncNumberIdentity::from_token(left).expect("valid left number");
        let right_value = JsoncNumberIdentity::from_token(right).expect("valid right number");
        assert_eq!(left_value, right_value, "{left} vs {right}");
    }
}

/// Check adjacent large integers and opposed huge exponents remain distinct.
#[test]
fn unequal_values() {
    for (left, right) in [
        ("9007199254740992", "9007199254740993"),
        ("1e99999999999999999999", "1e-99999999999999999999"),
        ("1002", "12"),
        ("1e-999999999999999999999999", "0"),
        ("1.00000000000000000000000001", "1"),
        ("1e1000000000000000000000", "1e1000000000000000000001"),
    ] {
        let left_value = JsoncNumberIdentity::from_token(left).expect("valid left number");
        let right_value = JsoncNumberIdentity::from_token(right).expect("valid right number");
        assert_ne!(left_value, right_value, "{left} vs {right}");
    }
}

/// Check malformed JSON-number spellings before any zero shortcut.
#[test]
fn invalid_tokens() {
    for source in [
        "", "+1", "00", "-01", ".1", "0.", "1.e2", "0e+", "0e-", "-", "1e", "1e+-2",
        "1.2.3", "1 ", "1\n", "1\0", "1١", "0١", "0e١", "1.é", "1eé",
    ] {
        assert!(JsoncNumberIdentity::from_token(source).is_err(), "{source}");
    }
}

/// Exercise digit-string carry and borrow beyond every machine exponent width.
#[test]
fn long_exponent_arithmetic() {
    let nines = "9".repeat(256);
    let next = format!("1{}", "0".repeat(256));
    let right = JsoncNumberIdentity::from_token(&format!("1e{next}")).expect("valid huge power");
    let left = JsoncNumberIdentity::from_token(&format!("10e{nines}")).expect("valid compensated power");
    assert_eq!(left, right, "positive full-width carry");

    let negative = JsoncNumberIdentity::from_token(&format!("0.1e-{nines}")).expect("valid negative power");
    let negative_expected = JsoncNumberIdentity::from_token(&format!("1e-{next}")).expect("valid negative carry");
    assert_eq!(negative, negative_expected, "negative full-width carry");

    let borrow = JsoncNumberIdentity::from_token(&format!("0.1e{next}")).expect("valid positive borrow");
    let borrow_expected = JsoncNumberIdentity::from_token(&format!("1e{nines}")).expect("valid borrow result");
    assert_eq!(borrow, borrow_expected, "positive full-width borrow");

    let fraction_adjustment = JsoncNumberIdentity::from_token("0.000000000001e20").expect("valid multi-digit adjustment");
    let fraction_expected = JsoncNumberIdentity::from_token("1e8").expect("valid decimal power");
    assert_eq!(fraction_adjustment, fraction_expected);
    let trailing_adjustment = JsoncNumberIdentity::from_token("1000000000000").expect("valid long integer");
    let trailing_expected = JsoncNumberIdentity::from_token("1e12").expect("valid trailing adjustment");
    assert_eq!(trailing_adjustment, trailing_expected);
}

/// Check derived hashing uses the same canonical identity as equality.
#[test]
fn hash_matches_value_equality() {
    // What: `HashSet` stores value identities by `Hash` and checks candidates by `Eq`.
    // Why: A value read under another spelling must be found in the same set.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const seen = new Set<string>();
    // ```
    let mut seen = std::collections::HashSet::new();
    seen.insert(JsoncNumberIdentity::from_token("1e0").expect("valid number"));
    assert!(seen.contains(&JsoncNumberIdentity::from_token("1.000").expect("valid number")));
    assert!(!seen.contains(&JsoncNumberIdentity::from_token("2").expect("valid number")));
    seen.insert(JsoncNumberIdentity::from_token("-0e123").expect("valid zero"));
    assert!(seen.contains(&JsoncNumberIdentity::from_token("0.000").expect("valid zero")));
}

/// What:
///  Represent a bounded decimal as an exact numerator and denominator.
/// Why:
///  This independent oracle can cross-multiply rather than reusing the normalization algorithm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rational(source: string): [bigint, bigint] { /* bounded fixture only */ }
/// ```
fn rational(source: &str) -> (i128, i128) {
    // The generated fixture always includes a base-ten exponent.
    let (mantissa, exp) = source.split_once('e').expect("generated exponent");
    let (int_part, frac_part) = if let Some(parts) = mantissa.split_once('.') {
        parts
    } else {
        (mantissa, "")
    };
    let digits = format!("{int_part}{frac_part}");
    let numerator: i128 = digits.parse().expect("bounded fixture coefficient");
    let power: i32 = exp.parse::<i32>().expect("bounded exponent")
        - i32::try_from(frac_part.len()).expect("bounded fractional length");
    if power >= 0 {
        // The bounded fixture's power fits an integer, unlike unrestricted JSON.
        return (numerator * 10_i128.pow(u32::try_from(power).expect("nonnegative power")), 1);
    }
    // Return an exact rational without rounding the fraction.
    return (numerator, 10_i128.pow(power.unsigned_abs()));
}

/// Compare normalized identities with a separately computed exact rational oracle.
#[test]
fn generated_against_rational_oracle() {
    // What: `Vec<String>` owns all generated sample spellings, unlike borrowed `&str` views.
    // Why: Each subsequent pair comparison can read a stable source token.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const samples: string[] = [];
    // ```
    let mut samples: Vec<String> = Vec::new();
    for negative in [false, true] {
        for coefficient in 0..20 {
            for fractional in 0..4 {
                for exponent in -3..4 {
                    let digits = format!("{coefficient:0width$}", width = fractional + 1);
                    let split = digits.len() - fractional;
                    let base = if fractional == 0 {
                        digits
                    } else {
                        format!("{}.{}", &digits[..split], &digits[split..])
                    };
                    let sign = if negative { "-" } else { "" };
                    samples.push(format!("{sign}{base}e{exponent}"));
                }
            }
        }
    }
    // Validate every generated spelling before sampling pairwise comparisons.
    for source in &samples {
        JsoncNumberIdentity::from_token(source).expect("every generated number is valid");
    }
    let mut i = 0;
    while i < samples.len() {
        let mut j = 0;
        while j < samples.len() {
            let left = &samples[i];
            let right = &samples[j];
            let (left_num, left_den) = rational(left);
            let (right_num, right_den) = rational(right);
            let oracle_equal = left_num * right_den == right_num * left_den;
            let left_value = JsoncNumberIdentity::from_token(left).expect("generated left number valid");
            let right_value = JsoncNumberIdentity::from_token(right).expect("generated right number valid");
            let actual_equal = left_value == right_value;
            assert_eq!(actual_equal, oracle_equal, "{left} vs {right}");
            j += 23;
        }
        i += 17;
    }
    // Positive control: the oracle distinguishes exponent directions on unchanged digits.
    assert_ne!(rational("1e2"), rational("1e-2"));
}
