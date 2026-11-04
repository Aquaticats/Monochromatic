//! What:
//!  The two Git vocabulary types every candidate carries:
//!  an object name and a file mode.
//! Why:
//!  Object names are written into the object reader's request stream,
//!  so only text
//!      proven to be a complete hexadecimal name may ever become one;
//!  modes decide
//!      whether a candidate has blob content at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ObjectId = string & { readonly brand: 'ObjectId' };
//! type CandidateMode = 'regular' | 'executable' | 'symlink' | 'gitlink';
//! ```

/// What:
///  Length in hexadecimal digits of a SHA-1 object name.
///       `usize` is the unsigned integer every length and index uses
///       (siblings `u32`,
///  `u64`,
///  `i64`).
/// Why:
///   `usize` is what `.len()` returns,
///  so comparing needs no conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHA1_HEX_LENGTH = 40;
/// ```
pub const SHA1_HEX_LENGTH: usize = 40;

/// Length in hexadecimal digits of a SHA-256 object name,
///  for `--object-format=sha256` repositories.
pub const SHA256_HEX_LENGTH: usize = 64;

/// What:
///  A complete Git object name:
///  40 or 64 lowercase hexadecimal digits.
///       The field is private,
///  so the only way to obtain a value is `parse_object_id`.
///       `String` owns the text (sibling `&str` would borrow Git's output buffer).
///       `Hash` lets the value be a `HashMap` key.
/// Why:
///   A value of this type is safe to send as one request line:
///  it cannot contain
///       a line feed,
///  a space or a revision expression.
///  The name outlives the Git
///       output it was parsed from,
///  so it owns its text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ObjectId = string & { readonly brand: 'ObjectId' };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ObjectId {
    /// Validated lowercase hexadecimal text.
    hex: String,
}

/// What:
///  `impl ObjectId { ... }` attaches a read-only accessor,
///  like a class getter.
/// Why:
///   Callers print or send the name without being able to change it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a branded string is used directly
/// ```
impl ObjectId {
    /// What:
    ///  Lend the name as text.
    ///  `&self` borrows the value read-only;
    ///  `&str` is a
    ///       borrowed view into the owned `String`.
    /// Why:
    ///   Request lines and Git arguments are built from this view without copying.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    pub fn as_str(&self) -> &str {
        return self.hex.as_str();
    }
}

/// What:
///  Named predicate:
///  is this byte a lowercase hexadecimal digit?
///       `&u8` borrows one byte (`u8` is 0 to 255).
///  `.is_ascii_digit()` is true for
///       `0` to `9`;
///  `b'a'..=b'f'` is an inclusive byte range.
/// Why:
///   Git 2.56.0 prints object names in lowercase only;
///  accepting uppercase would
///       give one object two spellings and defeat lookups keyed by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLowerHex = (byte: number): boolean => /[0-9a-f]/.test(String.fromCharCode(byte));
/// ```
fn is_lower_hex(byte: &u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(byte);
}

/// What:
///  Accept bytes as an object name only when they are exactly 40 or 64 lowercase
///       hexadecimal digits.
///  `&[u8]` borrows bytes;
///  `Option<ObjectId>` is "a name or nothing".
/// Why:
///   Everything Git prints as an object name passes;
///  anything else (abbreviations,
///       revision expressions,
///  text containing a line feed) is refused before it can
///       reach the reader's request stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseObjectId(bytes: Uint8Array): ObjectId | undefined;
/// ```
pub fn parse_object_id(bytes: &[u8]) -> Option<ObjectId> {
    if bytes.len() != SHA1_HEX_LENGTH && bytes.len() != SHA256_HEX_LENGTH {
        // `None` is the "absent" variant.
        return None;
    }
    // What: `.iter().all(is_lower_hex)` asks the named predicate about every byte.
    // Why:  One pass decides validity before any text is built.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!bytes.every(isLowerHex)) return undefined;
    // ```
    if !bytes.iter().all(is_lower_hex) {
        return None;
    }
    // What: `String::new()` is an empty owned string; `char::from(*byte)` turns one
    //       ASCII byte into a character.
    // Why:  Every byte was just proven ASCII, so no decoding step can fail.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hex = String.fromCharCode(...bytes);
    // ```
    let mut hex: String = String::new();
    for byte in bytes {
        hex.push(char::from(*byte));
    }
    // `Some(...)` is the "present" variant.
    return Some(ObjectId { hex });
}

/// What:
///  The file modes Git records for a path that can be a candidate.
/// Why:
///   Regular,
///  executable and symbolic-link entries name a blob;
///  a gitlink names a
///       commit of another repository (a submodule),
///  which this repository's object
///       store does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateMode = 'regular' | 'executable' | 'symlink' | 'gitlink';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateMode {
    /// Mode `100644`:
    ///  a non-executable file.
    Regular,
    /// Mode `100755`:
    ///  an executable file.
    Executable,
    /// Mode `120000`:
    ///  a symbolic link whose blob holds the link target.
    Symlink,
    /// Mode `160000`:
    ///  a submodule commit,
    ///  with no blob in this repository.
    Gitlink,
}

/// What:
///  Map Git's six-digit mode text to a candidate mode.
///       `b"100644"` is a byte-string literal,
///  compared with `==` byte for byte.
/// Why:
///   Any other mode (a directory `040000`,
///  the absent side `000000`) cannot be a
///       candidate;
///  the caller reports it instead of guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modeFromGit(text: string): CandidateMode | undefined;
/// ```
pub fn mode_from_git(text: &[u8]) -> Option<CandidateMode> {
    if text == b"100644" {
        return Some(CandidateMode::Regular);
    }
    if text == b"100755" {
        return Some(CandidateMode::Executable);
    }
    if text == b"120000" {
        return Some(CandidateMode::Symlink);
    }
    if text == b"160000" {
        return Some(CandidateMode::Gitlink);
    }
    return None;
}

/// Name and mode controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_object_tests.rs"]
mod tests;
