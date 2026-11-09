//! What:
//!  Controls for the marker every real-Git start carries and for the check that stops a wrapper started as Git.
//! Why:
//!  The check decides from made-up process numbers and birth identities,
//! so each way a start can look (the starter
//!  replaced itself,
//! the starter is the parent,
//! something else stands in between,
//! a number was reused) is shown without
//!  starting a process;
//! one control then reads the marker the real builders put on a command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(startedAsGit({ starterPid: 10, mode: 'child' }, { ownPid: 12, parentPid: 10 })).toBe(true);
//! ```
#![cfg(unix)]

/// The marker,
/// its parser,
/// the check and the builders under test.
use super::{
    ANCESTORS_ASKED_FOR_THE_CHAIN, FIELD_SEPARATOR, MOST_STARTS_IN_ONE_CHAIN, ProcessFacts, STARTED_AS_GIT_VARIABLE, StartMarker,
    StartMode, ancestor_environments, chain_marker_and_where, chain_marker_among, environment_of, marker_of, marker_value,
    mark_real_git_start, parent_of, parse_marker, refusal_text, refusal_when_started_as_git, refusal_when_started_as_git_among,
    started_as_git,
};
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

/// The identity a control gives the starter.
const STARTER_IDENTITY: &str = "linux:1000";

/// A marker of a start by process `pid` in the given mode,
/// with the starter's identity,
/// as the first start of its chain.
fn marker(pid: u32, mode: StartMode) -> StartMarker {
    return StartMarker { starter_pid: pid, starter_identity: STARTER_IDENTITY.to_string(), mode, starts: 1 };
}

/// Whether the check judges a process with the numbers `own` and `parent` to be the start `marker` names,
/// when the
///  identity of the candidate process reads as `identity_of_candidate` (`None`:
/// unreadable).
fn started(marker: &StartMarker, own: u32, parent: u32, identity_of_candidate: Option<&str>) -> bool {
    let answer = |_pid: u32| -> Option<String> { return identity_of_candidate.map(str::to_string); };
    let facts: ProcessFacts<'_> = ProcessFacts { own_pid: own, parent_pid: parent, identity_of: &answer };
    return started_as_git(marker, &facts);
}

/// The marker value round-trips through its parser for both modes,
/// and text that is not a marker is none.
#[test]
fn a_marker_value_is_read_back_and_other_text_is_no_marker() {
    for mode in [StartMode::Replace, StartMode::Child, StartMode::Query] {
        for starts in [1u32, MOST_STARTS_IN_ONE_CHAIN + 5] {
            let value: OsString = marker_value(mode, 4242, STARTER_IDENTITY, starts);
            assert_eq!(parse_marker(value.as_os_str()), Some(StartMarker { starts, ..marker(4242, mode) }), "{mode:?} {starts}");
        }
    }
    let empty_identity: OsString = marker_value(StartMode::Child, 7, "", 1);
    assert_eq!(parse_marker(empty_identity.as_os_str()).map(|read| { return read.starter_identity; }), Some(String::new()));
    // A marker of three fields is one an earlier release wrote, which never counted its chain: it is the first start.
    let uncounted: OsString = OsString::from(format!("4242{FIELD_SEPARATOR}{STARTER_IDENTITY}{FIELD_SEPARATOR}child"));
    assert_eq!(parse_marker(uncounted.as_os_str()), Some(marker(4242, StartMode::Child)));
    for other in [
        "",
        "1234",
        "abc\u{1f}x\u{1f}child",
        "12\u{1f}x\u{1f}sideways",
        "12\u{1f}x",
        "12\u{1f}x\u{1f}child\u{1f}-1",
        "12\u{1f}x\u{1f}child\u{1f}two",
        "12\u{1f}x\u{1f}child\u{1f}1\u{1f}extra",
        "-5\u{1f}x\u{1f}child\u{1f}1",
    ] {
        assert_eq!(parse_marker(OsString::from(other).as_os_str()), None, "{other:?}");
    }
}

