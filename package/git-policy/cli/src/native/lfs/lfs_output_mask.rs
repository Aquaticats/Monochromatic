//! What:
//!  What the wrapper shows of a URL in the text it relays from `git-lfs` and from Git:
//! the scheme,
//! the host and its port,
//! the path,
//! the query and the fragment as they are,
//! with the user information and every part [`lfs_output_mask_shapes`] recognizes as a secret replaced by
//!  `<credentials>`.
//! Why:
//!  An `lfs.url` can carry its token in three places:
//! `https://user:token@host/...` in its user information,
//! `https://host/<token>/lfs` in a path segment,
//! and `https://host/lfs?token=<token>` in its query.
//! `git-lfs` names the URL it used in its errors,
//! the wrapper relays that text to the caller's terminal and,
//! for the auto-push,
//! records it in files,
//! so the places that can carry a token are what this module reads
//! (owner,
//! "Never print or commit secrets").
//!  The owner decided on 2026-11-12T10:32:10-05:00 (item 1.9 of `doc/handover/cli-git-rust-implementation.md`)
//! what to do about the tension between that rule and a reader who needs the path:
//! "Mask recognised secrets only".
//! A URL is shown,
//! and what is recognized as a secret in it is masked.
//! The cost was stated in the question and is the limit written here:
//!  a secret of a shape no rule knows is printed.
//! This module replaced an earlier rule that hid the path,
//! the query and the fragment whole,
//! and before that one that kept a plain lowercase label.
//! The rules that recognize a secret,
//! each with the documentation its shape comes from,
//! are in [`lfs_output_mask_shapes`];
//! this module reads the parts of a URL and applies them.
//!
//!  The authority of a URL ends at the first `/`,
//! `?`,
//! `#`,
//! `\`,
//! quote,
//! angle bracket,
//! backtick or white space,
//! and its user information ends at the last `@` before that,
//! so an `@` in a path,
//! a query or a bracketed IPv6 host is left alone.
//! The rest of the URL ends at the first `\`,
//! quote,
//! angle bracket,
//! backtick,
//! white space or control character.
//! The user information is always hidden:
//! a URL that carries a user and a password there carries them as a secret,
//! and nothing in its bytes can tell a user's name from a token.
//!
//!  What is not recognized, and is therefore printed,
//! is written here as the limit of that decision:
//!
//!  - a secret shorter than the run rules below,
//! such as a six-character access code in a path;
//!  - a secret made of ordinary words that names nothing,
//! such as `?ref=bluebird`, because no rule can tell it from a branch name;
//!  - a secret in a host name,
//! such as `https://<token>.example.invalid/`, because the host is the one part of a URL a person must read
//!  and no issuer we know puts a secret there;
//!  - a credential of a shape no rule here names,
//! such as a new token format of a host this package has not read the documentation of;
//!  - the first bytes of a token when an output is cut inside it,
//! shorter than the rule that would recognize it.
//!
//!  The head of an output kept at its size cap is masked as the head of a longer text it is:
//! a URL that reaches the end of what was kept may go on past it,
//! so an authority with no `@` that runs to that end is hidden whole
//! ([`mask_url_secrets_cut`]).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const masked = maskUrlSecrets(text) // 'https://user:secret@host/tokenvalue/lfs?token=abc'
//!                                     // -> 'https://<credentials>@host/<credentials>/lfs?token=<credentials>'
//! ```

/// The rules that recognize a secret in one part of a URL or one word of a line,
/// with the documentation each shape comes from.
#[path = "lfs_output_mask_shapes.rs"]
mod shapes;

use self::shapes::{holds_a_secret, is_a_credential, names_a_secret, scheme_credential_in, follows_a_word_byte};

//region The mask and the bytes that end a URL

/// What replaces the user information of a URL and every part of it that is recognized as a secret.
pub const CREDENTIALS_MASK: &str = "<credentials>";

/// The separator between a URL's scheme and its authority.
const SCHEME_SEPARATOR: &[u8] = b"://";

/// What:
///  How many lowercase hexadecimal characters an LFS object's name has:
/// the 256-bit hash the batch API calls its `oid`.
/// Why:
///   An object's name is not a secret:
/// it is a hash of the object's own bytes,
/// it is how a person names the object a failed upload lost,
/// and LFS writes it in a path segment
/// (`docs/api/batch.md` of git-lfs,
/// "`oid` - String OID of the LFS object",
/// and this repository's own server,
/// which answers a batch with `href` of `<origin>/<oid>`,
/// `package/config/lfs-r2-worker/src/batch.ts`,
/// "const href = `${origin}/${oid}`").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OBJECT_ID_BYTES = 64
/// ```
const OBJECT_ID_BYTES: usize = 64;

