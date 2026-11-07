//! What:
//!  Small text steps of the replay of captured changes:
//! a path as Latin-1 text and back,
//! and whether a line of Git's output is empty.
//! Why:
//!  `service_replay.rs` compares paths by their bytes and reads Git's output by lines;
//! these steps depend on nothing in it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! latin1(bytes); byteOfCharacter(character); isNotEmptyLine(line)
//! ```

/// What:
///  A byte as the Latin-1 character with that code.
/// Why:
///  `latin1` names a path by its bytes,
/// one character per byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const charOfByte = (byte: number) => String.fromCharCode(byte);
/// ```
pub(crate) fn char_of_byte(byte: &u8) -> char {
    return char::from(*byte);
}

/// What:
///  The Latin-1 text of a path given as bytes.
/// Why:
///  The names of the paths in conflict are compared with the shared-path listing by this text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const latin1OfPath = (path: Uint8Array) => latin1(path);
/// ```
pub(crate) fn latin1_of_path(path: &Vec<u8>) -> String {
    return latin1(path.as_slice());
}

/// What:
///  The byte a Latin-1 character stands for,
/// nothing for a character beyond Latin-1.
/// Why:
///  `capture_order` turns the text of a shared path back into the bytes of the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byteOfCharacter = (character: string) => character.charCodeAt(0) < 256 ? character.charCodeAt(0) : undefined;
/// ```
pub(crate) fn byte_of_character(character: char) -> Option<u8> {
    return u8::try_from(u32::from(character)).ok();
}

/// What:
///  Whether a line of Git's output is not empty.
/// Why:
///  The output ends in a line break,
/// so the last line of a split is empty and is no line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotEmptyLine = (line: string) => line !== '';
/// ```
pub(crate) fn is_not_empty_line(line: &&str) -> bool {
    return !line.is_empty();
}

/// The path as the Latin-1 text the shared-path listing names it by,
/// so every byte survives the comparison.
pub(crate) fn latin1(bytes: &[u8]) -> String {
    return bytes.iter().map(char_of_byte).collect();
}
