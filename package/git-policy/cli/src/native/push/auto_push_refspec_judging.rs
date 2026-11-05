//! What:
//!  The reading of a configured push refspec and the judgement of what the refspecs make of one branch: which remote names
//!  receive it, which pattern or negative refspec applies, and the refusal when two refspecs collide.
//! Why:
//!  `auto_push_destination.rs` decides where a plain push goes and asks Git for the local refs; the judgement itself only
//!  reads refspec text and a list of ref names, so it sits in a file of its own that does not use the destination file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function judge(branchRef: string, remote: string, refspecs: string[], localRefs: string[]): Judged;
//! ```

use std::collections::BTreeMap;

/// One `remote.<name>.push` refspec this module interprets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refspec = { negative: boolean; source: string; destination?: string; pattern: boolean };
/// ```
pub(crate) struct Refspec {
    /// A `^` refspec: it removes matching destinations.
    pub(crate) negative: bool,
    /// The local side.
    pub(crate) source: String,
    /// The remote side; the source when absent.
    pub(crate) destination: Option<String>,
    /// Whether the sides hold one `*`.
    pub(crate) pattern: bool,
}

/// What:
///  Read a configured push refspec, or nothing when it is a form this module does not interpret.
/// Why:
///   Only full ref names and one `*` per side are interpreted; `HEAD`, short names, object names, deletions and the
///  `:` of matching depend on more than the local ref names, so they are never guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRefspec(text: string): Refspec | undefined;
/// ```
pub(crate) fn parse_refspec(text: &str) -> Option<Refspec> {

    if let Some(source) = text.strip_prefix('^') {
        let negative_ok = source.starts_with("refs/") && !source.contains(':') && star_count(source) <= 1;
        if negative_ok {
            return Some(Refspec { negative: true, source: source.to_owned(), destination: None, pattern: star_count(source) == 1 });
        }
        return None;
    }
    let spec = text.strip_prefix('+').unwrap_or(text);
    let (source, destination) = match spec.split_once(':') {
        Some((source, destination)) => (source, Some(destination)),
        None => (spec, None),
    };
    if !source.starts_with("refs/") || side_outside_refs(destination) {
        return None;
    }
    let pattern = star_count(source) == 1;
    let sides_agree = star_count(source) == destination.map_or(star_count(source), star_count);
    if star_count(source) > 1 || !sides_agree {
        return None;
    }
    return Some(Refspec { negative: false, source: source.to_owned(), destination: destination.map(str::to_owned), pattern });
}

/// How many `*` the side of a refspec holds.
pub(crate) fn star_count(side: &str) -> usize {
    return side.matches('*').count();
}

/// Whether a refspec has a destination side that is not a full ref name.
pub(crate) fn side_outside_refs(destination: Option<&str>) -> bool {
    return match destination {
        Some(side) => !side.starts_with("refs/"),
        None => false,
    };
}

/// The text a `*` pattern captured in `name`, or nothing when `name` does not match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function capture(pattern: string, name: string): string | undefined;
/// ```
pub(crate) fn capture<'a>(pattern: &str, name: &'a str) -> Option<&'a str> {
    let (prefix, suffix) = pattern.split_once('*')?;
    let middle = name.strip_prefix(prefix)?.strip_suffix(suffix)?;
    return Some(middle);
}

/// Whether a negative refspec removes the remote ref `name`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function excludes(negative: Refspec, name: string): boolean;
/// ```
pub(crate) fn excludes(negative: &Refspec, name: &str) -> bool {
    if negative.pattern {
        return capture(&negative.source, name).is_some();
    }
    return negative.source == name;
}

/// What the configured push refspecs make of one branch.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Judged {
    /// The refs on the remote that receive the branch, possibly none.
    Targets(Vec<String>),
    /// Git refuses the whole push; the sentence says why.
    Refused(String),
    /// A refspec of a form not interpreted; its text.
    Unjudged(String),
}

/// What:
///  Apply the configured push refspecs to the local refs the way Git's `match_push_refs` does, and report where the branch goes.
/// Why:
///   Git matches every explicit refspec first (a source that is not a local ref, or two sources for one destination, fail
///  the whole push), then lets each local ref take the first matching pattern unless an earlier pairing already took its
///  destination, then drops the destinations a negative refspec names.
///  The `%(push:remoteref)` atom of `git for-each-ref` maps one branch alone, so it misses all three and cannot be used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judge({ branchRef, refspecs, localRefs }): Judged;
/// ```
pub(crate) fn judge(branch_ref: &str, remote: &str, refspecs: &[String], local_refs: &[String]) -> Judged {
    let mut items: Vec<Refspec> = Vec::new();
    for text in refspecs {
        match parse_refspec(text) {
            Some(item) => items.push(item),
            None => return Judged::Unjudged(text.clone()),
        }
    }
    let mut taken: BTreeMap<String, String> = BTreeMap::new();
    for item in &items {
        if item.negative || item.pattern {
            continue;
        }
        if !local_refs.contains(&item.source) {
            return Judged::Refused(format!(
                "`remote.{remote}.push` names `{}`, which is not a ref here, and Git refuses to push anything while it is not.",
                item.source
            ));
        }
        let destination: String = match item.destination.clone() {
            Some(side) => side,
            None => item.source.clone(),
        };
        if let Some(earlier) = taken.insert(destination.clone(), item.source.clone()) {
            return Judged::Refused(format!(
                "`remote.{remote}.push` sends both `{earlier}` and `{}` to `{destination}`, and Git refuses that (\"receives from more than one src\").",
                item.source
            ));
        }
    }
    let mut sorted: Vec<&String> = local_refs.iter().collect();
    sorted.sort();
    for name in sorted {
        if let Some(destination) = mapped_destination(&items, name)
            && !taken.contains_key(&destination)
        {
            taken.insert(destination, name.clone());
        }
    }
    for negative in &items {
        if !negative.negative {
            continue;
        }
        let mut kept: BTreeMap<String, String> = BTreeMap::new();
        for (destination, source) in taken {
            if !excludes(negative, &destination) {
                kept.insert(destination, source);
            }
        }
        taken = kept;
    }
    let mut targets: Vec<String> = Vec::new();
    for (destination, source) in taken {
        if source == branch_ref {
            targets.push(destination);
        }
    }
    return Judged::Targets(targets);
}

/// Where the first pattern refspec sends the local ref `name`, or nothing when no pattern matches it.
pub(crate) fn mapped_destination(items: &[Refspec], name: &str) -> Option<String> {
    for item in items {
        if item.negative || !item.pattern {
            continue;
        }
        let Some(middle) = capture(&item.source, name) else {
            continue;
        };
        let side: &str = item.destination.as_deref().unwrap_or(item.source.as_str());
        return Some(side.replacen('*', middle, 1));
    }
    return None;
}
