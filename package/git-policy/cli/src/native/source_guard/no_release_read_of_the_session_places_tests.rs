//! What:
//!  The ban on a release file reading this session's runtime directory or its session-bus address
//!  outside the one place that reads them:
//!  `foundation/session_runtime_directory_and_bus_address.rs`.
//! Why:
//!  Both names reach places outside any fixture,
//!  and a test process shares this machine's account with the commits of every other session.
//!  A test build answers them only from what a control handed in,
//!  so a read written beside that one place would put a test process back on the account's own manager:
//!  the hand-over wrote fixture units into `/run/user/<n>/systemd/user/`,
//!  pointed the account-wide program link at the test binary,
//!  and had the manager run libtest for the fixture until the unit's start limit hit (issue 6586).
//!  No module test sees such a read,
//!  because it only ever fires where the account's own places exist.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of releaseFiles) for (const line of namingLines(file)) expect(exempt(file) || line === undefined).toBe(true);
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

/// The two names whose reads belong to one place:
/// the runtime directory a session keeps its sockets and units under,
/// and the address of the session bus a service is provisioned through.
const THE_PLACES: &[&str] = &["XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS"];

/// The one release file that reads them,
/// which a release build reads from the process's own environment
/// and a build made for tests reads from what a control handed in.
const THE_ONE_PLACE: &str = "foundation/session_runtime_directory_and_bus_address.rs";

/// The files that name one of the places without reading the process's environment for it,
/// with why.
/// An entry here that matches nothing is reported,
/// so a file that moves or stops naming a place shrinks this list.
const NAMES_A_PLACE_WITHOUT_READING_IT: &[(&str, &str)] = &[
    (
        "invocation/management.rs",
        "reads the environment the caller handed in, which is the invocation's own list and not the process's",
    ),
    (
        "invocation/management_service_stop_units_of_removed_worktrees.rs",
        "reads the environment the caller handed in, and prints the setting's name in the refusal a person reads",
    ),
    (
        "invocation/real_git.rs",
        "reads the environment the caller handed in to find the unit file a session wrote, and names it in a comment",
    ),
    (
        "account_companion/companion_scratch.rs",
        "prints the setting's name in the refusals a person reads, and reads the directory through the one place",
    ),
    (
        "landing/service_messages.rs",
        "prints the setting's name in the words a person reads when a service cannot come",
    ),
    (
        "landing/service_messages_of_a_start_that_cannot_come.rs",
        "prints the setting's name in the words a person reads when a start cannot come",
    ),
    (
        "service/service_provision.rs",
        "prints the setting's name in the words a person reads when the unit folder cannot be written",
    ),
    (
        "service/service_provision_the_links_the_units_run.rs",
        "prints the setting's name in the words a person reads when the program link cannot be written",
    ),
];

/// What:
///  The names this text declares as a constant or a static that holds one of the places,
///  so a read that passes a name is seen as a read of that place.
/// Why:
///  A read written as `std::env::var_os(RUNTIME_VARIABLE)` names no place on its own line,
///  and a scan of the literal alone would pass it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declaredPlaceNames(text: string): string[] { return lines(text).filter(declaresAndNames).map(nameAfterConst); }
/// ```
fn declared_place_names(text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in text.lines() {
        // What:     everything after the `const` keyword on this line, which is the name and what follows it.
        // Why:      a declaration is where a name is spelled once and read many times.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const declared = line.split('const ')[1];
        // ```
        let Some((_before, declared)) = line.split_once("const ") else {
            continue;
        };
        if !THE_PLACES
            .iter()
            .any(|place| return declared.contains(place))
        {
            continue;
        }
        // What:     the name itself, which is everything before the first colon.
        // Why:      `const NAME: &str = "XDG_RUNTIME_DIR";` declares `NAME`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // names.push(declared.split(':')[0].trim());
        // ```
        let Some((name, _rest)) = declared.split_once(':') else {
            continue;
        };
        names.push(name.trim().to_owned());
    }
    return names;
}

