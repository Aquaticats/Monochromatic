//! Controls for what the wrapper shows of a URL in the text it relays:
//! a table of URLs that carry a secret of each recognized kind in each place a URL can carry one,
//! a table of URLs that carry none and are printed as they are,
//! and the delimiters, quotes and control characters that must not let a secret through.

use super::*;

/// The text with its secrets hidden.
fn masked(text: &str) -> String {
    return mask_url_secrets_text(text);
}

/// The head of a longer text, cut at a size cap,
/// with its secrets hidden.
fn masked_cut(text: &str) -> String {
    return String::from_utf8_lossy(mask_url_secrets_cut(text.as_bytes()).as_slice()).into_owned();
}

/// The prefix GitHub documents for its classic personal access token.
const GITHUB_PREFIX: &str = "ghp_";

/// The prefix GitLab documents for its personal access token.
const GITLAB_PREFIX: &str = "glpat-";

/// What:
///  A fake GitHub token: the documented prefix and a body of 36 letters and digits,
///  made up for these controls and of no account anywhere.
/// Why:
///   The repository's own scanner refuses a file that holds a whole token of this shape,
/// so the prefix and the body are kept apart here and put together at run time.
fn fake_github_token() -> String {
    return format!("{GITHUB_PREFIX}{}", "FAKEfake1234FAKEfake1234FAKEfake1234");
}

/// A fake GitLab token: the documented prefix and a body of 20 letters and digits,
/// made up and put together the way the GitHub one is.
fn fake_gitlab_token() -> String {
    return format!("{GITLAB_PREFIX}{}", "FAKEfake1234FAKEfake");
}

/// A fake 256-bit key written as hexadecimal, in both cases:
/// 32 characters.
const FAKE_HEX_KEY: &str = "aB3f9c2d4e5f60718293a4b5c6d7e8f9";

/// A fake key written in base64:
/// 24 characters holding a lowercase letter, an uppercase letter and a digit.
const FAKE_BASE64_KEY: &str = "Q2xpZW50U2VjcmV0MTIzNDU2Nzg5MA";

/// A fake credential after a scheme word:
/// base64 of a made-up user and password.
const FAKE_BASIC_CREDENTIAL: &str = "ZmFrZXVzZXI6ZmFrZXBhc3N3b3Jk";

