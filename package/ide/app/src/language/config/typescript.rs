//! The TypeScript family uses the project's own TypeScript 7 server instead of Helix's default.
//!
//! Helix's default,
//!  `typescript-language-server`,
//!  needs a `tsserver.js` that TypeScript 7 no
//! longer ships.
//!  TypeScript 7 has its own server,
//!  `tsc --lsp --stdio`;
//!  a project without it
//! shows the missing-executable state.
//!  Adopted in `doc/planning/slint-ide-language-intelligence.md`.

/// Helix's typed configuration and one server definition.
use helix_core::syntax::config::{Configuration, LanguageServerConfiguration};
/// What:
///  `Value` is any JSON value;
///  `json!` builds one from literal syntax.
/// Why:
///  The server's settings table is JSON.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::{Value, json};
/// What:
///  `HashMap` is a key-value table;
///  `Path`/`PathBuf` are borrowed and owned paths.
/// Why:
///  The launcher is located below the project root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { join } from 'node:path';
/// ```
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Name of Helix's default server for the TypeScript family,
///  which is replaced.
const DEFAULT_SERVER: &str = "typescript-language-server";

/// Name of the application's definition of the TypeScript 7 server.
pub(super) const SERVER: &str = "typescript-native";

/// The launcher a project's TypeScript package installs,
///  relative to the project root.
const LAUNCHER: &str = "node_modules/typescript/bin/tsc";

/// The package manifest that states the project's TypeScript version.
const MANIFEST: &str = "node_modules/typescript/package.json";

/// What:
///  The settings the server needs before it returns inlay hints.
/// Why:
///  Measured in the integration spike:
///  the server returned no hints until its settings held
///      these Visual Studio Code style keys under `typescript.inlayHints`;
///  the keys of Helix's
///      `typescript-language-server` table produced none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settings = { typescript: { inlayHints: { variableTypes: { enabled: true }, /* ... */ } } };
/// ```
fn settings() -> Value {
    let mut settings = json!({
        "typescript": {
            "inlayHints": {
                "parameterNames": { "enabled": null, "suppressWhenArgumentMatchesName": false },
                "parameterTypes": { "enabled": true },
                "variableTypes": { "enabled": true },
                "propertyDeclarationTypes": { "enabled": true },
                "functionLikeReturnTypes": { "enabled": true },
                "enumMemberValues": { "enabled": true },
            },
        },
    });
    // `Value::from` wraps the string; `json!` would unwrap a `Result` to do it, which the clippy configuration bans.
    // The `null` placeholder above is replaced here without moving the key.
    settings["typescript"]["inlayHints"]["parameterNames"]["enabled"] = Value::from("all");
    return settings;
}

/// What:
///  Add the TypeScript 7 server definition and point every language that used Helix's
///       default TypeScript server at it.
///  `&mut Configuration` lends the configuration for
///       modification;
///  `Option<&Path>` is the launcher `locate` found,
///  or nothing.
/// Why:
///  The command is an absolute path,
///  so it does not depend on the
///      process working directory the way a relative command would.
///  Without a located launcher it names the project root's own,
///  which `locate` then reports as missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function apply(configuration: Configuration, root: string, launcher?: string): void {
///   configuration.languageServer[SERVER] = { command: launcher ?? join(root, LAUNCHER), args: ['--lsp', '--stdio'], config: settings() };
///   for (const language of configuration.language)
///     for (const server of language.languageServers) if (server.name === DEFAULT_SERVER) server.name = SERVER;
/// }
/// ```
pub(super) fn apply(configuration: &mut Configuration, root: &Path, launcher: Option<&Path>) {
    // Variables an application-supplied definition of this server adds are kept, for example
    // probe markers in acceptance tests; command, arguments, and settings are always replaced.
    let environment = match configuration.language_server.get(SERVER) {
        Some(existing) => existing.environment.clone(),
        None => HashMap::new(),
    };
    configuration.language_server.insert(
        // `to_string` copies the constant into an owned `String` the table keeps.
        SERVER.to_string(),
        LanguageServerConfiguration {
            // `to_string_lossy` renders the path as text; availability is checked on the real path.
            command: match launcher {
                Some(found) => found.to_path_buf(),
                None => root.join(LAUNCHER),
            }
            .to_string_lossy()
            .into_owned(),
            args: vec!["--lsp".to_string(), "--stdio".to_string()],
            environment,
            // `Some(...)` is the "value present" variant of `Option`.
            config: Some(settings()),
            // Helix's default request timeout, in seconds.
            timeout: 20,
            // `None` is the "nothing" variant: no required root files.
            required_root_patterns: None,
        },
    );
    // `iter_mut` hands out modifiable borrows so names can be replaced in place.
    for language in configuration.language.iter_mut() {
        for features in language.language_servers.iter_mut() {
            if features.name == DEFAULT_SERVER {
                features.name = SERVER.to_string();
            }
        }
    }
}