/// The name of the namespace an object's name sits in when an LFS server puts it there.
const OBJECTS_SEGMENT: &[u8] = b"objects";

/// Whether a byte can end a scheme,
/// so that the separator after it starts a URL's authority.
fn ends_scheme(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'-' || byte == b'.';
}

/// Whether a byte ends the authority of a URL.
fn ends_authority(byte: u8) -> bool {
    return byte.is_ascii_whitespace()
        || byte.is_ascii_control()
        || matches!(byte, b'/' | b'?' | b'#' | b'\\' | b'"' | b'\'' | b'<' | b'>' | b'`');
}

/// Whether a byte ends the path,
/// query and fragment of a URL.
/// A `/`,
/// `?` and `#` are part of them,
/// so only the characters that end a URL in running text are here.
fn ends_url(byte: u8) -> bool {
    return byte.is_ascii_whitespace() || byte.is_ascii_control() || matches!(byte, b'\\' | b'"' | b'\'' | b'<' | b'>' | b'`');
}

/// Whether a byte is the `@` that ends the user information of a URL.
/// `&u8` borrows the byte,
/// as `rposition` hands it over.
fn is_at_sign(byte: &u8) -> bool {
    return *byte == b'@';
}

/// Whether a byte is the `=` that ends a query or fragment key.
/// `&u8` borrows the byte,
/// as `position` hands it over.
fn is_equals_sign(byte: &u8) -> bool {
    return *byte == b'=';
}

/// Whether a byte separates two parts of a path.
fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// The index of the next `/` of a path,
/// or nothing when the path ends first.
fn next_slash(path: &[u8]) -> Option<usize> {
    return path.iter().position(is_slash);
}

/// The index of the next `#` of a query,
/// or nothing when the query ends first.
fn next_hash(query: &[u8]) -> Option<usize> {
    return query.iter().position(is_hash);
}

/// Whether a byte separates two parts of a query or a fragment.
fn is_ampersand(byte: &u8) -> bool {
    return *byte == b'&';
}

/// Whether a byte begins the fragment of a URL.
fn is_hash(byte: &u8) -> bool {
    return *byte == b'#';
}

/// Whether a byte begins the query of a URL,
/// or its fragment.
fn begins_the_rest(byte: &u8) -> bool {
    return *byte == b'?' || *byte == b'#';
}

/// Whether a byte is one this module keeps as the punctuation that ends a URL in a sentence.
/// `&&u8` borrows the byte twice,
/// as a reversed iterator's `take_while` hands it over.
fn ends_a_sentence(byte: &&u8) -> bool {
    return matches!(**byte, b',' | b';' | b')' | b']' | b'}' | b'.' | b'!' | b'?' | b':' | b'\'');
}

/// Whether an offset into an authority is a byte for the user information,
/// which needs at least one byte before it.
fn is_not_the_first(offset: &usize) -> bool {
    return *offset > 0;
}

/// The index of the next scheme separator at or after `from`.
fn next_separator(text: &[u8], from: usize) -> Option<usize> {
    for (offset, window) in text.get(from..)?.windows(SCHEME_SEPARATOR.len()).enumerate() {
        if window == SCHEME_SEPARATOR {
            return Some(from + offset);
        }
    }
    return None;
}

/// The end of the authority that starts at `from`:
/// the first byte that ends one,
/// or the end of the text.
fn authority_end_of(text: &[u8], from: usize) -> usize {
    for (offset, byte) in text[from..].iter().enumerate() {
        if ends_authority(*byte) {
            return from + offset;
        }
    }
    return text.len();
}

/// The end of the URL whose authority ended at `from`:
/// the first byte that ends a URL in running text,
/// or the end of the text.
fn url_end_of(text: &[u8], from: usize) -> usize {
    for (offset, byte) in text[from..].iter().enumerate() {
        if ends_url(*byte) {
            return from + offset;
        }
    }
    return text.len();
}

//endregion

//region The parts of a URL

