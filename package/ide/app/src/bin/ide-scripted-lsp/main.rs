//! Scripted language server for integration tests:
//!  it speaks the protocol over standard streams
//! and produces,
//!  on demand,
//!  the states real servers do not produce reliably.
//!
//! It is configured only through `IDE_SCRIPTED_*` environment variables,
//!  which tests set through
//! the server definition's `environment` table.
//!  It never writes anything except the report file
//! a test names,
//!  and that file lives in the test's own temporary directory.

/// What the server can see and change from inside a sandbox.
mod audit;
/// Text mirroring and position arithmetic in the advertised column unit.
mod document;
/// Message framing on standard input and output,
///  plus the report file.
mod framing;
/// The inlay-hint answer and its optional delay.
mod inlay;
/// Requests this server sends to the client,
///  to check the client's replies.
mod probe;
/// Environment-variable settings.
mod script;
/// Request and notification handling.
mod server;

/// What:
///  `fn main()` is the program entry point;
///  it returns nothing.
/// Why:
///  All behavior lives in `server::run`;
///  a failure there is printed to standard error,
///  which
///      the client logs,
///  and the process exits with a failure status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { run(readScript()); } catch (error) { console.error(error); process.exit(1); }
/// ```
fn main() {
    let script = script::Script::from_environment();
    // What: `if let Ok(line) = ...` runs the block only when `IDE_SCRIPTED_STDERR_AT_START` is set.
    // Why: A server often reports its start on standard error; a test of the client's log needs a
    //      line that arrives before `initialize` is answered or refused.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const line = env.IDE_SCRIPTED_STDERR_AT_START; if (line !== undefined) console.error(line);
    // ```
    if let Ok(line) = std::env::var("IDE_SCRIPTED_STDERR_AT_START") {
        eprintln!("{line}");
    }
    // What: `if let Err(error) = ...` runs the block only when the call failed.
    // Why: A broken pipe or malformed message must end the process visibly, not silently.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { run(script); } catch (error) { console.error(error); process.exit(1); }
    // ```
    if let Err(error) = server::run(script) {
        eprintln!("scripted language server stopped: {error}");
        std::process::exit(1);
    }
}
