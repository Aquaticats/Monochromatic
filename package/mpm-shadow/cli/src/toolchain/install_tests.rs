//! Unit tests of every installer.
//! Each runs against the fake runner and fake network inside a disposable scope,
//! and
//! asserts the exact program lines that were started;
//! nothing here installs anything.

use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs;
use std::io::Write;

use super::error::ErrorKind;
use super::fakes::{Rig, write_executable};
use super::hash::sha256_hex;
use super::install::install_tool;
use super::install_archive::safe_file_name;
use super::install_bin::{percent_decode, release_tag_url, seed_bin_config};
use super::install_support::{atomic_symlink, restore_exec_bits};
use super::layout::tool_install_dir;
use super::model::{Artifact, Request};

/// Gives the tool in the rig's lock one artifact row for linux-x64.
fn set_artifact(rig: &mut Rig, id: &str, url: &str, sha256: &str) {
    let tool = rig.lock.tools.iter_mut().find(|candidate| return candidate.declaration.id == id).expect("the tool exists");
    tool.artifacts = vec![("linux-x64".to_string(), Artifact { url: url.to_string(), sha256: sha256.to_string() })];
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).expect("compress");
    return encoder.finish().expect("finish");
}

#[test]
fn cargo_binstall_runs_with_the_locked_version_the_features_and_the_token() {
    let mut rig = Rig::new();
    rig.env = rig.env.with("GITHUB_TOKEN", "secret-token");
    rig.lock.tools[6].declaration.options.cargo_features = Some(vec!["a".to_string(), "b".to_string()]);
    let tool = rig.tool("cargo:fd-find");
    install_tool(&rig.context(), &tool).expect("installed");
    assert_eq!(rig.runner.lines(), vec!["cargo binstall --no-confirm --version 1.2.3 --features=a --features=b fd-find".to_string()]);
    let env = rig.runner.calls()[0].env.clone().expect("an explicit environment");
    assert_eq!(env.get("GITHUB_TOKEN").map(String::as_str), Some("secret-token"));
    assert_eq!(env.get("HOME").map(String::as_str), Some(rig.scope.home.to_str().expect("utf-8")));
}

#[test]
fn a_failed_install_names_the_step_the_tool_and_the_programs_message() {
    let rig = Rig::new();
    rig.runner.fail("cargo binstall", "crate not found");
    let error = install_tool(&rig.context(), &rig.tool("cargo:fd-find")).expect_err("the install fails");
    assert_eq!(error.kind, ErrorKind::Failure);
    assert_eq!(error.message, "cargo binstall failed for cargo:fd-find: crate not found");
}

#[test]
fn cargo_git_installs_at_the_pinned_revision_into_the_tool_directory() {
    let mut rig = Rig::new();
    rig.lock.tools[6].declaration.route = super::model::Route::CargoGit;
    rig.lock.tools[6].declaration.request = Request::parse("git:https://git.example/repo@ref:abc123").expect("parses");
    install_tool(&rig.context(), &rig.tool("cargo:fd-find")).expect("installed");
    let root = tool_install_dir(&rig.scope.toolchain_root, "cargo:fd-find", "1.2.3");
    assert_eq!(
        rig.runner.lines(),
        vec![format!("cargo install --git https://git.example/repo --rev abc123 --root {} fd-find", root.display())]
    );
    assert!(root.is_dir());
}

#[test]
fn cargo_git_without_a_source_url_is_refused_before_anything_runs() {
    let mut rig = Rig::new();
    rig.lock.tools[6].declaration.route = super::model::Route::CargoGit;
    let error = install_tool(&rig.context(), &rig.tool("cargo:fd-find")).expect_err("no url");
    assert!(error.message.contains("its request is not git:<url>@<revision>"), "{error}");
    assert!(rig.runner.calls().is_empty());
}

