//! Controls for the gateway client against a loopback mock.
//! No test reaches a real host or sends a real key.
//!
//! What:
//!  Every verdict here comes from the order of events:
//! which bytes arrived,
//! whether the call reached the server,
//! and how
//!  the server's answer ended.
//! Why:
//!   A control that judged the host's speed failed in a loaded suite
//! (`doc/handover/cli-git-rust-implementation.md`);
//!  measured under `CPUQuota=3%` shared by these controls,
//! eight of the twelve failed,
//! seven of them
//!  with the client's timeout wording,
//! because a 400 ms quiet window elapsed before a busy host read a reply that had already arrived.
//!  Only the controls about the window and the ceiling keep a figure,
//! and the figure is their subject.

use super::*;
use crate::standards_review_mock::{ANSWER_WINDOW, Answer, AnswerEnded, CONTROL_CEILING, CONNECTION_DEADLINE, Mock, Served, SILENT_AT_MOST, accept_within, http_reply};
use std::time::Instant;

/// A stand-in credential;
/// it names no real key.
const KEY: &str = "test-key-not-a-secret";

/// What:
///  The quiet window of the controls about a reply that keeps arriving.
/// Why:
///   The reply's cadence must sit far under the window,
/// so neither a slow block nor a late poll can end a reply that keeps arriving;
///  `TRICKLE_PAUSE` is a fiftieth of it.
/// The window is also what the ceiling of the ceiling control sits under.
const TRICKLE_WINDOW: Duration = Duration::from_secs(5);

/// The pause between two body bytes of a reply that keeps arriving:
/// a fiftieth of `TRICKLE_WINDOW`.
const TRICKLE_PAUSE: Duration = Duration::from_millis(100);

/// What:
///  A gateway at `url` with the given quiet window and the ceiling no control reaches.
/// Why:
///   The ceiling is the outer bound of every call here,
/// so a call that neither answers nor falls silent fails
///  with the client's own message instead of holding the test.
fn gateway(url: &str, quiet_window: Duration) -> Gateway<'_> {
    return Gateway { url, api_key: KEY, quiet_window, ceiling: CONTROL_CEILING, trusted_pem: None };
}

/// A reply body of `bytes` bytes,
/// one every `pause`,
/// lasting `windows` windows so its whole time is that many times the window judging it.
fn trickle_bytes(windows: u128, window: Duration, pause: Duration) -> usize {
    return usize::try_from(windows * window.as_millis() / pause.as_millis()).expect("a byte count that fits");
}

/// The request has the method,
/// path,
/// JSON content type,
/// bearer header and the exact body bytes.
#[test]
fn the_request_carries_method_headers_and_body() {
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", "{}")));
    let url: String = mock.url.clone();
    let body: &str = "{\"input\":\"text with \\\"quotes\\\" and \u{e9}\"}";
    let reply: Reply = post_decision(&gateway(url.as_str(), ANSWER_WINDOW), body).expect("reply");
    assert_eq!(reply, Reply { status: 200, body: String::from("{}") });
    let served: Served = mock.finish();
    let seen: String = served.seen;
    let lowered: String = seen.to_ascii_lowercase();
    assert!(seen.starts_with("POST /v1/decisions HTTP/1.1\r\n"), "{seen}");
    assert!(lowered.contains("content-type: application/json\r\n"));
    assert!(lowered.contains(&format!("authorization: bearer {KEY}\r\n")));
    assert!(lowered.contains(&format!("content-length: {}\r\n", body.len())));
    assert!(seen.ends_with(body));
    assert_eq!(served.ended, AnswerEnded::Whole, "the mock wrote its whole answer");
}

/// An error status is a reply with its body,
/// not a client error.
#[test]
fn an_error_status_is_a_reply() {
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("503 Service Unavailable", "busy")));
    let url: String = mock.url.clone();
    let reply: Reply = post_decision(&gateway(url.as_str(), ANSWER_WINDOW), "{}").expect("reply");
    assert_eq!((reply.status, reply.body.as_str()), (503, "busy"));
    mock.finish();
}

