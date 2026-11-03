//! What:
//! The fetch refspecs of a remote (`+refs/heads/*:refs/remotes/origin/*`) and the mapping of a branch's `merge` reference through
//! them to the remote-tracking reference, as `@{upstream}` does.
//! Why:
//! Git rejects the whole remote when any of its refspecs is invalid, so every refspec is checked before one is used;
//! anything this reader does not know to be valid and plain (negative refspecs, several wildcards, unusual characters)
//! is an error and the caller asks Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-refspec.ts
//! export function mapThroughRefspecs(specs: string[], merge: string): string | undefined  // throws on a spec it cannot judge
//! export function isPlainRefName(name: string): boolean
//! ```

/// What:
/// `pub fn is_plain_ref_name(name: &str) -> bool` says whether a reference name or pattern consists only of characters and
/// shapes that are certainly valid in Git (letters, digits, `_ - . + /`, an optional `*`), with no empty, dot-leading or
/// `.lock` component and no `..`.
/// Why:
/// The reader judges only names it is sure about;
/// anything else (a space, a bracket, Unicode) is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isPlainRefName(name: string): boolean
/// ```
pub fn is_plain_ref_name(name: &str) -> bool {
    if name.is_empty() || name.contains("..") || name.ends_with('.') || name.ends_with('/') || name.starts_with('/') {
        return false;
    }
    if !name.chars().all(|character| { return character.is_ascii_alphanumeric() || "_-.+/*".contains(character); }) {
        return false;
    }
    return name.split('/').all(|part| { return !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"); });
}

/// One side of a refspec that is checked and split at its single wildcard, if any.
struct Side<'a> {
    /// The text before the wildcard (the whole text when there is none).
    prefix: &'a str,
    /// The text after the wildcard,
    /// or `None` when the side has no wildcard.
    suffix: Option<&'a str>,
}

/// What:
/// `fn side(text: &str) -> Result<Side, String>` checks one side of a refspec and splits it at the wildcard.
/// Why:
/// More than one wildcard, or a name that is not plainly valid, is what Git rejects or this reader cannot judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function side(text: string): { prefix: string; suffix?: string }  // throws
/// ```
fn side(text: &str) -> Result<Side<'_>, String> {
    if !is_plain_ref_name(text) {
        return Err(format!("the fetch refspec side {text:?} is not a plain reference name or pattern"));
    }
    return match text.split_once('*') {
        None => Ok(Side { prefix: text, suffix: None }),
        Some((prefix, suffix)) => {
            if suffix.contains('*') {
                return Err(format!("the fetch refspec side {text:?} has more than one wildcard"));
            }
            Ok(Side { prefix, suffix: Some(suffix) })
        }
    };
}

/// A fetch refspec that was checked.
struct Refspec<'a> {
    /// The remote side.
    source: Side<'a>,
    /// The local side,
    /// or `None` for a refspec without one (it maps nothing).
    destination: Option<Side<'a>>,
}

/// What:
/// `fn parse_refspec(spec: &str) -> Result<Refspec, String>` checks one fetch refspec.
/// Why:
/// Both sides must be present and plain, both with a wildcard or both without.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRefspec(spec: string): Refspec  // throws on a spec it cannot judge
/// ```
fn parse_refspec(spec: &str) -> Result<Refspec<'_>, String> {
    let without_force: &str = spec.strip_prefix('+').unwrap_or(spec);
    if without_force.starts_with('^') {
        return Err("a negative fetch refspec is applied by Git".to_string());
    }
    let Some((source_text, destination_text)) = without_force.split_once(':') else {
        return Ok(Refspec { source: side(without_force)?, destination: None });
    };
    let source: Side = side(source_text)?;
    let destination: Side = side(destination_text)?;
    if source.suffix.is_some() != destination.suffix.is_some() {
        return Err(format!("the fetch refspec {spec:?} has a pattern on one side only"));
    }
    return Ok(Refspec { source, destination: Some(destination) });
}

/// What:
/// `pub fn map_through_refspecs(specs: &[&str], merge: &str) -> Result<Option<String>, String>` returns the local reference that
/// the first matching fetch refspec stores `merge` in.
/// Why:
/// Every refspec is checked first (Git rejects the remote otherwise), then the first match in order wins, as in Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function mapThroughRefspecs(specs: string[], merge: string): string | undefined  // throws on a spec it cannot judge
/// ```
pub fn map_through_refspecs(specs: &[&str], merge: &str) -> Result<Option<String>, String> {
    let mut parsed: Vec<Refspec> = Vec::new();
    for spec in specs {
        parsed.push(parse_refspec(spec)?);
    }
    for refspec in &parsed {
        let Some(destination) = &refspec.destination else {
            continue;
        };
        match (&refspec.source.suffix, &destination.suffix) {
            (Some(source_suffix), Some(destination_suffix)) => {
                if let Some(rest) = merge.strip_prefix(refspec.source.prefix)
                    && let Some(middle) = rest.strip_suffix(source_suffix)
                {
                    return Ok(Some(format!("{}{middle}{destination_suffix}", destination.prefix)));
                }
            }
            _ => {
                if refspec.source.prefix == merge {
                    return Ok(Some(destination.prefix.to_string()));
                }
            }
        }
    }
    return Ok(None);
}
