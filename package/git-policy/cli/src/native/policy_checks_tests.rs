//! What:
//!  Each shipped policy's outcome from scripted facts,
//!  and which facts it asked for.
//! Why:
//!  A check that asks Git for nothing must start no process;
//!  a check whose fact could
//!      not be read must fail,
//!  not pass;
//!  and a content policy must never look clean where
//!      candidates exist but cannot be read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await checks.check('require-root', 'pre-forward')).toEqual({ kind: 'findings', findings: [] });
//! ```

/// The adapter under test,
///  the engine types it returns and the scripted facts.
use super::{ShippedChecks, shipped_checks};
use crate::candidate_prediction::CandidateRequest;
use crate::command_test_support::os_arguments;
use crate::diagnostics::EngineFailureCode;
use crate::policy_content::LifecycleContent;
use crate::policy_engine::{PolicyChecks, PolicyFinding, PolicyOutcome};
use crate::policy_registry::PolicyId;
use crate::policy_test_support::{
    ScriptedFacts, linked_worktree, main_worktree, outside_worktree, scripted_facts,
};
use crate::policy_trigger::Trigger;
use crate::rule_linked_worktree::{
    GuardedCommand, dirty_worktree_message, main_worktree_message, outside_worktree_message,
};
use crate::test_support::{fixture, remove};
use crate::worktree_identity::WorktreeIdentity;
use std::path::PathBuf;

/// The shipped checks over a command,
///  the given facts and no candidates.
fn checks(values: &[&str], facts: ScriptedFacts) -> ShippedChecks<ScriptedFacts> {
    return shipped_checks(
        facts,
        os_arguments(values),
        LifecycleContent::None,
        Vec::<PathBuf>::new(),
    );
}

/// Run one pre-forward check and return its outcome with the facts it asked for.
fn pre_forward(
    policy: PolicyId,
    values: &[&str],
    facts: ScriptedFacts,
) -> (PolicyOutcome, Vec<String>) {
    let mut shipped: ShippedChecks<ScriptedFacts> = checks(values, facts);
    let outcome: PolicyOutcome = shipped.check(policy, Trigger::PreForward);
    return (outcome, shipped.facts.asked);
}

/// The outcome "nothing found".
fn clean() -> PolicyOutcome {
    return PolicyOutcome::Findings(Vec::<PolicyFinding>::new());
}

/// The outcome "one finding with this code and message".
fn one(code: &'static str, message: &str) -> PolicyOutcome {
    return PolicyOutcome::Findings(vec![PolicyFinding {
        code,
        message: String::from(message),
        path: None,
        location: None,
        fix_available: false,
    }]);
}

/// Facts whose location is the given one.
fn located(location: crate::repository_location::RepositoryLocation) -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Ok(location);
    return facts;
}

/// Facts whose location could not be read.
fn unlocatable() -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Err(String::from("git could not be asked"));
    return facts;
}

/// The location is asked for once.
fn location_only() -> Vec<String> {
    return vec![String::from("location")];
}

/// The facts asked for, in order.
fn asked(names: &[&str]) -> Vec<String> {
    return names.iter().map(|name| { return String::from(*name); }).collect();
}

/// Facts that answer the dirtiness question as given, in the given location.
fn dirty_in(
    location: crate::repository_location::RepositoryLocation,
    dirty: bool,
) -> ScriptedFacts {
    let mut facts: ScriptedFacts = located(location);
    facts.dirty = Ok(dirty);
    return facts;
}

/// require-root:
///  exempt commands ask nothing;
///  other commands pass at the top level and outside a worktree.
#[test]
fn require_root_passes_exempt_commands_and_the_top_level() {
    for values in [
        vec![],
        vec!["--version"],
        vec!["help"],
        vec!["init"],
        vec!["clone", "url"],
        vec!["config", "--global", "user.name", "x"],
        vec!["config", "list"],
    ] {
        assert_eq!(
            pre_forward(PolicyId::RequireRoot, values.as_slice(), unlocatable()),
            (clean(), Vec::<String>::new()),
            "{values:?}"
        );
    }
    for location in [
        main_worktree("/r", ""),
        linked_worktree("/r", "/w"),
        outside_worktree(),
    ] {
        assert_eq!(
            pre_forward(PolicyId::RequireRoot, &["status"], located(location)),
            (clean(), location_only())
        );
    }
}

