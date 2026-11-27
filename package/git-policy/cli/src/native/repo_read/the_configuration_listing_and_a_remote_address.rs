//! What:
//!  The answers that read only the configuration:
//!  the merged listing,
//!  and the address of a remote.
//! Why:
//!  Neither needs a ref or an object,
//!  so neither walks the ref storage,
//!  and both stood in the file that answers the reference command lines.
//!  A file holds one job:
//!  this one reads configuration,
//!  and `repo_read_query_refs.rs` reads a ref and peels a name to its commit.
//!  The words Git prints when it refuses a remote are read here too,
//!  because the locale that decides them is a setting a person writes,
//!  and the reference answers reach that reading through this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listed = configList(context, parsed); // undefined: start Git
//! const url = remoteUrl(context, parsed);
//! ```

#![cfg(unix)]

use crate::repo_read_config_list::{
    ListedEntry,
    list_config,
    listing_bytes,
};
use crate::repo_read_query_request_answer_and_context::{
    Answer,
    Context,
    Parsed,
    flags_are,
};
use std::ffi::OsString;

/// The listing,
/// or `None`.
pub(crate) fn listing(context: &Context) -> Option<Vec<ListedEntry>> {
    return list_config(
        context.layout.common_dir.as_path(),
        context.layout.git_dir.as_path(),
        &context.scope,
        context.variables.as_slice(),
    );
}

/// The last non-empty text value of the variable `name` among `variables`,
/// or nothing.
fn text_variable(
    variables: &[(OsString, OsString)],
    name: &str,
) -> Option<String> {
    for (candidate, value) in variables.iter().rev() {
        if candidate == name {
            return match value.to_str() {
                Some(text) if !text.is_empty() => Some(text.to_string()),
                _ => None,
            };
        }
    }
    return None;
}

/// Whether a locale name is one whose messages are English:
/// `C`,
/// `POSIX` or starting with `en`.
fn is_english(value: &str) -> bool {
    return value == "C" || value == "POSIX" || value.starts_with("en");
}

/// Whether `byte` may appear in a remote name this reader takes:
/// a letter,
/// a digit,
/// `-`,
/// `_` or `.`.
fn is_remote_name_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.');
}

/// Whether Git's messages are in English under these variables,
/// which the synthesized failures assume.
pub(crate) fn messages_are_english(variables: &[(OsString, OsString)]) -> bool {
    let mut locale: Option<String> = text_variable(variables, "LC_ALL");
    if locale.is_none() {
        locale = text_variable(variables, "LC_MESSAGES");
    }
    if locale.is_none() {
        locale = text_variable(variables, "LANG");
    }
    if let Some(name) = locale.as_deref()
        && !is_english(name)
    {
        return false;
    }
    if let Some(name) = text_variable(variables, "LANGUAGE")
        && !is_english(name.as_str())
    {
        return false;
    }
    return true;
}

/// `config --list -z --show-scope`.
pub(crate) fn config_list(
    context: &Context,
    parsed: &Parsed<'_>,
) -> Option<Answer> {
    if !flags_are(&parsed.flags, &["--list", "-z", "--show-scope"], &[])
        || !parsed.operands.is_empty()
        || !parsed.specs.is_empty()
    {
        return None;
    }
    let listed: Vec<ListedEntry> = listing(context)?;
    return Some(Answer {
        stdout: listing_bytes(listed.as_slice()),
        stderr: Vec::new(),
        code: 0,
    });
}

/// `remote get-url <name>` for a remote with one plain URL and no rewriting rule anywhere.
pub(crate) fn remote_url(
    context: &Context,
    parsed: &Parsed<'_>,
) -> Option<Answer> {
    if !parsed.flags.is_empty()
        || !parsed.specs.is_empty()
        || parsed.operands.len() != 2
        || parsed.operands[0] != b"get-url"
    {
        return None;
    }
    let name: &str = std::str::from_utf8(parsed.operands[1]).ok()?;
    if name.is_empty() || !name.bytes().all(is_remote_name_byte) {
        return None;
    }
    let listed: Vec<ListedEntry> = listing(context)?;
    for item in &listed {
        if item.entry.key.starts_with("url.")
            && (item.entry.key.ends_with(".insteadof")
                || item.entry.key.ends_with(".pushinsteadof"))
        {
            return None;
        }
    }
    let key: String = format!("remote.{name}.url");
    let mut urls: Vec<&ListedEntry> = Vec::new();
    for item in &listed {
        if item.entry.key == key {
            urls.push(item);
        }
    }
    if urls.is_empty() {
        // A remote that is configured nowhere, not even by a legacy `remotes/` or `branches/` file, is Git's own refusal.
        let prefix: String = format!("remote.{name}.");
        for directory in ["remotes", "branches"] {
            if std::fs::symlink_metadata(context.layout.common_dir.join(directory).join(name))
                .is_ok()
            {
                return None;
            }
        }
        for item in &listed {
            if item.entry.key.starts_with(prefix.as_str()) {
                return None;
            }
        }
        if !messages_are_english(context.variables.as_slice()) {
            return None;
        }
        let message: String = format!("error: No such remote '{name}'\n");
        return Some(Answer {
            stdout: Vec::new(),
            stderr: message.into_bytes(),
            code: 2,
        });
    }
    let [only] = urls.as_slice() else {
        return None;
    };
    let value: Vec<u8> = only.entry.value.clone()?;
    if value.is_empty() {
        return None;
    }
    let mut stdout: Vec<u8> = value;
    stdout.push(b'\n');
    return Some(Answer {
        stdout,
        stderr: Vec::new(),
        code: 0,
    });
}
