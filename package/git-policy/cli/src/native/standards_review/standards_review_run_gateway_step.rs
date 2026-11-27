//! What:
//!  The one gateway step of a review:
//! the verdict the Decisions API answered,
//! or the message that says what failed instead.
//! Why:
//!  A missing credential,
//! a call that failed and an answer that could not be read are all a review with no verdict,
//! and none of them may fail a landing,
//! so each failure becomes the same entry with its own sentence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // askGateway(input, options): { top: string; outcome: string }
//! ```

/// The in-process call and what it transports.
use super::standards_review_client::{
    Gateway,
    Reply,
    post_decision,
};
/// The capped input the question is built from.
use super::standards_review_input::ReviewInput;
/// The request body of the question and the reading of the answer.
use super::standards_review_request::{
    Judged,
    read_answer,
    request_body,
};
/// What the run was told.
use super::standards_review_run_options::{
    NO_KEY_MESSAGE,
    ReviewOptions,
};

/// What:
///  The verdict of a gateway step that failed:
/// no top choice,
/// and the message in the outcome line.
/// Why:
///   `ask_gateway` reports every failure (no key,
/// no reply,
/// an unreadable reply) the same way,
/// and a named function
///  stands where a closure that built this verdict stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lunaFailed = (message: string) => ({ top: '', outcome: `Luna failed: ${message}` });
/// ```
fn luna_failed(message: String) -> Judged {
    return Judged {
        top: String::new(),
        outcome: format!("Luna failed: {message}"),
    };
}

/// What:
///  The gateway step:
/// the verdict,
/// or the failure message,
/// and the elapsed milliseconds.
/// Why:
///   A missing credential fails before any connection,
/// as in the script.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { ...fetch... } catch (error) { lunaOutcome = 'Luna failed: ' + message }
/// ```
pub(super) fn ask_gateway(
    input: &ReviewInput,
    options: &ReviewOptions,
) -> Judged {
    let key: &str = options.api_key.as_deref().unwrap_or("");
    if key.is_empty() {
        return luna_failed(String::from(NO_KEY_MESSAGE));
    }
    let gateway: Gateway = Gateway {
        url: options.endpoint.as_str(),
        api_key: key,
        quiet_window: options.quiet_window,
        ceiling: options.call_ceiling,
        trusted_pem: options.trusted_pem.as_deref(),
    };
    let reply: Result<Reply, String> =
        post_decision(&gateway, request_body(input.model_input.as_str()).as_str());
    let read: Result<Judged, String> = match reply {
        Ok(answer) => read_answer(answer.status, answer.body.as_str()),
        Err(message) => Err(message),
    };
    return match read {
        Ok(judged) => judged,
        Err(message) => luna_failed(message),
    };
}