/// A start that replaced its starter is the starter's own process:
/// it matches by the program's own number,
/// and a
///  parent with that number alone does not make it one.
#[test]
fn a_replaced_start_is_the_process_the_marker_names() {
    let replace: StartMarker = marker(100, StartMode::Replace);
    assert!(started(&replace, 100, 1, Some(STARTER_IDENTITY)), "same process");
    assert!(!started(&replace, 101, 100, Some(STARTER_IDENTITY)), "a child of the real Git that replaced the wrapper is a hook, not the start");
    assert!(!started(&replace, 101, 1, Some(STARTER_IDENTITY)), "another process");
}

/// A child start is the starter's child:
/// it matches by the parent's number,
/// and the starter's own number alone does not
///  (a child never has the starter's number).
#[test]
fn a_child_start_is_a_child_of_the_process_the_marker_names() {
    let child: StartMarker = marker(100, StartMode::Child);
    assert!(started(&child, 200, 100, Some(STARTER_IDENTITY)), "child of the starter");
    assert!(!started(&child, 200, 150, Some(STARTER_IDENTITY)), "something stands between: a hook or an alias of the real Git");
    assert!(!started(&child, 100, 1, Some(STARTER_IDENTITY)), "the number alone is not the child start");
}

/// A number that a later process reuses is not the starter:
/// its identity differs;
/// an identity that cannot be read does
///  not contradict the marker,
/// and a marker without an identity decides by the numbers.
#[test]
fn a_reused_number_is_not_the_starter() {
    let child: StartMarker = marker(100, StartMode::Child);
    assert!(!started(&child, 200, 100, Some("linux:2000")), "another process holds the number now");
    assert!(started(&child, 200, 100, None), "unreadable identity decides by the numbers");
    let without: StartMarker = StartMarker { starter_pid: 100, starter_identity: String::new(), mode: StartMode::Child, starts: 1 };
    assert!(started(&without, 200, 100, Some("linux:2000")), "no recorded identity to compare");
}

/// What:
///  A start too far down one chain is the wrapper leading back to itself,
/// whatever process stands between its starts,
/// and one at the bound is not.
/// Why:
///  A script named `git` that starts the wrapper as a child puts itself between the two,
/// so neither the number of the
///  marked process nor the parent's number matches,
/// and the covered chain never ended
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// "Reproductions" A).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(started({ starts: MOST + 1, mode: 'replace' }, 200, 150)).toBe(true);
/// expect(started({ starts: MOST, mode: 'replace' }, 200, 150)).toBe(false);
/// ```
#[test]
fn a_chain_longer_than_the_bound_is_the_wrapper_leading_back_to_itself() {
    let at_the_bound: StartMarker = StartMarker { starts: MOST_STARTS_IN_ONE_CHAIN, ..marker(100, StartMode::Replace) };
    assert!(!started(&at_the_bound, 200, 150, Some(STARTER_IDENTITY)), "one at the bound is an ordinary nesting");
    let past_the_bound: StartMarker = StartMarker { starts: MOST_STARTS_IN_ONE_CHAIN + 1, ..marker(100, StartMode::Replace) };
    assert!(started(&past_the_bound, 200, 150, Some(STARTER_IDENTITY)), "no number matches, and the chain is too long");
    let child_past_the_bound: StartMarker = StartMarker { starts: MOST_STARTS_IN_ONE_CHAIN + 1, ..marker(100, StartMode::Child) };
    assert!(started(&child_past_the_bound, 200, 150, Some("linux:2000")), "an identity that differs does not save a chain this long");
    assert!(started(&child_past_the_bound, 200, 100, Some("linux:2000")), "it is stopped by the bound, not by the parent");
}

/// What:
///  A wrapper that arrives as the Git of one of the wrapper's own reads stops at once,
/// whatever stands between the two.
/// Why:
///  Such a start runs nothing of the person's:
/// its output is captured and its input closed,
/// so a wrapper here
///  came from the file that start chose as Git.
/// Without this the chain branched at every level,
/// because each level's reads started chains of their own:
/// a probe measured 511 starts of one script named `git` before its own cap ended them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(started({ mode: 'query', starts: 1 }, 200, 150)).toBe(true);
/// ```
#[test]
fn a_wrapper_that_arrives_as_one_of_the_wrappers_own_reads_stops_at_once() {
    let query: StartMarker = marker(100, StartMode::Query);
    assert!(started(&query, 200, 150, Some(STARTER_IDENTITY)), "neither number matches, and the read runs nothing of the person's");
    assert!(started(&query, 100, 1, Some(STARTER_IDENTITY)), "even the starter's own number is no reason to go on");
    let said: String = refusal_text(&query, Path::new("/wrapper"), &[]);
    assert!(said.contains("started as Git to read the repository itself"), "{said}");
    assert!(said.contains("process 100"), "{said}");
}

