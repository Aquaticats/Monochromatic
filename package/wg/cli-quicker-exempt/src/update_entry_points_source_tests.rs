//! Checks the update entry-point list against the sources that say what exists and what fetches:
//! the root and package `mise.toml` tasks,
//! the `mpm-shadow` verb grammar
//! and the TypeScript launcher.
//! A name the sources no longer have fails here,
//! and so does a fetching task or verb the list lacks.

/// The list under test.
use crate::update_entry_points::{MISE_TASKS, MPM_SHADOW_VERBS, TOOLCHAIN_SCRIPT_TAILS};
/// Repository paths.
use std::path::{Path, PathBuf};

/// `mpm-shadow` verbs that open a network connection,
/// as `package/mpm-shadow/cli/README.md` documents them:
/// `install` downloads what the lock names,
/// `update` and `outdated` look versions up.
const FETCHING_VERBS: &[&str] = &["install", "update", "outdated"];

/// `mpm-shadow` verbs that read the lock,
/// the file system and archives already on disk,
/// and open no connection.
const OFFLINE_VERBS: &[&str] = &["list", "bin", "verify-archive", "prune"];

/// Root tasks that must be found as fetching,
/// proving the detection can see a task that runs `mpm-shadow` and an alias of one.
const DETECTION_POSITIVE_CONTROL: &[&str] = &["toolchain:install", "toolchain:update:global", "bootstrap", "prepare:android"];

/// Root tasks that read the lock and the file system only,
/// so the detection must not find them fetching and the list must not name them.
const OFFLINE_TASKS: &[&str] = &["toolchain:list", "toolchain:list:global", "toolchain:lock:check"];

/// One task of a `mise.toml` file.
struct Task {
    /// The name `mise run` takes:
    /// plain in the root file,
    /// `//package/path:name` in a package file.
    name: String,
    /// The aliases `mise run` also takes.
    aliases: Vec<String>,
    /// The task's own lines,
    /// without comments,
    /// `description` and `usage`.
    body: String,
}

/// The repository root,
/// three levels above this package's `package/wg/cli-quicker-exempt` directory.
fn repository_root() -> PathBuf {
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// Reads one repository file.
fn read_repository_file(relative: &str) -> String {
    return std::fs::read_to_string(repository_root().join(relative)).expect(relative);
}

/// The quoted strings of a line,
/// in order.
fn quoted_strings(line: &str) -> Vec<String> {
    return line.split('"').skip(1).step_by(2).map(str::to_string).collect();
}

/// Counts the multi-line string delimiters on a line.
fn string_delimiters(line: &str) -> usize {
    return line.matches("\"\"\"").count() + line.matches("'''").count();
}

/// Reads the task name of a `[tasks.<name>]` header,
/// or `None` for another table or a sub-table such as `[tasks."lint".env]`.
fn header_task_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("[tasks.")?;
    if let Some(quoted) = rest.strip_prefix('"') {
        let end = quoted.find('"')?;
        return match &quoted[end + 1..] {
            "]" => Some(quoted[..end].to_string()),
            _ => None,
        };
    }
    let end = rest.find(']')?;
    return match &rest[end..] {
        "]" if !rest[..end].contains('.') => Some(rest[..end].to_string()),
        _ => None,
    };
}

/// Splits the text of one `mise.toml` into its tasks.
/// `prefix` is empty for the root file and `//package/path:` for a package file.
/// Lines inside multi-line strings belong to the task whatever they start with.
fn parse_tasks(text: &str, prefix: &str) -> Vec<Task> {
    let mut tasks: Vec<Task> = Vec::new();
    let mut inside_task = false;
    let mut inside_string = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if !inside_string {
            if line.starts_with('[') {
                inside_task = false;
                if let Some(name) = header_task_name(line) {
                    tasks.push(Task { name: format!("{prefix}{name}"), aliases: Vec::new(), body: String::new() });
                    inside_task = true;
                }
                continue;
            }
            if trimmed.is_empty() || trimmed.starts_with('#') || !inside_task {
                continue;
            }
        }
        let was_inside_string = inside_string;
        if string_delimiters(line) % 2 == 1 {
            inside_string = !inside_string;
        }
        let Some(task) = tasks.last_mut() else {
            continue;
        };
        if !was_inside_string && trimmed.starts_with("alias") {
            task.aliases.extend(quoted_strings(line).into_iter().map(|alias| return format!("{prefix}{alias}")));
            continue;
        }
        if !was_inside_string && (trimmed.starts_with("description") || trimmed.starts_with("usage")) {
            continue;
        }
        task.body.push_str(line);
        task.body.push('\n');
    }
    return tasks;
}

