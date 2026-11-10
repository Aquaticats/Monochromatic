//! What:
//!  Controls for the dependent-version check over real candidates:
//!  findings on every
//!       lifecycle,
//!  corrections on a direct fix,
//!  files a narrow fix did not select,
//!  and
//!       failures by cause.
//! Why:
//!  These are the decisions the installed wrapper's policy and its direct fix make;
//!      each control mirrors one of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await findDependentBumps(context)).toEqual([{ code: 'dependent-version-stale', ... }]);
//! ```
#![cfg(unix)]

/// Import the check under test.
use super::DependentCheck;
/// The modes a correction keeps.
use crate::candidate_object::CandidateMode;
/// What a lifecycle asks the candidate layer for.
use crate::candidate_prediction::CandidateRequest;
/// The engine's failure codes.
use crate::diagnostics::EngineFailureCode;
/// The shared candidates,
///  corrections and lifecycle content.
use crate::policy_content::{ContentState, Correction, LifecycleContent};
/// The finding and outcome types of a check.
use crate::event_path::EventPath;
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
/// Scripted facts that prepare candidates with real Git.
use crate::policy_test_support::{ScriptedFacts, main_worktree, scripted_facts};
/// The lifecycle points.
use crate::policy_trigger::Trigger;
/// Shared fixture directories and real Git.
use crate::test_support::{fixture, git, remove, repository};
/// The registry configuration the generator writes,
///  and its repository path.
use crate::dependent_version_test_support::{CONFIG_PATH, config_text};
/// Owned operating-system text.
use std::ffi::OsString;
/// Unix permission bits.
use std::os::unix::fs::PermissionsExt;
/// Borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// Shared byte handles.
use std::rc::Rc;

/// The dependencies block of a manifest that names one workspace package.
fn dependency_block(depended: &str) -> String {
    return format!(",\n  \"dependencies\": {{\n    \"{depended}\": \"workspace:*\"\n  }}");
}

/// The check as a direct command runs it:
///  no forwarded commit and no real index.
/// Why:
///  Every control here but the commit path's own is a direct command,
///  and one wrapper
///       keeps that fact out of each call.
fn check_dependent_version(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    facts: &mut ScriptedFacts,
    trigger: Trigger,
) -> PolicyOutcome {
    return super::check_dependent_version(DependentCheck {
        content,
        lifecycle,
        facts,
        trigger,
        forwards_commit: false,
        real_index: None,
    });
}

/// A manifest as two-space JSON with a final newline.
fn manifest(name: &str, version: &str, dependency: Option<&str>) -> String {
    let dependencies: String = dependency.map_or_else(String::new, dependency_block);
    return format!(
        "{{\n  \"name\": \"{name}\",\n  \"version\": \"{version}\"{dependencies}\n}}\n"
    );
}

/// Write a file below the repository,
///  creating its directory.
fn write(repo: &Path, path: &str, text: &str) {
    let file: PathBuf = repo.join(path);
    std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
    std::fs::write(file, text).expect("file");
}

/// The manifest path of package `name` below `package/module/`.
fn path_of(name: &str) -> String {
    return format!("package/module/{name}/package.json");
}

/// A committed workspace of `@s/a`,
///  its dependents `@s/b` and `@s/c`,
///  and the registry
/// configuration;
///  then `@s/a` raised to `1.1.0` in the worktree.
fn workspace(root: &Path) -> PathBuf {
    return workspace_with(root, Some(&config_text(&["@s/a", "@s/b", "@s/c"])));
}

/// The same workspace,
///  with the registry configuration text the case names.
fn workspace_with(root: &Path, registry: Option<&str>) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    write(
        repo.as_path(),
        &path_of("a"),
        &manifest("@s/a", "1.0.0", None),
    );
    write(
        repo.as_path(),
        &path_of("b"),
        &manifest("@s/b", "2.0.0", Some("@s/a")),
    );
    write(
        repo.as_path(),
        &path_of("c"),
        &manifest("@s/c", "3.0.0", Some("@s/a")),
    );
    if let Some(text) = registry {
        write(repo.as_path(), CONFIG_PATH, text);
    }
    git(repo.as_path(), &["add", "--all"]);
    git(
        repo.as_path(),
        &["commit", "--quiet", "--message=workspace"],
    );
    write(
        repo.as_path(),
        &path_of("a"),
        &manifest("@s/a", "1.1.0", None),
    );
    return repo;
}

