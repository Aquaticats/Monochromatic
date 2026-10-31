//! Unit tests of the resolvers and of `update`, against a table-driven fake network and a recording fake runner. No
//! test reaches a registry or starts a package manager.

use std::fs;
use std::path::PathBuf;

use super::error::ErrorKind;
use super::fakes::{FakeHttp, FakeRunner, Rig, write_executable};
use super::fixtures::{SAMPLE_SHA256, sample_lock, sample_scope_on_disk};
use super::hash::sha256_hex;
use super::http::{HttpError, HttpGet, HttpResponse};
use super::layout::tool_install_dir;
use super::lock::{Lock, parse_lock, render_lock};
use super::lock_cmd::{LockMode, run_lock};
use super::model::{Artifact, Request};
use super::scope::{EnvMap, Scope};
use super::update::{OutcomeKind, UpdateReport, format_update_report, run_update};
use super::update_github::{glob_matches, parse_release_download, resolve_github, version_from_tag};
use super::update_registry::{registry_path_name, resolve_crate, resolve_npm, resolve_pypi, resolve_pypi_version};
use super::update_resolve::{ResolveError, resolve_tool};
use super::update_rust_node::{parse_shasums, resolve_node, resolve_rust};

const NPM_URL: &str = "https://registry.npmjs.org/wrangler";
const PYPI_URL: &str = "https://pypi.org/pypi/slopo/json";
const CRATES_URL: &str = "https://crates.io/api/v1/crates/fd-find";
const RUST_URL: &str = "https://static.rust-lang.org/dist/channel-rust-nightly.toml";

/// A plan of four tools resolved by registries and the Rust manifest, with `{version}` placeholders for the toolchain.
const REGISTRY_PLAN: &str = r#"{
  "schemaVersion": 1,
  "tools": {
    "cargo:fd-find": { "request": { "track": "latest" }, "route": "cargo-binstall", "manager": "cargo", "packageId": "fd-find", "bins": ["fd"] },
    "npm:wrangler": { "request": { "track": "latest" }, "route": "mpm-pnpm", "manager": "pnpm", "packageId": "wrangler", "bins": ["wrangler"] },
    "pipx:slopo": { "request": { "track": "latest" }, "route": "mpm-uvx", "manager": "uvx", "packageId": "slopo", "bins": ["slopo"] },
    "rust": {
      "request": { "channel": "nightly" }, "route": "rustup-toolchain", "manager": "rustup", "packageId": "{version}",
      "bins": ["rustc"], "env": { "RUSTUP_TOOLCHAIN": "{version}" }
    },
  },
}
"#;

fn serve_registries(http: &FakeHttp, crate_version: &str, npm_version: &str, pypi_version: &str, nightly_date: &str) {
    http.serve(CRATES_URL, format!("{{\"crate\": {{\"max_stable_version\": \"{crate_version}\"}}}}").as_bytes());
    http.serve(NPM_URL, format!("{{\"dist-tags\": {{\"latest\": \"{npm_version}\", \"next\": \"9.0.0-beta\"}}}}").as_bytes());
    http.serve(PYPI_URL, format!("{{\"info\": {{\"version\": \"{pypi_version}\"}}}}").as_bytes());
    http.serve(RUST_URL, format!("date = \"{nightly_date}\"\n[pkg.rust]\nversion = \"1.103.0-nightly (abc 2026-10-29)\"\n").as_bytes());
}

fn registry_scope() -> (tempfile::TempDir, Scope, EnvMap) {
    let (directory, scope, env) = sample_scope_on_disk();
    fs::write(&scope.plan_file, REGISTRY_PLAN).expect("write the plan");
    fs::remove_file(&scope.lock_file).expect("start without a lock");
    return (directory, scope, env);
}

fn linux() -> super::context::Host {
    return super::context::Host { platform: "linux-x64".to_string(), family: Some(super::model::OsFamily::Linux) };
}

fn outcome_of<'a>(report: &'a UpdateReport, id: &str) -> &'a OutcomeKind {
    return &report.outcomes.iter().find(|outcome| return outcome.id == id).expect("an outcome for the tool").kind;
}

fn read_lock(scope: &Scope) -> Lock {
    return parse_lock(&fs::read_to_string(&scope.lock_file).expect("read the lock"), "lock").expect("parses").lock;
}

//region resolvers

