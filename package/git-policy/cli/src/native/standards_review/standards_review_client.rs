//! What:
//!  The one HTTPS call of the standards-review shadow run,
//! made in-process on `hyper` and the process's runtime,
//! and ended only
//!  when it shows no sign of work.
//! Why:
//!  The root `mise.toml` task `shadow-review` calls `fetch` with a 30 s signal.
//! This port calls the same endpoint from
//!  the wrapper itself,
//! so no `node` or `mise` is started.
//! A call that keeps moving bytes is never cut off for being slow
//!  (owner,
//! 2026-11-03:
//! "Stop only on no progress";
//! pick 9 of `doc/planning/cli-git-shipped-limits-audit.md`,
//! the
//!  coordinator's under the owner's delegation and open to the owner's veto).
//! One clock does the judging:
//! a wrapper around the
//!  connection (`Marking`) records every read and write that completes,
//! and the call ends when nothing has moved for the quiet
//!  window,
//! the connect and the TLS handshake included (the owner,
//! 2026-11-07:
//! "Adopt tokio,
//! do not adopt ureq,
//! use the `hyper` set of
//!  https libs.";
//! the previous client set one timeout per phase),
//! and one ceiling covers the call as a whole (owner,
//! 2026-11-04,
//!  "Silence plus a ceiling (Recommended)";
//! `wait_ceilings.rs`,
//! `REVIEW_CALL_CEILING`).
//! A response body is read against `MAX_RESPONSE_BYTES`,
//! so an answer is never held whole
//!  in memory and a server cannot spend the service's memory by sending.
//! The longest quiet gap of every
//!  call is measured beside the call and kept in the ledger (`quiet_ledger.rs`,
//! `thread_quiet.rs`).
//! The trust roots are a
//!  parameter:
//! production uses the roots embedded in the binary,
//! a test passes the fixture authority of its local TLS server.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const response = await fetch(url, { method: 'POST', headers, body, dispatcher: new Agent({ bodyTimeout: 30000, headersTimeout: 30000 }) });
//! ```

/// The measuring of a call's quiet gap,
/// beside the call.
use crate::standards_review_client_name_lookup_on_the_blocking_pool::PoolResolver;
use crate::thread_quiet::QuietMeasurement;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// What:
///  The most answer bytes one call reads:
/// four times `INPUT_UNIT_CAP`,
/// the UTF-8 worst case of the request the answer replies to.
/// Why:
///   An answer to a question is not longer than the question,
/// and the request's own cap is the only bound this route already has.
///  A body past it is refused while it arrives,
/// so no answer is ever held in memory whole;
/// the
///  display excerpt of a failure message is separate and much smaller.
///  Citation:
/// none,
/// and **unmeasured as an answer size**:
/// the Decisions API publishes no answer bound and no answer's size was
///  logged,
/// so if a measured answer ever approaches this figure,
/// the relation to `INPUT_UNIT_CAP` has to be revisited.
pub const MAX_RESPONSE_BYTES: usize = crate::standards_review_input::INPUT_UNIT_CAP * 4;

/// How long a call may stay silent before it is ended,
/// when the caller sets nothing else (the script's own figure,
/// which
///  was a total timeout there).
///  Citation:
/// none,
/// and **unmeasured as a quiet window**:
/// the script's 30 seconds was inherited and no quiet gap of the call was
///  logged before `quiet_ledger.rs` kept one;
/// the number waits for that measurement.
pub const SCRIPT_QUIET_WINDOW: Duration = Duration::from_secs(30);

/// What:
///  Where and how to call:
/// the URL,
/// the bearer key,
/// the quiet window,
/// and optionally the one trusted authority (PEM).
/// Why:
///   Tests point `url` at a loopback server;
/// production passes `ENDPOINT`,
/// a quiet window and no authority.
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
    /// How long the call may stay silent (resolving,
    /// connecting,
    /// one read,
    /// one write) before it is ended.
    pub quiet_window: Duration,
    /// The longest the call runs in all,
    /// whatever it shows (`wait_ceilings.rs`,
    /// `REVIEW_CALL_CEILING`).
    pub ceiling: Duration,
    /// A PEM authority that replaces the embedded roots;
    /// `None` keeps them.
    pub trusted_pem: Option<&'a [u8]>,
}

/// What:
///  The gateway's status and body text.
/// Why:
///   The answer reading decides what a status means;
/// the client only transports.
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
///  POST `body` as JSON with the bearer key;
/// the call ends when it is silent for the quiet window or has run as long as the ceiling.
/// Any
///  HTTP status is a reply,
/// not an error.
///  Redirects are not followed and no proxy variable is read.
/// Why:
///   The script's failure line names timeouts and transport failures;
/// a redirect or a proxy would send the key and the
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

