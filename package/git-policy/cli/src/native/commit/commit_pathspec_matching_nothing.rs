//! What:
//!  The refusal of an explicit-path commit whose pathspec matches no path Git can stage,
//! read from the words of the private staging's own `git add`
//! and named as the caller's input error.
//! Why:
//!  Native Git decides which pathspecs are valid,
//! so nothing here predicts a match:
//! a directory,
//! a pattern,
//! a magic selector and a literal name reach Git and are answered exactly as before.
//!  Git refuses a pathspec that matches nothing before anything lands and before anything reaches the real index,
//!  so that start's own failure is what this classifies:
//!  "pathspec ... did not match any files" for a pathspec the caller wrote is the caller's input error,
//!  and every other failure of the step stays an engine failure (issue 6283).
//!  An option written after `--` is a pathspec to Git,
//! which is the shape issue 6283 reported,
//!  so a token that begins with a hyphen also says where options belong.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refused = [...stderr.matchAll(/pathspec '(.*)' did not match any files/g)].map((found) => found[1]);
//! ```

use crate::path_bytes::path_message_text;
use std::ffi::OsString;

/// The core of the finding:
///  the fixed behavior of an explicit-path commit's selection,
/// which no configuration can turn off.
pub const PATHSPEC_MATCHING_NOTHING_CORE_ID: &str = "explicit-path-commit";

/// The code of the finding,
/// reported as `explicit-path-commit/pathspec-matches-nothing`.
pub const PATHSPEC_MATCHING_NOTHING_CODE: &str = "pathspec-matches-nothing";

/// The bytes Git puts before the pathspec it names.
const NAMES_A_PATHSPEC: &[u8] = b"pathspec '";

/// The two wordings Git ends that refusal with:
///  `git add` says "files",
///  and `git commit` says "file(s) known to git".
///  Both are recognized,
/// because the pathspec may reach either of them.
const REFUSAL_ENDINGS: [&[u8]; 2] = [b"' did not match any files", b"' did not match any file(s) known to git"];

/// What:
///  The pathspecs one staging start carries,
/// as far as Git is concerned,
/// and therefore the set a refusal may be attributed to.
/// Why:
///  A pathspec the wrapper wrote itself belongs to the wrapper,
/// and a match it misses must stay an engine failure:
///  so the caller's named elements are listed and compared,
///  while a pathspec file is Git's to read and every path in it is the caller's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Written = { kind: 'elements'; elements: string[] } | { kind: 'theCallersFile' };
/// ```
pub enum WrittenPathspecs<'written> {
    /// The elements the caller named on the command line,
    /// which this staging put on its own.
    Elements(&'written [&'written OsString]),
    /// Every pathspec of the start,
    /// which Git read from the caller's own pathspec file.
    FromTheCallersFile,
}

/// What:
///  The pathspecs Git named in `stderr` as matching nothing,
/// in Git's order,
/// and `None` when Git's words are not that refusal or name no pathspec of the caller's.
/// Why:
///  Git's words are the only evidence of which pathspec matched nothing,
/// so every named token is kept only when this start could have read it from the caller,
///  and a message of any other kind answers `None`,
/// which leaves the failure with its engine-failure report.
///  The bytes are kept as Git wrote them,
/// so a path that is not valid UTF-8 is compared and shown as the caller spelled it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = namedRefusals(stderr).filter((token) => written.holds(token)); return refused.length ? refused : null;
/// ```
pub fn pathspec_matching_nothing(stderr: &[u8], written: WrittenPathspecs<'_>) -> Option<Vec<Vec<u8>>> {
    let mut refused: Vec<Vec<u8>> = Vec::new();
    for token in named_refusals(stderr) {
        if written.holds(token.as_slice()) && !refused.contains(&token) {
            refused.push(token);
        }
    }
    if refused.is_empty() {
        return None;
    }
    return Some(refused);
}

impl WrittenPathspecs<'_> {
    /// What:
    ///  Whether this start could have read `token` as the caller's own pathspec.
    /// Why:
    ///  The comparison is over the bytes the caller wrote,
    /// so the answer needs no decoding and holds for any path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// this.kind === 'theCallersFile' || this.elements.includes(token);
    /// ```
    fn holds(&self, token: &[u8]) -> bool {
        let Self::Elements(elements) = self else { return true };
        for element in elements.iter() {
            if element.as_encoded_bytes() == token {
                return true;
            }
        }
        return false;
    }
}

