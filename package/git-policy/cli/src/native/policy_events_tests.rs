//! What:
//!  Exact JSON Lines text of every event kind,
//!  its optional fields,
//!  escaping and numbering.
//! Why:
//!  Readers of the incumbent's events match field names and rely on one object per
//!      line;
//!  a reordered,
//!  missing or unescaped field breaks them without any error here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(renderPolicyEvent(0, finding)).toBe(JSON.stringify(expected) + '\n');
//! ```

/// The renderer,
///  its event types and the names it prints.
use super::{
    FindingEvent, FindingLocation, PolicyEvent, WARN_UNSAFE_CODE, event_blocks,
    render_policy_event, render_policy_events,
};
use crate::diagnostics::EngineFailureCode;
use crate::test_support::strip_hint_fields;

/// The rendering of one event without its additive `hint` field.
fn rendered_without_hint(sequence: u64, event: &PolicyEvent) -> String {
    return strip_hint_fields(render_policy_event(sequence, event).as_str());
}

/// A finding with a known hint carries it as its own field after the message, and the message stays the same.
#[test]
fn a_hinted_event_carries_the_hint_as_its_own_field() {
    let line: String = render_policy_event(
        0,
        &PolicyEvent::EngineFailure {
            code: EngineFailureCode::FixCycle,
            message: String::from("cycle"),
            trigger: None,
            policy: None,
            path: None,
        },
    );
    assert!(line.contains("\"message\":\"cycle\",\"hint\":\"Hint: "), "{line}");
    assert!(line.ends_with("\"}\n"), "{line}");
    let warning: String = render_policy_event(
        3,
        &PolicyEvent::WarnUnsafe { trigger: Trigger::PreForward, policy: PolicyId::AddExplicit },
    );
    assert!(warning.contains("\"hint\":\"Hint: if you set it to warn"), "{warning}");
    let unhinted: String = render_policy_event(
        0,
        &PolicyEvent::CoreFinding { core_id: "commit-only", code: "all-flag", message: String::from("m") },
    );
    assert!(!unhinted.contains("hint"), "{unhinted}");
}

/// The event form of a pathname.
use crate::event_path::EventPath;
use crate::policy_registry::{PolicyId, Severity};
use crate::policy_trigger::Trigger;

/// A finding with only its required fields.
fn finding(severity: Severity) -> FindingEvent {
    return FindingEvent {
        trigger: Trigger::PreForward,
        policy: PolicyId::RequireRoot,
        severity,
        code: "not-at-root",
        message: String::from("Not at root"),
        path: None,
        location: None,
        fix_available: false,
    };
}

/// A finding prints the incumbent's fields in the incumbent's order,
///  with the code prefixed by the policy name.
#[test]
fn finding_has_the_incumbent_shape() {
    assert_eq!(
        rendered_without_hint(0, &PolicyEvent::Finding(finding(Severity::Error))),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\
         \"policyId\":\"require-root\",\"severity\":\"error\",\"code\":\"require-root/not-at-root\",\
         \"message\":\"Not at root\",\"fix\":\"none\"}\n"
    );
    let full: FindingEvent = FindingEvent {
        trigger: Trigger::DirectCheck,
        policy: PolicyId::ForbiddenStrings,
        severity: Severity::Warn,
        code: "match",
        message: String::from("found"),
        path: Some(EventPath::from_git_bytes("dir/a.txt".as_bytes())),
        location: Some(FindingLocation {
            byte_start: 3,
            byte_end: 18_446_744_073_709_551_615,
        }),
        fix_available: true,
    };
    assert_eq!(
        rendered_without_hint(41, &PolicyEvent::Finding(full)),
        "{\"schemaVersion\":1,\"sequence\":41,\"type\":\"finding\",\"trigger\":\"direct-check\",\
         \"policyId\":\"security/forbidden-strings\",\"severity\":\"warn\",\
         \"code\":\"security/forbidden-strings/match\",\"message\":\"found\",\"path\":\"dir/a.txt\",\
         \"location\":{\"byteStart\":3,\"byteEnd\":18446744073709551615},\"fix\":\"available\"}\n"
    );
}