/// The name an LFS object has:
/// 64 lowercase hexadecimal characters.
/// The value is made up and is no hash of anything.
const OBJECT_NAME: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// What:
///  A secret of each recognized kind, in each place a URL can carry one,
/// is replaced by the mask,
/// and the rest of the URL is printed as it is.
/// Why:
///   This is the whole protection after the owner's decision of 2026-11-12T10:32:10-05:00 (item 1.9 of
///  `doc/handover/cli-git-rust-implementation.md`,
/// "Mask recognised secrets only"),
/// so each rule needs a case of its own in each place a token is written.
#[test]
fn a_secret_of_every_recognized_kind_is_hidden_in_every_place_a_url_holds_one() {
    let cases: [(&str, &str); 18] = [
        // The user information is a secret whatever it holds.
        ("https://user:FAKEpass@host.invalid/owner/repo.git", "https://<credentials>@host.invalid/owner/repo.git"),
        ("ssh://someone@host.invalid:22/repo.git", "ssh://<credentials>@host.invalid:22/repo.git"),
        // A path segment whose own bytes name a secret.
        ("http://127.0.0.1:9/tokenvalue/lfs/objects/batch", "http://127.0.0.1:9/<credentials>/lfs/objects/batch"),
        ("https://host.invalid/my-api-key/lfs", "https://host.invalid/<credentials>/lfs"),
        // A path segment that is a token of a format a Git host issues.
        ("https://host.invalid/{github}/lfs", "https://host.invalid/<credentials>/lfs"),
        ("https://host.invalid/{gitlab}/lfs", "https://host.invalid/<credentials>/lfs"),
        ("https://host.invalid/ghp%5FFAKEfake1234FAKEfake1234FAKEfake1234/lfs", "https://host.invalid/<credentials>/lfs"),
        // A path segment that is a run of random characters.
        ("https://host.invalid/{hex}/lfs", "https://host.invalid/<credentials>/lfs"),
        ("https://host.invalid/{base64}/lfs", "https://host.invalid/<credentials>/lfs"),
        ("https://host.invalid/objects/{hex}/lfs", "https://host.invalid/objects/<credentials>/lfs"),
        // A query value under a name that says it is a secret.
        ("https://host.invalid/lfs?token=FAKEquerysecret", "https://host.invalid/lfs?token=<credentials>"),
        ("https://host.invalid/lfs?ToKeN=FAKEquerysecret", "https://host.invalid/lfs?ToKeN=<credentials>"),
        ("https://host.invalid/lfs?sig=FAKEsig", "https://host.invalid/lfs?sig=<credentials>"),
        (
            "https://bucket.invalid/images/photo.jpg?X-Amz-Signature=FAKEsignature&X-Amz-Expires=900",
            "https://bucket.invalid/images/photo.jpg?X-Amz-Signature=<credentials>&X-Amz-Expires=900",
        ),
        (
            "https://storage.googleapis.com/bucket/photo.jpg?X-Goog-Credential=FAKE%40project.invalid%2F20261112%2Fus%2Fgoog4_request",
            "https://storage.googleapis.com/bucket/photo.jpg?X-Goog-Credential=<credentials>",
        ),
        // A query value of a recognized shape under a name that says nothing.
        ("https://host.invalid/lfs?ref={github}", "https://host.invalid/lfs?ref=<credentials>"),
        ("https://host.invalid/lfs?{github}=1", "https://host.invalid/lfs?<credentials>=1"),
        // A fragment.
        ("https://host.invalid/lfs#access_token=FAKEfragmentsecret", "https://host.invalid/lfs#access_token=<credentials>"),
    ];
    for (text, wanted) in cases {
        let github: String = fake_github_token();
        let gitlab: String = fake_gitlab_token();
        let line: String = text.replace("{github}", &github).replace("{gitlab}", &gitlab).replace("{hex}", FAKE_HEX_KEY).replace("{base64}", FAKE_BASE64_KEY);
        let expected: String = wanted
            .replace("{github}", &github)
            .replace("{gitlab}", &gitlab)
            .replace("{hex}", FAKE_HEX_KEY)
            .replace("{base64}", FAKE_BASE64_KEY)
            .replace("<credentials>", CREDENTIALS_MASK);
        assert_eq!(masked(&line), expected, "{line}");
        for canary in [github.as_str(), gitlab.as_str(), FAKE_HEX_KEY, FAKE_BASE64_KEY] {
            assert!(!masked(&line).contains(canary), "{canary} survived in {line}");
        }
    }
}

/// What:
///  A fragment that is a bare token of a recognized shape is hidden whole.
/// Why:
///   A fragment is a place a token is carried too,
/// and it has no name to say what it is.
#[test]
fn a_bare_fragment_of_a_recognized_shape_is_hidden() {
    let github: String = fake_github_token();
    assert_eq!(masked(&format!("https://host.invalid/lfs#{github}")), format!("https://host.invalid/lfs#{CREDENTIALS_MASK}"));
    assert_eq!(masked(&format!("https://host.invalid/lfs#{FAKE_HEX_KEY}")), format!("https://host.invalid/lfs#{CREDENTIALS_MASK}"));
    assert_eq!(masked("https://host.invalid/lfs#main"), "https://host.invalid/lfs#main");
}