/// require-root:
///  below the top level the finding names the root and the directory Git reported.
#[test]
fn require_root_rejects_a_directory_below_the_top_level() {
    assert_eq!(
        pre_forward(
            PolicyId::RequireRoot,
            &["status"],
            located(main_worktree("/r", "sub/dir/"))
        ),
        (
            one(
                "not-at-root",
                "cli-git: not at the root of the git repository. Repo root is /r but effective cwd \
                 is /r/sub/dir. This rule exists because agents and humans frequently forget which directory they are in. Tip: cd to /r or pass -C /r before the subcommand."
            ),
            location_only()
        )
    );
    // A repository-local `git config` is not exempt.
    let (outcome, _) = pre_forward(
        PolicyId::RequireRoot,
        &["config", "user.name", "x"],
        located(main_worktree("/r", "a/")),
    );
    assert!(matches!(outcome, PolicyOutcome::Findings(found) if found.len() == 1));
    assert_eq!(
        pre_forward(PolicyId::RequireRoot, &["status"], unlocatable()),
        (
            PolicyOutcome::Failed {
                code: EngineFailureCode::ContentUnavailable,
                message: String::from("git could not be asked"),
                path: None,
            },
            location_only()
        )
    );
}

/// require-root as a direct check has no command to exempt:
///  it always measures.
#[test]
fn require_root_direct_check_always_measures() {
    for values in [vec![], vec!["-C", "dir"]] {
        let mut at_root: ShippedChecks<ScriptedFacts> =
            checks(values.as_slice(), located(main_worktree("/r", "")));
        assert_eq!(
            at_root.check(PolicyId::RequireRoot, Trigger::DirectCheck),
            clean()
        );
        assert_eq!(at_root.facts.asked, location_only());
        let mut below: ShippedChecks<ScriptedFacts> =
            checks(values.as_slice(), located(main_worktree("/r", "sub/")));
        assert!(matches!(
            below.check(PolicyId::RequireRoot, Trigger::DirectCheck),
            PolicyOutcome::Findings(found) if found.len() == 1 && found[0].code == "not-at-root"
        ));
    }
}

/// linked-worktree-only:
///  harmless commands ask nothing;
///  guarded ones are judged by the worktree kind.
#[test]
fn linked_worktree_judges_guarded_commands_by_worktree_kind() {
    for values in [
        vec!["status"],
        vec!["clean", "-n"],
        vec!["reset", "--soft"],
        vec!["stash", "list"],
        vec!["stash", "show"],
        vec!["restore", "--help"],
        vec!["rm", "a.txt"],
        vec!["checkout", "-b", "topic"],
        vec!["merge", "topic"],
    ] {
        assert_eq!(
            pre_forward(
                PolicyId::LinkedWorktreeOnly,
                values.as_slice(),
                unlocatable()
            ),
            (clean(), Vec::<String>::new()),
            "{values:?}"
        );
    }
    assert_eq!(
        pre_forward(
            PolicyId::LinkedWorktreeOnly,
            &["reset", "--hard"],
            located(main_worktree("/r", ""))
        ),
        (
            one(
                "linked-worktree-required",
                &main_worktree_message(GuardedCommand::ResetHard)
            ),
            location_only()
        )
    );
    assert_eq!(
        pre_forward(
            PolicyId::LinkedWorktreeOnly,
            &["stash"],
            located(outside_worktree())
        ),
        (
            one(
                "linked-worktree-required",
                &outside_worktree_message(GuardedCommand::Stash)
            ),
            location_only()
        )
    );
    assert_eq!(
        pre_forward(
            PolicyId::LinkedWorktreeOnly,
            &["clean", "-fd"],
            located(linked_worktree("/r", "/w"))
        ),
        (clean(), asked(&["location", "dirty:TrackedAndUntracked"]))
    );
    assert_eq!(
        pre_forward(
            PolicyId::LinkedWorktreeOnly,
            &["clean", "-fd"],
            unlocatable()
        ),
        (
            PolicyOutcome::Failed {
                code: EngineFailureCode::ContentUnavailable,
                message: String::from("git could not be asked"),
                path: None,
            },
            location_only()
        )
    );
}