/// The unsafe-severity warning names its policy and trigger and carries the fixed code and message.
#[test]
fn warn_unsafe_has_the_incumbent_shape() {
    assert_eq!(WARN_UNSAFE_CODE, "warn-unsafe");
    assert_eq!(
        rendered_without_hint(
            2,
            &PolicyEvent::WarnUnsafe {
                trigger: Trigger::PreForward,
                policy: PolicyId::AddExplicit,
            }
        ),
        "{\"schemaVersion\":1,\"sequence\":2,\"type\":\"configuration-warning\",\
         \"trigger\":\"pre-forward\",\"policyId\":\"add-explicit\",\"code\":\"warn-unsafe\",\
         \"message\":\"Policy add-explicit is warn-unsafe but configured as warn.\"}\n"
    );
}

/// A fixed-transform rejection always carries the pre-forward trigger and a code prefixed by the transform.
#[test]
fn core_finding_has_the_incumbent_shape() {
    assert_eq!(
        rendered_without_hint(
            1,
            &PolicyEvent::CoreFinding {
                core_id: "commit-only",
                code: "all-flag",
                message: String::from("Name a path."),
            }
        ),
        "{\"schemaVersion\":1,\"sequence\":1,\"type\":\"core-finding\",\"trigger\":\"pre-forward\",\
         \"coreId\":\"commit-only\",\"code\":\"commit-only/all-flag\",\"message\":\"Name a path.\"}\n"
    );
}

/// An engine failure prints code and message,
///  then trigger,
///  policy and path only when present.
#[test]
fn engine_failure_prints_optional_fields_in_order() {
    assert_eq!(
        rendered_without_hint(
            0,
            &PolicyEvent::EngineFailure {
                code: EngineFailureCode::CoreIncomplete,
                message: String::from("transform failed"),
                trigger: None,
                policy: None,
                path: None,
            }
        ),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"core-incomplete\",\
         \"message\":\"transform failed\"}\n"
    );
    assert_eq!(
        rendered_without_hint(
            7,
            &PolicyEvent::EngineFailure {
                code: EngineFailureCode::ContentUnavailable,
                message: String::from("unreadable"),
                trigger: Some(Trigger::DirectFix),
                policy: Some(PolicyId::LinkedWorktreeOnly),
                path: Some(EventPath::from_git_bytes("a b".as_bytes())),
            }
        ),
        "{\"schemaVersion\":1,\"sequence\":7,\"type\":\"engine-failure\",\"code\":\"content-unavailable\",\
         \"message\":\"unreadable\",\"trigger\":\"direct-fix\",\"policyId\":\"linked-worktree-only\",\
         \"path\":\"a b\"}\n"
    );
    // Each optional field alone, so none depends on another being present.
    for (trigger, policy, path, tail) in [
        (
            Some(Trigger::ManualPush),
            None,
            None,
            ",\"trigger\":\"manual-push\"}\n",
        ),
        (
            None,
            Some(PolicyId::FinalNewline),
            None,
            ",\"policyId\":\"final-newline\"}\n",
        ),
        (
            None,
            None,
            Some(EventPath::from_git_bytes(b"p")),
            ",\"path\":\"p\"}\n",
        ),
    ] {
        let rendered: String = rendered_without_hint(
            0,
            &PolicyEvent::EngineFailure {
                code: EngineFailureCode::FixCycle,
                message: String::from("m"),
                trigger,
                policy,
                path,
            },
        );
        assert_eq!(
            rendered,
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"fix-cycle\",\"message\":\"m\"{tail}"
            )
        );
    }
}

/// A fix summary prints its pass count and its paths as a JSON array,
///  empty included.
#[test]
fn fix_summary_has_the_incumbent_shape() {
    assert_eq!(
        rendered_without_hint(
            3,
            &PolicyEvent::FixSummary {
                trigger: Trigger::DirectFix,
                passes: 2,
                changed_paths: vec![
                    EventPath::from_git_bytes(b"a.txt"),
                    EventPath::from_git_bytes(b"dir/b \"c\".md"),
                ],
            }
        ),
        "{\"schemaVersion\":1,\"sequence\":3,\"type\":\"fix-summary\",\"trigger\":\"direct-fix\",\
         \"passes\":2,\"changedPaths\":[\"a.txt\",\"dir/b \\\"c\\\".md\"]}\n"
    );
    assert_eq!(
        rendered_without_hint(
            0,
            &PolicyEvent::FixSummary {
                trigger: Trigger::PreForward,
                passes: 0,
                changed_paths: Vec::<EventPath>::new(),
            }
        ),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\
         \"passes\":0,\"changedPaths\":[]}\n"
    );
}