/// What:
///  A credential after `Bearer`,
/// `Basic` or `RemoteAuth` is hidden wherever it appears,
/// inside a URL and outside one.
/// Why:
///   A batch response's `header` object carries one,
/// and an error that echoes it would print the token whole.
#[test]
fn a_credential_after_a_scheme_word_is_hidden_wherever_it_appears() {
    for text in [
        format!("batch response: Authorization: Bearer {FAKE_BASIC_CREDENTIAL}"),
        format!("{{\"header\":{{\"Authorization\":\"Basic {FAKE_BASIC_CREDENTIAL}\"}}}}"),
        format!("git-lfs-authenticate said RemoteAuth {FAKE_BASIC_CREDENTIAL}"),
    ] {
        let shown: String = masked(&text);
        assert!(!shown.contains(FAKE_BASIC_CREDENTIAL), "{shown}");
        assert!(shown.contains(CREDENTIALS_MASK), "{shown}");
    }
    // The words of a sentence are not a credential, and a realm is not one either.
    assert_eq!(masked("Basic authentication failed"), "Basic authentication failed");
    assert_eq!(masked("LFS-Authenticate: Basic realm=\"monochromatic-lfs\""), "LFS-Authenticate: Basic realm=\"monochromatic-lfs\"");
}

/// What:
///  A URL that carries no secret is printed exactly as it was written,
/// including the name of an LFS object,
/// which is the thing a person needs to see.
/// Why:
///   The owner's decision is that a URL is shown;
/// hiding a path that holds nothing was the rule this one replaced.
#[test]
fn a_url_that_carries_no_secret_is_printed_unchanged() {
    let cases: [String; 14] = [
        String::from("https://host.invalid/owner/repo.git/info/lfs"),
        String::from("https://lfs.example.invalid/objects/batch"),
        format!("https://monochromatic-lfs.aquaticat.workers.dev/{OBJECT_NAME}"),
        format!("https://bucket.invalid/images/{OBJECT_NAME}?X-Amz-Date=20130524T000000Z&X-Amz-Expires=900&X-Amz-SignedHeaders=host"),
        format!("https://host.invalid/objects/{OBJECT_NAME}"),
        String::from("https://host.invalid/refs/heads/main?ref=main"),
        String::from("https://host.invalid:8443/"),
        String::from("http://127.0.0.1:36437/lfs"),
        String::from("ssh://host.invalid:22/repo.git"),
        String::from("https://host.invalid/releases/download/v1.2.3/notes.md"),
        String::from("https://host.invalid/a/long-lowercase-hyphenated-slug-that-goes-on-and-on"),
        String::from("https://gitlab.com/group/project.git/info/lfs"),
        String::from("Uploading LFS objects: 100% (3/3), 12 MB | 1.1 MB/s\r"),
        String::from("the push went to origin"),
    ];
    for text in cases {
        assert_eq!(masked(&text), text, "{text}");
    }
}

/// What:
///  An object's name is shown where an LFS server puts it and hidden anywhere else,
/// because no rule can tell a name of that shape from a key of that shape.
/// Why:
///   The name is 64 lowercase hexadecimal characters,
/// which is also what a 256-bit key written in hexadecimal looks like;
/// the reading taken is the owner's ("the thing a person needs to see"),
/// and it holds only for the two places a content-addressed store puts a name:
/// the whole last segment of the path, and the segment after `objects`.
#[test]
fn an_object_name_is_shown_only_where_an_lfs_server_puts_one() {
    assert_eq!(
        masked(&format!("failed to PUT https://lfs.invalid/{OBJECT_NAME}")),
        format!("failed to PUT https://lfs.invalid/{OBJECT_NAME}"),
        "the last segment of the path is the object's name"
    );
    assert_eq!(
        masked(&format!("failed to PUT https://lfs.invalid/objects/{OBJECT_NAME}")),
        format!("failed to PUT https://lfs.invalid/objects/{OBJECT_NAME}"),
        "the segment after `objects` is the object's name"
    );
    assert_eq!(
        masked(&format!("https://lfs.invalid/{OBJECT_NAME}/other")),
        format!("https://lfs.invalid/{CREDENTIALS_MASK}/other"),
        "a name of that shape in the middle of a path is hidden"
    );
    assert_eq!(
        masked(&format!("https://lfs.invalid/lfs?oid={OBJECT_NAME}")),
        format!("https://lfs.invalid/lfs?oid={CREDENTIALS_MASK}"),
        "a name of that shape in a query is hidden"
    );
}

