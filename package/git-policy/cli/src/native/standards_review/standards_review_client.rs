//! What:
//!  The one HTTPS call of the standards-review shadow run, made in-process and ended only when it shows no sign of work.
//! Why:
//!  The root `mise.toml` task `shadow-review` calls `fetch` with a 30 s signal. This port calls the same endpoint from
//!  the wrapper itself, so no `node` or `mise` is started. A call that keeps moving bytes is never cut off for being slow
//!  (owner, 2026-11-03: "Stop only on no progress"; pick 9 of `doc/planning/cli-git-shipped-limits-audit.md`, the
//!  coordinator's under the owner's delegation and open to the owner's veto). The transport's own idle timeouts do the
//!  judging: resolving, connecting, and each read and each write of the connection (TLS handshake, request, response head and
//!  body) may stay silent for the quiet window and no longer, and one ceiling covers the call as a whole (owner, 2026-11-04,
//!  "Silence plus a ceiling (Recommended)"; `wait_ceilings.rs`, `REVIEW_CALL_CEILING`). The longest quiet gap of every
//!  call is measured beside the call and kept in the ledger (`quiet_ledger.rs`, `thread_quiet.rs`). The trust roots are a
//!  parameter: production uses the roots embedded in the binary, a test passes the fixture authority of its local TLS server.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const response = await fetch(url, { method: 'POST', headers, body, dispatcher: new Agent({ bodyTimeout: 30000, headersTimeout: 30000 }) });
//! ```

/// The measuring of a call's quiet gap, beside the call.
use crate::thread_quiet::QuietMeasurement;
use std::time::Duration;
use ureq::tls::{Certificate, RootCerts, TlsConfig};

/// How long a call may stay silent before it is ended, when the caller sets nothing else (the script's own figure, which
///  was a total timeout there).
///  Citation: none, and **unmeasured as a quiet window**: the script's 30 seconds was inherited and no quiet gap of the call was
///  logged before `quiet_ledger.rs` kept one; the number waits for that measurement.
pub const SCRIPT_QUIET_WINDOW: Duration = Duration::from_secs(30);

/// What:
///  Where and how to call: the URL, the bearer key, the quiet window, and optionally the one trusted authority (PEM).
/// Why:
///   Tests point `url` at a loopback server; production passes `ENDPOINT`, a quiet window and no authority.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gateway = { url: string; apiKey: string; quietWindow: number; ceiling: number; trustedPem?: Uint8Array };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Gateway<'a> {
    /// The decisions endpoint.
    pub url: &'a str,
    /// The bearer credential.
    pub api_key: &'a str,
    /// How long the call may stay silent (resolving, connecting, one read, one write) before it is ended.
    pub quiet_window: Duration,
    /// The longest the call runs in all, whatever it shows (`wait_ceilings.rs`, `REVIEW_CALL_CEILING`).
    pub ceiling: Duration,
    /// A PEM authority that replaces the embedded roots; `None` keeps them.
    pub trusted_pem: Option<&'a [u8]>,
}

/// What:
///  The gateway's status and body text.
/// Why:
///   The answer reading decides what a status means; the client only transports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reply = { status: number; body: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reply {
    /// HTTP status code.
    pub status: u16,
    /// Response body as text.
    pub body: String,
}

/// What:
///  POST `body` as JSON with the bearer key; the call ends when it is silent for the quiet window or has run as long as the ceiling. Any
///  HTTP status is a reply, not an error.
///  Redirects are not followed and no proxy variable is read.
/// Why:
///   The script's failure line names timeouts and transport failures; a redirect or a proxy would send the key and the
///   repository text somewhere the endpoint constant does not name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const response = await fetch(url, { method: 'POST', headers: { 'content-type': 'application/json', authorization: 'Bearer ' + key }, body, signal });
/// ```
pub fn post_decision(gateway: &Gateway, body: &str) -> Result<Reply, String> {
    // The quiet gap of the call is measured beside it; the call itself ends on the transport's idle timeouts below.
    let measurement: QuietMeasurement = QuietMeasurement::begin("standards review gateway call");
    let reply: Result<Reply, String> = post_decision_unmeasured(gateway, body);
    measurement.finish();
    return reply;
}

/// `post_decision` without the measuring thread.
fn post_decision_unmeasured(gateway: &Gateway, body: &str) -> Result<Reply, String> {
    let tls: TlsConfig = match gateway.trusted_pem {
        Some(pem) => {
            let authority: Certificate<'static> = match Certificate::from_pem(pem) {
                Ok(parsed) => parsed,
                Err(error) => return Err(format!("the trusted authority is unreadable: {error}")),
            };
            TlsConfig::builder().root_certs(RootCerts::new_with_certs(&[authority])).build()
        }
        None => TlsConfig::default(),
    };
    let agent: ureq::Agent = ureq::Agent::config_builder()
        // Each phase and each transport read or write may be silent for the window; no timeout covers the call as a whole.
        .timeout_resolve(Some(gateway.quiet_window))
        .timeout_connect(Some(gateway.quiet_window))
        .timeout_per_read(Some(gateway.quiet_window))
        .timeout_per_write(Some(gateway.quiet_window))
        // The one clock over the whole call: a call that keeps moving bytes is ended here (owner, 2026-11-04, "Silence plus a
        // ceiling (Recommended)").
        .timeout_global(Some(gateway.ceiling))
        .http_status_as_error(false)
        .max_redirects(0)
        .proxy(None)
        .tls_config(tls)
        .build()
        .into();
    let sent = agent
        .post(gateway.url)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", gateway.api_key))
        .send(body.as_bytes());
    let mut response = match sent {
        Ok(received) => received,
        Err(error) => return Err(describe(error, gateway.ceiling)),
    };
    let status: u16 = response.status().as_u16();
    let text: String = match response.body_mut().read_to_string() {
        Ok(read) => read,
        Err(error) => return Err(describe(error, gateway.ceiling)),
    };
    return Ok(Reply { status, body: text });
}

/// What:
///  A failure as the log line prints it: the sentence for the ceiling, the script's timeout wording for an idle timeout,
///  otherwise `fetch failed` and the cause.
/// Why:
///   Node prints only `fetch failed` and hides the cause; the cause is what a person needs. The ceiling's sentence names the call,
///  the ceiling and the command that reviews the commit again, because the call ran as long as monochromatic-git-policy-cli lets it and a person
///  reading the log entry needs to know that it was not the endpoint's silence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error instanceof Error ? error.message : String(error)
/// ```
fn describe(error: ureq::Error, ceiling: Duration) -> String {
    return match error {
        ureq::Error::Timeout(ureq::Timeout::Global) => format!(
            "the standards-review gateway call ran for {} seconds, the longest monochromatic-git-policy-cli lets it run, and was ended, so this commit has no \
             verdict; review it again with `SHADOW_REVIEW_SHA=<commit> mise run shadow-review`",
            ceiling.as_secs()
        ),
        ureq::Error::Timeout(_) => String::from("The operation was aborted due to timeout"),
        other => format!("fetch failed: {other}"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_client_tests.rs"]
mod tests;
