//! What the server can see and change from inside its sandbox,
//!  recorded once at start.

/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// What:
///  `Value` is any JSON value;
///  `json!` builds one from literal syntax.
/// Why:
///  The audit is one report line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::{Value, json};
/// What:
///  `Write` adds `write_all` to files;
///  `OpenOptions` opens a file with chosen modes.
/// Why:
///  Write attempts use the same open-for-append the build-script probe uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { appendFileSync } from 'node:fs';
/// ```
use std::{fs::OpenOptions, io::Write};

/// True when the comma-separated mount `options` list `wanted`.
fn lists_option(options: &str, wanted: &str) -> bool {
    for option in options.split(',') {
        if option == wanted {
            return true;
        }
    }
    return false;
}

/// What:
///  Mount points mounted read-write,
///  from `/proc/self/mountinfo`.
/// Why:
///  Inside the sandbox only the private state,
///  `/tmp`,
///  and the kernel file systems may be writable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writableMounts(): string[]
/// ```
fn writable_mounts() -> Vec<String> {
    let mut found = Vec::new();
    // `unwrap_or_default` substitutes empty text when the table cannot be read.
    let table = std::fs::read_to_string("/proc/self/mountinfo").unwrap_or_default();
    for line in table.lines() {
        // Fields: id, parent, device, root, mount point, mount options, ...
        let fields: Vec<&str> = line.split(' ').collect();
        if fields.len() > 5 && lists_option(fields[5], "rw") {
            found.push(fields[4].to_string());
        }
    }
    return found;
}

/// Try to append to `path` and report the result with the raw error number.
fn attempt_write(path: &str) -> Value {
    let opened = OpenOptions::new().create(true).append(true).open(path);
    let outcome = match opened {
        Ok(mut file) => file.write_all(b"x"),
        Err(error) => Err(error),
    };
    return match outcome {
        Ok(()) => json_value!({ "path": path, "ok": true }),
        Err(error) => json_value!({ "path": path, "ok": false, "errno": error.raw_os_error() }),
    };
}

/// What:
///  Build the audit record.
///  `writes` is a colon-separated list of files to try writing.
/// Why:
///  The confinement acceptance tests read this record through the application's real start path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function audit(writes: string): object
/// ```
pub fn audit(writes: &str) -> Value {
    let mut names: Vec<String> = Vec::new();
    // `vars_os` lists every variable; only the names are recorded, never the values.
    for (name, _) in std::env::vars_os() {
        names.push(name.to_string_lossy().into_owned());
    }
    names.sort();
    let mut namespaces = json!({});
    for kind in ["user", "pid", "net", "mnt"] {
        let link = match std::fs::read_link(format!("/proc/self/ns/{kind}")) {
            Ok(target) => target.display().to_string(),
            Err(_) => String::new(),
        };
        namespaces[kind] = json_value!(link);
    }
    let mut results = Vec::new();
    for path in writes.split(':') {
        if !path.is_empty() {
            results.push(attempt_write(path));
        }
    }
    // The working directory shows which spelling of the project root exists inside the sandbox.
    let cwd = match std::env::current_dir() {
        Ok(found) => found.display().to_string(),
        Err(_) => String::new(),
    };
    return json_value!({
        "pid": std::process::id(),
        "cwd": cwd,
        "environment": names,
        "writableMounts": writable_mounts(),
        "namespaces": namespaces,
        "writes": results,
    });
}