/// Collects `mise.toml` files below a directory,
/// leaving out build output and installed dependencies.
fn collect_mise_files(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            if name != "node_modules" && name != "target" && name != "dist" && name != ".git" {
                collect_mise_files(&path, found);
            }
        } else if name == "mise.toml" {
            found.push(path);
        }
    }
}

/// Every task of the root `mise.toml` and of every package `mise.toml`.
fn all_tasks() -> Vec<Task> {
    let root = repository_root();
    let mut tasks = parse_tasks(&read_repository_file("mise.toml"), "");
    let mut files: Vec<PathBuf> = Vec::new();
    collect_mise_files(&root.join("package"), &mut files);
    for file in files {
        let directory = file.parent().expect("a file has a parent").strip_prefix(&root).expect("below the root");
        let prefix = format!("//{}:", directory.to_string_lossy());
        let text = std::fs::read_to_string(&file).expect("a listed file reads");
        tasks.extend(parse_tasks(&text, &prefix));
    }
    return tasks;
}

/// The words of each `mpm-shadow` call in a task body:
/// `runMpmShadow(['install', '--global'])` and a shell line `mpm-shadow install --global`.
fn mpm_shadow_calls(body: &str) -> Vec<Vec<String>> {
    let mut calls: Vec<Vec<String>> = Vec::new();
    for part in body.split("runMpmShadow([").skip(1) {
        let inside = part.split("])").next().unwrap_or("");
        calls.push(inside.split(',').map(|word| return word.trim().trim_matches(['\'', '"']).to_string()).collect());
    }
    for part in body.split("mpm-shadow ").skip(1) {
        let line = part.lines().next().unwrap_or("");
        calls.push(line.split_whitespace().map(|word| return word.trim_matches(['\'', '"', ',']).to_string()).collect());
    }
    return calls;
}

/// Reports whether an `mpm-shadow` call fetches:
/// its verb is a fetching verb,
/// and it is not `install --lockfile-only` combined with `--frozen-lockfile` or `--offline`,
/// the two forms the README documents as opening no connection.
fn call_fetches(words: &[String]) -> bool {
    let Some(verb) = words.iter().find(|word| return !word.starts_with('-')) else {
        return false;
    };
    if !FETCHING_VERBS.contains(&verb.as_str()) {
        return false;
    }
    let lock_only = words.iter().any(|word| return word == "--lockfile-only");
    let resolves_nothing = words.iter().any(|word| return word == "--frozen-lockfile" || word == "--offline");
    return !(lock_only && resolves_nothing);
}

/// The tasks a body starts with `mise run <task>`.
fn mise_run_targets(body: &str) -> Vec<String> {
    let mut targets: Vec<String> = Vec::new();
    for part in body.split("mise run ").skip(1) {
        let end = part.find(|character: char| return character.is_whitespace() || "\"',]".contains(character));
        targets.push(part[..end.unwrap_or(part.len())].to_string());
    }
    return targets;
}

/// The names and aliases of every task that fetches:
/// one that calls a fetching `mpm-shadow` verb,
/// or one that runs a task that does.
fn fetching_names(tasks: &[Task]) -> Vec<String> {
    let mut fetching: Vec<String> = Vec::new();
    loop {
        let before = fetching.len();
        for task in tasks {
            if fetching.contains(&task.name) {
                continue;
            }
            let calls = mpm_shadow_calls(&task.body).iter().any(|words| return call_fetches(words));
            let delegates = mise_run_targets(&task.body).iter().any(|target| return fetching.contains(target));
            if calls || delegates {
                fetching.push(task.name.clone());
                fetching.extend(task.aliases.iter().cloned());
            }
        }
        if fetching.len() == before {
            return fetching;
        }
    }
}