/// What:
///  Append one path segment:
/// as it is when it holds no secret,
/// [`CREDENTIALS_MASK`] when it does,
/// and nothing when it is empty.
/// Why:
///   A segment is the part of a URL a person most needs after the host:
/// `owner/repo.git/info/lfs` says which repository the push went to,
/// and `objects/batch` says which call failed,
/// so a segment is shown unless a rule recognizes a secret in it.
/// An empty segment (the one before a leading `/`,
/// after a trailing `/`,
/// or between two) carries nothing and stays empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// masked.push(segment === '' ? '' : holdsSecret(segment) ? '<credentials>' : segment)
/// ```
fn push_masked_segment(segment: &[u8], after_objects: bool, is_last: bool, masked: &mut Vec<u8>) {
    if segment.is_empty() {
        return;
    }
    if is_an_object_id(segment, after_objects, is_last) {
        masked.extend_from_slice(segment);
        return;
    }
    if holds_a_secret(segment) {
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
        return;
    }
    masked.extend_from_slice(segment);
}

/// What:
///  Whether a path segment is an LFS object's name,
/// which is shown though its shape is a run of hexadecimal characters.
/// Why:
///   The mask hides a run of 32 or more hexadecimal characters,
/// and an object's name is 64 of them,
/// so without this rule the one thing a person needs to see of a failed object upload is hidden.
/// The name is recognized by where the batch API puts it, not by its shape:
/// it is a whole segment of exactly [`OBJECT_ID_BYTES`] lowercase hexadecimal characters,
/// directly after a segment named `objects`,
/// or the last segment of the path,
/// which is where a content-addressed store puts the object it names
/// (this repository's server answers `<origin>/<oid>`,
/// and an S3-compatible bucket answers `<bucket>/<oid>`).
///  A secret written as a whole path segment of exactly that shape in one of those two places is shown
/// with it:
/// no rule can tell the two apart, and this module resolves that in favour of the object's name,
/// which is the reading the owner gave the mask ("the thing a person needs to see").
/// The same 64 characters in a query value,
/// in a fragment,
/// or in the middle of a path are hidden.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isAnObjectId(segment, afterObjects, isLast) {
///   return (afterObjects || isLast) && /^[0-9a-f]{64}$/.test(segment)
/// }
/// ```
fn is_an_object_id(segment: &[u8], after_objects: bool, is_last: bool) -> bool {
    if !after_objects && !is_last {
        return false;
    }
    if segment.len() != OBJECT_ID_BYTES {
        return false;
    }
    return segment.iter().all(|byte| { return byte.is_ascii_digit() || (b'a'..=b'f').contains(byte); });
}

/// Append the path of a URL with each of its secret-shaped segments masked.
fn push_masked_path(path: &[u8], masked: &mut Vec<u8>) {
    let mut from: usize = 0;
    let mut after_objects: bool = false;
    loop {
        match path.get(from..).and_then(next_slash) {
            Some(offset) => {
                let segment: &[u8] = &path[from..from + offset];
                push_masked_segment(segment, after_objects, false, masked);
                after_objects = segment == OBJECTS_SEGMENT;
                masked.push(b'/');
                from += offset + 1;
            }
            None => {
                let segment: &[u8] = path.get(from..).unwrap_or_default();
                push_masked_segment(segment, after_objects, true, masked);
                return;
            }
        }
    }
}

/// What:
///  Append a query or fragment with the value of every setting that names a secret hidden,
/// and every value of a recognized shape hidden,
/// showing the keys and the separators.
/// Why:
///   The name of a setting is what says whether its value is a secret:
/// `?token=` and `?X-Amz-Signature=` hide their values whatever the values look like,
/// while `?ref=main` keeps a value a person reads,
/// so the mask hides the value and keeps the name,
/// which is the part that tells a person what the URL carried.
/// A value of a recognized shape is hidden under any name,
/// and a key that is itself a token of a recognized shape is hidden too,
/// because a token can be written as the name of a setting.
/// A setting with no `=` is a value with no name,
/// and is hidden when its shape is a secret's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chunks.map((chunk) => chunk.includes('=') ? `${maskKey(key)}=${maskValue(key, value)}` : maskBare(chunk)).join('&')
/// ```
fn push_masked_values(values: &[u8], masked: &mut Vec<u8>) {
    let mut from: usize = 0;
    loop {
        let (chunk, separator): (&[u8], Option<usize>) = match values.get(from..) {
            Some(rest) => match rest.iter().position(is_ampersand) {
                Some(offset) => (&values[from..from + offset], Some(from + offset)),
                None => (&values[from..], None),
            },
            None => (&[], None),
        };
        push_masked_setting(chunk, masked);
        match separator {
            Some(offset) => {
                masked.push(b'&');
                from = offset + 1;
            }
            None => return,
        }
    }
}

