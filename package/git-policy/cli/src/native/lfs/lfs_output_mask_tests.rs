//! Controls for hiding URL credentials in relayed `git-lfs` text,
//! with URLs written to break the boundary of the userinfo.

use super::*;

/// The text with its credentials hidden.
fn masked(text: &str) -> String {
    return mask_userinfo_text(text);
}

/// The batch error of the incident:
/// the user and the token are gone,
/// the scheme,
/// host,
/// path and status stay.
#[test]
fn the_incident_line_loses_its_user_and_token() {
    assert_eq!(
        masked("batch response: Fatal error: Server error https://user:secret@lfs.example.invalid/objects/batch from HTTP 503"),
        format!("batch response: Fatal error: Server error https://{CREDENTIALS_MASK}@lfs.example.invalid/objects/batch from HTTP 503"),
    );
}

/// A URL with a user and no password,
/// a token as the user,
/// and a bare `@` host are all userinfo.
#[test]
fn a_user_alone_and_a_token_alone_are_masked() {
    for userinfo in ["git", "ghp_token", "user:", ":pass", "us.er:p-a_s~s"] {
        assert_eq!(masked(&format!("ssh://{userinfo}@host/r.git")), format!("ssh://{CREDENTIALS_MASK}@host/r.git"), "{userinfo}");
    }
}

/// An `@` after the first `/`,
/// `?` or `#` belongs to the path,
/// query or fragment,
/// and a URL without userinfo is unchanged.
#[test]
fn an_at_sign_outside_the_authority_is_left_alone() {
    for text in [
        "https://host/path@name/objects",
        "https://host/a/b@c@d",
        "https://host?next=user@example.invalid",
        "https://host#frag@ment",
        "https://host:8443/@scope/pkg",
        "https://[::1]:8080/@scope",
        "https://host\\path@name",
        "https://host",
        "no url here, only user@example.invalid",
        "scp-style git@host:path.git",
    ] {
        assert_eq!(masked(text), text);
    }
}

/// The userinfo ends at the last `@` of the authority,
/// so an `@` inside a password cannot leave a tail behind.
#[test]
fn the_userinfo_ends_at_the_last_at_sign_of_the_authority() {
    assert_eq!(masked("https://user:p@ss@host/x"), format!("https://{CREDENTIALS_MASK}@host/x"));
    assert_eq!(masked("https://a@b@c@host"), format!("https://{CREDENTIALS_MASK}@host"));
    assert_eq!(masked("https://user:p@ss@host/path@tail"), format!("https://{CREDENTIALS_MASK}@host/path@tail"));
}

/// Percent-encoded userinfo,
/// including an encoded `@` and `:`,
/// is masked whole,
/// and encoded text in the path stays.
#[test]
fn percent_encoded_userinfo_is_masked_whole_and_the_path_keeps_its_encoding() {
    assert_eq!(
        masked("https://us%40er:p%3Ass%2F@host/a%40b/objects"),
        format!("https://{CREDENTIALS_MASK}@host/a%40b/objects"),
    );
    assert_eq!(masked("https://%75ser:%73ecret@host/"), format!("https://{CREDENTIALS_MASK}@host/"));
}

/// An IPv6 host in brackets keeps its colons and port;
/// the userinfo before it is masked,
/// and without userinfo nothing is.
#[test]
fn bracketed_ipv6_hosts_are_kept_and_their_userinfo_is_masked() {
    assert_eq!(masked("https://user:secret@[::1]:8080/objects"), format!("https://{CREDENTIALS_MASK}@[::1]:8080/objects"));
    assert_eq!(masked("https://u:s@[2001:db8::7]/x"), format!("https://{CREDENTIALS_MASK}@[2001:db8::7]/x"));
    assert_eq!(masked("https://[fe80::1%25eth0]:9/x@y"), "https://[fe80::1%25eth0]:9/x@y");
}