#[test]
fn the_rust_toolchain_installs_directly_without_mpm_then_adds_components_and_targets() {
    let rig = Rig::new();
    install_tool(&rig.context(), &rig.tool("rust")).expect("installed");
    assert_eq!(
        rig.runner.lines(),
        vec![
            "rustup toolchain install --no-self-update nightly-2026-10-30".to_string(),
            "rustup component add clippy --toolchain nightly-2026-10-30".to_string(),
            "rustup component add rust-src --toolchain nightly-2026-10-30".to_string(),
            "rustup target add aarch64-linux-android --toolchain nightly-2026-10-30".to_string(),
        ]
    );
    assert!(rig.runner.lines().iter().all(|line| return !line.starts_with("mpm")), "mpm is not involved");
}

#[test]
fn a_failing_component_stops_the_rest_and_is_named() {
    let rig = Rig::new();
    rig.runner.fail("rustup component add clippy", "component unavailable");
    let error = install_tool(&rig.context(), &rig.tool("rust")).expect_err("fails");
    assert!(error.message.contains("rustup component add (clippy) failed for rust: component unavailable"), "{error}");
    assert_eq!(rig.runner.lines().len(), 2, "the toolchain install and the failing component only");
}

#[test]
fn an_npm_tool_needs_pnpm_installed_first() {
    let rig = Rig::new();
    let error = install_tool(&rig.context(), &rig.tool("npm:wrangler")).expect_err("no pnpm");
    assert!(error.message.contains("pnpm is not installed in the toolchain; the pnpm stage must run first"), "{error}");
    assert!(rig.runner.calls().is_empty());
}

#[test]
fn an_npm_tool_installs_globally_at_the_locked_version_with_its_allowed_builds() {
    let rig = Rig::new();
    let pnpm = rig.scope.toolchain_bin_dir.join("pnpm");
    write_executable(&pnpm);
    install_tool(&rig.context(), &rig.tool("npm:wrangler")).expect("installed");
    assert_eq!(rig.runner.lines(), vec!["pnpm add --global --allow-build esbuild wrangler@1.2.3".to_string()]);
    assert_eq!(rig.runner.calls()[0].command, pnpm.to_str().expect("utf-8"));
}

#[test]
fn a_python_tool_needs_uv_and_installs_the_pinned_requirement() {
    let rig = Rig::new();
    assert!(install_tool(&rig.context(), &rig.tool("pipx:slopo")).is_err());
    rig.install_binary("uv", "uv");
    install_tool(&rig.context(), &rig.tool("pipx:slopo")).expect("installed");
    assert_eq!(rig.runner.lines(), vec!["uv tool install --force slopo==1.2.3".to_string()]);
}

#[test]
fn a_wheel_bootstrap_is_downloaded_verified_and_only_then_handed_to_uv() {
    let mut rig = Rig::new();
    rig.install_binary("uv", "uv");
    let wheel = b"wheel bytes";
    let url = "https://files.example/packages/meta_package_manager-8.2.0-py3-none-any.whl";
    rig.http.serve(url, wheel);
    rig.lock.tools[9].declaration.options.bootstrap =
        Some(super::model::Bootstrap { requires_tool: Some("uv".to_string()), distribution: "wheel".to_string() });
    set_artifact(&mut rig, "pipx:slopo", url, &sha256_hex(wheel));
    install_tool(&rig.context(), &rig.tool("pipx:slopo")).expect("installed");
    let downloaded = rig.scope.toolchain_root.join("downloads").join("meta_package_manager-8.2.0-py3-none-any.whl");
    assert_eq!(rig.runner.lines(), vec![format!("uv tool install --force {}", downloaded.display())]);
    assert_eq!(fs::read(&downloaded).expect("the verified wheel is in place"), wheel);
}