/// What:
///  The lines of `text` that name one of the places in code,
///  as line numbers counted from one.
/// Why:
///  A message a person reads names the setting and reads nothing,
///  so only a line that is not a comment is a finding,
///  and the reader is sent to the line that decided.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namingLines(text: string, declared: string[]): number[] { return lines(text).flatMap((l, i) => !comment(l) && (THE_PLACES.some(p => l.includes(p)) || declared.some(n => l.includes(n))) ? [i + 1] : []); }
/// ```
fn naming_lines(
    text: &str,
    declared: &[String],
) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed: &str = line.trim_start();
        // What:     a comment line says what a place is and reads nothing, so it is no finding.
        // Why:      the crate explains these settings in its docs and its words to a person far more often than it
        //           reads them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (trimmed.startsWith('//') || trimmed.startsWith('/*') || trimmed.startsWith('*')) continue;
        // ```
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        let names_a_place: bool = THE_PLACES.iter().any(|place| return line.contains(place));
        let names_a_declared: bool = declared
            .iter()
            .any(|name| return line.contains(name.as_str()));
        if names_a_place || names_a_declared {
            found.push(index + 1);
        }
    }
    return found;
}

/// What:
///  The sources the scan reads:
///  every file the module tree reaches,
///  so a file this walk missed is a file the ban does not cover.
/// Why:
///  A read in a file the scan never opened would pass every control here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sourcesIn(root: string, roots: CrateRoot[]): SourceFile[] { return classifiedSources(root, roots); }
/// ```
fn sources_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<SourceFile>, String> {
    return classified_sources(root, roots);
}

/// What:
///  Every `file:line` of the crate's release sources that names one of the places outside the one place,
///  and outside the files whose entries say why they may.
/// Why:
///  The list is the finding list the ban's control asserts empty,
///  and the planted fixture below shows it fills.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsIn(root: string, roots: CrateRoot[]): string[] { ... }
/// ```
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let sources: Vec<SourceFile> = sources_in(root, roots)?;
    // What:     the names every release file declares as one of the places, gathered first.
    // Why:      a read may pass a name declared in another file, as the account companion's messages do.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const declared = releaseFiles.flatMap((f) => declaredPlaceNames(read(f)));
    // ```
    let mut declared: Vec<String> = Vec::new();
    for file in &sources {
        if file.test_only || file.path == THE_ONE_PLACE {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        declared.extend(declared_place_names(text.as_str()));
    }
    let mut found: Vec<String> = Vec::new();
    for file in &sources {
        if file.test_only
            || file.path == THE_ONE_PLACE
            || NAMES_A_PLACE_WITHOUT_READING_IT
                .iter()
                .any(|(named, _why)| return *named == file.path)
        {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for number in naming_lines(text.as_str(), declared.as_slice()) {
            found.push(format!("{}:{number}", file.path));
        }
    }
    return Ok(found);
}

/// The findings in this crate's own sources.
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's sources");
}

/// What:
///  No release source names a session place outside the one file that reads them,
///  except the files whose entries say they only print it or read an environment a caller handed in.
/// Why:
///  A test build answers these two names only from what a control handed in,
///  so a second read is a second way for a test process to reach this account's own service manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findings()).toEqual([]);
/// ```
#[test]
fn no_release_source_places_a_session_outside_the_one_place() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "this session's runtime directory and session-bus address are read in \
         `foundation/session_runtime_directory_and_bus_address.rs` alone, which a build made for tests answers \
         from what a control handed in and never from the environment the test process inherited (issue 6586); \
         take the name through `crate::session_runtime_directory_and_bus_address`:\n{}",
        found.join("\n")
    );
}