/// The environment carries the marker only under its own name,
/// and only a well-formed value counts.
#[test]
fn the_environment_marker_is_read_by_name() {
    let value: OsString = marker_value(StartMode::Child, 9, STARTER_IDENTITY, 1);
    let present: Vec<(OsString, OsString)> = vec![(OsString::from(STARTED_AS_GIT_VARIABLE), value.clone())];
    assert_eq!(marker_of(present.as_slice()), Some(marker(9, StartMode::Child)));
    let misnamed: Vec<(OsString, OsString)> = vec![(OsString::from("CLI_GIT_STARTED_AS_GIT_NOT"), value)];
    assert_eq!(marker_of(misnamed.as_slice()), None);
    let garbled: Vec<(OsString, OsString)> = vec![(OsString::from(STARTED_AS_GIT_VARIABLE), OsString::from("1"))];
    assert_eq!(marker_of(garbled.as_slice()), None);
    assert_eq!(marker_of(&[]), None);
}

/// This test process is neither the starter nor its child,
/// so a marker naming other processes does not stop it,
/// and a
///  marker naming this very process as a replacing starter does.
#[test]
fn the_live_check_reads_this_process() {
    let own_pid: u32 = std::process::id();
    let identity: String = crate::process_identity::current_birth_identity().expect("this process has an identity");
    let own_marker = |pid: u32, mode: StartMode| -> Vec<(OsString, OsString)> {
        return vec![(OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(mode, pid, identity.as_str(), 1))];
    };
    let elsewhere: u32 = own_pid.wrapping_add(7);
    assert_eq!(refusal_when_started_as_git(own_marker(elsewhere, StartMode::Replace).as_slice(), Path::new("/wrapper")), None);
    assert_eq!(refusal_when_started_as_git(own_marker(elsewhere, StartMode::Child).as_slice(), Path::new("/wrapper")), None);
    let said: String = refusal_when_started_as_git(own_marker(own_pid, StartMode::Replace).as_slice(), Path::new("/wrapper"))
        .expect("this process is the replaced starter");
    assert!(said.contains("started as Git"), "{said}");
    assert!(said.contains("/wrapper"), "{said}");
    // The parent of this test process is the process that started it; a child marker naming it matches.
    // SAFETY: `getppid` takes no argument, reads no memory and cannot fail.
    let parent: u32 = u32::try_from(unsafe { libc::getppid() }).expect("a parent number");
    let parent_identity: String = crate::process_identity::process_birth_identity(i64::from(parent))
        .expect("readable").expect("parent runs");
    let named_parent: Vec<(OsString, OsString)> =
        vec![(OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(StartMode::Child, parent, parent_identity.as_str(), 1))];
    assert!(refusal_when_started_as_git(named_parent.as_slice(), Path::new("/wrapper")).is_some());
}

/// The message names the executable,
/// the starter,
/// the way it started,
/// the Git it chose and the PATH,
/// and every way out.
#[test]
fn the_refusal_names_what_a_person_needs() {
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("CLI_GIT_NATIVE_FORWARD_TARGET"), OsString::from("/opt/launchers/git")),
        (OsString::from("PATH"), OsString::from("/opt/launchers")),
    ];
    let said: String = refusal_text(&marker(321, StartMode::Child), Path::new("/opt/other-release/monochromatic-git-policy-cli"), environment.as_slice());
    for needle in [
        "/opt/other-release/monochromatic-git-policy-cli",
        "was started as Git by a monochromatic-git-policy-cli wrapper",
        "process 321",
        "waits for its answer",
        "/opt/launchers/git",
        "PATH here is /opt/launchers",
        "never runs itself as Git",
        "(for example /usr/bin)",
        "remove the entry that leads back to monochromatic-git-policy-cli",
    ] {
        assert!(said.contains(needle), "{needle}: {said}");
    }
    let replaced: String = refusal_text(&marker(321, StartMode::Replace), Path::new("/x"), &[]);
    assert!(replaced.contains("replaced with this program"), "{replaced}");
    assert!(replaced.contains("PATH is not set here"), "{replaced}");
    // A chain past the bound says how long it is and still names the Git and the PATH, so a person can find the entry.
    let chained: String = refusal_text(
        &StartMarker { starts: MOST_STARTS_IN_ONE_CHAIN + 1, ..marker(321, StartMode::Replace) },
        Path::new("/x"),
        environment.as_slice(),
    );
    assert!(chained.contains(format!("start {}", MOST_STARTS_IN_ONE_CHAIN + 1).as_str()), "{chained}");
    assert!(chained.contains("leads back to monochromatic-git-policy-cli"), "{chained}");
    assert!(chained.contains("/opt/launchers/git"), "{chained}");
    assert!(chained.contains("PATH here is /opt/launchers"), "{chained}");
    assert!(chained.contains("remove the entry that leads back to monochromatic-git-policy-cli"), "{chained}");
}

