//! Unit tests of the `tagPrefix` option of the `github:` resolver:
//! one repository that tags the releases of several programs,
//! of which a request names one by the start of its tag.

use super::error::ErrorKind;
use super::fakes::Rig;
use super::hash::sha256_hex;
use super::model::Declaration;
use super::plan::parse_plan;
use super::update_github::{PREFIX_SEARCH_PAGES, RELEASES_PER_PAGE, resolve_github};

/// The prefix of the program the tests ask for.
const PREFIX: &str = "tool-v";

/// A declaration that carries the prefix.
fn prefixed(rig: &mut Rig) -> Declaration {
    rig.lock.tools[2].declaration.options.tag_prefix = Some(PREFIX.to_string());
    return rig.tool("pnpm").declaration;
}

/// One release object of the list:
/// its tag,
/// its flags and one Linux asset.
fn listed(tag: &str, draft: bool, prerelease: bool) -> String {
    let name: String = format!("{tag}-linux-x64.tar.gz");
    return format!(
        "{{\"tag_name\": \"{tag}\", \"draft\": {draft}, \"prerelease\": {prerelease}, \"assets\": [{{\"name\": \"{name}\", \"browser_download_url\": \"https://github.com/o/r/releases/download/{tag}/{name}\", \"digest\": \"sha256:{}\"}}]}}",
        sha256_hex(name.as_bytes())
    );
}

#[test]
fn the_newest_release_of_the_named_program_wins_over_newer_releases_of_others() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    let list: Vec<String> = vec![
        listed("other-v9.0.0", false, false),
        listed("tool-v3.0.0-rc.1", false, true),
        listed("tool-v2.5.0", true, false),
        listed("tool-vnext", false, false),
        listed("tool-v2.1", false, false),
        listed("tool-v2.1.0", false, false),
        listed("tool-v2.0.0", false, false),
    ];
    let url = page_url(1);
    rig.http.serve(&url, format!("[{}]", list.join(", ")).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved");
    assert_eq!(resolution.version, "2.1.0", "prerelease, draft, other programs and a non-version tail are skipped");
    let linux_x64 = resolution.artifacts.iter().find(|(platform, _)| return platform == "linux-x64").expect("row");
    assert_eq!(linux_x64.1.url, "https://github.com/o/r/releases/download/tool-v2.1.0/tool-v2.1.0-linux-x64.tar.gz");
    assert_eq!(rig.http.requests(), vec![url.to_string()], "the page was short, so it was the last one");
}

#[test]
fn an_alpha_release_that_github_does_not_flag_as_a_prerelease_beats_the_older_stable_release() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    // `gh release create` without `--prerelease` (what `.github/workflows/cargo-publish.yml` runs) leaves the flag false.
    let list: Vec<String> = vec![
        listed("other-v9.0.0", false, false),
        listed("tool-v2.2.0-alpha.1", false, false),
        listed("tool-v2.1.0", false, false),
    ];
    let url = page_url(1);
    rig.http.serve(&url, format!("[{}]", list.join(", ")).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved");
    assert_eq!(resolution.version, "2.2.0-alpha.1", "an unflagged alpha of a higher core version is the newest release");
    let linux_x64 = resolution.artifacts.iter().find(|(platform, _)| return platform == "linux-x64").expect("row");
    assert_eq!(
        linux_x64.1.url,
        "https://github.com/o/r/releases/download/tool-v2.2.0-alpha.1/tool-v2.2.0-alpha.1-linux-x64.tar.gz"
    );
    let mut flagged = Rig::new();
    let flagged_declaration = prefixed(&mut flagged);
    let flagged_list: Vec<String> = vec![listed("tool-v2.2.0-alpha.1", false, true), listed("tool-v2.1.0", false, false)];
    flagged.http.serve(&url, format!("[{}]", flagged_list.join(", ")).as_bytes());
    assert_eq!(
        resolve_github(&flagged.context(), &flagged_declaration, "o/r", None).expect("resolved").version,
        "2.1.0",
        "the same alpha flagged as a prerelease is skipped"
    );
}

#[test]
fn a_later_alpha_number_wins_when_it_has_more_digits() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    let list: Vec<String> = vec![
        listed("tool-v2.2.0-alpha.2", false, false),
        listed("tool-v2.2.0-alpha.10", false, false),
        listed("tool-v2.2.0-alpha.9", false, false),
    ];
    rig.http.serve(&page_url(1), format!("[{}]", list.join(", ")).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved");
    assert_eq!(resolution.version, "2.2.0-alpha.10", "semver compares the identifier after alpha as a number");
}

