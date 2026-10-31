//! What:
//!  Decodes caniuse-lite's packed agent data into the per-browser version lists browserslist reads.
//! Why:
//!  `data/agents.js` stores each browser under a one-letter code and each version as a short code;
//!  `data/browsers.js` and `data/browserVersions.js` are the two lookup tables,
//!  exactly as caniuse-lite's `dist/unpacker/agents.js` decodes them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parseAgents(agentsJs: string, browsersJs: string, versionsJs: string): Map<string, Agent>;
//! ```

/// What:
///  Import `BrowserslistError, Outcome` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome } from './error';
/// ```
use crate::error::{BrowserslistError, Outcome};
/// What:
///  Import `JsValue, object_get, parse_module_exports` from `crate::js_literal`.
/// Why:
///  The packed data files are read with the owned literal parser.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsValue, objectGet, parseModuleExports } from './js_literal';
/// ```
use crate::js_literal::{JsValue, object_get, parse_module_exports};
/// What:
///  Import `HashMap` from `std::collections::HashMap`.
/// Why:
///  Hash tables give linear-time lookups.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { HashMap } from 'std/collections';
/// ```
use std::collections::HashMap;

/// Diagnostic name of the agents file.
const AGENTS_FILE: &str = "caniuse-lite data/agents.js";
/// Diagnostic name of the browser-name table file.
const BROWSERS_FILE: &str = "caniuse-lite data/browsers.js";
/// Diagnostic name of the version table file.
const VERSIONS_FILE: &str = "caniuse-lite data/browserVersions.js";

/// What:
///  How many trailing slots of a browser's version array are reserved for unreleased versions.
///  `usize` (not `i32`) because it is subtracted from a length.
/// Why:
///  browserslist's `released` list is `versions.slice(0, -3)` taken on the raw array before nulls are dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNRELEASED_SLOTS = 3;
/// ```
const UNRELEASED_SLOTS: usize = 3;

/// What:
///  One browser:
/// its key name and its raw version slots (`None` marks an empty slot).
/// Why:
///  Keeping the raw slots lets `released` cut positionally,
/// exactly as browserslist does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Agent = { name: string; slots: (string | null)[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    /// The browserslist key,
    /// for example `and_chr`.
    pub name: String,
    /// Version strings in file order with `None` for empty slots.
    pub slots: Vec<Option<String>>,
}

/// What:
///  Methods of `Agent`.
/// Why:
///  `versions` and `released` mirror browserslist's `data.versions` and `data.released`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Agent { versions(): string[]; released(): string[] }
/// ```
impl Agent {
    /// Every non-empty version,
    /// including the unreleased ones (browserslist's `data.versions`).
    pub fn versions(&self) -> Vec<&str> {
        return self.slots.iter().flatten().map(String::as_str).collect();
    }

    /// The released versions:
    /// the slots minus the last three,
    /// then minus empty ones (`data.released`).
    pub fn released(&self) -> Vec<&str> {
        let cut = self.slots.len().saturating_sub(UNRELEASED_SLOTS);
        return self.slots[..cut].iter().flatten().map(String::as_str).collect();
    }
}

/// What:
///  All browsers of one caniuse-lite revision,
/// in file order.
/// Why:
///  Queries look browsers up by their lowercase key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AgentTable = Agent[];
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTable {
    /// The browsers in file order.
    agents: Vec<Agent>,
}

/// What:
///  Methods of `AgentTable`.
/// Why:
///  Queries look browsers up here,
/// with browserslist's own error for unknown ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AgentTable { get(name: string): Agent | undefined; require(written: string): Agent }
/// ```
impl AgentTable {
    /// Build a table from agents (used by tests that need a synthetic revision).
    pub fn new(agents: Vec<Agent>) -> Self {
        return Self { agents };
    }

    /// Find a browser by its lowercase key.
    pub fn get(&self, name: &str) -> Option<&Agent> {
        for agent in &self.agents {
            if agent.name == name {
                return Some(agent);
            }
        }
        return None;
    }

