//! Controls for hiding the secrets of a URL in relayed `git-lfs` text,
//! with URLs written to break the boundary of the userinfo and of a path label.

use super::*;

/// The text with its credentials hidden.
fn masked(text: &str) -> String {
    return mask_url_secrets_text(text);
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
/// `?` or `#` names no user:
/// the authority keeps the host it had,
/// and what follows is the path,
/// the query or the fragment,
/// where a segment that is not a plain label is hidden.
#[test]
fn an_at_sign_outside_the_authority_is_not_userinfo() {
    for (text, expected) in [
        ("https://host/path@name/objects", "https://host/<credentials>/objects"),
        ("https://host/a/b@c@d", "https://host/a/<credentials>"),
        ("https://host?next=user@example.invalid", "https://host?next=<credentials>"),
        ("https://host#frag@ment", "https://host#<credentials>"),
        ("https://host:8443/@scope/pkg", "https://host:8443/<credentials>/pkg"),
        ("https://[::1]:8080/@scope", "https://[::1]:8080/<credentials>"),
        ("https://host\\path@name", "https://host\\path@name"),
        ("https://host", "https://host"),
        ("no url here, only user@example.invalid", "no url here, only user@example.invalid"),
        ("scp-style git@host:path.git", "scp-style git@host:path.git"),
    ] {
        let expected: String = expected.replace("<credentials>", CREDENTIALS_MASK);
        assert_eq!(masked(text), expected, "{text}");
    }
}

/// The userinfo ends at the last `@` of the authority,
/// so an `@` inside a password cannot leave a tail behind.
#[test]
fn the_userinfo_ends_at_the_last_at_sign_of_the_authority() {
    assert_eq!(masked("https://user:p@ss@host/x"), format!("https://{CREDENTIALS_MASK}@host/x"));
    assert_eq!(masked("https://a@b@c@host"), format!("https://{CREDENTIALS_MASK}@host"));
    assert_eq!(masked("https://user:p@ss@host/plain"), format!("https://{CREDENTIALS_MASK}@host/plain"));
}

/// Percent-encoded userinfo,
/// including an encoded `@` and `:`,
/// is masked whole,
/// and an encoded path segment is hidden with it,
/// because a percent sign is not what a plain label is made of.
#[test]
fn percent_encoded_userinfo_is_masked_whole_and_an_encoded_segment_is_hidden() {
    assert_eq!(
        masked("https://us%40er:p%3Ass%2F@host/a%40b/objects"),
        format!("https://{CREDENTIALS_MASK}@host/{CREDENTIALS_MASK}/objects"),
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
    assert_eq!(masked("https://[fe80::1%25eth0]:9/x@y"), format!("https://[fe80::1%25eth0]:9/{CREDENTIALS_MASK}"));
}

/// Every URL on a line is handled on its own,
/// with text,
/// quotes,
/// brackets and punctuation between them.
#[test]
fn several_urls_on_one_line_are_masked_one_by_one() {
    assert_eq!(
        masked("from https://a:1@one.invalid/x to \"http://b:2@two.invalid:81/y\" via <ssh://c@three.invalid> (https://four.invalid/plain)"),
        format!(
            "from https://{m}@one.invalid/x to \"http://{m}@two.invalid:81/y\" via <ssh://{m}@three.invalid> (https://four.invalid/plain)",
            m = CREDENTIALS_MASK
        ),
    );
    assert_eq!(
        masked("https://a:1@h1/x, https://b:2@h2/y"),
        format!("https://{m}@h1/x, https://{m}@h2/y", m = CREDENTIALS_MASK),
    );
    assert_eq!(
        masked("Put \"https://u:p@storage.invalid/o?sig=a@b\": EOF; Post https://u:p@lfs.invalid/objects/batch: dial tcp"),
        format!("Put \"https://{m}@storage.invalid/o?sig={m}\": EOF; Post https://{m}@lfs.invalid/objects/batch: dial tcp", m = CREDENTIALS_MASK),
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
    assert_eq!(masked("file:///home/u@x/repo"), format!("file:///home/{CREDENTIALS_MASK}/repo"));
    assert_eq!(masked(" ://u:p@host/r"), " ://u:p@host/r");
    assert_eq!(masked("://u:p@host"), "://u:p@host");
}

/// Text that is not a URL passes through byte for byte,
/// including bytes that are not UTF-8 and the empty text.
#[test]
fn other_text_and_other_bytes_pass_through_unchanged() {
    assert_eq!(masked(""), "");
    assert_eq!(masked("Uploading LFS objects: 100% (3/3), 12 MB | 1.1 MB/s\r"), "Uploading LFS objects: 100% (3/3), 12 MB | 1.1 MB/s\r");
    let raw: Vec<u8> = b"bad \xff\xfe bytes https://\xc3\xa9:p\xff@host/objects \xff tail".to_vec();
    let expected: Vec<u8> = [b"bad \xff\xfe bytes https://".as_slice(), CREDENTIALS_MASK.as_bytes(), b"@host/objects \xff tail".as_slice()].concat();
    assert_eq!(mask_url_secrets(raw.as_slice()), expected);
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

//region The rest of an LFS URL: a token in a path segment, a query and a fragment

/// The user the audit's fixture wrote in `lfs.url`, made up for this control.
const CANARY_USER: &str = "CANARYUSER4411";

/// The password of the same fixture, made up.
const CANARY_PASS: &str = "CANARYPASS9137";

/// The token the fixture wrote as a path segment, made up.
const CANARY_PATH: &str = "CANARYPATH5521";

/// The token the fixture wrote as a query value, made up.
const CANARY_QUERY: &str = "CANARYQUERY7720";

/// The batch error of the audit's fixture:
/// `lfs.url` carried a token in a path segment and another in its query,
/// the user and the password were masked and those two were printed.
///  Every secret-shaped part of the URL is hidden now,
/// and the scheme,
/// host and port a person acts on stay.
#[test]
fn a_token_in_the_path_of_an_lfs_url_is_hidden() {
    // Written from the parts, as the appending of `/objects/batch` to a URL that already holds a query is git-lfs's own.
    let line: String = format!(
        "batch response: Post \"http://{CANARY_USER}:{CANARY_PASS}@127.0.0.1:1/{CANARY_PATH}/lfs?token={CANARY_QUERY}/objects/batch\": \
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

/// Every value of a query is hidden and every plain key stays,
/// so a person still sees which settings the server was given.
#[test]
fn every_value_of_a_query_is_hidden_and_a_plain_key_stays() {
    assert_eq!(
        masked("https://lfs.example.invalid/lfs?token=CANARYQUERY7720&ref=main&sig=a%2Fb"),
        format!("https://lfs.example.invalid/lfs?token={m}&ref={m}&sig={m}", m = CREDENTIALS_MASK),
    );
    // A key is hidden too when it is shaped like a token: the config key of the incident carried one.
    assert_eq!(
        masked("https://host.invalid/lfs?CANARYKEY7720=1"),
        format!("https://host.invalid/lfs?{m}={m}", m = CREDENTIALS_MASK),
    );
}

/// A URL that carries no secret at all is printed as it was typed,
/// so the server and the path it names stay readable.
#[test]
fn a_url_with_no_secret_is_left_alone() {
    for text in [
        "http://127.0.0.1:36437/lfs",
        "http://127.0.0.1:36437/silent.git",
        "https://lfs.example.invalid/objects/batch",
        "https://storage.example.invalid/info/lfs",
        "https://host:8443/",
        "ssh://host:22/repo.git",
        "the push went to origin",
    ] {
        assert_eq!(masked(text), text);
    }
}

/// A secret written with percent encoding is hidden as it stands,
/// in a path segment and in a query value alike.
#[test]
fn an_encoded_secret_in_a_path_or_a_query_is_hidden() {
    assert_eq!(masked("https://host.invalid/a%2Fb/lfs"), format!("https://host.invalid/{m}/lfs", m = CREDENTIALS_MASK));
    assert_eq!(masked("https://host.invalid/lfs?token=%43ANARY%2B7720"), format!("https://host.invalid/lfs?token={m}", m = CREDENTIALS_MASK));
    // A segment carries digits, which no plain label does.
    assert_eq!(masked("https://host.invalid/CANARYPATH5521/lfs"), format!("https://host.invalid/{m}/lfs", m = CREDENTIALS_MASK));
}

/// A fragment is a place a token is carried too,
/// and its plain keys stay while every value is hidden.
#[test]
fn a_fragment_keeps_its_plain_keys_and_hides_its_values() {
    assert_eq!(
        masked("https://host.invalid/lfs#access_token=CANARYQUERY7720"),
        format!("https://host.invalid/lfs#access_token={m}", m = CREDENTIALS_MASK),
    );
    assert_eq!(masked("https://host.invalid/lfs#CANARYQUERY7720"), format!("https://host.invalid/lfs#{m}", m = CREDENTIALS_MASK));
}

/// A long line of URLs whose paths and queries carry tokens is masked with no token left and no quadratic scan:
/// the pass that hid the userinfo now hides the rest of the URL too,
/// and it still reads every byte a constant number of times.
#[test]
fn a_long_line_of_tokens_is_hidden_in_linear_time() {
    let one: String = format!("https://lfs:{CANARY_USER}:{CANARY_PASS}@host/{CANARY_PATH}/lfs?token={CANARY_QUERY} ");
    let line: String = one.repeat(100_000);
    let started: std::time::Instant = std::time::Instant::now();
    let result: String = masked(&line);
    assert!(started.elapsed() < std::time::Duration::from_secs(30), "a quadratic scan of this line takes minutes: {:?}", started.elapsed());
    for canary in [CANARY_USER, CANARY_PASS, CANARY_PATH, CANARY_QUERY] {
        assert!(!result.contains(canary), "{canary} survives in a line of 100000 URLs");
    }
    assert!(result.contains(&format!("https://{CREDENTIALS_MASK}@host/{CREDENTIALS_MASK}/lfs?token={CREDENTIALS_MASK} ")), "the masked shape is kept");
}

//endregion
