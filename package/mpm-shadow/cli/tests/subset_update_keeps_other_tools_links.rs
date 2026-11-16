//! End-to-end test of what a command that converges only a subset of the plan does to postinstall links.
//! `update <one tool>` installs the tools its lookup needs before it looks that tool up again,
//! and that partial pass must retire only the links of the tools it covers:
//! a link another tool made stays where it is and stays in the record,
//! even when that other tool stopped declaring it,
//! so a command that runs through that link at that moment never finds it missing.
//! A tool that left the plan altogether still loses its links through the command that converges the whole plan.
//! The built `mpm-shadow` binary runs against a disposable checkout and a disposable home,
//! so nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`,
//! and the failed lookup this fixture relies on needs no network and no outside host.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::SystemTime;

use serde_json::{Map, Value};

use mpm_shadow::toolchain::lock::{parse_lock, render_lock};
use mpm_shadow::toolchain::scope::{LOCK_FILE_NAME, PLAN_FILE_NAME};

/// The version every fixture tool is locked at.
const VERSION: &str = "1.0.0";

/// The whole-content hash the fixture archive rows carry.
/// Nothing downloads them (the fixture's store already holds `VERSION`),
/// so any well-formed row serves.
const SHA256: &str = "439e9c438acdf6a6ffe47418da709153d66c4bed812e30a473faacb107dd2791";

/// The tool whose failed lookup makes `update` install the subset.
/// Its plan entry asks for the version text `release/1.2`,
/// which cannot name a directory under the toolchain root,
/// so the lookup fails on any machine with no network and no outside host;
/// its lock row still holds the request that edit replaced,
/// so the exact-version shortcut cannot answer it from the lock instead.
const TRIGGER: &str = "fixture-npm";

/// The request the trigger's plan entry holds.
const TRIGGER_REQUEST: &str = "git:example/fixture-npm@release/1.2";

/// The request the trigger's lock row still holds.
const TRIGGER_LOCK_REQUEST: &str = "git:example/fixture-npm@release/1.1";

/// One postinstall link declaration, as the plan and the lock store it.
struct Link {
    /// The declared bin the link points at.
    source_bin: String,
    /// The name of the link.
    link_name: String,
    /// The directory of the link, relative to the checkout.
    link_dir: String,
}

/// One fixture tool, in both documents.
struct Tool {
    /// The id, the key of both documents.
    id: String,
    /// The route text.
    route: String,
    /// The request text of the plan entry.
    request: String,
    /// The request text of the lock row, when the plan was edited past it.
    lock_request: Option<String>,
    /// The package the route installs, when it differs from the request's own name.
    package_id: Option<String>,
    /// The manager the route installs through.
    manager: Option<String>,
    /// The bins the tool declares.
    bins: Vec<String>,
    /// The postinstall link the tool declares.
    link: Option<Link>,
}

/// A linked archive tool of another repository:
/// a pinned request,
/// its own program as its only bin,
/// and a postinstall link in a directory of its own.
fn linked(id: &str, request: &str, link_name: &str) -> Tool {
    return Tool {
        id: id.to_string(),
        route: "archive".to_string(),
        request: request.to_string(),
        lock_request: None,
        package_id: None,
        manager: None,
        bins: vec![id.to_string()],
        link: Some(Link { source_bin: id.to_string(), link_name: link_name.to_string(), link_dir: format!(".cache/{id}-bin") }),
    };
}

/// The same tool, with the link its plan entry and lock row no longer declare.
fn without_link(tool: Tool) -> Tool {
    let mut dropped: Tool = tool;
    dropped.link = None;
    return dropped;
}

/// The npm tool whose failed lookup makes `update` install a subset.
fn trigger() -> Tool {
    return Tool {
        id: TRIGGER.to_string(),
        route: "mpm-pnpm".to_string(),
        request: TRIGGER_REQUEST.to_string(),
        lock_request: Some(TRIGGER_LOCK_REQUEST.to_string()),
        package_id: Some(TRIGGER.to_string()),
        manager: Some("pnpm".to_string()),
        bins: vec![TRIGGER.to_string()],
        link: None,
    };
}

/// `text` as a JSON string.
fn text_value(text: &str) -> Value {
    return Value::String(text.to_string());
}

/// The JSON array of these texts.
fn text_array(items: &[String]) -> Value {
    let mut values: Vec<Value> = Vec::new();
    for item in items {
        values.push(text_value(item));
    }
    return Value::Array(values);
}

/// The JSON object of these entries, in the order given.
fn object(entries: &[(&str, Value)]) -> Value {
    let mut map: Map<String, Value> = Map::new();
    for (key, value) in entries {
        map.insert((*key).to_string(), value.clone());
    }
    return Value::Object(map);
}

/// The request text of `tool`'s lock row.
fn lock_request(tool: &Tool) -> String {
    return match &tool.lock_request {
        Some(edited) => edited.clone(),
        None => tool.request.clone(),
    };
}

