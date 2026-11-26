//! Unit tests for the error type rendering and equality.

use super::*;

// Decode and seek errors render distinct, message-bearing strings.
#[test]
fn display_includes_cause() {
    let decode = TruePeakError::Decode {
        message: "bad packet".to_string(),
    };
    let seek = TruePeakError::Seek {
        message: "no index".to_string(),
    };
    assert_eq!(decode.to_string(), "true-peak decode failed: bad packet");
    assert_eq!(seek.to_string(), "true-peak seek failed: no index");
}

// The derived equality distinguishes variants and messages.
#[test]
fn equality_distinguishes_variants() {
    let a = TruePeakError::Decode {
        message: "x".to_string(),
    };
    let b = TruePeakError::Decode {
        message: "x".to_string(),
    };
    let c = TruePeakError::Seek {
        message: "x".to_string(),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// The type participates in the standard error ecosystem.
#[test]
fn is_std_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}
    assert_error(&TruePeakError::Decode {
        message: "x".to_string(),
    });
}
