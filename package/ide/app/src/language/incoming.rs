//! The read-only client's answer to every request a language server can send it.
//!
//! helix-lsp only delivers these requests.
//!  An unanswered one stalls the server,
//!  so each method
//! gets a reply here:
//!  what a server needs to keep working is answered,
//!  every mutation is refused.

/// What:
///  `MethodCall` is helix-lsp's parsed form of a server-to-client request;
///  `jsonrpc` holds
///       the wire-level request,
///  parameter,
///  and error types;
///  `lsp` holds the protocol's data types.
/// Why:
///  The reply must be shaped exactly as the protocol defines for each method.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MethodCall, jsonrpc, lsp } from 'helix-lsp';
/// ```
use helix_lsp::{MethodCall, jsonrpc, lsp};
/// What:
///  `Value` is any JSON value;
///  `json!` is a macro that builds one from literal syntax.
/// Why:
///  Replies travel as JSON,
///  and a few of them are fixed literals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::{Value, json};

/// The exact sentence sent with every refused workspace edit.
pub const EDIT_REFUSAL: &str = "read-only client: workspace edits are not applied";

/// What:
///  What the worker must do besides sending the reply.
///  An `enum` with data is a tagged union.
/// Why:
///  Deciding is kept free of side effects so every reply can be tested without a server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Effect =
///   | { kind: 'none' } | { kind: 'refreshDiagnostics' }
///   | { kind: 'registerWatchers'; registrations: [string, WatcherOptions][] }
///   | { kind: 'unregisterWatchers'; ids: string[] };
/// ```
#[derive(Debug, PartialEq)]
pub enum Effect {
    /// Nothing besides the reply.
    None,
    /// Pull diagnostics again for the displayed file.
    RefreshDiagnostics,
    /// Hand these file-watcher registrations to helix-lsp's file-event handler.
    ///  The tuple pairs
    /// a registration identifier with its options.
    RegisterWatchers(
        /// Registration identifier and options of each watcher.
        Vec<(String, lsp::DidChangeWatchedFilesRegistrationOptions)>,
    ),
    /// Remove these file-watcher registrations.
    UnregisterWatchers(
        /// Registration identifiers to remove.
        Vec<String>,
    ),
}

/// What:
///  The reply plus its side effect and a short policy word for the log.
///  `Result<Value,
///       jsonrpc::Error>` is either a JSON result (`Ok`) or a protocol error (`Err`);
///       `&'static str` is a string literal that lives for the whole program (sibling:
///  owned `String`).
/// Why:
///  The three parts are produced together and consumed together by the worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Decision = { reply: { result: unknown } | { error: RpcError }; effect: Effect; policy: string };
/// ```
#[derive(Debug, PartialEq)]
pub struct Decision {
    /// What is sent back to the server.
    pub reply: Result<Value, jsonrpc::Error>,
    /// What else the worker must do.
    pub effect: Effect,
    /// One of `answered`,
    ///  `acknowledged`,
    ///  `refused`,
    ///  `method-not-found`,
    ///  `malformed`.
    pub policy: &'static str,
}

/// What the decision needs to know about the client the request arrived on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClientView = { settings?: unknown; folders: WorkspaceFolder[] };
/// ```
pub struct ClientView<'a> {
    /// What:
    ///  `Option<&'a Value>` is "a borrowed JSON value,
    ///  or nothing".
    ///  The `'a` is a lifetime
    ///       name:
    ///  it states that the borrow cannot outlive the client it was taken from.
    /// Why:
    ///  The server's configured settings answer `workspace/configuration` without a copy.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// settings?: Readonly<unknown>;
    /// ```
    pub settings: Option<&'a Value>,
    /// Workspace folders helix-lsp sent in `initialize`.
    ///  `&'a [T]` is a borrowed list.
    pub folders: &'a [lsp::WorkspaceFolder],
}

/// What:
///  Look up one dotted section such as `typescript.inlayHints` in the settings.
/// Why:
///  `workspace/configuration` asks for sections of the same object that was sent as
///      `initializationOptions`;
///  an absent section must become JSON `null`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function section(settings: unknown, path?: string): unknown {
///   let current = settings;
///   for (const part of (path ?? '').split('.').filter(Boolean)) current = current?.[part];
///   return current ?? null;
/// }
/// ```
pub fn configuration_section(settings: Option<&Value>, section: Option<&str>) -> Value {
    // What: `let Some(mut x) = option else { ... }` binds the inner value or leaves the function;
    //       `mut` lets the loop move `current` deeper into the object.
    // Why: A server without configured settings gets `null` for every item.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (settings === undefined) return null;
    // ```
    let Some(mut current) = settings else {
        return Value::Null;
    };
    // Some servers send an empty section and mean the whole object.
    for part in section.unwrap_or("").split('.') {
        if part.is_empty() {
            continue;
        }
        // What: `get` returns `Option<&Value>`; `match` handles the found and the missing key.
        // Why: A missing key ends the walk with `null` rather than an error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!(part in current)) return null; current = current[part];
        // ```
        match current.get(part) {
            Some(next) => current = next,
            None => return Value::Null,
        }
    }
    // `clone` makes an independent copy, because the reply outlives the borrow of the settings.
    return current.clone();
}

/// Build a decision whose reply is a successful result.
fn result(value: Value, effect: Effect, policy: &'static str) -> Decision {
    return Decision {
        // `Ok(...)` is the success variant of `Result`.
        reply: Ok(value),
        effect,
        policy,
    };
}