/// Scripted facts that prepare candidates in `repo` and locate its main worktree.
fn facts_in(repo: &Path) -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.to_path_buf());
    facts.location = Ok(main_worktree(repo.to_str().expect("text path"), ""));
    return facts;
}

/// The lifecycle content of a direct command over `pathspecs`.
fn direct(pathspecs: &[&str]) -> LifecycleContent {
    return LifecycleContent::Requested(CandidateRequest::Direct(
        pathspecs.iter().map(OsString::from).collect(),
    ));
}

/// The stale finding about dependent `name` moving from `from` to `to`.
fn stale(name: &str, from: &str, to: &str) -> PolicyFinding {
    return PolicyFinding {
        code: "dependent-version-stale",
        message: format!(
            "@s/{name} reaches a package bumped in this commit (@s/a); bump it from {from} to {to} in the same commit."
        ),
        path: Some(EventPath::display(&path_of(name))),
        location: None,
        fix_available: true,
    };
}

/// The correction of dependent `name` from `from` to `to`.
fn bump(name: &str, from: &str, to: &str) -> Correction {
    return Correction {
        path: path_of(name).into_bytes(),
        mode: CandidateMode::Regular,
        before: Rc::from(manifest(&format!("@s/{name}"), from, Some("@s/a")).as_bytes()),
        after: Rc::from(manifest(&format!("@s/{name}"), to, Some("@s/a")).as_bytes()),
    };
}

/// A direct check reports each dependent with a fix available and proposes nothing;
///  a
/// direct fix proposes each correction;
///  the next pass reads them back and is clean.
#[test]
fn direct_check_reports_and_direct_fix_corrects_every_dependent() {
    let root: PathBuf = fixture("dependent-check-fix");
    let repo: PathBuf = workspace(root.as_path());
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let lifecycle: LifecycleContent = direct(&[":/"]);
    let mut content: ContentState = ContentState::new();
    let findings: PolicyOutcome = PolicyOutcome::Findings(vec![
        stale("b", "2.0.0", "2.0.1"),
        stale("c", "3.0.0", "3.0.1"),
    ]);
    assert_eq!(
        check_dependent_version(&mut content, &lifecycle, &mut facts, Trigger::DirectCheck),
        findings
    );
    assert_eq!(content.take_proposals(), Vec::<Correction>::new());
    assert_eq!(
        check_dependent_version(&mut content, &lifecycle, &mut facts, Trigger::DirectFix),
        findings
    );
    let proposals: Vec<Correction> = content.take_proposals();
    assert_eq!(
        proposals,
        vec![bump("b", "2.0.0", "2.0.1"), bump("c", "3.0.0", "3.0.1")]
    );
    content.apply(proposals);
    assert_eq!(
        check_dependent_version(&mut content, &lifecycle, &mut facts, Trigger::DirectFix),
        PolicyOutcome::Findings(Vec::new())
    );
    assert_eq!(content.take_proposals(), Vec::<Correction>::new());
    remove(root.as_path());
}

/// `git add` reads nothing;
///  a scope without a workspace manifest plans nothing.
#[test]
fn pre_forward_and_scopes_without_manifests_plan_nothing() {
    let root: PathBuf = fixture("dependent-nothing");
    let repo: PathBuf = workspace(root.as_path());
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let mut content: ContentState = ContentState::new();
    let add: LifecycleContent =
        LifecycleContent::Requested(CandidateRequest::Add(vec![OsString::from("--all")]));
    assert_eq!(
        check_dependent_version(&mut content, &add, &mut facts, Trigger::PreForward),
        PolicyOutcome::Findings(Vec::new())
    );
    assert_eq!(facts.asked, Vec::<String>::new());
    let mut scoped: ContentState = ContentState::new();
    let config: LifecycleContent = direct(&["package/config"]);
    assert_eq!(
        check_dependent_version(&mut scoped, &config, &mut facts, Trigger::DirectCheck),
        PolicyOutcome::Findings(Vec::new())
    );
    let mut none: ContentState = ContentState::new();
    assert_eq!(
        check_dependent_version(
            &mut none,
            &LifecycleContent::None,
            &mut facts,
            Trigger::DirectCheck
        ),
        PolicyOutcome::Findings(Vec::new())
    );
    remove(root.as_path());
}

