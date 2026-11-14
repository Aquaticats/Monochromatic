//! What:
//!  Guard for the rule that no control's service writes into the user journal of this machine:
//! no source the build compiles for tests only hands a child the test's own standard streams,
//! names the journal's trigger,
//! or copies a fixture's service words to the stream the process running the tests was given,
//! and the one support that builds a service start records the journal's trigger removed,
//! the journal's address a name nothing listens at,
//! and its streams `/dev/null` unless a control reads them.
//! Why:
//!  On 2026-11-14 this machine's user journal held services that were never this machine's:
//! `started release="0.6.1-alpha.30"` and eleven others written by debug builds in two agents' worktrees,
//! in units that were those agents' transient scopes.
//! A test's service reaches the journal two ways,
//! both decided by the service's own code (`journald_native.rs`):
//! its standard error is the journal's stream (so its plain lines land there),
//! or `$JOURNAL_STREAM` names that stream (so its entries go as datagrams to `/run/systemd/journal/socket`).
//! A disposable `HOME` and `XDG_RUNTIME_DIR` move neither:
//! the socket is a fixed path of the machine,
//! and the streams are those of whichever process started the test
//! (`doc/troubleshooting/git-policy-cli-test-services-write-into-the-user-journal.md`).
//! So a control may give its service only the streams a fixture owns,
//! and only a control that stands a journal of its own in may name the journal's trigger.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of testOnlySources()) expect(codeLines(file).filter((line) => /Stdio::inherit|JOURNAL_STREAM/.test(line))).toEqual([]);
//! ```

use crate::source_scan_tests::{classified_sources, real_roots, source_root};

/// Text that gives a child the streams of the process running the tests:
/// a service started with it writes into the journal whenever that process's standard error is the journal.
const INHERITED_STREAMS: &str = "Stdio::inherit";

/// The words that name the journal's trigger:
/// the variable the service manager sets when a unit's standard error is the journal,
/// and the constant this crate reads it by.
const JOURNAL_TRIGGER_WORDS: &[&str] = &["\"JOURNAL_STREAM\"", "STREAM_VARIABLE"];

/// What:
///  The lines of test sources that may name the journal's trigger,
///  and why each may.
/// Why:
///   What is banned is handing the trigger to a child;
///  a line that records it *removed* hands nothing,
///  and a control that reads the start's own record must name it to say what it expects.
///  Each entry's text is checked to be still there,
///  so the list cannot rot into an excuse for a line that went.
///  The one control that stands a journal of its own in is here for the other reason:
/// it hands the trigger to a service deliberately,
///  with an address of its own,
///  and it is the control that proves the native path still works.
const ALLOWED_TRIGGER_LINES: &[(&str, &str)] = &[
    // The support removes the trigger from every control's service, which is the fix this guard keeps.
    ("binary/binary_support_fixture_service.rs", "env_remove(git_policy_cli::journald_native::STREAM_VARIABLE)"),
    // The control that reads the start's env records and expects the removal.
    ("binary/binary_fixture_tests.rs", "journald_native::STREAM_VARIABLE),"),
    // The one control that stands a journal of its own in.
    ("binary/binary_commit_timing_tests.rs", "STREAM_VARIABLE"),
];

/// The support that builds every service a control starts,
/// and the fixture log its streams go to.
const SERVICE_SUPPORT: &str = "binary/binary_support_fixture_service.rs";
const SERVICE_LOG_SUPPORT: &str = "binary/binary_support_fixture_service_log.rs";

/// What:
///  This scan's own file,
///  which the rules skip.
/// Why:
///   Its tables spell both forbidden forms,
/// and its detector controls feed those forms to the detectors,
/// so reading it would report the reader.
///  Nothing here starts a process,
/// and the control below keeps it that way.
const THIS_SCAN: &str = "source_guard/service_journal_isolation_tests.rs";

/// The text of one source of this crate,
/// by its path relative to `src/native`.
fn source_text(relative: &str) -> String {
    return std::fs::read_to_string(source_root().join(relative)).expect("a readable source");
}

/// What:
///  Every source the build compiles for tests only,
///  as `/`-separated paths relative to `src/native`.
/// Why:
///   A rule about what a control may do belongs to the files that ship nothing,
///  and release code is guarded by what it must do instead
///  (`blocking_code_off_the_runtime_tests.rs`).
fn test_only_sources() -> Vec<String> {
    let found = classified_sources(source_root().as_path(), real_roots().as_slice()).expect("the module tree of this crate");
    return found.into_iter().filter(|file| return file.test_only).map(|file| return file.path).collect();
}

