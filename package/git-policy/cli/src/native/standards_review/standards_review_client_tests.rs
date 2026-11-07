//! Controls for the gateway client against a loopback mock. No test reaches a real host or sends a real key.

use super::*;
use crate::standards_review_mock::{Answer, Mock, SILENT_AT_MOST, http_reply};
use std::time::Instant;

/// A stand-in credential; it names no real key.
const KEY: &str = "test-key-not-a-secret";

/// A quiet window long enough for loopback and short enough for a timeout test.
const SHORT: Duration = Duration::from_millis(400);

/// A ceiling no control reaches unless the control is about the ceiling.
const ROOMY: Duration = Duration::from_secs(3600);

/// A gateway at `url` with the given deadline.
fn gateway(url: &str, quiet_window: Duration) -> Gateway<'_> {
    return Gateway { url, api_key: KEY, quiet_window, ceiling: ROOMY, trusted_pem: None };
}

/// The request has the method, path, JSON content type, bearer header and the exact body bytes.
#[test]
fn the_request_carries_method_headers_and_body() {
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", "{}")));
    let url: String = mock.url.clone();
    let body: &str = "{\"input\":\"text with \\\"quotes\\\" and \u{e9}\"}";
    let reply: Reply = post_decision(&gateway(url.as_str(), SHORT), body).expect("reply");
    assert_eq!(reply, Reply { status: 200, body: String::from("{}") });
    let seen: String = mock.finish();
    let lowered: String = seen.to_ascii_lowercase();
    assert!(seen.starts_with("POST /v1/decisions HTTP/1.1\r\n"), "{seen}");
    assert!(lowered.contains("content-type: application/json\r\n"));
    assert!(lowered.contains(&format!("authorization: bearer {KEY}\r\n")));
    assert!(lowered.contains(&format!("content-length: {}\r\n", body.len())));
    assert!(seen.ends_with(body));
}

/// An error status is a reply with its body, not a client error.
#[test]
fn an_error_status_is_a_reply() {
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("503 Service Unavailable", "busy")));
    let url: String = mock.url.clone();
    let reply: Reply = post_decision(&gateway(url.as_str(), SHORT), "{}").expect("reply");
    assert_eq!((reply.status, reply.body.as_str()), (503, "busy"));
    mock.finish();
}

/// A chunked body is reassembled.
#[test]
fn a_chunked_body_is_read_whole() {
    let chunked: Vec<u8> = b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\nconnection: close\r\n\r\n3\r\nabc\r\n2\r\nde\r\n0\r\n\r\n".to_vec();
    let mock: Mock = Mock::start(Answer::Bytes(chunked));
    let url: String = mock.url.clone();
    let reply: Reply = post_decision(&gateway(url.as_str(), SHORT), "{}").expect("reply");
    assert_eq!(reply.body, "abcde");
    mock.finish();
}

/// A redirect is returned as a status; its target is never contacted.
#[test]
fn a_redirect_is_not_followed() {
    let redirect: Vec<u8> = b"HTTP/1.1 302 Found\r\nlocation: http://127.0.0.1:1/elsewhere\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_vec();
    let mock: Mock = Mock::start(Answer::Bytes(redirect));
    let url: String = mock.url.clone();
    let reply: Reply = post_decision(&gateway(url.as_str(), SHORT), "{}").expect("reply");
    assert_eq!(reply.status, 302);
    mock.finish();
}

/// A server that never answers ends the call when it has been silent for the quiet window, with the timeout wording, not later.
/// The server holds the connection until the call closes it, so only the window can end the call in time.
#[test]
fn a_silent_server_ends_after_the_quiet_window() {
    let mock: Mock = Mock::start(Answer::Silent(SILENT_AT_MOST));
    let url: String = mock.url.clone();
    let started: Instant = Instant::now();
    let failure: String = post_decision(&gateway(url.as_str(), SHORT), "{}").expect_err("timeout");
    let elapsed: Duration = started.elapsed();
    assert_eq!(failure, "The operation was aborted due to timeout");
    assert!(elapsed >= SHORT, "{elapsed:?}");
    // No upper bound by the clock:
    // the server's silence outlasts the deadline by a multiple,
    // so a call that did not end at the
    //  deadline ends with the server's close,
    // whose wording differs from the timeout's.
    mock.finish();
}

/// A refused connection is a transport failure that names its cause.
#[test]
fn a_refused_connection_names_the_cause() {
    let mock: Mock = Mock::start(Answer::Bytes(Vec::new()));
    let url: String = mock.url.clone();
    // The mock serves one connection; take it so the port is closed for the call under test.
    let _ = std::net::TcpStream::connect(url.trim_start_matches("http://").trim_end_matches("/v1/decisions"));
    mock.finish();
    let failure: String = post_decision(&gateway(url.as_str(), SHORT), "{}").expect_err("refused");
    assert!(failure.starts_with("fetch failed: "), "{failure}");
}

/// A trusted authority that is not PEM is refused before any connection is made.
#[test]
fn an_unreadable_authority_is_refused() {
    let mut bad: Gateway = gateway("https://127.0.0.1:1/", SHORT);
    bad.trusted_pem = Some(b"not a certificate");
    assert!(post_decision(&bad, "{}").expect_err("authority").starts_with("the trusted authority is unreadable"));
}