/// A narrow fix corrects a clean dependent it did not select from `HEAD`'s bytes.
#[test]
fn a_narrow_fix_admits_clean_unselected_dependents() {
    let root: PathBuf = fixture("dependent-admitted");
    let repo: PathBuf = workspace(root.as_path());
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let mut content: ContentState = ContentState::new();
    let narrow: LifecycleContent = direct(&[path_of("a").as_str()]);
    assert_eq!(
        check_dependent_version(&mut content, &narrow, &mut facts, Trigger::DirectFix),
        PolicyOutcome::Findings(vec![
            stale("b", "2.0.0", "2.0.1"),
            stale("c", "3.0.0", "3.0.1")
        ])
    );
    assert_eq!(
        content.take_proposals(),
        vec![bump("b", "2.0.0", "2.0.1"), bump("c", "3.0.0", "3.0.1")]
    );
    remove(root.as_path());
}

/// The conflict a narrow fix reports after `change` alters dependent `@s/b`'s copies.
fn narrow_conflict(name: &str, change: fn(&Path)) -> PolicyOutcome {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = workspace(root.as_path());
    change(repo.as_path());
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let mut content: ContentState = ContentState::new();
    let narrow: LifecycleContent = direct(&[path_of("a").as_str()]);
    let outcome: PolicyOutcome =
        check_dependent_version(&mut content, &narrow, &mut facts, Trigger::DirectFix);
    assert_eq!(content.take_proposals(), Vec::<Correction>::new(), "{name}");
    remove(root.as_path());
    return outcome;
}

/// Make `@s/b`'s worktree copy differ from `HEAD`.
fn unstaged_edit(repo: &Path) {
    write(
        repo,
        &path_of("b"),
        &format!("{} ", manifest("@s/b", "2.0.0", Some("@s/a"))),
    );
}

/// Stage a different `@s/b`.
fn staged_edit(repo: &Path) {
    unstaged_edit(repo);
    git(repo, &["add", path_of("b").as_str()]);
}

/// Remove `@s/b`'s worktree copy,
///  keeping its index entry.
fn missing_copy(repo: &Path) {
    std::fs::remove_file(repo.join(path_of("b"))).expect("remove");
}

/// Stage `@s/b` as executable with the same bytes.
fn staged_mode(repo: &Path) {
    let file: PathBuf = repo.join(path_of("b"));
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    git(repo, &["add", path_of("b").as_str()]);
}

/// The `patch-conflict` outcome of dependent `b` for one reason.
fn b_patch_conflict(reason: &str) -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code: EngineFailureCode::PatchConflict,
        message: crate::policy_added_path::added_path_message(&path_of("b"), reason),
        path: None,
    };
}

/// Each way an unselected dependent can differ from `HEAD` is a `patch-conflict` naming it.
#[test]
fn a_narrow_fix_refuses_unselected_dependents_that_differ_from_head() {
    assert_eq!(
        narrow_conflict("dependent-unstaged", unstaged_edit),
        b_patch_conflict("its worktree copy has unstaged changes")
    );
    assert_eq!(
        narrow_conflict("dependent-staged", staged_edit),
        b_patch_conflict("HEAD does not hold it as the ordinary file the fix was computed against")
    );
    assert_eq!(
        narrow_conflict("dependent-missing", missing_copy),
        b_patch_conflict("its worktree copy is missing")
    );
    assert_eq!(
        narrow_conflict("dependent-mode", staged_mode),
        b_patch_conflict("it has staged changes")
    );
}

/// The outcome of one check of `trigger` over `lifecycle` after `change`,
///  and the
/// proposals it left.
fn checked(
    name: &str,
    change: fn(&Path),
    lifecycle: &LifecycleContent,
    trigger: Trigger,
) -> (PolicyOutcome, Vec<Correction>) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = workspace(root.as_path());
    change(repo.as_path());
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let mut content: ContentState = ContentState::new();
    let outcome: PolicyOutcome =
        check_dependent_version(&mut content, lifecycle, &mut facts, trigger);
    let proposals: Vec<Correction> = content.take_proposals();
    remove(root.as_path());
    return (outcome, proposals);
}

/// Break `@s/b`'s manifest in the worktree.
fn broken_manifest(repo: &Path) {
    write(repo, &path_of("b"), "{ \"name\": \"@s/b\",\n");
}

