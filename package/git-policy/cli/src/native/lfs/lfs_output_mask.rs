//! What:
//!  Hide every secret-shaped part of the URLs in the text the wrapper relays from `git-lfs` and from Git:
//! the userinfo,
//! a path segment that is not a plain label,
//! and every value of the query and of the fragment.
//! Why:
//!  An `lfs.url` can carry its token in three places,
//! and `git-lfs` names the URL it used in its errors:
//! `https://user:token@host/...` carries one in its userinfo,
//! `https://host/<token>/lfs` one in a path segment,
//! and `https://host/lfs?token=<token>` one in its query.
//! The wrapper relays that text to the caller's terminal and,
//! for the auto-push,
//! records
//!  it in files,
//! so every one of those places is replaced at the one place where the text leaves the wrapper
//! (owner,
//! "Never print or commit secrets").
//!  The authority of a URL ends at the first `/`,
//! `?`,
//! `#`,
//! `\`,
//! quote,
//! angle bracket,
//! backtick or white space,
//! and its
//!  userinfo ends at the last `@` before that,
//! so an `@` in a path,
//! a query or a bracketed IPv6 host is left alone.
//! The rest of the URL ends at the first `\`,
//! quote,
//! angle bracket,
//! backtick,
//! white space or control character.
//!  A path segment is kept only while it is a plain label:
//! the letters `a` to `z` and the separators `-`,
//! `.`,
//! `_` and `~` alone,
//! which is what `lfs`,
//! `objects`,
//! `batch`,
//! `info` and `repo.git` are,
//! and none of its unbroken runs of lowercase letters is as long as
//!  [`LONGEST_PLAIN_WORD`],
//! because a token written in lowercase letters alone reads as a label otherwise.
//! A query key and a fragment key are kept while they are a plain label.
//! A token carries a capital,
//! a digit,
//! a percent encoding or another punctuation mark,
//! and a value is a secret wherever it sits,
//! so it is always hidden.
//!  What this cannot do:
//! a token made of fewer than [`LONGEST_PLAIN_WORD`] lowercase letters alone still reads as a label,
//! which the owner accepted when choosing this rule on 2026-11-19;
//! and a query key of that shape is not hidden either,
//! because the rule is about path words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const masked = text.replace(/([a-z][a-z0-9+.-]*:\/\/)[^\/?#\\\s"'<>`]*@/gi, '$1<credentials>@'); // then the path, query and fragment
//! ```

/// What replaces the userinfo of a URL and every other secret-shaped part of it.
pub const CREDENTIALS_MASK: &str = "<credentials>";

/// What:
///  The longest run of lowercase letters a path segment may hold and still be printed as it stands:
///  a longer unbroken word is a token and is replaced by [`CREDENTIALS_MASK`].
/// Why:
///  A token written in lowercase letters alone reads as a plain label under the rule above,
///  and hiding every path segment after the host refused the names a person needs (`objects`,
/// `info`,
///  a repository's own name).
///  The owner's answer of 2026-11-19T16:58:00-05:00 ("Hide long words only",
///  `doc/handover/questions-for-the-owner.md`) chose the length as the rule:
///  twenty or more unbroken lowercase letters read as a token,
///  and the cost the owner accepted is that a shorter lowercase token still prints.
pub const LONGEST_PLAIN_WORD: usize = 20;

