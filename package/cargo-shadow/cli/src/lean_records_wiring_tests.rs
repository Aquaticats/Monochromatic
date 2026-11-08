//! The wiring of mechanism D to the tasks that write run records:
//! a task whose `run` starts a program that writes a record under `target/verification`
//! runs `cargo-shadow lean records target/verification` before that program,
//! or runs a task that does.
//! Why:
//! The retention lives in cargo-shadow (`doc/handover/cargo-shadow-lean-build-folders.md`),
//! and nothing makes a producing task call it;
//! a task that stops calling it leaves records that nothing removes.
//! The ledger below is every such program,
//! read from the programs themselves;
//! a program that starts writing records fails the last test until it is listed here.

/// Files,
/// maps and sets,
/// paths and process arguments.
use std::{collections::{BTreeMap, BTreeSet}, fs, path::{Path, PathBuf}};

/// Reads the task files.
use toml_edit::{DocumentMut, Item};

/// Imports the name this file uses from `crate::lean_run_records`.
use crate::lean_run_records::is_record_name;

/// One program that writes a record of its own run under `target/verification`.
struct Producer {
    /// The package it lives in,
    /// relative to the repository root.
    package: &'static str,
    /// Its path inside that package.
    script: &'static str,
    /// The name prefixes it writes.
    prefixes: &'static [&'static str],
    /// Options that make this program write nothing this run.
    unless: &'static [&'static str],
}

/// The sweep a producing task runs first.
const SWEEP: &str = "cargo-shadow lean records target/verification";

/// Every program of this repository that writes a record of its own run into a verification directory.
/// Why:
/// This is the ledger of what the retention has to reach;
/// each script path and prefix was read from the program named beside it.
const PRODUCERS: [Producer; 16] = [
    Producer { package: "package/git-policy/cli", script: "bin/test-native-container.ts", prefixes: &["native-"], unless: &[] },
    Producer { package: "package/git-policy/cli", script: "bin/build-git-test-image.ts", prefixes: &["git-image-"], unless: &[] },
    Producer { package: "package/git-policy/cli", script: "bin/mutate-native-container.ts", prefixes: &["native-mutation-"], unless: &[] },
    Producer { package: "package/git-policy/cli", script: "bin/native-planted-controls-only.ts", prefixes: &["native-mutation-"], unless: &[] },
    Producer { package: "package/git-policy/cli.fuzz", script: "bin/container.ts", prefixes: &["campaign-"], unless: &[] },
    Producer { package: "package/git-policy/cli.fuzz", script: "bin/planted-controls.ts", prefixes: &["planted-"], unless: &[] },
    Producer { package: "package/linter/monochromatic-lint", script: "bin/mutate-container.ts", prefixes: &["mutation-", "coverage-"], unless: &["--list"] },
    Producer { package: "package/linter/monochromatic-lint", script: "bin/mutate-processors.ts", prefixes: &["processors-mutation-"], unless: &[] },
    Producer { package: "package/linter/monochromatic-lint.fuzz", script: "bin/container.ts", prefixes: &["campaign-"], unless: &[] },
    Producer { package: "package/linter/monochromatic-lint.fuzz", script: "bin/planted-controls.ts", prefixes: &["planted-"], unless: &[] },
    Producer { package: "package/forbidden-strings/cli", script: "bin/test-container.ts", prefixes: &["test-", "clippy-", "release-test-"], unless: &[] },
    Producer { package: "package/forbidden-strings/cli", script: "bin/guard-container.ts", prefixes: &["guard-protected-", "guard-without-"], unless: &[] },
    Producer { package: "package/forbidden-strings/cli", script: "bin/observe-guards-container.ts", prefixes: &["guard-observer-"], unless: &[] },
    Producer { package: "package/forbidden-strings/cli", script: "bin/mutate-container.ts", prefixes: &["mutation-", "startup-mutation-", "pathname-mutation-"], unless: &[] },
    Producer { package: "package/forbidden-strings/cli.fuzz", script: "bin/smoke-container.ts", prefixes: &["embedding-fuzz-", "fuzz-clippy-"], unless: &[] },
    Producer { package: "package/forbidden-strings/core", script: "bin/mutate-container.ts", prefixes: &["mutation-"], unless: &[] },
];