/// What:
///  A start whose own environment carries no marker continues the chain of the nearest process above it that does.
/// Why:
///  `env --ignore-environment` clears the environment it passes on and not the one of the process that ran it,
/// so a
///  launcher that wipes the marker cannot hide the count from the wrapper it starts
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the round-1 review);
/// without this the count of every level was zero and the chain
///  never ended.
/// What this process carries itself is the record of its own chain,
/// so an ancestor says nothing when this
///  environment has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(chainMarker(own = [], ancestors = [unmarked, marked(starts: 3)])).toEqual({ starts: 3 });
/// ```
#[test]
fn a_chain_whose_own_environment_has_no_marker_continues_the_nearest_one_above_it() {
    let own: Vec<(OsString, OsString)> = vec![(OsString::from("PATH"), OsString::from("/nowhere"))];
    let unmarked: Vec<(OsString, OsString)> = vec![(OsString::from("HOME"), OsString::from("/home/someone"))];
    let nearer: Vec<(OsString, OsString)> = vec![(OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(StartMode::Child, 700, STARTER_IDENTITY, 2))];
    let further: Vec<(OsString, OsString)> = vec![(OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(StartMode::Child, 600, STARTER_IDENTITY, 5))];
    assert_eq!(chain_marker_among(own.as_slice(), &[]), None, "nothing above and nothing here");
    assert_eq!(chain_marker_among(own.as_slice(), std::slice::from_ref(&unmarked)), None, "an ancestor without a marker says nothing");
    assert_eq!(
        chain_marker_among(own.as_slice(), &[unmarked.clone(), nearer.clone(), further.clone()]),
        Some(StartMarker { starts: 2, ..marker(700, StartMode::Child) }),
        "the nearest marker above"
    );
    let mut with_own: Vec<(OsString, OsString)> = own.clone();
    with_own.push((OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(StartMode::Child, 800, STARTER_IDENTITY, 1)));
    assert_eq!(
        chain_marker_among(with_own.as_slice(), std::slice::from_ref(&nearer)),
        Some(marker(800, StartMode::Child)),
        "this process's own record is its chain"
    );
    let above: Vec<Vec<(OsString, OsString)>> = vec![unmarked, nearer];
    let (found, said_in) = chain_marker_and_where(own.as_slice(), above.as_slice()).expect("the marker above");
    assert_eq!(found.starts, 2);
    assert_eq!(said_in, above.get(1).expect("the second ancestor").as_slice(), "the environment the message is written from");
}

/// What:
///  A start that continues a chain past the bound is refused,
/// and the message names the Git and the PATH of the environment
///  the marker was read from.
/// Why:
///   A chain of launchers that clear the environment never writes those two variables into the arriving wrapper's
///  environment,
/// so without the chain's own environment the message a person needs would name nothing they can
///  remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refusalWhenStartedAsGitAmong([], '/wrapper', [above])).toContain('/opt/launchers/git');
/// ```
#[test]
fn a_chain_refused_above_names_the_git_and_the_path_it_was_told_about() {
    let above: Vec<(OsString, OsString)> = vec![
        (OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(StartMode::Replace, 700, STARTER_IDENTITY, MOST_STARTS_IN_ONE_CHAIN + 1)),
        (OsString::from("CLI_GIT_NATIVE_FORWARD_TARGET"), OsString::from("/opt/launchers/git")),
        (OsString::from("PATH"), OsString::from("/opt/launchers")),
    ];
    let said: String = refusal_when_started_as_git_among(Vec::new().as_slice(), Path::new("/wrapper"), &[above]).expect("a chain past the bound");
    assert!(said.contains("/opt/launchers/git"), "{said}");
    assert!(said.contains("PATH here is /opt/launchers"), "{said}");
    assert!(said.contains(format!("start {}", MOST_STARTS_IN_ONE_CHAIN + 1).as_str()), "{said}");
    assert!(said.contains("started as Git"), "{said}");
    assert_eq!(refusal_when_started_as_git_among(Vec::new().as_slice(), Path::new("/wrapper"), &[]), None, "nothing to continue");
}

