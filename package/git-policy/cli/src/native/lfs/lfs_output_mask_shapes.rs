//! What:
//!  What counts as a secret inside one part of a URL or one word of a line the wrapper relays:
//! the token formats the Git and LFS hosts issue,
//! the names a signed URL gives its signature,
//! the credential after a `Bearer`,
//! `Basic` or `RemoteAuth` word,
//! and a run of characters random enough and long enough to be a key.
//! Why:
//!  The owner decided on 2026-11-12T10:32:10-05:00 (item 1.9 of `doc/handover/cli-git-rust-implementation.md`)
//! that a URL is shown as it is and only what is recognised as a secret in it is masked,
//! with the cost stated in the question:
//! a secret of a shape this module does not know is printed.
//! So these rules are the whole protection,
//! and every one of them names where its shape comes from:
//!
//!  - The token formats of the Git hosts in use:
//! GitHub's own credential types reference fixes the prefixes of its six tokens
//! (`ghp_`,
//! `github_pat_`,
//! `gho_`,
//! `ghu_`,
//! `ghs_`,
//! `ghr_`;
//! <https://docs.github.com/en/enterprise-cloud@latest/organizations/managing-programmatic-access-to-your-organization/github-credential-types>),
//! GitLab's token overview fixes the prefixes of its fourteen
//! (`glpat-`,
//! `gloas-`,
//! `gldt-`,
//! `glrt-`,
//! `glrtr-`,
//! `glcbt-`,
//! `glptt-`,
//! `glft-`,
//! `glimt-`,
//! `glagent-`,
//! `glsoat-`,
//! and the runner registration prefix `GR1348941`;
//! <https://docs.gitlab.com/security/tokens/#token-prefixes>),
//! and a Gitea or Codeberg token has no prefix at all:
//! it is 40 hexadecimal characters
//! (`go-gitea/gitea` pull request 21959,
//! "Try to be in the same format of old output (`[0-9a-f]{40}`)"),
//! which the run rule below recognizes.
//!  Neither vendor's page gives the length of a token's body,
//! so the body lengths and alphabets here are those of the rule set this repository's own scanner ships
//! (`package/forbidden-strings/cli/data/builtin-rules.txt`),
//! and every one is a minimum,
//! so a token that grows keeps being recognized.
//!  - The signed URLs of the object stores a batch's upload action points at:
//! AWS Signature Version 4 puts `X-Amz-Signature`,
//! `X-Amz-Credential` and `X-Amz-Security-Token` in the query
//! (<https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html>),
//! Cloudflare R2 hands out the same parameters for its presigned URLs
//! (<https://developers.cloudflare.com/r2/api/s3/presigned-urls/>),
//! Google Cloud Storage uses `X-Goog-Signature` and `X-Goog-Credential`
//! (<https://cloud.google.com/storage/docs/access-control/signed-urls>),
//! and Azure's shared access signature is carried by `sig`
//! (<https://learn.microsoft.com/en-us/rest/api/storageservices/create-service-sas>).
//!  - `Bearer` and `Basic`,
//!    and git-lfs's own `RemoteAuth`
//! (<https://github.com/git-lfs/git-lfs/blob/main/docs/api/authentication.md>),
//! which a batch response's `header` object carries and an error can echo.
//!
//! What this module does not recognize is written beside the rules:
//! a short secret,
//! a secret made of ordinary words,
//! a secret in a host name,
//! and a credential of a shape no rule here names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! shapes.namesASecret(part) // true for `token`, `X-Amz-Credential`, `tokenvalue`, `sig`
//! shapes.isCredential(part) // true for `ghp_...`, 32 hex characters, a base64 body of 24 or more
//! ```

//region The alphabets a token's body is written in

/// The bytes of a hexadecimal number.
const LOWER_HEX: &[u8] = b"0123456789abcdef";


/// The bytes of a token body that is letters and digits.
const ALNUM: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// The bytes of a token body that is letters,
/// digits and underscores,
/// as GitHub's fine-grained token is.
const ALNUM_UNDERSCORE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_";