/// One task of a package's `mise.toml`.
#[derive(Default)]
struct Task {
    /// Every element of its `run`.
    run: Vec<String>,
    /// Every task it depends on.
    depends: Vec<String>,
}

/// What:
/// `fn repository_root() -> PathBuf` is the root of the checkout this test runs in.
/// Why:
/// The producing tasks live in other packages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repositoryRoot = (): string => resolve(import.meta.dirname, '../../..');
/// ```
fn repository_root() -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// What:
/// `fn text_elements(item: &Item) -> Vec<String>` are the strings of a `run` or `depends` value,
/// whether it is one string or an array.
/// Why:
/// A task file writes either shape,
/// and a table element (`{ tasks = [...] }`) has no program to look for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const textElements = (item: Item): string[] => typeof item === 'string' ? [item] : (item ?? []).flatMap((one) => typeof one === 'string' ? [one] : []);
/// ```
fn text_elements(item: &Item) -> Vec<String> {
    let Some(text) = item.as_str() else {
        let Some(array) = item.as_array() else {
            return Vec::new();
        };
        let mut found: Vec<String> = Vec::new();
        for element in array {
            if let Some(one) = element.as_str() {
                found.push(one.to_string());
            }
        }
        return found;
    };
    return vec![text.to_string()];
}

/// What:
/// `fn tasks_of(package: &Path) -> BTreeMap<String, Task>` reads the tasks of one package's `mise.toml`.
/// Why:
/// The wiring is a property of the task file,
/// so the test reads the file a person edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tasksOf = (package_: string): Map<string, Task> => new Map(Object.entries(parse(readFileSync(join(package_, 'mise.toml'), 'utf8')).tasks ?? {}));
/// ```
fn tasks_of(package: &Path) -> BTreeMap<String, Task> {
    let text: String = fs::read_to_string(package.join("mise.toml")).expect("read a task file");
    let document: DocumentMut = text.parse().expect("parse a task file");
    let mut found: BTreeMap<String, Task> = BTreeMap::new();
    let Some(tasks) = document.get("tasks").and_then(Item::as_table_like) else {
        return found;
    };
    for (name, item) in tasks.iter() {
        let mut task: Task = Task::default();
        if let Some(run) = item.get("run") {
            task.run = text_elements(run);
        }
        if let Some(depends) = item.get("depends") {
            task.depends = text_elements(depends);
        }
        found.insert(name.to_string(), task);
    }
    return found;
}

/// What:
/// `fn producers_of(package: &str) -> Vec<&'static Producer>` are the ledger entries of one package.
/// Why:
/// The same script name means a different program in another package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const producersOf = (package_: string): Producer[] => PRODUCERS.filter((one) => one.package === package_);
/// ```
fn producers_of(package: &str) -> Vec<&'static Producer> {
    let mut found: Vec<&'static Producer> = Vec::new();
    for producer in &PRODUCERS {
        if producer.package == package {
            found.push(producer);
        }
    }
    return found;
}

/// What:
/// `fn producing_elements(task: &Task, producers: &[&Producer]) -> Vec<(usize, String)>` are the positions in a task's
/// `run` that start a program writing run records,
/// with the prefixes that program writes.
/// Why:
/// The sweep has to come before the first of them,
/// and a task can start more than one producer.
/// `--list` only prints a listing and writes nothing,
/// so a run with it is not a producing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const producingElements = (task: Task, producers: Producer[]): [number, string][] => task.run.flatMap((entry, index) => producers.filter((one) => entry.split(/\s+/u).includes(one.script)).map((one) => [index, one.prefixes.join(', ')]));
/// ```
fn producing_elements(task: &Task, producers: &[&Producer]) -> Vec<(usize, String)> {
    let mut found: Vec<(usize, String)> = Vec::new();
    for (index, entry) in task.run.iter().enumerate() {
        for producer in producers {
            let tokens: Vec<&str> = entry.split_whitespace().collect();
            if tokens.contains(&producer.script) && !tokens.iter().any(|token| { return producer.unless.contains(token); }) {
                found.push((index, producer.prefixes.join(", ")));
            }
        }
    }
    return found;
}