/// A chunked body is reassembled.
#[test]
fn a_chunked_body_is_read_whole() {
    let chunked: Vec<u8> = b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\nconnection: close\r\n\r\n3\r\nabc\r\n2\r\nde\r\n0\r\n\r\n".to_vec();
    let mock: Mock = Mock::start(Answer::Bytes(chunked));
    let url: String = mock.url.clone();
    let reply: Reply = post_decision(&gateway(url.as_str(), ANSWER_WINDOW), "{}").expect("reply");
    assert_eq!(reply.body, "abcde");
    mock.finish();
}

/// A redirect is returned as a status;
/// its target is never contacted.
#[test]
fn a_redirect_is_not_followed() {
    let redirect: Vec<u8> = b"HTTP/1.1 302 Found\r\nlocation: http://127.0.0.1:1/elsewhere\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_vec();
    let mock: Mock = Mock::start(Answer::Bytes(redirect));
    let url: String = mock.url.clone();
    let reply: Reply = post_decision(&gateway(url.as_str(), ANSWER_WINDOW), "{}").expect("reply");
    assert_eq!(reply.status, 302);
    mock.finish();
}

/// A server that never answers ends the call when it has been silent for the quiet window,
/// with the timeout wording.
/// The order of events is the verdict,
/// not a duration:
/// the call reached the server,
/// and the server was still holding the connection when the call closed it.
/// The server's own hold (`SILENT_AT_MOST`) is the slower alternative,
/// and a call it ended would answer with a transport failure instead of the wording below.
#[test]
fn a_silent_server_ends_after_the_quiet_window() {
    let mock: Mock = Mock::start(Answer::Silent(SILENT_AT_MOST));
    let url: String = mock.url.clone();
    let failure: String = post_decision(&gateway(url.as_str(), ANSWER_WINDOW), "{}").expect_err("the window ends a silent call");
    assert_eq!(failure, "The operation was aborted due to timeout");
    let served: Served = mock.finish();
    assert_eq!(served.ended, AnswerEnded::CallerClosedTheSilence, "the server's hold of {SILENT_AT_MOST:?} ended before the call did");
    assert!(served.seen.starts_with("POST /v1/decisions HTTP/1.1\r\n"), "the call reached the server: {}", served.seen);
}

/// A refused connection is a transport failure that names its cause.
#[test]
fn a_refused_connection_names_the_cause() {
    let mock: Mock = Mock::start(Answer::Bytes(Vec::new()));
    let url: String = mock.url.clone();
    // The mock serves one connection; take it so the port is closed for the call under test.
    let _ = std::net::TcpStream::connect(url.trim_start_matches("http://").trim_end_matches("/v1/decisions"));
    let served: Served = mock.finish();
    assert_eq!(served.ended, AnswerEnded::Whole, "the mock answered the connection that closed the port");
    let failure: String = post_decision(&gateway(url.as_str(), ANSWER_WINDOW), "{}").expect_err("refused");
    assert!(failure.starts_with("fetch failed: "), "{failure}");
}

/// A trusted authority that is not PEM is refused before any connection is made.
#[test]
fn an_unreadable_authority_is_refused() {
    let mut bad: Gateway = gateway("https://127.0.0.1:1/", ANSWER_WINDOW);
    bad.trusted_pem = Some(b"not a certificate");
    assert!(post_decision(&bad, "{}").expect_err("authority").starts_with("the trusted authority is unreadable"));
}

