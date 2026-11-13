//! What:
//!  What the wrapper shows of a URL in the text it relays from `git-lfs` and from Git:
//! the scheme,
//! the host and its port,
//! the path,
//! the query and the fragment as they are,
//! with the user information and every part a rule recognizes as a secret replaced by
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
//!
//!  What is not recognized,
//! and is therefore printed,
//! is the limit of that decision:
//!
//!  - a secret shorter than the run rules,
//! such as a six-character access code in a path;
//!  - a secret made of ordinary words that names nothing,
//! such as `?ref=bluebird`,
//! because no rule can tell it from a branch name;
//!  - a secret in a host name,
//! such as `https://<token>.example.invalid/`,
//! because the host is the one part of a URL a person must read,
//! and no issuer we know puts a secret there;
//!  - a credential of a shape no rule here names,
//! such as a new token format of a host this package has not read the documentation of;
//!  - the first bytes of a token when an output is cut inside it,
//! shorter than the rule that would recognize it.
//!
//!  The user information is hidden always:
//! a URL that carries a user and a password there carries them as a secret,
//! and nothing in its bytes can tell a user's name from a token.
//! The rules that recognize a secret in a part of a URL,
//! each with the documentation its shape comes from,
//! are in [`shapes`].
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

/// What the rules that recognize a secret in one part of a URL or one word of a line are made of.
#[path = "lfs_output_mask_shapes.rs"]
mod shapes;

/// Where a URL,
/// its authority and the rest of it end.
#[path = "lfs_output_mask_reading_a_url.rs"]
mod lfs_output_mask_reading_a_url;

/// Each part of a URL as the wrapper shows it.
#[path = "lfs_output_mask_parts_of_a_url.rs"]
mod lfs_output_mask_parts_of_a_url;

/// The server a destination or a line names.
#[path = "lfs_output_mask_destinations.rs"]
mod lfs_output_mask_destinations;

/// The mask of a stream,
/// kept across the reads a relay makes.
#[path = "lfs_output_mask_stream.rs"]
mod lfs_output_mask_stream;

pub use self::lfs_output_mask_destinations::{host_of, hosts_in};
pub use self::lfs_output_mask_stream::{LONGEST_HELD_WORD, UrlMaskingStream};

use self::lfs_output_mask_parts_of_a_url::push_masked_rest;
use self::lfs_output_mask_reading_a_url::{authority_end_of, ends_scheme, is_at_sign, is_not_the_first, next_separator, url_end_of};
use self::shapes::{follows_a_word_byte, scheme_credential_in};

/// What replaces the user information of a URL and every part of it that is recognized as a secret.
pub const CREDENTIALS_MASK: &str = "<credentials>";

/// Where the text a mask is applied to ended.
/// Why:
///   A text cut at a size cap can end inside a URL,
/// and a URL cut before the `@` that ends its user information reads as a host,
/// so the last URL of a cut text is classified differently from the last URL of a whole one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextEnd {
    /// The text holds all of it:
    /// a URL that reaches its end ended there.
    Whole,
    /// The text is the head of a longer one,
    /// cut at a size cap:
    /// a URL that reaches its end may go on past it.
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
        let authority_start: usize = separator + self::lfs_output_mask_reading_a_url::SCHEME_SEPARATOR.len();
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

/// Masking is controlled in the test module.
#[cfg(test)]
#[path = "lfs_output_mask_tests.rs"]
mod tests;