/// Give `@s/b` a version the patch bump does not support,
///  committed alone,
///  so it is not raised itself.
fn prerelease_dependent(repo: &Path) {
    write(
        repo,
        &path_of("b"),
        &manifest("@s/b", "2.0.0-rc.1", Some("@s/a")),
    );
    git(repo, &["add", path_of("b").as_str()]);
    git(
        repo,
        &["commit", "--quiet", "--message=prerelease", "--", path_of("b").as_str()],
    );
}

/// Make `@s/b`'s worktree copy unreadable.
fn unreadable_copy(repo: &Path) {
    let file: PathBuf = repo.join(path_of("b"));
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o200)).expect("chmod");
}

/// Leave an unmerged entry for another file in the index.
fn conflict_elsewhere(repo: &Path) {
    write(repo, "x.txt", "base\n");
    git(repo, &["add", "x.txt"]);
    git(repo, &["commit", "--quiet", "--message=x"]);
    git(repo, &["checkout", "--quiet", "-b", "side"]);
    write(repo, "x.txt", "side\n");
    git(repo, &["commit", "--quiet", "--all", "--message=side"]);
    git(repo, &["checkout", "--quiet", "main"]);
    write(repo, "x.txt", "main\n");
    git(
        repo,
        &["commit", "--quiet", "--message=main", "--", "x.txt"],
    );
    let _ = crate::test_support::git_output(repo, &["merge", "--quiet", "side"]);
}

/// Content the planner cannot use,
///  content it cannot read,
///  and an unsupported dependent
/// each end the check with their own outcome.
#[test]
fn failures_follow_their_cause() {
    let all: LifecycleContent = direct(&[":/"]);
    let narrow: LifecycleContent = direct(&[path_of("a").as_str()]);
    let (broken, _) = checked(
        "dependent-broken",
        broken_manifest,
        &all,
        Trigger::DirectCheck,
    );
    let PolicyOutcome::Failed { code, message, .. } = broken else {
        panic!("expected a failure, got {broken:?}");
    };
    assert_eq!(code, EngineFailureCode::PolicyIncomplete);
    assert!(
        message.contains("package/module/b/package.json is not JSON"),
        "{message}"
    );
    let (unsupported, proposals) = checked(
        "dependent-unsupported",
        prerelease_dependent,
        &all,
        Trigger::DirectFix,
    );
    let PolicyOutcome::Findings(findings) = unsupported else {
        panic!("expected findings, got {unsupported:?}");
    };
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "dependent-version-unsupported");
    assert_eq!(findings[0].path, None);
    assert!(!findings[0].fix_available);
    assert_eq!(proposals, Vec::<Correction>::new());
    let (conflicted, _) = checked(
        "dependent-unmerged",
        conflict_elsewhere,
        &narrow,
        Trigger::DirectCheck,
    );
    let PolicyOutcome::Failed {
        code: unmerged_code,
        message: unmerged_message,
        ..
    } = conflicted
    else {
        panic!("expected a failure, got {conflicted:?}");
    };
    assert_eq!(unmerged_code, EngineFailureCode::ContentUnavailable);
    assert!(
        unmerged_message.contains("could not read x.txt"),
        "{unmerged_message}"
    );
    let (unreadable, _) = checked(
        "dependent-unreadable",
        unreadable_copy,
        &narrow,
        Trigger::DirectFix,
    );
    if !matches!(&unreadable, PolicyOutcome::Findings(_)) {
        assert_eq!(
            unreadable,
            PolicyOutcome::Failed {
                code: EngineFailureCode::ContentUnavailable,
                message: String::from(
                    "mono/dependent-version-bump could not read the worktree copy of \
                     package/module/b/package.json: Permission denied (os error 13)."
                ),
                path: None,
            }
        );
    }
}

/// Remove the registry configuration and commit the removal alone,
/// so the raised manifest
///  stays in the worktree.
fn no_registry_configuration(repo: &Path) {
    git(repo, &["rm", "--quiet", CONFIG_PATH]);
    git(
        repo,
        &["commit", "--quiet", "--message=no-registry", "--", CONFIG_PATH],
    );
}

/// Replace the publish set with an empty list and commit that alone.
fn empty_publish_set(repo: &Path) {
    write(repo, CONFIG_PATH, &config_text(&[]));
    git(repo, &["add", "--", CONFIG_PATH]);
    git(
        repo,
        &["commit", "--quiet", "--message=empty-publish-set", "--", CONFIG_PATH],
    );
}

