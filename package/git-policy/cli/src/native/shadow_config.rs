//! What:
//!  The shadow repository's `config` file, written by cli-git itself,
//!       and the repository format facts it copies from the real common config.
//! Why:
//!  Git does not apply `core.repositoryformatversion` and `extensions.*` from an included file,
//!  so they are copied explicitly.
//!  `core.hooksPath` precedes the include so a real `core.hooksPath` still overrides it;
//!  `core.worktree`, `gc.auto` and `maintenance.auto` follow it,
//!  so the real config cannot redirect the worktree or start maintenance in the shadow
//!  (`src/shadow-repository/shadow-config.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! formatShadowConfig({ format, commonDir, worktreeRoot });
//! ```

/// The one configuration read the commit shares.
use super::config_snapshot::ConfigSnapshot;
/// One required Git start, and its failure text.
use super::transaction_git::{GitContext, GitFailure, GitRequest, run_git_checked};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Repository format facts the shadow copies from the real common config.
/// Why:
///   A shadow with a different object format or ref backend would misread the real store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepositoryFormat = { version: string; extensions: [string, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryFormat {
    /// `core.repositoryformatversion`, `0` when unset.
    pub version: String,
    /// Every `extensions.*` key with its value, in file order, the prefix removed.
    pub extensions: Vec<(String, String)>,
}

/// What:
///  Quotes one value under Git's config syntax.
/// Why:
///   The value is always double-quoted,
///  so `;`, `#` and surrounding whitespace stay literal;
///  backslash, double quote, newline and tab use Git's escapes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// quoteGitConfigValue('/a "b"'); // '"/a \\"b\\""'
/// ```
pub fn quote_git_config_value(value: &str) -> String {
    let mut body: String = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => body.push_str(r"\\"),
            '"' => body.push_str(r#"\""#),
            '\n' => body.push_str(r"\n"),
            '\t' => body.push_str(r"\t"),
            other => body.push(other),
        }
    }
    return format!("\"{body}\"");
}

/// What:
///  Reads the format facts from the real common config file alone.
/// Why:
///   `git config --file config --list` reads only that file,
///  so an included or worktree config never leaks into the copied format.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readRepositoryFormat({ gitPath, commonDir });
/// ```
pub fn read_repository_format(
    context: &GitContext,
    common_dir: &Path,
) -> Result<RepositoryFormat, GitFailure> {
    let mut request: GitRequest = GitRequest::new(
        common_dir,
        &["config", "--file", "config", "--null", "--list"],
    );
    request.without_prefix = true;
    let output = run_git_checked(context, &request)?;
    let mut version: Option<String> = None;
    let mut extensions: Vec<(String, String)> = Vec::new();
    for entry in output.stdout.split(|byte| { return *byte == 0; }).filter(|entry| { return !entry.is_empty(); }) {
        // A key without a value line is a valueless boolean, which reads as `true`.
        let (key, value): (&[u8], &[u8]) = match entry.iter().position(|byte| { return *byte == b'\n'; }) {
            Some(newline) => (&entry[..newline], &entry[newline + 1..]),
            None => (entry, &b"true"[..]),
        };
        note_format_entry(
            String::from_utf8_lossy(key).as_ref(),
            String::from_utf8_lossy(value).into_owned(),
            &mut version,
            &mut extensions,
        );
    }
    return Ok(RepositoryFormat {
        version: version.unwrap_or_else(|| { return String::from("0"); }),
        extensions,
    });
}

/// What:
///  Record one config entry in the format facts when it is the version or an extension.
/// Why:
///   Both readers of the common config (Git's `--file` listing and the snapshot) keep the last version, and every
///  extension in file order with the `extensions.` prefix removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (key.toLowerCase() === 'core.repositoryformatversion') version = value;
/// if (key.slice(0, 11).toLowerCase() === 'extensions.') extensions.push([key.slice(11), value]);
/// ```
fn note_format_entry(
    key_text: &str,
    value_text: String,
    version: &mut Option<String>,
    extensions: &mut Vec<(String, String)>,
) {
    if key_text.to_lowercase() == "core.repositoryformatversion" {
        *version = Some(value_text.clone());
    }
    if key_text.is_char_boundary(11) && key_text[..11].to_lowercase() == "extensions." {
        extensions.push((key_text[11..].to_string(), value_text));
    }
}

/// What:
///  The format facts from the configuration snapshot, when its local entries are exactly the common config file.
/// Why:
///   `git config --file config --list` reads that one file without includes, and the snapshot's `local` scope is
///  that file plus whatever it includes; with no include key anywhere, and a read made without global options, the
///  two lists are the same entries in the same order, so the facts are the same.
///  Any include key makes the answer `None`, and the caller starts Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.hasNoIncludes() ? formatFrom(snapshot.entries.filter((entry) => entry.scope === 'local')) : undefined;
/// ```
pub fn repository_format_from_snapshot(snapshot: &ConfigSnapshot) -> Option<RepositoryFormat> {
    if !snapshot.without_global_options() {
        return None;
    }
    let includes: bool = snapshot.entries().iter().any(|entry| {
        return entry.key == "include.path" || (entry.key.starts_with("includeif.") && entry.key.ends_with(".path"));
    });
    if includes {
        return None;
    }
    let mut version: Option<String> = None;
    let mut extensions: Vec<(String, String)> = Vec::new();
    for entry in snapshot.entries().iter().filter(|candidate| { return candidate.scope == "local"; }) {
        // A key without a value is a valueless boolean, which reads as `true`.
        let value_text: String = match &entry.value {
            Some(bytes) => String::from_utf8_lossy(bytes).into_owned(),
            None => String::from("true"),
        };
        note_format_entry(entry.key.as_str(), value_text, &mut version, &mut extensions);
    }
    return Some(RepositoryFormat {
        version: version.unwrap_or_else(|| { return String::from("0"); }),
        extensions,
    });
}

/// What:
///  Formats the shadow `config` file.
/// Why:
///   The line order is the incumbent's:
///  hooks path before the include, then the worktree, gc and maintenance overrides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// formatShadowConfig({ format, commonDir, worktreeRoot });
/// ```
pub fn format_shadow_config(format: &RepositoryFormat, common_dir: &Path, worktree_root: &Path) -> String {
    let common: String = common_dir.display().to_string();
    let mut lines: Vec<String> = vec![
        String::from("[core]"),
        format!("\trepositoryformatversion = {}", quote_git_config_value(format.version.as_str())),
        format!("\thooksPath = {}", quote_git_config_value(format!("{common}/hooks").as_str())),
    ];
    if !format.extensions.is_empty() {
        lines.push(String::from("[extensions]"));
        for (name, value) in &format.extensions {
            lines.push(format!("\t{name} = {}", quote_git_config_value(value.as_str())));
        }
    }
    lines.push(String::from("[include]"));
    lines.push(format!(
        "\tpath = {}",
        quote_git_config_value(format!("{common}/config").as_str())
    ));
    lines.push(String::from("[core]"));
    lines.push(format!(
        "\tworktree = {}",
        quote_git_config_value(worktree_root.display().to_string().as_str())
    ));
    lines.push(String::from("[gc]"));
    lines.push(String::from("\tauto = 0"));
    lines.push(String::from("[maintenance]"));
    lines.push(String::from("\tauto = false"));
    lines.push(String::new());
    return lines.join("\n");
}

/// Shadow config controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_config_tests.rs"]
mod tests;