/// A reply that keeps arriving is never cut off however long it takes:
/// the window judges quiet time,
/// not wall time.
///  The mock sends one byte a pause for two windows,
/// so the reply is never silent for the window,
/// and every byte of it is read and none is left unsent.
#[test]
fn a_reply_that_keeps_arriving_is_never_cut_off() {
    /// How many windows the reply lasts,
    /// so the call runs for twice the window that judges it.
    const WINDOWS: u128 = 2;
    let bytes: usize = trickle_bytes(WINDOWS, TRICKLE_WINDOW, TRICKLE_PAUSE);
    let mock: Mock = Mock::start(Answer::Trickle { bytes, pause: TRICKLE_PAUSE });
    let url: String = mock.url.clone();
    let reply: Reply = post_decision(&gateway(url.as_str(), TRICKLE_WINDOW), "{}").expect("a reply that keeps arriving is read whole");
    assert_eq!(reply.body.len(), bytes, "the reply is read whole");
    assert_eq!(mock.finish().ended, AnswerEnded::Whole, "the call closed the reply before its last byte");
}

/// Owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)":
/// a reply that keeps arriving is ended at the ceiling all the same,
/// with
/// an error that names the call,
/// the ceiling and what to run.
/// The mock's reply is twice the ceiling in pauses,
/// so the ceiling passes while the reply still arrives,
/// and its unsent bytes are the event that says so.
#[test]
fn a_reply_that_keeps_arriving_is_ended_at_the_ceiling() {
    let ceiling: Duration = Duration::from_millis(1500);
    let bytes: usize = trickle_bytes(2, ceiling, TRICKLE_PAUSE);
    let mock: Mock = Mock::start(Answer::Trickle { bytes, pause: TRICKLE_PAUSE });
    let url: String = mock.url.clone();
    let failure: String = post_decision(&Gateway { ceiling, ..gateway(url.as_str(), TRICKLE_WINDOW) }, "{}").expect_err("the ceiling ended the call");
    assert!(failure.contains("standards-review gateway call") && failure.contains(&format!("{} seconds", ceiling.as_secs())) && failure.contains("shadow-review"), "{failure}");
    let served: Served = mock.finish();
    assert!(matches!(served.ended, AnswerEnded::CallerLeft { unsent } if unsent > 0), "the reply had ended before the ceiling reached it: {:?}", served.ended);
}

/// The client is built on `hyper` and no longer on `ureq` (owner,
/// 2026-11-07:
/// "Adopt tokio,
/// do not adopt ureq,
/// use the `hyper`
///  set of https libs."),
/// and reads the network on the process's runtime.
#[test]
fn the_client_is_built_on_hyper_and_no_longer_on_ureq() {
    let source: &str = include_str!("standards_review_client.rs");
    let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    assert!(!code.contains("ureq"), "the client still names `ureq`");
    assert!(code.contains("hyper_util::"), "the client does not use `hyper`");
}

/// How long one `openssl` step may take before the control fails naming it:
/// a throwaway key is milliseconds of work,
/// and the tool is the only outside program this module starts.
const OPENSSL_DEADLINE: Duration = Duration::from_secs(30);

/// How often a control looks at a child it started while it waits for it.
const CHILD_POLL: Duration = Duration::from_millis(5);

/// How many calls the TLS server serves:
/// the trusted one,
/// then the one it refuses.
const TLS_CALLS: usize = 2;

/// What:
///  Run one `openssl` step and wait for it within `OPENSSL_DEADLINE`.
/// Why:
///   `status` waits for as long as the child runs,
/// so a tool that never ends would hold the control at the runner's stop;
///  this names the command and the figure instead.
fn run_openssl(directory: &std::path::Path, arguments: &[&str]) -> std::process::ExitStatus {
    let mut child: std::process::Child = std::process::Command::new("openssl")
        .args(arguments)
        .current_dir(directory)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("start openssl");
    let began: Instant = Instant::now();
    loop {
        match child.try_wait().expect("ask how openssl ended") {
            Some(status) => return status,
            None => {
                assert!(began.elapsed() < OPENSSL_DEADLINE, "`openssl {}` was still running after {OPENSSL_DEADLINE:?}", arguments.join(" "));
                std::thread::sleep(CHILD_POLL);
            }
        }
    }
}

