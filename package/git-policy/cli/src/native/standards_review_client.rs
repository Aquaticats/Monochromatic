//! What:
//!  The one HTTPS call of the standards-review shadow run, made in-process with a total deadline.
//! Why:
//!  The root `mise.toml` task `shadow-review` calls `fetch` with a 30 s signal. This port calls the same endpoint from
//!  the wrapper itself, so no `node` or `mise` is started. The deadline covers connecting, TLS, sending and reading
//!  the body, so the call can never hold a caller longer than the deadline. The trust roots are a parameter: production
//!  uses the roots embedded in the binary, a test passes the fixture authority of its local TLS server.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const response = await fetch(url, { method: 'POST', headers, body, signal: AbortSignal.timeout(30000) });
//! ```

use std::time::Duration;
use ureq::tls::{Certificate, RootCerts, TlsConfig};

/// The script's timeout.
pub const SCRIPT_DEADLINE: Duration = Duration::from_secs(30);

/// What:
///  Where and how to call: the URL, the bearer key, the deadline, and optionally the one trusted authority (PEM).
/// Why:
///   Tests point `url` at a loopback server; production passes `ENDPOINT`, the script's deadline and no authority.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gateway = { url: string; apiKey: string; deadline: number; trustedPem?: Uint8Array };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Gateway<'a> {
    /// The decisions endpoint.
    pub url: &'a str,
    /// The bearer credential.
    pub api_key: &'a str,
    /// The total time the call may take.
    pub deadline: Duration,
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
///  POST `body` as JSON with the bearer key, within the deadline; any HTTP status is a reply, not an error.
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
    let tls: TlsConfig = match gateway.trusted_pem {
        Some(pem) => {
            let authority: Certificate<'static> = Certificate::from_pem(pem).map_err(|error| return format!("the trusted authority is unreadable: {error}"))?;
            TlsConfig::builder().root_certs(RootCerts::new_with_certs(&[authority])).build()
        }
        None => TlsConfig::default(),
    };
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(gateway.deadline))
        .http_status_as_error(false)
        .max_redirects(0)
        .proxy(None)
        .tls_config(tls)
        .build()
        .into();
    let mut response = agent
        .post(gateway.url)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", gateway.api_key))
        .send(body.as_bytes())
        .map_err(describe)?;
    let status: u16 = response.status().as_u16();
    let text: String = response.body_mut().read_to_string().map_err(describe)?;
    return Ok(Reply { status, body: text });
}

/// What:
///  A failure as the log line prints it: the script's timeout wording, otherwise `fetch failed` and the cause.
/// Why:
///   Node prints only `fetch failed` and hides the cause; the cause is what a person needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error instanceof Error ? error.message : String(error)
/// ```
fn describe(error: ureq::Error) -> String {
    return match error {
        ureq::Error::Timeout(_) => String::from("The operation was aborted due to timeout"),
        other => format!("fetch failed: {other}"),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_client_tests.rs"]
mod tests;