/// linked-worktree-only:
///  a main worktree whose Git directory lies under an exempt tool cache passes.
#[test]
fn linked_worktree_exempts_an_allowed_tool_cache() {
    let root: PathBuf = fixture("checks-tool-cache");
    let cache: PathBuf = root.join("cache");
    let clone: PathBuf = cache.join("clone");
    std::fs::create_dir_all(clone.join(".git")).expect("cache clone");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Ok(crate::repository_location::RepositoryLocation {
        identity: WorktreeIdentity::MainWorktree {
            common_dir: clone.join(".git"),
            git_dir: clone.join(".git"),
            worktree_root: clone,
        },
        prefix: Vec::<u8>::new(),
    });
    let mut exempt: ShippedChecks<ScriptedFacts> = checks(&["reset", "--hard"], facts.clone());
    exempt.allowed_worktree_dirs = vec![cache];
    assert_eq!(
        exempt.check(PolicyId::LinkedWorktreeOnly, Trigger::PreForward),
        clean()
    );
    // The same repository without the exemption is an ordinary main worktree.
    let mut guarded: ShippedChecks<ScriptedFacts> = checks(&["reset", "--hard"], facts);
    assert_eq!(
        guarded.check(PolicyId::LinkedWorktreeOnly, Trigger::PreForward),
        one(
            "linked-worktree-required",
            &main_worktree_message(GuardedCommand::ResetHard)
        )
    );
    remove(root.as_path());
}

/// branch-worktree-only:
///  explicit creation is rejected without a query;
///  a bare name asks once.
#[test]
fn branch_worktree_rejects_creation_and_guessed_creation() {
    assert_eq!(
        pre_forward(PolicyId::BranchWorktreeOnly, &["status"], scripted_facts()),
        (clean(), Vec::<String>::new())
    );
    assert_eq!(
        pre_forward(
            PolicyId::BranchWorktreeOnly,
            &["switch", "-c", "topic"],
            scripted_facts()
        ),
        (
            one(
                "branch-creation-requires-worktree",
                "cli-git: git switch branch creation is rejected in the current worktree. Use \
                 `git worktree add -b <branch> <path> [<start-point>]` so new branch work starts \
                 in its own checkout, or pass --no-enforce-worktree-branch to bypass for this \
                 invocation."
            ),
            Vec::<String>::new()
        )
    );
    let asked: Vec<String> = vec![String::from("remote-guess:topic")];
    assert_eq!(
        pre_forward(
            PolicyId::BranchWorktreeOnly,
            &["switch", "topic"],
            scripted_facts()
        ),
        (clean(), asked.clone())
    );
    let mut guessing: ScriptedFacts = scripted_facts();
    guessing.remote_guess = Ok(true);
    assert_eq!(
        pre_forward(
            PolicyId::BranchWorktreeOnly,
            &["checkout", "topic"],
            guessing
        ),
        (
            one(
                "branch-creation-requires-worktree",
                "cli-git: git checkout for topic branch creation is rejected in the current \
                 worktree. Use `git worktree add -b <branch> <path> [<start-point>]` so new \
                 branch work starts in its own checkout, or pass --no-enforce-worktree-branch to \
                 bypass for this invocation."
            ),
            asked.clone()
        )
    );
    let mut failing: ScriptedFacts = scripted_facts();
    failing.remote_guess = Err(String::from("no git"));
    assert_eq!(
        pre_forward(PolicyId::BranchWorktreeOnly, &["switch", "topic"], failing),
        (
            PolicyOutcome::Failed {
                code: EngineFailureCode::ContentUnavailable,
                message: String::from("no git"),
                path: None,
            },
            asked
        )
    );
}

