//! The scripted language server definition a test hands its worker,
//!  with and without extra project roots.

use super::child_process::{
    SERVER,
    report_path,
};
use std::path::Path;

/// Language definitions for the scripted server with the given environment and request timeout.
pub fn scripted(
    root: &Path,
    variables: &[(&str, &str)],
    timeout: u64,
) -> String {
    return scripted_with_roots(root, variables, timeout, "[]");
}

/// Like `scripted`,
///  with root-marker file names in TOML array syntax.
pub fn scripted_with_roots(
    root: &Path,
    variables: &[(&str, &str)],
    timeout: u64,
    roots: &str,
) -> String {
    let mut environment = format!("IDE_SCRIPTED_REPORT = '{}'", report_path(root).display());
    for (name, value) in variables {
        environment.push_str(&format!(", IDE_SCRIPTED_{name} = '{value}'"));
    }
    return format!(
        r#"
[language-server.{SERVER}]
command = '{program}'
timeout = {timeout}
environment = {{ {environment} }}

[language-server.{SERVER}.config.scripted]
flag = true
nested = {{ value = 42 }}

[[language]]
name = "scripted"
scope = "source.scripted"
file-types = ["scripted"]
roots = {roots}
language-servers = ["{SERVER}"]
"#,
        program = env!("CARGO_BIN_EXE_ide-scripted-lsp"),
    );
}