/// Parses the verbs a grammar source declares:
/// each `Verb { name: "...", aliases: &["..."], build: ... }` literal as its name and aliases.
fn declared_verbs(text: &str) -> Vec<(String, Vec<String>)> {
    let mut verbs: Vec<(String, Vec<String>)> = Vec::new();
    for part in text.split("Verb {").skip(1) {
        let literal = part.split('}').next().unwrap_or("");
        if !literal.contains("build:") {
            continue;
        }
        let Some(after_name) = literal.split("name: \"").nth(1) else {
            continue;
        };
        let name = after_name.split('"').next().unwrap_or("").to_string();
        let aliases = match literal.split("aliases: &[").nth(1) {
            Some(after) => quoted_strings(after.split(']').next().unwrap_or("")),
            None => Vec::new(),
        };
        verbs.push((name, aliases));
    }
    return verbs;
}

/// Every verb `mpm-shadow` declares:
/// the toolchain verbs and `prune`.
fn grammar_verbs() -> Vec<(String, Vec<String>)> {
    let mut verbs = declared_verbs(&read_repository_file("package/mpm-shadow/cli/src/toolchain/verb_grammar.rs"));
    verbs.extend(declared_verbs(&read_repository_file("package/mpm-shadow/cli/src/cli.rs")));
    return verbs;
}

/// A fixture `mise.toml` with the shapes that must not fool the reader.
const FIXTURE_MISE: &str = r##"
[tasks."alpha"]
description = "mise run beta mentions a task in prose"
alias = "a"
run = """
[not-a-header]
runMpmShadow(['update'])
"""

# mise run alpha in a comment
[tasks.beta]
run = "echo hello"
# mise run alpha in a comment inside the task

[tasks."beta".env]
X = "mise run alpha"

[tasks.gamma]
run = ["mise run alpha"]

[tasks.delta]
run = "runMpmShadow(['install', '--lockfile-only', '--frozen-lockfile'])"

[tasks.epsilon]
run = "runMpmShadow(['install', '--lockfile-only', '--offline'])"

[tasks.zeta]
run = "mpm-shadow install --global"

[tasks.eta]
run = "mise run //package/x:other"
"##;