/// What:
/// `fn sweeps_before(task: &Task, first_producer: usize) -> bool` says whether the task's own `run` sweeps first.
/// Why:
/// A sweep after the producer has already written this run's record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sweepsBefore = (task: Task, firstProducer: number): boolean => task.run.findIndex((one) => one === SWEEP) !== -1 && task.run.indexOf(SWEEP) < firstProducer;
/// ```
fn sweeps_before(task: &Task, first_producer: usize) -> bool {
    let Some(at) = task.run.iter().position(|entry| { return entry == SWEEP; }) else {
        return false;
    };
    return at < first_producer;
}

/// What:
/// `fn sweeps_through_dependencies(tasks: &BTreeMap<String, Task>, name: &str) -> bool` says whether a task or anything it
/// depends on,
/// directly or through another dependency,
/// runs the sweep.
/// Why:
/// `mise` runs a task's dependencies before it,
/// so a producer started after them is swept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sweepsThroughDependencies(tasks: Map<string, Task>, name: string, seen = new Set<string>()): boolean
/// ```
fn sweeps_through_dependencies(tasks: &BTreeMap<String, Task>, name: &str) -> bool {
    let mut pending: Vec<String> = vec![name.to_string()];
    let mut seen: BTreeSet<String> = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        let Some(task) = tasks.get(&current) else {
            continue;
        };
        for entry in &task.run {
            if entry == SWEEP {
                return true;
            }
        }
        for dependency in &task.depends {
            pending.push(dependency.clone());
        }
    }
    return false;
}

/// Every name a producing program writes is a run record,
/// so the sweep the tasks run reaches it.
#[test]
fn every_prefix_a_producing_program_writes_is_a_run_record() {
    for producer in &PRODUCERS {
        for prefix in producer.prefixes {
            assert!(
                is_record_name(&format!("{prefix}abc123")),
                "{} writes {prefix}, which the retention does not sweep",
                producer.script
            );
        }
    }
}

/// Every ledger entry names a program that is there,
/// so a renamed script fails here instead of quietly leaving its records unswept.
#[test]
fn every_producing_program_the_ledger_names_is_there() {
    let root: PathBuf = repository_root();
    for producer in &PRODUCERS {
        let script: PathBuf = root.join(producer.package).join(producer.script);
        assert!(script.is_file(), "{} names {} of {}, which is not there", "the run-record ledger", producer.script, producer.package);
    }
}

/// Every task that starts a program writing run records sweeps them first,
/// itself or through a task it depends on.
#[test]
fn every_task_that_writes_run_records_sweeps_them_first() {
    let root: PathBuf = repository_root();
    let mut packages: BTreeSet<&'static str> = BTreeSet::new();
    for producer in &PRODUCERS {
        if producer.prefixes.is_empty() {
            continue;
        }
        packages.insert(producer.package);
    }
    let mut unwired: Vec<String> = Vec::new();
    for package in packages {
        let tasks: BTreeMap<String, Task> = tasks_of(&root.join(package));
        let producers: Vec<&'static Producer> = producers_of(package);
        for (name, task) in &tasks {
            let producing: Vec<(usize, String)> = producing_elements(task, &producers);
            if producing.is_empty() {
                continue;
            }
            let first: usize = producing.iter().map(|(index, _)| { return *index; }).min().expect("one producing element");
            if sweeps_before(task, first) || sweeps_through_dependencies(&tasks, name) {
                continue;
            }
            unwired.push(format!("{package}: {name} writes {}", producing.iter().map(|(_, prefixes)| { return prefixes.clone(); }).collect::<Vec<String>>().join(", ")));
        }
    }
    assert!(unwired.is_empty(), "these tasks write run records and never sweep them first:\n{}", unwired.join("\n"));
}