/// Every URL on a line is handled on its own,
/// with text,
/// quotes,
/// brackets and punctuation between them.
#[test]
fn several_urls_on_one_line_are_masked_one_by_one() {
    assert_eq!(
        masked("from https://a:1@one.invalid/x to \"http://b:2@two.invalid:81/y\" via <ssh://c@three.invalid> (https://four.invalid/z@q)"),
        format!(
            "from https://{m}@one.invalid/x to \"http://{m}@two.invalid:81/y\" via <ssh://{m}@three.invalid> (https://four.invalid/z@q)",
            m = CREDENTIALS_MASK
        ),
    );
    assert_eq!(
        masked("https://a:1@h1/x,https://b:2@h2/y"),
        format!("https://{m}@h1/x,https://{m}@h2/y", m = CREDENTIALS_MASK),
    );
    assert_eq!(
        masked("Put \"https://u:p@storage.invalid/o?sig=a@b\": EOF; Post https://u:p@lfs.invalid/objects/batch: dial tcp"),
        format!("Put \"https://{m}@storage.invalid/o?sig=a@b\": EOF; Post https://{m}@lfs.invalid/objects/batch: dial tcp", m = CREDENTIALS_MASK),
    );
}

/// A URL whose authority ends at a quote,
/// a bracket,
/// a backtick or the end of the text still hides its userinfo.
#[test]
fn the_authority_may_end_at_a_quote_a_bracket_or_the_end_of_the_text() {
    for (before, after) in [("'", "'"), ("`", "`"), ("<", ">"), ("\"", "\""), ("", "")] {
        assert_eq!(
            masked(&format!("{before}https://u:p@host{after}")),
            format!("{before}https://{CREDENTIALS_MASK}@host{after}"),
            "{before}{after}"
        );
    }
}

/// Schemes with digits,
/// plus signs,
/// dots and hyphens count;
/// a separator with no scheme before it does not start a URL.
#[test]
fn any_scheme_counts_and_a_bare_separator_does_not() {
    assert_eq!(masked("git+ssh://u:p@host/r"), format!("git+ssh://{CREDENTIALS_MASK}@host/r"));
    assert_eq!(masked("x-y.z1://u:p@host/r"), format!("x-y.z1://{CREDENTIALS_MASK}@host/r"));
    assert_eq!(masked("file:///home/u@x/repo"), "file:///home/u@x/repo");
    assert_eq!(masked(" ://u:p@host/r"), " ://u:p@host/r");
    assert_eq!(masked("://u:p@host"), "://u:p@host");
}

/// Text that is not a URL passes through byte for byte,
/// including bytes that are not UTF-8 and the empty text.
#[test]
fn other_text_and_other_bytes_pass_through_unchanged() {
    assert_eq!(masked(""), "");
    assert_eq!(masked("Uploading LFS objects: 100% (3/3), 12 MB | 1.1 MB/s\r"), "Uploading LFS objects: 100% (3/3), 12 MB | 1.1 MB/s\r");
    let raw: Vec<u8> = b"bad \xff\xfe bytes https://\xc3\xa9:p\xff@host/\xff tail".to_vec();
    let expected: Vec<u8> = [b"bad \xff\xfe bytes https://".as_slice(), CREDENTIALS_MASK.as_bytes(), b"@host/\xff tail".as_slice()].concat();
    assert_eq!(mask_userinfo(raw.as_slice()), expected);
}

/// A very long line of separators and `@` signs is masked in one pass:
/// a quadratic scan would not finish.
#[test]
fn a_long_line_is_masked_in_linear_time() {
    let line: String = "https://a@h ".repeat(200_000);
    let started = std::time::Instant::now();
    let result: String = masked(&line);
    // The claim is that the scan is linear and not quadratic, and the only witness of a work rate is the time it took; a
    //  count of the bytes the scan looked at would need a seam in the mask. Measured on this host at a load average of 47: this
    //  line was masked in 0.08 s, so the 30 s ceiling carries 375 times the figure it was measured at and a quadratic scan is
    //  still minutes away from it.
    assert!(started.elapsed() < std::time::Duration::from_secs(30), "a quadratic scan of this line takes minutes: {:?}", started.elapsed());
    assert!(!result.contains("//a@"), "every userinfo is hidden");
    let unterminated: String = format!("https://{}", "@".repeat(1_000_000));
    assert!(masked(&unterminated).starts_with("https://<credentials>@"));
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
    // Text this module already masked still names its server: the mask stands where the userinfo was.
    assert_eq!(
        hosts_in(&format!("batch response: Authorization error: https://lfs:{CREDENTIALS_MASK}@host.invalid/objects/batch")),
        vec![String::from("host.invalid")]
    );
}
