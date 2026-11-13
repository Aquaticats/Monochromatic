//! Controls for the rules that recognize a secret in one part of a URL or one word of a line:
//! one case for every token format the Git hosts issue,
//! one for every name that says a value is a secret,
//! one for each of the three runs of random characters,
//! and the near misses that must stay readable.

use super::*;

/// What:
///  One token of every format the Git hosts issue is recognized,
/// and the same bytes without their prefix are not.
/// Why:
///   A token is recognized by the prefix its issuer documents,
/// so a body of the right shape with no prefix is an object's name,
/// not a token.
#[test]
fn every_issued_token_is_recognized_by_its_prefix() {
    let github: [&str; 6] = ["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_"];
    for prefix in github {
        let token: String = format!("{prefix}{}", "A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0U1v2W3x4Y5z6".repeat(2));
        assert!(holds_an_issued_token(token.as_bytes()), "{token}");
        assert!(!holds_an_issued_token(token.get(prefix.len()..).expect("a body").as_bytes()), "{prefix} with no prefix");
    }
    let gitlab: [&str; 11] =
        ["glpat-", "gloas-", "gldt-", "glrt-", "glrtr-", "glcbt-", "glft-", "glimt-", "glagent-", "glsoat-", "GR1348941"];
    for prefix in gitlab {
        let token: String = format!("{prefix}{}", "abcDEF123_-".repeat(6));
        assert!(holds_an_issued_token(token.as_bytes()), "{token}");
    }
    // A pipeline trigger token is hexadecimal, so the same body in another alphabet is no token.
    assert!(holds_an_issued_token(format!("glptt-{}", "0123456789abcdef".repeat(3)).as_bytes()), "a trigger token");
    assert!(!holds_an_issued_token(format!("glptt-{}", "abcDEF123_-".repeat(6)).as_bytes()), "a trigger token is hexadecimal");
}

/// What:
///  A prefix with too short a body,
/// or a body holding a byte its issuer does not use,
/// is not a token.
/// Why:
///   The body length and alphabet are what keep the rule off the word `ghp_` in a sentence and off a
///  repository named `github_pat_notes`.
#[test]
fn a_prefix_with_the_wrong_body_is_not_a_token() {
    assert!(!holds_an_issued_token(format!("ghp_{}", "a".repeat(35)).as_bytes()), "one byte short");
    assert!(holds_an_issued_token(format!("ghp_{}", "a".repeat(36)).as_bytes()), "the documented length");
    assert!(holds_an_issued_token(format!("ghp_{}", "a".repeat(80)).as_bytes()), "a token that grew keeps its prefix");
    assert!(!holds_an_issued_token(format!("glptt-{}", "g".repeat(40)).as_bytes()), "a GitLab trigger token is hexadecimal");
    assert!(!holds_an_issued_token(format!("ghp_{}", "a-".repeat(36)).as_bytes()), "the body holds no hyphen");
    assert!(!holds_an_issued_token(b"github_pat_notes about tokens"), "a word that begins like a prefix is not one");
}

/// What:
///  Every name this module knows says its value is a secret,
/// in any case,
/// with any of the separators a name is written with,
/// and once percent-decoded.
/// Why:
///   A signed URL's parameters are the names the object stores document,
/// and a person writes the same names with `-`,
/// `_` or capitals.
#[test]
fn every_secret_name_is_recognized_however_it_is_written() {
    let names: [&str; 24] = [
        "X-Amz-Signature",
        "X-Amz-Credential",
        "X-Amz-Security-Token",
        "x-goog-signature",
        "SIG",
        "sig",
        "Signature",
        "token",
        "TOKEN",
        "access_token",
        "accessToken",
        "key",
        "api_key",
        "api-key",
        "apikey",
        "AWSAccessKeyId",
        "GoogleAccessId",
        "client_secret",
        "password",
        "passwd",
        "credential",
        "auth",
        "bearer",
        "tokenvalue",
    ];
    for name in names {
        assert!(names_a_secret(name.as_bytes()), "{name}");
    }
    // The audit's canary is caught here and by no special case of its own: the stem `token` inside the segment's own
    //  bytes is what recognizes it (audit `alpha-12-lfs`, "Lowercase path credentials survive the URL mask").
    assert!(names_a_secret(b"tokenvalue"), "the audit's lowercase path secret is a secret name");
    assert!(names_a_secret(b"X-Amz-Signature%3D"), "an encoded name is read as it decodes");
    assert!(names_a_secret(b"to%6Ben"), "a half-encoded name is read as it decodes");
}