/// The bytes of a token body written in base64url without its padding,
/// which is what most hosts issue.
const ALNUM_URLSAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

//endregion

//region The token formats the Git hosts issue

/// What:
///  One token format a Git or LFS host issues:
/// the prefix its documentation fixes,
/// the bytes its body is written in and the shortest body that is still a token.
/// Why:
///   A prefix alone would hide the word `ghp_` in a sentence,
/// and a body length alone would hide an object's name;
/// together they are the shape of a token and of nothing a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IssuedToken = { prefix: string; alphabet: string; shortestBody: number }
/// ```
struct IssuedToken {
    /// The prefix the issuer's documentation fixes,
    /// as it is written there.
    prefix: &'static [u8],
    /// The bytes the body is written in.
    alphabet: &'static [u8],
    /// The shortest body that is still a token.
    shortest_body: usize,
}

/// What:
///  Every token format the Git hosts named in this package's own history and rules issue.
/// Why:
///   A token is recognized by the prefix its issuer documents,
/// never by the shape of its body alone:
/// `ghp_` in front of 36 letters and digits is a token,
/// while the same 36 bytes without the prefix are an object's name or a word.
/// The runner registration prefix carries no separator,
/// so it is listed as a prefix of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ISSUED_TOKENS: IssuedToken[] = [ { prefix: 'ghp_', alphabet: ALNUM, shortestBody: 36 }, ... ]
/// ```
const ISSUED_TOKENS: [IssuedToken; 18] = [
    // GitHub, from the credential types reference: the six prefixes it lists.
    IssuedToken { prefix: b"github_pat_", alphabet: ALNUM_UNDERSCORE, shortest_body: 82 },
    IssuedToken { prefix: b"ghp_", alphabet: ALNUM, shortest_body: 36 },
    IssuedToken { prefix: b"gho_", alphabet: ALNUM, shortest_body: 36 },
    IssuedToken { prefix: b"ghu_", alphabet: ALNUM, shortest_body: 36 },
    IssuedToken { prefix: b"ghs_", alphabet: ALNUM, shortest_body: 36 },
    IssuedToken { prefix: b"ghr_", alphabet: ALNUM, shortest_body: 36 },
    // GitLab, from the token overview's prefix table.
    IssuedToken { prefix: b"glpat-", alphabet: ALNUM_URLSAFE, shortest_body: 20 },
    IssuedToken { prefix: b"gloas-", alphabet: ALNUM_URLSAFE, shortest_body: 64 },
    IssuedToken { prefix: b"gldt-", alphabet: ALNUM_URLSAFE, shortest_body: 20 },
    IssuedToken { prefix: b"glrtr-", alphabet: ALNUM_URLSAFE, shortest_body: 20 },
    IssuedToken { prefix: b"glrt-", alphabet: ALNUM_URLSAFE, shortest_body: 20 },
    IssuedToken { prefix: b"glcbt-", alphabet: ALNUM_URLSAFE, shortest_body: 20 },
    IssuedToken { prefix: b"glptt-", alphabet: LOWER_HEX, shortest_body: 40 },
    IssuedToken { prefix: b"glft-", alphabet: ALNUM_URLSAFE, shortest_body: 20 },
    IssuedToken { prefix: b"glimt-", alphabet: ALNUM_URLSAFE, shortest_body: 25 },
    IssuedToken { prefix: b"glagent-", alphabet: ALNUM_URLSAFE, shortest_body: 50 },
    IssuedToken { prefix: b"glsoat-", alphabet: ALNUM_URLSAFE, shortest_body: 20 },
    IssuedToken { prefix: b"GR1348941", alphabet: ALNUM_URLSAFE, shortest_body: 20 },
];

//endregion

//region The names that say a value is a secret