/// Text that tries to end its string or its line stays inside one JSON string on one line.
#[test]
fn hostile_text_cannot_leave_its_field() {
    let mut hostile: FindingEvent = finding(Severity::Error);
    hostile.message = String::from("a\"}\n{\"type\":\"x\"\\");
    hostile.path = Some(EventPath::from_git_bytes("p\n\"q\"\u{1}".as_bytes()));
    let rendered: String = rendered_without_hint(0, &PolicyEvent::Finding(hostile));
    assert_eq!(
        rendered,
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\
         \"policyId\":\"require-root\",\"severity\":\"error\",\"code\":\"require-root/not-at-root\",\
         \"message\":\"a\\\"}\\n{\\\"type\\\":\\\"x\\\"\\\\\",\"path\":\"p\\n\\\"q\\\"\\u0001\",\"fix\":\"none\"}\n"
    );
    assert_eq!(rendered.matches('\n').count(), 1);
}

/// A list is numbered consecutively from its first sequence;
///  an empty list prints nothing.
#[test]
fn lists_are_numbered_from_the_first_sequence() {
    assert_eq!(render_policy_events(5, &[]), "");
    let events: [PolicyEvent; 3] = [
        PolicyEvent::Finding(finding(Severity::Warn)),
        PolicyEvent::WarnUnsafe {
            trigger: Trigger::PreForward,
            policy: PolicyId::RequireRoot,
        },
        PolicyEvent::Finding(finding(Severity::Error)),
    ];
    let rendered: String = strip_hint_fields(render_policy_events(4, &events).as_str());
    let mut lines: Vec<&str> = Vec::<&str>::new();
    for line in rendered.lines() {
        lines.push(line);
    }
    assert_eq!(lines.len(), 3);
    assert!(rendered.ends_with('\n'));
    for (index, line) in lines.iter().enumerate() {
        assert!(
            line.starts_with(format!("{{\"schemaVersion\":1,\"sequence\":{},", index + 4).as_str()),
            "{line}"
        );
        assert_eq!(
            format!("{line}\n"),
            rendered_without_hint(index as u64 + 4, &events[index])
        );
    }
    assert_eq!(
        render_policy_events(0, &events[..1]),
        rendered_without_hint(0, &events[0])
    );
}

/// Only error findings and fixed-transform rejections block the command.
#[test]
fn only_errors_and_core_findings_block() {
    assert!(event_blocks(&PolicyEvent::Finding(finding(
        Severity::Error
    ))));
    assert!(!event_blocks(&PolicyEvent::Finding(finding(
        Severity::Warn
    ))));
    assert!(event_blocks(&PolicyEvent::CoreFinding {
        core_id: "commit-only",
        code: "all-flag",
        message: String::from("m"),
    }));
    assert!(!event_blocks(&PolicyEvent::WarnUnsafe {
        trigger: Trigger::PreForward,
        policy: PolicyId::AddExplicit,
    }));
    assert!(!event_blocks(&PolicyEvent::EngineFailure {
        code: EngineFailureCode::CoreIncomplete,
        message: String::from("m"),
        trigger: None,
        policy: None,
        path: None,
    }));
    assert!(!event_blocks(&PolicyEvent::FixSummary {
        trigger: Trigger::DirectFix,
        passes: 1,
        changed_paths: Vec::<EventPath>::new(),
    }));
}

/// A finding about a name that is not UTF-8 prints the readable `path` and then its exact
/// bytes as base64 `pathBytes`;
///  a UTF-8 name,
///  multi-byte characters included,
///  prints none.
#[test]
fn a_finding_about_a_name_that_is_not_utf8_adds_path_bytes() {
    let mut latin: FindingEvent = finding(Severity::Warn);
    latin.path = Some(EventPath::from_git_bytes(b"caf\xe9.txt"));
    assert_eq!(
        rendered_without_hint(0, &PolicyEvent::Finding(latin)),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\
         \"policyId\":\"require-root\",\"severity\":\"warn\",\"code\":\"require-root/not-at-root\",\
         \"message\":\"Not at root\",\"path\":\"caf\u{fffd}.txt\",\"pathBytes\":\"Y2Fm6S50eHQ=\",\
         \"fix\":\"none\"}\n"
    );
    let mut utf8: FindingEvent = finding(Severity::Warn);
    utf8.path = Some(EventPath::from_git_bytes("café.txt".as_bytes()));
    let rendered: String = rendered_without_hint(0, &PolicyEvent::Finding(utf8));
    assert!(
        rendered.contains(",\"path\":\"café.txt\",\"fix\""),
        "{rendered}"
    );
    assert!(!rendered.contains("pathBytes"), "{rendered}");
}