/// The postinstall object both documents store for `tool`.
fn postinstall_value(link: &Link) -> Value {
    let inner: Value = object(&[
        ("sourceBin", text_value(&link.source_bin)),
        ("linkName", text_value(&link.link_name)),
        ("linkDir", text_value(&link.link_dir)),
    ]);
    return object(&[("when", text_value("always")), ("link", inner)]);
}

/// The plan entry of `tool`, as a person writes it.
fn plan_entry(tool: &Tool) -> Value {
    let mut entries: Vec<(&str, Value)> = vec![("request", text_value(&tool.request)), ("route", text_value(&tool.route))];
    if let Some(manager) = &tool.manager {
        entries.push(("manager", text_value(manager)));
    }
    if let Some(package_id) = &tool.package_id {
        entries.push(("packageId", text_value(package_id)));
    }
    entries.push(("bins", text_array(&tool.bins)));
    if let Some(link) = &tool.link {
        entries.push(("options", object(&[("postinstall", postinstall_value(link))])));
    }
    return object(&entries);
}

/// The lock row of `tool`: its plan entry plus the resolved version and the archive rows.
fn lock_entry(tool: &Tool) -> Value {
    let mut entries: Vec<(&str, Value)> = vec![
        ("request", text_value(&lock_request(tool))),
        ("version", text_value(VERSION)),
        ("route", text_value(&tool.route)),
    ];
    if let Some(manager) = &tool.manager {
        entries.push(("manager", text_value(manager)));
    }
    if let Some(package_id) = &tool.package_id {
        entries.push(("packageId", text_value(package_id)));
    }
    entries.push(("bins", text_array(&tool.bins)));
    entries.push(("env", object(&[])));
    let mut options: Vec<(&str, Value)> = Vec::new();
    if let Some(link) = &tool.link {
        options.push(("postinstall", postinstall_value(link)));
    }
    entries.push(("options", object(&options)));
    let mut artifacts: Map<String, Value> = Map::new();
    if tool.route == "archive" {
        for platform in ["linux-arm64", "linux-x64"] {
            let url: String = format!("https://example.invalid/{}-{VERSION}.tar.gz", tool.id);
            artifacts.insert(platform.to_string(), object(&[("url", text_value(&url)), ("sha256", text_value(SHA256))]));
        }
    }
    entries.push(("artifacts", Value::Object(artifacts)));
    return object(&entries);
}

/// The program stand-in of one tool:
/// the pnpm one answers its global listing,
/// every other one does nothing.
fn program_text(id: &str) -> String {
    if id == "pnpm" {
        return "#!/bin/sh\ncase \"$*\" in\n  \"list --global --depth=0 --json\"*) printf '[{\"dependencies\":{}}]\\n'; exit 0 ;;\nesac\necho \"pnpm stand-in: this fixture has no npm registry\" >&2\nexit 1\n"
            .to_string();
    }
    return "#!/bin/sh\nexit 0\n".to_string();
}

/// A disposable home, a disposable checkout, and an installed program per tool that is not the trigger.
struct Fixture {
    /// The directory everything sits in.
    directory: tempfile::TempDir,
}

/// The methods of `Fixture`.
impl Fixture {
    /// An empty fixture: a home and a checkout, with no plan yet.
    fn new() -> Fixture {
        let directory = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a fixture root");
        fs::create_dir_all(directory.path().join("home")).expect("make the home");
        fs::create_dir_all(directory.path().join("checkout")).expect("make the checkout");
        return Fixture { directory };
    }

    /// The disposable home.
    fn home(&self) -> PathBuf {
        return self.directory.path().join("home");
    }

    /// The disposable checkout.
    fn checkout(&self) -> PathBuf {
        return self.directory.path().join("checkout");
    }

    /// The toolchain store of the repository scope,
    /// where each installed tool's version directory lives.
    fn store(&self) -> PathBuf {
        return self.home().join(".local/share/monochromatic-dev/toolchain");
    }

    /// The record of the postinstall links, as text.
    fn record(&self) -> String {
        return fs::read_to_string(self.checkout().join(".cache/toolchain-launchers/.postinstall-links.json")).expect("the record exists");
    }

    /// Writes this round's plan and signed lock,
    /// and the installed program of every tool that is not the trigger.
    fn write(&self, tools: &[Tool]) {
        let mut plan_tools: Map<String, Value> = Map::new();
        let mut lock_tools: Map<String, Value> = Map::new();
        for tool in tools {
            plan_tools.insert(tool.id.clone(), plan_entry(tool));
            lock_tools.insert(tool.id.clone(), lock_entry(tool));
        }
        let plan: Value = object(&[("schemaVersion", Value::from(1)), ("tools", Value::Object(plan_tools))]);
        let plan_text: String = serde_json::to_string_pretty(&plan).expect("an in-memory value always serializes");
        fs::write(self.checkout().join(PLAN_FILE_NAME), format!("{plan_text}\n")).expect("write the plan");
        let unsigned: Value = object(&[
            ("schemaVersion", Value::from(1)),
            ("generatedFrom", text_value(PLAN_FILE_NAME)),
            ("tools", Value::Object(lock_tools)),
        ]);
        let parsed = parse_lock(&unsigned.to_string(), "the fixture lock").expect("the fixture lock parses");
        fs::write(self.checkout().join(LOCK_FILE_NAME), render_lock(&parsed.lock)).expect("write the signed lock");

        for tool in tools {
            if tool.id == TRIGGER {
                continue;
            }
            let bin: PathBuf = self.store().join(&tool.id).join(VERSION).join("bin").join(&tool.id);
            fs::create_dir_all(bin.parent().expect("a parent")).expect("make the install directory");
            fs::write(&bin, program_text(&tool.id)).expect("write the program");
            fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).expect("make it executable");
        }
    }

    /// Runs one mpm-shadow command in the checkout, with a cleared environment.
    fn run(&self, arguments: &[&str]) -> Output {
        return Command::new(env!("CARGO_BIN_EXE_mpm-shadow"))
            .args(arguments)
            .current_dir(self.checkout())
            .env_clear()
            .env("HOME", self.home())
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("run mpm-shadow");
    }
}