/// A throwaway authority and a leaf it signed for `127.0.0.1`,
/// made by the `openssl` tool in `directory`;
/// returns the authority's
///  certificate,
/// the leaf's certificate and the leaf's private key,
/// all as PEM text.
fn throwaway_authority_and_leaf(directory: &std::path::Path) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let run = |arguments: &[&str]| {
        let status: std::process::ExitStatus = run_openssl(directory, arguments);
        assert!(status.success(), "openssl {arguments:?}");
    };
    std::fs::write(directory.join("leaf.ext"), "subjectAltName=IP:127.0.0.1\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\n").expect("extensions");
    run(&["req", "-x509", "-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:prime256v1", "-nodes", "-keyout", "ca-key.pem", "-out", "ca.pem", "-days", "2", "-subj", "/CN=throwaway-authority", "-addext", "basicConstraints=critical,CA:TRUE", "-addext", "keyUsage=critical,keyCertSign"]);
    run(&["req", "-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:prime256v1", "-nodes", "-keyout", "key.pem", "-out", "leaf.csr", "-subj", "/CN=throwaway-leaf"]);
    run(&["x509", "-req", "-in", "leaf.csr", "-CA", "ca.pem", "-CAkey", "ca-key.pem", "-CAcreateserial", "-out", "cert.pem", "-days", "2", "-extfile", "leaf.ext"]);
    let read = |name: &str| { return std::fs::read(directory.join(name)).expect("a generated file"); };
    return (read("ca.pem"), read("cert.pem"), read("key.pem"));
}

/// What:
///  A TLS server on loopback that answers each request it reads with a 200 whose body is the request head,
///  until it has accepted `connections` connections or waited `CONNECTION_DEADLINE` for the next one.
/// Why:
///   The control reads how many calls reached the server,
/// so a call that never arrived is an event and never a wait;
/// a connection whose handshake fails is still a call that arrived.
/// Returns the port and the serving thread,
/// which returns the number of connections it accepted.
fn serve_tls(certificate: Vec<u8>, key: Vec<u8>, connections: usize) -> (u16, std::thread::JoinHandle<usize>) {
    use rustls::pki_types::pem::PemObject as _;
    use std::io::{Read as _, Write as _};
    let chain: Vec<rustls::pki_types::CertificateDer<'static>> =
        rustls::pki_types::CertificateDer::pem_slice_iter(certificate.as_slice()).collect::<Result<_, _>>().expect("the certificate");
    let private: rustls::pki_types::PrivateKeyDer<'static> = rustls::pki_types::PrivateKeyDer::from_pem_slice(key.as_slice()).expect("the key");
    let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    let config = std::sync::Arc::new(
        rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("protocol versions")
            .with_no_client_auth()
            .with_single_cert(chain, private)
            .expect("the server certificate"),
    );
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let port: u16 = listener.local_addr().expect("address").port();
    listener.set_nonblocking(true).expect("a listener the server polls");
    let served = std::thread::spawn(move || {
        let mut accepted: usize = 0;
        for _ in 0..connections {
            let Some(mut tcp) = accept_within(&listener, CONNECTION_DEADLINE) else { break };
            accepted += 1;
            tcp.set_read_timeout(Some(CONNECTION_DEADLINE)).expect("read timeout");
            let Ok(mut session) = rustls::ServerConnection::new(std::sync::Arc::clone(&config)) else { continue };
            let mut stream = rustls::Stream::new(&mut session, &mut tcp);
            let mut seen: Vec<u8> = Vec::new();
            let mut chunk: [u8; 1024] = [0; 1024];
            while !seen.windows(4).any(|window| { return window == b"\r\n\r\n"; }) {
                match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => seen.extend_from_slice(&chunk[..count]),
                }
            }
            if seen.is_empty() {
                continue;
            }
            let body: String = String::from_utf8_lossy(seen.as_slice()).into_owned();
            let reply: String = format!("HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}", body.len());
            let _written = stream.write_all(reply.as_bytes());
            let _flushed = stream.flush();
        }
        return accepted;
    });
    return (port, served);
}

