//! Redacted cache diagnostics shared by scanner output and cli-git parsing.
//!
//! Every value is constructed from fixed ASCII tokens and rendered as one compact
//! JSON line.
//!  Private token fields make an unsupported reason/recovery pairing
//! unrepresentable without adding a constructor in this module.

/// Cache condition recovered by compiling authoritative text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CacheWarningReason {
    /// Expected content-addressed artifact did not exist.
    Missing,
    /// Per-user cache root could not be resolved or used.
    CacheRootUnavailable,
    /// Existing artifact could not be read completely.
    Unreadable,
    /// Envelope digest disagreed with current source content.
    SourceMismatch,
    /// Envelope belongs to another schema,
    ///  scanner,
    ///  or platform.
    Incompatible,
    /// Envelope framing,
    ///  names,
    ///  or engine bytes failed validation.
    Invalid,
}

/// One valid cache-warning record storing only fixed protocol tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CacheWarning {
    /// Closed reason token.
    reason: &'static str,
    /// Closed recovery token paired by constructor.
    recovery: &'static str,
}

/// Cache-warning constructors and fixed-token rendering.
impl CacheWarning {
    /// Read the fixed reason token without parsing its terminal JSON representation.
    pub fn reason(&self) -> &'static str {
        return self.reason;
    }

    /// Read the fixed recovery token paired with this warning by the existing constructors.
    pub fn recovery(&self) -> &'static str {
        return self.recovery;
    }

    /// Builds a warning for condition recovered by compiling authoritative text.
    pub(crate) fn compile_from_text(reason: CacheWarningReason) -> Self {
        return Self {
            reason: reason_token(reason),
            recovery: "compile-from-text",
        }
    }

    /// Builds only warning recovered by retaining already compiled rules.
    pub(crate) fn write_failed() -> Self {
        return Self {
            reason: "write-failed",
            recovery: "continue-with-compiled-rules",
        }
    }
}

/// Returns fixed JSON token for one text-compilation warning reason.
fn reason_token(reason: CacheWarningReason) -> &'static str {
    if reason == CacheWarningReason::Missing {
        return "missing";
    }
    if reason == CacheWarningReason::CacheRootUnavailable {
        return "cache-root-unavailable";
    }
    if reason == CacheWarningReason::Unreadable {
        return "unreadable";
    }
    if reason == CacheWarningReason::SourceMismatch {
        return "source-mismatch";
    }
    if reason == CacheWarningReason::Incompatible {
        return "incompatible";
    }
    return "invalid"
}

/// Renders compact single-line JSON without accepting arbitrary values.
impl std::fmt::Display for CacheWarning {
    /// Writes exact schema-version-1 protocol record.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            formatter,
            "{{\"type\":\"forbidden-strings/cache-warning\",\"schemaVersion\":1,\"reason\":\"{}\",\"recovery\":\"{}\"}}",
            self.reason,
            self.recovery,
        )
    }
}

/// Registers closed-protocol tests.
#[cfg(test)]
#[path = "warning_tests.rs"]
mod tests;