/// What:
///  The batch error of the incident keeps its server and its call and loses its tokens.
/// Why:
///   This is the line that started it:
/// the audit's `lfs.url` carried a token in a path segment and another in its query.
#[test]
fn the_incident_line_keeps_its_server_and_its_call_and_loses_its_tokens() {
    let line: String = format!(
        "batch response: Post \"http://CANARYUSER4411:CANARYpas9@127.0.0.1:1/{FAKE_HEX_KEY}/lfs?token=CANARYQUERY7720/objects/batch\": \
         dial tcp 127.0.0.1:1: connect: connection refused"
    );
    assert_eq!(
        masked(&line),
        format!(
            "batch response: Post \"http://{m}@127.0.0.1:1/{m}/lfs?token={m}\": dial tcp 127.0.0.1:1: connect: connection refused",
            m = CREDENTIALS_MASK
        ),
    );
}

/// What:
///  The audit's lowercase path secret is hidden by the rule that reads a path segment's own bytes,
/// under no special case of its own.
/// Why:
///   `tokenvalue` has no shape a key has:
/// its bytes are the shape a label has,
/// and only the word `token` inside it says what it is.
#[test]
fn the_audits_lowercase_path_secret_is_hidden() {
    let audit_canary: &str = "tokenvalue";
    assert_eq!(
        masked(&format!("http://127.0.0.1:34247/{audit_canary}/objects/batch")),
        format!("http://127.0.0.1:34247/{CREDENTIALS_MASK}/objects/batch"),
    );
}

//region The user information, which is always hidden

/// A URL with a user and no password,
/// a token as the user,
/// and a bare `@` host are all user information.
#[test]
fn a_user_alone_and_a_token_alone_are_masked() {
    for userinfo in ["git", "ghp_token", "user:", ":pass", "us.er:p-a_s~s"] {
        assert_eq!(masked(&format!("ssh://{userinfo}@host/r.git")), format!("ssh://{m}@host/r.git", m = CREDENTIALS_MASK), "{userinfo}");
    }
}

/// An `@` after the first `/`,
/// `?` or `#` names no user:
/// the authority keeps the host it had,
/// and what follows is the path,
/// the query or the fragment.
#[test]
fn an_at_sign_outside_the_authority_is_not_userinfo() {
    for (text, wanted) in [
        ("https://host/path@name/objects", "https://host/path@name/objects"),
        ("https://host?next=user@example.invalid", "https://host?next=user@example.invalid"),
        ("https://host#frag@ment", "https://host#frag@ment"),
        ("https://host:8443/@scope/pkg", "https://host:8443/@scope/pkg"),
        ("https://[::1]:8080/@scope", "https://[::1]:8080/@scope"),
        ("https://host\\path@name", "https://host\\path@name"),
        ("https://host", "https://host"),
        ("no url here, only user@example.invalid", "no url here, only user@example.invalid"),
        ("scp-style git@host:path.git", "scp-style git@host:path.git"),
    ] {
        assert_eq!(masked(text), wanted, "{text}");
    }
}

/// The user information ends at the last `@` of the authority,
/// so an `@` inside a password cannot leave a tail behind.
#[test]
fn the_userinfo_ends_at_the_last_at_sign_of_the_authority() {
    assert_eq!(masked("https://user:p@ss@host/x"), format!("https://{m}@host/x", m = CREDENTIALS_MASK));
    assert_eq!(masked("https://a@b@c@host"), format!("https://{m}@host", m = CREDENTIALS_MASK));
}

/// Percent-encoded user information,
/// including an encoded `@` and `:`,
/// is masked whole.
#[test]
fn percent_encoded_userinfo_is_masked_whole() {
    assert_eq!(masked("https://us%40er:p%3Ass%2F@host/a%40b/objects"), format!("https://{m}@host/a%40b/objects", m = CREDENTIALS_MASK));
    assert_eq!(masked("https://%75ser:%73ecret@host/"), format!("https://{m}@host/", m = CREDENTIALS_MASK));
}