/// A TLS server is reached when its certificate chains to the authority the caller trusts,
/// with the bearer header on the
///  request,
/// and the same server is refused when the caller trusts only the embedded roots,
/// which do not know the authority.
/// Both calls reached the server:
/// a refused handshake is still a call that arrived,
/// and the count is how the control knows it.
#[test]
fn a_tls_server_is_reached_only_through_a_trusted_authority() {
    let root: std::path::PathBuf = crate::test_support::fixture("standards-review-tls");
    let (authority, certificate, key) = throwaway_authority_and_leaf(root.as_path());
    let (port, served) = serve_tls(certificate, key, TLS_CALLS);
    let url: String = format!("https://127.0.0.1:{port}/v1/decisions");
    let trusted: Gateway = Gateway { url: url.as_str(), api_key: KEY, quiet_window: ANSWER_WINDOW, ceiling: CONTROL_CEILING, trusted_pem: Some(authority.as_slice()) };
    let reply: Reply = post_decision(&trusted, "{\"a\":1}").expect("a trusted TLS server answers");
    assert_eq!(reply.status, 200);
    assert!(reply.body.to_ascii_lowercase().contains(&format!("authorization: bearer {KEY}")), "{}", reply.body);
    let refused: String = post_decision(&gateway(url.as_str(), ANSWER_WINDOW), "{}").expect_err("the embedded roots do not know the authority");
    assert!(refused.starts_with("fetch failed"), "{refused}");
    let accepted: usize = served.join().expect("the server thread");
    assert_eq!(accepted, TLS_CALLS, "the call the embedded roots refuse never reached the server within {CONNECTION_DEADLINE:?}");
    crate::test_support::remove(root.as_path());
}

/// What:
///  The quiet gap kept for a call is the call's own:
/// it comes from the bytes the call moved on the process's sockets,
/// not from the idle thread that waits for it.
/// Why:
///   Both halves are read from what a look at a waiting thread saw,
/// never from how long the host took:
/// a thread that moves nothing of its own is at work while the call's bytes move on the process's sockets,
///  and at rest when only its own counters are read.
/// The figure the measurement keeps is a stretch in which the call showed no sign of work,
/// so it can
///  never be shorter than the cadence of the bytes themselves.
#[test]
fn the_quiet_gap_kept_for_a_call_is_the_calls_own_and_not_the_idle_waiting_thread() {
    /// The pause between two body bytes,
    /// a fiftieth of the window judging them.
    const PAUSE: Duration = Duration::from_millis(100);
    /// How many body bytes the reply holds,
    /// so the call lasts many sampling intervals of the measurement.
    const BYTES: usize = 40;
    /// How long a control lets pass between two looks,
    /// so several bytes reach the process's socket between them.
    const BETWEEN_LOOKS: Duration = PAUSE.saturating_mul(3);
    let mock: Mock = Mock::start(Answer::Trickle { bytes: BYTES, pause: PAUSE });
    let url: String = mock.url.clone();
    let call: std::thread::JoinHandle<Result<Reply, String>> = std::thread::spawn(move || {
        return post_decision(&gateway(url.as_str(), TRICKLE_WINDOW), "{}");
    });
    // A thread that waits for nothing but its own channel: it moves no byte and takes no processor time,
    // which is what an idle waiting thread looks like.
    let (release, blocked) = std::sync::mpsc::channel::<()>();
    let (waiters, waiter_numbers) = std::sync::mpsc::channel::<i32>();
    let waiter: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        waiters.send(crate::thread_quiet::current_thread_id().expect("Linux names the thread")).expect("tell the control which thread waits");
        let _ = blocked.recv();
    });
    let waiter_tid: i32 = waiter_numbers.recv().expect("the waiting thread's number");
    let mut counting_sockets: crate::thread_quiet::ThreadSigns = crate::thread_quiet::ThreadSigns::new(waiter_tid, true);
    let mut own_counters_only: crate::thread_quiet::ThreadSigns = crate::thread_quiet::ThreadSigns::new(waiter_tid, false);
    // The first look of each has nothing to compare with, so it counts as work.
    assert!(counting_sockets.look() && own_counters_only.look());
    // The bytes reaching the process's socket are the event that the call is under way.
    let began: Instant = Instant::now();
    while !counting_sockets.look() {
        assert!(began.elapsed() < CONNECTION_DEADLINE, "the call's bytes never reached the process's socket within {CONNECTION_DEADLINE:?}");
        std::thread::sleep(CHILD_POLL);
    }
    std::thread::sleep(BETWEEN_LOOKS);
    assert!(counting_sockets.look(), "the bytes the call moved on the process's sockets were not the waiting thread's work");
    std::thread::sleep(BETWEEN_LOOKS);
    assert!(!own_counters_only.look(), "a thread that moved nothing of its own was at work on its own counters alone");
    let reply: Reply = call.join().expect("the call's thread ends").expect("the reply arrives");
    assert_eq!(reply.body.len(), BYTES, "the reply is read whole");
    assert_eq!(mock.finish().ended, AnswerEnded::Whole, "the call closed the reply before its last byte");
    drop(release);
    waiter.join().expect("the waiting thread ends");
    let figures: Vec<(String, Duration)> = crate::quiet_ledger::drain();
    let kept: Duration = figures.iter().find(|(step, _)| { return step == "standards review gateway call"; }).map(|(_, gap)| { return *gap; }).expect("the call keeps a figure");
    assert!(kept >= PAUSE, "the call kept a quiet gap of {kept:?} though a byte arrived every {PAUSE:?}");
}

