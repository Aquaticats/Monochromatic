//! What:
//!      Depth-envelope target:
//!  the published nesting boundary must hold on both sides.
//! Why:
//!       The limit is the crate's protection against stack exhaustion,
//!  and it is the boundary a
//!           fuzzer is most likely to find a hole in.
//!  This target asserts the exact accepted depth,
//!           the exact rejected depth,
//!  and that malformed over-deep input is refused rather than
//!           overflowing.

#![no_main]

/// What:
///      Import the harness macro.
/// Why:
///       Every target in this sidecar is a libFuzzer entry point.
use libfuzzer_sys::fuzz_target;
/// What:
///      Import the exact-depth document builder.
/// Why:
///       A boundary needs inputs on both sides rather than generated approximations.
use jsonc_edit_fuzz::generators::nested_document;
/// What:
///      Import the iterative depth measurement.
/// Why:
///       The accepted side must be shown to hold the depth it was built with.
use jsonc_edit_fuzz::invariants::depth_of;
/// What:
///      Import the parser entry point.
/// Why:
///       The nesting limit is enforced while parsing.
use monochromatic_jsonc_edit::parse_jsonc;

/// The contract's accepted container count,
///  duplicated here on purpose:
///  the target must fail if the
/// crate's bound moves without the shared fixtures and this assertion moving with it.
const ACCEPTED_DEPTH: usize = 512;

/// Deepest generated nesting,
///  chosen past the bound so both sides are exercised every campaign.
const PROBE_CEILING: usize = 900;

fuzz_target!(|input: (u16, bool)| {
    let (raw_depth, malformed) = input;
    let depth = ((raw_depth as usize) % PROBE_CEILING) + 1;
    let source = nested_document(depth, malformed);

    if malformed {
        // An unterminated document must be refused whatever its depth, and must not exhaust the
        // stack while being refused.
        let error = parse_jsonc(&source).expect_err("malformed input must be rejected");
        assert!(!error.message.is_empty(), "malformed rejection had no message");
        return;
    }

    if depth <= ACCEPTED_DEPTH {
        let parsed = parse_jsonc(&source).unwrap_or_else(|error| panic!("depth {depth} must parse: {error}"));
        assert_eq!(depth_of(&parsed), depth, "parsed depth does not match the generated nesting");
        return;
    }

    let error = parse_jsonc(&source).err().unwrap_or_else(|| panic!("depth {depth} must be rejected"));
    assert_eq!(error.message, "JSONC nesting too deep", "depth {depth} was rejected for the wrong reason");
});
