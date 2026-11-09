//! Unit tests of the names that become directory names under the toolchain root:
//! the allowed names of the shipped lock,
//! the rejected crafted ones,
//! every boundary that must refuse them (the plan,
//! the lock,
//! the cleanup's reader of other worktrees' locks,
//! the path builders and the release resolver),
//! and the deletion a crafted lock used to cause.
//! Every deleting test runs in a disposable home inside the build directory.

use std::fs;
use std::path::Path;

use super::error::ErrorKind;
use super::fakes::{Rig, write_executable};
use super::layout::{tool_directory, tool_downloads_dir, tool_install_dir};
use super::lock::{parse_lock, render_lock};
use super::model::Request;
use super::plan::parse_plan;
use super::prune_cargo::prune_toolchain;
use super::published_archive::named_archives;
use super::segment::{RESERVED_TOOL_DIRECTORIES, check_bin_name, check_tool_id, check_version, name_problem};
use super::update_resolve::resolve_tool;

/// A plan with one tool whose id,
/// bins and options the test supplies as raw JSON text.
fn plan_with(id: &str, entry_fields: &str) -> String {
    return format!("{{\"schemaVersion\": 1, \"tools\": {{{id:?}: {{\"request\": \"node@latest\", \"route\": \"archive\"{entry_fields}}}}}}}");
}

#[test]
fn the_names_of_the_shipped_lock_are_accepted() {
    for id in [
        "node",
        "bin",
        "uv",
        "llama.cpp",
        "cargo:fd-find",
        "conda:bubblewrap",
        "npm:@actions/languageserver",
        "github:docker/docker-language-server",
        "pipx:slopo",
    ] {
        assert_eq!(check_tool_id(id), Ok(()), "{id}");
    }
    for version in ["26.13.0", "0.29.4", "temurin-21.0.13+9.0.LTS", "b9829", "2026-02-08", "nightly-2026-10-31", "v1.2.3", "1"] {
        assert_eq!(check_version(version), Ok(()), "{version}");
    }
    for name in ["mpm-shadow", "taplo-linux-x86_64", "wrangler2", "cf-wrangler"] {
        assert_eq!(check_bin_name(name), Ok(()), "{name}");
    }
}

#[test]
fn a_crafted_tool_id_is_refused_and_the_message_names_it() {
    for id in ["..", ".", "", ".hidden", "cargo", "downloads", "exposure", "launchers", "bin.config.json"] {
        let message = check_tool_id(id).expect_err(id);
        assert!(message.contains(&format!("{id:?}")), "the message names the id {id:?}: {message}");
        assert!(message.contains("toolchain.plan.jsonc"), "and says what to do: {message}");
    }
    for reserved in RESERVED_TOOL_DIRECTORIES {
        assert!(check_tool_id(reserved).is_err(), "{reserved}");
    }
    // An id with separators is not refused: `install_dir_name` turns them into a single dash.
    assert_eq!(check_tool_id("a/../b"), Ok(()));
    assert_eq!(check_tool_id("a\\b\0c"), Ok(()));
}

#[test]
fn a_crafted_version_is_refused_and_the_message_names_it() {
    for version in ["", ".", "..", "../x", "x/..", "a/b", "a\\b", "a\0b", ".hidden", ".0.2.1.staging.1", "current"] {
        let message = check_version(version).expect_err(version);
        assert!(message.contains(&format!("{version:?}")), "the message names the version {version:?}: {message}");
    }
}

#[test]
fn a_crafted_bin_name_is_refused() {
    for name in ["", ".", "..", "../x", "a/b", "a\\b", "a\0b"] {
        assert!(check_bin_name(name).is_err(), "{name:?}");
    }
    assert_eq!(name_problem(".hidden"), None, "a bin may start with a dot; only the installer's own directories may not");
}