#[test]
fn a_download_whose_hash_differs_from_the_lock_is_discarded_and_nothing_runs() {
    let mut rig = Rig::new();
    rig.install_binary("uv", "uv");
    let url = "https://files.example/packages/tool.whl";
    rig.http.serve(url, b"tampered");
    rig.lock.tools[9].declaration.options.bootstrap =
        Some(super::model::Bootstrap { requires_tool: None, distribution: "wheel".to_string() });
    set_artifact(&mut rig, "pipx:slopo", url, &sha256_hex(b"expected"));
    let error = install_tool(&rig.context(), &rig.tool("pipx:slopo")).expect_err("the hash differs");
    assert!(error.message.contains(&format!("SHA-256 mismatch for {url}")), "{error}");
    assert!(error.message.contains(&sha256_hex(b"expected")), "{error}");
    assert!(error.message.contains(&sha256_hex(b"tampered")), "{error}");
    assert!(rig.runner.calls().is_empty(), "uv was never started");
    let downloads = rig.scope.toolchain_root.join("downloads");
    let leftovers: Vec<_> = fs::read_dir(&downloads).map(|entries| return entries.flatten().collect()).unwrap_or_default();
    assert!(leftovers.is_empty(), "the unverified file is removed: {leftovers:?}");
}

#[test]
fn a_download_failure_names_the_url() {
    let mut rig = Rig::new();
    set_artifact(&mut rig, "node", "https://nodejs.example/node.tar.gz", &sha256_hex(b"x"));
    rig.http.fail("https://nodejs.example/node.tar.gz", "connection reset");
    let error = install_tool(&rig.context(), &rig.tool("node")).expect_err("the download fails");
    assert!(error.message.contains("https://nodejs.example/node.tar.gz: connection reset"), "{error}");
}

#[test]
fn an_archive_is_verified_then_unpacked_with_tar() {
    let mut rig = Rig::new();
    let body = b"archive bytes";
    let url = "https://nodejs.example/dist/node-v1.2.3-linux-x64.tar.gz";
    rig.http.serve(url, body);
    set_artifact(&mut rig, "node", url, &sha256_hex(body));
    install_tool(&rig.context(), &rig.tool("node")).expect("installed");
    let download = rig.scope.toolchain_root.join("downloads").join("node-1.2.3-node-v1.2.3-linux-x64.tar.gz");
    let install_dir = tool_install_dir(&rig.scope.toolchain_root, "node", "1.2.3");
    assert_eq!(
        rig.runner.lines(),
        vec![format!("tar --extract --file {} --directory {}", download.display(), install_dir.display())]
    );
    assert!(download.is_file());
}

#[test]
fn a_zip_is_unpacked_with_unzip() {
    let mut rig = Rig::new();
    let body = b"zip bytes";
    let url = "https://x.example/tools.zip";
    rig.http.serve(url, body);
    set_artifact(&mut rig, "node", url, &sha256_hex(body));
    install_tool(&rig.context(), &rig.tool("node")).expect("installed");
    assert!(rig.runner.lines()[0].starts_with("unzip -q -o "), "{:?}", rig.runner.lines());
}

#[test]
fn a_failed_extraction_names_tar_and_the_archive() {
    let mut rig = Rig::new();
    let body = b"not really a tarball";
    let url = "https://x.example/node.tar.gz";
    rig.http.serve(url, body);
    set_artifact(&mut rig, "node", url, &sha256_hex(body));
    rig.runner.fail("tar", "not in gzip format");
    let error = install_tool(&rig.context(), &rig.tool("node")).expect_err("extraction fails");
    assert!(error.message.starts_with("tar failed for "), "{error}");
    assert!(error.message.ends_with("not in gzip format"), "{error}");
}

#[test]
fn a_single_binary_is_copied_under_the_first_bin_name_and_made_executable() {
    let mut rig = Rig::new();
    let body = b"#!/bin/sh\necho hi\n";
    let url = "https://github.com/marcosnils/bin/releases/download/v1.2.3/bin_1.2.3_linux_amd64";
    rig.http.serve(url, body);
    set_artifact(&mut rig, "bin", url, &sha256_hex(body));
    install_tool(&rig.context(), &rig.tool("bin")).expect("installed");
    let installed = tool_install_dir(&rig.scope.toolchain_root, "bin", "0.29.4").join("bin");
    assert_eq!(fs::read(&installed).expect("installed"), body);
    assert!(super::layout::is_executable(&installed));
    assert!(rig.runner.calls().is_empty());
}