/// Whether the mask this module writes starts at `offset` of `text`.
/// Why:
///   Masking runs again on text that was already masked,
///  because the attempt the auto-push records is masked where it is made and
///  once more where it is written down;
///  and the mask holds the `>` and `<` this module reads as the end of a URL,
///  so a second pass that did not know it over its own mask would cut the URL short and
///  print the mask again beside it (`?token=<credentials><credentials>`) or drop a host.
fn mask_starts_at(text: &[u8], offset: usize) -> bool {
    return text
        .get(offset..)
        .is_some_and(|rest| { return rest.starts_with(CREDENTIALS_MASK.as_bytes()); });
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
const SCHEME_SEPARATOR: &[u8] = b"://";

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

/// Whether a byte is the `@` that ends the userinfo of a URL.
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

/// Whether a byte is one a plain label is made of.
fn is_label_byte(byte: &u8) -> bool {
    return byte.is_ascii_lowercase() || matches!(byte, b'-' | b'.' | b'_' | b'~');
}

/// Whether an offset into an authority is a byte for the userinfo,
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
/// or the end of the text;
/// a mask already written stands inside the authority as it stands.
fn authority_end_of(text: &[u8], from: usize) -> usize {
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
fn url_end_of(text: &[u8], from: usize) -> usize {
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
///  Whether a path segment,
/// a query key or a fragment key is a plain label,
/// which carries no token.
/// Why:
///   The rule is one sentence a person can check against a URL they wrote:
/// a label is lowercase letters and the separators of a filename,
/// and anything else,
/// a capital,
/// a digit,
/// a percent encoding,
/// a slash written encoded,
/// is what a token looks like.
/// An empty label is plain:
/// `//` and a trailing `/` carry nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^[a-z._~-]*$/.test(label)
/// ```
fn is_plain_label(label: &[u8]) -> bool {
    return label.iter().all(is_label_byte);
}

/// What:
///  Whether a label holds an unbroken run of lowercase letters as long as [`LONGEST_PLAIN_WORD`],
///  which is what a token written in lowercase letters alone looks like.
/// Why:
///   The rule a person can check against a URL they wrote is one number:
///  a word of that many letters is a token,
///  a shorter one is a name (`objects`,
/// `info`,
/// a repository called `notes`);
///  every byte that is not a lowercase letter breaks a word,
///  so `repository-name.git` holds no long word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /[a-z]{20}/.test(label)
/// ```
fn has_a_long_word(label: &[u8]) -> bool {
    let mut run: usize = 0;
    for byte in label {
        if byte.is_ascii_lowercase() {
            run += 1;
            if run >= LONGEST_PLAIN_WORD {
                return true;
            }
        } else {
            run = 0;
        }
    }
    return false;
}

/// Append one label as it stands when it is plain,
/// and the mask when it is not.
fn push_label(label: &[u8], masked: &mut Vec<u8>) {
    if is_plain_label(label) {
        masked.extend_from_slice(label);
    } else {
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
    }
}

/// Append one path segment as it stands while it is a plain label of short words,
/// and the mask when it is not.
fn push_path_label(label: &[u8], masked: &mut Vec<u8>) {
    if is_plain_label(label) && !has_a_long_word(label) {
        masked.extend_from_slice(label);
    } else {
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
    }
}

/// Append the path of a URL with every segment that is not a plain label of short words masked.
fn push_masked_path(path: &[u8], masked: &mut Vec<u8>) {
    let mut from: usize = 0;
    loop {
        match path.get(from..).and_then(next_slash) {
            Some(offset) => {
                push_path_label(&path[from..from + offset], masked);
                masked.push(b'/');
                from += offset + 1;
            }
            None => {
                push_path_label(path.get(from..).unwrap_or_default(), masked);
                return;
            }
        }
    }
}

/// What:
///  Append a query or fragment with every value masked and every plain key kept.
/// Why:
///   A value is a secret wherever it sits:
/// `?token=`,
/// `?sig=` and a bare token with no `=` are all hidden,
/// and a key that is not a plain label (a token used as a name,
/// as the incident's config key was) is hidden with it.
///  The separators and the key stay,
/// so a person still reads which setting the server was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// values.split('&').map((chunk) => { const [key, value] = chunk.split('='); return value === undefined ? mask : `${plain(key) ? key : mask}=${mask}` }).join('&')
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
        match chunk.iter().position(is_equals_sign) {
            Some(equals) => {
                push_label(&chunk[..equals], masked);
                masked.push(b'=');
                masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
            }
            None if !chunk.is_empty() => masked.extend_from_slice(CREDENTIALS_MASK.as_bytes()),
            None => {}
        }
        match separator {
            Some(offset) => {
                masked.push(b'&');
                from = offset + 1;
            }
            None => return,
        }
    }
}

/// What:
///  The punctuation that ends a URL written in a sentence,
/// and that is printed after the mask,
/// as in `(https://host/plain)` or `https://host/lfs,` .
/// Why:
///   A closing bracket or a comma cannot be told from a token's last byte,
/// and printing it as it stands costs nothing:
/// the segment it follows is hidden whole,
/// so the most a reader learns from it is that punctuation was there.
///  A URL that really ends with such a byte keeps it,
/// with everything before it masked as before.
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

/// What:
///  The text with the secrets of every URL in it replaced by [`CREDENTIALS_MASK`]:
/// the userinfo,
/// every path segment that is not a plain label,
/// and every query and fragment value.
/// Why:
///   One pass over the bytes:
/// each search starts where the previous URL ended,
/// so the work is linear however many
///  URLs a line holds.
/// Bytes outside the URLs are copied unchanged,
/// whatever their encoding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function maskUrlSecrets(text: Uint8Array): Uint8Array;
/// ```
pub fn mask_url_secrets(text: &[u8]) -> Vec<u8> {
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
        // The userinfo runs to the last `@` of the authority, and needs at least one byte before it.
        let userinfo_end: Option<usize> = text[authority_start..authority_end]
            .iter()
            .rposition(is_at_sign)
            .filter(is_not_the_first);
        if let Some(offset) = userinfo_end {
            masked.extend_from_slice(&text[copied..authority_start]);
            masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
            copied = authority_start + offset;
        }
        // The host and its port stay, and the path, query and fragment follow with their secrets masked.
        let url_end: usize = url_end_of(text, authority_end);
        masked.extend_from_slice(&text[copied..authority_end]);
        push_masked_rest(&text[authority_end..url_end], &mut masked);
        copied = url_end;
        search_from = url_end;
    }
    masked.extend_from_slice(&text[copied..]);
    return masked;
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

/// Whether a byte ends a line.
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  The masked head of a run's output,
///  and whether a cap kept only part of it.
/// Why:
///   The two are one answer:
///  a caller that relays the text must say when it is not all of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MaskedHead = { text: Uint8Array; cut: boolean };
/// ```
pub struct MaskedHead {
    /// The masked bytes of the whole lines the cap holds.
    pub text: Vec<u8>,
    /// Whether the output was longer than the cap,
    /// so `text` is its head and not all of it.
    pub cut: bool,
}

/// What:
///  The masked head of `text` that fits in `cap` bytes,
/// and whether there was more.
/// Why:
///   A cap applied to the raw bytes cuts a URL in half,
///  and half a URL hides no secret:
///  a cut before the `@` of a userinfo keeps the front of a password,
///  and the mask that runs afterwards cannot see a userinfo that is not there.
///  So the cut falls on a line break and only whole lines are kept,
///  which the mask reads as it reads any line,
///  and the last line is dropped whole when it does not fit.
///  What is dropped is said by the caller,
///  so the person reading the output knows it is the head of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function maskUrlSecretsWithin(text: Uint8Array, cap: number): MaskedHead;
/// ```
pub fn mask_url_secrets_within(text: &[u8], cap: u64) -> MaskedHead {
    let limit: usize = usize::try_from(cap).unwrap_or(usize::MAX);
    if text.len() <= limit {
        return MaskedHead { text: mask_url_secrets(text), cut: false };
    }
    let kept: usize = text
        .get(..limit)
        .and_then(|head| { return head.iter().rposition(is_line_feed); })
        .map_or(0, |at| { return at + 1; });
    return MaskedHead { text: mask_url_secrets(text.get(..kept).unwrap_or_default()), cut: true };
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

/// Masking is controlled in the test module.
#[cfg(test)]
#[path = "lfs_output_mask_tests.rs"]
mod tests;