/// What:
///  The live reads of the chain walk:
///  this process's own environment,
///  its parent's number,
///  and the walk itself.
/// Why:
///   The walk is the one part the controls above hand made-up values to;
/// a read that cannot find the process or the
///  variable must end it rather than fail the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(environmentOf(process.pid).PATH).toBeDefined(); expect(parentOf(process.pid)).toBe(process.ppid);
/// ```
#[test]
fn the_chain_walk_reads_this_process_and_the_ones_above_it() {
    let own: Vec<(OsString, OsString)> = environment_of(std::process::id()).expect("this process has an environment");
    assert!(own.iter().any(|(name, _value)| { return name == "PATH"; }), "the test process has a PATH: {} entries", own.len());
    // SAFETY: `getppid` takes no argument, reads no memory of this process and cannot fail.
    let parent: u32 = u32::try_from(unsafe { libc::getppid() }).expect("a parent number");
    assert_eq!(parent_of(std::process::id()), Some(parent), "the parent's number as the process table says it");
    let above: Vec<Vec<(OsString, OsString)>> = ancestor_environments();
    assert!(!above.is_empty(), "a test process is started by another");
    assert!(above.len() <= ANCESTORS_ASKED_FOR_THE_CHAIN);
    assert_eq!(environment_of(0), None, "there is no process zero");
    assert_eq!(parent_of(u32::MAX), None, "there is no process with that number");
}

/// The builders put the marker of the right mode on a command,
/// naming this process,
/// and a later mark overrides the
///  earlier one (the exec path marks "replace" over the shared builder's "child").
#[test]
fn marking_a_command_names_this_process_and_the_last_mark_wins() {
    let mut command: Command = Command::new("/bin/true");
    mark_real_git_start(&mut command, StartMode::Child);
    mark_real_git_start(&mut command, StartMode::Replace);
    let value: OsString = command
        .get_envs()
        .find(|(name, _value)| { return *name == STARTED_AS_GIT_VARIABLE; })
        .and_then(|(_name, value)| { return value.map(OsString::from); })
        .expect("the command carries the marker");
    let read: StartMarker = parse_marker(value.as_os_str()).expect("a marker");
    assert_eq!(read.starter_pid, std::process::id());
    assert_eq!(read.mode, StartMode::Replace);
    // This process was started by the test harness, which carries no marker: the start it makes is the first of its chain.
    assert_eq!(read.starts, 1);
    assert_eq!(command.get_envs().filter(|(name, _value)| { return *name == STARTED_AS_GIT_VARIABLE; }).count(), 1);
}

