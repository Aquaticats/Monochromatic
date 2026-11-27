//! What:
//!  The clause a sentence about a commit ends with when no reference of the repository is proven to name it.
//! Why:
//!  Two files say it and neither defines the other:
//! the file that lands a batch without its scratch reference,
//!  and the file that writes that reference.
//!  Held at the bottom of the folder,
//! it is the one sentence they share.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! noReferenceProves(tip) // "and no reference is proven to name it, ..."
//! ```

/// What:
///  The clause a sentence about a commit ends with when no reference of the repository is proven to name it.
/// Why:
///   A landed batch is named by its branch and a kept one by `refs/cli-git/kept/<batch>`,
///  so a commit that no reference names is an ending the wrapper has to say out loud:
///  a maintenance that drops what no reference names takes it,
///  and the one thing left to give the person is the object's own name and the command that makes a reference of it
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `and no reference is proven to name it, ... keep it with \`git update-ref refs/heads/<the name you want> ${tip}\``
/// ```
pub(crate) fn the_words_for_a_commit_no_reference_names(tip: &str) -> String {
    return format!(
        "and no reference is proven to name it, so a maintenance that drops what no reference names would take it; keep it with `git update-ref refs/heads/<the name you want> {tip}`"
    );
}
