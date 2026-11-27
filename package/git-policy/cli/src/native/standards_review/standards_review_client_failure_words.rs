//! What:
//!  The words a failed standards-review call is named with:
//! an error as its text,
//! the words of a transport failure with every cause below it,
//! and the refusal of a trusted authority.
//! Why:
//!  The first line of a transport error hides what a person needs,
//! so every failure the run records names the whole chain of causes,
//! and one place builds that text for the call and for its configuration.
//!  Kept apart from the call so both stay under the file limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // textOf(error); fetchFailed(error); fetchFailedWithCauses(error); errorChain(error); authorityUnreadable(error)
//! ```

/// An error as its text,
/// for the places that turn a failure into the call's message.
pub(super) fn text_of(error: impl std::fmt::Display) -> String {
    return error.to_string();
}

/// A failure to read the trusted authority,
/// in the call's words.
pub(super) fn authority_unreadable(error: impl std::fmt::Display) -> String {
    return format!("the trusted authority is unreadable: {error}");
}

/// A failure of the call,
/// in the words the log line prints.
pub(super) fn fetch_failed(error: impl std::fmt::Display) -> String {
    return format!("fetch failed: {error}");
}

/// A failure of the call with every cause below it,
/// in the words the log line prints.
pub(super) fn fetch_failed_with_causes(error: impl std::error::Error) -> String {
    return format!("fetch failed: {}", error_chain(&error));
}

/// An error with every cause below it,
/// as `a: b: c`,
/// because the first line of a transport error hides what a person needs.
fn error_chain(error: &dyn std::error::Error) -> String {
    let mut text: String = error.to_string();
    let mut below = error.source();
    while let Some(cause) = below {
        text.push_str(": ");
        text.push_str(cause.to_string().as_str());
        below = cause.source();
    }
    return text;
}
