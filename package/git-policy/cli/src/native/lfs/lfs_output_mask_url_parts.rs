//! What:
//!  The parts of a URL,
//! read as bytes:
//! where its scheme,
//! its authority and the rest of it end,
//! which host a destination or a text names,
//! and the one word that replaces a secret in it.
//! Why:
//!  The mask in `lfs_output_mask.rs` and the host names a refusal prints
//! read a URL through these same rules,
//! and the two files pass the package's length limit only apart.
//! A host comes out of the same authority rule the mask uses,
//! so a token in front of it is never part of what is returned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const authorityEnd = urlAuthorityEnd(text, start); const host = urlHostOf(destination);
//! ```
/// What replaces the userinfo of a URL and every other secret-shaped part of it.
pub const CREDENTIALS_MASK: &str = "<credentials>";

/// Whether the mask this module writes starts at `offset` of `text`.
/// Why:
///   Masking runs again on text that was already masked,
///  because the attempt the auto-push records is masked where it is made and
///  once more where it is written down;
///  and the mask holds the `>` and `<` this module reads as the end of a URL,
///  so a second pass that did not know it over its own mask would cut the URL short and
///  print the mask again beside it (`?token=<credentials><credentials>`) or drop a host.
fn mask_starts_at(
    text: &[u8],
    offset: usize,
) -> bool {
    return text.get(offset..).is_some_and(|rest| {
        return rest.starts_with(CREDENTIALS_MASK.as_bytes());
    });
}

/// What:
///  The part of a destination that names the server a person must recognize:
/// the host and its port,
/// with any userinfo left out.
/// Why:
///   A refusal that says which server asked for a login is actionable,
/// and one that repeats the destination as typed would carry the token of an
///  `lfs.url` written with one.
/// A destination with no URL authority (a remote name,
/// a local path,
/// `file://`) has no host to name,
///  and answers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hostOf(destination: string): string | undefined
/// ```
pub fn host_of(destination: &str) -> Option<String> {
    // The mask this module writes stands where the userinfo was, and its angle brackets end a URL's authority, so it is
    //  removed first: `https://lfs:<credentials>@host` names `host`, not `lfs:`.
    let stripped: String = destination.replace(CREDENTIALS_MASK, "");
    // `as_bytes()` lends the text as raw bytes, so the rules below match what a URL really holds.
    let bytes: &[u8] = stripped.as_bytes();
    let separator: usize = next_separator(bytes, 0)?;
    // A separator that follows no scheme character is a path, not a URL.
    if separator == 0 || !ends_scheme(bytes[separator - 1]) {
        return None;
    }
    let authority_start: usize = separator + SCHEME_SEPARATOR.len();
    let authority_end: usize = authority_end_of(bytes, authority_start);
    let authority: &[u8] = &bytes[authority_start..authority_end];
    // The host begins after the last `@`, so a user and a token are never part of it.
    let host_start: usize = match authority.iter().rposition(is_at_sign) {
        Some(offset) => offset + 1,
        None => 0,
    };
    let host: &[u8] = &authority[host_start..];
    if host.is_empty() {
        return None;
    }
    // `from_utf8_lossy` reads a host that is not UTF-8 without failing; a host name is text in practice.
    return Some(String::from_utf8_lossy(host).into_owned());
}

/// The separator between a URL's scheme and its authority.
pub(crate) const SCHEME_SEPARATOR: &[u8] = b"://";

/// Whether a byte can end a scheme,
/// so that the separator after it starts a URL's authority.
pub(crate) fn ends_scheme(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'-' || byte == b'.';
}

/// Whether a byte ends the authority of a URL.
fn ends_authority(byte: u8) -> bool {
    return byte.is_ascii_whitespace()
        || byte.is_ascii_control()
        || matches!(
            byte,
            b'/' | b'?' | b'#' | b'\\' | b'"' | b'\'' | b'<' | b'>' | b'`'
        );
}

/// Whether a byte ends the path,
/// query and fragment of a URL.
/// A `/`,
/// `?` and `#` are part of them,
/// so only the characters that end a URL in running text are here.
fn ends_url(byte: u8) -> bool {
    return byte.is_ascii_whitespace()
        || byte.is_ascii_control()
        || matches!(byte, b'\\' | b'"' | b'\'' | b'<' | b'>' | b'`');
}

