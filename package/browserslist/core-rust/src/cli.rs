//! What:
//!  The command-line surface of the `monochromatic-browserslist` binary: `update`, `export` and `resolve`.
//! Why:
//!  `update` is the only way versions are ever picked (a `mise` task runs it), `export` hands the verified
//!  tarballs to the conformance harness, and `resolve` lets a person or a script run the exact library call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function run(args: string[]): string; // parses `update | export | resolve` plus --options
//! ```

use crate::api::render_browserslistrc;
use crate::error::{BrowserslistError, Outcome};
use crate::lock::DataLock;
use crate::source::{DataSource, pinned_tarball, write_atomically};
use crate::update::{MOZILLA_PRODUCT_DETAILS, build_lock};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The lock file name, relative to the directory the command runs in (the package directory under `mise`).
pub const DEFAULT_LOCK_PATH: &str = "browserslist-data.lock.json";

/// Build a usage error.
fn usage(message: &str) -> BrowserslistError {
    return BrowserslistError::Data {
        file: "command line".to_string(),
        offset: None,
        message: format!(
            "{message}; usage: monochromatic-browserslist update|export|resolve with options \
             --lock PATH --cache-dir DIR --registry URL --mozilla URL --out DIR --config PATH"
        ),
    };
}

/// Split `--name value` pairs into a map, refusing options the command does not take.
fn parse_options(arguments: &[String], allowed: &[&str]) -> Outcome<HashMap<String, String>> {
    let mut options: HashMap<String, String> = HashMap::new();
    let mut index = 0;
    while index < arguments.len() {
        let name = arguments[index].trim_start_matches("--");
        let is_option = arguments[index].starts_with("--") && allowed.contains(&name);
        if !is_option {
            return Err(usage(&format!("unknown option {:?}", arguments[index])));
        }
        let Some(value) = arguments.get(index + 1) else {
            return Err(usage(&format!("option {:?} needs a value", arguments[index])));
        };
        options.insert(name.to_string(), value.clone());
        index += 2;
    }
    return Ok(options);
}

/// Read a file as UTF-8 text or report the path.
fn read_text(path: &Path) -> Outcome<String> {
    return fs::read_to_string(path)
        .map_err(|error| return BrowserslistError::Io { path: path.display().to_string(), message: error.to_string() });
}

/// Build the data source from `--cache-dir` and `--registry`, defaulting to the user cache and npm.
fn source_from(options: &HashMap<String, String>) -> Outcome<DataSource> {
    let mut source = DataSource::user_default()?;
    if let Some(directory) = options.get("cache-dir") {
        source.cache_dir = PathBuf::from(directory);
    }
    if let Some(registry) = options.get("registry") {
        source.registry_url = registry.trim_end_matches('/').to_string();
    }
    return Ok(source);
}

/// The lock path from `--lock`, or the default.
fn lock_path(options: &HashMap<String, String>) -> PathBuf {
    return PathBuf::from(options.get("lock").map_or(DEFAULT_LOCK_PATH, String::as_str));
}

/// `update`: build a lock from the newest data and write it.
fn run_update(options: &HashMap<String, String>) -> Outcome<String> {
    let path = lock_path(options);
    let source = source_from(options)?;
    let mozilla = options.get("mozilla").map_or(MOZILLA_PRODUCT_DETAILS, String::as_str);
    let lock = build_lock(&source, mozilla)?;
    write_atomically(&path, lock.to_json().as_bytes())?;
    return Ok(format!(
        "wrote {}: caniuse-lite {}, node-releases {}, Firefox ESR {}\n",
        path.display(),
        lock.caniuse_lite.version,
        lock.node_releases.version,
        lock.firefox_esr
    ));
}

/// `export`: write the verified pinned tarballs into a directory for the conformance harness.
fn run_export(options: &HashMap<String, String>) -> Outcome<String> {
    let Some(out) = options.get("out") else {
        return Err(usage("export needs --out DIR"));
    };
    let lock = DataLock::from_json(&read_text(&lock_path(options))?)?;
    let source = source_from(options)?;
    for package in [&lock.caniuse_lite, &lock.node_releases] {
        let bytes = pinned_tarball(package, &source)?;
        write_atomically(&Path::new(out).join(format!("{}-{}.tgz", package.name, package.version)), &bytes)?;
    }
    return Ok(format!("exported caniuse-lite {} and node-releases {} to {out}\n", lock.caniuse_lite.version, lock.node_releases.version));
}

/// `resolve`: print the rendered resolved JSON for a config file.
fn run_resolve(options: &HashMap<String, String>) -> Outcome<String> {
    let Some(config) = options.get("config") else {
        return Err(usage("resolve needs --config PATH"));
    };
    let lock = DataLock::from_json(&read_text(&lock_path(options))?)?;
    return render_browserslistrc(&read_text(Path::new(config))?, &lock, &source_from(options)?);
}

/// What:
///  Run one command and return the text to print on success.
/// Why:
///  `main` only prints this or the error, so the whole command surface is testable as a function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(args: string[]): string;
/// ```
pub fn run(arguments: &[String]) -> Outcome<String> {
    let Some(command) = arguments.first() else {
        return Err(usage("a command is required"));
    };
    let rest = &arguments[1..];
    match command.as_str() {
        "update" => return run_update(&parse_options(rest, &["lock", "cache-dir", "registry", "mozilla"])?),
        "export" => return run_export(&parse_options(rest, &["lock", "cache-dir", "registry", "out"])?),
        "resolve" => return run_resolve(&parse_options(rest, &["lock", "cache-dir", "registry", "config"])?),
        other => return Err(usage(&format!("unknown command {other:?}"))),
    }
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