/// What:
///  The folders a document's TypeScript launcher is looked for in,
///  nearest first:
///  the document's own folder,
///  then each parent,
///  ending with the project root.
///  `Vec<PathBuf>` is a growable list of owned paths.
/// Why:
///  Node and TypeScript find a package by walking up from the importing file,
///  so a pnpm monorepo keeps one `node_modules/typescript` at its root
///      and the packages below it use that one.
///  A document outside the project,
///  or none,
///  is looked up at the project root only:
///  the search never leaves the project.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function searched(root: string, document?: string): string[]
/// ```
fn searched(root: &Path, document: Option<&Path>) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    // `and_then` continues only when a document was given; `starts_with` keeps the walk inside the root.
    let mut start = None;
    if let Some(folder) = document.and_then(Path::parent)
        && folder.starts_with(root)
    {
        start = Some(folder);
    }
    if let Some(first) = start {
        // `ancestors` yields the folder itself and then each parent in turn.
        for folder in first.ancestors() {
            folders.push(folder.to_path_buf());
            if folder == root {
                return folders;
            }
        }
    }
    // A document that is not below the root, or no document, leaves only the root to search.
    folders.clear();
    folders.push(root.to_path_buf());
    return folders;
}

/// What:
///  The nearest `node_modules/typescript/bin/tsc` above the project root,
///  or nothing.
///  `Option<PathBuf>` is "a path,
///  or nothing".
/// Why:
///  A TypeScript install that exists only outside the project cannot be used,
///  but the reason shown to the reader can name it,
///  so they know to open the folder that holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installAbove(root: string): string | undefined
/// ```
fn install_above(root: &Path) -> Option<PathBuf> {
    // `skip(1)` leaves out the root itself, which was already searched.
    for folder in root.ancestors().skip(1) {
        let launcher = folder.join(LAUNCHER);
        if launcher.is_file() {
            return Some(launcher);
        }
    }
    return None;
}

/// What:
///  Find the TypeScript 7 launcher for a document,
///  or explain why there is none.
///       `Result<PathBuf, String>` is the absolute launcher path or a user-facing reason.
/// Why:
///  The nearest install wins,
///  as in Node's and TypeScript's own resolution,
///  so a package is analyzed with the TypeScript it imports;
///  an older install is reported instead of skipped for a farther one.
///  A project without TypeScript,
///  or with a version before 7,
///  has no built-in language
///      server;
///  both are the missing-executable state,
///  with a reason that names the remedy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locate(root: string, document?: string): string // throws with the reason
/// ```
pub(super) fn locate(root: &Path, document: Option<&Path>) -> Result<PathBuf, String> {
    let folders = searched(root, document);
    for folder in &folders {
        let launcher = folder.join(LAUNCHER);
        if !launcher.is_file() {
            continue;
        }
        // The nearest install decides; it is checked here and never skipped.
        return check_version(folder, launcher);
    }
    // The nearest and the farthest searched folder name the whole range in the reason.
    let nearest = match folders.first() {
        Some(folder) => folder.clone(),
        None => root.to_path_buf(),
    };
    let range = if nearest == root {
        format!(
            "{} does not exist in the project folder",
            root.join(LAUNCHER).display()
        )
    } else {
        format!(
            "no {LAUNCHER} exists in {} or in any folder above it up to the project folder {}",
            nearest.display(),
            root.display()
        )
    };
    let above = match install_above(root) {
        Some(launcher) => format!(
            ". A TypeScript install exists outside the project folder, at {}; open the folder that holds it as the project to use it",
            launcher.display()
        ),
        None => String::new(),
    };
    // `Err(...)` is the failure variant of `Result`.
    return Err(format!(
        "the project has no TypeScript language server: {range}{above}. Install the project's dependencies; TypeScript 7 or later is required"
    ));
}

/// What:
///  Accept the install in `folder` when its package manifest states version 7 or later.
///       `Result<PathBuf, String>` is the launcher or the reason it cannot be used.
/// Why:
///  Only the major version decides whether the package contains the server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkVersion(folder: string, launcher: string): string // throws with the reason
/// ```
fn check_version(folder: &Path, launcher: PathBuf) -> Result<PathBuf, String> {
    let manifest = folder.join(MANIFEST);
    // What: `read_to_string` returns `Result`; `map_err` converts the I/O error into the text
    //       reason, and the trailing `?` returns it on failure.
    // Why: Without a readable manifest the version cannot be shown to be 7 or later.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const encoded = readFileSync(manifest, 'utf8');
    // ```
    let encoded = match std::fs::read_to_string(&manifest) {
        Ok(text) => text,
        Err(error) => {
            return Err(format!(
                "cannot read the project's TypeScript version from {}: {error}",
                manifest.display()
            ));
        }
    };
    let decoded: Value = match serde_json::from_str(&encoded) {
        Ok(value) => value,
        Err(error) => {
            return Err(format!(
                "cannot decode the project's TypeScript version from {}: {error}",
                manifest.display()
            ));
        }
    };
    // `as_str` is `Some` for a JSON string; `unwrap_or("")` substitutes empty text otherwise.
    let version = decoded["version"].as_str().unwrap_or("");
    // What: `split('.').next()` takes the text before the first dot; `parse::<u64>()` converts it
    //       to a number and returns `Result`. `unwrap_or(0)` treats an unreadable version as too old.
    // Why: Only the major version decides whether the package contains the server.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const major = Number(version.split('.')[0]) || 0;
    // ```
    let major = version
        .split('.')
        .next()
        .unwrap_or("")
        .parse::<u64>()
        .unwrap_or(0);
    if major < 7 {
        return Err(format!(
            "the project's TypeScript {version} in {} has no built-in language server; version 7 or later is required",
            folder.display()
        ));
    }
    // `Ok(...)` is the success variant of `Result`.
    return Ok(launcher);
}