#[test]
fn a_gzipped_single_binary_is_decompressed() {
    let mut rig = Rig::new();
    let compressed = gzip(b"the binary");
    let url = "https://github.com/tamasfe/taplo/releases/download/0.10.0/taplo-linux-x86_64.gz";
    rig.http.serve(url, &compressed);
    set_artifact(&mut rig, "uv", url, &sha256_hex(&compressed));
    rig.lock.tools[1].declaration.bins = vec!["taplo".to_string()];
    install_tool(&rig.context(), &rig.tool("uv")).expect("installed");
    let installed = tool_install_dir(&rig.scope.toolchain_root, "uv", "1.2.3").join("taplo");
    assert_eq!(fs::read(&installed).expect("installed"), b"the binary");
    assert!(super::layout::is_executable(&installed));
}

#[test]
fn a_tool_without_an_artifact_for_this_platform_says_so() {
    let mut rig = Rig::new();
    rig.host.platform = "linux-arm64-musl".to_string();
    let error = install_tool(&rig.context(), &rig.tool("node")).expect_err("no row");
    assert!(error.message.contains("no artifact for node on linux-arm64-musl"), "{error}");
}

#[test]
fn bin_sourced_installs_each_primary_through_bin_with_the_seeded_config_and_the_token() {
    let mut rig = Rig::new();
    rig.env = rig.env.with("GITHUB_TOKEN", "t0ken");
    rig.install_binary("bin", "bin");
    set_artifact(&mut rig, "pnpm", "https://github.com/pnpm/pnpm/releases/download/v1.2.3/pnpm-linux-x64", &sha256_hex(b"x"));
    install_tool(&rig.context(), &rig.tool("pnpm")).expect("installed");
    assert_eq!(
        rig.runner.lines(),
        vec!["bin install --force https://github.com/pnpm/pnpm/releases/tag/v1.2.3 --name pnpm-linux-x64/**/pnpm".to_string()],
        "the alias pn is linked, not downloaded"
    );
    let env = rig.runner.calls()[0].env.clone().expect("an environment");
    assert_eq!(env.get("GITHUB_TOKEN").map(String::as_str), Some("t0ken"));
    assert_eq!(env.get("BIN_CONFIG").map(String::as_str), Some(rig.scope.bin_config_file.to_str().expect("utf-8")));
    let config = fs::read_to_string(&rig.scope.bin_config_file).expect("the config is seeded");
    assert!(config.contains(rig.scope.toolchain_bin_dir.to_str().expect("utf-8")), "{config}");
}

#[test]
fn bin_sourced_falls_back_through_the_three_name_patterns_and_reports_the_last_failure() {
    let mut rig = Rig::new();
    rig.install_binary("bin", "bin");
    set_artifact(&mut rig, "pnpm", "https://github.com/pnpm/pnpm/releases/download/v1.2.3/asset.tgz", &sha256_hex(b"x"));
    rig.runner.fail("bin install", "no matching asset");
    let error = install_tool(&rig.context(), &rig.tool("pnpm")).expect_err("all patterns fail");
    let attempts: Vec<String> = rig.runner.lines().into_iter().filter(|line| return line.starts_with("bin install")).collect();
    assert_eq!(attempts.len(), 3);
    assert!(attempts[0].ends_with("--name asset.tgz/**/pnpm"));
    assert!(attempts[1].ends_with("--name asset.tgz/pnpm"));
    assert!(attempts[2].ends_with("--name asset.tgz pnpm"));
    assert_eq!(rig.runner.lines()[0], "gh auth token", "without GITHUB_TOKEN the token comes from gh, through the runner");
    assert!(error.message.contains("bin install (pnpm) failed for pnpm: no matching asset"), "{error}");
}