/// What:
///  The stems that name a secret wherever they sit inside a parameter's name or a path segment.
/// Why:
///   A parameter's name is the only thing that says what a plain value means:
/// `X-Amz-Signature` holds a signature,
/// `X-Amz-Security-Token` a session token,
/// `access_token` a token,
/// and `sig` a signature,
/// so a value under one of these names is hidden though its bytes look like a word.
/// A path segment carries the same information:
/// the audit's `lfs.url` wrote its token as the segment `tokenvalue`,
/// and the stem `token` inside it is what recognizes it
/// (audit `alpha-12-lfs`,
/// "Lowercase path credentials survive the URL mask",
/// in `~/temp/agent/kimi-trials/kimi-audit-alpha-12-lfs/findings.md`);
/// no special case names that token.
/// Each stem is matched after a part is decoded and stripped to letters and digits,
/// so `api-key`,
/// `api_key` and `apiKey` are one stem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SECRET_NAME_PARTS = ['token', 'secret', 'password', ...]
/// ```
const SECRET_NAME_PARTS: [&[u8]; 15] = [
    b"token",
    b"secret",
    b"password",
    b"passwd",
    b"passphrase",
    b"credential",
    b"signature",
    b"apikey",
    b"accesskey",
    b"accessid",
    b"privatekey",
    b"bearer",
    b"authorization",
    b"sastoken",
    b"jwt",
];

/// What:
///  The names that are a secret name whole,
/// and are too short or too common to be looked for inside another name.
/// Why:
///   `sig`,
///   `key`,
///   `auth`,
///   `sas`,
///   `jwt` and `session` each name a credential when they are the whole name,
/// as in `?sig=`,
/// `?key=` and `?access_token=`;
/// looked for inside a longer name they would hide `design`,
/// `assign` and `author` too,
/// so they are matched whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SECRET_NAME_WORDS = ['sig', 'key', 'auth', 'sas', 'jwt', 'session']
/// ```
const SECRET_NAME_WORDS: [&[u8]; 6] = [b"sig", b"key", b"auth", b"sas", b"jwt", b"session"];

//endregion

//region A run of characters long enough to be a key

/// What:
///  The shortest run of hexadecimal characters this module calls a secret:
/// 32 characters,
/// which is 128 bits and the length of an MD5 hash.
/// Why:
///   Shorter hexadecimal runs are the ordinary shape of identifiers a person reads:
/// a Git abbreviation of 7 to 12 characters,
/// a short object name,
/// a date.
/// A secret can be written in this alphabet,
/// a Gitea token is 40 lowercase hexadecimal characters,
/// and a key generated as hexadecimal is at least this long,
/// so the run is hidden wherever it sits,
/// both cases counted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHORTEST_HEX_RUN = 32
/// ```
const SHORTEST_HEX_RUN: usize = 32;

/// What:
///  The shortest run of letters and digits that holds a digit and is not hexadecimal:
/// 32 characters.
/// Why:
///   A token written in this alphabet,
/// as a Gitea token and many shared keys are,
/// carries no other mark;
/// 32 letters and digits with a digit among them is 160 bits of base32,
/// while a word of that length in a URL is written with hyphens or in capitals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHORTEST_ALNUM_RUN = 32
/// ```
const SHORTEST_ALNUM_RUN: usize = 32;

/// What:
///  The shortest run of base64 or base64url characters this module calls a secret:
/// 24 characters,
/// if it holds a lowercase letter,
/// an uppercase letter and a digit.
/// Why:
///  24 characters of this alphabet are 144 bits,
///  above the 128 bits below which a key is not worth calling a key,
/// and the three classes are what keeps the rule off everything else:
/// a path segment of a release name is lowercase and hyphenated,
/// and a camelCase identifier of a path holds no digit.
/// A base64 secret shorter than 24 characters is not recognized here,
/// and that is a limit of this rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHORTEST_BASE64_RUN = 24
/// ```
const SHORTEST_BASE64_RUN: usize = 24;