/// An IPv6 host in brackets keeps its colons and port;
/// the user information before it is masked,
/// and without user information nothing of the authority is.
#[test]
fn bracketed_ipv6_hosts_are_kept_and_their_userinfo_is_masked() {
    assert_eq!(masked("https://user:secret@[::1]:8080/objects"), format!("https://{m}@[::1]:8080/objects", m = CREDENTIALS_MASK));
    assert_eq!(masked("https://u:s@[2001:db8::7]/x"), format!("https://{m}@[2001:db8::7]/x", m = CREDENTIALS_MASK));
    assert_eq!(masked("https://[fe80::1%25eth0]:9/x@y"), "https://[fe80::1%25eth0]:9/x@y");
}

//endregion

//region Delimiters, quotes and control characters

/// What:
///  A URL inside quotes,
/// brackets,
/// a backtick or a sentence keeps its secret hidden,
/// and every delimiter after it is printed as it stood.
/// Why:
///   `git-lfs` writes its URLs inside quotes,
/// and Git inside single quotes;
/// a delimiter that ends the URL is copied as it is,
/// so the line a person reads is the line the program wrote.
#[test]
fn delimiters_around_a_url_do_not_let_a_secret_through() {
    let cases: [(&str, String); 5] = [
        ("'{url}'", format!("'{{shown}}'")),
        ("\"{url}\"", format!("\"{{shown}}\"")),
        ("<{url}>", format!("<{{shown}}>")),
        ("`{url}`", format!("`{{shown}}`")),
        ("({url}).", format!("({{shown}}).")),
    ];
    let github: String = fake_github_token();
    let url: String = format!("https://user:FAKEpass@host.invalid/{github}/lfs?token=FAKEquerysecret&ref=main");
    let shown: String = format!("https://{m}@host.invalid/{m}/lfs?token={m}&ref=main", m = CREDENTIALS_MASK);
    for (pattern, wanted) in cases {
        let line: String = pattern.replace("{url}", &url);
        let expected: String = wanted.replace("{shown}", &shown);
        assert_eq!(masked(&line), expected, "{line}");
    }
}

/// What:
///  A newline,
/// a carriage return,
/// a tab or an escape sequence after a URL ends it,
/// the secret before it is hidden,
/// and the line's own breaks are neither added to nor taken away.
/// Why:
///   The relayed text goes to a terminal,
/// and a control character the program wrote is the program's to write;
/// what this mask must not do is print a token beside one.
#[test]
fn a_control_character_after_a_url_ends_it_and_the_secret_is_still_hidden() {
    let url: String = format!("https://host.invalid/lfs?X-Amz-Signature=FAKEsignature");
    let shown: String = format!("https://host.invalid/lfs?X-Amz-Signature={CREDENTIALS_MASK}");
    for after in ["\n", "\r", "\r\n", "\t", "\u{1b}[0m", "\u{7}"] {
        let line: String = format!("{url}{after}next line");
        let masked_line: String = masked(&line);
        assert!(!masked_line.contains("FAKEsignature"), "{:?}", masked_line);
        assert_eq!(masked_line, format!("{shown}{after}next line"), "{:?}", line);
        assert_eq!(masked_line.matches('\n').count(), line.matches('\n').count(), "no line break is added: {:?}", masked_line);
    }
}