/// A reply that keeps arriving is never cut off however long it takes: the window judges quiet time, not wall time.
///  The mock sends one byte a pause for a multiple of the window, so it is never silent for the window.
#[test]
fn a_reply_that_keeps_arriving_is_never_cut_off() {
    let window: Duration = Duration::from_millis(1200);
    let pause: Duration = Duration::from_millis(200);
    let bytes: usize = 15;
    let mock: Mock = Mock::start(Answer::Trickle { bytes, pause });
    let url: String = mock.url.clone();
    let started: Instant = Instant::now();
    let reply: Reply = post_decision(&gateway(url.as_str(), window), "{}").expect("a reply that keeps arriving is read whole");
    assert_eq!(reply.body.len(), bytes);
    assert!(started.elapsed() > window * 2, "the reply took only {:?}", started.elapsed());
    mock.finish();
}

/// Owner, 2026-11-04, "Silence plus a ceiling (Recommended)": a reply that keeps arriving is ended at the ceiling all the same, with
/// an error that names the call, the ceiling and what to run. The mock sends one byte a pause, so it is never silent for the window.
#[test]
fn a_reply_that_keeps_arriving_is_ended_at_the_ceiling() {
    let window: Duration = Duration::from_millis(1200);
    let ceiling: Duration = Duration::from_millis(1500);
    let mock: Mock = Mock::start(Answer::Trickle { bytes: 40, pause: Duration::from_millis(200) });
    let url: String = mock.url.clone();
    let started: Instant = Instant::now();
    let failure: String = post_decision(&Gateway { ceiling, ..gateway(url.as_str(), window) }, "{}").expect_err("the ceiling ended the call");
    let elapsed: Duration = started.elapsed();
    assert!(failure.contains("standards-review gateway call") && failure.contains("1 seconds") && failure.contains("shadow-review"), "{failure}");
    assert!(elapsed >= ceiling && elapsed < ceiling * 4, "{elapsed:?}");
    mock.finish();
}

/// The client is built on `hyper` and no longer on `ureq` (owner, 2026-11-07: "Adopt tokio, do not adopt ureq, use the `hyper`
///  set of https libs."), and reads the network on the process's runtime.
#[test]
fn the_client_is_built_on_hyper_and_no_longer_on_ureq() {
    let source: &str = include_str!("standards_review_client.rs");
    let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    assert!(!code.contains("ureq"), "the client still names `ureq`");
    assert!(code.contains("hyper_util::"), "the client does not use `hyper`");
}

/// A throwaway authority and a leaf it signed for `127.0.0.1`, made by the `openssl` tool in `directory`; returns the authority's
///  certificate, the leaf's certificate and the leaf's private key, all as PEM text.
fn throwaway_authority_and_leaf(directory: &std::path::Path) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let run = |arguments: &[&str]| {
        let status = std::process::Command::new("openssl").args(arguments).current_dir(directory).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().expect("start openssl");
        assert!(status.success(), "openssl {arguments:?}");
    };
    std::fs::write(directory.join("leaf.ext"), "subjectAltName=IP:127.0.0.1\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\n").expect("extensions");
    run(&["req", "-x509", "-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:prime256v1", "-nodes", "-keyout", "ca-key.pem", "-out", "ca.pem", "-days", "2", "-subj", "/CN=throwaway-authority", "-addext", "basicConstraints=critical,CA:TRUE", "-addext", "keyUsage=critical,keyCertSign"]);
    run(&["req", "-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:prime256v1", "-nodes", "-keyout", "key.pem", "-out", "leaf.csr", "-subj", "/CN=throwaway-leaf"]);
    run(&["x509", "-req", "-in", "leaf.csr", "-CA", "ca.pem", "-CAkey", "ca-key.pem", "-CAcreateserial", "-out", "cert.pem", "-days", "2", "-extfile", "leaf.ext"]);
    let read = |name: &str| { return std::fs::read(directory.join(name)).expect("a generated file"); };
    return (read("ca.pem"), read("cert.pem"), read("key.pem"));
}

/// A TLS server on loopback that answers each request it reads with a 200 whose body is the request head, for `connections`
///  connections, ignoring the ones whose handshake fails; returns the port.
fn serve_tls(certificate: Vec<u8>, key: Vec<u8>, connections: usize) -> (u16, std::thread::JoinHandle<()>) {
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
    let served = std::thread::spawn(move || {
        for _ in 0..connections {
            let (mut tcp, _) = listener.accept().expect("a client");
            tcp.set_read_timeout(Some(Duration::from_secs(10))).expect("read timeout");
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
    });
    return (port, served);
}

/// A TLS server is reached when its certificate chains to the authority the caller trusts, with the bearer header on the
///  request, and the same server is refused when the caller trusts only the embedded roots, which do not know the authority.
#[test]
fn a_tls_server_is_reached_only_through_a_trusted_authority() {
    let root: std::path::PathBuf = crate::test_support::fixture("standards-review-tls");
    let (authority, certificate, key) = throwaway_authority_and_leaf(root.as_path());
    let (port, served) = serve_tls(certificate, key, 2);
    let url: String = format!("https://127.0.0.1:{port}/v1/decisions");
    let trusted: Gateway = Gateway { url: url.as_str(), api_key: KEY, quiet_window: Duration::from_secs(5), ceiling: ROOMY, trusted_pem: Some(authority.as_slice()) };
    let reply: Reply = post_decision(&trusted, "{\"a\":1}").expect("a trusted TLS server answers");
    assert_eq!(reply.status, 200);
    assert!(reply.body.to_ascii_lowercase().contains(&format!("authorization: bearer {KEY}")), "{}", reply.body);
    let refused: String = post_decision(&gateway(url.as_str(), Duration::from_secs(5)), "{}").expect_err("the embedded roots do not know the authority");
    assert!(refused.starts_with("fetch failed"), "{refused}");
    served.join().expect("the server thread");
    crate::test_support::remove(root.as_path());
}