/// A publish set that is missing or empty is not a clean result:
///  the check fails
///  with code `policy-incomplete`,
///  says the ripple could not be judged,
///  and proposes
///  nothing.
#[test]
fn a_missing_or_empty_publish_set_is_not_clean() {
    let cases: [(&str, fn(&Path), &str); 2] = [
        (
            "dependent-no-registry",
            no_registry_configuration,
            "the publish set of package/config/pnpr/config.yaml cannot be read \
             (it is not tracked in this repository), \
             so whether a dependent is published cannot be judged",
        ),
        (
            "dependent-empty-publish-set",
            empty_publish_set,
            "the publish set of package/config/pnpr/config.yaml cannot be read \
             (no `auth.oidc` workload lists a package name), \
             so whether a dependent is published cannot be judged",
        ),
    ];
    for (name, change, cause) in cases {
        let (outcome, proposals) = checked(name, change, &direct(&[":/"]), Trigger::DirectCheck);
        let PolicyOutcome::Failed {
            code,
            message,
            path,
        } = outcome
        else {
            panic!("expected a failure for {name}, got {outcome:?}");
        };
        assert_eq!(code, EngineFailureCode::PolicyIncomplete, "{name}");
        assert_eq!(path, None, "{name}");
        assert!(message.contains(cause), "{name}: {message}");
        assert!(
            message.ends_with(
                "Correct the file and retry, or pass \
                 --no-enforce-mono/dependent-version-bump to skip this policy for one command."
            ),
            "{name}: {message}"
        );
        assert_eq!(proposals, Vec::<Correction>::new(), "{name}");
    }
}

/// A direct fix that cannot locate the worktree reports the location failure.
#[test]
fn a_fix_without_a_location_reports_it() {
    let root: PathBuf = fixture("dependent-unlocated");
    let repo: PathBuf = workspace(root.as_path());
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    facts.location = Err(String::from("monochromatic-git-policy-cli could not locate the repository"));
    let mut content: ContentState = ContentState::new();
    assert_eq!(
        check_dependent_version(
            &mut content,
            &direct(&[":/"]),
            &mut facts,
            Trigger::DirectFix
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::ContentUnavailable,
            message: String::from("monochromatic-git-policy-cli could not locate the repository"),
            path: None,
        }
    );
    remove(root.as_path());
}

/// The lifecycle content of a commit that names `paths`,
/// and the real index it read them from.
fn commit_naming(repo: &Path, paths: &[&str]) -> (LifecycleContent, PathBuf) {
    let head: String = String::from_utf8_lossy(
        crate::test_support::git(repo, &["rev-parse", "HEAD"])
            .stdout
            .as_slice(),
    )
    .trim()
    .to_string();
    let base = crate::candidate_object::parse_object_id(head.as_bytes()).expect("a commit name");
    let pathspecs: Vec<OsString> = paths
        .iter()
        .map(|path: &&str| return OsString::from(format!(":(top,literal){path}")))
        .collect();
    return (
        LifecycleContent::Requested(CandidateRequest::StagedAtBase {
            base: Some(base),
            pathspecs,
        }),
        repo.join(".git/index"),
    );
}

/// One commit-path check after `change`,
/// its outcome and the corrections it proposed.
fn committed(
    name: &str,
    change: fn(&Path),
    paths: &[&str],
) -> (PolicyOutcome, Vec<Correction>) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = workspace(root.as_path());
    let (lifecycle, real_index): (LifecycleContent, PathBuf) =
        commit_naming(repo.as_path(), paths);
    // The commit's selection: the raised manifest, which every control below names.
    git(repo.as_path(), &["add", "--", path_of("a").as_str()]);
    change(repo.as_path());
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let mut content: ContentState = ContentState::new();
    let outcome: PolicyOutcome = super::check_dependent_version(DependentCheck {
        content: &mut content,
        lifecycle: &lifecycle,
        facts: &mut facts,
        trigger: Trigger::PreForward,
        forwards_commit: true,
        real_index: Some(real_index.as_path()),
    });
    let proposals: Vec<Correction> = content.take_proposals();
    remove(root.as_path());
    return (outcome, proposals);
}

/// Leave every copy as `HEAD` holds it but the raised manifest the commit names.
fn nothing(_repo: &Path) {}