#[test]
fn package_names_are_checked_before_they_reach_a_url() {
    assert_eq!(registry_path_name("wrangler").expect("fine"), "wrangler");
    assert_eq!(registry_path_name("@actions/languageserver").expect("scoped"), "@actions%2Flanguageserver");
    for bad in ["", "a b", "a?x=1", "a#b", "../etc", "a\nb", "a%2Fb"] {
        assert!(registry_path_name(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn npm_is_asked_at_registry_npmjs_org_directly_for_the_latest_dist_tag() {
    let rig = Rig::new();
    serve_registries(&rig.http, "1.0.0", "4.156.0", "0.9.0", "2026-10-30");
    let declaration = rig.tool("npm:wrangler").declaration;
    let resolution = resolve_npm(&rig.context(), &declaration).expect("resolved");
    assert_eq!(resolution.version, "4.156.0");
    assert!(resolution.artifacts.is_empty());
    assert_eq!(rig.http.requests(), vec![NPM_URL.to_string()]);
    assert!(rig.runner.calls().is_empty(), "no package manager is asked");
}

#[test]
fn a_scoped_npm_package_is_requested_with_an_escaped_slash_and_a_channel_picks_its_tag() {
    let mut rig = Rig::new();
    rig.lock.tools[8].declaration.package_id = Some("@cloudflare/wrangler".to_string());
    rig.lock.tools[8].declaration.request = Request::Channel("next".to_string());
    rig.http.serve("https://registry.npmjs.org/@cloudflare%2Fwrangler", b"{\"dist-tags\": {\"latest\": \"4.0.0\", \"next\": \"5.0.0-rc.1\"}}");
    let declaration = rig.tool("npm:wrangler").declaration;
    assert_eq!(resolve_npm(&rig.context(), &declaration).expect("resolved").version, "5.0.0-rc.1");
}

#[test]
fn npm_without_the_dist_tag_names_the_url_and_the_tag() {
    let rig = Rig::new();
    rig.http.serve(NPM_URL, b"{\"dist-tags\": {}}");
    let declaration = rig.tool("npm:wrangler").declaration;
    let error = resolve_npm(&rig.context(), &declaration).expect_err("no tag");
    assert!(error.message().contains(NPM_URL), "{error:?}");
    assert!(error.message().contains("no latest dist-tag"), "{error:?}");
}

#[test]
fn a_registry_error_status_is_reported_with_the_url_and_the_status() {
    let rig = Rig::new();
    let declaration = rig.tool("npm:wrangler").declaration;
    let error = resolve_npm(&rig.context(), &declaration).expect_err("the fake answers 404");
    assert_eq!(error, ResolveError::Failed(format!("{NPM_URL}: HTTP 404")));
}

#[test]
fn a_network_failure_is_reported_with_the_url() {
    let rig = Rig::new();
    rig.http.fail(PYPI_URL, "no route to host");
    let declaration = rig.tool("pipx:slopo").declaration;
    let error = resolve_pypi(&rig.context(), &declaration, None).expect_err("network down");
    assert_eq!(error, ResolveError::Failed(format!("{PYPI_URL}: no route to host")));
}

#[test]
fn an_answer_that_is_not_json_names_the_url() {
    let rig = Rig::new();
    rig.http.serve(PYPI_URL, b"<html>maintenance</html>");
    let declaration = rig.tool("pipx:slopo").declaration;
    let error = resolve_pypi(&rig.context(), &declaration, None).expect_err("not JSON");
    assert!(error.message().starts_with(&format!("{PYPI_URL}: the answer is not JSON")), "{error:?}");
}

#[test]
fn pypi_gives_info_version_and_crates_io_gives_the_max_stable_version() {
    let rig = Rig::new();
    serve_registries(&rig.http, "10.5.0", "4.156.0", "0.9.1", "2026-10-30");
    let slopo = rig.tool("pipx:slopo").declaration;
    assert_eq!(resolve_pypi(&rig.context(), &slopo, None).expect("resolved").version, "0.9.1");
    let fd = rig.tool("cargo:fd-find").declaration;
    assert_eq!(resolve_crate(&rig.context(), &fd).expect("resolved").version, "10.5.0");
}

#[test]
fn a_wheel_bootstrap_records_the_wheel_url_and_hash_for_the_locked_platforms() {
    let mut rig = Rig::new();
    rig.lock.tools[9].declaration.options.bootstrap =
        Some(super::model::Bootstrap { requires_tool: Some("uv".to_string()), distribution: "wheel".to_string() });
    let wheel_hash = "ab".repeat(32);
    rig.http.serve(
        "https://pypi.org/pypi/slopo/8.2.0/json",
        format!(
            "{{\"urls\": [{{\"packagetype\": \"sdist\", \"url\": \"https://f/slopo.tar.gz\", \"digests\": {{\"sha256\": \"{0}\"}}}}, \
             {{\"packagetype\": \"bdist_wheel\", \"url\": \"https://files.pythonhosted.org/slopo-8.2.0-py3-none-any.whl\", \"digests\": {{\"sha256\": \"{wheel_hash}\"}}}}]}}",
            "cd".repeat(32)
        )
        .as_bytes(),
    );
    let declaration = rig.tool("pipx:slopo").declaration;
    let mut current = rig.tool("pipx:slopo");
    current.artifacts = vec![
        ("linux-x64".to_string(), Artifact { url: "old".to_string(), sha256: SAMPLE_SHA256.to_string() }),
        ("linux-arm64".to_string(), Artifact { url: "old".to_string(), sha256: SAMPLE_SHA256.to_string() }),
    ];
    let resolution = resolve_pypi_version(&rig.context(), &declaration, Some(&current), "8.2.0").expect("resolved");
    assert_eq!(resolution.version, "8.2.0");
    assert_eq!(resolution.artifacts.len(), 2);
    assert_eq!(resolution.artifacts[0].0, "linux-x64");
    assert_eq!(resolution.artifacts[0].1.url, "https://files.pythonhosted.org/slopo-8.2.0-py3-none-any.whl");
    assert_eq!(resolution.artifacts[0].1.sha256, wheel_hash);
}

#[test]
fn a_wheel_bootstrap_with_no_pure_python_wheel_fails() {
    let mut rig = Rig::new();
    rig.lock.tools[9].declaration.options.bootstrap =
        Some(super::model::Bootstrap { requires_tool: None, distribution: "wheel".to_string() });
    rig.http.serve(PYPI_URL, b"{\"info\": {\"version\": \"1.0.0\"}, \"urls\": []}");
    let declaration = rig.tool("pipx:slopo").declaration;
    let error = resolve_pypi(&rig.context(), &declaration, None).expect_err("no wheel");
    assert!(error.message().contains("no pure-Python wheel"), "{error:?}");
}

#[test]
fn the_nightly_channel_gives_a_dated_name_and_a_stable_channel_gives_its_version() {
    let mut rig = Rig::new();
    serve_registries(&rig.http, "1", "1", "1", "2026-11-02");
    let declaration = rig.tool("rust").declaration;
    assert_eq!(resolve_rust(&rig.context(), &declaration).expect("resolved").version, "nightly-2026-11-02");
    rig.lock.tools[4].declaration.request = Request::Channel("stable".to_string());
    rig.http.serve(
        "https://static.rust-lang.org/dist/channel-rust-stable.toml",
        b"[pkg.cargo]\nversion = \"0.1\"\n[pkg.rust]\nversion = \"1.99.0 (abcdef 2026-10-01)\"\n",
    );
    let stable = rig.tool("rust").declaration;
    assert_eq!(resolve_rust(&rig.context(), &stable).expect("resolved").version, "1.99.0");
}

#[test]
fn a_rust_manifest_without_a_date_or_a_channel_name_with_odd_characters_fails() {
    let mut rig = Rig::new();
    rig.http.serve(RUST_URL, b"[pkg.rust]\nversion = \"1.0.0\"\n");
    let declaration = rig.tool("rust").declaration;
    assert!(resolve_rust(&rig.context(), &declaration).expect_err("no date").message().contains("carries no date"));
    rig.lock.tools[4].declaration.request = Request::Channel("../../x".to_string());
    let odd = rig.tool("rust").declaration;
    assert!(resolve_rust(&rig.context(), &odd).is_err());
    assert_eq!(rig.http.requests().len(), 1, "an odd channel name is refused before any request");
}

#[test]
fn shasums_files_are_read_line_by_line_and_malformed_lines_are_ignored() {
    let digest_a = "a".repeat(64);
    let digest_b = "b".repeat(64);
    let text = format!("{digest_a}  node-v1-linux-x64.tar.gz\n{digest_b} *node-v1-win.zip\nnot a digest line\n{digest_a}node-glued.tgz\n\n");
    let sums = parse_shasums(&text);
    assert_eq!(sums.len(), 2);
    assert_eq!(sums.get("node-v1-linux-x64.tar.gz"), Some(&digest_a));
    assert_eq!(sums.get("node-v1-win.zip"), Some(&digest_b));
}

#[test]
fn node_follows_the_latest_stable_release_and_its_checksums() {
    let mut rig = Rig::new();
    rig.lock.tools[3].version = "26.13.0".to_string();
    rig.lock.tools[3].artifacts = vec![
        ("linux-x64".to_string(), Artifact { url: "https://nodejs.org/dist/v26.13.0/node-v26.13.0-linux-x64.tar.gz".to_string(), sha256: SAMPLE_SHA256.to_string() }),
        (
            "linux-x64-musl".to_string(),
            Artifact {
                url: "https://unofficial-builds.nodejs.org/download/release/v26.13.0/node-v26.13.0-linux-x64-musl.tar.gz".to_string(),
                sha256: SAMPLE_SHA256.to_string(),
            },
        ),
    ];
    rig.http.serve("https://nodejs.org/dist/index.json", b"[{\"version\": \"v27.0.0-rc.1\"}, {\"version\": \"v26.14.0\"}, {\"version\": \"v26.13.0\"}]");
    let official = "1".repeat(64);
    let musl = "2".repeat(64);
    rig.http.serve("https://nodejs.org/dist/v26.14.0/SHASUMS256.txt", format!("{official}  node-v26.14.0-linux-x64.tar.gz\n").as_bytes());
    rig.http.serve(
        "https://unofficial-builds.nodejs.org/download/release/v26.14.0/SHASUMS256.txt",
        format!("{musl}  node-v26.14.0-linux-x64-musl.tar.gz\n").as_bytes(),
    );
    let current = rig.tool("node");
    let resolution = resolve_node(&rig.context(), &current.declaration, Some(&current)).expect("resolved");
    assert_eq!(resolution.version, "26.14.0");
    assert_eq!(resolution.artifacts[0].1.url, "https://nodejs.org/dist/v26.14.0/node-v26.14.0-linux-x64.tar.gz");
    assert_eq!(resolution.artifacts[0].1.sha256, official);
    assert_eq!(resolution.artifacts[1].1.sha256, musl);
}

#[test]
fn node_fails_naming_the_archive_when_the_checksums_lack_it() {
    let mut rig = Rig::new();
    rig.lock.tools[3].version = "26.13.0".to_string();
    rig.lock.tools[3].artifacts =
        vec![("linux-x64".to_string(), Artifact { url: "https://nodejs.org/dist/v26.13.0/node-v26.13.0-linux-x64.tar.gz".to_string(), sha256: SAMPLE_SHA256.to_string() })];
    rig.http.serve("https://nodejs.org/dist/index.json", b"[{\"version\": \"v26.14.0\"}]");
    let current = rig.tool("node");
    let error = resolve_node(&rig.context(), &current.declaration, Some(&current)).expect_err("no checksum row");
    assert!(error.message().contains("node-v26.14.0-linux-x64.tar.gz"), "{error:?}");
}

#[test]
fn glob_patterns_match_with_star_wildcards_only() {
    assert!(glob_matches("tool-*-linux.tar.gz", "tool-1.2.3-linux.tar.gz"));
    assert!(glob_matches("*", "anything"));
    assert!(glob_matches("a*b*c", "a--b--c"));
    assert!(!glob_matches("a*b*c", "a--c--b"));
    assert!(!glob_matches("tool-*", "other-1"));
    assert!(glob_matches("exact", "exact"));
    assert!(!glob_matches("exact", "exactly"));
}

#[test]
fn release_urls_and_tags_give_the_repository_and_the_version_shape() {
    assert_eq!(
        parse_release_download("https://github.com/o/r/releases/download/bun-v1.3.14/bun-linux-x64.zip"),
        Some(("o/r".to_string(), "bun-v1.3.14".to_string()))
    );
    assert_eq!(parse_release_download("https://example.com/o/r/releases/download/v1/a"), None);
    assert_eq!(version_from_tag("v1.2.3", "1.2.3", "v1.3.0").expect("derived"), "1.3.0");
    assert_eq!(version_from_tag("bun-v1.3.14", "1.3.14", "bun-v1.4.0").expect("derived"), "1.4.0");
    assert_eq!(version_from_tag("2026-02-08", "2026-02-08", "2026-03-01").expect("no prefix"), "2026-03-01");
    assert!(version_from_tag("v1.2.3", "1.2.3", "release-2.0").is_err());
    assert!(version_from_tag("v1.2.3", "9.9.9", "v2.0.0").is_err());
}

fn github_tool(rig: &mut Rig) -> super::model::LockedTool {
    rig.env = rig.env.with("GITHUB_TOKEN", "gh-token");
    rig.lock.tools[2].version = "1.2.3".to_string();
    rig.lock.tools[2].artifacts = vec![(
        "linux-x64".to_string(),
        Artifact { url: "https://github.com/o/r/releases/download/v1.2.3/tool-1.2.3-linux-x64.tar.gz".to_string(), sha256: SAMPLE_SHA256.to_string() },
    )];
    return rig.tool("pnpm");
}

#[test]
fn a_github_tool_follows_its_latest_release_and_takes_the_hash_from_the_asset_digest() {
    let mut rig = Rig::new();
    let current = github_tool(&mut rig);
    let digest = "9".repeat(64);
    rig.http.serve(
        "https://api.github.com/repos/o/r/releases/latest",
        format!(
            "{{\"tag_name\": \"v1.3.0\", \"assets\": [{{\"name\": \"tool-1.3.0-linux-x64.tar.gz\", \
             \"browser_download_url\": \"https://github.com/o/r/releases/download/v1.3.0/tool-1.3.0-linux-x64.tar.gz\", \
             \"digest\": \"sha256:{digest}\"}}]}}"
        )
        .as_bytes(),
    );
    let resolution = resolve_github(&rig.context(), &current.declaration, Some(&current)).expect("resolved");
    assert_eq!(resolution.version, "1.3.0");
    assert_eq!(resolution.artifacts[0].1.url, "https://github.com/o/r/releases/download/v1.3.0/tool-1.3.0-linux-x64.tar.gz");
    assert_eq!(resolution.artifacts[0].1.sha256, digest);
    assert_eq!(rig.http.requests(), vec!["https://api.github.com/repos/o/r/releases/latest".to_string()]);
}

#[test]
fn a_github_asset_without_a_digest_is_downloaded_and_hashed() {
    let mut rig = Rig::new();
    let current = github_tool(&mut rig);
    let asset_url = "https://github.com/o/r/releases/download/v1.3.0/tool-1.3.0-linux-x64.tar.gz";
    rig.http.serve(
        "https://api.github.com/repos/o/r/releases/latest",
        format!("{{\"tag_name\": \"v1.3.0\", \"assets\": [{{\"name\": \"tool-1.3.0-linux-x64.tar.gz\", \"browser_download_url\": \"{asset_url}\"}}]}}").as_bytes(),
    );
    rig.http.serve(asset_url, b"the asset bytes");
    let resolution = resolve_github(&rig.context(), &current.declaration, Some(&current)).expect("resolved");
    assert_eq!(resolution.artifacts[0].1.sha256, sha256_hex(b"the asset bytes"));
    let leftovers: Vec<_> = fs::read_dir(rig.scope.toolchain_root.join("downloads")).map(|entries| return entries.flatten().collect()).unwrap_or_default();
    assert!(leftovers.is_empty(), "the hashing download is removed: {leftovers:?}");
}

#[test]
fn a_github_tool_already_at_the_latest_tag_is_unchanged_without_asset_lookups() {
    let mut rig = Rig::new();
    let current = github_tool(&mut rig);
    rig.http.serve("https://api.github.com/repos/o/r/releases/latest", b"{\"tag_name\": \"v1.2.3\", \"assets\": []}");
    let resolution = resolve_github(&rig.context(), &current.declaration, Some(&current)).expect("resolved");
    assert_eq!(resolution.version, "1.2.3");
    assert_eq!(resolution.artifacts, current.artifacts);
}

#[test]
fn an_asset_pattern_must_match_exactly_one_asset() {
    let mut rig = Rig::new();
    let mut current = github_tool(&mut rig);
    current.declaration.options.asset_pattern = Some("tool-*-linux*".to_string());
    current.artifacts[0].1.url = "https://github.com/o/r/releases/download/v1.2.3/unrelated-name".to_string();
    rig.http.serve(
        "https://api.github.com/repos/o/r/releases/latest",
        b"{\"tag_name\": \"v1.3.0\", \"assets\": [{\"name\": \"tool-a-linux.tgz\", \"browser_download_url\": \"https://x/a\", \"digest\": \"sha256:aa\"}, \
          {\"name\": \"tool-b-linux.tgz\", \"browser_download_url\": \"https://x/b\", \"digest\": \"sha256:bb\"}]}",
    );
    let error = resolve_github(&rig.context(), &current.declaration, Some(&current)).expect_err("two matches");
    assert!(error.message().contains("matches 2 release assets"), "{error:?}");
}

#[test]
fn a_github_tool_not_in_the_lock_is_not_implemented_and_other_sources_are_named() {
    let rig = Rig::new();
    let declaration = rig.tool("pnpm").declaration;
    let error = resolve_tool(&rig.context(), &declaration, None).expect_err("nothing to derive the repository from");
    assert!(matches!(error, ResolveError::NotImplemented(_)), "{error:?}");
    assert!(error.message().contains("pnpm (route bin-sourced)"), "{error:?}");
    let sample = rig.tool("pnpm");
    let other = resolve_tool(&rig.context(), &sample.declaration, Some(&sample)).expect_err("example.invalid is not GitHub");
    assert!(other.message().contains("a source other than GitHub releases"), "{other:?}");
}

#[test]
fn go_android_and_conda_tools_say_not_implemented_with_their_route() {
    let mut rig = Rig::new();
    let android = rig.tool("android-sdk");
    assert!(resolve_tool(&rig.context(), &android.declaration, Some(&android)).expect_err("no resolver").message().contains("android-sdk"));
    rig.lock.tools[6].declaration.route = super::model::Route::GoInstall;
    let go = rig.tool("cargo:fd-find");
    assert!(resolve_tool(&rig.context(), &go.declaration, Some(&go)).expect_err("no resolver").message().contains("go-install"));
}

//endregion resolvers

//region update

#[test]
fn the_first_update_resolves_every_tool_writes_the_lock_and_passes_the_gate() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    let runner = FakeRunner::new();
    let report = run_update(&scope, &env, &runner, &http, &linux(), false).expect("updated");
    assert!(report.wrote);
    assert!(report.outcomes.iter().all(|outcome| return matches!(outcome.kind, OutcomeKind::Added { .. })), "{:?}", report.outcomes);
    assert!(runner.calls().is_empty(), "nothing was installed");

    let lock = read_lock(&scope);
    assert_eq!(lock.tool("cargo:fd-find").expect("fd").version, "10.5.0");
    assert_eq!(lock.tool("npm:wrangler").expect("wrangler").version, "4.156.0");
    let rust = lock.tool("rust").expect("rust");
    assert_eq!(rust.version, "nightly-2026-10-30");
    assert_eq!(rust.declaration.package_id.as_deref(), Some("nightly-2026-10-30"), "the placeholder is expanded in the lock");
    assert_eq!(rust.declaration.env, vec![("RUSTUP_TOOLCHAIN".to_string(), "nightly-2026-10-30".to_string())]);
    run_lock(&scope, LockMode::Check).expect("the written lock passes the gate");
}

#[test]
fn a_second_update_with_nothing_new_writes_nothing() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("first");
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let report = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("second");
    assert!(!report.wrote);
    assert!(report.outcomes.iter().all(|outcome| return outcome.kind == OutcomeKind::Unchanged), "{:?}", report.outcomes);
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
    assert!(format_update_report(&report, false).contains("is already current"));
}

#[test]
fn a_moved_registry_updates_the_versions_and_the_placeholder_follows_the_toolchain() {
    let (_directory, scope, env) = registry_scope();
    let first = FakeHttp::new();
    serve_registries(&first, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &first, &linux(), false).expect("first");
    let second = FakeHttp::new();
    serve_registries(&second, "10.6.0", "4.156.0", "0.9.0", "2026-11-03");
    let report = run_update(&scope, &env, &FakeRunner::new(), &second, &linux(), false).expect("second");
    assert!(report.wrote);
    assert_eq!(
        outcome_of(&report, "cargo:fd-find"),
        &OutcomeKind::Updated { old_version: "10.5.0".to_string(), new_version: "10.6.0".to_string() }
    );
    assert_eq!(outcome_of(&report, "npm:wrangler"), &OutcomeKind::Unchanged);
    let rust = read_lock(&scope).tool("rust").expect("rust").clone();
    assert_eq!(rust.version, "nightly-2026-11-03");
    assert_eq!(rust.declaration.package_id.as_deref(), Some("nightly-2026-11-03"));
    assert_eq!(rust.declaration.env[0].1, "nightly-2026-11-03");
    run_lock(&scope, LockMode::Check).expect("plan and lock still agree after the toolchain moved");
    let text = format_update_report(&report, false);
    assert!(text.contains("updated   cargo:fd-find  10.5.0 -> 10.6.0"), "{text}");
    assert!(text.contains("wrote toolchain.lock.json (2 tools moved"), "{text}");
}

#[test]
fn check_reports_movement_without_writing_or_starting_anything() {
    let (_directory, scope, env) = registry_scope();
    let first = FakeHttp::new();
    serve_registries(&first, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &first, &linux(), false).expect("first");
    let before = fs::read_to_string(&scope.lock_file).expect("read");
    let second = FakeHttp::new();
    serve_registries(&second, "10.6.0", "4.156.0", "0.9.0", "2026-10-30");
    let runner = FakeRunner::new();
    let report = run_update(&scope, &env, &runner, &second, &linux(), true).expect("checked");
    assert!(report.has_movement());
    assert!(!report.wrote);
    assert_eq!(fs::read_to_string(&scope.lock_file).expect("read"), before);
    assert!(runner.calls().is_empty());
    assert!(format_update_report(&report, true).contains("nothing was written or installed"));
}

#[test]
fn a_failed_lookup_writes_nothing_and_names_the_url() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    http.fail(PYPI_URL, "timed out");
    let report = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("a report");
    assert!(report.has_failures());
    assert!(!report.wrote);
    assert!(!scope.lock_file.exists(), "no lock is written when any lookup failed");
    let text = format_update_report(&report, false);
    assert!(text.contains(&format!("failed    pipx:slopo  {PYPI_URL}: timed out")), "{text}");
    assert!(text.contains("toolchain.lock.json was not written"), "{text}");
}

#[test]
fn a_changed_pin_resolves_to_the_pinned_version_without_a_lookup() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("first");
    fs::write(&scope.plan_file, REGISTRY_PLAN.replace(
        "\"npm:wrangler\": { \"request\": { \"track\": \"latest\" }",
        "\"npm:wrangler\": { \"request\": { \"pinned\": \"3.0.0\" }",
    ))
    .expect("pin wrangler");
    let quiet = FakeHttp::new();
    serve_registries(&quiet, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    let report = run_update(&scope, &env, &FakeRunner::new(), &quiet, &linux(), false).expect("updated");
    assert_eq!(
        outcome_of(&report, "npm:wrangler"),
        &OutcomeKind::Updated { old_version: "4.156.0".to_string(), new_version: "3.0.0".to_string() }
    );
    assert!(!quiet.requests().contains(&NPM_URL.to_string()), "a pin needs no registry lookup");
    run_update(&scope, &env, &FakeRunner::new(), &quiet, &linux(), false).expect("again");
    assert_eq!(*outcome_of(&run_update(&scope, &env, &FakeRunner::new(), &quiet, &linux(), true).expect("checked"), "npm:wrangler"), OutcomeKind::Pinned);
}

#[test]
fn update_refuses_a_hand_edited_lock() {
    let (_directory, scope, env) = registry_scope();
    let http = FakeHttp::new();
    serve_registries(&http, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect("first");
    let text = fs::read_to_string(&scope.lock_file).expect("read");
    fs::write(&scope.lock_file, text.replace("\"version\": \"4.156.0\"", "\"version\": \"0.0.1\"")).expect("hand edit");
    let error = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect_err("the digest fails");
    assert_eq!(error.kind, ErrorKind::Drift);
    assert!(error.message.contains("edited by hand"), "{error}");
    assert!(error.message.contains("delete it and run `mpm-shadow update`"), "{error}");
}

#[test]
fn an_unimplemented_resolver_keeps_a_locked_tool_and_fails_a_new_one() {
    let (_directory, scope, env) = sample_scope_on_disk();
    let http = FakeHttp::new();
    let runner = FakeRunner::new();
    // The sample lock holds android-sdk (no resolver yet) and the rest resolve from absent registries: keep to a plan
    // of the android entry alone.
    let plan = "{ \"schemaVersion\": 1, \"tools\": { \"android-sdk\": { \"request\": { \"track\": \"latest\" }, \"route\": \"android-sdk\", \"bins\": [\"adb\", \"sdkmanager\"], \"env\": { \"ANDROID_HOME\": \"android-sdk/23.0\" }, \"options\": { \"androidPackages\": [\"platforms/android-37.0\", \"platform-tools\"] } } } }";
    fs::write(&scope.plan_file, plan).expect("write the plan");
    let mut lock = sample_lock();
    lock.tools.retain(|tool| return tool.declaration.id == "android-sdk");
    fs::write(&scope.lock_file, render_lock(&lock)).expect("write the lock");
    let kept = run_update(&scope, &env, &runner, &http, &linux(), false).expect("a report");
    match outcome_of(&kept, "android-sdk") {
        OutcomeKind::Skipped { reason } => assert!(reason.contains("android-sdk (route android-sdk)"), "{reason}"),
        other => panic!("expected a skipped tool, found {other:?}"),
    }
    assert!(!kept.has_failures());
    assert!(format_update_report(&kept, false).contains("skipped   android-sdk"));

    fs::remove_file(&scope.lock_file).expect("remove the lock");
    let added = run_update(&scope, &env, &runner, &http, &linux(), false).expect("a report");
    assert!(matches!(outcome_of(&added, "android-sdk"), OutcomeKind::Failed { .. }), "a new tool with no resolver is a failure");
    assert!(!scope.lock_file.exists());
}

/// A scope whose lock holds node and pnpm (pinned archives that a fake `tar` unpacks) and an unlocked-looking npm tool
/// whose registry lookup the test controls.
fn auto_sync_rig() -> (tempfile::TempDir, Scope, EnvMap, FakeHttp, FakeRunner) {
    let (directory, scope, env) = sample_scope_on_disk();
    let plan = "{ \"schemaVersion\": 1, \"tools\": {
      \"node\": { \"request\": { \"pinned\": \"1.2.3\" }, \"route\": \"archive\", \"bins\": [\"node\", \"npm\", \"npx\"] },
      \"pnpm\": { \"request\": { \"pinned\": \"1.2.3\" }, \"route\": \"archive\", \"bins\": [\"pnpm\"] },
      \"npm:wrangler\": { \"request\": { \"track\": \"latest\" }, \"route\": \"mpm-pnpm\", \"manager\": \"pnpm\", \"packageId\": \"wrangler\", \"bins\": [\"wrangler\"] }
    } }";
    fs::write(&scope.plan_file, plan).expect("write the plan");
    let http = FakeHttp::new();
    let mut lock = sample_lock();
    lock.tools.retain(|tool| return ["node", "pnpm", "npm:wrangler"].contains(&tool.declaration.id.as_str()));
    for tool in &mut lock.tools {
        match tool.declaration.id.as_str() {
            "node" | "pnpm" => {
                tool.declaration.request = Request::Pinned("1.2.3".to_string());
                if tool.declaration.id == "pnpm" {
                    tool.declaration.route = super::model::Route::Archive;
                    tool.declaration.options.bin_aliases = None;
                    tool.declaration.bins = vec!["pnpm".to_string()];
                }
                let url = format!("https://example.invalid/{}.tar.gz", tool.declaration.id);
                http.serve(&url, tool.declaration.id.as_bytes());
                tool.artifacts = vec![("linux-x64".to_string(), Artifact { url, sha256: sha256_hex(tool.declaration.id.as_bytes()) })];
            }
            _ => {
                tool.version = "4.0.0".to_string();
                tool.declaration.options.allow_builds = None;
            }
        }
    }
    fs::write(&scope.lock_file, render_lock(&lock)).expect("write the lock");
    let runner = FakeRunner::new();
    let root = scope.toolchain_root.clone();
    runner.on(
        "tar",
        Box::new(move |request| {
            let into = PathBuf::from(request.args.last().expect("tar --directory <dir>"));
            assert!(into.starts_with(&root), "the unpack target is inside the disposable toolchain root");
            for name in ["node", "npm", "npx", "pnpm"] {
                write_executable(&into.join(name));
            }
        }),
    );
    return (directory, scope, env, http, runner);
}