/// What:
///  Every pathspec Git's standard error names as matching nothing,
/// in the order it names them.
/// Why:
///  A token is only read between the wording Git puts around it,
/// so no other sentence of Git's can be mistaken for a refusal,
///  and the scan moves forward from one reading to the next,
/// so it reads the bytes once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tokens = []; for (const found of stderr.matchAll(/pathspec '(.*?)' did not match any files?/g)) tokens.push(found[1]);
/// ```
fn named_refusals(stderr: &[u8]) -> Vec<Vec<u8>> {
    let mut named: Vec<Vec<u8>> = Vec::new();
    let mut at: usize = 0;
    while let Some(found) = find(stderr, NAMES_A_PATHSPEC, at) {
        let start: usize = found + NAMES_A_PATHSPEC.len();
        match refusal_end(stderr, start) {
            Some(end) => {
                named.push(stderr[start..end].to_vec());
                at = end;
            }
            // A sentence this reader does not recognize is passed over, so a message that only resembles a refusal names nothing.
            None => at = start,
        }
    }
    return named;
}

/// What:
///  The offset of the first byte of the wording that closes a refusal named at `start`,
/// and `None` when Git's sentence has no such wording.
/// Why:
///  The nearest closing wording is the one that belongs to this pathspec,
/// so a token that itself holds a closing wording cannot swallow the rest of the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ends = ENDINGS.map((ending) => stderr.indexOf(ending, start)).filter((at) => at >= 0); return Math.min(...ends);
/// ```
fn refusal_end(stderr: &[u8], start: usize) -> Option<usize> {
    let mut nearest: Option<usize> = None;
    for ending in REFUSAL_ENDINGS {
        let Some(found) = find(stderr, ending, start) else { continue };
        let is_nearer: bool = match nearest {
            Some(at) => found < at,
            None => true,
        };
        if is_nearer {
            nearest = Some(found);
        }
    }
    return nearest;
}

/// What:
///  The offset of `needle` in `haystack` at or after `from`,
/// and `None` when it is not there.
/// Why:
///  The needles here are fixed wordings,
/// so every slice of the needle's own length is compared once as the scan advances one byte at a time,
///  and a position past the end reads nothing instead of panicking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = haystack.indexOf(needle, from); return at < 0 ? null : at;
/// ```
fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    let rest: &[u8] = haystack.get(from..)?;
    let mut at: usize = 0;
    while at + needle.len() <= rest.len() {
        if &rest[at..at + needle.len()] == needle {
            return Some(from + at);
        }
        at += 1;
    }
    return None;
}

/// What:
///  The message of the finding:
/// the pathspecs Git refused,
/// what happened to the commit,
/// and the one way forward that fits the shape of the refused token.
/// Why:
///  Git's own sentence names the token,
/// so the report repeats it instead of naming a step of the wrapper.
///  A token that begins with a hyphen is the report of issue 6283,
/// where the option belongs before the separator,
///  and telling someone to stage a token that is an option would be wrong advice,
/// so the two shapes get one sentence each and never both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ... pathspec ${listed} did not match any files.${hyphen ? ' An option written after -- is a pathspec to Git: ...' : ' Correct the spelling ...'}`
/// ```
pub fn pathspec_matching_nothing_message(tokens: &[Vec<u8>]) -> String {
    let mut named: Vec<String> = Vec::new();
    for token in tokens {
        named.push(format!("'{}'", path_message_text(token.as_slice())));
    }
    let listed: String = named.join(", ");
    let mut message: String = format!(
        "monochromatic-git-policy-cli: git commit with explicit paths selects worktree content, and Git refuses a \
pathspec that matches nothing, so this commit was not made and nothing was staged by it: pathspec {listed} did not match \
any files."
    );
    if tokens.iter().map(Vec::as_slice).any(begins_with_a_hyphen) {
        message.push_str(
            " An option written after -- is a pathspec to Git: options belong before --, as in git commit --message <text> -- <paths>.",
        );
        return message;
    }
    message.push_str(" Correct the spelling of the path, or stage what it names with git add -- <path> when the path is new.");
    return message;
}

/// What:
///  Whether one pathspec token begins with a hyphen,
/// which makes Git read it as no path at all unless it was separated.
/// Why:
///  That shape is the report of issue 6283,
/// and it is the one shape whose remedy is where the option belongs,
///  so the test for it is named instead of written at the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// token[0] === '-'
/// ```
fn begins_with_a_hyphen(token: &[u8]) -> bool {
    return token.first() == Some(&b'-');
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_pathspec_matching_nothing_tests.rs"]
mod tests;
