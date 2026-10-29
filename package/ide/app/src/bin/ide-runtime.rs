//! Build-time language assets;
//!  the read-only application never invokes this executable.

/// Asset preparation failures must stop the build instead of shipping partial language support.
use anyhow::{Context, Result, bail};
/// Serialize configuration through TOML rather than interpolating a configuration language.
use serde::Serialize;
/// Filesystem paths are owned when retained and borrowed during recursive directory copies.
use std::{
    fs,
    path::{Path, PathBuf},
};

/// What:
///  `const GRAMMARS: &[&str]` is a fixed,
///  read-only list of borrowed text values.
///       Each entry is a pinned Helix grammar id (the parser name),
///  not a language name:
///       the `qml` language uses `qmljs`,
///  and `markdown.inline` uses `markdown_inline`.
///       Siblings a reader might expect:
///  `Vec<String>` (growable,
///  owned) and `[&str; 27]`
///       (a fixed-length array whose length is part of the type).
/// Why:
///   The list never changes at run time,
///  so no owned `Vec<String>` is needed,
///  and a slice
///       avoids restating the length whenever the measured inventory changes.
///       Selection evidence lives in `doc/planning/slint-ide-runtime-languages.md`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GRAMMARS = ['bash', 'batch', /* ... */ 'tsx'] as const;
/// ```
const GRAMMARS: &[&str] = &[
    // Languages measured as repository source or configuration.
    "bash",
    "batch",
    "c",
    "cpp",
    "css",
    "dockerfile",
    "hcl",
    "html",
    "javascript",
    "json",
    "kotlin",
    "markdown",
    "qmljs",
    "rust",
    "slint",
    "sql",
    "toml",
    "typescript",
    "xml",
    "yaml",
    // Companion grammars injected by those languages for content the repository contains.
    "awk",
    "comment",
    "jsdoc",
    "markdown_inline",
    "regex",
    "rust-format-args",
    // Retained from the initial slice; no `.tsx` file is currently measured.
    "tsx",
];

/// Selection shape understood by the pinned Helix grammar manager.
#[derive(Serialize)]
struct Selection {
    /// Compile only explicitly selected grammars,
    ///  not the entire built-in catalog.
    only: Vec<String>,
}

/// Build-only configuration lives under target,
///  never in the user's real Helix configuration.
#[derive(Serialize)]
struct Configuration {
    /// Serde maps the Rust field name to the existing Helix key.
    #[serde(rename = "use-grammars")]
    selection: Selection,
}

/// Copy the pinned runtime's query tree and preserve its exact file contents.
fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry_result in fs::read_dir(source)? {
        let entry = entry_result?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&from, &to)?;
        } else if kind.is_file() {
            fs::copy(&from, &to)?;
        } else {
            bail!("Unexpected non-regular runtime asset {}", from.display());
        }
    }
    return Ok(());
}

/// Validate the build boundary before the Helix helper writes configuration or grammars.
fn prepare(source: &Path, operation: &str) -> Result<()> {
    let target = std::env::current_dir()?.join("target");
    let config = helix_loader::config_dir();
    let runtime = helix_loader::runtime_dirs()
        .first()
        .context("No Helix runtime directory")?;
    if !config.starts_with(&target) || !runtime.starts_with(&target) {
        bail!(
            "Language preparation writes only under {}. Run the scoped runtime task with its private configuration environment.",
            target.display()
        );
    }
    fs::create_dir_all(&config)?;
    // What: iter/map/collect copy static names into an owned Vec<String>.
    // Why: The serialized selection contains only this task's explicit language slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const config = { 'use-grammars': { only: [...grammars] } };
    // ```
    let mut only: Vec<String> = Vec::new();
    for name in GRAMMARS.iter() {
        only.push((*name).to_string());
    }
    let configuration = Configuration {
        selection: Selection { only },
    };
    let encoded = toml::to_string(&configuration)?;
    fs::write(config.join("languages.toml"), encoded)?;
    copy_tree(&source.join("queries"), &runtime.join("queries"))?;
    let repository = source
        .parent()
        .context("Helix runtime has no repository parent")?;
    fs::copy(repository.join("LICENSE"), runtime.join("Helix-LICENSE"))?;
    // Publish the same selection used by the manager so packaging cannot duplicate its inventory.
    fs::write(
        runtime.join("selected-grammars.json"),
        serde_json::to_vec(GRAMMARS)?,
    )?;
    if operation == "fetch" {
        helix_loader::grammar::fetch_grammars(true)?;
        return Ok(());
    }
    helix_loader::grammar::build_grammars(None, true)?;
    return Ok(());
}

/// Expose build-only fetch/build operations;
///  help never touches runtime state.
fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let wants_help = match arguments.first() {
        Some(value) => value == "--help",
        None => false,
    };
    if wants_help {
        println!(
            "Usage: ide-runtime <fetch|build> <PINNED_HELIX_RUNTIME>\nBuild-only helper. Use mise run //package/ide/app:runtime."
        );
        return Ok(());
    }
    if arguments.len() != 2 || (arguments[0] != "fetch" && arguments[0] != "build") {
        bail!("Expected fetch or build and the pinned Helix runtime path; use --help");
    }
    let source = PathBuf::from(&arguments[1]);
    prepare(&source, &arguments[0])?;
    return Ok(());
}
