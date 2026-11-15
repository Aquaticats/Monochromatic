//! What:
//!  Guard for the rule that no control's service writes into the user journal of this machine:
//! no source the build compiles for tests only gives a child `Stdio::inherit`,
//!  or leaves a child's standard error to the run where that child can write a service's words
//! (the wrapper's service itself,
//! a shell that can run it,
//! a start whose program the guard cannot name,
//! or a program the inventory does not name),
//! or names the journal's trigger,
//! or copies a fixture's service words to the stream the process running the tests was given,
//!  and the one support that builds a service start records the journal's trigger removed,
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
//!  A start's standard error is what the child is given,
//! and `std::process::Command`'s default is to inherit it:
//! a review of 2026-11-14 found this guard looking only for the token `Stdio::inherit`
//! and passing a start that set nothing,
//! so the rule is written as what each start gives its child,
//! read by `service_journal_isolation_starts_tests`
//! (`doc/troubleshooting/git-policy-cli-test-services-write-into-the-user-journal.md`,
//! "The rule as the guard states it").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of testOnlySources()) for (const start of startsOf(file)) expect(judged(file, start)).toEqual([]);
//! ```

/// The lists the rules judge by, each entry with the reason it stays.
#[path = "service_journal_isolation_inventories_tests.rs"]
mod inventories;

/// The reader of child starts: what each start gives its child, and the program it names.
/// A child module, so the guard's own file carries it wherever the guard's file is read from.
#[path = "service_journal_isolation_starts_tests.rs"]
mod starts_reader;
use inventories::{
    ALLOWED_TRIGGER_LINES, CAPTURING_HELPERS, INHERITING_PROGRAMS, INHERITING_STARTS, KEEPS_THE_JOURNAL_OFF, ROADS_TO_THE_SOCKET, SENDERS, SERVICE_LAUNCHERS,
};

use starts_reader::{Start, StandardError, program_name};
use crate::source_scan_tests::{classified_sources, real_roots, source_root};

/// Text that gives a child the streams of the process running the tests:
/// a service started with it writes into the journal whenever that process's standard error is the journal.
const INHERITED_STREAMS: &str = "Stdio::inherit";

/// The words that name the journal's trigger:
/// the variable the service manager sets when a unit's standard error is the journal,
/// and the constant this crate reads it by.
const JOURNAL_TRIGGER_WORDS: &[&str] = &["\"JOURNAL_STREAM\"", "STREAM_VARIABLE"];

/// The support that builds every service a control starts,
/// and the fixture log its streams go to.
const SERVICE_SUPPORT: &str = "binary/binary_support_fixture_service.rs";
const SERVICE_LOG_SUPPORT: &str = "binary/binary_support_fixture_service_log.rs";

/// What:
///  This scan's own file,
///  and the two files it reads its rules from,
///  which the rules skip.
/// Why:
///   They spell every forbidden form in their tables,
/// their controls feed those forms to the detectors,
/// and the reader holds the terminals it looks for as text,
/// so reading them would report the reader.
///  Nothing of them starts a process,
/// and the control below keeps it that way.
const THIS_SCAN: &str = "source_guard/service_journal_isolation_tests.rs";
const THE_READER: &str = "source_guard/service_journal_isolation_starts_tests.rs";
const THE_INVENTORIES: &str = "source_guard/service_journal_isolation_inventories_tests.rs";

/// How a start whose program the guard cannot name is counted in the inventory.
const NO_NAME: &str = "no name";

