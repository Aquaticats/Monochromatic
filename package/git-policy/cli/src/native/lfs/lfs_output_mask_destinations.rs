//! What:
//!  The server a destination or a line names,
//! as its host and port,
//! with any user information left out.
//! Why:
//!  A refusal that says which server asked for a login is actionable,
//! and one that repeats the destination as typed would carry the token of an `lfs.url` written with
//!  one.
//! The program's own words name the server:
//! Git quotes the URL in "could not read Username for '<http://host:port>'",
//! and git-lfs names it in "Git credentials for <http://host:port/lfs> not found."
//! The host comes out of the same authority rule the mask uses,
//! so a token in front of it is never part of what is returned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hostOf = (destination: string) => new URL(destination).host || undefined
//! ```

use super::CREDENTIALS_MASK;
use super::lfs_output_mask_reading_a_url::{SCHEME_SEPARATOR, authority_end_of, ends_scheme, is_at_sign, next_separator};

/// What:
///  The part of a destination that names the server a person must recognize:
/// the host and its port,
/// with any user information left out.
/// Why:
///   A destination with no URL authority (a remote name,
/// a local path,
/// `file://`) has no host to name,
/// and answers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hostOf(destination: string): string | undefined
/// ```
pub fn host_of(destination: &str) -> Option<String> {
    let stripped: String = without_the_mask(destination);
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

/// What:
///  Every server a text names,
/// in the order it names them,
/// each as its host and port with any user information left out.
/// Why:
///   A failure's own words name more than one server when a push falls back from one to another,
/// and a person needs each of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hostsIn(text: string): string[]
/// ```
pub fn hosts_in(text: &str) -> Vec<String> {
    let stripped: String = without_the_mask(text);
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

/// What:
///  The text with the mask removed.
/// Why:
///   The mask this module writes stands where the user information was,
/// and its angle brackets end a URL's authority,
/// so `https://lfs:<credentials>@host` would name `lfs:` and not `host`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutTheMask = (text: string) => text.replaceAll(MASK, '')
/// ```
fn without_the_mask(text: &str) -> String {
    return text.replace(CREDENTIALS_MASK, "");
}