#[test]
fn a_match_on_a_later_page_is_found_and_the_search_stops_when_a_page_is_short() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    let mut first: Vec<String> = Vec::new();
    for number in 0..RELEASES_PER_PAGE {
        first.push(listed(&format!("other-v1.0.{number}"), false, false));
    }
    rig.http.serve(&page_url(1), format!("[{}]", first.join(", ")).as_bytes());
    rig.http.serve(&page_url(2), format!("[{}]", listed("tool-v1.0.0", false, false)).as_bytes());
    assert_eq!(resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved").version, "1.0.0");
    let mut empty = Rig::new();
    let empty_declaration = prefixed(&mut empty);
    empty.http.serve(&page_url(1), format!("[{}]", first.join(", ")).as_bytes());
    empty.http.serve(&page_url(2), b"[]");
    let error = resolve_github(&empty.context(), &empty_declaration, "o/r", None).expect_err("no release of the program");
    assert!(error.message().contains("\"tool-v\""), "{error:?}");
    assert_eq!(empty.http.requests().len(), 2, "page 1 was full, page 2 was empty, so the search ended");
}

#[test]
fn a_repository_with_no_release_of_the_program_fails_naming_the_prefix() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    rig.http.serve(
        &page_url(1),
        format!("[{}, {}]", listed("other-v1.0.0", false, false), listed("tool-v1.0.0-beta", false, true)).as_bytes(),
    );
    let error = resolve_github(&rig.context(), &declaration, "o/r", None).expect_err("only a prerelease and another program");
    assert!(error.message().contains("tool-v"), "{error:?}");
    rig.http.serve(&page_url(1), b"{\"message\": \"not a list\"}");
    assert!(resolve_github(&rig.context(), &declaration, "o/r", None).is_err());
}

#[test]
fn a_pinned_range_with_a_prefix_asks_for_the_prefixed_tag_as_written_then_with_a_v() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    let tag = "tool-v1.2.3";
    rig.http.serve(&format!("https://api.github.com/repos/o/r/releases/tags/{tag}"), listed(tag, false, false).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", Some("1.2.3")).expect("resolved");
    assert_eq!(resolution.version, "1.2.3");
    assert_eq!(
        rig.http.requests(),
        vec!["https://api.github.com/repos/o/r/releases/tags/tool-v1.2.3".to_string()],
        "the tag in the repository's own spelling is the first the resolver asks for"
    );
    let mut other = Rig::new();
    let without = prefixed(&mut other);
    assert!(resolve_github(&other.context(), &without, "o/r", Some("9.9.9")).is_err());
    assert_eq!(
        other.http.requests(),
        vec!["https://api.github.com/repos/o/r/releases/tags/tool-v9.9.9".to_string()],
        "a prefix that ends in v has spelled the v out, so there is no second spelling to ask for"
    );
    let mut plain = Rig::new();
    plain.lock.tools[2].declaration.options.tag_prefix = Some("tool-".to_string());
    let plain_declaration = plain.tool("pnpm").declaration;
    assert!(resolve_github(&plain.context(), &plain_declaration, "o/r", Some("9.9.9")).is_err());
    assert_eq!(
        plain.http.requests(),
        vec![
            "https://api.github.com/repos/o/r/releases/tags/tool-9.9.9".to_string(),
            "https://api.github.com/repos/o/r/releases/tags/tool-v9.9.9".to_string(),
        ]
    );
}

#[test]
fn the_plan_accepts_a_tag_prefix_and_refuses_one_that_is_not_the_start_of_a_tag() {
    let accepted = r#"{"schemaVersion": 1, "tools": {"mpm-shadow": {"request": "github:o/r@latest", "route": "archive", "bins": ["x"], "options": {"tagPrefix": "tool-v"}}}}"#;
    let plan = parse_plan(accepted, "toolchain.plan.jsonc").expect("parses");
    assert_eq!(plan.tools[0].options.tag_prefix.as_deref(), Some("tool-v"));
    for bad in ["", "a b", "a/b", "tool-v*", "tool\\nv"] {
        let source = accepted.replace("tool-v", bad);
        let error = parse_plan(&source, "toolchain.plan.jsonc").expect_err(bad);
        assert_eq!(error.kind, ErrorKind::Config, "{bad:?}");
        assert!(error.message.contains("tagPrefix"), "{error}");
    }
}

/// The URL of one page of the release list.
fn page_url(page: u32) -> String {
    return format!("https://api.github.com/repos/o/r/releases?per_page={RELEASES_PER_PAGE}&page={page}");
}

/// A full page of releases of another program.
fn full_page_of_other_releases() -> String {
    let entries: Vec<String> = (0..RELEASES_PER_PAGE).map(|number| return listed(&format!("other-v1.0.{number}"), false, false)).collect();
    return format!("[{}]", entries.join(", "));
}

#[test]
fn the_highest_version_wins_over_a_backport_released_later() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    // The list is ordered by creation: the 1.9.9 backport was made after 2.1.0.
    let list: Vec<String> = vec![
        listed("tool-v1.9.9", false, false),
        listed("other-v9.0.0", false, false),
        listed("tool-v2.1.0", false, false),
        listed("tool-v2.0.0", false, false),
    ];
    rig.http.serve(&page_url(1), format!("[{}]", list.join(", ")).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved");
    assert_eq!(resolution.version, "2.1.0", "the first match in list order is not the answer");
    let linux_x64 = resolution.artifacts.iter().find(|(platform, _)| return platform == "linux-x64").expect("row");
    assert!(linux_x64.1.url.contains("tool-v2.1.0"), "{}", linux_x64.1.url);
}