/// A commit proposes one bump per dependent while every copy of its manifest holds the file
/// the bump was computed against,
/// so the hand bump and its ripple land together.
#[test]
fn a_commit_proposes_every_clean_dependent() {
    let (outcome, proposals) = committed(
        "dependent-commit-clean",
        nothing,
        &[path_of("a").as_str()],
    );
    assert_eq!(
        outcome,
        PolicyOutcome::Findings(vec![
            stale("b", "2.0.0", "2.0.1"),
            stale("c", "3.0.0", "3.0.1"),
        ])
    );
    assert_eq!(
        proposals,
        vec![bump("b", "2.0.0", "2.0.1"), bump("c", "3.0.0", "3.0.1")]
    );
}

/// A commit leaves out only the dependent whose copies hold something the commit does not
/// carry:
/// it proposes the rest,
/// names the file it left alone,
/// and states the reason,
/// so nothing is overwritten and the commit still lands.
#[test]
fn a_commit_names_the_dependent_it_leaves_behind() {
    let (outcome, proposals) = committed(
        "dependent-commit-kept",
        unstaged_edit,
        &[path_of("a").as_str()],
    );
    let PolicyOutcome::FindingsWithNotes { findings, notes } = outcome else {
        panic!("a note is expected beside the findings, got {outcome:?}");
    };
    assert_eq!(findings, vec![stale("c", "3.0.0", "3.0.1")]);
    assert_eq!(proposals, vec![bump("c", "3.0.0", "3.0.1")]);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].path, Some(EventPath::display(&path_of("b"))));
    assert_eq!(
        notes[0].message,
        crate::policy_added_path::commit_added_path_message(
            &path_of("b"),
            "@s/b",
            "2.0.0",
            "2.0.1",
            "its worktree copy has unstaged changes"
        )
    );
}

/// A ripple the planner cannot judge costs the commit its report and never its landing:
/// the reason is a note,
/// no finding and no correction,
/// and the pass goes on.
#[test]
fn a_commit_reports_what_it_cannot_judge_instead_of_failing() {
    for (name, change, cause) in [
        (
            "dependent-commit-no-registry",
            no_registry_configuration as fn(&Path),
            "the publish set of package/config/pnpr/config.yaml cannot be read",
        ),
        (
            "dependent-commit-empty-publish-set",
            empty_publish_set as fn(&Path),
            "the publish set of package/config/pnpr/config.yaml cannot be read",
        ),
    ] {
        let (outcome, proposals) =
            committed(name, change, &[path_of("a").as_str()]);
        let PolicyOutcome::FindingsWithNotes { findings, notes } = outcome else {
            panic!("a note is expected for {name}, got {outcome:?}");
        };
        assert_eq!(findings, Vec::<PolicyFinding>::new(), "{name}");
        assert_eq!(proposals, Vec::<Correction>::new(), "{name}");
        assert_eq!(notes.len(), 1, "{name}");
        assert!(notes[0].message.contains(cause), "{name}: {}", notes[0].message);
    }
}

/// A dependent whose version cannot be bumped automatically is named and left alone;
///  a
/// commit never stops for it.
#[test]
fn a_commit_names_a_dependent_it_cannot_bump() {
    let root: PathBuf = fixture("dependent-commit-unsupported");
    let repo: PathBuf = workspace(root.as_path());
    // `@s/b`'s unsupported version sits in `HEAD`, so the dependent is stale rather than raised itself.
    prerelease_dependent(repo.as_path());
    let (lifecycle, real_index): (LifecycleContent, PathBuf) =
        commit_naming(repo.as_path(), &[path_of("a").as_str()]);
    git(repo.as_path(), &["add", "--", path_of("a").as_str()]);
    let mut facts: ScriptedFacts = facts_in(repo.as_path());
    let mut content: ContentState = ContentState::new();
    let outcome: PolicyOutcome = super::check_dependent_version(DependentCheck {
        content: &mut content,
        lifecycle: &lifecycle,
        facts: &mut facts,
        trigger: Trigger::PreForward,
        forwards_commit: true,
        real_index: Some(real_index.as_path()),
    });
    let proposals: Vec<Correction> = content.take_proposals();
    remove(root.as_path());
    let PolicyOutcome::FindingsWithNotes { findings, notes } = outcome else {
        panic!("a note is expected, got {outcome:?}");
    };
    assert_eq!(findings, Vec::<PolicyFinding>::new());
    assert_eq!(proposals, Vec::<Correction>::new());
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].path, None);
    assert!(
        notes[0].message.contains("@s/b"),
        "{}",
        notes[0].message
    );
}