/// What:
///  A name that says nothing about a secret is not one,
/// and the words this module matches only whole are not looked for inside another word.
/// Why:
///   A path is shown so that a person reads which repository the push went to,
/// and `objects`,
/// `ref`,
/// `batch` and `info` are what an LFS URL says.
#[test]
fn a_name_that_says_nothing_about_a_secret_is_kept() {
    let names: [&str; 15] =
        ["objects", "batch", "info", "lfs", "ref", "main", "owner", "repo.git", "design", "assign", "author", "authentication", "monochromatic-lfs", "X-Amz-Date", "X-Amz-SignedHeaders"];
    for name in names {
        assert!(!names_a_secret(name.as_bytes()), "{name}");
    }
}

/// What:
///  A run of random characters is recognized in each of the three alphabets,
/// and a run that is one byte short,
/// or that is what an identifier looks like,
/// is not.
/// Why:
///   The lengths are what make the rule useful:
/// a Git abbreviation and a release name are ordinary,
/// and a key of 128 bits is not.
#[test]
fn a_run_of_random_characters_is_recognized_at_its_length() {
    assert!(holds_a_random_run("8f14e45fceea167a5a36dedd4bea2543".as_bytes()), "32 lowercase hexadecimal characters");
    assert!(!holds_a_random_run("8f14e45fceea167a5a36dedd4bea254".as_bytes()), "31 is an identifier");
    assert!(holds_a_random_run("DEADBEEF".repeat(4).as_bytes()), "32 hexadecimal characters in capitals");
    assert!(holds_a_random_run("0123456789abcdef0123456789abcdef".as_bytes()), "a Gitea token is 40 of them");
    assert!(holds_a_random_run("Q2xpZW50U2VjcmV0MTIzNDU2Nzg5MA".as_bytes()), "24 base64 characters with a digit and both cases");
    let base64_at_twenty_three: &str = "Q2xpZW50U2VjcmV0MTIzNDU";
    assert_eq!(base64_at_twenty_three.len(), 23, "the control's own length");
    assert!(!holds_a_random_run(base64_at_twenty_three.as_bytes()), "23 is below the run of base64");
    assert!(!holds_a_random_run("a-few-hyphenated-words-in-a-row".as_bytes()), "a lowercase slug");
    assert!(!holds_a_random_run("someCamelCaseIdentifierNameHere".as_bytes()), "no digit");
    assert!(!holds_a_random_run("20130524T000000Z".as_bytes()), "a signed URL's date");
    assert!(!holds_a_random_run("deadbeef".as_bytes()), "a Git abbreviation");
}

/// What:
///  A credential after `Bearer`,
/// `Basic` or `RemoteAuth` is recognized wherever it sits,
/// and the words of a sentence are not.
/// Why:
///   A batch response's `header` object carries one,
/// and an error that echoes it would print the token whole;
/// `Basic realm="monochromatic-lfs"` and "Basic authentication failed" are what a person reads.
#[test]
fn a_credential_after_a_scheme_word_is_recognized() {
    let credential: &str = "ZmFrZXVzZXI6ZmFrZXBhc3N3b3Jk";
    for text in [format!("Bearer {credential}"), format!("basic {credential}"), format!("RemoteAuth {credential}")] {
        assert!(scheme_credential_in(text.as_bytes(), false).is_some(), "{text}");
    }
    assert!(scheme_credential_in(format!("Authorization: Bearer {credential}").as_bytes(), false).is_some(), "after a header name");
    assert!(scheme_credential_in(format!("\"Authorization\":\"Bearer {credential}\"").as_bytes(), false).is_some(), "inside JSON");
    assert!(scheme_credential_in(b"Basic realm=\"monochromatic-lfs\"", false).is_none(), "a realm is no credential");
    assert!(scheme_credential_in(b"Basic authentication failed", false).is_none(), "a word of the sentence");
    assert!(scheme_credential_in(b"Bearer short", false).is_none(), "a short word");
    let joined: String = format!("Cyberbasic {credential}");
    assert!(scheme_credential_in(joined.as_bytes(), false).is_none(), "a scheme has to be a word of its own");
    assert!(scheme_credential_in(credential.as_bytes(), true).is_none(), "and so does the word before it");
}

/// What:
///  The bytes `%XX` writes are read as the bytes they are,
/// and a `%` that starts nothing stays as it is.
/// Why:
///   A secret written with percent encoding is the same secret.
#[test]
fn percent_encoding_is_read_once() {
    assert_eq!(percent_decoded(b"a%2Fb"), b"a/b".to_vec());
    assert_eq!(percent_decoded(b"to%6Ben"), b"token".to_vec());
    assert_eq!(percent_decoded(b"100%25"), b"100%".to_vec());
    assert_eq!(percent_decoded(b"a%zz"), b"a%zz".to_vec());
    assert_eq!(percent_decoded(b"plain"), b"plain".to_vec());
    assert_eq!(percent_decoded(b"%41%42"), b"AB".to_vec());
}