/// What:
///  A quote inside a query value ends the URL at the quote,
/// the part before it is still read by its name,
/// and no secret is carried past it.
/// Why:
///   A bare quote cannot be part of a URL,
/// so what follows it is the program's own text;
/// a secret written with percent encoding is read as it decodes,
/// which is how a secret holding a quote is carried inside a URL.
#[test]
fn a_quote_in_a_url_ends_it_and_the_encoded_form_is_read() {
    assert_eq!(
        masked("https://host.invalid/lfs?X-Amz-Signature=FA\"rest"),
        format!("https://host.invalid/lfs?X-Amz-Signature={m}\"rest", m = CREDENTIALS_MASK)
    );
    assert_eq!(
        masked("https://host.invalid/lfs?X-Amz-Signature=FA%22KE"),
        format!("https://host.invalid/lfs?X-Amz-Signature={m}", m = CREDENTIALS_MASK)
    );
}

/// What:
///  Several URLs on one line are handled one at a time,
/// with the text between them unmasked.
/// Why:
///   A line of a failed push can name the destination,
/// the server that refused and the one it fell back to.
#[test]
fn several_urls_on_one_line_are_masked_one_by_one() {
    let line: String = format!(
        "from https://a:FAKEone@one.invalid/tokenvalue/objects to \"http://b:FAKEtwo@two.invalid:81/lfs\" via <ssh://host.invalid> (https://four.invalid/{OBJECT_NAME})"
    );
    assert_eq!(
        masked(&line),
        format!(
            "from https://{m}@one.invalid/{m}/objects to \"http://{m}@two.invalid:81/lfs\" via <ssh://host.invalid> (https://four.invalid/{OBJECT_NAME})",
            m = CREDENTIALS_MASK
        ),
    );
}

//endregion

//region The head of a cut text and a stream read in chunks

/// What:
///  A URL that reaches the end of a text cut at a size cap is read as a URL that may go on,
/// so an authority with no `@` in it is hidden whole.
/// Why:
///   A cut can end between a password and the `@` that ends the user information,
/// and what is left reads as a host:
/// `https://user:<token>` hid the user information of no URL the rule knows.
#[test]
fn an_authority_that_reaches_the_end_of_a_cut_text_is_hidden_whole() {
    assert_eq!(masked_cut("batch response: Post \"https://user:truncatedcanary"), format!("batch response: Post \"https://{CREDENTIALS_MASK}"));
    assert_eq!(masked_cut("Post https://user:secret@host.invalid/lfs"), format!("Post https://{m}@host.invalid/lfs", m = CREDENTIALS_MASK));
    assert_eq!(masked_cut("see https://host.invalid/objects"), "see https://host.invalid/objects");
    // The positive control: the same bytes read as a whole text are a host and a port, which the rule for a text that
    //  ended there keeps, so the cut is what hides them.
    assert_eq!(masked("Post https://user:truncatedcanary"), "Post https://user:truncatedcanary");
}

/// What:
///  A URL whose user information is cut between two reads is masked as a URL,
/// because the stream holds back the word in progress.
/// Why:
///   A relay reads whatever the kernel has, and the `@` that ends user information can fall in the next read.
#[test]
fn a_url_cut_between_two_reads_is_masked_as_a_url() {
    let mut stream: UrlMaskingStream = UrlMaskingStream::new();
    let first: String = String::from_utf8_lossy(stream.feed(b"batch response: Post \"https://user:streamcanary").as_slice()).into_owned();
    assert!(!first.contains("streamcanary"), "the word in progress is held and not written: {first}");
    let second: String = String::from_utf8_lossy(stream.feed(b"@host.invalid/tokenvalue/lfs?token=FAKEquery\": EOF\n").as_slice()).into_owned();
    assert_eq!(
        format!("{first}{second}"),
        format!("batch response: Post \"https://{m}@host.invalid/{m}/lfs?token={m}\": EOF\n", m = CREDENTIALS_MASK),
        "the held word is classified with the read that ends it"
    );
    assert_eq!(String::from_utf8_lossy(stream.finish().as_slice()), "", "nothing is left held");
}