/// What:
///  The shortest credential after a `Bearer`,
/// `Basic` or `RemoteAuth` word:
/// 16 characters.
/// Why:
///  A base64 credential below this length is more often a word of the sentence:
/// `authentication` is 14 characters and `authorization` 13,
/// while the base64 of `user:password` is 20.
/// A shorter credential after one of these words is not recognized,
/// and that is a limit of this rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHORTEST_SCHEME_CREDENTIAL = 16
/// ```
const SHORTEST_SCHEME_CREDENTIAL: usize = 16;

/// The words a credential follows,
/// lowercased.
/// `RemoteAuth` is git-lfs's own scheme (`docs/api/authentication.md`).
const SCHEME_WORDS: [&[u8]; 3] = [b"bearer", b"basic", b"remoteauth"];

//endregion

//region Reading one part

/// What:
///  Whether one part of a URL or one word names or holds a secret.
/// Why:
///   A part's own name is read first,
/// and its shape second,
/// so `?token=` hides its value whatever the value looks like,
/// and `?ref=` hides a value that is a token of a recognized shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdsASecret(part: string): boolean { return namesASecret(part) || isCredential(part) }
/// ```
pub fn holds_a_secret(part: &[u8]) -> bool {
    return names_a_secret(part) || is_a_credential(part);
}

/// What:
///  Whether a part's own bytes name a secret:
/// the mask,
/// the name or the segment of a token.
/// Why:
///   A name says what its value is,
/// and a path segment can say the same thing,
/// so a short lowercase token written as a name is recognized without any shape of a key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesASecret(part: string): boolean
/// ```
pub fn names_a_secret(part: &[u8]) -> bool {
    if named_secret(part) {
        return true;
    }
    let decoded: Vec<u8> = percent_decoded(part);
    return decoded.as_slice() != part && named_secret(decoded.as_slice());
}