#[test]
fn bin_sourced_without_the_bin_cli_says_where_it_looked() {
    let mut rig = Rig::new();
    set_artifact(&mut rig, "pnpm", "https://github.com/pnpm/pnpm/releases/download/v1.2.3/a", &sha256_hex(b"x"));
    let error = install_tool(&rig.context(), &rig.tool("pnpm")).expect_err("no bin");
    assert!(error.message.contains("the bin CLI is not installed"), "{error}");
}

#[test]
fn release_tag_urls_are_derived_from_github_asset_urls_only() {
    assert_eq!(
        release_tag_url("https://github.com/o/r/releases/download/v1.0/a.tgz").expect("derived"),
        "https://github.com/o/r/releases/tag/v1.0"
    );
    assert_eq!(
        release_tag_url("https://github.com/adoptium/t/releases/download/jdk-21.0.13%2B9/x.tgz").expect("derived"),
        "https://github.com/adoptium/t/releases/tag/jdk-21.0.13+9"
    );
    for bad in [
        "https://example.com/o/r/releases/download/v1/a",
        "https://github.com/o/r/releases/latest",
        "https://github.com/o/r/archive/download/v1/a",
        "https://github.com//r/releases/download/v1/a",
    ] {
        assert!(release_tag_url(bad).is_err(), "{bad}");
    }
}

#[test]
fn percent_escapes_decode_and_malformed_ones_stay() {
    assert_eq!(percent_decode("a%2Bb%20c"), "a+b c");
    assert_eq!(percent_decode("100%"), "100%");
    assert_eq!(percent_decode("%zz"), "%zz");
    assert_eq!(percent_decode("%2"), "%2");
}

#[test]
fn the_bin_configuration_keeps_other_settings_and_replaces_a_malformed_file() {
    let rig = Rig::new();
    let config = rig.scope.root.join("bin.config.json");
    fs::write(&config, "{\"default_path\": \"/old\", \"bins\": {\"x\": 1}, \"keep\": true}").expect("write");
    seed_bin_config(&rig.scope.toolchain_bin_dir, &config).expect("seeded");
    let text = fs::read_to_string(&config).expect("read");
    assert!(text.contains("\"keep\": true"), "{text}");
    assert!(text.contains("\"x\": 1"), "{text}");
    assert!(!text.contains("/old"), "{text}");
    fs::write(&config, "not json").expect("write");
    seed_bin_config(&rig.scope.toolchain_bin_dir, &config).expect("seeded again");
    assert!(fs::read_to_string(&config).expect("read").contains("\"bins\": {}"));
}

#[test]
fn go_install_builds_the_locked_module_into_a_scoped_gobin() {
    let mut rig = Rig::new();
    rig.lock.tools[6].declaration.route = super::model::Route::GoInstall;
    rig.lock.tools[6].declaration.package_id = Some("example.com/tool/cmd".to_string());
    assert!(install_tool(&rig.context(), &rig.tool("cargo:fd-find")).is_err(), "go is not in the sample plan");
    let go_tool = rig.lock.tools[1].clone();
    let mut go = go_tool;
    go.declaration.id = "go".to_string();
    go.declaration.bins = vec!["go".to_string()];
    rig.lock.tools.push(go);
    rig.install_binary("go", "go");
    install_tool(&rig.context(), &rig.tool("cargo:fd-find")).expect("installed");
    assert_eq!(rig.runner.lines(), vec!["go install example.com/tool/cmd@v1.2.3".to_string()]);
    let env = rig.runner.calls()[0].env.clone().expect("environment");
    let gobin = tool_install_dir(&rig.scope.toolchain_root, "cargo:fd-find", "1.2.3").join("bin");
    assert_eq!(env.get("GOBIN").map(String::as_str), Some(gobin.to_str().expect("utf-8")));
}

