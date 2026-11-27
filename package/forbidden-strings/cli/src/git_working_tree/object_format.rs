//! What:
//!  The object format a repository's configuration records,
//!  read as `gix-index` must parse its entries by it.
//! Why:
//!  A Sha1 kind applied to a Sha256 index lands every entry's flags and path
//!  offsets twelve bytes off,
//!  so the wrong answer here corrupts paths instead of failing.
//!  Anything but an explicit `extensions.objectformat = sha256` means Sha1,
//!  which is what a missing or unreadable configuration gets.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // function indexHashKind(configPath: string): 'sha1' | 'sha256'
//! ```

/// Imports the path type of the configuration file that records the format.
use std::path::Path;

/// The object format the repository's configuration records:
///  `Sha256` only when `extensions.objectformat` names it,
///  `Sha1` otherwise.
///
/// `gix_index::File::at` parses entries by that format,
///  so a Sha1 kind on a Sha256 index lands every entry's flags and path
///  offsets twelve bytes off and the paths come out silently corrupted.
///  Reading the wrong configuration therefore corrupts paths rather than
///  failing.
pub(crate) fn index_hash_kind(config_path: &Path) -> gix_hash::Kind {
    let Ok(config) = std::fs::read_to_string(config_path) else {
        return gix_hash::Kind::Sha1;
    };
    let mut in_extensions: bool = false;
    for raw in config.lines() {
        // Strip line comments (`# ...`, `; ...`) and surrounding whitespace; only the value before them is a signal.
        let line: &str = raw.split(['#', ';']).next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        // Section headers are `[section]` or `[section "subsection"]`; only the section name selects `objectformat`.
        if let Some(rest) = line.strip_prefix('[') {
            let section: &str = rest.trim_end_matches(']').trim();
            let name: &str = section.split_whitespace().next().unwrap_or("");
            in_extensions = name.eq_ignore_ascii_case("extensions");
            continue;
        }
        if !in_extensions {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("objectformat")
            && value.trim().eq_ignore_ascii_case("sha256")
        {
            return gix_hash::Kind::Sha256;
        }
    }
    return gix_hash::Kind::Sha1;
}