/// Whether the letters and digits of a part are a secret name.
/// `part` is read as it is,
/// then once percent-decoded,
/// by the caller.
fn named_secret(part: &[u8]) -> bool {
    let name: Vec<u8> = letters_and_digits(part);
    if name.is_empty() {
        return false;
    }
    for word in SECRET_NAME_WORDS {
        if name.as_slice() == word {
            return true;
        }
    }
    for stem in SECRET_NAME_PARTS {
        if holds(name.as_slice(), stem) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a part's bytes are a credential of a recognized shape,
/// whatever it is called.
/// Why:
///   A token can be written under a name that says nothing (`?ref=ghp_...`),
/// in a path segment that says nothing,
/// or as the value of a signed URL's parameter the name rule would hide anyway;
/// the shape is what recognizes it there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCredential(part: string): boolean
/// ```
pub fn is_a_credential(part: &[u8]) -> bool {
    if shaped_credential(part) {
        return true;
    }
    let decoded: Vec<u8> = percent_decoded(part);
    return decoded.as_slice() != part && shaped_credential(decoded.as_slice());
}

/// Whether the bytes of a part are a credential of a recognized shape without decoding.
fn shaped_credential(part: &[u8]) -> bool {
    if holds_an_issued_token(part) {
        return true;
    }
    if holds_a_random_run(part) {
        return true;
    }
    return scheme_credential_in(part, false).is_some();
}

/// What:
///  Whether a part holds a token of a format one of the Git hosts issues.
/// Why:
///   The prefix is what makes a token a token:
/// the issuer documents it and it cannot be confused with a person's word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdsAnIssuedToken(part: string): boolean
/// ```
pub fn holds_an_issued_token(part: &[u8]) -> bool {
    for token in ISSUED_TOKENS {
        let mut from: usize = 0;
        while let Some(offset) = position_of(&part[from..], token.prefix) {
            let body_start: usize = from + offset + token.prefix.len();
            let body_end: usize = body_start.saturating_add(token.shortest_body);
            if let Some(body) = part.get(body_start..body_end)
                && written_in(body, token.alphabet)
            {
                return true;
            }
            from += offset + 1;
        }
    }
    return false;
}

/// What:
///  Whether a part holds a run of characters random enough and long enough to be a key,
/// in any of the three alphabets a key is written in.
/// Why:
///   Three lengths serve three alphabets:
/// a hexadecimal run of 32 characters,
/// a run of 32 letters and digits holding a digit,
/// and a run of 24 base64 or base64url characters holding a lowercase letter,
/// an uppercase letter and a digit.
/// Each run is read once,
/// so a line of any length is read a constant number of times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdsARandomRun(part: string): boolean
/// ```
pub fn holds_a_random_run(part: &[u8]) -> bool {
    let mut from: usize = 0;
    while from < part.len() {
        if !part[from].is_ascii_alphanumeric() && !matches!(part[from], b'+' | b'/' | b'_' | b'-') {
            from += 1;
            continue;
        }
        let mut to: usize = from;
        let mut lower: bool = false;
        let mut upper: bool = false;
        let mut digit: bool = false;
        let mut hexadecimal: bool = true;
        let mut letters_and_digits: bool = true;
        while to < part.len() {
            let byte: u8 = part[to];
            if !byte.is_ascii_alphanumeric() && !matches!(byte, b'+' | b'/' | b'_' | b'-') {
                break;
            }
            lower = lower || byte.is_ascii_lowercase();
            upper = upper || byte.is_ascii_uppercase();
            digit = digit || byte.is_ascii_digit();
            hexadecimal = hexadecimal && LOWER_HEX.contains(&byte.to_ascii_lowercase());
            letters_and_digits = letters_and_digits && byte.is_ascii_alphanumeric();
            to += 1;
        }
        let run: usize = to - from;
        if hexadecimal && run >= SHORTEST_HEX_RUN {
            return true;
        }
        if letters_and_digits && digit && run >= SHORTEST_ALNUM_RUN {
            return true;
        }
        if lower && upper && digit && run >= SHORTEST_BASE64_RUN {
            return true;
        }
        from = to;
    }
    return false;
}

/// What:
///  Where the credential of a `Bearer`,
/// `Basic` or `RemoteAuth` scheme starts and ends,
/// when a part holds one.
/// Why:
///   These words name the credential that follows them
/// (`docs/api/authentication.md` of git-lfs,
/// and Git's own `Authorization: Basic` and `Authorization: Bearer`),
/// and a batch response's `header` object carries one,
/// so a token is recognized after the word that says what it is even when its bytes look like a word.
/// The word before must be a word of its own,
/// so `Cyberbasic` is no scheme
/// (`after_a_word` says that the byte before `part` is part of a word),
/// and the credential must be at least [`SHORTEST_SCHEME_CREDENTIAL`] bytes of base64,
/// so `Basic realm=...` and the word `authentication` are no credential.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function schemeCredentialIn(part: string, afterAWord: boolean): [start, end] | undefined
/// ```
pub fn scheme_credential_in(part: &[u8], after_a_word: bool) -> Option<(usize, usize)> {
    for word in SCHEME_WORDS {
        let mut from: usize = 0;
        while let Some(offset) = position_of_ignoring_case(&part[from..], word) {
            let word_start: usize = from + offset;
            let word_end: usize = word_start + word.len();
            let before_ends_a_word: bool = if word_start == 0 { !after_a_word } else { !follows_a_word_byte(part[word_start - 1]) };
            let spaced: bool = part.get(word_end).is_some_and(is_space_or_tab);
            if before_ends_a_word && spaced
                && let Some(credential) = credential_at(part, word_end + 1)
            {
                return Some(credential);
            }
            from = word_start + 1;
        }
    }
    return None;
}

/// The start and end of the credential word that begins at `from`,
/// skipping the spaces and tabs in front of it,
/// when it is long enough and written in base64.
fn credential_at(part: &[u8], from: usize) -> Option<(usize, usize)> {
    let mut start: usize = from;
    while part.get(start).is_some_and(is_space_or_tab) {
        start += 1;
    }
    let mut end: usize = start;
    while part.get(end).is_some_and(is_a_base64_or_padding_byte) {
        end += 1;
    }
    if end - start < SHORTEST_SCHEME_CREDENTIAL {
        return None;
    }
    return Some((start, end));
}

/// Whether a byte is the space or the tab that separates a scheme word from its credential.
fn is_space_or_tab(byte: &u8) -> bool {
    return *byte == b' ' || *byte == b'\t';
}

/// Whether a byte is written in base64 or base64url,
/// or is the padding a credential may end with.
fn is_a_base64_or_padding_byte(byte: &u8) -> bool {
    return is_a_base64_byte(*byte) || *byte == b'=';
}

/// Whether a byte is part of a word:
/// a letter,
/// a digit or an underscore.
pub fn follows_a_word_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// Whether a byte is written in the base64 or base64url alphabet.
fn is_a_base64_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'-' | b'_');
}