/// add-explicit:
///  decided from arguments alone.
#[test]
fn add_explicit_never_asks_for_a_fact() {
    assert_eq!(
        pre_forward(PolicyId::AddExplicit, &["add", "file"], unlocatable()),
        (clean(), Vec::<String>::new())
    );
    let (outcome, asked) = pre_forward(PolicyId::AddExplicit, &["add", "-A"], unlocatable());
    assert!(matches!(
        outcome,
        PolicyOutcome::Findings(found)
            if found.len() == 1 && found[0].code == "bulk-add-rejected" && found[0].message.contains("(-A)")
    ));
    assert_eq!(asked, Vec::<String>::new());
}

/// The candidates of `git add file`.
fn add_candidates() -> LifecycleContent {
    return LifecycleContent::Requested(CandidateRequest::Add(os_arguments(&["file"])));
}

/// The content-unavailable failure of candidates these scripted facts cannot prepare.
fn unprepared() -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code: EngineFailureCode::ContentUnavailable,
        message: String::from("the scripted facts prepare no candidates"),
        path: None,
    };
}

/// Content policies report nothing without candidates;
///  with candidates the ported ones
/// read them and the unported one refuses without reading.
///  The dependent-version policy
/// reads nothing for `git add`,
///  the one forwarded command with candidates,
///  as the installed
/// wrapper's policy plans only for a forwarded `commit`.
#[test]
fn content_policies_follow_the_lifecycle_content() {
    for policy in [
        PolicyId::FinalNewline,
        PolicyId::MarkdownAutofix,
        PolicyId::ForbiddenRootContext,
        PolicyId::DependentVersionBump,
        PolicyId::ForbiddenStrings,
    ] {
        for trigger in [
            Trigger::PreForward,
            Trigger::DirectCheck,
            Trigger::DirectFix,
        ] {
            let mut without: ShippedChecks<ScriptedFacts> = checks(&["status"], unlocatable());
            assert_eq!(without.check(policy, trigger), clean(), "{policy:?}");
            assert_eq!(without.facts.asked, Vec::<String>::new());
        }
    }
    let mut with: ShippedChecks<ScriptedFacts> = checks(&["add", "file"], unlocatable());
    with.candidates = add_candidates();
    assert_eq!(
        with.check(PolicyId::DependentVersionBump, Trigger::PreForward),
        clean()
    );
    assert_eq!(with.facts.asked, Vec::<String>::new());
    for policy in [
        PolicyId::FinalNewline,
        PolicyId::MarkdownAutofix,
        PolicyId::ForbiddenRootContext,
        PolicyId::ForbiddenStrings,
        PolicyId::DependentVersionBump,
    ] {
        for trigger in [
            Trigger::PreForward,
            Trigger::DirectCheck,
            Trigger::DirectFix,
        ] {
            if policy == PolicyId::DependentVersionBump && trigger == Trigger::PreForward {
                continue;
            }
            let mut read: ShippedChecks<ScriptedFacts> = checks(&["add", "file"], unlocatable());
            read.candidates = add_candidates();
            assert_eq!(read.check(policy, trigger), unprepared(), "{policy:?}");
            // A second content policy shares the one failed attempt and does not retry it.
            assert_eq!(read.check(policy, trigger), unprepared(), "{policy:?}");
            assert_eq!(read.facts.asked, vec![String::from("candidates")]);
        }
    }
    // A command policy is not a content policy: the lifecycle content does not touch it.
    let mut command: ShippedChecks<ScriptedFacts> = checks(&["add", "file"], scripted_facts());
    command.candidates = add_candidates();
    assert_eq!(
        command.check(PolicyId::AddExplicit, Trigger::PreForward),
        clean()
    );
    assert_eq!(command.facts.asked, Vec::<String>::new());
}