#[test]
fn the_path_builders_refuse_a_name_that_leaves_the_directory() {
    let root = Path::new("/data/toolchain");
    for id in ["..", ".", "", "cargo", "downloads"] {
        assert!(tool_directory(root, id).is_err(), "{id:?}");
        assert!(tool_install_dir(root, id, "1.0.0").is_err(), "{id:?}");
        assert!(tool_downloads_dir(root, id, "1.0.0").is_err(), "{id:?}");
    }
    for version in ["", ".", "..", "../x", "a/b", "a\\b", "a\0b", "current"] {
        let error = tool_install_dir(root, "node", version).expect_err(version);
        assert!(error.message.contains(&format!("{version:?}")), "{error}");
        assert!(tool_downloads_dir(root, "node", version).is_err(), "{version:?}");
    }
    assert_eq!(tool_install_dir(root, "node", "26.13.0").expect("plain names"), Path::new("/data/toolchain/node/26.13.0"));
    assert_eq!(tool_downloads_dir(root, "node", "26.13.0").expect("plain names"), Path::new("/data/toolchain/downloads/node/26.13.0"));
}

#[test]
fn the_plan_refuses_a_crafted_tool_id_bin_name_alias_or_package_name() {
    let accepted = parse_plan(&plan_with("tools-ok", ", \"bins\": [\"ok\"]"), "toolchain.plan.jsonc").expect("a plain plan parses");
    assert_eq!(accepted.tools[0].id, "tools-ok");
    let cases: [(&str, String); 8] = [
        ("..", plan_with("..", "")),
        (".", plan_with(".", "")),
        ("cargo", plan_with("cargo", "")),
        ("../x", plan_with("ok", ", \"bins\": [\"../x\"]")),
        ("a/b", plan_with("ok", ", \"bins\": [\"a/b\"]")),
        ("../x", plan_with("ok", ", \"options\": {\"binAliases\": {\"../x\": \"ok\"}}")),
        ("", plan_with("ok", ", \"options\": {\"binAliases\": {\"\": \"ok\"}}")),
        ("..", "{\"schemaVersion\": 1, \"tools\": {\"cargo:x\": {\"request\": \"cargo:x@latest\", \"route\": \"cargo-binstall\", \"packageId\": \"..\"}}}".to_string()),
    ];
    for (named, source) in cases {
        let error = parse_plan(&source, "toolchain.plan.jsonc").expect_err(&source);
        assert_eq!(error.kind, ErrorKind::Config, "{source}");
        assert!(error.message.contains(&format!("{named:?}")), "the message names {named:?}: {error}");
    }
    let by_request = "{\"schemaVersion\": 1, \"tools\": {\"cargo:x\": {\"request\": \"cargo:..@latest\", \"route\": \"cargo-binstall\"}}}";
    assert!(parse_plan(by_request, "toolchain.plan.jsonc").is_err(), "the crate name in the request becomes a directory too");
}

#[test]
fn the_lock_refuses_a_version_or_id_that_leaves_the_tool_directory() {
    let rendered = render_lock(&super::fixtures::sample_lock());
    assert!(parse_lock(&rendered, "toolchain.lock.json").is_ok());
    for crafted in ["../x", "..", "a/b", "a\\b", ".hidden", "current"] {
        let source = rendered.replacen("\"version\": \"1.2.3\"", &format!("\"version\": {crafted:?}"), 1);
        let error = parse_lock(&source, "toolchain.lock.json").expect_err(crafted);
        assert!(error.message.contains(&format!("{crafted:?}")), "{error}");
        assert!(error.message.contains(".version"), "the message names the field: {error}");
    }
    let renamed = rendered.replacen("\"node\": {", "\"..\": {", 1);
    assert!(parse_lock(&renamed, "toolchain.lock.json").is_err(), "the lock's tool id is checked too");
}

