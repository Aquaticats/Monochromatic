//! Request and notification handling:
//!  one blocking read loop on the main thread.

/// Text mirroring and position arithmetic.
use crate::document::{apply_change, line_at, offset_at};
/// Framing,
///  the report,
///  and pending client replies.
use crate::framing::{Wire, read_message};
/// Settings of this run.
use crate::script::{Hover, Init, Script, Step, step_for};
/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// JSON values and the literal-building macro.
use serde_json::{Value, json};
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
use std::{collections::HashMap, io, path::Path, sync::Arc, thread, time::Duration};

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
///  Wait until a file exists,
///  checking every 5 ms,
///  for at most one minute.
///  `&Path` lends
///       the path (sibling:
///  owned `PathBuf`).
/// Why:
///  The read loop holds `initialize` until the test allows the start;
///  a test that never
///      creates the file still ends,
///  because the client's start deadline stops the server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitFor(gate: string) { for (let i = 0; i < 12_000 && !existsSync(gate); i++) sleepSync(5); }
/// ```
fn wait_for(gate: &Path) {
    for _ in 0..12_000 {
        if gate.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(5));
    }
}

/// Capabilities announced in the `initialize` answer.
fn capabilities(script: &Script) -> Value {
    // `mut` allows adding optional members below.
    let mut result = json!({ "hoverProvider": true });
    if let Some(encoding) = script.encoding {
        result["positionEncoding"] = json_value!(match encoding {
            crate::script::Unit::Utf8 => "utf-8",
            crate::script::Unit::Utf16 => "utf-16",
            crate::script::Unit::Utf32 => "utf-32",
        });
    }
    if let Some(change) = script.sync {
        result["textDocumentSync"] = json_value!({ "openClose": true, "change": change });
        if script.save {
            result["textDocumentSync"]["save"] = json!(true);
        }
    }
    if script.all_features {
        result["definitionProvider"] = json!(true);
        result["referencesProvider"] = json!(true);
        result["inlayHintProvider"] = json!({ "resolveProvider": true });
    }
    if script.pull_diagnostics {
        result["diagnosticProvider"] = json_value!({ "identifier": "scripted", "interFileDependencies": false, "workspaceDiagnostics": false });
    }
    return result;
}

/// Session behavior.
impl Session {
    /// What:
    ///  Send a message,
    ///  turning a write failure into a line on standard error.
    /// Why:
    ///  A closed pipe ends the loop at the next read;
    ///  one failed write needs no other handling.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// send(message: object) { try { wire.send(message); } catch (error) { console.error(error); } }
    /// ```
    fn send(&self, message: Value) {
        if let Err(error) = self.wire.send(message) {
            eprintln!("scripted language server cannot write: {error}");
        }
    }

