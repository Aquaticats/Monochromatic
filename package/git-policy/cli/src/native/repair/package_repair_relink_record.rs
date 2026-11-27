//! What:
//!  The record one worktree's relink is queued as:
//! the reference it carries,
//! the tools a record may ask for,
//! the three answers a search path can be recorded with,
//! the entry itself,
//! and the one text the queue stores it in.
//! Why:
//!   The queue's record holds one text field,
//! so the entry travels as one JSON object,
//! and a path or a search path that no record could name back is refused rather than written lossily.
//! The producer and its controls read this text,
//! so it lives where both can name it and where nothing else of the hand-over is needed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const RELINK_REFERENCE = 'refs/monochromatic/relink';
//! type RelinkTool = 'pnpm' | 'toolchain';
//! type SearchPathOfTheAdd = { kind: 'not-recorded' } | { kind: 'unset' } | { kind: 'set'; path: string };
//! type RelinkEntry = { root: string; searchPath: SearchPathOfTheAdd; tools: RelinkTool[] };
//! const encode = (entry: RelinkEntry) => string;
//! const decode = (payload: string) => RelinkEntry;
//! ```

/// Import the owned,
/// platform-native string a recorded search path is.
use std::ffi::OsString;
/// Import the owned filesystem path a record names.
use std::path::PathBuf;
/// The reference a relink record carries.
/// It names no branch:
/// the record is about working state,
/// and a Git reference is only what the queue's record format asks every kind for.
pub const RELINK_REFERENCE: &str = "refs/monochromatic/relink";

/// What:
///  One tool whose copied state the service relinks in the worktree.
/// Why:
///   The record says which tools the worktree needs,
/// so a worktree with only a pnpm layout pays for no toolchain run,
/// and the name is what a record carries,
/// so a record another release wrote cannot ask for a tool this one does not know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RelinkTool = 'pnpm' | 'toolchain';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelinkTool {
    /// pnpm relinks the copied `node_modules` for the new depth.
    Pnpm,
    /// mpm-shadow rebuilds the toolchain links and launchers for the new checkout.
    Toolchain,
}

impl RelinkTool {
    /// The name a record carries for this tool.
    pub fn name(self) -> &'static str {
        return match self {
            RelinkTool::Pnpm => "pnpm",
            RelinkTool::Toolchain => "toolchain",
        };
    }

    /// The tool a name spells,
    /// or nothing for a name this release does not know.
    pub fn from_name(name: &str) -> Option<RelinkTool> {
        return match name {
            "pnpm" => Some(RelinkTool::Pnpm),
            "toolchain" => Some(RelinkTool::Toolchain),
            _ => None,
        };
    }
}

/// What:
///  The search path a record carries.
/// Why:
///   The record must say which programs the add resolved,
/// and "the add held no search path" is not "this record is too old to carry one".
/// With no path,
/// the add selects the worktree's own `.cache/toolchain-bin/mpm-shadow`,
/// while a record that names none makes the service search its own path and start another program;
/// so the three answers travel as three,
/// not as the two an optional field could carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchPathOfTheAdd = { kind: 'not-recorded' } | { kind: 'unset' } | { kind: 'set'; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchPathOfTheAdd {
    /// A record written before this release carried the add's environment:
    /// the service searches its own path.
    NotRecorded,
    /// The add's environment held no search path,
    /// so the service's own must not stand in for one.
    Unset,
    /// The directories the add resolved each tool's program on.
    Set(OsString),
}

/// What:
///  The work one relink record asks for:
/// the new worktree,
/// the search path the add resolved its programs on,
/// and the tools whose state needs relinking there.
/// Why:
///   The first two facts are what the add already read from the worktree and the person's environment,
/// so the service repeats no decision the add made,
/// and the record says exactly what the add would have run.
/// The search path travels because the service is a resident process of another session:
/// its own path may hold no pnpm or another one,
/// and the owner decided the repair starts whatever pnpm is first on the person's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RelinkEntry = { root: string; searchPath: SearchPathOfTheAdd; tools: RelinkTool[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelinkEntry {
    /// The new worktree.
    pub root: PathBuf,
    /// The search path the add resolved each tool's program on,
    /// and that the tools' own children inherit.
    pub search_path: SearchPathOfTheAdd,
    /// The tools the service runs there,
    /// in the order the report has always named them.
    pub tools: Vec<RelinkTool>,
}