    /// Find a browser or fail with browserslist's own `Unknown browser` message.
    pub fn require(&self, written: &str) -> Outcome<&Agent> {
        let Some(agent) = self.get(&written.to_ascii_lowercase()) else {
            return Err(BrowserslistError::Browserslist { message: format!("Unknown browser {written}") });
        };
        return Ok(agent);
    }
}

/// Read an object whose values are all strings into a code-to-text map.
fn string_table(file: &str, value: &JsValue) -> Outcome<HashMap<String, String>> {
    let JsValue::Object(entries) = value else {
        return Err(BrowserslistError::Data {
            file: file.to_string(),
            offset: None,
            message: "expected an object of strings".to_string(),
        });
    };
    let mut table: HashMap<String, String> = HashMap::new();
    for (code, entry) in entries {
        let JsValue::Text(text) = entry else {
            return Err(BrowserslistError::Data {
                file: file.to_string(),
                offset: None,
                message: format!("entry {code} is not a string"),
            });
        };
        table.insert(code.clone(), text.clone());
    }
    return Ok(table);
}

/// Decode one browser's `C` array through the version table.
fn decode_slots(code: &str, browser: &JsValue, versions: &HashMap<String, String>) -> Outcome<Vec<Option<String>>> {
    let Some(JsValue::Array(items)) = object_get(browser, "C") else {
        return Err(BrowserslistError::Data {
            file: AGENTS_FILE.to_string(),
            offset: None,
            message: format!("browser {code} has no C version array"),
        });
    };
    let mut slots: Vec<Option<String>> = Vec::with_capacity(items.len());
    for item in items {
        let JsValue::Text(version_code) = item else {
            return Err(BrowserslistError::Data {
                file: AGENTS_FILE.to_string(),
                offset: None,
                message: format!("browser {code} has a non-string version slot"),
            });
        };
        if version_code.is_empty() {
            slots.push(None);
            continue;
        }
        let Some(version) = versions.get(version_code) else {
            return Err(BrowserslistError::Data {
                file: AGENTS_FILE.to_string(),
                offset: None,
                message: format!("browser {code} uses version code {version_code} missing from browserVersions.js"),
            });
        };
        slots.push(Some(version.clone()));
    }
    return Ok(slots);
}

/// What:
///  Parse the three caniuse-lite data files into an `AgentTable`.
/// Why:
///  This is the only place the packed format is interpreted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAgents(agentsJs: string, browsersJs: string, versionsJs: string): AgentTable;
/// ```
pub fn parse_agents(agents_js: &str, browsers_js: &str, versions_js: &str) -> Outcome<AgentTable> {
    let browsers = string_table(BROWSERS_FILE, &parse_module_exports(BROWSERS_FILE, browsers_js)?)?;
    let versions = string_table(VERSIONS_FILE, &parse_module_exports(VERSIONS_FILE, versions_js)?)?;
    let agents_value = parse_module_exports(AGENTS_FILE, agents_js)?;
    let JsValue::Object(entries) = &agents_value else {
        return Err(BrowserslistError::Data {
            file: AGENTS_FILE.to_string(),
            offset: None,
            message: "top level is not an object".to_string(),
        });
    };
    let mut agents: Vec<Agent> = Vec::with_capacity(entries.len());
    for (code, browser) in entries {
        let Some(name) = browsers.get(code) else {
            return Err(BrowserslistError::Data {
                file: AGENTS_FILE.to_string(),
                offset: None,
                message: format!("browser code {code} is missing from browsers.js"),
            });
        };
        let slots = decode_slots(code, browser, &versions)?;
        agents.push(Agent { name: name.clone(), slots });
    }
    return Ok(AgentTable { agents });
}

/// What:
///  The unit tests of `caniuse.rs`,
/// kept in the sidecar file `caniuse_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './caniuse.test';
/// ```
#[cfg(test)]
#[path = "caniuse_tests.rs"]
mod tests;