#[test]
fn a_failed_npm_lookup_of_a_missing_locked_tool_syncs_the_missing_tools_then_retries_once() {
    let (_directory, scope, env, http, runner) = auto_sync_rig();
    http.fail_first(NPM_URL, "connection reset", 1);
    http.serve(NPM_URL, b"{\"dist-tags\": {\"latest\": \"4.156.0\"}}");
    let report = run_update(&scope, &env, &runner, &http, &linux(), false).expect("updated");
    assert_eq!(
        outcome_of(&report, "npm:wrangler"),
        &OutcomeKind::Updated { old_version: "4.0.0".to_string(), new_version: "4.156.0".to_string() }
    );
    assert_eq!(http.requests().iter().filter(|url| return url.as_str() == NPM_URL).count(), 2, "looked up exactly twice");
    assert_eq!(report.synced, vec!["node".to_string(), "pnpm".to_string(), "npm:wrangler".to_string()]);
    let lines = runner.lines();
    assert!(lines.iter().any(|line| return line == "pnpm add --global wrangler@4.0.0"), "the missing tool installs at its locked version: {lines:?}");
    assert!(report.wrote);
    assert_eq!(read_lock(&scope).tool("npm:wrangler").expect("wrangler").version, "4.156.0");
}

#[test]
fn a_second_failure_is_final_and_says_the_lookup_was_made_twice() {
    let (_directory, scope, env, http, runner) = auto_sync_rig();
    http.fail(NPM_URL, "connection reset");
    let report = run_update(&scope, &env, &runner, &http, &linux(), false).expect("a report");
    assert!(report.has_failures());
    assert_eq!(http.requests().iter().filter(|url| return url.as_str() == NPM_URL).count(), 2);
    match outcome_of(&report, "npm:wrangler") {
        OutcomeKind::Failed { reason } => {
            assert!(reason.contains("looked up twice"), "{reason}");
            assert!(reason.contains(NPM_URL), "{reason}");
        }
        other => panic!("expected a failure, found {other:?}"),
    }
    assert!(!report.wrote);
}

