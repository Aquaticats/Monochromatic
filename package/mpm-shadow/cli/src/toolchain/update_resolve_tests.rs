//! Unit tests of every resolver.
//! Each is asked about one origin with only the request in hand
//! (no previous lock row exists in any of these tests),
//! against a table-driven fake network.

use super::fakes::{FakeHttp, Rig};
use super::hash::sha256_hex;
use super::model::{Artifact, Declaration, Request};
use super::update_android_conda::{cmdline_packages, resolve_android, resolve_conda};
use super::update_github::{glob_matches, resolve_github};
use super::update_github_assets::pick_asset;
use super::update_go_zig::{resolve_go_module, resolve_go_runtime, resolve_zig};
use super::update_java_aws::{resolve_aws, resolve_java};
use super::update_registry::{registry_path_name, resolve_crate, resolve_npm, resolve_pypi};
use super::update_resolve::{ResolveError, resolve_tool};
use super::update_rust_node::{parse_shasums, resolve_node, resolve_rust, resolve_rustup};

const LINUX: [&str; 4] = ["linux-arm64", "linux-arm64-musl", "linux-x64", "linux-x64-musl"];

/// The platform ids of a resolution's rows.
fn platforms(rows: &[(String, Artifact)]) -> Vec<&str> {
    let mut found: Vec<&str> = Vec::new();
    for (platform, _) in rows {
        found.push(platform.as_str());
    }
    return found;
}

/// The row of one platform.
fn row<'a>(rows: &'a [(String, Artifact)], platform: &str) -> &'a Artifact {
    for (candidate, artifact) in rows {
        if candidate == platform {
            return artifact;
        }
    }
    panic!("no row for {platform} in {:?}", platforms(rows));
}

/// A declaration for `github:` tests, with an optional `asset` hint.
fn github_declaration(rig: &mut Rig, hint: Option<&str>) -> Declaration {
    rig.lock.tools[2].declaration.options.asset = hint.map(str::to_string);
    return rig.tool("pnpm").declaration;
}

/// A GitHub release JSON with these asset names;
/// every asset's digest is the SHA-256 of its own name.
fn release_json(tag: &str, names: &[&str]) -> String {
    let mut assets: Vec<String> = Vec::new();
    for name in names {
        assets.push(format!(
            "{{\"name\": \"{name}\", \"browser_download_url\": \"https://github.com/o/r/releases/download/{tag}/{name}\", \"digest\": \"sha256:{}\"}}",
            sha256_hex(name.as_bytes())
        ));
    }
    return format!("{{\"tag_name\": \"{tag}\", \"assets\": [{}]}}", assets.join(", "));
}

//region registries