/// Append one query or fragment setting with the part of it that is a secret masked.
fn push_masked_setting(chunk: &[u8], masked: &mut Vec<u8>) {
    let Some(equals) = chunk.iter().position(is_equals_sign) else {
        if chunk.is_empty() {
            return;
        }
        if holds_a_secret(chunk) {
            masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
            return;
        }
        masked.extend_from_slice(chunk);
        return;
    };
    let key: &[u8] = &chunk[..equals];
    let value: &[u8] = &chunk[equals + 1..];
    if is_a_credential(key) {
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
    } else {
        masked.extend_from_slice(key);
    }
    masked.push(b'=');
    if value.is_empty() {
        return;
    }
    if names_a_secret(key) || is_a_credential(value) {
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
        return;
    }
    masked.extend_from_slice(value);
}

/// What:
///  The punctuation that ends a URL written in a sentence,
/// and that is printed after the mask,
/// as in `(https://host/plain)` or `https://host/lfs,` .
/// Why:
///   A closing bracket or a comma cannot be told from a URL's own last byte,
/// and printing it as it stands costs nothing:
/// a segment that ends in it is shown whole or hidden whole,
/// so the most a reader learns from it is that punctuation was there.
///  A URL that really ends with such a byte keeps it,
/// with everything before it read as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const punctuation = text.match(/[,;)\]}.!?:]*$/)[0];
/// ```
fn trailing_punctuation(rest: &[u8]) -> &[u8] {
    let kept: usize = rest
        .iter()
        .rev()
        .take_while(ends_a_sentence)
        .count();
    return &rest[rest.len() - kept..];
}

/// Append the rest of a URL (its path,
/// query and fragment) with its secrets masked,
/// and the punctuation that follows it as it stands.
fn push_masked_rest(rest: &[u8], masked: &mut Vec<u8>) {
    let sentence_end: &[u8] = trailing_punctuation(rest);
    push_masked_url(&rest[..rest.len() - sentence_end.len()], masked);
    masked.extend_from_slice(sentence_end);
}

/// Append the path,
/// query and fragment of a URL with their secrets masked.
fn push_masked_url(rest: &[u8], masked: &mut Vec<u8>) {
    let path_end: usize = rest.iter().position(begins_the_rest).unwrap_or(rest.len());
    push_masked_path(&rest[..path_end], masked);
    let mut from: usize = path_end;
    if rest.get(from) == Some(&b'?') {
        masked.push(b'?');
        from += 1;
        let query_end: usize = match rest.get(from..).and_then(next_hash) {
            Some(offset) => from + offset,
            None => rest.len(),
        };
        push_masked_values(&rest[from..query_end], masked);
        from = query_end;
    }
    if rest.get(from) == Some(&b'#') {
        masked.push(b'#');
        push_masked_values(rest.get(from + 1..).unwrap_or_default(), masked);
    }
}

//endregion

//region The one pass

/// Where the text a mask is applied to ended.
/// Why:
///   A text cut at a size cap can end inside a URL,
/// and a URL cut before the `@` that ends its user information reads as a host,
/// so the last URL of a cut text is classified differently from the last URL of a whole one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextEnd {
    /// The text holds all of it: a URL that reaches its end ended there.
    Whole,
    /// The text is the head of a longer one, cut at a size cap: a URL that reaches its end may go on past it.
    Cut,
}

/// What:
///  The text with the secrets of every URL in it replaced by [`CREDENTIALS_MASK`]:
/// the user information,
/// and every path segment,
/// query key and query value,
/// fragment key and fragment value that a rule recognizes,
/// plus every credential after a `Bearer`,
/// `Basic` or `RemoteAuth` word.
/// Why:
///   One pass over the bytes:
/// each search starts where the previous URL ended,
/// so the work is linear however many URLs a line holds,
/// and each part of a URL is decoded and read a constant number of times.
/// Bytes outside the URLs are copied unchanged,
/// whatever their encoding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function maskUrlSecrets(text: Uint8Array): Uint8Array;
/// ```
pub fn mask_url_secrets(text: &[u8]) -> Vec<u8> {
    return mask_url_secrets_in(text, TextEnd::Whole);
}