/// Task names and aliases that the fixture reader finds,
/// and what it must leave out.
#[test]
fn the_reader_ignores_comments_descriptions_and_sub_tables() {
    let tasks = parse_tasks(FIXTURE_MISE, "");
    let names: Vec<&str> = tasks.iter().map(|task| return task.name.as_str()).collect();
    assert_eq!(names, ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta"]);
    let beta = tasks.iter().find(|task| return task.name == "beta").expect("beta");
    assert_eq!(beta.body, "run = \"echo hello\"\n", "comments and the sub-table stay out of the task");
    let alpha = tasks.iter().find(|task| return task.name == "alpha").expect("alpha");
    assert_eq!(alpha.aliases, ["a"]);
    assert!(alpha.body.contains("[not-a-header]"), "a line inside a multi-line string stays in the task");
}

/// A task is found fetching through a direct call,
/// through a shell line,
/// and through a task that runs a fetching task,
/// and the two offline `install` forms are not.
#[test]
fn the_reader_finds_fetching_tasks_and_skips_the_offline_install_forms() {
    let found = fetching_names(&parse_tasks(FIXTURE_MISE, ""));
    for name in ["alpha", "a", "zeta"] {
        assert!(found.contains(&name.to_string()), "{name}: {found:?}");
    }
    assert!(found.contains(&"gamma".to_string()), "a task that runs a fetching task fetches: {found:?}");
    for name in ["beta", "delta", "epsilon", "eta"] {
        assert!(!found.contains(&name.to_string()), "{name}: {found:?}");
    }
}

/// A package file qualifies its task names the way `mise run` spells them.
#[test]
fn package_task_names_carry_the_package_path() {
    let tasks = parse_tasks("[tasks.build]\nalias = \"b\"\nrun = \"runMpmShadow(['update'])\"\n", "//package/x/y:");
    assert_eq!(fetching_names(&tasks), ["//package/x/y:build", "//package/x/y:b"]);
}

/// The grammar literal reader finds names and aliases and skips the struct declaration.
#[test]
fn the_verb_reader_reads_literals_and_not_the_declaration() {
    let text = "pub struct Verb { pub name: &'static str, pub build: fn() }\n\
                const X: &[Verb] = &[Verb { name: \"install\", aliases: &[\"i\", \"add\"], build: f }, Verb { name: \"bin\", aliases: &[], build: g }];";
    assert_eq!(
        declared_verbs(text),
        [("install".to_string(), vec!["i".to_string(), "add".to_string()]), ("bin".to_string(), Vec::new())]
    );
}

/// Every fetching task in the root and package `mise.toml` files must be on the list.
#[test]
fn every_fetching_mise_task_is_listed() {
    let fetching = fetching_names(&all_tasks());
    for name in DETECTION_POSITIVE_CONTROL {
        assert!(fetching.contains(&(*name).to_string()), "the detection did not find {name}: {fetching:?}");
    }
    for name in &fetching {
        assert!(MISE_TASKS.contains(&name.as_str()), "mise task {name} fetches but is not on the wg list");
    }
}

/// Every listed name is a real task or alias.
#[test]
fn every_listed_mise_task_exists() {
    let tasks = all_tasks();
    for listed in MISE_TASKS {
        let exists = tasks.iter().any(|task| return task.name == *listed || task.aliases.iter().any(|alias| return alias == listed));
        assert!(exists, "{listed} is on the wg list but no mise task or alias has that name");
    }
}

/// The offline toolchain tasks exist,
/// stay off the list,
/// and are not found fetching.
#[test]
fn offline_mise_tasks_are_not_listed() {
    let tasks = all_tasks();
    let fetching = fetching_names(&tasks);
    for name in OFFLINE_TASKS {
        assert!(tasks.iter().any(|task| return task.name == *name), "{name} no longer exists: update OFFLINE_TASKS");
        assert!(!MISE_TASKS.contains(name), "{name}");
        assert!(!fetching.contains(&(*name).to_string()), "{name} is found fetching");
    }
}

/// The grammar is read,
/// every listed verb is a name or alias in it,
/// and every verb it declares is classified as fetching or offline.
#[test]
fn listed_mpm_shadow_verbs_match_the_current_grammar() {
    let grammar = grammar_verbs();
    let names: Vec<&str> = grammar.iter().map(|(name, _aliases)| return name.as_str()).collect();
    assert!(names.contains(&"install"), "the grammar reader found nothing useful: {names:?}");
    let spellings: Vec<&str> = grammar.iter().flat_map(|(name, aliases)| return std::iter::once(name).chain(aliases.iter()).map(String::as_str)).collect();
    for verb in MPM_SHADOW_VERBS {
        assert!(spellings.contains(verb), "mpm-shadow has no verb or alias {verb}: {spellings:?}");
    }
    for (name, aliases) in &grammar {
        let fetches = FETCHING_VERBS.contains(&name.as_str());
        assert!(fetches || OFFLINE_VERBS.contains(&name.as_str()), "verb {name} is new: classify it as fetching or offline");
        for spelling in std::iter::once(name).chain(aliases.iter()) {
            assert_eq!(MPM_SHADOW_VERBS.contains(&spelling.as_str()), fetches, "{spelling}");
        }
    }
}

/// The listed launcher exists:
/// the source file,
/// and for the built entry the package's `bin` map and its source entry.
#[test]
fn listed_toolchain_scripts_exist() {
    for tail in TOOLCHAIN_SCRIPT_TAILS {
        let relative = format!("package{tail}");
        match relative.split_once("/dist/final/node/") {
            None => assert!(repository_root().join(&relative).is_file(), "{relative}"),
            Some((package, built)) => {
                let source = built.replace(".mjs", ".ts");
                assert!(repository_root().join(package).join("src").join(&source).is_file(), "{package}/src/{source}");
                let manifest = read_repository_file(&format!("{package}/package.json"));
                assert!(manifest.contains(&format!("dist/final/node/{built}")), "{package}/package.json does not name {built}");
            }
        }
    }
}