/// A lock text that names a release archive of this repository under `id`,
/// pinned at `version`,
/// with `first_bin` as its first program.
fn archive_lock(id: &str, version: &str, first_bin: &str) -> String {
    return format!(
        "{{\"tools\":{{{id:?}:{{\"request\":\"github:Aquaticats/Monochromatic@latest\",\"version\":{version:?},\"route\":\"archive\",\
         \"bins\":[{first_bin:?}],\"options\":{{\"tagPrefix\":\"monochromatic-x-v\"}}}}}}}}"
    );
}

#[test]
fn the_cleanup_refuses_a_lock_that_names_a_release_archive_under_a_crafted_name() {
    assert!(named_archives(&archive_lock("x", "1.0.0", "marker"), "the lock").is_ok());
    for (id, version, first_bin) in [("..", "1.0.0", "marker"), ("x", "../y", "marker"), ("x", "1.0.0", "../marker"), (".", "1.0.0", "m"), ("cargo", "1.0.0", "m")] {
        let error = named_archives(&archive_lock(id, version, first_bin), "/work/toolchain.lock.json").expect_err(id);
        assert!(error.message.contains("/work/toolchain.lock.json"), "the message names the lock: {error}");
        assert!(error.message.contains("nothing was deleted"), "{error}");
    }
    // A crafted entry that is not a repository release names nothing, so it is not judged and does not stop the cleanup.
    let foreign = "{\"tools\":{\"..\":{\"version\":\"../x\",\"route\":\"archive\",\"bins\":[\"m\"]}}}";
    assert!(named_archives(foreign, "the lock").expect("reads").is_empty());
}

#[test]
fn a_crafted_lock_cannot_make_the_cleanup_delete_the_toolchain_root() {
    let rig = Rig::new();
    let root = rig.scope.toolchain_root.clone();
    let precious = root.join("precious").join("file");
    fs::create_dir_all(precious.parent().expect("a parent")).expect("make");
    fs::write(&precious, "keep me").expect("write");
    // The id `..` makes the tool's directory the parent of the toolchain root, so its children look like versions:
    // `toolchain` (the root itself) reads as an unfinished 0.0.0 and `9.0.0` as a finished newer version.
    write_executable(&root.parent().expect("a parent").join("9.0.0").join("marker"));
    fs::write(rig.scope.root.join("toolchain.lock.json"), archive_lock("..", "1.0.0", "marker")).expect("write the lock");
    rig.runner.answer("git -C", &format!("worktree {}\nHEAD a\n\n", rig.scope.root.display()), 0);

    let error = prune_toolchain(&rig.scope, &rig.runner, &[], false).expect_err("refused");
    assert!(error.message.contains("tools.."), "{error}");
    assert!(precious.is_file(), "the toolchain root and what is in it are untouched");
    assert!(root.is_dir());
}

#[test]
fn a_release_whose_tag_is_not_a_plain_name_is_refused_before_it_reaches_the_lock() {
    let rig = Rig::new();
    let mut declaration = rig.tool("pnpm").declaration;
    declaration.request = Request::parse("github:o/r@latest").expect("parses");
    let asset = "tool-linux-x64.tar.gz";
    let release = |tag: &str| {
        return format!(
            "{{\"tag_name\": {tag:?}, \"assets\": [{{\"name\": \"{asset}\", \"browser_download_url\": \"https://github.com/o/r/releases/download/x/{asset}\", \"digest\": \"sha256:{}\"}}]}}",
            "0".repeat(64)
        );
    };
    rig.http.serve("https://api.github.com/repos/o/r/releases/latest", release("v2.0.0").as_bytes());
    assert_eq!(resolve_tool(&rig.context(), &declaration, None).expect("plain").version, "2.0.0");

    let crafted = Rig::new();
    crafted.http.serve("https://api.github.com/repos/o/r/releases/latest", release("release/../../x").as_bytes());
    let error = resolve_tool(&crafted.context(), &declaration, None).expect_err("a tag with separators");
    assert!(error.message().contains("release/../../x"), "{error:?}");
}