#[test]
fn the_android_sdk_unpacks_command_line_tools_into_a_versioned_directory_then_installs_packages() {
    let mut rig = Rig::new();
    let body = b"cmdline tools zip";
    let url = "https://dl.google.example/commandlinetools-linux.zip";
    rig.http.serve(url, body);
    set_artifact(&mut rig, "android-sdk", url, &sha256_hex(body));
    rig.runner.on(
        "unzip",
        Box::new(|request| {
            let target = request.args.last().expect("unzip -d <dir>");
            write_executable(&std::path::Path::new(target).join("cmdline-tools").join("bin").join("android"));
        }),
    );
    install_tool(&rig.context(), &rig.tool("android-sdk")).expect("installed");
    let sdk_root = tool_install_dir(&rig.scope.toolchain_root, "android-sdk", "23.0");
    let android = sdk_root.join("cmdline-tools").join("23.0").join("bin").join("android");
    assert!(android.is_file());
    assert!(!sdk_root.join(".cmdline-tools-staging").exists(), "the staging directory is removed");
    let lines = rig.runner.lines();
    assert!(lines[0].starts_with("unzip -q -o "), "{lines:?}");
    assert_eq!(lines[1], "android sdk install platforms/android-37.0 platform-tools");
    let env = rig.runner.calls()[1].env.clone().expect("environment");
    assert_eq!(env.get("ANDROID_HOME").map(String::as_str), Some(sdk_root.to_str().expect("utf-8")));
    assert_eq!(env.get("ANDROID_SDK_ROOT").map(String::as_str), Some(sdk_root.to_str().expect("utf-8")));
}

#[test]
fn file_names_from_urls_are_safe_to_join_onto_a_directory() {
    assert_eq!(safe_file_name("https://x.example/a/b/tool-1.0.tar.gz"), "tool-1.0.tar.gz");
    assert_eq!(safe_file_name("https://x.example/a/b/tool.tgz?token=abc#frag"), "tool.tgz");
    assert_eq!(safe_file_name("https://x.example/a/.."), "download");
    assert_eq!(safe_file_name("https://x.example/a/."), "download");
    assert_eq!(safe_file_name("https://x.example/a/"), "download");
    assert_eq!(safe_file_name("https://x.example/a/we ird%2Fname"), "we-ird-2Fname");
}

#[test]
fn execute_bits_are_restored_on_named_launchers_within_the_depth_limit() {
    let rig = Rig::new();
    let root = rig.scope.root.join("unpacked");
    let deep = root.join("a").join("b").join("c").join("tool");
    let shallow = root.join("a").join("tool");
    let too_deep = root.join("a").join("b").join("c").join("d").join("tool");
    for path in [&deep, &shallow, &too_deep] {
        fs::create_dir_all(path.parent().expect("a parent")).expect("make");
        fs::write(path, "data").expect("write");
    }
    let fixed = restore_exec_bits(&root, &["tool".to_string()]);
    assert_eq!(fixed.len(), 2);
    assert!(super::layout::is_executable(&deep));
    assert!(super::layout::is_executable(&shallow));
    assert!(!super::layout::is_executable(&too_deep));
}

#[test]
fn an_atomic_symlink_replaces_an_existing_link_and_leaves_no_temporary_name() {
    let rig = Rig::new();
    let directory = rig.scope.root.join("links");
    let first = rig.scope.root.join("first");
    let second = rig.scope.root.join("second");
    fs::write(&first, "1").expect("write");
    fs::write(&second, "2").expect("write");
    atomic_symlink(&directory.join("name"), &first).expect("link");
    atomic_symlink(&directory.join("name"), &second).expect("relink");
    assert_eq!(fs::read_to_string(directory.join("name")).expect("through the link"), "2");
    let names: Vec<String> = fs::read_dir(&directory).expect("list").flatten().map(|entry| return entry.file_name().to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["name".to_string()]);
}