//endregion

//region Reading the bytes of a part

/// What:
///  The bytes `%XX` writes,
/// decoded once.
/// Why:
///   A secret written with percent encoding is the same secret,
/// and a segment of `%2F`-joined parts is read as the parts it holds;
/// a `%` that does not start two hexadecimal digits stays as it is,
/// so a part that is not encoded is returned unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function percentDecoded(part: string): string
/// ```
pub fn percent_decoded(part: &[u8]) -> Vec<u8> {
    let mut decoded: Vec<u8> = Vec::with_capacity(part.len());
    let mut from: usize = 0;
    while from < part.len() {
        let byte: u8 = part[from];
        if byte == b'%'
            && let (Some(high), Some(low)) = (part.get(from + 1).copied().and_then(hexadecimal_value), part.get(from + 2).copied().and_then(hexadecimal_value))
        {
            decoded.push(high * 16 + low);
            from += 3;
            continue;
        }
        decoded.push(byte);
        from += 1;
    }
    return decoded;
}

/// The value of one hexadecimal digit,
/// or nothing when the byte is none.
fn hexadecimal_value(byte: u8) -> Option<u8> {
    return match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    };
}

/// What:
///  The letters and digits of a part,
/// lowercased.
/// Why:
///   Stripping the separators makes `api-key`,
/// `api_key` and `apiKey` one name,
/// and case never decides whether a name is a secret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lettersAndDigits(part: string): string
/// ```
pub fn letters_and_digits(part: &[u8]) -> Vec<u8> {
    let mut name: Vec<u8> = Vec::with_capacity(part.len());
    for byte in part {
        if byte.is_ascii_alphanumeric() {
            name.push(byte.to_ascii_lowercase());
        }
    }
    return name;
}

/// Whether every byte of `body` is written in `alphabet`.
fn written_in(body: &[u8], alphabet: &[u8]) -> bool {
    for byte in body {
        if !alphabet.contains(byte) {
            return false;
        }
    }
    return true;
}

/// Where `pattern` first sits in `bytes`,
/// or nothing when it does not.
fn position_of(bytes: &[u8], pattern: &[u8]) -> Option<usize> {
    if pattern.is_empty() || bytes.len() < pattern.len() {
        return None;
    }
    for from in 0..=bytes.len() - pattern.len() {
        if &bytes[from..from + pattern.len()] == pattern {
            return Some(from);
        }
    }
    return None;
}

/// Where `pattern` first sits in `bytes`,
/// whatever the case of either,
/// or nothing when it does not.
fn position_of_ignoring_case(bytes: &[u8], pattern: &[u8]) -> Option<usize> {
    if pattern.is_empty() || bytes.len() < pattern.len() {
        return None;
    }
    for from in 0..=bytes.len() - pattern.len() {
        if bytes[from..from + pattern.len()].eq_ignore_ascii_case(pattern) {
            return Some(from);
        }
    }
    return None;
}

/// Whether `bytes` holds `pattern` at all.
fn holds(bytes: &[u8], pattern: &[u8]) -> bool {
    return position_of(bytes, pattern).is_some();
}

//endregion

/// The shapes are controlled in the test module.
#[cfg(test)]
#[path = "lfs_output_mask_shapes_tests.rs"]
mod tests;