#[test]
fn a_higher_version_on_a_later_page_beats_a_match_on_an_earlier_one() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    let mut first: Vec<String> = vec![listed("tool-v1.0.5", false, false)];
    for number in 1..RELEASES_PER_PAGE {
        first.push(listed(&format!("other-v1.0.{number}"), false, false));
    }
    rig.http.serve(&page_url(1), format!("[{}]", first.join(", ")).as_bytes());
    rig.http.serve(&page_url(2), format!("[{}]", listed("tool-v1.2.0", false, false)).as_bytes());
    assert_eq!(resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved").version, "1.2.0");
    assert_eq!(rig.http.requests(), vec![page_url(1), page_url(2)], "a full page is followed by the next, a short page ends it");
}

#[test]
fn a_program_whose_name_extends_the_prefix_is_not_matched_and_the_tail_must_be_a_whole_version() {
    let mut rig = Rig::new();
    rig.lock.tools[2].declaration.options.tag_prefix = Some("tool-".to_string());
    let declaration = rig.tool("pnpm").declaration;
    let list: Vec<String> = vec![
        listed("tool-2-v9.0.0", false, false),
        listed("tool-extra-v8.0.0", false, false),
        listed("tool-v1.0", false, false),
        listed("tool-v1.0.0.4", false, false),
        listed("tool-vv7.0.0", false, false),
        listed("tool-v1.0.0", false, false),
        listed("tool-0.9.0", false, false),
    ];
    rig.http.serve(&page_url(1), format!("[{}]", list.join(", ")).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved");
    assert_eq!(resolution.version, "1.0.0", "a prefix that does not end in v takes an optional v, and nothing else");

    // A prefix that ends in v has spelled it out: no second v is allowed.
    let mut spelled = Rig::new();
    let spelled_declaration = prefixed(&mut spelled);
    spelled.http.serve(&page_url(1), format!("[{}, {}]", listed("tool-vv7.0.0", false, false), listed("tool-v1.0.0", false, false)).as_bytes());
    assert_eq!(resolve_github(&spelled.context(), &spelled_declaration, "o/r", None).expect("resolved").version, "1.0.0");
}

#[test]
fn a_search_that_reaches_its_cap_says_so_and_one_that_finds_a_release_does_not() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    for page in 1..=PREFIX_SEARCH_PAGES {
        rig.http.serve(&page_url(page), full_page_of_other_releases().as_bytes());
    }
    let error = resolve_github(&rig.context(), &declaration, "o/r", None).expect_err("ten full pages, none of the program");
    let cap = format!(
        "stopped at its cap of {PREFIX_SEARCH_PAGES} pages ({} releases)",
        PREFIX_SEARCH_PAGES as usize * RELEASES_PER_PAGE
    );
    assert!(error.message().contains(&cap), "{error:?}");
    assert!(error.message().contains("\"tool-v\""), "{error:?}");
    assert_eq!(rig.http.requests().len(), usize::try_from(PREFIX_SEARCH_PAGES).expect("the cap fits a length"), "page 11 is never asked for");

    // The same ten pages with a release of the program on the last one: found, and no cap is reported.
    let mut found = Rig::new();
    let found_declaration = prefixed(&mut found);
    for page in 1..PREFIX_SEARCH_PAGES {
        found.http.serve(&page_url(page), full_page_of_other_releases().as_bytes());
    }
    let mut last: Vec<String> = vec![listed("tool-v3.1.4", false, false)];
    for number in 1..RELEASES_PER_PAGE {
        last.push(listed(&format!("other-v2.0.{number}"), false, false));
    }
    found.http.serve(&page_url(PREFIX_SEARCH_PAGES), format!("[{}]", last.join(", ")).as_bytes());
    assert_eq!(resolve_github(&found.context(), &found_declaration, "o/r", None).expect("resolved").version, "3.1.4");

    // A search that ends on a short page before the cap does not claim the cap.
    let mut short = Rig::new();
    let short_declaration = prefixed(&mut short);
    short.http.serve(&page_url(1), full_page_of_other_releases().as_bytes());
    short.http.serve(&page_url(2), format!("[{}]", listed("other-v1.0.0", false, false)).as_bytes());
    let short_error = resolve_github(&short.context(), &short_declaration, "o/r", None).expect_err("none");
    assert!(!short_error.message().contains("cap"), "{short_error:?}");
}

#[test]
fn a_pinned_range_whose_tag_does_not_continue_the_prefix_with_a_version_is_refused() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    rig.http.serve("https://api.github.com/repos/o/r/releases/tags/tool-vnext", listed("tool-vnext", false, false).as_bytes());
    let error = resolve_github(&rig.context(), &declaration, "o/r", Some("next")).expect_err("not a version");
    assert!(error.message().contains("does not continue the tagPrefix"), "{error:?}");
}