#[test]
fn package_names_are_checked_before_they_reach_a_url() {
    assert_eq!(registry_path_name("wrangler").expect("fine"), "wrangler");
    assert_eq!(registry_path_name("@actions/languageserver").expect("scoped"), "@actions%2Flanguageserver");
    for bad in ["", "a b", "a?x=1", "a#b", "../etc", "a\nb", "a%2Fb"] {
        assert!(registry_path_name(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn npm_is_asked_at_registry_npmjs_org_directly_for_a_dist_tag_and_an_exact_version_needs_no_lookup() {
    let rig = Rig::new();
    rig.http.serve("https://registry.npmjs.org/wrangler", b"{\"dist-tags\": {\"latest\": \"4.156.0\", \"next\": \"5.0.0-rc.1\"}}");
    assert_eq!(resolve_npm(&rig.context(), "wrangler", None).expect("resolved").version, "4.156.0");
    assert_eq!(resolve_npm(&rig.context(), "wrangler", Some("next")).expect("resolved").version, "5.0.0-rc.1");
    assert_eq!(resolve_npm(&rig.context(), "wrangler", Some("3.0.0")).expect("resolved").version, "3.0.0");
    assert_eq!(rig.http.requests().len(), 2, "the exact version made no request");
    assert!(rig.runner.calls().is_empty(), "no package manager is asked");
}

#[test]
fn a_scoped_npm_package_is_requested_with_an_escaped_slash() {
    let rig = Rig::new();
    rig.http.serve("https://registry.npmjs.org/@cloudflare%2Fwrangler", b"{\"dist-tags\": {\"latest\": \"4.0.0\"}}");
    assert_eq!(resolve_npm(&rig.context(), "@cloudflare/wrangler", None).expect("resolved").version, "4.0.0");
}

#[test]
fn registry_failures_name_the_url_and_the_reason() {
    let rig = Rig::new();
    let missing = resolve_npm(&rig.context(), "wrangler", None).expect_err("the fake answers 404");
    assert_eq!(missing, ResolveError::Failed("https://registry.npmjs.org/wrangler: HTTP 404".to_string()));
    rig.http.serve("https://registry.npmjs.org/wrangler", b"{\"dist-tags\": {}}");
    let no_tag = resolve_npm(&rig.context(), "wrangler", None).expect_err("no tag");
    assert!(no_tag.message().contains("no latest dist-tag"), "{no_tag:?}");
    rig.http.fail("https://pypi.org/pypi/slopo/json", "no route to host");
    let declaration = rig.tool("pipx:slopo").declaration;
    let down = resolve_pypi(&rig.context(), &declaration, "slopo", None).expect_err("network down");
    assert_eq!(down, ResolveError::Failed("https://pypi.org/pypi/slopo/json: no route to host".to_string()));
    rig.http.serve("https://crates.io/api/v1/crates/fd-find", b"<html>maintenance</html>");
    let html = resolve_crate(&rig.context(), "fd-find", None).expect_err("not JSON");
    assert!(html.message().starts_with("https://crates.io/api/v1/crates/fd-find: the answer is not JSON"), "{html:?}");
}

#[test]
fn pypi_and_crates_io_give_their_newest_version_and_exact_ranges_need_no_lookup() {
    let rig = Rig::new();
    rig.http.serve("https://pypi.org/pypi/slopo/json", b"{\"info\": {\"version\": \"0.9.1\"}}");
    rig.http.serve("https://crates.io/api/v1/crates/fd-find", b"{\"crate\": {\"max_stable_version\": \"10.5.0\"}}");
    let declaration = rig.tool("pipx:slopo").declaration;
    assert_eq!(resolve_pypi(&rig.context(), &declaration, "slopo", None).expect("resolved").version, "0.9.1");
    assert_eq!(resolve_pypi(&rig.context(), &declaration, "slopo", Some("0.8.0")).expect("resolved").version, "0.8.0");
    assert_eq!(resolve_crate(&rig.context(), "fd-find", None).expect("resolved").version, "10.5.0");
    assert_eq!(resolve_crate(&rig.context(), "fd-find", Some("10.4.0")).expect("resolved").version, "10.4.0");
    assert_eq!(rig.http.requests().len(), 2);
}

#[test]
fn a_wheel_bootstrap_records_the_wheel_url_and_hash_for_the_four_linux_platforms() {
    let mut rig = Rig::new();
    rig.lock.tools[9].declaration.options.bootstrap = Some(super::model::Bootstrap { requires_tool: Some("uv".to_string()), distribution: "wheel".to_string() });
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
    let resolution = resolve_pypi(&rig.context(), &declaration, "slopo", Some("8.2.0")).expect("resolved");
    assert_eq!(resolution.version, "8.2.0");
    assert_eq!(platforms(&resolution.artifacts), LINUX.to_vec());
    assert_eq!(row(&resolution.artifacts, "linux-x64").sha256, wheel_hash);
    rig.http.serve("https://pypi.org/pypi/slopo/json", b"{\"info\": {\"version\": \"1.0.0\"}, \"urls\": []}");
    let error = resolve_pypi(&rig.context(), &declaration, "slopo", None).expect_err("no wheel");
    assert!(error.message().contains("no pure-Python wheel"), "{error:?}");
}

//endregion registries

//region rust and node

#[test]
fn the_nightly_channel_gives_a_dated_name_stable_gives_its_version_and_an_exact_name_needs_no_lookup() {
    let rig = Rig::new();
    rig.http.serve("https://static.rust-lang.org/dist/channel-rust-nightly.toml", b"date = \"2026-11-02\"\n[pkg.rust]\nversion = \"1.103.0-nightly (abc)\"\n");
    rig.http.serve("https://static.rust-lang.org/dist/channel-rust-stable.toml", b"[pkg.cargo]\nversion = \"0.1\"\n[pkg.rust]\nversion = \"1.99.0 (abcdef 2026-10-01)\"\n");
    assert_eq!(resolve_rust(&rig.context(), "nightly").expect("resolved").version, "nightly-2026-11-02");
    assert_eq!(resolve_rust(&rig.context(), "stable").expect("resolved").version, "1.99.0");
    assert_eq!(resolve_rust(&rig.context(), "nightly-2026-10-29").expect("resolved").version, "nightly-2026-10-29");
    assert_eq!(rig.http.requests().len(), 2);
    rig.http.serve("https://static.rust-lang.org/dist/channel-rust-beta.toml", b"[pkg.rust]\nversion = \"1.0\"\n");
    assert!(resolve_rust(&rig.context(), "beta").is_ok());
    let dateless = Rig::new();
    dateless.http.serve("https://static.rust-lang.org/dist/channel-rust-nightly.toml", b"[pkg.rust]\nversion = \"1.0.0\"\n");
    assert!(resolve_rust(&dateless.context(), "nightly").expect_err("no date").message().contains("carries no date"));
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

fn serve_node(http: &FakeHttp, version: &str) {
    let mut official = String::new();
    let mut unofficial = String::new();
    for arch in ["x64", "arm64"] {
        official.push_str(&format!("{}  node-v{version}-linux-{arch}.tar.gz\n", sha256_hex(format!("official-{arch}").as_bytes())));
        unofficial.push_str(&format!("{}  node-v{version}-linux-{arch}-musl.tar.gz\n", sha256_hex(format!("musl-{arch}").as_bytes())));
    }
    http.serve(&format!("https://nodejs.org/dist/v{version}/SHASUMS256.txt"), official.as_bytes());
    http.serve(&format!("https://unofficial-builds.nodejs.org/download/release/v{version}/SHASUMS256.txt"), unofficial.as_bytes());
}

#[test]
fn node_follows_the_latest_stable_release_and_builds_the_four_rows_from_the_version_alone() {
    let rig = Rig::new();
    rig.http.serve("https://nodejs.org/dist/index.json", b"[{\"version\": \"v27.0.0-rc.1\"}, {\"version\": \"v26.14.0\"}, {\"version\": \"v26.13.0\"}]");
    serve_node(&rig.http, "26.14.0");
    let resolution = resolve_node(&rig.context(), None).expect("resolved");
    assert_eq!(resolution.version, "26.14.0");
    assert_eq!(platforms(&resolution.artifacts), LINUX.to_vec());
    assert_eq!(row(&resolution.artifacts, "linux-x64").url, "https://nodejs.org/dist/v26.14.0/node-v26.14.0-linux-x64.tar.gz");
    assert_eq!(row(&resolution.artifacts, "linux-x64").sha256, sha256_hex(b"official-x64"));
    assert_eq!(row(&resolution.artifacts, "linux-arm64-musl").url, "https://unofficial-builds.nodejs.org/download/release/v26.14.0/node-v26.14.0-linux-arm64-musl.tar.gz");
    assert_eq!(row(&resolution.artifacts, "linux-arm64-musl").sha256, sha256_hex(b"musl-arm64"));
}

#[test]
fn node_at_an_exact_version_skips_the_index_and_a_missing_checksum_names_the_archive() {
    let rig = Rig::new();
    serve_node(&rig.http, "26.13.0");
    assert_eq!(resolve_node(&rig.context(), Some("26.13.0")).expect("resolved").version, "26.13.0");
    assert!(!rig.http.requests().contains(&"https://nodejs.org/dist/index.json".to_string()));
    let error = resolve_node(&rig.context(), Some("26.99.0")).expect_err("no checksums for that release");
    assert!(error.message().contains("has no row for node-v26.99.0-linux-"), "{error:?}");
}

#[test]
fn rustup_takes_the_stable_release_and_a_checksum_per_target() {
    let rig = Rig::new();
    rig.http.serve("https://static.rust-lang.org/rustup/release-stable.toml", b"schema-version = '1'\nversion = '1.29.2'\n");
    for triple in ["aarch64-unknown-linux-gnu", "aarch64-unknown-linux-musl", "x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"] {
        let digest = sha256_hex(triple.as_bytes());
        rig.http.serve(&format!("https://static.rust-lang.org/rustup/archive/1.29.2/{triple}/rustup-init.sha256"), format!("{digest}  rustup-init\n").as_bytes());
    }
    let resolution = resolve_rustup(&rig.context(), None).expect("resolved");
    assert_eq!(resolution.version, "1.29.2");
    assert_eq!(row(&resolution.artifacts, "linux-x64-musl").url, "https://static.rust-lang.org/rustup/archive/1.29.2/x86_64-unknown-linux-musl/rustup-init");
    assert_eq!(row(&resolution.artifacts, "linux-arm64").sha256, sha256_hex(b"aarch64-unknown-linux-gnu"));
}

//endregion rust and node

//region go, zig, java, aws, android, conda

#[test]
fn the_go_runtime_is_the_newest_stable_release_with_the_indexs_own_checksums() {
    let rig = Rig::new();
    rig.http.serve(
        "https://go.dev/dl/?mode=json&include=all",
        b"[{\"version\": \"go1.28rc1\", \"stable\": false, \"files\": []}, \
          {\"version\": \"go1.27.3\", \"stable\": true, \"files\": [\
            {\"filename\": \"go1.27.3.linux-amd64.tar.gz\", \"os\": \"linux\", \"arch\": \"amd64\", \"kind\": \"archive\", \"sha256\": \"aa\"}, \
            {\"filename\": \"go1.27.3.linux-arm64.tar.gz\", \"os\": \"linux\", \"arch\": \"arm64\", \"kind\": \"archive\", \"sha256\": \"bb\"}, \
            {\"filename\": \"go1.27.3.windows-amd64.zip\", \"os\": \"windows\", \"arch\": \"amd64\", \"kind\": \"archive\", \"sha256\": \"cc\"}]}, \
          {\"version\": \"go1.26.9\", \"stable\": true, \"files\": [\
            {\"filename\": \"go1.26.9.linux-amd64.tar.gz\", \"os\": \"linux\", \"arch\": \"amd64\", \"kind\": \"archive\", \"sha256\": \"dd\"}, \
            {\"filename\": \"go1.26.9.linux-arm64.tar.gz\", \"os\": \"linux\", \"arch\": \"arm64\", \"kind\": \"archive\", \"sha256\": \"ee\"}]}]",
    );
    let latest = resolve_go_runtime(&rig.context(), None).expect("resolved");
    assert_eq!(latest.version, "1.27.3");
    assert_eq!(row(&latest.artifacts, "linux-x64-musl").url, "https://go.dev/dl/go1.27.3.linux-amd64.tar.gz");
    assert_eq!(row(&latest.artifacts, "linux-arm64").sha256, "bb");
    let older = resolve_go_runtime(&rig.context(), Some("1.26.9")).expect("resolved");
    assert_eq!(row(&older.artifacts, "linux-x64").sha256, "dd");
    assert!(resolve_go_runtime(&rig.context(), Some("1.0.0")).is_err());
}

#[test]
fn a_go_module_asks_the_proxy_with_capitals_escaped() {
    let rig = Rig::new();
    rig.http.serve("https://proxy.golang.org/github.com/!burnt!sushi/toml/@latest", b"{\"Version\": \"v1.5.0\"}");
    assert_eq!(resolve_go_module(&rig.context(), "github.com/BurntSushi/toml", None).expect("resolved").version, "1.5.0");
    assert_eq!(resolve_go_module(&rig.context(), "github.com/BurntSushi/toml", Some("1.4.0")).expect("resolved").version, "1.4.0");
    assert!(resolve_go_module(&rig.context(), "bad module", None).is_err());
    assert!(resolve_go_module(&rig.context(), "../x", None).is_err());
}

#[test]
fn zig_takes_the_newest_stable_key_and_records_the_tarballs_of_the_index() {
    let rig = Rig::new();
    rig.http.serve(
        "https://ziglang.org/download/index.json",
        b"{\"master\": {}, \"0.16.0-dev.1\": {}, \"0.15.2\": {\
            \"x86_64-linux\": {\"tarball\": \"https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz\", \"shasum\": \"11\"}, \
            \"aarch64-linux\": {\"tarball\": \"https://ziglang.org/download/0.15.2/zig-aarch64-linux-0.15.2.tar.xz\", \"shasum\": \"22\"}}, \
          \"0.9.1\": {}, \"0.15.10\": {\
            \"x86_64-linux\": {\"tarball\": \"https://ziglang.org/download/0.15.10/zig-x86_64-linux-0.15.10.tar.xz\", \"shasum\": \"33\"}, \
            \"aarch64-linux\": {\"tarball\": \"https://ziglang.org/download/0.15.10/zig-aarch64-linux-0.15.10.tar.xz\", \"shasum\": \"44\"}}}",
    );
    let newest = resolve_zig(&rig.context(), None).expect("resolved");
    assert_eq!(newest.version, "0.15.10", "versions compare numerically, not as text");
    assert_eq!(row(&newest.artifacts, "linux-arm64-musl").sha256, "44");
    let pinned = resolve_zig(&rig.context(), Some("0.15.2")).expect("resolved");
    assert_eq!(row(&pinned.artifacts, "linux-x64").url, "https://ziglang.org/download/0.15.2/zig-x86_64-linux-0.15.2.tar.xz");
    assert!(resolve_zig(&rig.context(), Some("0.9.1")).is_err());
}

#[test]
fn java_follows_the_adoptium_release_and_picks_each_archive_by_name() {
    let rig = Rig::new();
    rig.http.serve("https://api.adoptium.net/v3/assets/latest/21/hotspot?architecture=x64&image_type=jdk&os=linux", b"[{\"release_name\": \"jdk-21.0.13+9\"}]");
    let names = [
        "OpenJDK21U-jdk_x64_linux_hotspot_21.0.13_9.tar.gz",
        "OpenJDK21U-jdk_aarch64_linux_hotspot_21.0.13_9.tar.gz",
        "OpenJDK21U-jdk_x64_alpine-linux_hotspot_21.0.13_9.tar.gz",
        "OpenJDK21U-jdk_aarch64_alpine-linux_hotspot_21.0.13_9.tar.gz",
        "OpenJDK21U-jdk_x64_linux_hotspot_21.0.13_9.tar.gz.sha256.txt",
    ];
    rig.http.serve("https://api.github.com/repos/adoptium/temurin21-binaries/releases/tags/jdk-21.0.13%2B9", release_json("jdk-21.0.13+9", &names).as_bytes());
    let resolution = resolve_java(&rig.context(), "21").expect("resolved");
    assert_eq!(resolution.version, "temurin-21.0.13+9.0.LTS");
    assert!(row(&resolution.artifacts, "linux-x64-musl").url.ends_with("OpenJDK21U-jdk_x64_alpine-linux_hotspot_21.0.13_9.tar.gz"));
    assert_eq!(row(&resolution.artifacts, "linux-arm64").sha256, sha256_hex(names[1].as_bytes()));
    let exact = resolve_java(&rig.context(), "21.0.13+9").expect("an exact release skips the Adoptium query");
    assert_eq!(exact.version, resolution.version);
    assert!(resolve_java(&rig.context(), "latest").is_err());
}

#[test]
fn the_aws_cli_takes_the_newest_stable_v2_tag_and_hashes_each_installer_once() {
    let rig = Rig::new();
    rig.http.serve(
        "https://api.github.com/repos/aws/aws-cli/tags?per_page=100",
        b"[{\"name\": \"2.39.2\"}, {\"name\": \"2.9.0\"}, {\"name\": \"1.99.0\"}, {\"name\": \"2.40.0-rc1\"}, {\"name\": \"2.39.10\"}]",
    );
    rig.http.serve("https://awscli.amazonaws.com/awscli-exe-linux-x86_64-2.39.10.zip", b"x86 installer");
    rig.http.serve("https://awscli.amazonaws.com/awscli-exe-linux-aarch64-2.39.10.zip", b"arm installer");
    let resolution = resolve_aws(&rig.context(), None).expect("resolved");
    assert_eq!(resolution.version, "2.39.10", "tags compare numerically");
    assert_eq!(row(&resolution.artifacts, "linux-x64-musl").sha256, sha256_hex(b"x86 installer"));
    assert_eq!(row(&resolution.artifacts, "linux-arm64").sha256, sha256_hex(b"arm installer"));
    let downloads: Vec<String> = rig.http.requests().into_iter().filter(|url| return url.ends_with(".zip")).collect();
    assert_eq!(downloads.len(), 2, "one download per distinct URL: {downloads:?}");
}

const ANDROID_XML: &str = "<repository>\n<remotePackage path=\"cmdline-tools;12.0\">\n<revision><major>12</major><minor>0</minor></revision>\n<archive><complete><url>commandlinetools-linux-11076708_latest.zip</url></complete></archive></remotePackage>\n\
<remotePackage path=\"cmdline-tools;23.0\">\n<revision>\n<major>23</major>\n<minor>0</minor>\n</revision>\n<archive><complete><url>commandlinetools-linux-16111833_latest.zip</url></complete></archive></remotePackage>\n\
<remotePackage path=\"platform-tools\"><revision><major>35</major></revision></remotePackage>\n<remotePackage path=\"cmdline-tools;bad\"><revision><major>x</major><minor>1</minor></revision><url>commandlinetools-linux-1_latest.zip</url></remotePackage></repository>";

#[test]
fn the_android_listing_yields_the_cmdline_tools_packages_and_skips_unreadable_blocks() {
    let packages = cmdline_packages(ANDROID_XML);
    assert_eq!(packages.len(), 2);
    assert_eq!((packages[1].major, packages[1].minor, packages[1].build), (23, 0, 16_111_833));
}

#[test]
fn android_takes_the_newest_package_or_the_wanted_revision_and_hashes_the_archive_once() {
    let rig = Rig::new();
    rig.http.serve("https://dl.google.com/android/repository/repository2-3.xml", ANDROID_XML.as_bytes());
    rig.http.serve("https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip", b"newest zip");
    rig.http.serve("https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip", b"older zip");
    let newest = resolve_android(&rig.context(), None).expect("resolved");
    assert_eq!(newest.version, "23.0");
    assert_eq!(platforms(&newest.artifacts), LINUX.to_vec());
    assert_eq!(row(&newest.artifacts, "linux-arm64").sha256, sha256_hex(b"newest zip"));
    let older = resolve_android(&rig.context(), Some("12.0")).expect("resolved");
    assert_eq!(older.version, "12.0");
    assert_eq!(row(&older.artifacts, "linux-x64").sha256, sha256_hex(b"older zip"));
    assert!(resolve_android(&rig.context(), Some("99.0")).is_err());
}

#[test]
fn conda_takes_the_latest_version_and_the_highest_build_per_subdir() {
    let rig = Rig::new();
    rig.http.serve(
        "https://api.anaconda.org/package/conda-forge/bubblewrap",
        b"{\"latest_version\": \"0.13.1\", \"files\": [\
          {\"version\": \"0.13.1\", \"basename\": \"linux-64/bubblewrap-0.13.1-h0_0.conda\", \"sha256\": \"a1\", \"attrs\": {\"subdir\": \"linux-64\", \"build_number\": 0}}, \
          {\"version\": \"0.13.1\", \"basename\": \"linux-64/bubblewrap-0.13.1-h1_1.conda\", \"sha256\": \"a2\", \"attrs\": {\"subdir\": \"linux-64\", \"build_number\": 1}}, \
          {\"version\": \"0.13.1\", \"basename\": \"linux-aarch64/bubblewrap-0.13.1-h2_0.conda\", \"sha256\": \"b1\", \"attrs\": {\"subdir\": \"linux-aarch64\", \"build_number\": 0}}, \
          {\"version\": \"0.12.0\", \"basename\": \"linux-64/bubblewrap-0.12.0-h3_0.conda\", \"sha256\": \"c1\", \"attrs\": {\"subdir\": \"linux-64\", \"build_number\": 0}}, \
          {\"version\": \"0.12.0\", \"basename\": \"linux-aarch64/bubblewrap-0.12.0-h4_0.conda\", \"sha256\": \"c2\", \"attrs\": {\"subdir\": \"linux-aarch64\", \"build_number\": 0}}]}",
    );
    let latest = resolve_conda(&rig.context(), "bubblewrap", None).expect("resolved");
    assert_eq!(latest.version, "0.13.1");
    assert_eq!(row(&latest.artifacts, "linux-x64").url, "https://conda.anaconda.org/conda-forge/linux-64/bubblewrap-0.13.1-h1_1.conda");
    assert_eq!(row(&latest.artifacts, "linux-arm64-musl").sha256, "b1");
    assert_eq!(row(&resolve_conda(&rig.context(), "bubblewrap", Some("0.12.0")).expect("resolved").artifacts, "linux-x64").sha256, "c1");
    assert!(resolve_conda(&rig.context(), "../x", None).is_err());
}

//endregion go, zig, java, aws, android, conda

//region github

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

/// Parses the assets of a release JSON for the chooser.
fn assets_of(names: &[&str]) -> Vec<serde_json::Value> {
    let release: serde_json::Value = serde_json::from_str(&release_json("v1", names)).expect("JSON");
    return release["assets"].as_array().expect("assets").clone();
}

/// The names `pick_asset` chooses for the four Linux platforms, or `-` for none.
fn chosen(names: &[&str], hint: Option<&str>) -> Vec<String> {
    let assets = assets_of(names);
    let mut picked: Vec<String> = Vec::new();
    for platform in LINUX {
        let asset = pick_asset(&assets, platform, hint, "o/r", "v1").expect("no tie");
        picked.push(asset.and_then(|found| return found["name"].as_str()).unwrap_or("-").to_string());
    }
    return picked;
}

#[test]
fn the_chooser_reproduces_the_assets_of_todays_lock_from_the_usual_naming_conventions() {
    // ripgrep: only a musl build for x86_64, gnu for aarch64.
    assert_eq!(
        chosen(
            &[
                "ripgrep-15.2.0-x86_64-unknown-linux-musl.tar.gz",
                "ripgrep-15.2.0-aarch64-unknown-linux-gnu.tar.gz",
                "ripgrep-15.2.0-x86_64-pc-windows-msvc.zip",
                "ripgrep-15.2.0-x86_64-apple-darwin.tar.gz",
                "ripgrep-15.2.0-x86_64-unknown-linux-musl.tar.gz.sha256",
                "ripgrep_15.2.0-1_amd64.deb"
            ],
            None
        ),
        vec![
            "ripgrep-15.2.0-aarch64-unknown-linux-gnu.tar.gz",
            "ripgrep-15.2.0-aarch64-unknown-linux-gnu.tar.gz",
            "ripgrep-15.2.0-x86_64-unknown-linux-musl.tar.gz",
            "ripgrep-15.2.0-x86_64-unknown-linux-musl.tar.gz"
        ]
    );
    // uv: both libc builds, the platform's own libc wins; archives beat their checksums.
    let uv = chosen(
        &[
            "uv-x86_64-unknown-linux-gnu.tar.gz",
            "uv-x86_64-unknown-linux-musl.tar.gz",
            "uv-aarch64-unknown-linux-gnu.tar.gz",
            "uv-aarch64-unknown-linux-musl.tar.gz",
            "uv-x86_64-unknown-linux-gnu.tar.gz.sha256",
            "uv-installer.sh"
        ],
        None,
    );
    assert_eq!(uv, vec!["uv-aarch64-unknown-linux-gnu.tar.gz", "uv-aarch64-unknown-linux-musl.tar.gz", "uv-x86_64-unknown-linux-gnu.tar.gz", "uv-x86_64-unknown-linux-musl.tar.gz"]);
    // conventions seen in the lock: linux_amd64, linux-x64, linux.amd64, ubuntu-x64, a bare binary.
    assert_eq!(chosen(&["caddy_2.11.7_linux_amd64.tar.gz", "caddy_2.11.7_linux_arm64.tar.gz", "caddy_2.11.7_linux_amd64.sbom", "caddy_2.11.7_windows_amd64.zip"], None)[2], "caddy_2.11.7_linux_amd64.tar.gz");
    assert_eq!(chosen(&["sops-v3.13.3.linux.amd64", "sops-v3.13.3.linux.arm64", "sops-v3.13.3.darwin.arm64", "sops-v3.13.3.amd64.deb"], None)[0], "sops-v3.13.3.linux.arm64");
    assert_eq!(chosen(&["llama-b9829-bin-ubuntu-x64.tar.gz", "llama-b9829-bin-ubuntu-arm64.tar.gz", "llama-b9829-bin-win-cpu-x64.zip"], None)[2], "llama-b9829-bin-ubuntu-x64.tar.gz");
    assert_eq!(chosen(&["marksman-linux-x64", "marksman-linux-arm64", "marksman-macos", "marksman.exe"], None)[2], "marksman-linux-x64");
    assert_eq!(chosen(&["x86_64-linux-musl.tar.xz", "aarch64-linux-musl.tar.xz", "x86_64-windows.zip"], None)[3], "x86_64-linux-musl.tar.xz");
}

#[test]
fn the_chooser_prefers_the_commonest_archive_format_and_skips_baseline_and_debug_builds() {
    let names = ["bun-linux-x64.zip", "bun-linux-x64-baseline.zip", "bun-linux-x64-profile.zip", "bun-linux-aarch64.zip", "bun-linux-x64.tar.gz"];
    assert_eq!(chosen(&names, None)[2], "bun-linux-x64.tar.gz");
    assert_eq!(chosen(&names[..4], None)[2], "bun-linux-x64.zip");
}

#[test]
fn two_equally_good_assets_fail_naming_both_and_a_hint_settles_it() {
    let names = ["harper-cli-x86_64-unknown-linux-gnu.tar.gz", "harper-ls-x86_64-unknown-linux-gnu.tar.gz", "harper-cli-aarch64-unknown-linux-gnu.tar.gz", "harper-ls-aarch64-unknown-linux-gnu.tar.gz"];
    let assets = assets_of(&names);
    let error = pick_asset(&assets, "linux-x64", None, "Automattic/harper", "v3.0.0").expect_err("a tie");
    assert!(error.message().contains("harper-cli-x86_64-unknown-linux-gnu.tar.gz, harper-ls-x86_64-unknown-linux-gnu.tar.gz"), "{error:?}");
    assert!(error.message().contains("options.asset"), "{error:?}");
    assert_eq!(chosen(&names, Some("harper-cli-*"))[2], "harper-cli-x86_64-unknown-linux-gnu.tar.gz");
}

#[test]
fn a_github_tool_resolves_from_its_repository_alone() {
    let mut rig = Rig::new();
    let declaration = github_declaration(&mut rig, None);
    let names = ["tool-1.3.0-linux-x64.tar.gz", "tool-1.3.0-linux-arm64.tar.gz", "tool-1.3.0-macos-arm64.tar.gz"];
    rig.http.serve("https://api.github.com/repos/o/r/releases/latest", release_json("v1.3.0", &names).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved");
    assert_eq!(resolution.version, "1.3.0");
    assert_eq!(platforms(&resolution.artifacts), LINUX.to_vec());
    assert_eq!(row(&resolution.artifacts, "linux-x64-musl").url, "https://github.com/o/r/releases/download/v1.3.0/tool-1.3.0-linux-x64.tar.gz");
    assert_eq!(row(&resolution.artifacts, "linux-x64").sha256, sha256_hex(names[0].as_bytes()));
    assert_eq!(rig.http.requests(), vec!["https://api.github.com/repos/o/r/releases/latest".to_string()]);
}

#[test]
fn a_github_asset_without_a_digest_is_downloaded_and_hashed() {
    let mut rig = Rig::new();
    let declaration = github_declaration(&mut rig, None);
    let url = "https://github.com/o/r/releases/download/v1/tool-linux-x64.tar.gz";
    rig.http.serve("https://api.github.com/repos/o/r/releases/latest", format!("{{\"tag_name\": \"v1\", \"assets\": [{{\"name\": \"tool-linux-x64.tar.gz\", \"browser_download_url\": \"{url}\"}}]}}").as_bytes());
    rig.http.serve(url, b"the asset bytes");
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", None).expect("resolved");
    assert_eq!(platforms(&resolution.artifacts), vec!["linux-x64", "linux-x64-musl"], "arm64 has no asset and is left out");
    assert_eq!(row(&resolution.artifacts, "linux-x64").sha256, sha256_hex(b"the asset bytes"));
}

#[test]
fn a_pinned_github_tag_is_asked_for_as_written_then_with_a_v() {
    let mut rig = Rig::new();
    let declaration = github_declaration(&mut rig, None);
    rig.http.serve("https://api.github.com/repos/o/r/releases/tags/v0.64.3", release_json("v0.64.3", &["tool-linux-x64.tar.gz"]).as_bytes());
    let resolution = resolve_github(&rig.context(), &declaration, "o/r", Some("0.64.3")).expect("resolved");
    assert_eq!(resolution.version, "0.64.3");
    assert_eq!(
        rig.http.requests(),
        vec!["https://api.github.com/repos/o/r/releases/tags/0.64.3".to_string(), "https://api.github.com/repos/o/r/releases/tags/v0.64.3".to_string()]
    );
    rig.http.serve("https://api.github.com/repos/o/r/releases/tags/bun-v1.3.14", release_json("bun-v1.3.14", &["bun-linux-x64.zip"]).as_bytes());
    assert_eq!(resolve_github(&rig.context(), &declaration, "o/r", Some("bun-v1.3.14")).expect("resolved").version, "1.3.14");
}

#[test]
fn a_release_with_no_linux_asset_or_a_malformed_repository_fails_with_a_diagnostic() {
    let mut rig = Rig::new();
    let declaration = github_declaration(&mut rig, None);
    rig.http.serve("https://api.github.com/repos/o/r/releases/latest", release_json("v1", &["tool-windows-x64.zip", "tool.dmg"]).as_bytes());
    let error = resolve_github(&rig.context(), &declaration, "o/r", None).expect_err("no linux asset");
    assert!(error.message().contains("has no Linux asset"), "{error:?}");
    assert!(resolve_github(&rig.context(), &declaration, "o", None).is_err());
    assert!(resolve_github(&rig.context(), &declaration, "o/r/x", None).is_err());
}

#[test]
fn a_brand_new_downloaded_tool_resolves_with_no_lock_at_all() {
    let rig = Rig::new();
    let declaration = {
        let mut copy = rig.tool("pnpm").declaration;
        copy.request = Request::parse("github:o/r@latest").expect("parses");
        copy
    };
    rig.http.serve("https://api.github.com/repos/o/r/releases/latest", release_json("v2.0.0", &["tool-linux-amd64.tar.gz", "tool-linux-arm64.tar.gz"]).as_bytes());
    let resolution = resolve_tool(&rig.context(), &declaration, None).expect("no previous lock row is needed");
    assert_eq!(resolution.version, "2.0.0");
    assert_eq!(resolution.artifacts.len(), 4);
}

//endregion github
