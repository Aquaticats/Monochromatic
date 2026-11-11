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
use crate::effective_target::{ToolCacheDirectory, default_allowed_worktree_dirs};
use crate::policy_test_support::{
    ScriptedFacts, linked_worktree, main_worktree, outside_worktree, scripted_facts,
};
use crate::policy_trigger::Trigger;
use crate::rule_add_explicit::StagedSelection;
use crate::rule_linked_worktree::{
    GuardedCommand, dirty_worktree_message, exemption_notice, main_worktree_message, outside_worktree_message,
    repository_stash_message,
};
/// Whether a command writes the remote settings every worktree shares,
///  and the text its refusal prints.
use crate::rule_shared_remote::{SHARED_REMOTE_SETTINGS_CODE, SharedRemoteWrite};
use crate::rule_shared_remote_messages::shared_remote_settings_message;
use crate::test_support::{fixture, remove};
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The shipped checks over a command,
///  the given facts and no candidates.
fn checks(values: &[&str], facts: ScriptedFacts) -> ShippedChecks<ScriptedFacts> {
    return shipped_checks(
        facts,
        os_arguments(values),
        LifecycleContent::None,
        Vec::<ToolCacheDirectory>::new(),
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

/// The facts asked for,
/// in order.
fn asked(names: &[&str]) -> Vec<String> {
    return names.iter().map(|name| { return String::from(*name); }).collect();
}

/// Facts that answer the dirtiness question as given,
/// in the given location.
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
                "monochromatic-git-policy-cli: not at the root of the git repository. Repo root is /r but effective cwd \
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
    exempt.allowed_worktree_dirs = vec![ToolCacheDirectory {
        path: cache,
        named_by: "MISE_DATA_DIR",
    }];
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

/// linked-worktree-only:
///  a checkout in a clone a build script made in its own output directory runs,
/// and the same clone anywhere else keeps its guard.
/// Why:
///  Cargo sets `OUT_DIR` for every build script and passes it to what it starts,
/// and a crate that vendors another project clones it there,
///  so the clone is the build step's work and the exemption names that one directory.
#[test]
fn linked_worktree_exempts_a_cargo_build_script_output_directory() {
    let root: PathBuf = fixture("checks-cargo-out-dir");
    let out_dir: PathBuf = root.join("cargo-build/aa/0123456789abcd/debug/build/libghostty-vt-sys-1/out");
    let clone: PathBuf = out_dir.join("ghostty-src");
    std::fs::create_dir_all(clone.join(".git")).expect("vendored clone");
    let worktree: &str = clone.to_str().expect("fixture paths are UTF-8");
    let facts: ScriptedFacts = dirty_in(main_worktree(worktree, ""), true);
    let allowed: Vec<ToolCacheDirectory> = default_allowed_worktree_dirs(
        &[(OsString::from("OUT_DIR"), out_dir.as_os_str().to_owned())],
        None,
    );
    let mut exempt: ShippedChecks<ScriptedFacts> = shipped_checks(
        facts.clone(),
        os_arguments(&["checkout", "bebca846"]),
        LifecycleContent::None,
        allowed.clone(),
    );
    assert_eq!(exempt.check(PolicyId::LinkedWorktreeOnly, Trigger::PreForward), clean());
    assert_eq!(exempt.facts.asked, location_only(), "an exempt repository needs no status");
    // A form that also writes files reaches the exemption's own branch, which names the variable that chose it.
    let mut writing: ShippedChecks<ScriptedFacts> = shipped_checks(
        facts.clone(),
        os_arguments(&["reset", "--hard"]),
        LifecycleContent::None,
        allowed,
    );
    assert_eq!(writing.check(PolicyId::LinkedWorktreeOnly, Trigger::PreForward), clean());
    assert_eq!(writing.notices, vec![exemption_notice("OUT_DIR", out_dir.as_path())]);
    // The same clone outside the build script's directory is still refused while it holds changes.
    let mut guarded: ShippedChecks<ScriptedFacts> = checks(&["checkout", "bebca846"], facts);
    assert_eq!(
        guarded.check(PolicyId::LinkedWorktreeOnly, Trigger::PreForward),
        one(
            "linked-worktree-required",
            &dirty_worktree_message(GuardedCommand::CheckoutMove)
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
                "monochromatic-git-policy-cli: git switch branch creation is rejected in the current worktree. Use \
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
                "monochromatic-git-policy-cli: git checkout for topic branch creation is rejected in the current \
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
///  an ordinary name and a bulk flag are decided from the arguments alone,
/// and a pathspec is judged by what Git says it selects.
#[test]
fn add_explicit_asks_for_a_selection_only_for_a_pathspec() {
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
    // The measurement is the one fact a pathspec needs, and it is logged with the caller's spelling.
    let (measured, measured_asked) = pre_forward(PolicyId::AddExplicit, &["add", "."], sweeping(4));
    assert!(matches!(
        measured,
        PolicyOutcome::Findings(ref found)
            if found.len() == 1 && found[0].code == "bulk-add-rejected" && found[0].message.contains("(.)")
    ));
    assert_eq!(measured_asked, vec![String::from("selection:.")]);
    let (magic, magic_asked) = pre_forward(PolicyId::AddExplicit, &["add", ":(top)"], sweeping(4));
    assert!(matches!(
        magic,
        PolicyOutcome::Findings(ref found)
            if found.len() == 1 && found[0].message.contains("It selects all 4 paths of this repository.")
    ));
    assert_eq!(magic_asked, vec![String::from("selection::(top)")]);
    // A selection that leaves a path out is no bulk staging.
    assert_eq!(
        pre_forward(PolicyId::AddExplicit, &["add", "."], selecting(3, 4)),
        (clean(), vec![String::from("selection:.")])
    );
}

/// Facts whose pathspec measurement found every path of a repository holding `count` paths.
fn sweeping(count: usize) -> ScriptedFacts {
    return selecting(count, count);
}

/// Facts whose pathspec measurement found `selected` of a repository holding `count` paths.
fn selecting(selected: usize, count: usize) -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.selection = Ok(StagedSelection {
        selected,
        repository: count,
        directory: count,
    });
    return facts;
}

/// A measurement that cannot be made lets the add through and says so once,
///  because a guard may never stop a command it could not judge.
#[test]
fn a_failed_selection_measurement_lets_the_add_through_with_one_notice() {
    let mut failing: ScriptedFacts = scripted_facts();
    failing.selection = Err(String::from("the listing did not answer"));
    let mut shipped: ShippedChecks<ScriptedFacts> = checks(&["add", "."], failing);
    assert_eq!(
        shipped.check(PolicyId::AddExplicit, Trigger::PreForward),
        clean()
    );
    assert_eq!(
        shipped.notices,
        vec![String::from(
            "monochromatic-git-policy-cli: could not check whether this git add stages the whole repository, \
             so the command runs unchanged: the listing did not answer\n"
        )]
    );
    // A bulk flag needs no measurement, so it is still refused and no line is said.
    let mut flag_facts: ScriptedFacts = scripted_facts();
    flag_facts.selection = Err(String::from("the listing did not answer"));
    let mut flag_checks: ShippedChecks<ScriptedFacts> = checks(&["add", "-A"], flag_facts);
    assert!(matches!(
        flag_checks.check(PolicyId::AddExplicit, Trigger::PreForward),
        PolicyOutcome::Findings(found) if found.len() == 1
    ));
    assert!(flag_checks.notices.is_empty());
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
///  a form that overwrites is rejected in the main worktree without asking for the status,
/// and in a linked worktree
///  only when the status shows uncommitted state.
#[test]
fn overwriting_forms_are_rejected_in_main_always_and_in_linked_when_dirty() {
    for (values, command, scope, tree) in [
        (vec!["restore", "--staged", "a.txt"], GuardedCommand::RestoreStaged, "Tracked", None),
        (vec!["checkout", "--", "a.txt"], GuardedCommand::CheckoutPaths, "Tracked", Some("HEAD")),
        (vec!["switch", "--discard-changes", "topic"], GuardedCommand::SwitchForce, "Tracked", Some("topic")),
        (vec!["reset"], GuardedCommand::ResetMixed, "Tracked", None),
        (vec!["rm", "-rf", "dir"], GuardedCommand::Remove, "Tracked", None),
        (vec!["merge", "--abort"], GuardedCommand::AbortMerge, "Tracked", None),
        (vec!["clean", "-fd"], GuardedCommand::Clean, "TrackedAndUntracked", None),
        (vec!["clean", "-fdx"], GuardedCommand::CleanIgnored, "Everything", None),
        (vec!["clean", "-fdX"], GuardedCommand::CleanIgnoredOnly, "IgnoredOnly", None),
        (vec!["stash", "push"], GuardedCommand::Stash, "Tracked", None),
    ] {
        let dirty_scope = format!("dirty:{scope}");
        // A form that writes a tree asks which tree once the tracked state is clean, and asks nothing while it is dirty.
        let tree_question = match tree {
            Some(revision) => format!("ignored-and-tracked:{revision}"),
            None => String::new(),
        };
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
        let mut clean_questions: Vec<&str> = vec!["location", dirty_scope.as_str()];
        if !tree_question.is_empty() {
            clean_questions.push(tree_question.as_str());
        }
        assert_eq!(
            pre_forward(
                PolicyId::LinkedWorktreeOnly,
                values.as_slice(),
                dirty_in(linked_worktree("/r", "/w"), false)
            ),
            (clean(), asked(clean_questions.as_slice())),
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
///  `git stash drop` and `git stash clear` are rejected in a linked worktree while the repository holds a stash entry,
/// because the entry belongs to the repository and not to the worktree the command runs in.
#[test]
fn a_stash_drop_asks_the_repository_not_the_worktree() {
    let command: GuardedCommand = GuardedCommand::StashDrop;
    for values in [vec!["stash", "drop"], vec!["stash", "clear"]] {
        // The worktree's own status is never asked, whatever it would say.
        let mut clean_worktree: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        clean_worktree.stash = Ok(true);
        assert_eq!(
            pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), clean_worktree),
            (
                one("linked-worktree-required", &repository_stash_message(command)),
                asked(&["location", "stash"])
            ),
            "{values:?} with a stash"
        );
        let mut no_stash: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), true);
        no_stash.stash = Ok(false);
        assert_eq!(
            pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), no_stash),
            (clean(), asked(&["location", "stash"])),
            "{values:?} without a stash"
        );
        // The main worktree is refused before either question.
        let mut main: ScriptedFacts = dirty_in(main_worktree("/r", ""), false);
        main.stash = Ok(false);
        assert_eq!(
            pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), main),
            (
                one("linked-worktree-required", &main_worktree_message(command)),
                location_only()
            ),
            "{values:?} in the main worktree"
        );
        // A question Git cannot answer stops the command instead of reading as "no stash".
        let mut unreadable: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        unreadable.stash = Err(String::from("Git could not read the repository stash: fatal"));
        assert_eq!(
            pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), unreadable),
            (
                PolicyOutcome::Failed {
                    code: EngineFailureCode::ContentUnavailable,
                    message: String::from("Git could not read the repository stash: fatal"),
                    path: None,
                },
                asked(&["location", "stash"])
            ),
            "{values:?} unreadable"
        );
    }
}