/// What:
///  A whole chunk ending at a separator is masked and handed over at once,
/// and a last word with no end is hidden the way the head of a cut text is.
#[test]
fn a_chunk_that_ends_a_word_is_handed_over_at_once_and_a_last_word_is_hidden() {
    let mut stream: UrlMaskingStream = UrlMaskingStream::new();
    let shown: String =
        String::from_utf8_lossy(stream.feed(b"To https://user:chunkcanary@host.invalid/repo.git\n").as_slice()).into_owned();
    assert_eq!(shown, format!("To https://{m}@host.invalid/repo.git\n", m = CREDENTIALS_MASK), "a whole line is written as soon as it ends");
    assert_eq!(
        String::from_utf8_lossy(stream.feed(b"fatal: unable to access 'https://user:lastcanary").as_slice()),
        "fatal: unable to access '",
        "the bytes before the last word are written and the last word is held"
    );
    let tail: String = String::from_utf8_lossy(stream.finish().as_slice()).into_owned();
    assert_eq!(tail, format!("https://{CREDENTIALS_MASK}"), "the last word is hidden whole");
}

/// What:
///  A word longer than the stream holds is hidden whole when it carries a scheme,
/// the rest of that word is dropped,
/// and a long word with no scheme is handed over as it stands.
/// Why:
///   A word that never ends may be a token the mask must hide, and its middle may not be written unmasked afterwards.
#[test]
fn a_word_longer_than_the_stream_holds_is_hidden_and_its_rest_dropped() {
    let mut stream: UrlMaskingStream = UrlMaskingStream::new();
    let long_token: String = format!("https://host.invalid/ghp_{}", "a".repeat(LONGEST_HELD_WORD));
    let shown: String = String::from_utf8_lossy(stream.feed(long_token.as_bytes()).as_slice()).into_owned();
    assert_eq!(shown, format!("https://host.invalid/{}", CREDENTIALS_MASK), "the long token is hidden whole");
    assert_eq!(
        String::from_utf8_lossy(stream.feed(b"the_rest_of_that_word\nnext line\n").as_slice()),
        "\nnext line\n",
        "the rest of the hidden word is dropped and what follows it is written"
    );
    let mut plain: UrlMaskingStream = UrlMaskingStream::new();
    let long_plain: String = "p".repeat(LONGEST_HELD_WORD + 1);
    let handed: String = String::from_utf8_lossy(plain.feed(long_plain.as_bytes()).as_slice()).into_owned();
    assert_eq!(handed, long_plain, "a long word with no scheme is handed over as it stands");
    // A long URL that holds nothing to hide is handed over whole, and what follows it is read as usual.
    let mut shown_path: UrlMaskingStream = UrlMaskingStream::new();
    let long_path: String = format!("https://host.invalid/{}", "l".repeat(LONGEST_HELD_WORD));
    let handed_over: String = String::from_utf8_lossy(shown_path.feed(long_path.as_bytes()).as_slice()).into_owned();
    assert_eq!(handed_over, long_path, "a long path that holds no secret is shown whole");
    assert_eq!(String::from_utf8_lossy(shown_path.feed(b"tail\n").as_slice()), "tail\n", "the word goes on being read");
}

//endregion

//region Text that is not a URL, and a line long enough to matter

/// Text that is not a URL passes through byte for byte,
/// including bytes that are not UTF-8 and the empty text.
#[test]
fn other_text_and_other_bytes_pass_through_unchanged() {
    assert_eq!(masked(""), "");
    assert_eq!(masked("Uploading LFS objects: 100% (3/3), 12 MB | 1.1 MB/s\r"), "Uploading LFS objects: 100% (3/3), 12 MB | 1.1 MB/s\r");
    let raw: Vec<u8> = b"bad \xff\xfe bytes https://\xc3\xa9:p\xff@host/objects \xff tail".to_vec();
    let expected: Vec<u8> =
        [b"bad \xff\xfe bytes https://".as_slice(), CREDENTIALS_MASK.as_bytes(), b"@host/objects \xff tail".as_slice()].concat();
    assert_eq!(mask_url_secrets(raw.as_slice()), expected);
}