#[test]
fn an_installed_tool_whose_lookup_fails_is_not_synced() {
    let (_directory, scope, env, http, runner) = auto_sync_rig();
    for name in ["node", "npm", "npx"] {
        write_executable(&tool_install_dir(&scope.toolchain_root, "node", "1.2.3").join(name));
    }
    write_executable(&tool_install_dir(&scope.toolchain_root, "pnpm", "1.2.3").join("pnpm"));
    runner.answer("pnpm list", "[{\"dependencies\": {\"wrangler\": {\"version\": \"4.0.0\"}}}]", 0);
    http.fail(NPM_URL, "503");
    let report = run_update(&scope, &env, &runner, &http, &linux(), false).expect("a report");
    assert!(report.synced.is_empty());
    assert_eq!(http.requests().iter().filter(|url| return url.as_str() == NPM_URL).count(), 1, "no retry for an installed tool");
    assert!(report.has_failures());
}

#[test]
fn check_never_installs_even_when_a_lookup_fails() {
    let (_directory, scope, env, http, runner) = auto_sync_rig();
    http.fail(NPM_URL, "connection reset");
    let report = run_update(&scope, &env, &runner, &http, &linux(), true).expect("a report");
    assert!(report.has_failures());
    assert!(report.synced.is_empty());
    assert!(runner.calls().is_empty(), "update --check starts no program");
    assert_eq!(http.requests().iter().filter(|url| return url.as_str() == NPM_URL).count(), 1);
}

