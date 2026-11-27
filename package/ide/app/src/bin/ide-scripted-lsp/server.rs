//! Request and notification handling:
//!  one blocking read loop on the main thread.

/// Framing,
///  the report,
///  and pending client replies.
use crate::framing::{
    Wire,
    read_message,
};
/// Settings of this run.
use crate::script::Script;
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// The answer a delayed request carries,
///  and the message every pass through the loop records.
use serde_json::Value;
/// What:
///  `HashMap` is a key-value table;
///  `Arc` is a thread-safe shared pointer (siblings:
///  `Rc`
///       for one thread,
///  `Box` for one owner);
///  `thread` starts helper threads.
/// Why:
///  A delayed hover answer and the client probe must not block the read loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const documents = new Map<string, string>();
/// ```
use std::{
    collections::HashMap,
    io,
    sync::Arc,
    thread,
    time::Duration,
};

/// What each request the client sends is answered with.
mod answers;
/// What each notification the client sends does.
mod notifications;
/// The dispatch of one request.
mod requests;

/// What:
///  The loop's own state.
///  `String` owns its text (sibling:
///  borrowed `&str`).
/// Why:
///  Only the main thread reads and edits documents,
///  so no lock is needed for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Session { documents = new Map<string, string>(); probed = false; hovers = 0; }
/// ```
struct Session {
    /// Settings of this run.
    script: Script,
    /// Shared output stream,
    ///  report,
    ///  and reply table.
    wire: Arc<Wire>,
    /// Text of every open document by address.
    documents: HashMap<String, String>,
    /// Whether the client probe was already started.
    probed: bool,
    /// Number of hover requests seen.
    hovers: u64,
    /// Number of inlay-hint requests seen.
    inlay_hints: u64,
    /// Number of pull-diagnostics requests seen.
    pulls: u64,
    /// What:
    ///  The answer to a held-back hint request while it waits.
    ///  `Option<Value>` is
    ///       "a JSON answer,
    ///  or nothing".
    /// Why:
    ///  It is written right after the next hint answer.
    held_hint: Option<Value>,
}

/// What:
///  Run until the client closes standard input or sends `exit`.
///  `io::Result<()>` is success
///       without a value or an I/O error.
/// Why:
///  Every received message is recorded before it is handled,
///  so the report shows the exact
///      order the client produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const message of messages(process.stdin)) { record(message); handle(message); }
/// ```
pub fn run(script: Script) -> io::Result<()> {
    // `as_deref()` turns `Option<PathBuf>` into `Option<&Path>` without copying.
    let wire = Arc::new(Wire::new(script.report.as_deref())?);
    if let Ok(writes) = std::env::var("IDE_SCRIPTED_AUDIT") {
        // Recorded before anything else, so a sandbox audit exists even if the session fails.
        wire.record(json_value!({ "audit": crate::audit::audit(&writes) }));
    }
    wire.record(json_value!({ "started": { "cwd": std::env::current_dir()?.display().to_string(), "pid": std::process::id() } }));
    let mut session = Session {
        script,
        wire: Arc::clone(&wire),
        documents: HashMap::new(),
        probed: false,
        hovers: 0,
        inlay_hints: 0,
        pulls: 0,
        held_hint: None,
    };
    // `lock()` on standard input returns a buffered reader this thread owns.
    let mut input = io::stdin().lock();
    // `while let Some(x) = ...` repeats until the call yields "nothing", which is end of input.
    while let Some(message) = read_message(&mut input)? {
        let method = message["method"].as_str();
        let id = &message["id"];
        match method {
            Some(name) => {
                session.wire.record(
                    json_value!({ "received": name, "id": id, "params": message["params"] }),
                );
                if id.is_null() {
                    session.notification(name, &message["params"]);
                } else {
                    session.request(id, name, &message["params"]);
                }
            }
            None => {
                let reply = if message["error"].is_null() {
                    json_value!({ "result": message["result"] })
                } else {
                    json_value!({ "error": message["error"] })
                };
                session.wire.resolve(id.as_u64().unwrap_or(0), reply);
            }
        }
    }
    // What: `loop` without a condition repeats forever; `thread::sleep` pauses this thread.
    // Why: A lingering server outlives the end of its input, as a stuck server would, until it
    //      is killed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (script.linger) for (;;) await sleep(3_600_000);
    // ```
    if session.script.linger {
        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    }
    // `Ok(())` reports success without a value.
    return Ok(());
}
