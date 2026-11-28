//! What:
//!  The ban on a release file reading a test-only switch itself:
//!  every `CLI_GIT_TEST_ONLY_*` value the release code uses is taken through
//!  `foundation/test_only_switches.rs`,
//!  whose read a build without the `test-only-switches` feature does not hold,
//!  and no other release file names such a variable beside an environment read.
//! Why:
//!  A unit's `Environment=` line can set a test-only variable,
//!  and one that arms a fault can kill or stall the commit service,
//!  so a release build must read none of them
//!  (`doc/decision/cli-git-test-only-switches-compiled-out-of-release.md`).
//!  The feature is what compiles the read out,
//!  and a new read written beside it would be compiled into the release again,
//!  which no module test sees because such a read only ever fires when its variable is set.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of releaseFiles) for (const line of lines(file)) expect(environmentReadBesideASwitch(line)).toBe(false);
//! ```

use std::path::Path;

use crate::source_scan_tests::{
    CrateRoot,
    SourceFile,
    classified_sources,
    real_roots,
    release_root,
    source_root,
    write_tree,
};

/// The prefix every test-only switch of this crate carries,
/// which a declaration of one names and no other release line may sit beside a read of the environment.
const TEST_ONLY_PREFIX: &str = "CLI_GIT_TEST_ONLY_";

/// The calls that read the environment,
/// which a test-only switch may reach only through the gated read.
const ENVIRONMENT_READS: &[&str] = &["env::var(", "env::var_os(", "env::vars(", "vars_os("];

/// The release files that name a test-only switch beside an environment read without being a finding,
/// with why.
/// The start probe's raised deadline is gated by its own feature rather than by `test-only-switches`,
/// and its own control asks that gate its question,
/// so this one read stays where its design put it
/// (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
/// choice 11).
/// An entry here that matches nothing is reported,
/// so a file that moves or stops reading shrinks this list.
const GATED_ELSEWHERE: &[(&str, &str)] = &[(
    "start_probe/probe_deadline.rs",
    "the probe's raised deadline is its own feature, `start-probe-raised-deadline`, and its own control asks it",
)];

/// What:
///  The names this text declares as a test-only switch.
/// Why:
///  A read passes the name rather than the literal,
///  so the scan has to know which identifiers name a switch before it can see the read;
///  a declaration is a `const` line whose value holds the prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declaredSwitchNames(text: string): string[] { return lines(text).filter(l => l.includes('const ') && l.includes('CLI_GIT_TEST_ONLY_')).map(nameAfterConst) }
/// ```
fn declared_switch_names(text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in text.lines() {
        // Everything after the `const` keyword on this line, which is the name and what follows it.
        let Some((_before, declared)) = line.split_once("const ") else {
            continue;
        };
        // The name and the rest of the declaration: a name is everything before the first colon.
        let Some((name, rest)) = declared.split_once(':') else {
            continue;
        };
        if !rest.contains(TEST_ONLY_PREFIX) {
            continue;
        }
        names.push(name.trim().to_owned());
    }
    return names;
}

/// What:
///  The lines of `text` that both read the environment and name a test-only switch,
///  as line numbers.
/// Why:
///  A read of a switch is the one thing the release must not hold,
///  and a release file that names a switch at all may do it for an error message,
///  a comment or a declaration,
///  which are not reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readingsOfSwitches(text: string, names: string[]): number[] { return lines(text).flatMap((l, i) => environmentRead(l) && namesASwitch(l, names) ? [i + 1] : []) }
/// ```
fn readings_of_switches(
    text: &str,
    names: &[String],
) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let reads: bool = ENVIRONMENT_READS
            .iter()
            .any(|needle| return line.contains(needle));
        if !reads {
            continue;
        }
        // Whether this line names a switch: the prefix itself, or an identifier some release file declares as one.
        let names_one: bool = line.contains(TEST_ONLY_PREFIX)
            || names.iter().any(|name| return line.contains(name.as_str()));
        if names_one {
            found.push(index + 1);
        }
    }
    return found;
}

/// The sources the scan reads:
/// every file the module tree reaches,
/// so a file this walk missed is a file the ban does not cover.
fn sources_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<SourceFile>, String> {
    return classified_sources(root, roots);
}

