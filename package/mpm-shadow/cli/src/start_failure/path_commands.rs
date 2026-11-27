//! What:
//! The `mpm` commands a PATH lookup read,
//! printed in the order it read them with the reason it took or passed over each,
//! and the message for a PATH that holds no real `mpm`.
//! Why:
//! A message that sends a person to sort out their `mpm` commands has to show which ones exist:
//! a file without its execute bit,
//! a directory of that name and a link that leads nowhere need three different repairs,
//! and `command -v -a mpm` is refused by bash and by zsh,
//! so the shadow prints the list itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function describeMissingRealMpm(lookup: Lookup): string
//! ```

/// Imports the name the shadow answers to,
/// which every sentence here spells.
use crate::constants::SHADOWED_NAME;
/// Imports one `mpm` the lookup saw,
/// the lookup's whole answer,
/// and why it passed one over.
use crate::real_binary::{
    Candidate,
    Lookup,
    SkipReason,
};

/// What:
/// The line that introduces the `mpm` commands a lookup read,
/// and one line per candidate under it,
/// each with the reason the lookup took it or passed it over.
/// Why:
/// Every message that tells a person to sort out their `mpm` commands must show which ones were seen:
/// a file without its execute bit and a directory of that name need different repairs,
/// and neither can be guessed from "no real `mpm` found".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandsOnPath(lookup: Lookup): string
/// ```
pub(super) fn commands_on_path(lookup: &Lookup) -> String {
    let mut lines: Vec<String> = vec![format!(
        "  the `{SHADOWED_NAME}` commands this run found on PATH, in the order the lookup read them:"
    )];
    if lookup.candidates.is_empty() {
        lines.push(format!(
            "    none: no directory on PATH holds a file named `{SHADOWED_NAME}`"
        ));
        return lines.join("\n");
    }
    for candidate in &lookup.candidates {
        lines.push(format!(
            "    {}: {}",
            candidate.path.display(),
            candidate_note(candidate)
        ));
    }
    return lines.join("\n");
}

/// What:
/// What one candidate's line says about it:
/// that the lookup chose it,
/// or why it passed it over.
/// Why:
/// The chosen line has to be recognizable as the program the message already names,
/// and each skipped line has to carry its own repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function candidateNote(candidate: Candidate): string
/// ```
fn candidate_note(candidate: &Candidate) -> String {
    return match &candidate.skipped_because {
        None => "chosen by the lookup".to_string(),
        Some(reason) => skipped_note(reason),
    };
}

/// What:
/// The words that name why the lookup passed one `mpm` over.
/// Why:
/// A person acts on the reason,
/// so it has to say what the file is rather than repeat that it was skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skippedNote(reason: SkipReason): string
/// ```
fn skipped_note(reason: &SkipReason) -> String {
    return match reason {
        SkipReason::ThisShadow => "skipped: it is this shadow".to_string(),
        SkipReason::LauncherOfThisShadow => {
            "skipped: a launcher whose record names this shadow".to_string()
        }
        SkipReason::Shadow {
            identity,
        } => format!("skipped: a shadow ({identity})"),
        SkipReason::Directory => "skipped: a directory, not a program".to_string(),
        SkipReason::NoExecuteBit {
            mode,
        } => format!("skipped: a file with no execute bit (mode {mode:04o})"),
        SkipReason::BrokenLink {
            target,
        } => format!(
            "skipped: a symbolic link that leads to no file (it points at {})",
            target.display()
        ),
        SkipReason::NotARegularFile => "skipped: not a regular file".to_string(),
    };
}

/// What:
/// The message for a lookup that found no real `mpm` at all:
/// every candidate it saw with the reason it was passed over,
/// and the remedy that fits what was found.
/// Why:
/// The message used to say that `mpm` is not installed,
/// that every `mpm` is a shadow,
/// or that an install moved it
/// aside,
/// and a file without its execute bit is none of the three:
/// naming the file and its mode is the only sentence that leads to `chmod`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function describeMissingRealMpm(lookup: Lookup): string
/// ```
pub fn describe_missing_real_mpm(lookup: &Lookup) -> String {
    let opening: String = if lookup.candidates.is_empty() {
        format!(
            "mpm-shadow: no real {SHADOWED_NAME} found on PATH: nothing named `{SHADOWED_NAME}` is in any directory on PATH"
        )
    } else {
        format!(
            "mpm-shadow: no real {SHADOWED_NAME} found on PATH: every `{SHADOWED_NAME}` the lookup saw was skipped, and the reason stands beside each"
        )
    };
    return format!(
        "{opening}\n{commands}\nmpm-shadow: what to do: {remedy}",
        commands = commands_on_path(lookup),
        remedy = missing_remedy(lookup),
    );
}

/// What:
/// The remedy for a lookup that found no real `mpm`,
/// built from what the candidates are.
/// Why:
/// A file that lost its execute bit is repaired by `chmod`,
/// a directory of that name by replacing or dropping it,
/// and a link that leads nowhere by pointing it somewhere;
/// only when none of those is there is installing the real `mpm` the whole answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingRemedy(lookup: Lookup): string
/// ```
fn missing_remedy(lookup: &Lookup) -> String {
    let install: String = format!(
        "install the real `{SHADOWED_NAME}` (this shadow starts it and never provides it), or put its directory on PATH after this shadow's"
    );
    let mut steps: Vec<String> = Vec::new();
    for candidate in &lookup.candidates {
        let path: String = candidate.path.display().to_string();
        match &candidate.skipped_because {
            Some(SkipReason::NoExecuteBit {
                ..
            }) => steps.push(format!("make `{path}` runnable (`chmod +x {path}`)")),
            Some(SkipReason::Directory) => steps.push(format!(
                "replace the directory `{path}` with the real `{SHADOWED_NAME}`, or take its directory off PATH"
            )),
            Some(SkipReason::BrokenLink {
                ..
            }) => steps.push(format!("point the link `{path}` at the real `{SHADOWED_NAME}`")),
            _ => {}
        }
    }
    if steps.is_empty() {
        return format!("{install}, then run the command again");
    }
    return format!(
        "{}, then run the command again. Installing the real `{SHADOWED_NAME}` (this shadow starts it and never provides it), or putting its directory on PATH after this shadow's, also ends this",
        steps.join("; ")
    );
}