/// How many body bytes one chunk of a chunked answer carries:
/// small,
/// so a refused answer is refused inside its stream and not at its end.
const CHUNK_BYTES: usize = 1024;

/// What:
///  A body of `bytes` bytes,
/// filling every byte but the first,
/// which is the brace a JSON answer opens with.
/// Why:
///   The control reads a length,
/// and the first byte keeps the text a body a reader could parse,
/// not a value whose own size is the subject.
fn body_of(bytes: usize) -> Vec<u8> {
    let mut body: Vec<u8> = vec![b'{'];
    body.resize(bytes, b'x');
    return body;
}

/// An answer exactly at the byte bound is read whole:
/// the probe that shows a refusal can tell a bound from a client that refuses everything.
#[test]
fn an_answer_at_the_byte_bound_is_read_whole() {
    let body: Vec<u8> = body_of(MAX_RESPONSE_BYTES);
    let text: String = String::from_utf8(body.clone()).expect("the body is text");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", text.as_str())));
    let url: String = mock.url.clone();
    let reply: Reply = post_decision(&gateway(url.as_str(), ANSWER_WINDOW), "{}").expect("an answer at the bound is read");
    assert_eq!((reply.status, reply.body.len()), (200, MAX_RESPONSE_BYTES));
    mock.finish();
}

/// An error answer past the byte bound is refused while it arrives,
/// with the bound named,
/// instead of being collected whole and excerpted afterwards.
#[test]
fn an_error_answer_past_the_byte_bound_is_refused_while_it_arrives() {
    let body: Vec<u8> = body_of(MAX_RESPONSE_BYTES + CHUNK_BYTES);
    let mock: Mock = Mock::start(Answer::Chunked { body, chunk: CHUNK_BYTES });
    let url: String = mock.url.clone();
    match post_decision(&gateway(url.as_str(), ANSWER_WINDOW), "{}") {
        Ok(reply) => panic!("an answer past the bound was collected whole, {} bytes", reply.body.len()),
        Err(refusal) => assert!(refusal.contains(&format!("{MAX_RESPONSE_BYTES}")), "the refusal names the bound: {refusal}"),
    }
    mock.finish();
}