/// What:
///  A record this very process wrote for its Git is read even when an older marker sits in the
///  environment above.
/// Why:
///  A launcher that clears the environment and replaces itself with the wrapper leaves the marker
/// of the re-entering start
///  only in the chain record;
/// the environment above still holds the chain's older marker,
/// which names
///  the outer start and matches nothing,
/// and the guard returned there without reading the record,
/// so the
///  wrapper replaced itself with the launcher without end
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 1 of the
///  round-3 review).
/// The record names this process,
///  so it is the process starting itself again whatever the marker above says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rememberMarker({ pid: process.pid, mode: 'replace' }); expect(refusalAmong([], [olderMarkerAbove])).toBeDefined();
/// ```
#[test]
fn a_record_of_this_process_is_read_even_when_an_older_marker_sits_above() {
    if std::env::var_os(RECORD_ABOVE_CHILD_VARIABLE).is_some() {
        let identity: String = crate::process_identity::current_birth_identity().expect("this process has an identity");
        // The record the start above left in the descriptor this process inherited: the launcher replaced this very
        //  process with the wrapper again.
        let inherited: i32 = crate::chain_starts_record::create_descriptor().expect("a record is made");
        let written: String = marker_value(StartMode::Replace, std::process::id(), identity.as_str(), 2).to_string_lossy().into_owned();
        assert!(crate::chain_starts_record::write_marker_for_controls(inherited, written.as_str()), "the record holds the start above");
        // The chain's older marker names another process and matches nothing here.
        let above: Vec<(OsString, OsString)> =
            vec![(OsString::from(STARTED_AS_GIT_VARIABLE), marker_value(StartMode::Child, 4242, "linux:1", 1))];
        let said: String = refusal_when_started_as_git_among(Vec::new().as_slice(), Path::new("/wrapper"), std::slice::from_ref(&above))
            .expect("the record names this process, which started itself again");
        assert!(said.contains("replaced with this program"), "{said}");
        assert!(said.contains("/wrapper"), "{said}");
        return;
    }
    // The record belongs to one process, so the body runs in a child and the parent asserts its report.
    let output: std::process::Output = crate::test_support::child_test_process::run(
        RECORD_ABOVE_CHILD_VARIABLE,
        "invocation::started_as_git::tests::a_record_of_this_process_is_read_even_when_an_older_marker_sits_above",
    );
    let said: String = format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
    assert!(output.status.success(), "the record of this process was not read:\n{said}");
    assert!(said.contains("1 passed"), "the child ran no test:\n{said}");
}

/// The variable that marks the child of the record control,
/// whose record belongs to that child alone.
const RECORD_ABOVE_CHILD_VARIABLE: &str = "CLI_GIT_RECORD_ABOVE_CHILD";

/// What:
///  A record whose start is a live process above this one continues the chain,
///  however many relays stand
///  between the two.
/// Why:
///  Two shells that only relay the invocation put two processes between the wrapper that wrote the
/// record and the
///  wrapper that reads it,
/// and a rule that accepted only this process or its parent counted every such generation from
///  one:
/// the ceiling never applied and wrappers and relays accumulated
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the
///  round-3 review).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(recordContinues({ starterPid: grandparent }, [own, parent, grandparent])).toBe(true);
/// expect(recordContinues({ starterPid: sibling }, [own, parent, grandparent])).toBe(false);
/// ```
#[test]
fn a_record_of_a_live_process_above_continues_the_chain_through_relays() {
    let identity: String = crate::process_identity::current_birth_identity().expect("this process has an identity");
    let own: u32 = std::process::id();
    let ancestors: [u32; 3] = [own, parent_pid(), own.wrapping_add(4_242)];
    let of_this_process: StartMarker = StartMarker { starter_pid: own, starter_identity: identity.clone(), ..marker(own, StartMode::Replace) };
    assert!(super::record_continues(&of_this_process, ancestors.as_slice()), "this process");
    let of_a_relay: StartMarker = StartMarker { starter_pid: own.wrapping_add(4_242), starter_identity: identity.clone(), ..marker(0, StartMode::Replace) };
    assert!(super::record_continues(&of_a_relay, ancestors.as_slice()), "a live process above, through its identity");
    let of_a_sibling: StartMarker = StartMarker { starter_pid: 1, starter_identity: identity, ..marker(1, StartMode::Replace) };
    assert!(!super::record_continues(&of_a_sibling, ancestors.as_slice()), "a process outside this chain");
    // A read of the wrapper's own continues the chain from any process of it, as it does through a marker.
    let read_above: StartMarker =
        StartMarker { starter_pid: own.wrapping_add(4_242), starter_identity: String::from("linux:1"), ..marker(0, StartMode::Query) };
    assert!(super::record_continues(&read_above, ancestors.as_slice()), "a read above is the wrapper leading back to itself");
    let read_here: StartMarker = StartMarker { starter_pid: own, starter_identity: String::new(), ..marker(own, StartMode::Query) };
    assert!(super::record_continues(&read_here, ancestors.as_slice()), "a read this process made, arriving again");
}

/// The number of this process's parent,
/// for the controls above.
fn parent_pid() -> u32 {
    // SAFETY: `getppid` takes no argument, reads no memory of this process and cannot fail.
    return u32::try_from(unsafe { libc::getppid() }).expect("a parent number");
}