/// An engine failure about one file adds `pathBytes` after `path` the same way.
#[test]
fn an_engine_failure_about_a_name_that_is_not_utf8_adds_path_bytes() {
    assert_eq!(
        rendered_without_hint(
            1,
            &PolicyEvent::EngineFailure {
                code: EngineFailureCode::PolicyIncomplete,
                message: String::from("m"),
                trigger: Some(Trigger::DirectCheck),
                policy: Some(PolicyId::MarkdownAutofix),
                path: Some(EventPath::from_git_bytes(b"\xff\xfe.md")),
            }
        ),
        "{\"schemaVersion\":1,\"sequence\":1,\"type\":\"engine-failure\",\"code\":\"policy-incomplete\",\
         \"message\":\"m\",\"trigger\":\"direct-check\",\"policyId\":\"markdown/autofix\",\
         \"path\":\"\u{fffd}\u{fffd}.md\",\"pathBytes\":\"//4ubWQ=\"}\n"
    );
}

/// One rendered fix summary of one changed pass over `paths`.
fn render_summary(paths: Vec<EventPath>) -> String {
    return rendered_without_hint(
        0,
        &PolicyEvent::FixSummary {
            trigger: Trigger::DirectFix,
            passes: 1,
            changed_paths: paths,
        },
    );
}

/// A fix summary whose paths are all UTF-8 has no byte list;
///  one that holds a name that is
/// not UTF-8 lists the bytes of every path,
///  in the same order,
///  so indices line up.
#[test]
fn a_fix_summary_lists_every_path_s_bytes_once_one_is_not_utf8() {
    assert_eq!(
        render_summary(vec![
            EventPath::from_git_bytes(b"a.txt"),
            EventPath::from_git_bytes(b"caf\xe9.txt"),
            EventPath::from_git_bytes("é.md".as_bytes()),
        ]),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"direct-fix\",\
         \"passes\":1,\"changedPaths\":[\"a.txt\",\"caf\u{fffd}.txt\",\"é.md\"],\
         \"changedPathBytes\":[\"YS50eHQ=\",\"Y2Fm6S50eHQ=\",\"w6kubWQ=\"]}\n"
    );
    let exact: String = render_summary(vec![
        EventPath::from_git_bytes(b"a.txt"),
        EventPath::from_git_bytes("é.md".as_bytes()),
    ]);
    assert!(!exact.contains("changedPathBytes"), "{exact}");
    // The last path alone not being UTF-8 is enough.
    let last: String = render_summary(vec![
        EventPath::from_git_bytes(b"a.txt"),
        EventPath::from_git_bytes(b"z\xff"),
    ]);
    assert!(
        last.ends_with(",\"changedPathBytes\":[\"YS50eHQ=\",\"ev8=\"]}\n"),
        "{last}"
    );
}

