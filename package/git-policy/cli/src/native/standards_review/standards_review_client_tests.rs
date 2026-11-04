//! Controls for the gateway client against a loopback mock. No test reaches a real host or sends a real key.

use super::*;
use crate::standards_review_mock::{Answer, Mock, SILENT_AT_MOST, http_reply};
use std::time::Instant;

/// A stand-in credential; it names no real key.
const KEY: &str = "test-key-not-a-secret";

/// A quiet window long enough for loopback and short enough for a timeout test.
const SHORT: Duration = Duration::from_millis(400);

/// A gateway at `url` with the given deadline.
fn gateway(url: &str, quiet_window: Duration) -> Gateway<'_> {
    return Gateway { url, api_key: KEY, quiet_window, trusted_pem: None };
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
