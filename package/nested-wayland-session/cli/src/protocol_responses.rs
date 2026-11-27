//! The one answer line a request yields,
//!  and its formatting as `ok`,
//!  `ok <data>`,
//!  or `err <message>`.
//!
//! Split out of `protocol.rs` to keep that file under the code-line limit./// A response to send back on the control socket.
///
/// What:
///      `pub enum Response { Ok, OkWith(String), Err(String) }`.
///  Success with no
///           data,
///  success with a payload,
///  or an error message.
/// Why:
///       A uniform,
///  machine-readable reply shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Response = { ok: true; data?: string } | { ok: false; error: string };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Success,
    ///  no payload.
    Ok,
    /// Success with a text payload.
    OkWith(
        /// Payload text.
        String,
    ),
    /// Failure with a message.
    Err(
        /// Error message.
        String,
    ),
}

/// Format a response as its single wire line (no trailing newline).
///
/// What:
///      `pub fn format_response(response: &Response) -> String`.
///  Borrows the
///           response,
///  returns an owned `String`.
/// Why:
///       The control thread appends a newline and writes this to the socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function formatResponse(r: Response): string { ... }
/// ```
///
/// @example
/// ```ts
/// formatResponse({ ok: false, error: "bad" }); // => "err bad"
/// ```
pub fn format_response(response: &Response) -> String {
    // What:     `match response { ... }`. Map each variant to its wire form. `Ok` is the
    //           literal `"ok"`; `OkWith(data)` is `"ok "` plus the payload; `Err(message)`
    //           is `"err "` plus the message. `.to_string()` / `format!` allocate owned
    //           strings.
    // Why:      Produce the exact bytes the client parses.
    match response {
        Response::Ok => return "ok".to_string(),
        Response::OkWith(data) => return format!("ok {data}"),
        Response::Err(message) => return format!("err {message}"),
    }
}
