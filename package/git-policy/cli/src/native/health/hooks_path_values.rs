//! What:
//!  Pathnames as Git prints them:
//! one answer with its one terminator removed,
//! a pathname from the bytes of it,
//! and every value of `core.hooksPath` with the scope it came from.
//! Why:
//!   A pathname is a byte string,
//! and a configured directory whose name holds a line feed or bytes that are not UTF-8
//!  is still a directory Git runs hooks in;
//! reading them as lines lost both,
//! and the listing of a path-valued key needs
//!  its scopes to tell the wrapper's own override from the person's.
//!  The report and its location questions share these,
//! and they are a file of their own so the report's own file stays within the package's size limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const value = withoutTheTerminator(await git(args)); const dir = pathAnswer(value);
//! ```

/// Pathnames are owned and built from bytes here.
use std::path::PathBuf;

/// What:
///  One answer without the single line feed Git ends it with.
/// Why:
///   Exactly one line feed is the terminator;
/// a pathname that ends with a line feed of its own keeps it,
///  which trimming every trailing line feed threw away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bytes.endsWith('\n') ? bytes.slice(0, -1) : bytes
/// ```
pub(crate) fn without_the_terminator(answer: &[u8]) -> &[u8] {
    return match answer.strip_suffix(b"\n".as_slice()) {
        Some(body) => body,
        None => answer,
    };
}

/// What:
///  A pathname from the bytes Git printed for it.
/// Why:
///   A pathname is a byte string here,
/// so a directory whose name is not UTF-8 is scanned like any other,
///  as Git runs the hooks in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(bytes); // Node pathnames are byte strings
/// ```
#[cfg(unix)]
pub(crate) fn path_answer(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    return PathBuf::from(std::ffi::OsStr::from_bytes(bytes));
}

/// What:
///  A pathname from the bytes Git printed for it,
/// for a platform that spells pathnames in text.
/// Why:
///   Refusing a pathname Git named would hide a directory it runs hooks in,
/// and naming it approximately is
///  better than not looking at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder().decode(bytes);
/// ```
#[cfg(not(unix))]
pub(crate) fn path_answer(bytes: &[u8]) -> PathBuf {
    return PathBuf::from(String::from_utf8_lossy(bytes).into_owned());
}

/// What:
///  Whether one byte of the listing is the NUL that ends a record.
/// Why:
///   The listing is split at that byte,
/// and the split takes a named rule rather than a closure that captures
///  nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isANulByte = (byte: number) => byte === 0;
/// ```
fn is_a_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The `core.hooksPath` directory the person configured,
/// when Git reports one that is not the wrapper's own.
/// Why:
///   Git reports every value of the key and the last one wins,
/// and a command-scope entry the wrapper injected travels into a child
///  Git,
/// so the wrapper's own hooks-off directory is last in a run inside one.
/// The person's value is the last one that
///  is not the wrapper's,
/// an empty one names no directory,
/// and a value that cannot be decoded is not read.
///   The values are read from the NUL-delimited listing Git was asked for:
/// a directory whose name holds a line break
///  is one value,
/// and splitting on line breaks read it as two,
/// the second of which named a directory the person
///  never chose.
///  Each record is `scope NUL value NUL`,
/// so the wrapper's own override is recognised by the scope it came from
/// (`is_wrapper_hooks_path`)
///  and a file that merely spells that path stays the person's.
///  A record that is not UTF-8 is no value the wrapper wrote,
/// and it names a directory Git runs,
/// so it  is kept as the bytes it arrived as.
///  An empty value is the person's own reset,
/// so it answers no directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const records = (await git(['config', '--type=path', '--null', '--show-scope', '--get-all', 'core.hooksPath'])).split('\0').slice(0, -1);
/// for (let at = 0; at + 1 < records.length; at += 2) if (!isWrapperHooksPath(records[at], records[at + 1])) chosen = records[at + 1];
/// ```
pub(crate) fn configured_hooks_directory(listing: &[u8]) -> Option<PathBuf> {
    // Git terminates every record with a NUL, so the empty piece after the last one is no record of its own.
    let body: &[u8] = match listing.strip_suffix(0u8.to_be_bytes().as_slice()) {
        Some(rest) => rest,
        None => listing,
    };
    if body.is_empty() {
        return None;
    }
    let mut chosen: Option<PathBuf> = None;
    let mut fields = body.split(is_a_nul_byte);
    while let (Some(scope), Some(value)) = (fields.next(), fields.next()) {
        let scope_text: &str = std::str::from_utf8(scope).unwrap_or_default();
        let wrapper_wrote_it: bool = match std::str::from_utf8(value) {
            Ok(text) => crate::traditional_hook_warning::is_wrapper_hooks_path(scope_text, text),
            Err(_) => false,
        };
        if wrapper_wrote_it {
            continue;
        }
        chosen = Some(path_answer(value));
    }
    // An empty value is the person's own reset and names no directory, so it is no answer at all.
    return match chosen {
        Some(value) if value.as_os_str().is_empty() => None,
        other => other,
    };
}