    /// Record the server's copy of a document and,
    ///  when configured,
    ///  push diagnostics that quote it.
    fn publish(&self, uri: &str, version: &Value) {
        // `get` returns `Option<&String>`; `map_or` substitutes empty text for an unknown document.
        let text = match self.documents.get(uri) {
            Some(found) => found.as_str(),
            None => "",
        };
        self.wire
            .record(json_value!({ "text": { "uri": uri, "version": version, "text": text } }));
        if !self.script.push_diagnostics {
            return;
        }
        let mut params = json_value!({
            "uri": uri,
            "diagnostics": [{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } },
                "severity": 2,
                "source": "scripted",
                "message": format!("TEXT:{text}"),
            }],
        });
        if self.script.versioned_diagnostics {
            // `clone` copies the JSON number so the notification owns its own value.
            params["version"] = version.clone();
        }
        self.send(json_value!({ "method": "textDocument/publishDiagnostics", "params": params }));
    }

    /// What:
    ///  Answer a hover request according to the script.
    ///  `&mut self` allows counting requests.
    /// Why:
    ///  Hover is the request tests use for delay,
    ///  crash,
    ///  supersession,
    ///  and silence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hover(id: unknown, params: HoverParams): void
    /// ```
    fn hover(&mut self, id: &Value, params: &Value) {
        self.hovers += 1;
        let modified = self.script.hover == Hover::Modified
            || (self.script.hover == Hover::ModifiedOnce && self.hovers == 1);
        if self.script.hover == Hover::Crash {
            // The configured line comes right before the end, as a crashing server's last words.
            if let Some(line) = &self.script.stderr_at_shutdown {
                eprintln!("{line}");
            }
            std::process::exit(7);
        }
        if self.script.hover == Hover::Silent {
            return;
        }
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let text = match self.documents.get(uri) {
            Some(found) => found.as_str(),
            None => "",
        };
        let position = &params["position"];
        let offset = offset_at(
            text,
            position["line"].as_u64().unwrap_or(0),
            position["character"].as_u64().unwrap_or(0),
            self.script.unit(),
        );
        let line = line_at(text, offset);
        let answer = if modified {
            // The answer a server gives when its own state moved under the request.
            json_value!({ "id": id, "error": { "code": -32801, "message": "content modified" } })
        } else if line.trim().is_empty() {
            json_value!({ "id": id, "result": null })
        } else {
            // `chars().next()` reads the character at the offset, if any; `map_or` renders it as text.
            let character = match text[offset..].chars().next() {
                Some(found) => found.to_string(),
                None => String::new(),
            };
            json_value!({ "id": id, "result": {
                "contents": { "kind": "markdown", "value": format!("line={line} char={character}") },
                "range": { "start": position, "end": position },
            } })
        };
        if self.script.hover_delay == 0 {
            self.send(answer);
            if self.script.push_after_hover {
                // An unversioned push that arrives after an answer, as delayed analysis produces.
                self.publish(uri, &Value::Null);
            }
            return;
        }
        // What: `Arc::clone` copies the shared pointer; `thread::spawn(move || ...)` runs the
        //       closure on a new thread, moving the captured values into it.
        // Why: The answer was computed from the text at request time and is sent late, so a
        //      change notification can overtake it, which is the stale-reply case.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // setTimeout(() => wire.send(answer), hoverDelay);
        // ```
        let wire = Arc::clone(&self.wire);
        let delay = Duration::from_millis(self.script.hover_delay);
        thread::spawn(move || {
            thread::sleep(delay);
            if let Err(error) = wire.send(answer) {
                eprintln!("scripted language server cannot write a delayed answer: {error}");
            }
        });
    }

    /// Answer one request from the client.
    fn request(&mut self, id: &Value, method: &str, params: &Value) {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        if method == "initialize" {
            match self.script.init {
                Init::Hang => {}
                Init::Exit => std::process::exit(3),
                Init::Error => self.send(
                    json_value!({ "id": id, "error": { "code": -32603, "message": "scripted initialize failure" } }),
                ),
                Init::Ok => {
                    // A gated start waits for the test's file, for at most one minute.
                    if let Some(gate) = &self.script.init_gate {
                        wait_for(gate);
                    }
                    // A slow start: the client must not send anything else in the meantime.
                    thread::sleep(Duration::from_millis(self.script.init_delay));
                    self.send(json_value!({ "id": id, "result": {
                        "capabilities": capabilities(&self.script),
                        "serverInfo": { "name": "ide-scripted-lsp" },
                    } }));
                }
            }
        } else if method == "shutdown" {
            // `if let Some` writes the configured line only when one was given.
            if let Some(line) = &self.script.stderr_at_shutdown {
                eprintln!("{line}");
            }
            self.send(json_value!({ "id": id, "result": null }));
        } else if method == "textDocument/hover" {
            self.hover(id, params);
        } else if method == "textDocument/definition"
            && let Some(result) = &self.script.definition
        {
            // A scripted result is sent exactly as the test wrote it.
            self.send(json_value!({ "id": id, "result": result }));
        } else if method == "textDocument/references"
            && let Some(result) = &self.script.references
        {
            self.send(json_value!({ "id": id, "result": result }));
        } else if method == "textDocument/definition" {
            let range = json_value!({ "start": { "line": 2, "character": 0 }, "end": { "line": 2, "character": 6 } });
            let mut targets = vec![
                json_value!({ "uri": uri, "range": range }),
                json_value!({ "uri": "untitled:Untitled-1", "range": range }),
                json_value!({ "uri": "jdt://contents/java.base/java/lang/String.class", "range": range }),
                json_value!({ "uri": "file:///definitely/missing/file.rs", "range": range }),
            ];
            if let Some(outside) = &self.script.outside {
                targets.push(json_value!({
                    "uri": format!("file://{}", outside.display()),
                    "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } },
                }));
            }
            self.send(json_value!({ "id": id, "result": targets }));
        } else if method == "textDocument/references" {
            self.send(json_value!({ "id": id, "error": { "code": -32603, "message": "scripted internal failure" } }));
        } else if method == "textDocument/inlayHint" {
            self.inlay_hints += 1;
            // `&self.script.hint_steps` lends the scripted list; the count picks this request's step.
            let step = step_for(&self.script.hint_steps, self.inlay_hints);
            // `&mut self.held_hint` lends the slot of a held-back answer, which `respond` fills or empties.
            let first_line = params["range"]["start"]["line"].as_u64().unwrap_or(0);
            crate::inlay::respond(&self.wire, id, step, &mut self.held_hint, first_line);
        } else if method == "textDocument/diagnostic" {
            self.pulls += 1;
            // A scripted silence leaves this pull unanswered; the client's timeout decides.
            if step_for(&self.script.pull_steps, self.pulls) == Step::Silent {
                return;
            }
            let text = match self.documents.get(uri) {
                Some(found) => found.as_str(),
                None => "",
            };
            self.send(json_value!({ "id": id, "result": { "kind": "full", "items": [{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } },
                "severity": 1,
                "source": "scripted-pull",
                "message": format!("PULLED:{text}"),
            }] } }));
        } else {
            self.send(json_value!({ "id": id, "error": { "code": -32601, "message": format!("scripted server has no {method}") } }));
        }
    }

    /// React to one notification from the client.
    fn notification(&mut self, method: &str, params: &Value) {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        let version = &params["textDocument"]["version"];
        if method == "exit" {
            // A lingering server stays; the client then has to kill it.
            if !self.script.linger {
                std::process::exit(0);
            }
        } else if method == "textDocument/didOpen" {
            let text = params["textDocument"]["text"].as_str().unwrap_or("");
            self.documents.insert(uri.to_string(), text.to_string());
            self.publish(uri, version);
            if self.script.probe && !self.probed {
                self.probed = true;
                let wire = Arc::clone(&self.wire);
                let target = uri.to_string();
                thread::spawn(move || {
                    crate::probe::run(&wire, &target);
                });
            }
        } else if method == "textDocument/didChange" {
            // `entry(...).or_default()` borrows the stored text for editing, creating empty text first if absent.
            let text = self.documents.entry(uri.to_string()).or_default();
            // `as_array()` is `Some` for a JSON array; `into_iter().flatten()` walks its items or nothing.
            for change in params["contentChanges"].as_array().into_iter().flatten() {
                apply_change(text, change, self.script.unit());
            }
            self.publish(uri, version);
        } else if method == "textDocument/didClose" {
            self.documents.remove(uri);
        } else if method == "initialized"
            && let Some(watchers) = self.script.watchers.clone()
        {
            // Registered from its own thread, because the reply arrives through this read loop.
            let wire = Arc::clone(&self.wire);
            thread::spawn(move || {
                let reply = wire.ask(
                    "client/registerCapability",
                    json_value!({ "registrations": [{ "id": "scripted-watchers", "method": "workspace/didChangeWatchedFiles", "registerOptions": { "watchers": watchers } }] }),
                );
                wire.record(json_value!({ "registered-watchers": reply }));
            });
        }
    }
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