/// What:
///  Every file the scan excuses is still a source of this crate.
/// Why:
///  An entry that named a path nothing holds would hide a real finding if that path came back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(exempt.filter((named) => !sources.some((file) => file.path === named))).toEqual([]);
/// ```
#[test]
fn every_excused_file_is_a_source_of_this_crate() {
    let sources: Vec<SourceFile> =
        sources_in(&source_root(), &real_roots()).expect("the crate's sources");
    let stale: Vec<String> = NAMES_A_PLACE_WITHOUT_READING_IT
        .iter()
        .filter(|(named, _why)| return !sources.iter().any(|file| return file.path == *named))
        .map(|(named, why)| return format!("{named}: {why}"))
        .collect();
    assert!(
        stale.is_empty(),
        "these entries name no source of this crate, so remove them:\n{}",
        stale.join("\n")
    );
}

/// What:
///  Every excused file still names one of the places,
///  so an entry that stopped doing so is removed rather than hiding a file that could read one again.
/// Why:
///  The list may only shrink with the text it excuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(exempt.filter((named) => !namingLines(read(named)).length)).toEqual([]);
/// ```
#[test]
fn every_excused_file_still_names_a_place() {
    let sources: Vec<SourceFile> =
        sources_in(&source_root(), &real_roots()).expect("the crate's sources");
    let mut declared: Vec<String> = Vec::new();
    let mut texts: Vec<(String, String)> = Vec::new();
    for file in &sources {
        if file.test_only || file.path == THE_ONE_PLACE {
            continue;
        }
        let text: String = std::fs::read_to_string(source_root().join(file.path.as_str()))
            .expect("a source of this crate");
        declared.extend(declared_place_names(text.as_str()));
        texts.push((file.path.clone(), text));
    }
    let quiet: Vec<String> = NAMES_A_PLACE_WITHOUT_READING_IT
        .iter()
        .filter(|(named, _why)| {
            return !texts.iter().any(|(path, text)| {
                return path == named
                    && !naming_lines(text.as_str(), declared.as_slice()).is_empty();
            });
        })
        .map(|(named, why)| return format!("{named}: {why}"))
        .collect();
    assert!(
        quiet.is_empty(),
        "these entries excuse a file that names no place, so remove them:\n{}",
        quiet.join("\n")
    );
}

/// What:
///  The scan fires on a read of each place,
///  on a read that passes a declared name,
///  and not on a comment or on a message a person reads.
/// Why:
///  A check that stopped seeing its own needle would pass every tree,
///  and the shapes a passing read takes are the ones a later round writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findingsIn(fixtureRoot, [releaseRoot('lib.rs')])).toEqual(['reader.rs:4', 'reader.rs:6']);
/// ```
#[test]
fn the_scan_finds_a_planted_read() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-session-places");
    write_tree(
        root.as_path(),
        &[
            ("lib.rs", "mod reader;\nmod words;\n"),
            (
                "reader.rs",
                "pub fn route() -> Option<std::ffi::OsString> {\n\
                 \x20   // The runtime directory is `XDG_RUNTIME_DIR`, and this comment reads nothing.\n\
                 \x20   let runtime = std::env::var_os(RUNTIME_VARIABLE);\n\
                 \x20   let bus = std::env::var(\"DBUS_SESSION_BUS_ADDRESS\").ok();\n\
                 \x20   return runtime.or(bus.map(std::ffi::OsString::from));\n\
                 }\n",
            ),
            (
                "words.rs",
                "pub const RUNTIME_VARIABLE: &str = \"XDG_RUNTIME_DIR\";\n\
                 pub fn refusal() -> String {\n\
                 \x20   format!(\"{RUNTIME_VARIABLE} is not set, so there is no place to keep the socket in\")\n\
                 }\n",
            ),
        ],
    );
    let found: Vec<String> =
        findings_in(root.as_path(), &[release_root("lib.rs")]).expect("the fixture sources");
    assert_eq!(
        found,
        vec![
            String::from("reader.rs:3"),
            String::from("reader.rs:4"),
            String::from("words.rs:1"),
            String::from("words.rs:3"),
        ],
        "the planted read of each place, the declaration that holds one, and the message that names that \
         declaration were not the findings"
    );
    crate::test_support::remove(root.as_path());
}