/// What:
///  The same mask for the first bytes of a longer text,
/// cut at a size cap.
/// Why:
///   A text cut at a cap can end inside a URL,
/// and a URL cut before the `@` that ends its user information has no user information for the rule above:
/// `https://user:<token>` would read as a host and be relayed whole.
/// An authority that reaches the end of a cut text and holds no `@` is therefore hidden whole,
/// host included,
/// because no byte of it can be told from a user and a token.
///  A URL whose authority ended inside the text is read as in [`mask_url_secrets`]:
/// its user information is the text up to the last `@` of that authority,
/// which the cut kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function maskUrlSecretsOfCutText(text: Uint8Array): Uint8Array;
/// ```
pub fn mask_url_secrets_cut(text: &[u8]) -> Vec<u8> {
    return mask_url_secrets_in(text, TextEnd::Cut);
}

/// The one pass both masks are made of.
fn mask_url_secrets_in(text: &[u8], end: TextEnd) -> Vec<u8> {
    let mut masked: Vec<u8> = Vec::with_capacity(text.len());
    let mut copied: usize = 0;
    let mut search_from: usize = 0;
    while let Some(separator) = next_separator(text, search_from) {
        let authority_start: usize = separator + SCHEME_SEPARATOR.len();
        let authority_end: usize = authority_end_of(text, authority_start);
        // A separator that follows no scheme character starts no URL, and the scan goes on after what it ended.
        if separator == 0 || !ends_scheme(text[separator - 1]) {
            search_from = authority_end;
            continue;
        }
        // The user information runs to the last `@` of the authority, and needs at least one byte before it.
        let userinfo_end: Option<usize> = text[authority_start..authority_end]
            .iter()
            .rposition(is_at_sign)
            .filter(is_not_the_first);
        // A cut can end inside user information, before its `@`, so an authority that reaches the end of a cut text
        //  without one is hidden whole: it may be a user and a token, and its bytes cannot be told from a host's.
        if end == TextEnd::Cut && authority_end == text.len() && userinfo_end.is_none() {
            push_text_with_scheme_credentials(text, copied, authority_start, &mut masked);
            masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
            return masked;
        }
        push_text_with_scheme_credentials(text, copied, authority_start, &mut masked);
        match userinfo_end {
            Some(offset) => {
                masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
                // The copied bytes go on at the `@` that ended the user information, so the host follows whole.
                copied = authority_start + offset;
            }
            None => copied = authority_start,
        }
        // The host, its port, the path and the rest of the URL follow with their secrets masked.
        let url_end: usize = url_end_of(text, authority_end);
        masked.extend_from_slice(&text[copied..authority_end]);
        push_masked_rest(&text[authority_end..url_end], &mut masked);
        copied = url_end;
        search_from = url_end;
    }
    push_text_with_scheme_credentials(text, copied, text.len(), &mut masked);
    return masked;
}

/// What:
///  Append the text between two URLs with the credential of every `Bearer`,
/// `Basic` or `RemoteAuth` word masked.
/// Why:
///   These words carry a credential outside a URL as well as inside one:
/// a batch response's `header` object holds `"Authorization":"Bearer <token>"`,
/// and an error that echoes it would print the token whole.
/// Only the text between URLs is read here,
/// because a URL's own parts are read by the rules above.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pushTextWithSchemeCredentials(text, from, to, masked)
/// ```
fn push_text_with_scheme_credentials(text: &[u8], from: usize, to: usize, masked: &mut Vec<u8>) {
    let mut copied: usize = from;
    let mut search: usize = from;
    while search < to {
        let after_a_word: bool = search > 0 && follows_a_word_byte(text[search - 1]);
        let Some((start, end)) = scheme_credential_in(&text[search..to], after_a_word) else {
            break;
        };
        masked.extend_from_slice(&text[copied..search + start]);
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
        copied = search + end;
        search = copied;
    }
    masked.extend_from_slice(&text[copied..to]);
}

/// The same for text:
/// bytes that are not UTF-8 stay as they were and the result is read back lossily.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function maskUrlSecretsText(text: string): string;
/// ```
pub fn mask_url_secrets_text(text: &str) -> String {
    return String::from_utf8_lossy(mask_url_secrets(text.as_bytes()).as_slice()).into_owned();
}

//endregion

//region The host a destination names

/// What:
///  The part of a destination that names the server a person must recognize:
/// the host and its port,
/// with any user information left out.
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
    // The mask this module writes stands where the user information was, and its angle brackets end a URL's
    //  authority, so it is removed first: `https://lfs:<credentials>@host` names `host`, not `lfs:`.
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