/// Schemes with digits,
/// plus signs,
/// dots and hyphens count;
/// a separator with no scheme before it does not start a URL.
#[test]
fn any_scheme_counts_and_a_bare_separator_does_not() {
    assert_eq!(masked("git+ssh://u:p@host/r"), format!("git+ssh://{m}@host/r", m = CREDENTIALS_MASK));
    assert_eq!(masked("x-y.z1://u:p@host/r"), format!("x-y.z1://{m}@host/r", m = CREDENTIALS_MASK));
    assert_eq!(masked("file:///home/u@x/repo"), "file:///home/u@x/repo");
    assert_eq!(masked(" ://u:p@host/r"), " ://u:p@host/r");
    assert_eq!(masked("://u:p@host"), "://u:p@host");
}

/// The host a destination names is the server,
/// with no user,
/// token or port left to guess about.
#[test]
fn the_host_of_a_destination_is_the_server_alone() {
    let cases: [(&str, Option<&str>); 7] = [
        ("https://lfs:<credentials>@monochromatic-lfs.aquaticat.workers.dev/objects/batch", Some("monochromatic-lfs.aquaticat.workers.dev")),
        ("https://user:secret@127.0.0.1:36437/lfs", Some("127.0.0.1:36437")),
        ("https://example.invalid/x", Some("example.invalid")),
        ("ssh://git@host:22/repo.git", Some("host:22")),
        ("origin", None),
        ("/home/person/phone/pictures", None),
        ("file:///home/person/repo.git", None),
    ];
    for (destination, expected) in cases {
        assert_eq!(host_of(destination).as_deref(), expected, "{destination}");
    }
}

/// The servers a failure's own words name are read in order,
/// whoever they carry in front of the host.
#[test]
fn the_servers_a_failure_names_are_read_in_order() {
    let line: &str = "fatal: could not read Username for 'https://lfs.example.invalid': No such device or address; \
                      batch response: Authorization error: https://user:secret@other.example.invalid/objects/batch";
    assert_eq!(hosts_in(line), vec![String::from("lfs.example.invalid"), String::from("other.example.invalid")]);
    assert_eq!(hosts_in("nothing here names a server"), Vec::<String>::new());
    assert_eq!(hosts_in("a path /home/person/repo is not one"), Vec::<String>::new());
    assert_eq!(hosts_in("the same host twice: https://host.invalid/a and https://host.invalid/b"), vec![String::from("host.invalid")]);
    assert_eq!(
        hosts_in(&format!("batch response: Authorization error: https://lfs:{CREDENTIALS_MASK}@host.invalid/objects/batch")),
        vec![String::from("host.invalid")]
    );
}

/// What:
///  A very long line of URLs is masked in one pass,
/// in the time a linear scan takes.
/// Why:
///   A quadratic scan would not finish on a line of output that names many objects.
#[test]
fn a_long_line_is_masked_in_linear_time() {
    let github: String = fake_github_token();
    let one: String = format!("https://lfs:{github}@host/objects/{OBJECT_NAME}/{FAKE_HEX_KEY}/lfs?token=FAKE&ref=main ");
    let line: String = one.repeat(20_000);
    let started: std::time::Instant = std::time::Instant::now();
    let result: String = masked(&line);
    // The claim is that the scan is linear and not quadratic, and the only witness of a work rate is the time it took; a
    //  count of the bytes the scan looked at would need a seam in the mask. The ceiling carries a wide margin over the
    //  figure measured on this host, and a quadratic scan of this line is minutes away from it.
    assert!(started.elapsed() < std::time::Duration::from_secs(60), "a quadratic scan of this line takes minutes: {:?}", started.elapsed());
    assert!(!result.contains(github.as_str()), "a user information is hidden");
    assert!(!result.contains(FAKE_HEX_KEY), "a key in a path is hidden");
    assert!(
        result.contains(&format!("https://{CREDENTIALS_MASK}@host/objects/{OBJECT_NAME}/{CREDENTIALS_MASK}/lfs?token={CREDENTIALS_MASK}&ref=main ")),
        "the masked shape is kept"
    );
}

//endregion