/// linked-worktree-only:
///  a form that only moves `HEAD` is rejected in any worktree only while the status shows uncommitted state,
/// and
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

/// linked-worktree-only:
///  a forced `checkout`,
///  a forced `switch` and a hard `reset` are rejected in a clean linked worktree
///  while the worktree ignores a path the tree they write also tracks,
///  and the line names the path it saw.
#[test]
fn a_tree_that_tracks_an_ignored_path_is_rejected_in_a_clean_linked_worktree() {
    for (values, revision) in [
        (vec!["checkout", "--force", "tracks"], "tracks"),
        (vec!["switch", "--force", "tracks"], "tracks"),
        (vec!["reset", "--hard", "tracks"], "tracks"),
        (vec!["reset", "--hard"], "HEAD"),
    ] {
        let mut shared: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        shared.ignored_and_tracked = Ok(vec![String::from(".env")]);
        let (outcome, questions): (PolicyOutcome, Vec<String>) = pre_forward(
            PolicyId::LinkedWorktreeOnly,
            values.as_slice(),
            shared,
        );
        let PolicyOutcome::Findings(findings) = outcome else {
            panic!("{values:?} was not rejected")
        };
        assert_eq!(findings.len(), 1, "{values:?}");
        assert_eq!(findings[0].code, "linked-worktree-required", "{values:?}");
        assert!(findings[0].message.contains("ignores .env"), "{values:?}: {}", findings[0].message);
        assert!(
            findings[0].message.contains(format!("the tree of {revision}").as_str()),
            "{values:?}: {}",
            findings[0].message
        );
        assert_eq!(
            questions,
            asked(&["location", "dirty:Tracked", format!("ignored-and-tracked:{revision}").as_str()]),
            "{values:?}"
        );
        // A tree Git cannot read is a failure, never a clean answer.
        let mut unreadable: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        unreadable.ignored_and_tracked = Err(String::from("Git could not read the tree of a revision: fatal"));
        assert!(
            matches!(
                pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), unreadable).0,
                PolicyOutcome::Failed { code: EngineFailureCode::ContentUnavailable, .. }
            ),
            "{values:?} unreadable"
        );
        // The same form with nothing shared runs, and the questions it asked are the whole cost.
        let mut nothing: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        nothing.ignored_and_tracked = Ok(Vec::<String>::new());
        assert_eq!(
            pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), nothing).0,
            clean(),
            "{values:?} nothing shared"
        );
        // A form that writes no tree asks nothing about one.
        let mut unasked: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        unasked.ignored_and_tracked = Err(String::from("this form writes no tree"));
        assert_eq!(
            pre_forward(PolicyId::LinkedWorktreeOnly, &["restore", "a.txt"], unasked).0,
            clean(),
            "a restore asks nothing about a tree"
        );
    }
}

