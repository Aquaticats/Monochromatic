//! What:
//!  A reference name as the bytes Git printed it,
//!  and the spellings a person and a command line take it in.
//! Why:
//!  A ref name is a byte string and not text:
//! Git accepts any bytes a shelf keeps in a file name
//!  (`topic-\xff` is an ordinary branch),
//!  and a spelling with replacement characters names another ref.
//!  Reading `git symbolic-ref HEAD` as text therefore failed a commit Git itself makes
//!  (`doc/troubleshooting/git-policy-cli-ref-names-are-bytes.md`),
//!  and writing such a spelling into a record would move a different branch after a recovery.
//!  The name is kept as an `OsString`,
//! which holds the operating system's own bytes on Unix,
//!  and the spellings are explicit:
//! `as_os_str` names Git,
//! `Display` and `spoken` name the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const refName = { bytes: Buffer.from(printed) }; refName.toString() /* lossy */; shellQuote(refName.bytes)
//! ```

use crate::path_bytes::{argument_bytes, os_from_path_bytes};
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter};

/// The prefix of every local branch,
/// which the wrapper's own words leave out.
const HEADS_PREFIX: &str = "refs/heads/";

/// What:
///  One reference name,
///  as the bytes Git printed.
/// Why:
///   Everything that names a ref to Git takes these bytes,
/// and everything that names it to a person takes a reading of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefName = { os: string /* with its own bytes */ };
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RefName {
    /// The name,
    /// with every byte Git printed.
    os: OsString,
}

/// A name that is not text,
/// and the words a person reads when one reaches a message.
impl RefName {
    /// What:
    ///  The name Git printed,
    ///  without the line terminator that ends the answer.
    /// Why:
    ///   `git symbolic-ref` prints one name and a line feed,
    ///  and the name is everything before it:
    /// a name holding a line feed cannot be a ref,
    ///  so the last line feed is the answer's own.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const refNameFromGit = (printed: Uint8Array) => ({ os: printed.subarray(0, printed.lastIndexOf(10)) })
    /// ```
    pub fn from_git_output(printed: &[u8]) -> Self {
        let end: usize = printed.iter().rposition(|byte| *byte == b'\n').unwrap_or(printed.len());
        return Self { os: os_from_path_bytes(&printed[..end]) };
    }

    /// What:
    ///  A name that is text,
    ///  as a name.
    /// Why:
    ///   Every ref this wrapper makes itself has a text name,
    /// and this is where one becomes a name for the code that reads bytes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const refNameOf = (text: string) => ({ os: text })
    /// ```
    pub fn from_text(text: &str) -> Self {
        return Self { os: OsString::from(text) };
    }

    /// What:
    ///  A name whose bytes a record held in hexadecimal.
    /// Why:
    ///   A record is text,
    /// so a name that is not text travels in it as its bytes
    ///  (`path_bytes_in_records`), and a reader turns those back into the name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const refNameOfHex = (hex: string) => ({ os: Buffer.from(hex, 'hex') })
    /// ```
    pub fn from_hex(hex: &str) -> Option<Self> {
        return Some(Self { os: os_from_path_bytes(crate::path_bytes_in_records::bytes_of_hex(hex)?.as_slice()) });
    }

    /// What:
    ///  The name's bytes,
    ///  as the operating system holds them.
    /// Why:
    ///   Git takes bytes,
    /// and the shadow's ref files and the landing's transaction are written as bytes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Buffer.from(refName.os)
    /// ```
    pub fn bytes(&self) -> &[u8] {
        return argument_bytes(self.os.as_os_str());
    }

    /// What:
    ///  The name as the argument a Git child receives.
    /// Why:
    ///   A command line is bytes,
    /// so this is the spelling every `update-ref`,
    /// `rev-parse` and `symbolic-ref` takes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refName.os
    /// ```
    pub fn as_os_str(&self) -> &OsStr {
        return self.os.as_os_str();
    }

    /// What:
    ///  The name's bytes as lowercase hexadecimal digits.
    /// Why:
    ///   A line-oriented record holds a name that is not text only in this form,
    /// and the digits are the whole name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Buffer.from(refName.os).toString('hex')
    /// ```
    pub fn hex(&self) -> String {
        return crate::path_bytes_in_records::hex_of_bytes(self.bytes());
    }

    /// What:
    ///  Whether the name is text,
    ///  so a record written by an earlier release can hold it as it reads.
    /// Why:
    ///   The plain `ref=` field of a record is read by releases that predate the bytes field,
    /// so it is written whenever a text spelling is exact.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const isText = /^[\x00-\x7f]*$/u.test(refName.os) && new TextDecoder('utf-8', { fatal: true }).decode(refName.os) === refName.os
    /// ```
    pub fn is_text(&self) -> bool {
        return self.os.to_str().is_some();
    }

    /// What:
    ///  The name as text,
    ///  nothing when a byte of it is not text.
    /// Why:
    ///   A record and a push name a branch in text,
    ///  so the one place that may take a name there asks this first
    ///  instead of spelling it with replacement characters.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = new TextDecoder('utf-8', { fatal: true }).decode(refName.os) // undefined when it does not decode
    /// ```
    pub fn text(&self) -> Option<&str> {
        return self.os.to_str();
    }

    /// What:
    ///  The name without the `refs/heads/` its branch words leave out.
    /// Why:
    ///   The wrapper's own messages name a branch by its short name,
    /// as Git's own summary does.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refName.bytes.slice('refs/heads/'.length)
    /// ```
    pub fn short(&self) -> Self {
        let bytes: &[u8] = self.bytes();
        let Some(rest) = bytes.strip_prefix(HEADS_PREFIX.as_bytes()) else { return self.clone(); };
        return Self { os: os_from_path_bytes(rest) };
    }

    /// What:
    ///  The name as one word a shell reads back,
    ///  with the `$'...'` form for a byte that is not text.
    /// Why:
    ///   A person who reads a branch name in a message may type it,
    /// and a name that is not text is typed only in that form.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// shellQuote(refName.os)
    /// ```
    pub fn spoken(&self) -> String {
        return crate::shell_quote::shell_quote(self.bytes());
    }

    /// What:
    ///  Whether this name is `HEAD` itself,
    /// which the landing moves as the head and not as a branch.
    /// Why:
    ///   A detached head is recorded as `HEAD`,
    /// so this question is asked about every landing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refName.os === 'HEAD'
    /// ```
    pub fn is_head(&self) -> bool {
        return self.os.as_os_str() == OsStr::new("HEAD");
    }
}

/// The name in the wrapper's own messages,
/// where a name that is not text is shown with the replacement characters of its bytes.
impl Display for RefName {
    /// Writes the name's readable text.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.os.to_string_lossy().as_ref());
    }
}

/// Reference-name controls stay out of the release executable.
#[cfg(test)]
#[path = "ref_name_tests.rs"]
mod tests;