/// An `HttpGet` that rewrites the plan file the first time it is asked for anything, as a person editing the plan
/// while an update waits on the network would.
struct EditingHttp<'a> {
    inner: &'a FakeHttp,
    plan: PathBuf,
}

impl HttpGet for EditingHttp<'_> {
    fn get(&self, url: &str, headers: &[(String, String)], max_bytes: usize) -> Result<HttpResponse, HttpError> {
        let current = fs::read_to_string(&self.plan).expect("read the plan");
        if !current.contains("// edited during the update") {
            fs::write(&self.plan, format!("{current}\n// edited during the update\n")).expect("edit the plan");
        }
        return self.inner.get(url, headers, max_bytes);
    }

    fn download(&self, url: &str, target: &std::path::Path, max_bytes: u64) -> Result<String, HttpError> {
        return self.inner.download(url, target, max_bytes);
    }
}

#[test]
fn an_update_does_not_publish_over_a_plan_that_changed_while_it_ran() {
    let (_directory, scope, env) = registry_scope();
    let inner = FakeHttp::new();
    serve_registries(&inner, "10.5.0", "4.156.0", "0.9.0", "2026-10-30");
    let http = EditingHttp { inner: &inner, plan: scope.plan_file.clone() };
    let error = run_update(&scope, &env, &FakeRunner::new(), &http, &linux(), false).expect_err("the plan moved");
    assert!(error.message.contains("toolchain.plan.jsonc changed while this command was running"), "{error}");
    assert!(!scope.lock_file.exists());
}

//endregion update
