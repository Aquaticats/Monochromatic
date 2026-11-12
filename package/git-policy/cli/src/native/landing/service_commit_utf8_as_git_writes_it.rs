//! What:
//!  The repair Git applies to a commit's bytes before it writes the object:
//!  every byte sequence that is not valid UTF-8 replaced by the Latin-1 character
//!  of each of its bytes,
//!  which is Git 2.56.0's `ensure_utf8` in `commit.c`
//!  applied to the whole buffer `write_commit_tree` built,
//!  headers and message together.
//! Why:
//!  The service writes a commit object through `git fast-import`,
//!  which records the identities and the message exactly as given,
//! while real `git commit`
//!  repairs them and warns in Git's own words.
//!  So a committer name held in Latin-1 bytes was recorded as the replacement character
//!  where real Git records the bytes of its repair,
//! and the audit that found it holds the two objects side by side.
//!  The rule is Git's own, byte for byte:
//! a sequence that is too long,
//!  too short,
//! overlong for its length,
//!  a surrogate,
//! an out-of-range code point,
//!  or a non-character is not text,
//! and neither is a byte that begins no sequence.
//!  Git repairs one byte at a time and rescans,
//! so a byte of a broken sequence is repaired as the Latin-1 character it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { bytes, repaired } = repairCommitUtf8(raw); // Git's ensure_utf8
//! ```

/// What Git prints to standard error after it repaired a commit's bytes,
/// word for word,
/// so a person sees the same warning from either program.
pub const COMMIT_UTF8_WARN: &str = "Warning: commit message did not conform to UTF-8.\nYou may want to amend it after fixing the message, or set the config\nvariable i18n.commitEncoding to the encoding your project uses.\n";

/// The largest code point each sequence length may hold,
/// as Git's `max_codepoint` table states it:
/// one continuation byte, two, then three.
const MAX_CODEPOINT: [u32; 4] = [0x7f, 0x7ff, 0xffff, 0x10ffff];

/// The first sequence of `bytes` that is not UTF-8 text,
/// as the offset it starts at,
/// or nothing when every byte is text.
/// Why:
///  Git's `has_invalid_utf8` answers exactly this question,
/// and the repair repeats it after every repaired byte,
///  so the rule and its order are Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = firstInvalid(bytes.subarray(from)); // Git's has_invalid_utf8
/// ```
fn first_invalid(bytes: &[u8]) -> Option<usize> {
    let mut offset: usize = 0;
    while offset < bytes.len() {
        let (lead, rest): (u8, &[u8]) = (bytes[offset], &bytes[offset + 1..]);
        // Plain ASCII is text, and it is the byte most names are made of.
        if lead < 0x80 {
            offset += 1;
            continue;
        }
        // However many high bits are set beyond the first says how many more bytes the sequence has.
        let mut shifted: u8 = lead;
        let mut extra: usize = 0;
        while shifted & 0x40 != 0 {
            shifted <<= 1;
            extra += 1;
        }
        // A sequence may hold one, two or three more bytes; a longer one names a code point that cannot exist.
        if !(1..=3).contains(&extra) {
            return Some(offset);
        }
        let Some(continuations) = rest.get(..extra) else { return Some(offset) };
        // Git reads the payload bits out of the shifted copy, not out of the byte as it arrived.
        let mut codepoint: u32 = u32::from((shifted & 0x7f) >> extra);
        for byte in continuations {
            codepoint <<= 6;
            codepoint |= u32::from(byte & 0x3f);
            if byte & 0xc0 != 0x80 {
                return Some(offset);
            }
        }
        let least: u32 = MAX_CODEPOINT[extra - 1] + 1;
        let most: u32 = MAX_CODEPOINT[extra];
        if codepoint < least || codepoint > most {
            return Some(offset);
        }
        // A surrogate belongs to UTF-16 and no surrogate is UTF-8 text.
        if codepoint & 0x1ff800 == 0xd800 {
            return Some(offset);
        }
        // U+xFFFE and U+xFFFF are guaranteed non-characters,
        // and so is everything from U+FDD0 to U+FDEF.
        if codepoint & 0xfffe == 0xfffe || (0xfdd0..=0xfdef).contains(&codepoint) {
            return Some(offset);
        }
        offset += 1 + extra;
    }
    return None;
}

/// What:
///  `bytes` repaired the way Git repairs a commit's bytes,
/// and whether any byte was not text as given.
/// Why:
///   The caller records the repaired bytes in the commit,
/// and warns in Git's words only when the repair changed something,
///  as Git does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repaired = repairCommitUtf8(Buffer.from('Ren\xe9')); // { bytes: Buffer.from('Ren\xc3\xa9'), repaired: true }
/// ```
pub fn repair_commit_utf8(bytes: &[u8]) -> (Vec<u8>, bool) {
    let mut repaired: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut at: usize = 0;
    let mut changed: bool = false;
    while at < bytes.len() {
        let Some(bad) = first_invalid(&bytes[at..]) else {
            repaired.extend_from_slice(&bytes[at..]);
            break;
        };
        repaired.extend_from_slice(&bytes[at..at + bad]);
        // The byte that began nothing is read as the Latin-1 character it is.
        let byte: u8 = bytes[at + bad];
        repaired.push(0xc0 | (byte >> 6));
        repaired.push(0x80 | (byte & 0x3f));
        changed = true;
        at += bad + 1;
    }
    return (repaired, changed);
}

/// What:
///  A command's own bytes repaired as Git repairs them,
/// as text,
/// or nothing when even the repair is not text.
/// Why:
///   A commit's identities travel to the service as text,
/// and Git's repair always ends in text,
///  so this is the one place the two meet;
/// `None` keeps a byte the repair cannot make text out of the service's hands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repaired = repairedTextBytes(raw); // { text: 'Ren\u00e9', repaired: true }
/// ```
pub fn repaired_text_bytes(bytes: &[u8]) -> (Option<String>, bool) {
    let (repaired, changed): (Vec<u8>, bool) = repair_commit_utf8(bytes);
    return (String::from_utf8(repaired).ok(), changed);
}

/// The repair's own controls.
#[cfg(test)]
#[path = "service_commit_utf8_as_git_writes_it_tests.rs"]
mod tests;