/// Standard output and error as text.
fn text(output: &Output) -> String {
    return format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

/// The link path of the link `link_name` of the tool `id`, in the fixture's layout.
fn link_path(fixture: &Fixture, id: &str, link_name: &str) -> PathBuf {
    return fixture.checkout().join(format!(".cache/{id}-bin")).join(link_name);
}

/// What the link at `path` points at, and when it was last written.
fn link_state(path: &Path) -> (PathBuf, SystemTime) {
    let target: PathBuf = fs::read_link(path).expect("the link is a link");
    let modified: SystemTime = fs::symlink_metadata(path).expect("the link exists").modified().expect("the link has a modification time");
    return (target, modified);
}

/// Whether anything, even a dangling link, sits at `path`.
fn exists_at(path: &Path) -> bool {
    return fs::symlink_metadata(path).is_ok();
}

#[test]
fn a_subset_update_retires_no_link_of_a_tool_it_does_not_cover() {
    let fixture: Fixture = Fixture::new();
    let one: Tool = linked("fixture-one", "github:example/fixture-one@1.0.0", "one");
    let two: Tool = linked("fixture-two", "github:example/fixture-two@1.0.0", "two");
    let node: Tool = linked("node", "github:example/node@1.0.0", "node-link");
    let pnpm: Tool = linked("pnpm", "github:example/pnpm@1.0.0", "pnpm-link");

    // Round 1: both linked tools and the two prerequisites of an npm tool, all installed and all declared.
    fixture.write(&[one, two, node, pnpm]);
    let first: Output = fixture.run(&["install"]);
    assert_eq!(first.status.code(), Some(0), "{}", text(&first));
    let one_path: PathBuf = link_path(&fixture, "fixture-one", "one");
    let two_path: PathBuf = link_path(&fixture, "fixture-two", "two");
    let pnpm_path: PathBuf = link_path(&fixture, "pnpm", "pnpm-link");
    let one_before: (PathBuf, SystemTime) = link_state(&one_path);
    let two_before: (PathBuf, SystemTime) = link_state(&two_path);

    // Round 2: fixture-one and pnpm stop declaring their links, and the trigger joins the plan.
    let reloaded_one: Tool = linked("fixture-one", "github:example/fixture-one@1.0.0", "one");
    let reloaded_pnpm: Tool = linked("pnpm", "github:example/pnpm@1.0.0", "pnpm-link");
    let reloaded_two: Tool = linked("fixture-two", "github:example/fixture-two@1.0.0", "two");
    let reloaded_node: Tool = linked("node", "github:example/node@1.0.0", "node-link");
    fixture.write(&[without_link(reloaded_one), reloaded_two, reloaded_node, without_link(reloaded_pnpm), trigger()]);
    let updated: Output = fixture.run(&["update", TRIGGER]);
    let update_text: String = text(&updated);
    assert_eq!(updated.status.code(), Some(1), "{update_text}");
    assert!(update_text.contains("looked up twice"), "the failed lookup made update install a subset: {update_text}");

    assert_eq!(link_state(&one_path), one_before, "the link of a tool the pass never covered is neither removed nor rewritten: {update_text}");
    assert_eq!(link_state(&two_path), two_before, "the other link of a tool the pass never covered is neither removed nor rewritten: {update_text}");
    assert!(!exists_at(&pnpm_path), "the link of a tool the pass covers, dropped from the lock, is retired: {update_text}");
    assert!(
        fixture.record().contains(&one_path.display().to_string()),
        "the record still remembers the link the pass left alone:\n{}",
        fixture.record()
    );

    // Round 3: fixture-one leaves the plan, so the command that converges the whole plan retires its link.
    let last_two: Tool = linked("fixture-two", "github:example/fixture-two@1.0.0", "two");
    fixture.write(&[last_two]);
    let converged: Output = fixture.run(&["install"]);
    let converged_text: String = text(&converged);
    assert_eq!(converged.status.code(), Some(0), "{converged_text}");
    assert!(converged_text.contains("retired the postinstall link"), "{converged_text}");
    assert!(!exists_at(&one_path), "the link of a tool that left the plan is retired by the whole plan: {converged_text}");
    assert!(exists_at(&two_path), "the link of a tool still in the plan stays: {converged_text}");
}