/// What:
///  The name of one tool as the payload carries it.
/// Why:
///   The record's list is one JSON string per tool,
/// and a named function stands where a closure that only forwarded its argument stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nameOfTool = (tool) => tool.name();
/// ```
fn name_of_tool(tool: &RelinkTool) -> serde_json::Value {
    return serde_json::Value::from(tool.name());
}

/// What:
///  The entry as the one text a record can carry.
/// Why:
///   The queue's record holds one text field,
/// so the entry travels as one JSON object,
/// and a path or a search path is refused rather than written lossily.
///
/// # Errors
///
/// A path or a search path that is not UTF-8,
/// which no record could name back,
/// and an entry that asks for no tool,
/// which would be a record of nothing.
pub fn encode(entry: &RelinkEntry) -> Result<String, String> {
    if entry.tools.is_empty() {
        return Err(String::from("a relink of no tool is no work"));
    }
    let Some(root) = entry.root.to_str() else {
        return Err(String::from(
            "the path of the relink is not valid UTF-8, so no record can name it",
        ));
    };
    let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    fields.insert(String::from("root"), serde_json::Value::from(root));
    // The three answers are three spellings: no field for a record written before the add's environment travelled,
    // a null for an add that held no search path, and the text for the one it held.
    match entry.search_path {
        SearchPathOfTheAdd::NotRecorded => {}
        SearchPathOfTheAdd::Unset => {
            fields.insert(String::from("searchPath"), serde_json::Value::Null);
        }
        SearchPathOfTheAdd::Set(ref search_path) => {
            let Some(text) = search_path.to_str() else {
                return Err(String::from(
                    "the search path of the relink is not valid UTF-8, so no record can carry it",
                ));
            };
            fields.insert(String::from("searchPath"), serde_json::Value::from(text));
        }
    }
    let names: Vec<serde_json::Value> = entry.tools.iter().map(name_of_tool).collect();
    fields.insert(String::from("tools"), serde_json::Value::Array(names));
    return Ok(serde_json::Value::Object(fields).to_string());
}

/// What:
///  The entry a relink record carries.
/// Why:
///   A payload that is not the object `encode` writes is refused,
/// and so is a tool name this release does not know or a list of no tool:
/// the service must run exactly the tools the add decided on.
/// A record written before this release carried a search path names none,
/// and the service then searches its own path,
/// so such a record still relinks the worktree it was written for.
/// A record whose search path is null names an add that held none,
/// so the service searches nowhere rather than searching its own.
///
/// # Errors
///
/// Text that is not the object,
/// a root that is not named,
/// a tool that cannot be read,
/// and a search path that is neither text nor null.
pub fn decode(payload: &str) -> Result<RelinkEntry, String> {
    let parsed: serde_json::Value = match serde_json::from_str(payload) {
        Ok(value) => value,
        Err(error) => return Err(format!("the relink payload is not JSON ({error})")),
    };
    let Some(root) = parsed.get("root").and_then(serde_json::Value::as_str) else {
        return Err(String::from("the relink names no worktree"));
    };
    let Some(names) = parsed.get("tools").and_then(serde_json::Value::as_array) else {
        return Err(String::from("the relink names no tools"));
    };
    let mut tools: Vec<RelinkTool> = Vec::with_capacity(names.len());
    for name in names {
        let Some(named) = name.as_str() else {
            return Err(String::from("a tool of the relink is not text"));
        };
        let Some(tool) = RelinkTool::from_name(named) else {
            return Err(format!("this release knows no tool named {named}"));
        };
        tools.push(tool);
    }
    if tools.is_empty() {
        return Err(String::from("the relink names no tools"));
    }
    let search_path: SearchPathOfTheAdd = match parsed.get("searchPath") {
        None => SearchPathOfTheAdd::NotRecorded,
        Some(serde_json::Value::Null) => SearchPathOfTheAdd::Unset,
        Some(serde_json::Value::String(text)) => SearchPathOfTheAdd::Set(OsString::from(text)),
        Some(_) => {
            return Err(String::from(
                "the search path of the relink is neither text nor null",
            ));
        }
    };
    return Ok(RelinkEntry {
        root: PathBuf::from(root),
        search_path,
        tools,
    });
}