/// linked-worktree-only:
///  a form that overwrites is rejected in the main worktree without asking for the status, and in a linked worktree
///  only when the status shows uncommitted state.
#[test]
fn overwriting_forms_are_rejected_in_main_always_and_in_linked_when_dirty() {
    for (values, command, scope) in [
        (vec!["restore", "--staged", "a.txt"], GuardedCommand::RestoreStaged, "Tracked"),
        (vec!["checkout", "--", "a.txt"], GuardedCommand::CheckoutPaths, "Tracked"),
        (vec!["switch", "--discard-changes", "topic"], GuardedCommand::SwitchForce, "Tracked"),
        (vec!["reset"], GuardedCommand::ResetMixed, "Tracked"),
        (vec!["rm", "-rf", "dir"], GuardedCommand::Remove, "Tracked"),
        (vec!["merge", "--abort"], GuardedCommand::AbortMerge, "Tracked"),
        (vec!["clean", "-fd"], GuardedCommand::Clean, "TrackedAndUntracked"),
        (vec!["stash", "push"], GuardedCommand::Stash, "Tracked"),
    ] {
        let dirty_scope = format!("dirty:{scope}");
        for dirty in [false, true] {
            assert_eq!(
                pre_forward(
                    PolicyId::LinkedWorktreeOnly,
                    values.as_slice(),
                    dirty_in(main_worktree("/r", ""), dirty)
                ),
                (
                    one("linked-worktree-required", &main_worktree_message(command)),
                    location_only()
                ),
                "{values:?} main dirty={dirty}"
            );
        }
        assert_eq!(
            pre_forward(
                PolicyId::LinkedWorktreeOnly,
                values.as_slice(),
                dirty_in(linked_worktree("/r", "/w"), false)
            ),
            (clean(), asked(&["location", dirty_scope.as_str()])),
            "{values:?} linked clean"
        );
        assert_eq!(
            pre_forward(
                PolicyId::LinkedWorktreeOnly,
                values.as_slice(),
                dirty_in(linked_worktree("/r", "/w"), true)
            ),
            (
                one(
                    "linked-worktree-required",
                    &dirty_worktree_message(command)
                ),
                asked(&["location", dirty_scope.as_str()])
            ),
            "{values:?} linked dirty"
        );
    }
}

/// linked-worktree-only:
///  a form that only moves `HEAD` is rejected in any worktree only while the status shows uncommitted state, and
///  never outside a worktree.
#[test]
fn head_moves_are_rejected_only_while_dirty() {
    for values in [vec!["checkout", "topic"], vec!["switch", "topic"]] {
        let command: GuardedCommand = if values[0] == "checkout" {
            GuardedCommand::CheckoutMove
        } else {
            GuardedCommand::SwitchMove
        };
        for location in [main_worktree("/r", ""), linked_worktree("/r", "/w")] {
            assert_eq!(
                pre_forward(
                    PolicyId::LinkedWorktreeOnly,
                    values.as_slice(),
                    dirty_in(location.clone(), false)
                ),
                (clean(), asked(&["location", "dirty:Tracked"])),
                "{values:?} clean"
            );
            assert_eq!(
                pre_forward(
                    PolicyId::LinkedWorktreeOnly,
                    values.as_slice(),
                    dirty_in(location, true)
                ),
                (
                    one(
                        "linked-worktree-required",
                        &dirty_worktree_message(command)
                    ),
                    asked(&["location", "dirty:Tracked"])
                ),
                "{values:?} dirty"
            );
        }
        assert_eq!(
            pre_forward(
                PolicyId::LinkedWorktreeOnly,
                values.as_slice(),
                dirty_in(outside_worktree(), true)
            ),
            (clean(), location_only()),
            "{values:?} outside"
        );
    }
}

/// linked-worktree-only:
///  a status that cannot be read stops the command instead of reading as clean.
#[test]
fn an_unreadable_status_is_reported() {
    let mut facts: ScriptedFacts = located(linked_worktree("/r", "/w"));
    facts.dirty = Err(String::from("Git could not read the worktree status: x"));
    assert_eq!(
        pre_forward(PolicyId::LinkedWorktreeOnly, &["restore", "a.txt"], facts),
        (
            PolicyOutcome::Failed {
                code: EngineFailureCode::ContentUnavailable,
                message: String::from("Git could not read the worktree status: x"),
                path: None,
            },
            asked(&["location", "dirty:Tracked"])
        )
    );
}