/// Every `file:line` of the crate's release sources that reads a test-only switch itself.
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let sources: Vec<SourceFile> = sources_in(root, roots)?;
    // The names every release file declares as a test-only switch, gathered first: a read may pass one declared in
    // another file, as the journal's socket switch is.
    let mut names: Vec<String> = Vec::new();
    for file in &sources {
        if file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        names.extend(declared_switch_names(text.as_str()));
    }
    let mut found: Vec<String> = Vec::new();
    for file in &sources {
        if file.test_only
            || GATED_ELSEWHERE
                .iter()
                .any(|(named, _)| return *named == file.path)
        {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for number in readings_of_switches(text.as_str(), names.as_slice()) {
            found.push(format!("{}:{number}", file.path));
        }
    }
    return Ok(found);
}

/// The findings in this crate's own sources.
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's sources");
}

/// No release source reads a test-only switch itself.
/// Every one of them is read through `foundation/test_only_switches.rs`,
/// which a release build does not hold.
#[test]
fn no_release_source_reads_a_test_only_switch() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "a test-only switch read here is compiled into the release, and a unit's `Environment=` line can set one, so \
         take the value through `crate::test_only_switches`:\n{}",
        found.join("\n")
    );
}

/// Every file the scan excuses is still a source of this crate,
/// so the list shrinks when a file moves rather than excusing nothing.
#[test]
fn every_excused_file_is_a_source_of_this_crate() {
    let sources: Vec<SourceFile> =
        sources_in(&source_root(), &real_roots()).expect("the crate's sources");
    let stale: Vec<String> = GATED_ELSEWHERE
        .iter()
        .filter(|(named, _)| return !sources.iter().any(|file| return file.path == *named))
        .map(|(named, why)| return format!("{named}: {why}"))
        .collect();
    assert!(
        stale.is_empty(),
        "these entries name no source of this crate, so remove them:\n{}",
        stale.join("\n")
    );
}

/// Every entry of the excused list is still a file that really reads a switch beside the environment,
/// so an entry that stopped doing so is removed rather than hiding a file that could read one again.
#[test]
fn every_excused_entry_still_reads_a_switch() {
    let sources: Vec<SourceFile> =
        sources_in(&source_root(), &real_roots()).expect("the crate's sources");
    let mut names: Vec<String> = Vec::new();
    let mut texts: Vec<(String, String)> = Vec::new();
    for file in &sources {
        if file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(source_root().join(file.path.as_str()))
            .expect("a source of this crate");
        names.extend(declared_switch_names(text.as_str()));
        texts.push((file.path.clone(), text));
    }
    let quiet: Vec<String> = GATED_ELSEWHERE
        .iter()
        .filter(|(named, _)| {
            return !texts.iter().any(|(path, text)| {
                return path == named
                    && !readings_of_switches(text.as_str(), names.as_slice()).is_empty();
            });
        })
        .map(|(named, why)| return format!("{named}: {why}"))
        .collect();
    assert!(
        quiet.is_empty(),
        "these entries excuse a file that reads no switch, so remove them:\n{}",
        quiet.join("\n")
    );
}

/// The scan fires on a read it exists for,
/// and on a literal one,
/// so a check that stopped reading would fail here rather than passing every tree.
#[test]
fn the_scan_finds_a_planted_read() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-test-only-switch");
    write_tree(
        root.as_path(),
        &[
            ("lib.rs", "mod switches;\n"),
            (
                "switches.rs",
                "pub const FAULT_VARIABLE: &str = \"CLI_GIT_TEST_ONLY_SERVICE_FAULT\";\n\
                 pub fn armed(point: &str) -> bool {\n\
                     let read = std::env::var(FAULT_VARIABLE);\n\
                     return read.is_ok_and(|armed| return armed == point);\n\
                 }\n\
                 pub fn literal() -> bool {\n\
                     std::env::var_os(\"CLI_GIT_TEST_ONLY_ANOTHER\").is_some()\n\
                 }\n\
                 pub fn a_message() -> String {\n\
                     format!(\"{FAULT_VARIABLE} names no step\")\n\
                 }\n",
            ),
        ],
    );
    let found: Vec<String> =
        findings_in(root.as_path(), &[release_root("lib.rs")]).expect("the fixture sources");
    assert_eq!(
        found,
        vec![String::from("switches.rs:3"), String::from("switches.rs:7")],
        "the planted reads were not found, or the message that names the variable was"
    );
    crate::test_support::remove(root.as_path());
}
