//! Parse the arguments of the `record` control verb.

/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `Command` from `super`.
use super::Command;

/// Parse the arguments of a `record` command (`<dir> [fps] [format]` or `stop`).
///
/// What:
///      `pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command,
///           String>`.
///  The first token is either `stop` (stop recording) or the output
///           directory,
///  optionally followed by an fps and a format name.
/// Why:
///       Isolate the recorder command grammar.
pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    // What:     `let first = tokens.next().ok_or_else(|| "record requires a directory or
    //           'stop'".to_string())?;`. Require the first token.
    // Why:      A bare `record` is ambiguous.
    let first = tokens
        .next()
        .ok_or_else(|| return "record requires a directory or 'stop'".to_string())?;

    // What:     `if first == "stop" { if tokens.next().is_some() { return Err(...); } return
    //           Ok(Command::RecordStop); }`. Handle the stop form (which takes no more args).
    // Why:      `record stop` ends the recording.
    if first == "stop" {
        if tokens.next().is_some() {
            return Err("record stop takes no arguments".to_string());
        }
        return Ok(Command::RecordStop);
    }

    // What:     `let dir = PathBuf::from(first);`. The output directory (no spaces).
    // Why:      Where the frame sequence is written.
    let dir = PathBuf::from(first);

    // What:     `let fps = match tokens.next() { Some(text) => text.parse::<f64>()
    //           .map_err(|_| "record fps is not a number".to_string())?, None => 60.0 };`.
    //           Optional fps, defaulting to 60.
    // Why:      The requested capture rate.
    let fps = match tokens.next() {
        Some(text) => text
            .parse::<f64>()
            .map_err(|_| return "record fps is not a number".to_string())?,
        None => 60.0,
    };

    // What:     `let format = tokens.next().unwrap_or("png").to_string();`. Optional format
    //           name, defaulting to `png`; validated later by the encoder.
    // Why:      Choose the output codec.
    let format = tokens.next().unwrap_or("png").to_string();

    // What:     `if tokens.next().is_some() { return Err(...); }`. Reject extra tokens.
    // Why:      `record` takes at most dir, fps, format.
    if tokens.next().is_some() {
        return Err("record takes at most a directory, fps, and format".to_string());
    }

    // What:     `Ok(Command::Record { dir, fps, format })`. Build the command (tail
    //           expression).
    // Why:      Return the parsed record request.
    return Ok(Command::Record { dir, fps, format })
}
