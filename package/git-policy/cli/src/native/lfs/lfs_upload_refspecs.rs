//! What:
//!  The refs one refspec names:
//! the source it reads,
//! the pattern it expands,
//! and the listing both faces of the upload plan filter on.
//! Why:
//!  `git lfs push` takes refs and no ref-update lines,
//!  so both faces of the plan expand a pattern by a refspec's own rule,
//!  and this file holds that rule,
//!  apart from the plan that reads it so that each file stays short enough for the repository's `rust/max-lines` rule.
//!  The colon it splits on separates a refspec's source from its destination,
//!  and a host from its path in an scp-like remote,
//!  so the one test of a colon serves the plan's destination too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lfs/lfsUploadRefspecs.ts: the refs one refspec names.
//! ```

/// The plan's refs and the errors it reports.
use crate::lfs_upload_settings::{PlanError, Query};
/// Owned OS text for refs.
use std::ffi::{OsStr, OsString};

/// What the source side of a refspec is:
/// nothing for a deletion (`:dst`),
/// the matching marker for `:`,
/// else the part
///  before the last colon (Git splits at the last one,
/// so a source such as `:/message` survives);
/// a leading `+` is dropped.
pub(crate) enum Source {
    /// The refspec only deletes.
    Deletion,
    /// The refspec `:` (or `+:`):
    /// every branch that exists on both sides.
    Matching,
    /// A ref,
    /// a revision expression or a pattern.
    Named(OsString),
}

/// Read the source of one refspec.
pub(crate) fn refspec_source(spec: &OsStr) -> Source {
    let bytes: &[u8] = spec.as_encoded_bytes();
    let unforced: &[u8] = bytes.strip_prefix(b"+").unwrap_or(bytes);
    if unforced == b":" {
        return Source::Matching;
    }
    let source: &[u8] = match unforced.iter().rposition(is_colon_byte) {
        Some(position) => &unforced[..position],
        None => unforced,
    };
    if source.is_empty() {
        return Source::Deletion;
    }
    return Source::Named(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(source.to_vec()));
}

/// Whether one byte is the colon that separates the source of a refspec from its destination,
/// or the host of an
///  scp-like remote from its path.
/// `&u8` borrows the byte,
/// as `position` hands it over.
pub(super) fn is_colon_byte(byte: &u8) -> bool {
    return *byte == b':';
}

/// The refs under a prefix,
/// listed by one `for-each-ref`;
/// with a `(front, back)` pattern,
/// only the full names that
///  start with `front` and end with `back` without the two overlapping,
/// and with `None` every name.
pub(crate) fn list(query: Query<'_>, prefix: &str, pattern: Option<(&str, &str)>) -> Result<Vec<OsString>, PlanError> {
    let listing: String = match query.ask(&["for-each-ref", "--format=%(refname)", prefix]) {
        Ok(text) => text,
        Err(why) => return Err(PlanError(format!("git for-each-ref {prefix}: {why}"))),
    };
    return Ok(names_of(listing.as_str(), pattern));
}

/// What:
///  The names one finished listing keeps,
/// by the pattern the caller asked for.
/// Why:
///   Both faces of the listing keep the same names here,
/// so a pattern either face reads means the same refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listing.split('\n').filter((name) => !pattern || (name.length >= front.length + back.length && name.startsWith(front) && name.endsWith(back)));
/// ```
pub(crate) fn names_of(listing: &str, pattern: Option<(&str, &str)>) -> Vec<OsString> {
    let mut names: Vec<OsString> = Vec::new();
    for name in listing.lines() {
        let kept: bool = match pattern {
            Some((front, back)) => {
                name.len() >= front.len() + back.len() && name.starts_with(front) && name.ends_with(back)
            }
            None => true,
        };
        if kept {
            names.push(OsString::from(name));
        }
    }
    return names;
}

/// What:
///  The directory a `for-each-ref` pattern of a refspec is listed under.
/// Why:
///   Both faces of the expansion list the same directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// front.lastIndexOf('/') >= 0 ? front.slice(0, front.lastIndexOf('/')) : 'refs';
/// ```
pub(crate) fn directory_of(pattern: &str) -> &str {
    let (front, _back) = star_split(pattern);
    return match front.rsplit_once('/') {
        Some((head, _)) => head,
        None => "refs",
    };
}

/// What:
///  The text before a refspec pattern's star and the text after it.
/// Why:
///   Both faces of the expansion filter on the same two pieces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pattern.includes('*') ? pattern.split('*') : [pattern, ''];
/// ```
pub(crate) fn star_split(pattern: &str) -> (&str, &str) {
    return pattern.split_once('*').unwrap_or((pattern, ""));
}

/// The refs a source with a `*` stands for,
/// by a refspec's own rule:
/// the text before the star,
/// the text after it,
/// and
///  anything between,
/// `/` included (which a `for-each-ref` pattern does not do).
pub(crate) fn expand_pattern(query: Query<'_>, pattern: &str) -> Result<Vec<OsString>, PlanError> {
    return list(query, directory_of(pattern), Some(star_split(pattern)));
}
