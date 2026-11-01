//! Deterministic local Responses endpoint;
//! no model or external network calls.

/// What:
/// `use` imports I/O,
/// networking,
/// time,
/// and JSON names.
/// Why:
/// The native host can execute its real tool loop using fixed test responses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { createServer } from 'node:http';
/// ```
use std::{io::{self, BufRead, BufReader, Read, Write}, net::TcpListener, time::Duration};
/// Import the fixed-fixture parser and dynamic values for field assignment.
use crate::rpc::parse;
/// JSON scalars own their data independently of the transmitted text.
use serde_json::Value;

/// What:
/// `pub fn` exports a function;
/// `usize` counts collection sizes and loop iterations.
/// Siblings u32/u64 fix integer width;
/// i32/i64 also permit negative values.
/// Why:
/// The fixture exchanges a bounded number of indexed responses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function serve(listener: Server, count: number): void { ... }
/// ```
pub fn serve(listener: TcpListener, count: usize) -> io::Result<()> {
    for index in 0..=count {
        // What: `?` propagates socket errors; `mut` permits writing the accepted connection.
        // Why: Each request receives a complete SSE response and closes its connection.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const socket = listener.accept();
        // ```
        let (mut stream, _address) = listener.accept()?;
        // Some supplies a timeout; None would permit an unlimited read.
        stream.set_read_timeout(Some(Duration::from_secs(20)))?;
        // Borrow the socket for buffered request parsing without transferring ownership.
        let mut reader = BufReader::new(&stream);
        let mut length = 0;
        loop {
            // Own a growable line; borrowed &str cannot be filled by read_line.
            let mut line = String::new();
            // Lend the buffer exclusively while reading it.
            if reader.read_line(&mut line)? == 0 {
                // Err is the failure variant paired with successful Ok.
                return Err(io::Error::other("local provider request ended before its headers"));
            }
            if line == "\r\n" { break; }
            // if let extracts a present Option prefix; absent prefixes continue the loop.
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = value.trim().parse::<usize>().map_err(io::Error::other)?;
            }
        }
        // Allocate exactly the announced request length; vec! creates owned repeated elements.
        let mut body = vec![0; length];
        // Read the complete body before responding so the client's upload finishes normally.
        reader.read_exact(&mut body)?;
        if index > 0 {
            // Decode request text losslessly to confirm the previous tool actually ran.
            let request = String::from_utf8(body).map_err(io::Error::other)?;
            assert!(request.contains("host-payload-executed"), "{request}");
        }
        // Decode fixed syntax; field assignment supplies dynamic values without source interpolation.
        let mut events: Vec<Value> = vec![parse(r#"{"type":"response.created","response":{"id":"fixture"}}"#)?];
        if index < count {
            // format! creates the distinct call ID required by the real host tool loop.
            let call_id = format!("fixture-{index}");
            // Raw string literals retain JSON backslashes; Value assignment owns the dynamic call ID.
            let mut event = parse(r#"{"type":"response.output_item.done","item":{
                "type":"function_call","call_id":"","name":"exec_command",
                "arguments":"{\"cmd\":\"printf host-payload-executed\",\"login\":false,\"max_output_tokens\":100}"
            }}"#)?;
            event["item"]["call_id"] = Value::from(call_id);
            events.push(event);
        } else {
            events.push(parse(r#"{"type":"response.output_item.done","item":{
                "type":"message","role":"assistant","id":"done",
                "content":[{"type":"output_text","text":"fixture completed"}]
            }}"#)?);
        }
        events.push(parse(r#"{"type":"response.completed","response":{"id":"fixture",
            "usage":{"input_tokens":0,"output_tokens":0,"total_tokens":0}}}"#)?);
        // Build the SSE body in owned text so Content-Length describes exact transmitted bytes.
        let mut response = String::new();
        for event in events {
            response.push_str("data: ");
            // Borrow the encoded event while appending it.
            response.push_str(&event.to_string());
            response.push_str("\n\n");
        }
        let header = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", response.len());
        stream.write_all(header.as_bytes())?;
        stream.write_all(response.as_bytes())?;
    }
    // Return successful completion after the bounded response sequence.
    return Ok(());
}