/// What:
///  The words that name a program of this wrapper inside a shell's script:
///  its executable,
///  the support that builds a service start,
///  and the name a fixture's `git` link carries.
/// Why:
///   A shell start may inherit the run's standard error only while its script runs a command this source writes out,
/// and a script that names the wrapper can run its service,
/// which is the one child whose words the guard is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesWrapper = (chain) => /monochromatic-git-policy-cli|SERVICE_PROGRAM|service_command\(\)|service_executable\(\)|bin\/git/u.test(chain);
/// ```
fn names_a_wrapper_program(chain: &str) -> bool {
    for word in ["monochromatic-git-policy-cli", "SERVICE_PROGRAM", "service_command()", "service_executable()", "bin/git"] {
        if chain.contains(word) {
            return true;
        }
    }
    return false;
}

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
///  The sources the release program is built from, by path relative to `src/native`.
/// Why:
///  The reader needs the crate's own helpers to follow a start this crate spells its own way
/// (`status_retrying`, `git`, `observe`),
/// and those live in the release sources as well as in the test support.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const releaseSources = () => classifiedSources().filter((file) => !file.testOnly).map((file) => file.path)
/// ```
fn release_sources() -> Vec<String> {
    let found = classified_sources(source_root().as_path(), real_roots().as_slice()).expect("the module tree of this crate");
    return found.into_iter().filter(|file| return !file.test_only).map(|file| return file.path).collect();
}

/// What:
///  The lines of `text` that hold `word` in code,
///  read with Rust's own grammar:
///  comments are blanked everywhere,
///  and strings too unless the word being looked for is what a string says.
/// Why:
///   A `//` inside `https://` is no comment's start,
/// and a `Stdio::inherit` written inside a string is no call:
/// a guard that read raw lines missed a forbidden token behind a URL
/// (a review of 2026-11-15, finding 3),
/// and the same reading would have called a quoted token a finding.
///  Masking keeps every line's length and every line's number,
/// so a finding still names the line it is on,
/// and the line a finding quotes is the raw source's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesHolding = (text, word) => maskedFor(word, text).split(NEWLINE).flatMap((line, at) => line.includes(word) ? [[at + 1, rawLine(at)]] : []);
/// ```
fn lines_holding(text: &str, word: &str) -> Vec<(usize, String)> {
    // A word that is what a string says (a quoted name, a path) is read in the text with its strings kept; any other is code.
    let says_a_string: bool = word.starts_with('"') || word.starts_with('/');
    let masked: String = if says_a_string {
        starts_reader::mask_comments(text)
    } else {
        starts_reader::mask(text)
    };
    let mut found: Vec<(usize, String)> = Vec::new();
    for (index, line) in masked.lines().enumerate() {
        if !line.contains(word) {
            continue;
        }
        let raw: &str = text.lines().nth(index).unwrap_or(line);
        found.push((index + 1, raw.trim().to_owned()));
    }
    return found;
}

