//! Line-oriented client for the installed Codex app-server protocol.

/// What:
/// `use` imports standard I/O and JSON types;
/// `::` separates their modules.
/// Why:
/// Exercise the real host without another SDK or provider dependency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readLine, writeLine } from './stdio';
/// ```
use std::{io::{self, BufRead, BufReader, Write}, process::{ChildStdin, ChildStdout}};
/// Import a dynamically shaped JSON value without panic-generating construction macros.
use serde_json::Value;

/// Decode a fixed JSON fixture,
/// preserving parse failures as test errors.
pub fn parse(input: &str) -> io::Result<Value> {
    return serde_json::from_str(input).map_err(io::Error::other);
}

/// What:
/// `pub fn` exports a function;
/// `&mut` lends exclusive write access to the pipe.
/// `Result<()>` carries success without data or an I/O error.
/// Why:
/// Send one complete protocol message without interleaving writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function send(stdin: Pipe, value: JsonValue): void { ... }
/// ```
pub fn send(pipe: &mut ChildStdin, value: Value) -> io::Result<()> {
    // What: `to_string` creates owned JSON text; `?` propagates errors to the test.
    // Why: Only serde_json may encode protocol boundaries.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // stdin.write(JSON.stringify(value));
    // ```
    pipe.write_all(value.to_string().as_bytes())?;
    pipe.write_all(b"\n")?;
    return pipe.flush();
}

/// Read and decode one host message,
/// rejecting EOF instead of spinning.
pub fn receive(reader: &mut BufReader<ChildStdout>) -> io::Result<Value> {
    // What: `String` owns growable text; `&str` would only borrow existing text.
    // Why: read_line fills a buffer whose final length is unknown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let line = '';
    // ```
    let mut line = String::new();
    // Lend the buffer to read_line, preserving it for JSON parsing.
    if reader.read_line(&mut line)? == 0 {
        // What: `Err` reports failure, unlike successful `Ok`.
        // Why: A closed host pipe cannot satisfy a pending RPC request.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // throw new Error('Codex app-server closed stdout');
        // ```
        return Err(io::Error::other("Codex app-server closed stdout; inspect host-stderr.log in the fixture"));
    }
    // Borrow the line for parsing and convert parse failures to test I/O errors.
    return serde_json::from_str(&line).map_err(io::Error::other);
}

/// Send a request and wait for its matching response,
/// allowing intervening notifications.
pub fn call(pipe: &mut ChildStdin, reader: &mut BufReader<ChildStdout>, id: u64, method: &str, params: Value) -> io::Result<Value> {
    // Parse fixed syntax, then assign values without interpolating them into JSON source.
    let mut request = parse("{\"id\":0,\"method\":\"\",\"params\":null}")?;
    // Value::from owns each JSON scalar; params already owns its JSON representation.
    request["id"] = Value::from(id);
    request["method"] = Value::from(method);
    request["params"] = params;
    send(pipe, request)?;
    loop {
        let message = receive(reader)?;
        // Some is a present optional ID; notifications have None and are skipped here.
        if message["id"].as_u64() == Some(id) {
            if !message["error"].is_null() {
                // Preserve the host's exact RPC error as owned diagnostic text.
                return Err(io::Error::other(message.to_string()));
            }
            // clone owns the result after the surrounding response is dropped.
            return Ok(message["result"].clone());
        }
    }
}
