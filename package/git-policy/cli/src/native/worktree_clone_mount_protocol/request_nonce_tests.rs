//! What:
//!  Controls of the nonce:
//!  the accepted alphabet,
//!  the length,
//!  the refusals that name the byte,
//!  and a fresh nonce's shape.
//! Why:
//!  The nonce is the only thing a receipt is matched on,
//!  so a second accepted spelling of one value would let a stale receipt
//!  answer a later request.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(() => RequestNonce.parse('A'.repeat(32))).toThrow();
//! ```

use super::{RequestNonce, shown_byte};

/// Every lowercase hexadecimal digit is accepted at its place.
#[test]
fn all_lowercase_hexadecimal_digits_are_accepted() {
    let digits: String = String::from("0123456789abcdef").repeat(2);
    let nonce: RequestNonce = RequestNonce::parse(digits.as_str()).expect("32 lowercase hex digits are a nonce");
    assert_eq!(nonce.as_str(), digits);
}

/// A value of another length is refused and names the length.
#[test]
fn a_value_of_another_length_is_refused() {
    for length in [0, 1, 31, 33, 64] {
        let text: String = "0".repeat(length);
        let refusal = RequestNonce::parse(text.as_str()).expect_err("another length is refused");
        assert_eq!(refusal, super::NonceRefusal::Length { found: length });
        assert!(refusal.message().contains(&format!("{length} bytes")), "{}", refusal.message());
    }
}

/// An uppercase digit is refused at its place rather than folded.
#[test]
fn an_uppercase_digit_is_refused_at_its_place() {
    let text: String = format!("0{}", "A".repeat(31));
    let refusal = RequestNonce::parse(text.as_str()).expect_err("uppercase is another value");
    assert_eq!(refusal, super::NonceRefusal::NotLowercaseHex { place: 2, shown: String::from("A") });
}

/// A byte that is not hexadecimal at all is refused,
/// and a control byte reaches the message as an escape.
#[test]
fn a_non_hexadecimal_byte_is_refused() {
    let text: String = format!("0\n{}", "0".repeat(30));
    let refusal = RequestNonce::parse(text.as_str()).expect_err("a newline is not a digit");
    assert_eq!(refusal, super::NonceRefusal::NotLowercaseHex { place: 2, shown: String::from("\\x0a") });
    assert!(!refusal.message().contains('\n') || refusal.message().contains("\\x0a"));
    assert_eq!(shown_byte(b'g'), String::from("g"));
}

/// A fresh nonce is 32 lowercase hexadecimal digits,
/// and two of them differ.
#[test]
fn a_fresh_nonce_has_the_designed_shape() {
    let first: RequestNonce = RequestNonce::fresh().expect("the random source answers");
    let second: RequestNonce = RequestNonce::fresh().expect("the random source answers again");
    assert_eq!(first.as_str().len(), 32);
    assert!(first.as_str().bytes().all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    assert_ne!(first, second);
    assert_eq!(RequestNonce::parse(first.as_str()).as_ref(), Ok(&first));
}
