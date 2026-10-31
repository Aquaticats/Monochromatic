//! What:
//!  Tests for the Firefox ESR value parser.
//! Why:
//!  The decision record requires a diagnostic for every value not shaped `<major>.<minor>.<patch>esr`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('parseEsrMajor', () => { /* accepted and rejected shapes */ });
//! ```

use crate::firefox_esr::parse_esr_major;

/// The shape Mozilla publishes yields its major.
#[test]
fn accepts_the_published_shape() {
    assert_eq!(parse_esr_major("153.6.0esr").expect("shape"), 153);
    assert_eq!(parse_esr_major("115.44.0esr").expect("shape"), 115);
    assert_eq!(parse_esr_major("0.0.0esr").expect("shape"), 0);
}

/// Anything else fails with a message that quotes the value.
#[test]
fn rejects_other_shapes() {
    for raw in [
        "", "esr", "153esr", "153.6esr", "153.6.0", "153.6.0ESR", "153.6.0.1esr", "153.6.0 esr", "1a.6.0esr",
        "153..0esr", "-1.6.0esr", "153.6.0esr\n", "99999999999999999999999.0.0esr", "153.6.0b3",
    ] {
        let message = parse_esr_major(raw).expect_err("rejected").to_string();
        assert_eq!(message.contains("<major>.<minor>.<patch>esr"), true, "{raw:?}: {message}");
        assert_eq!(message.contains(&format!("{raw:?}")), true, "{raw:?}: {message}");
    }
}