/// What:
///  The result that refuses a workspace edit:
///  `applied` is false and `failureReason` carries the refusal sentence.
///  `Value::from` wraps the string.
/// Why:
///  `json!` unwraps a `Result` when it expands an interpolated value,
///  and the repository's clippy configuration bans that call.
///  The keys are inserted in the order `json!` would insert them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = { applied: false, failureReason: EDIT_REFUSAL };
/// ```
fn edit_refusal_reply() -> Value {
    let mut reply = serde_json::Map::new();
    reply.insert("applied".to_string(), Value::Bool(false));
    reply.insert("failureReason".to_string(), Value::from(EDIT_REFUSAL));
    return Value::Object(reply);
}

/// What:
///  Decide the reply for one server-to-client request.
///  `&str` borrows the method name;
///       `params` is moved in because parsing consumes it.
/// Why:
///  Every method helix-lsp can deliver,
///  and every method it cannot parse,
///  gets an explicit
///      reviewed answer;
///  none is left pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decide(method: string, params: Params, client: ClientView): Decision { /* switch */ }
/// ```
pub fn decide(method: &str, params: jsonrpc::Params, client: &ClientView<'_>) -> Decision {
    // What: `MethodCall::parse` returns `Result`: a typed request, `Error::Unhandled` for a method
    //       helix-lsp does not model, or a parameter decoding error.
    // Why: The three outcomes need three different replies.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let call; try { call = MethodCall.parse(method, params); } catch (error) { /* see arms */ }
    // ```
    let parsed = MethodCall::parse(method, params);
    return match parsed {
        Err(helix_lsp::Error::Unhandled) => Decision {
            // `Err(...)` is the failure variant: the server learns the client has no such method.
            reply: Err(jsonrpc::Error {
                code: jsonrpc::ErrorCode::MethodNotFound,
                message: format!("Method not found: {method}"),
                data: None,
            }),
            effect: Effect::None,
            policy: "method-not-found",
        },
        Err(error) => Decision {
            reply: Err(jsonrpc::Error::invalid_params(format!(
                "Malformed {method}: {error}"
            ))),
            effect: Effect::None,
            policy: "malformed",
        },
        // Progress tokens are bookkeeping; refusing them stalls rust-analyzer's start.
        Ok(MethodCall::WorkDoneProgressCreate(_)) => {
            result(Value::Null, Effect::None, "acknowledged")
        }
        // The one mutation a server can start by itself: always refused, with a normal result,
        // because servers built on vscode-languageserver-node can abort on an error reply.
        Ok(MethodCall::ApplyWorkspaceEdit(_)) => {
            result(edit_refusal_reply(), Effect::None, "refused")
        }
        Ok(MethodCall::WorkspaceFolders) => result(
            // A folder holds a URI and a name, which always serialise to JSON.
            serde_json::to_value(client.folders)
                .expect("workspace folders hold only a URI and a name, which serialise to JSON"),
            Effect::None,
            "answered",
        ),
        Ok(MethodCall::WorkspaceConfiguration(request)) => {
            // `Vec::new()` creates an empty growable list; the reply holds one entry per item, in order.
            let mut values: Vec<Value> = Vec::new();
            for item in &request.items {
                // `as_deref()` turns `Option<String>` into `Option<&str>` without copying.
                values.push(configuration_section(
                    client.settings,
                    item.section.as_deref(),
                ));
            }
            result(Value::Array(values), Effect::None, "answered")
        }
        Ok(MethodCall::RegisterCapability(request)) => {
            let mut watchers = Vec::new();
            for registration in request.registrations {
                // Helix declares no other dynamic registration; those are acknowledged and ignored.
                if registration.method != "workspace/didChangeWatchedFiles" {
                    continue;
                }
                let Some(options) = registration.register_options else {
                    continue;
                };
                // What: `serde_json::from_value` decodes JSON into a typed record and returns `Result`;
                //       `if let Ok(x) = ...` runs the block only on success.
                // Why: Malformed watcher options are skipped, not fatal.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const parsed = WatcherOptions.safeParse(options); if (parsed.success) watchers.push(...);
                // ```
                if let Ok(decoded) = serde_json::from_value(options) {
                    watchers.push((registration.id, decoded));
                }
            }
            result(
                Value::Null,
                Effect::RegisterWatchers(watchers),
                "acknowledged",
            )
        }
        Ok(MethodCall::UnregisterCapability(request)) => {
            let mut ids = Vec::new();
            for unregistration in request.unregisterations {
                if unregistration.method == "workspace/didChangeWatchedFiles" {
                    ids.push(unregistration.id);
                }
            }
            result(Value::Null, Effect::UnregisterWatchers(ids), "acknowledged")
        }
        // A server asking the client to open a document or address is not a reader-initiated action.
        Ok(MethodCall::ShowDocument(_)) => {
            result(json!({ "success": false }), Effect::None, "refused")
        }
        Ok(MethodCall::WorkspaceDiagnosticRefresh) => {
            result(Value::Null, Effect::RefreshDiagnostics, "acknowledged")
        }
        // `null` is the protocol's "no action selected"; the minimal interface shows no server prompts.
        Ok(MethodCall::ShowMessageRequest(_)) => result(Value::Null, Effect::None, "answered"),
    };
}

/// Every method's exact reply is asserted as JSON,
///  without a server.
#[cfg(test)]
#[path = "incoming_tests.rs"]
mod tests;