/// linked-worktree-only:
///  a clean that carries its own exclusion patterns is measured by Git's own dry run of that clean,
///  because the status cannot see those patterns,
///  and a dry run Git cannot answer stops the command.
#[test]
fn a_clean_with_an_exclusion_asks_git_what_it_would_delete() {
    let command: GuardedCommand = GuardedCommand::CleanIgnoredOnly;
    for values in [
        vec!["clean", "-fdX", "--exclude=notes.txt"],
        vec!["clean", "--force", "-X", "-e", "notes.txt"],
    ] {
        let mut deleting: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        deleting.clean_deletes = Ok(true);
        let (outcome, questions): (PolicyOutcome, Vec<String>) =
            pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), deleting);
        assert_eq!(outcome, one("linked-worktree-required", &dirty_worktree_message(command)), "{values:?}");
        let question: &String = questions.last().expect("a last question");
        assert!(question.starts_with("clean-would-delete:"), "{values:?}: {question}");
        assert!(question.contains("-X"), "{values:?}: {question}");
        assert!(question.contains("notes.txt"), "{values:?}: {question}");
        // A dry run Git cannot answer is a failure, never "nothing to delete".
        let mut unreadable: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        unreadable.clean_deletes = Err(String::from("Git could not read what the clean would delete: fatal"));
        assert!(
            matches!(
                pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), unreadable).0,
                PolicyOutcome::Failed { code: EngineFailureCode::ContentUnavailable, .. }
            ),
            "{values:?} unreadable"
        );
        // A clean that would delete nothing runs.
        let mut empty: ScriptedFacts = dirty_in(linked_worktree("/r", "/w"), false);
        empty.clean_deletes = Ok(false);
        assert_eq!(
            pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), empty).0,
            clean(),
            "{values:?} nothing to delete"
        );
    }
}