/// Whether a byte is the `@` that ends the userinfo of a URL.
/// `&u8` borrows the byte,
/// as `rposition` hands it over.
pub(crate) fn is_at_sign(byte: &u8) -> bool {
    return *byte == b'@';
}

/// Whether an offset into an authority is a byte for the userinfo,
/// which needs at least one byte before it.
pub(crate) fn is_not_the_first(offset: &usize) -> bool {
    return *offset > 0;
}

/// The index of the next scheme separator at or after `from`.
pub(crate) fn next_separator(
    text: &[u8],
    from: usize,
) -> Option<usize> {
    for (offset, window) in text
        .get(from..)?
        .windows(SCHEME_SEPARATOR.len())
        .enumerate()
    {
        if window == SCHEME_SEPARATOR {
            return Some(from + offset);
        }
    }
    return None;
}

/// The end of the authority that starts at `from`:
/// the first byte that ends one,
/// or the end of the text;
/// a mask already written stands inside the authority as it stands.
pub(crate) fn authority_end_of(
    text: &[u8],
    from: usize,
) -> usize {
    let mut offset: usize = from;
    while offset < text.len() {
        if mask_starts_at(text, offset) {
            offset += CREDENTIALS_MASK.len();
            continue;
        }
        if ends_authority(text[offset]) {
            return offset;
        }
        offset += 1;
    }
    return text.len();
}

/// The end of the URL whose authority ended at `from`:
/// the first byte that ends a URL in running text,
/// or the end of the text;
/// a mask already written is part of the URL it stands in.
pub(crate) fn url_end_of(
    text: &[u8],
    from: usize,
) -> usize {
    let mut offset: usize = from;
    while offset < text.len() {
        if mask_starts_at(text, offset) {
            offset += CREDENTIALS_MASK.len();
            continue;
        }
        if ends_url(text[offset]) {
            return offset;
        }
        offset += 1;
    }
    return text.len();
}

/// What:
///  Every server a text names,
/// in the order it names them,
/// each as its host and port with any userinfo left out.
/// Why:
///   A refusal that says which server asked for a login is actionable,
/// and the program's own words name it:
/// Git quotes the URL in "could not read Username for '<http://host:port>'",
///  as the incident's own run printed,
/// and git-lfs names it in "Git credentials for
///  <http://host:port/lfs> not found."
/// The host comes out of the same authority rule the mask uses,
/// so a token in front of it is never part of what is returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hostsIn(text: string): string[]
/// ```
pub fn hosts_in(text: &str) -> Vec<String> {
    // The mask this module writes stands where the userinfo was, and its angle brackets end a URL's authority, so it is
    //  removed first: `https://lfs:<credentials>@host` names `host`, not `lfs:`.
    let stripped: String = text.replace(CREDENTIALS_MASK, "");
    // `as_bytes()` lends the text as raw bytes, so every URL in it is scanned by the rules below.
    let bytes: &[u8] = stripped.as_bytes();
    let mut hosts: Vec<String> = Vec::new();
    let mut from: usize = 0;
    while let Some(separator) = next_separator(bytes, from) {
        // A separator that follows no scheme character starts no URL, and the scan goes on after it.
        if separator == 0 || !ends_scheme(bytes[separator - 1]) {
            from = separator + SCHEME_SEPARATOR.len();
            continue;
        }
        let authority_start: usize = separator + SCHEME_SEPARATOR.len();
        let authority_end: usize = authority_end_of(bytes, authority_start);
        // The next search starts where this authority ended, so each byte is read a constant number of times.
        from = authority_end;
        let authority: &[u8] = &bytes[authority_start..authority_end];
        let host_start: usize = match authority.iter().rposition(is_at_sign) {
            Some(offset) => offset + 1,
            None => 0,
        };
        let host: &[u8] = &authority[host_start..];
        if host.is_empty() {
            continue;
        }
        let named: String = String::from_utf8_lossy(host).into_owned();
        if !hosts.contains(&named) {
            hosts.push(named);
        }
    }
    return hosts;
}
