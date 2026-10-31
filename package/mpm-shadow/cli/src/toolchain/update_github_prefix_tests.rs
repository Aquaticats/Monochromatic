//! Unit tests of the `tagPrefix` option of the `github:` resolver:
//! one repository that tags the releases of several programs,
//! of which a request names one by the start of its tag.

use super::error::ErrorKind;
use super::fakes::Rig;
use super::hash::sha256_hex;
use super::model::Declaration;
use super::plan::parse_plan;
use super::update_github::resolve_github;

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
        listed("tool-v2.1.0", false, false),
        listed("tool-v2.0.0", false, false),
    ];
    let url = "https://api.github.com/repos/o/r/releases?per_page=100&page=1";
    rig.http.serve(url, format!("[{}]", list.join(", ")).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved");
    assert_eq!(resolution.version, "2.1.0", "prerelease, draft, other programs and a non-version tail are skipped");
    let linux_x64 = resolution.artifacts.iter().find(|(platform, _)| return platform == "linux-x64").expect("row");
    assert_eq!(linux_x64.1.url, "https://github.com/o/r/releases/download/tool-v2.1.0/tool-v2.1.0-linux-x64.tar.gz");
    assert_eq!(rig.http.requests(), vec![url.to_string()], "the first page held a match, so no later page is read");
}

#[test]
fn a_match_on_a_later_page_is_found_and_the_search_stops_when_a_page_is_short() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    let mut first: Vec<String> = Vec::new();
    for number in 0..100 {
        first.push(listed(&format!("other-v1.0.{number}"), false, false));
    }
    rig.http.serve("https://api.github.com/repos/o/r/releases?per_page=100&page=1", format!("[{}]", first.join(", ")).as_bytes());
    rig.http.serve("https://api.github.com/repos/o/r/releases?per_page=100&page=2", format!("[{}]", listed("tool-v1.0.0", false, false)).as_bytes());
    assert_eq!(resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved").version, "1.0.0");
    let mut empty = Rig::new();
    let empty_declaration = prefixed(&mut empty);
    empty.http.serve("https://api.github.com/repos/o/r/releases?per_page=100&page=1", format!("[{}]", first.join(", ")).as_bytes());
    empty.http.serve("https://api.github.com/repos/o/r/releases?per_page=100&page=2", b"[]");
    let error = resolve_github(&empty.context(), &empty_declaration, "o/r", None).expect_err("no release of the program");
    assert!(error.message().contains("\"tool-v\""), "{error:?}");
    assert_eq!(empty.http.requests().len(), 2, "page 1 was full, page 2 was empty, so the search ended");
}

#[test]
fn a_repository_with_no_release_of_the_program_fails_naming_the_prefix() {
    let mut rig = Rig::new();
    let declaration = prefixed(&mut rig);
    rig.http.serve(
        "https://api.github.com/repos/o/r/releases?per_page=100&page=1",
        format!("[{}, {}]", listed("other-v1.0.0", false, false), listed("tool-v1.0.0-beta", false, true)).as_bytes(),
    );
    let error = resolve_github(&rig.context(), &declaration, "o/r", None).expect_err("only a prerelease and another program");
    assert!(error.message().contains("tool-v"), "{error:?}");
    rig.http.serve("https://api.github.com/repos/o/r/releases?per_page=100&page=1", b"{\"message\": \"not a list\"}");
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