/// The concurrent-commit events have the incumbent's field order and sequence numbering.
#[test]
fn concurrent_commit_events_have_the_incumbent_shape() {
    let events: Vec<PolicyEvent> = vec![
        PolicyEvent::LandingRaceLost {
            attempt: 2,
            winning_oid: String::from("abc"),
        },
        PolicyEvent::LandingReserved { lost_races: 2 },
        PolicyEvent::CommitReplayed {
            prepared_oid: String::from("p"),
            from_base: Some(String::from("b")),
            onto: String::from("o"),
            oid: String::from("n"),
        },
        PolicyEvent::CommitReplayed {
            prepared_oid: String::from("p"),
            from_base: None,
            onto: String::from("o"),
            oid: String::from("n"),
        },
        PolicyEvent::ReplayHeadersDropped {
            prepared_oid: String::from("p"),
            oid: String::from("n"),
            headers: vec![String::from("x-one")],
        },
        PolicyEvent::ReplayHeadersDropped {
            prepared_oid: String::from("p"),
            oid: String::from("n"),
            headers: vec![String::from("x-one"), String::from("x-two")],
        },
        PolicyEvent::ReplayConflict {
            target_ref: String::from("refs/heads/main"),
            paths: vec![EventPath::from_git_bytes(b"a.txt"), EventPath::from_git_bytes(b"b.txt")],
            winning_oid: String::from("w"),
            prepared_oid: String::from("p"),
        },
    ];
    assert_eq!(
        strip_hint_fields(render_policy_events(0, events.as_slice()).as_str()),
        concat!(
            "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"landing-race-lost\",\"attempt\":2,\"winningOid\":\"abc\"}\n",
            "{\"schemaVersion\":1,\"sequence\":1,\"type\":\"landing-reserved\",\"lostRaces\":2}\n",
            "{\"schemaVersion\":1,\"sequence\":2,\"type\":\"commit-replayed\",\"preparedOid\":\"p\",\"fromBase\":\"b\",\"onto\":\"o\",\"oid\":\"n\"}\n",
            "{\"schemaVersion\":1,\"sequence\":3,\"type\":\"commit-replayed\",\"preparedOid\":\"p\",\"onto\":\"o\",\"oid\":\"n\"}\n",
            "{\"schemaVersion\":1,\"sequence\":4,\"type\":\"replay-headers-dropped\",\"preparedOid\":\"p\",\"oid\":\"n\",\"headers\":[\"x-one\"],\"message\":\"Replaying signed commit p as n dropped its x-one header, because Git can re-sign only the headers git commit-tree writes.\"}\n",
            "{\"schemaVersion\":1,\"sequence\":5,\"type\":\"replay-headers-dropped\",\"preparedOid\":\"p\",\"oid\":\"n\",\"headers\":[\"x-one\",\"x-two\"],\"message\":\"Replaying signed commit p as n dropped its x-one, x-two headers, because Git can re-sign only the headers git commit-tree writes.\"}\n",
            "{\"schemaVersion\":1,\"sequence\":6,\"type\":\"core-finding\",\"trigger\":\"pre-forward\",\"coreId\":\"concurrent-commit\",\"code\":\"concurrent-commit/replay-conflict\",\"message\":\"refs/heads/main moved while this commit was prepared, and replaying it conflicts with w in a.txt, b.txt; nothing landed. The prepared commit p stays in the object store until gc expires it: resolve with git cherry-pick p.\",\"paths\":[\"a.txt\",\"b.txt\"],\"winningOid\":\"w\",\"preparedOid\":\"p\"}\n",
        )
    );
    // Only the conflict blocks the command.
    assert!(event_blocks(&events[6]));
    for event in &events[..6] {
        assert!(!event_blocks(event), "{event:?}");
    }
}

/// A replay conflict that names a file whose name is not UTF-8 lists every readable name in `paths`,
///  then the exact bytes of every path in `pathsBytes`,
///  and a conflict over UTF-8 names only has no such list.
#[test]
fn a_replay_conflict_over_a_name_that_is_not_utf8_adds_paths_bytes() {
    let conflict = |paths: Vec<EventPath>| -> PolicyEvent {
        return PolicyEvent::ReplayConflict {
            target_ref: String::from("refs/heads/main"),
            paths,
            winning_oid: String::from("w"),
            prepared_oid: String::from("p"),
        };
    };
    assert_eq!(
        rendered_without_hint(
            0,
            &conflict(vec![EventPath::from_git_bytes(b"a.txt"), EventPath::from_git_bytes(b"caf\xe9.txt")])
        ),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"core-finding\",\"trigger\":\"pre-forward\",\"coreId\":\"concurrent-commit\",\"code\":\"concurrent-commit/replay-conflict\",\"message\":\"refs/heads/main moved while this commit was prepared, and replaying it conflicts with w in a.txt, caf\u{fffd}.txt; nothing landed. The prepared commit p stays in the object store until gc expires it: resolve with git cherry-pick p.\",\"paths\":[\"a.txt\",\"caf\u{fffd}.txt\"],\"pathsBytes\":[\"YS50eHQ=\",\"Y2Fm6S50eHQ=\"],\"winningOid\":\"w\",\"preparedOid\":\"p\"}\n"
    );
    let utf8: String = rendered_without_hint(0, &conflict(vec![EventPath::from_git_bytes("café.txt".as_bytes())]));
    assert!(utf8.contains(",\"paths\":[\"café.txt\"],\"winningOid\""), "{utf8}");
    assert!(!utf8.contains("Bytes"), "{utf8}");
}
