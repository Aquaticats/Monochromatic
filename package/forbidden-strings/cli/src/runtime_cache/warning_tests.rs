//! Cache-warning JSON protocol tests.

use super::{CacheWarning, CacheWarningReason};

/// Every load failure renders exact closed JSON without dynamic data.
#[test]
fn load_failure_reasons_render_exact_json() {
    let cases = [
        (CacheWarningReason::Missing, "missing"),
        (CacheWarningReason::CacheRootUnavailable, "cache-root-unavailable"),
        (CacheWarningReason::Unreadable, "unreadable"),
        (CacheWarningReason::SourceMismatch, "source-mismatch"),
        (CacheWarningReason::Incompatible, "incompatible"),
        (CacheWarningReason::Invalid, "invalid"),
    ];
    for (reason, token) in cases {
        let warning = CacheWarning::compile_from_text(reason);
        assert_eq!(warning.reason(), token);
        assert_eq!(warning.recovery(), "compile-from-text");
        assert_eq!(
            warning.to_string(),
            format!(
                "{{\"type\":\"forbidden-strings/cache-warning\",\"schemaVersion\":1,\"reason\":\"{token}\",\"recovery\":\"compile-from-text\"}}",
            ),
        );
    }
}

/// Write failure renders only continue-with-compiled-rules pairing.
#[test]
fn write_failure_renders_exact_json() {
    let warning: CacheWarning = CacheWarning::write_failed();
    assert_eq!(warning.reason(), "write-failed");
    assert_eq!(warning.recovery(), "continue-with-compiled-rules");
    assert_eq!(
        CacheWarning::write_failed().to_string(),
        "{\"type\":\"forbidden-strings/cache-warning\",\"schemaVersion\":1,\"reason\":\"write-failed\",\"recovery\":\"continue-with-compiled-rules\"}",
    );
}