/// linked-worktree-only:
///  a command that changes the remote settings every worktree shares is refused in a linked
///  worktree,
///  names the main checkout and the file the worktrees share,
///  and runs as before everywhere else.
#[test]
fn linked_worktree_refuses_the_shared_remote_settings_only_in_a_linked_worktree() {
    // The scripted repository's linked worktree is `/w` of the main checkout `/r`, whose shared file is `/r/.git/config`.
    let shared_file: &Path = Path::new("/r/.git/config");
    let main_checkout: &Path = Path::new("/r");
    for (values, written) in [
        (
            vec!["remote", "set-url", "origin", "https://example.invalid/two.git"],
            SharedRemoteWrite::RemoteSubcommand("set-url"),
        ),
        (
            vec!["config", "--add", "remote.origin.pushurl", "DISABLED"],
            SharedRemoteWrite::Key(String::from("remote.origin.pushurl")),
        ),
        (
            vec!["config", "remove-section", "remote.origin"],
            SharedRemoteWrite::Section(String::from("remote.origin")),
        ),
    ] {
        assert_eq!(
            pre_forward(
                PolicyId::LinkedWorktreeOnly,
                values.as_slice(),
                located(linked_worktree("/r", "/w"))
            ),
            (
                one(
                    SHARED_REMOTE_SETTINGS_CODE,
                    &shared_remote_settings_message(&written, shared_file, main_checkout)
                ),
                asked(&["location", "main-worktree"])
            ),
            "{values:?}"
        );
    }
    // The main checkout owns the file, and a place outside a worktree shares it with nothing.
    for location in [main_worktree("/r", ""), outside_worktree()] {
        assert_eq!(
            pre_forward(
                PolicyId::LinkedWorktreeOnly,
                &["remote", "set-url", "origin", "https://example.invalid/two.git"],
                located(location)
            ),
            (clean(), location_only())
        );
    }
    // A read and a per-worktree write are not the shared settings, so not even the location is asked.
    for values in [
        vec!["remote", "-v"],
        vec!["remote", "get-url", "origin"],
        vec!["config", "--get", "remote.origin.url"],
        vec!["config", "--worktree", "remote.origin.url", "own"],
    ] {
        assert_eq!(
            pre_forward(PolicyId::LinkedWorktreeOnly, values.as_slice(), unlocatable()),
            (clean(), Vec::<String>::new()),
            "{values:?}"
        );
    }
}