/// What:
///  Every server a text names,
/// in the order it names them,
/// each as its host and port with any user information left out.
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
    // The mask this module writes stands where the user information was, and its angle brackets end a URL's
    //  authority, so it is removed first: `https://lfs:<credentials>@host` names `host`, not `lfs:`.
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

//endregion

//region A stream read in chunks

/// How long a word with no end is held before it is hidden whole.
/// Why:
///   A stream that never ends a word would otherwise grow this value without bound.
pub const LONGEST_HELD_WORD: usize = 64 * 1024;

/// What:
///  The mask of a stream, kept across the reads a relay makes of a child's output.
/// Why:
///   A relay reads a stream in chunks, and a chunk ends wherever the kernel had bytes:
/// a URL cut at a chunk's end can end before the `@` that ends its user information,
/// and a fragment read on its own would read as a host.
/// So the bytes after the last byte that ends a URL are held back,
/// everything before them is masked and handed over,
/// and the next chunk is masked with the held bytes in front of it.
///  A held word that passes [`LONGEST_HELD_WORD`] is dealt with at once:
/// one that holds no `://` cannot be a URL and is handed over as it stands,
/// and one that holds a scheme is hidden whole (`mask_url_secrets_cut`),
/// with the rest of that word dropped,
/// because a word whose middle was hidden may not be written unmasked afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UrlMaskingStream { feed(bytes: Uint8Array): Uint8Array; finish(): Uint8Array }
/// ```
#[derive(Debug, Default)]
pub struct UrlMaskingStream {
    /// The bytes of the word in progress, which may belong to a URL that goes on in the next chunk.
    held: Vec<u8>,
    /// Whether the bytes up to the next byte that ends a URL belong to a word already hidden whole and are dropped.
    swallowing: bool,
}

impl UrlMaskingStream {
    /// A stream that has read nothing.
    pub fn new() -> Self {
        return Self::default();
    }

    /// What:
    ///  Mask what this chunk lets be classified and return what may be written now.
    /// Why:
    ///   The held word is the only bytes this cannot classify yet,
    /// and it is what the next chunk starts with.
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut shown: Vec<u8> = Vec::new();
        let mut from: usize = 0;
        if self.swallowing {
            match bytes.iter().position(|byte| { return ends_url(*byte); }) {
                Some(offset) => {
                    self.swallowing = false;
                    shown.push(bytes[offset]);
                    from = offset + 1;
                }
                None => return shown,
            }
        }
        // The held word comes first, so a URL that began in an earlier chunk is classified with this one.
        let mut text: Vec<u8> = std::mem::take(&mut self.held);
        text.extend_from_slice(&bytes[from..]);
        match text.iter().rposition(|byte| { return ends_url(*byte); }) {
            Some(last) => {
                let (whole, tail): (&[u8], &[u8]) = text.split_at(last + 1);
                // The masked part is a text whose URLs all ended, so it is read as one.
                shown.extend_from_slice(mask_url_secrets(whole).as_slice());
                self.held = tail.to_vec();
            }
            None => self.held = text,
        }
        if self.held.len() > LONGEST_HELD_WORD {
            let long: Vec<u8> = std::mem::take(&mut self.held);
            if next_separator(long.as_slice(), 0).is_none() {
                // A word with no scheme carries no URL, so it is handed over as it stands.
                shown.extend_from_slice(long.as_slice());
            } else {
                let hidden: Vec<u8> = mask_url_secrets_cut(long.as_slice());
                // The rest of the word is dropped only when this mask hid something in it: a word that hid nothing may go
                //  on being read, and a word whose middle was hidden may not be written unmasked afterwards.
                self.swallowing = hidden.as_slice() != long.as_slice();
                shown.extend_from_slice(hidden.as_slice());
            }
        }
        return shown;
    }

    /// What:
    ///  The end of the stream:
    /// the held word is hidden the way the head of a cut text is,
    /// because the stream can end in the middle of a URL too.
    pub fn finish(mut self) -> Vec<u8> {
        if self.held.is_empty() {
            return Vec::new();
        }
        let held: Vec<u8> = std::mem::take(&mut self.held);
        return mask_url_secrets_cut(held.as_slice());
    }
}

//endregion

/// Masking is controlled in the test module.
#[cfg(test)]
#[path = "lfs_output_mask_tests.rs"]
mod tests;