/// The findings of one word over the test sources, skipping the files the rules live in:
/// `<path>:<line>: <the line>`.
fn findings(word: &str, excused: &[(&str, &str)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for relative in test_only_sources() {
        if relative == THIS_SCAN || relative == THE_READER || relative == THE_INVENTORIES {
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
///  No source compiled for tests only gives a child the streams of the process running the tests,
///  in either spelling:
///  no start passes `Stdio::inherit`,
///  and every start of a child accounts for the child's standard error
///  (a setter of its own,
///  `output()`'s capture,
///  or an inventory entry with its reason).
/// Why:
///   A service started with the run's streams writes wherever that process's standard error points,
///  and a run of the suite under a unit points it at the journal,
///  which is how a throwaway fixture's service said its release into this machine's journal on 2026-11-14.
///  A control that reads what its service said gives it `kept_stderr_pipe`,
///  which the fixture holds.
///  The token is not the rule: `std::process::Command` inherits all three by default,
/// a review of 2026-11-14 showed a guard that looked only for the token passing a start that set nothing,
///  and what a child is given is what this test reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findings('Stdio::inherit', [])).toEqual([]);
/// expect(startFindings(testOnlyListing())).toEqual([]);
/// ```
#[test]
fn no_test_source_gives_a_child_the_streams_of_the_run() {
    let spelled: Vec<String> = findings(INHERITED_STREAMS, &[]);
    assert_eq!(spelled, Vec::<String>::new(), "a control's child would write into whatever stream the run was given, the journal included");
    let listing: Vec<(String, String)> = test_only_sources().into_iter().map(|relative| { return (relative.clone(), source_text(relative.as_str())); }).collect();
    let inherited: Vec<String> = start_findings(listing.as_slice());
    assert_eq!(inherited, Vec::<String>::new(), "a child of a test would write into the stream the run was given");
}

/// What:
///  No source compiled for tests only names the journal's trigger,
///  but the lines listed with their reasons.
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
///  The findings of the start rule over one listing of sources,
///  as `<path>:<line>: <why not>: <the start's clause>`.
/// Why:
///   The rule is read once here and fed two listings:
/// the crate's own test-only sources,
///  and the fixture that reproduces the review's case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startFindings(sources) { return sources.flatMap(([path, text]) => starts(text).filter(inherits).filter(notExcused).map(report)); }
/// ```
fn start_findings(sources: &[(String, String)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut counted: std::collections::BTreeMap<(String, String), usize> = std::collections::BTreeMap::new();
    let mut everything: Vec<String> = Vec::new();
    for relative in release_sources() {
        everything.push(source_text(relative.as_str()));
    }
    for (_, text) in sources {
        everything.push(text.clone());
    }
    let mut names: starts_reader::Names = starts_reader::read_names(everything.as_slice());
    for (helper, _why) in CAPTURING_HELPERS {
        let _read: Option<starts_reader::Shape> = names.starts.insert(String::from(*helper), starts_reader::Shape { captures: true, sets_error: false, sets_all: false });
    }
    for (relative, text) in sources {
        for start in starts_reader::starts_following(text.as_str(), &names) {
            if start.standard_error == StandardError::Given {
                continue;
            }
            if let Some(why) = why_a_finding(&start) {
                found.push(format!("{relative}:{}: {why}: {}", start.line, start.anchor));
                continue;
            }
            let what: String = what_the_child_is(&start);
            *counted.entry((relative.clone(), what.clone())).or_insert(0) += 1;
        }
    }
    found.extend(entries_that_no_longer_match(&counted, sources));
    found.sort();
    return found;
}

/// The word a start is counted by in the inventory:
/// the program's name for a shell,
/// the program expression for any other named program,
/// and `no name` for a start the reader cannot name.
fn what_the_child_is(start: &Start) -> String {
    return match start.program.as_deref() {
        None => String::from(NO_NAME),
        Some(program) if is_a_shell(program) => String::from(program_name(program)),
        Some(program) => String::from(program),
    };
}

/// Whether a program expression names a shell or a runner that takes its program as an argument.
fn is_a_shell(program: &str) -> bool {
    let name: &str = program_name(program);
    return matches!(name, "sh" | "bash" | "/bin/sh" | "/usr/bin/sh" | "/bin/bash" | "/usr/bin/bash" | "env" | "/usr/bin/env" | "/usr/bin/timeout");
}

/// What:
///  Why a start that inherits the run's standard error is a finding,
///  or nothing when it is one the guard may not excuse at all.
/// Why:
///   The three forms no entry can allow are the ones that carry a service's words:
///  the service itself,
///  a shell whose script names this wrapper,
///  and a program the inventory does not name at all.
/// The rest are the inventory's to allow or to refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refuses = (start) => start.runsTheService ? 'the service cannot inherit' : namesWrapper(start) ? 'the shell can run the service' : null;
/// ```
fn why_a_finding(start: &Start) -> Option<&'static str> {
    if start.runs_the_service {
        return Some("a start that runs this wrapper's service must give it a standard error of its own");
    }
    if start.program.as_deref().is_some_and(is_a_shell) && names_a_wrapper_program(start.chain.as_str()) {
        return Some("a shell whose script names a program of this wrapper must give the child a standard error of its own");
    }
    return None;
}

/// What:
///  The entries that no longer describe the sources:
///  an entry whose source holds fewer starts of that child than it says,
///  and a counted start the inventory does not cover at all.
/// Why:
///   An entry with a count is what keeps a start from being added under cover of an old reason,
/// and an entry whose count outlives its starts is a reason that no longer describes anything.
///  A program the inventory names is covered for every source;
/// a shell and a start the reader cannot name need their own entry,
/// which is how a new shell in a new place is a decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [key, count] of counted) expect(entry(key)?.count ?? programs(key) ? count : 0).toBe(count);
/// ```
fn entries_that_no_longer_match(counted: &std::collections::BTreeMap<(String, String), usize>, sources: &[(String, String)]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for ((relative, what), count) in counted {
        let allowed_by_program: bool = INHERITING_PROGRAMS.iter().any(|(program, _)| { return *program == *what; });
        if allowed_by_program {
            continue;
        }
        let entry = INHERITING_STARTS.iter().find(|(file, child, _, _)| { return *file == relative.as_str() && *child == what.as_str(); });
        match entry {
            Some((_, _, said, _)) if said == count => {}
            Some((_, _, said, reason)) => found.push(format!("{relative}: the inventory says {said} starts of {what} stay ({reason}), and the source holds {count}")),
            None => found.push(format!("{relative}: {count} start(s) of {what} inherit the run's standard error and no entry with a reason covers them")),
        }
    }
    for (file, child, said, reason) in INHERITING_STARTS {
        if !sources.iter().any(|(path, _)| { return path == file; }) {
            continue;
        }
        let held: usize = counted.get(&(String::from(*file), String::from(*child))).copied().unwrap_or(0);
        if held != *said {
            found.push(format!("{file}: the entry for {said} starts of {child} ({reason}) has no such starts; the source holds {held}"));
        }

    }
    return found;
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
///  The fixture's log of what its services said writes nothing to a stream the fixture does not hold,
///  but one line about a log file that failed,
///  and that line asks first whether this process's standard error is the journal's.
/// Why:
///  That stream is the journal whenever the run was given the journal,
///  and then a fixture's service words would be there although the service itself was kept off it.
///  A file that cannot be written must not lose the words
/// (`kept_stderr_pipe` keeps them in this process),
/// so the one line that says the file failed is the only write outside the fixture,
/// and `names_the_stream`, read from `systemd.exec(5)`, decides it.
///  The file the log support keeps is what a failure quotes
///  (`service_log`),
///  so nothing is lost when the line is not written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(linesHolding(logSupportText, 'std::io::stderr()')).toHaveLength(1);
/// expect(logSupportText).toContain('stderr_is_the_journal()');
/// ```
#[test]
fn the_fixture_service_log_writes_nothing_the_run_owns_but_a_line_that_asks() {
    let log_support: String = source_text(SERVICE_LOG_SUPPORT);
    let writes: Vec<(usize, String)> = lines_holding(log_support.as_str(), "std::io::stderr()");
    assert_eq!(writes.len(), 1, "the log support writes one line outside the fixture, the one that says its file failed: {writes:?}");
    assert!(log_support.contains("stderr_is_the_journal()"), "the line that leaves the fixture must ask whether this process's standard error is the journal's");
    assert!(log_support.contains("names_the_stream("), "the reading of `$JOURNAL_STREAM` is the one `systemd.exec(5)` gives, and it is named here");
    // The write and the question are in one function, so the line cannot be written without asking.
    let before_the_write: &str = &log_support[..log_support.find("std::io::stderr()").expect("the write")];
    let function_start: usize = before_the_write.rfind("fn ").expect("the function that writes");
    assert!(before_the_write[function_start..].contains("stderr_is_the_journal()"), "the write sits in a function that asks first");
}


/// What:
///  The release program reaches `/run/systemd/journal/socket` only by the roads listed,
///  and only where the listed reason says what stops a process a test started.
/// Why:
///   A reviewer of 2026-11-14 found a second road after the streams were closed:
/// the service's own sink was kept off the machine's journal, and the sender the wrapper uses as a client
/// (`client_journal`) still opened the socket from inside a service.
///  A rule that names the roads, and the one place each is chosen, is what makes the next one a decision.
///  The reading is textual and reads release code only:
/// the socket's name and the constructor of its target appear in the listed files and nowhere else,
///  and a `send_to_addr` stands only in the two senders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [file, word] of roads) expect(linesHolding(read(file), word)).not.toHaveLength(0);
/// expect(otherReleaseFilesHolding('journal/socket')).toEqual([]);
/// ```
#[test]
fn the_journal_socket_is_reached_only_by_the_roads_the_inventory_names() {
    // Every release line that names the socket or builds its address must be one the inventory names, for its own file.
    let words: [&str; 2] = ["/run/systemd/journal/socket", "Target::journal()"];
    let mut found: Vec<String> = Vec::new();
    for relative in release_sources() {
        let text: String = source_text(relative.as_str());
        for word in words {
            for (number, line) in lines_holding(text.as_str(), word) {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                let named: bool = ROADS_TO_THE_SOCKET.iter().any(|(file, listed, _)| { return *file == relative && line.contains(listed); });
                if !named {
                    found.push(format!("{relative}:{number}: {line}"));
                }
            }
        }
    }
    // Every road the inventory names must still be there: a reason for a line that went is no reason.
    for (file, word, _) in ROADS_TO_THE_SOCKET {
        assert!(source_text(file).contains(word), "the road {word} in {file} is still needed; remove it from the list when the line goes");
    }
    assert_eq!(found, Vec::<String>::new(), "a road to the machine's journal socket that no entry names");
}

/// What:
///  Each road is chosen in the file the inventory names,
///  `send_to_addr` stands only in the two senders,
///  the client sender refuses inside the service,
///  and the code that records an entry from inside a service routes it through the service's sink.
/// Why:
///   The roads rule above says where;
/// this one says what decides there,
/// so a file that keeps its `Target::journal()` but loses the condition that stops a test's process fails.
///  The condition is read as the words that carry it, and the runtime controls hold the behaviour:
/// `client_journal_tests` for the refusal and the route,
/// and the fixture's merged-words control for a service's entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(recordEntryAsText).toContain('if in_service()'); expect(sendEntryAsText).toContain('if in_service()');
/// ```
#[test]
fn the_client_sender_refuses_inside_the_service_and_a_service_s_entry_goes_to_the_sink() {
    let client: String = source_text("service/client_journal.rs");
    let record: String = starts_reader::function_body(client.as_str(), "record_entry_as").expect("the recorder's body");
    let send: String = starts_reader::function_body(client.as_str(), "send_entry_as").expect("the sender's body");
    assert!(record.contains("in_service()"), "the recorder routes a service's entry through the sink: {record}");
    assert!(record.contains("log_entry_as("), "a service's entry goes through the service's sink: {record}");
    assert!(send.contains("in_service()"), "the client's sender refuses inside the service: {send}");
    assert!(!send.contains("datagram_as(") || send.contains("return false"), "the sender names the datagram only after the refusal");
    for (file, _why) in SENDERS {
        let text: String = source_text(file);
        let sends: usize = lines_holding(text.as_str(), "send_to_addr(").len();
        assert!(sends > 0, "{file} is still a sender; remove it from the list when it sends nothing");
    }
    let mut senders: Vec<String> = Vec::new();
    for relative in release_sources() {
        if SENDERS.iter().any(|(file, _)| { return *file == relative; }) {
            continue;
        }
        let text: String = source_text(relative.as_str());
        if !lines_holding(text.as_str(), "send_to_addr(").is_empty() {
            senders.push(relative);
        }
    }
    assert_eq!(senders, Vec::<String>::new(), "a sender the inventory does not name: {senders:?}");
}

/// What:
///  Every launcher that starts a service in a test keeps the journal off the command it starts.
/// Why:
///   The variable that keeps it off is the shape that failed once:
/// the fixture's main service launcher was written by hand, cleared its environment and never set it.
///  A rule about each launcher's own body is what makes a launcher that forgets it fail here,
///  and a file-level rule is what makes a new launcher in a new file fail too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [file, name] of SERVICE_LAUNCHERS) expect(bodyOf(read(file), name)).toContain('keep_the_journal_off(');
/// ```
#[test]
fn every_service_launcher_keeps_the_journal_off_the_service_it_starts() {
    let mut missing: Vec<String> = Vec::new();
    for (file, name) in SERVICE_LAUNCHERS {
        let text: String = source_text(file);
        let Some(body) = starts_reader::function_body(text.as_str(), name) else {
            panic!("{file} no longer defines {name}; remove it from the list when the launcher goes");
        };
        if !body.contains(KEEPS_THE_JOURNAL_OFF) {
            missing.push(format!("{file}: {name}"));
        }
    }
    assert_eq!(missing, Vec::<String>::new(), "a launcher that starts a service without keeping the journal off it");
    // A test-only source that can start the service names one of the two ways that keep the journal off it.
    let mut unnamed: Vec<String> = Vec::new();
    for relative in test_only_sources() {
        let text: String = source_text(relative.as_str());
        if !text.contains("SERVICE_PROGRAM") && !text.contains("service_executable()") && !text.contains("service_command()") {
            continue;
        }
        if !text.contains(KEEPS_THE_JOURNAL_OFF) && !text.contains("service_command()") && !text.contains("hermetic(") {
            unnamed.push(relative);
        }
    }
    assert_eq!(unnamed, Vec::<String>::new(), "a file that starts a service and keeps the journal off it by no named way");
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
///  The detectors find each forbidden form in a text,
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
    for named in [SERVICE_SUPPORT, SERVICE_LOG_SUPPORT, THE_READER, THE_INVENTORIES] {
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
/// for (const file of [THIS_SCAN, THE_READER, THE_INVENTORIES]) expect(read(file)).not.toMatch(/process::Command|\.spawn\(/);
/// ```
#[test]
fn the_scan_its_own_files_start_no_process() {
    // The two words are spelled in pieces so this control's own lines are not findings of themselves.
    let starting: String = ["Command", "::new"].concat();
    let spawning: String = [".spa", "wn("].concat();
    for relative in [THIS_SCAN, THE_READER, THE_INVENTORIES] {
        let text: String = source_text(relative);
        assert_eq!(lines_holding(text.as_str(), starting.as_str()), Vec::<(usize, String)>::new(), "{relative}");
        assert_eq!(lines_holding(text.as_str(), spawning.as_str()), Vec::<(usize, String)>::new(), "{relative}");
    }
}

/// What:
///  The rule fails on the two starts the review of 2026-11-14 showed it passing:
///  a child started with nothing set,
///  and the same start through a shell,
///  and it passes the forms a control may use.
/// Why:
///  A guard that cannot fail is no guard,
/// and the review's two files are the cases that prove it can:
/// `reproductions/default_inherited_service.rs` started `/bin/sh` with `.status()` and nothing else,
/// and the guard passed it
/// (`reproductions/guard_default_inheritance.output.txt`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(startFindings([['a.rs', 'fn f() { let s = std::process::Command::new("/bin/sh").arg("-c").arg("printf x >&2").status(); }']])).toHaveLength(1);
/// ```
#[test]
fn the_rule_fails_on_a_start_that_sets_nothing() {
    let review_case: Vec<(String, String)> = vec![(
        String::from("review/default_inherited_service.rs"),
        String::from(
            "fn main() {\n    let status: std::process::ExitStatus = std::process::Command::new(\"/bin/sh\")\n        .args([\"-c\", \"printf 'default-inherited-stderr-marker\\\\n' >&2\"])\n        .status()\n        .expect(\"child starts\");\n    assert!(status.success());\n}\n",
        ),
    )];
    let found: Vec<String> = start_findings(review_case.as_slice());
    assert_eq!(found.len(), 1, "the review's start must be a finding: {found:?}");
    assert!(found[0].contains("1 start(s) of /bin/sh"), "{found:?}");
    assert!(found[0].starts_with("review/default_inherited_service.rs:"), "{found:?}");
}

/// What:
///  A start that gives the child a standard error of its own is no finding,
///  whatever program it runs,
///  and neither is a start of a program the inventory names,
///  and a start of the service is a finding however its streams are placed elsewhere.
/// Why:
///  The rule must not be so wide that a control cannot start a shell it configures,
/// and it must not be so narrow that a start of the service passes:
/// both directions are the review's second finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(startFindings([['a.rs', configured]])).toEqual([]); expect(startFindings([['a.rs', serviceStart]])).toHaveLength(1);
/// ```
#[test]
fn a_configured_shell_is_no_finding_and_an_uncovered_start_is_one() {
    let configured: Vec<(String, String)> = vec![(
        String::from("a.rs"),
        String::from("fn f() {\n    let out = std::process::Command::new(\"/bin/sh\").arg(\"-c\").arg(\"true\").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();\n}\n"),
    )];
    assert_eq!(start_findings(configured.as_slice()), Vec::<String>::new());
    let captured: Vec<(String, String)> = vec![(
        String::from("a.rs"),
        String::from("fn f() {\n    let out = std::process::Command::new(\"/bin/sh\").arg(\"-c\").arg(\"true\").output();\n}\n"),
    )];
    assert_eq!(start_findings(captured.as_slice()), Vec::<String>::new());
    let service: Vec<(String, String)> = vec![(
        String::from("a.rs"),
        String::from("fn f() {\n    let mut command = super::support::service_command();\n    command.args([\"service\", \"run\"]);\n    let child = command.spawn();\n}\n"),
    )];
    let found: Vec<String> = start_findings(service.as_slice());
    assert_eq!(found.len(), 1, "a start of the service never inherits: {found:?}");
    assert!(found[0].contains("runs this wrapper's service"), "{found:?}");
}

/// What:
///  A start the reader cannot name is a finding until an entry counts it,
///  and an entry that counts more starts than the source holds is a finding too.
/// Why:
///  The inventory is what a reader's gap is written down in,
/// and a gap that grew or shrank silently would make the list a story rather than a rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(startFindings([['a.rs', helperBuilt]])).toHaveLength(1);
/// ```
#[test]
fn a_start_the_reader_cannot_name_needs_its_own_entry() {
    let helper_built: Vec<(String, String)> = vec![(
        String::from("a.rs"),
        String::from("fn f() {\n    let mut command = direct(&fixture);\n    command.args([\"fsck\"]).status().expect(\"fsck\");\n}\n"),
    )];
    let found: Vec<String> = start_findings(helper_built.as_slice());
    assert_eq!(found.len(), 1, "a start the reader cannot name must be a finding: {found:?}");
    assert!(found[0].contains("no name"), "{found:?}");
}



/// What:
///  A start this crate spells its own way is a start:
///  `.status_retrying()` on a command that names the service mode, with no setter, is read and refused.
/// Why:
///   A reviewer of 2026-11-14 found the reader listing four terminal names and passing this one,
///  although it ends in `Command::status` and inherits the run's standard error.
///  A list of names misses the next name;
///  a chain that ends in a method which is no configuration call is a start whatever the name,
///  which is what this control holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(startsFollowing(retryingSource).map((start) => start.standardError)).toEqual(['inherited']); expect(startFindings([['a.rs', retryingSource]])).toHaveLength(1);
/// ```
#[test]
fn a_start_that_retries_is_still_a_start() {
    let source: String = String::from(
        "fn start() {\n    let mut command = Command::new(SERVICE_PROGRAM);\n    command.args(SERVICE_WORDS);\n    let status = command.status_retrying().expect(\"service starts\");\n}\n",
    );
    let names: starts_reader::Names = starts_reader::read_names(&[source.as_str().to_owned()]);
    let found: Vec<Start> = starts_reader::starts_following(source.as_str(), &names);
    assert_eq!(found.len(), 1, "the retrying start is read: {found:?}");
    assert_eq!(found[0].standard_error, StandardError::Inherited, "it sets no standard error: {found:?}");
    assert!(found[0].runs_the_service, "its words name the service mode: {found:?}");
    assert_eq!(found[0].followed.as_deref(), Some("status_retrying"), "the reader names the helper it followed: {found:?}");
    let listing: Vec<(String, String)> = vec![(String::from("a.rs"), source)];
    let findings: Vec<String> = start_findings(listing.as_slice());
    assert_eq!(findings.len(), 1, "the rule refuses it: {findings:?}");
    assert!(findings[0].contains("runs this wrapper's service"), "{findings:?}");
}

/// What:
///  Both new guards read Rust's tokens, not raw lines:
///  a `//` or a `/*` inside a string opens nothing,
///  a quote inside a comment opens nothing,
///  an escaped quote ends no string early,
///  a raw string is a value,
///  a setter split over lines is one chain,
///  and a setter is judged by what it is given.
/// Why:
///   A review of 2026-11-15 reproduced both false negatives with one legal line,
/// `c.args(SERVICE_WORDS).arg("https://example.invalid").stderr(Stdio::inherit()).spawn();`:
/// the `//` of the URL was taken for a comment's start,
/// and every `.stderr(...)` was taken for a stream given.
///  The cases below are what a reader of another syntax needs:
/// a quote,
/// an escape,
/// a raw string,
/// a block comment,
/// and a chain broken over lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(linesHolding(reviewLine, 'Stdio::inherit')).toHaveLength(1); expect(judged(reviewLine)).toHaveLength(1);
/// ```
#[test]
fn the_guards_read_rust_s_tokens_and_judge_a_setter_by_what_it_is_given() {
    let review_line: &str = "fn f() { let mut c = Command::new(SERVICE_PROGRAM); c.args(SERVICE_WORDS).arg(\"https://example.invalid\").stderr(Stdio::inherit()).spawn(); }\n";
    assert_eq!(lines_holding(review_line, INHERITED_STREAMS).len(), 1, "the token behind a URL is found");
    let names: starts_reader::Names = starts_reader::read_names(&[String::from(review_line)]);
    let found: Vec<Start> = starts_reader::starts_following(review_line, &names);
    assert_eq!(found.len(), 1, "the start is read: {found:?}");
    assert_eq!(found[0].standard_error, StandardError::Inherited, "a setter given the parent's stream is no isolation: {found:?}");
    assert!(found[0].runs_the_service, "its words name the service mode");
    let findings: Vec<String> = start_findings(&[(String::from("probe.rs"), String::from(review_line))]);
    assert_eq!(findings.len(), 1, "the rule refuses it: {findings:?}");

    // A `//` and a `/*` inside a string are no comment's start: what follows them on the line is still read.
    let in_a_string: &str = "fn f() { let note = \"a // b /* c */ d\"; let after = Needle; }\n";
    assert_eq!(lines_holding(in_a_string, "Needle").len(), 1, "a comment's start inside a string hides nothing after it");
    // A quote inside a comment opens no string, so a token after the comment is still comment text.
    let quote_in_a_comment: &str = "fn f() { /* it's \"quoted\" */ let after = Needle; }\n";
    assert_eq!(lines_holding(quote_in_a_comment, "quoted").len(), 0, "a quoted word inside a comment is no code");
    // An escaped quote ends no string early.
    let escaped: &str = "fn f() { let s = \"a \\\" b\"; let c = Command::new(\"x\").stderr(Stdio::inherit()).spawn(); }\n";
    assert_eq!(lines_holding(escaped, INHERITED_STREAMS).len(), 1, "an escaped quote ends no string early");
    // A raw string is a value: its `//` is no comment and its contents are no call.
    let raw: &str = "fn f() { let s = r#\"a // Stdio::inherit b\"#; let c = Command::new(\"x\").stderr(Stdio::null()).spawn(); }\n";
    assert_eq!(lines_holding(raw, INHERITED_STREAMS).len(), 0, "nothing inside a raw string is a call");
    let after_raw: &str = "fn f() { let s = r#\"a // b\"#; let c = Command::new(\"x\").arg(\"y\").stderr(Stdio::inherit()).spawn(); }\n";
    assert_eq!(lines_holding(after_raw, INHERITED_STREAMS).len(), 1, "a raw string's `//` hides nothing after it");
    // A setter split over lines is one chain, and a stream given is no isolation.
    let split: &str = "fn f() { let mut c = Command::new(SERVICE_PROGRAM);\n    c.args(SERVICE_WORDS)\n        .stderr(Stdio::inherit())\n        .spawn();\n}\n";
    let split_names: starts_reader::Names = starts_reader::read_names(&[String::from(split)]);
    let split_starts: Vec<Start> = starts_reader::starts_following(split, &split_names);
    assert_eq!(split_starts.len(), 1, "the split chain is one start: {split_starts:?}");
    assert_eq!(split_starts[0].standard_error, StandardError::Inherited, "a setter given the parent's stream is no isolation: {split_starts:?}");
    // A setter given a stream of the fixture's own is isolation, and the same line with it is no finding.
    let configured: &str = "fn f() { let mut c = Command::new(\"x\"); c.arg(\"https://example.invalid\").stderr(Stdio::null()).spawn(); }\n";
    let configured_names: starts_reader::Names = starts_reader::read_names(&[String::from(configured)]);
    assert_eq!(starts_reader::starts_following(configured, &configured_names)[0].standard_error, StandardError::Given);
}