/// What:
///  The instant of the last byte moved on any connection of one call.
/// Why:
///  The quiet window is judged against it;
/// the connect and the handshake count as movement when they complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Progress { began = Date.now(); lastMs = 0; mark() { this.lastMs = Date.now() - this.began; } quietFor() { return Date.now() - this.began - this.lastMs; } }
/// ```
struct Progress {
    /// When the call began.
    began: Instant,
    /// Milliseconds after `began` of the last movement.
    last_ms: AtomicU64,
}

impl Progress {
    /// Milliseconds since the call began,
    /// saturated.
    fn elapsed_ms(&self) -> u64 {
        return u64::try_from(self.began.elapsed().as_millis()).unwrap_or(u64::MAX);
    }

    /// Records that bytes moved now.
    fn mark(&self) {
        self.last_ms.store(self.elapsed_ms(), Ordering::Relaxed);
    }

    /// How long nothing has moved.
    fn quiet_for(&self) -> Duration {
        return Duration::from_millis(self.elapsed_ms().saturating_sub(self.last_ms.load(Ordering::Relaxed)));
    }
}

/// A connection that marks every read or write that moves bytes,
/// so the call can be ended when nothing has moved for the window.
struct Marking<T> {
    /// The connection underneath.
    inner: T,
    /// Where the movement is recorded.
    progress: Arc<Progress>,
}

impl<T: hyper::rt::Read + Unpin> hyper::rt::Read for Marking<T> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: hyper::rt::ReadBufCursor<'_>) -> Poll<std::io::Result<()>> {
        let polled = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &polled {
            self.progress.mark();
        }
        return polled;
    }
}

impl<T: hyper::rt::Write + Unpin> hyper::rt::Write for Marking<T> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let polled = Pin::new(&mut self.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(count)) = &polled
            && *count > 0
        {
            self.progress.mark();
        }
        return polled;
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        return Pin::new(&mut self.inner).poll_flush(cx);
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        return Pin::new(&mut self.inner).poll_shutdown(cx);
    }
}

impl<T: hyper_util::client::legacy::connect::Connection> hyper_util::client::legacy::connect::Connection for Marking<T> {
    fn connected(&self) -> hyper_util::client::legacy::connect::Connected {
        return self.inner.connected();
    }
}

/// The plain TCP connector,
/// wrapping each connection it makes in `Marking`.
#[derive(Clone)]
struct MarkingConnector {
    /// The connector underneath.
    inner: hyper_util::client::legacy::connect::HttpConnector<PoolResolver>,
    /// Where the movement is recorded.
    progress: Arc<Progress>,
}

impl tower_service::Service<hyper::Uri> for MarkingConnector {
    type Response = Marking<hyper_util::rt::TokioIo<tokio::net::TcpStream>>;
    type Error = <hyper_util::client::legacy::connect::HttpConnector<PoolResolver> as tower_service::Service<hyper::Uri>>::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        return self.inner.poll_ready(cx);
    }

    fn call(&mut self, uri: hyper::Uri) -> Self::Future {
        let connecting = self.inner.call(uri);
        let progress: Arc<Progress> = Arc::clone(&self.progress);
        return Box::pin(async move {
            let stream = connecting.await?;
            progress.mark();
            return Ok(Marking { inner: stream, progress });
        });
    }
}

/// What:
///  Ends when `progress` has been quiet for `window`.
/// Why:
///  The call is raced against it;
/// sleeping for what is left of the window means a call that keeps moving is never woken for nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (progress.quietFor() < window) await sleep(window - progress.quietFor());
/// ```
async fn silence(progress: Arc<Progress>, window: Duration) {
    loop {
        let quiet: Duration = progress.quiet_for();
        if quiet >= window {
            return;
        }
        tokio::time::sleep(window - quiet).await;
    }
}

/// The client's TLS configuration:
/// the roots embedded in the binary,
/// or the one authority a caller trusts instead.
fn client_config(trusted_pem: Option<&[u8]>) -> Result<rustls::ClientConfig, String> {
    use rustls::pki_types::pem::PemObject as _;
    let mut roots: rustls::RootCertStore = rustls::RootCertStore::empty();
    match trusted_pem {
        Some(pem) => {
            let authority = rustls::pki_types::CertificateDer::from_pem_slice(pem).map_err(authority_unreadable)?;
            roots.add(authority).map_err(authority_unreadable)?;
        }
        None => roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()),
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(text_of)?
        .with_root_certificates(roots)
        .with_no_client_auth();
    return Ok(config);
}

/// An error as its text,
/// for the places that turn a failure into the call's message.
fn text_of(error: impl std::fmt::Display) -> String {
    return error.to_string();
}

/// A failure to read the trusted authority,
/// in the call's words.
fn authority_unreadable(error: impl std::fmt::Display) -> String {
    return format!("the trusted authority is unreadable: {error}");
}

/// A failure of the call,
/// in the words the log line prints.
fn fetch_failed(error: impl std::fmt::Display) -> String {
    return format!("fetch failed: {error}");
}