/// What:
///  The lines of `text` that hold `word` in code:
/// not a line whose first word starts a comment,
/// and not a match after a `//` on the same line.
/// Why:
///   These rules are about what a control does,
///  and the files they read explain themselves in prose that may name the same words.
/// The reading is coarse and says so:
/// it does not parse raw strings,
/// so a `//` inside one hides the rest of that line.
fn lines_holding(text: &str, word: &str) -> Vec<(usize, String)> {
    let mut found: Vec<(usize, String)> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let code: &str = line.trim_start();
        if code.starts_with("//") {
            continue;
        }
        let Some(at) = line.find(word) else { continue };
        if line[..at].contains("//") {
            continue;
        }
        found.push((index + 1, line.trim().to_owned()));
    }
    return found;
}

/// The findings of one rule over the test sources:
/// `<path>:<line>: <what the line does>`.
fn findings(word: &str, excused: &[(&str, &str)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for relative in test_only_sources() {
        if relative == THIS_SCAN {
            continue;
        }
        let text: String = source_text(relative.as_str());
        for (number, line) in lines_holding(text.as_str(), word) {
            let allowed: bool = excused.iter().any(|(file, allowed_text)| { return *file == relative && line.contains(allowed_text); });
            if !allowed {
                found.push(format!("{relative}:{number}: {line}"));
            }
        }
    }
    return found;
}

/// What:
///  No source compiled for tests only gives a child the streams of the process running the tests.
/// Why:
///   A service started with them writes wherever that process's standard error points,
///  and a run of the suite under a unit points it at the journal,
///  which is how a throwaway fixture's service said its release into this machine's journal on 2026-11-14.
///  A control that reads what its service said gives it `kept_stderr_pipe`,
///  which the fixture holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findings('Stdio::inherit', [])).toEqual([]);
/// ```
#[test]
fn no_test_source_gives_a_child_the_streams_of_the_run() {
    let found: Vec<String> = findings(INHERITED_STREAMS, &[]);
    assert_eq!(found, Vec::<String>::new(), "a control's child would write into whatever stream the run was given, the journal included");
}

/// What:
///  No source compiled for tests only names the journal's trigger,
///  but the lines listed above with their reasons.
/// Why:
///   The trigger is how the service knows its standard error is the journal,
///  and with it the entries leave as datagrams for `/run/systemd/journal/socket`,
///  a path of the machine that no fixture directory moves.
///  A line that takes it away hands nothing,
///  and a control that stands a journal of its own in may hand it over with an address of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findings('JOURNAL_STREAM', ALLOWED_TRIGGER_LINES)).toEqual([]);
/// ```
#[test]
fn no_test_source_but_the_listed_lines_names_the_journal_s_trigger() {
    let mut found: Vec<String> = findings(JOURNAL_TRIGGER_WORDS[0], ALLOWED_TRIGGER_LINES);
    found.extend(findings(JOURNAL_TRIGGER_WORDS[1], ALLOWED_TRIGGER_LINES));
    assert_eq!(found, Vec::<String>::new(), "the service would take its entries to the machine's journal socket");
    for (file, allowed_text) in ALLOWED_TRIGGER_LINES {
        assert!(source_text(file).contains(allowed_text), "the exception {allowed_text} in {file} is still needed; remove it from the list when the line goes");
    }
}

/// What:
///  The one support that builds every service start keeps the journal off that service by construction:
/// the trigger removed,
/// the address a name nothing listens at,
/// and its streams `/dev/null` unless a control names its own.
/// Why:
///   Removing the trigger is what the removal of the runtime directory and the session bus already does for the manager
///  (`service_command`),
///  and the streams are the second half:
/// a control that forgets to give its service a pipe must get silence,
/// never the journal of the machine.
///  The address is a *set* value rather than a removal,
///  so a control that inherits one cannot reach the machine's socket either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(supportText).toContain('env_remove(git_policy_cli::journald_native::STREAM_VARIABLE)');
/// expect(supportText).toContain('TEST_SOCKET_VARIABLE, NOBODY_JOURNAL_ADDRESS');
/// expect(supportText).toContain('.stderr(Stdio::null())');
/// ```
#[test]
fn the_service_start_support_keeps_the_journal_off_every_control_s_service() {
    let support: String = source_text(SERVICE_SUPPORT);
    for line in [
        ".env_remove(git_policy_cli::journald_native::STREAM_VARIABLE)",
        ".env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, NOBODY_JOURNAL_ADDRESS)",
        ".stdin(Stdio::null())",
        ".stdout(Stdio::null())",
        ".stderr(Stdio::null())",
    ] {
        assert!(support.contains(line), "the support that starts every control's service no longer holds `{line}`");
    }
}

/// What:
///  The fixture's log of what its services said never writes to the stream the process running the tests was given.
/// Why:
///  That stream is the journal whenever the run was given the journal,
///  and then a fixture's service words would be there although the service itself was kept off it.
///  The file the log support keeps is what a failure quotes
///  (`service_log`),
///  so nothing is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(linesHolding(logSupportText, 'std::io::stderr()')).toEqual([]);
/// ```
#[test]
fn the_fixture_service_log_writes_nowhere_but_the_fixture() {
    let log_support: String = source_text(SERVICE_LOG_SUPPORT);
    assert_eq!(lines_holding(log_support.as_str(), "std::io::stderr()"), Vec::<(usize, String)>::new(), "a fixture's service words would reach the stream the run was given");
}

/// What:
///  The scan reaches the controls it guards:
///  the files that start the service build most of this crate's controls,
///  so a rule about them that found nothing would read as a pass.
/// Why:
///   `QIV` and `TRV`:
/// a scan whose scope went empty,
/// or whose walk stopped reaching the files,
/// passes for the wrong reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(testOnlySources().filter((file) => read(file).includes('service_command'))).toHaveLength(atLeast(10));
/// ```
#[test]
fn the_scan_reaches_the_controls_that_start_a_service() {
    let starting: Vec<String> = test_only_sources()
        .into_iter()
        .filter(|relative| return source_text(relative.as_str()).contains("service_command()"))
        .collect();
    assert!(starting.len() >= 10, "the scan found {} files that start a control's service: {starting:?}", starting.len());
}

/// What:
///  The two detectors find each forbidden form in a text,
///  skip a comment that names it,
///  and honour an exception by file.
/// Why:
///   A guard that cannot fail is no guard:
/// the assertions above pass when the detector reads nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(linesHolding('let a = Stdio::inherit();', 'Stdio::inherit')).toHaveLength(1);
/// expect(linesHolding('// Stdio::inherit is banned', 'Stdio::inherit')).toHaveLength(0);
/// ```
#[test]
fn the_detectors_find_the_forbidden_forms_and_skip_prose() {
    assert_eq!(lines_holding("let mut c = builder(); c.stderr(Stdio::inherit());", INHERITED_STREAMS).len(), 1);
    assert_eq!(lines_holding("// a child given Stdio::inherit writes where the run writes", INHERITED_STREAMS).len(), 0);
    assert_eq!(lines_holding("    c.stderr(Stdio::inherit()); // never again", INHERITED_STREAMS).len(), 1);
    assert_eq!(lines_holding("    c.stderr(Stdio::inherit()); // odd", INHERITED_STREAMS).len(), 1);
    assert_eq!(lines_holding("let x = c.stderr(Stdio::null()); // Stdio::inherit is banned", INHERITED_STREAMS).len(), 0);
    assert_eq!(lines_holding("command.env(git_policy_cli::journald_native::STREAM_VARIABLE, identity);", JOURNAL_TRIGGER_WORDS[1]).len(), 1);
    assert_eq!(lines_holding("let _ = std::env::var(\"JOURNAL_STREAM\");", JOURNAL_TRIGGER_WORDS[0]).len(), 1);
    assert_eq!(lines_holding("/// The variable is $JOURNAL_STREAM, which systemd sets.", JOURNAL_TRIGGER_WORDS[0]).len(), 0);
}

/// What:
///  A source the crate compiles for tests only is found by the classification the scan reads,
///  so no rule here can pass because the listing went empty.
/// Why:
///   The listing is the module tree from the manifest's own targets;
/// a branch that moved a control into a folder this scan does not look at would otherwise be unfindable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(testOnlySources()).toContain('binary/binary_support_fixture_service.rs');
/// ```
#[test]
fn the_test_only_listing_holds_the_support_this_guard_reads() {
    let sources: Vec<String> = test_only_sources();
    for named in [SERVICE_SUPPORT, SERVICE_LOG_SUPPORT] {
        assert!(sources.iter().any(|path| return path == named), "{named} is not among the sources compiled for tests only: {} of them", sources.len());
    }
}

/// What:
///  The file the rules skip starts nothing,
///  so skipping it can hide no start of a control's service.
/// Why:
///   A rule that passes because its reader was skipped is a rule that does not hold (`QIV`);
/// this keeps the one skip honest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(read(THIS_SCAN)).not.toMatch(/process::Command|\.spawn\(/);
/// ```
#[test]
fn the_scan_its_own_file_starts_no_process() {
    let text: String = source_text(THIS_SCAN);
    // The two words are spelled in pieces so this control's own lines are not findings of themselves.
    let starting: String = ["Command", "::new"].concat();
    let spawning: String = [".spa", "wn("].concat();
    assert_eq!(lines_holding(text.as_str(), starting.as_str()), Vec::<(usize, String)>::new());
    assert_eq!(lines_holding(text.as_str(), spawning.as_str()), Vec::<(usize, String)>::new());
}