/// A failure of the call with every cause below it,
/// in the words the log line prints.
fn fetch_failed_with_causes(error: impl std::error::Error) -> String {
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

/// What:
///  The request and its reply on one new connection,
/// every byte it moves marked in `progress`.
/// Why:
///  A client per call,
/// as the previous agent was per call;
/// no redirect is followed,
/// no proxy variable is read,
/// and plain `http` is
///  accepted for the loopback mock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const response = await fetch(url, { method: 'POST', headers, body, redirect: 'error' });
/// ```
async fn exchange(gateway: &Gateway<'_>, body: &str, progress: Arc<Progress>) -> Result<Reply, String> {
    let tls: rustls::ClientConfig = client_config(gateway.trusted_pem)?;
    let mut http = hyper_util::client::legacy::connect::HttpConnector::new_with_resolver(PoolResolver);
    http.enforce_http(false);
    http.set_nodelay(true);
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_or_http()
        .enable_http1()
        .wrap_connector(MarkingConnector { inner: http, progress });
    let client: hyper_util::client::legacy::Client<_, http_body_util::Full<bytes::Bytes>> =
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(connector);
    let request = hyper::Request::post(gateway.url)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", gateway.api_key))
        .body(http_body_util::Full::new(bytes::Bytes::copy_from_slice(body.as_bytes())))
        .map_err(fetch_failed)?;
    let response = client.request(request).await.map_err(fetch_failed_with_causes)?;
    let status: u16 = response.status().as_u16();
    let text: String = read_body(response.into_body()).await?;
    return Ok(Reply { status, body: text });
}

/// What:
///  Read a response body frame by frame into text,
/// refusing it while it arrives once it would pass `MAX_RESPONSE_BYTES`.
/// Why:
///   A status says nothing about how much body arrives,
/// so an error answer can be as large as a success one;
/// collecting it first
///  would hold every byte of it in memory and cost the service's process what a remote answer chose to send.
/// A refused
///  body ends with the call,
/// and the review records the refusal in its entry,
/// as it records any failed call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const chunk of response.body) { if (bytes.length + chunk.length > MAX) throw new Error(...); bytes.push(...chunk) }
/// ```
async fn read_body(mut body: hyper::body::Incoming) -> Result<String, String> {
    use http_body_util::BodyExt as _;
    let mut bytes: Vec<u8> = Vec::new();
    while let Some(next) = body.frame().await {
        let frame = next.map_err(fetch_failed_with_causes)?;
        let Some(chunk) = frame.data_ref() else {
            continue;
        };
        if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(bytes.len()) {
            return Err(format!(
                "the decisions endpoint answered more than {MAX_RESPONSE_BYTES} bytes, the most one answer may carry, and the answer was not read"
            ));
        }
        bytes.extend_from_slice(chunk);
    }
    return String::from_utf8(bytes).map_err(fetch_failed);
}

/// How a call that was raced against the silence ended.
enum Raced {
    /// The call finished first,
    /// with its reply or its failure.
    Finished(Result<Reply, String>),
    /// Nothing moved for the quiet window.
    Quiet,
}

/// What:
///  Runs `call_future` until it ends or `quiet_future` does,
/// whichever is first.
/// Why:
///  Both are polled by the same task,
/// so the silence cannot be ended before the call has had its poll.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([call.then(Finished), quiet.then(() => Quiet)]);
/// ```
async fn race<F, Q>(call_future: F, quiet_future: Q) -> Raced
where
    F: Future<Output = Result<Reply, String>>,
    Q: Future<Output = ()>,
{
    let mut call = std::pin::pin!(call_future);
    let mut quiet = std::pin::pin!(quiet_future);
    // The call is polled first, so the silence cannot end a call that has just finished.
    let raced: Raced = tokio::select! {
        biased;
        result = &mut call => Raced::Finished(result),
        () = &mut quiet => Raced::Quiet,
    };
    return raced;
}

/// `post_decision` without the measuring thread.
fn post_decision_unmeasured(gateway: &Gateway, body: &str) -> Result<Reply, String> {
    let progress: Arc<Progress> = Arc::new(Progress { began: Instant::now(), last_ms: AtomicU64::new(0) });
    let (ceiling, window): (Duration, Duration) = (gateway.ceiling, gateway.quiet_window);
    let outcome = crate::lazy_runtime::run_on_runtime(async {
        return tokio::time::timeout(ceiling, race(exchange(gateway, body, Arc::clone(&progress)), silence(Arc::clone(&progress), window))).await;
    })
    .map_err(text_of)?;
    return match outcome {
        Ok(Raced::Finished(reply)) => reply,
        Ok(Raced::Quiet) => Err(String::from("The operation was aborted due to timeout")),
        Err(_elapsed) => Err(format!(
            "the standards-review gateway call ran for {} seconds, the longest monochromatic-git-policy-cli lets it run, and was ended, so this commit has no \
             verdict; review it again with `SHADOW_REVIEW_SHA=<commit> mise run shadow-review`",
            ceiling.as_secs()
        )),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_client_tests.rs"]
mod tests;
