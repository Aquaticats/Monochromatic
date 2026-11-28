//! What:
//!  The hint a `final-newline` finding gives at each place it prints:
//!  an `add`,
//!  a direct check,
//!  a commit whose selection is read-only,
//!  a landed commit and a manual push,
//!  and the two lifecycles that correct the bytes themselves.
//! Why:
//!  A hint must be true where it is printed.
//!  An `add` of a file without its line feed told the person to edit the file by hand,
//!  and the commit that followed corrected the file itself,
//!  which is how a round of issue 6622 came to report a warning that was true (issue 6627);
//!  the same place then named a commit's correction for a file the person could not stage at `error`
//!  and for a commit whose selection is read-only.
//!  A manual push's finding told the person to stage the file again
//!  although the bytes were already in the pushed commit.
//!  Each control runs the policy stage for one lifecycle and reads the `hint` field of the
//!  finding line that stage rendered,
//!  so every word asserted here is the program's own,
//!  and the `code`,
//!  `message` and `fix` fields beside it are held to what `SPEC.md` says they keep.
//!  The two commit places take the read-only answer from the commit's own words through
//!  `decide_selection`,
//!  which is the function the commit path asks,
//!  so no control hands the stage a flag it made up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(hintOf(theStageFor('add'))).toBe(THE_HINT_OF_A_COMMIT_THAT_CORRECTS);
//! ```
#![cfg(unix)]

/// Import the report and the stage under test.
use crate::candidate_object::{
    ObjectId,
    parse_object_id,
};
use crate::candidate_prediction::{
    CandidateRequest,
    PushedScope,
};
/// Import the settings a stage runs with,
///  the shipped checks that answer it,
///  and the candidates a lifecycle offers.
use crate::command_commit::{
    CommitRegion,
    parse_commit_region,
};
use crate::commit_transaction::decide_selection;
use crate::config_schema::PolicyConfig;
use crate::policy_checks::shipped_checks;
use crate::policy_content::LifecycleContent;
/// Import the stage,
///  its request and the renderer of the events it produces.
use crate::policy_engine::{
    StageRequest,
    run_policy_stage,
};
use crate::policy_events::render_policy_events;
/// Import the policy whose hint these controls read,
///  its severity,
///  and the lifecycle points.
use crate::policy_registry::{
    PolicyId,
    Severity,
};
/// Import the scripted facts that prepare real candidates in a fixture repository.
use crate::policy_test_support::{
    ScriptedFacts,
    scripted_facts,
};
use crate::policy_trigger::Trigger;
use crate::start_directory::StartDirectory;
/// Import the fixture helpers every control of this crate shares.
use crate::test_support::{
    fixture,
    git,
    git_text,
    remove,
    repository,
};
use crate::transaction_journal::Conclusion;
/// Import the record of an invocation that used no wrapper control.
use crate::wrapper_controls::no_controls;
/// Owned operating-system text stands in for the pathspecs,
///  and the fixture is a pair of paths.
use std::ffi::OsString;
use std::path::PathBuf;

/// The file every control of this file writes without its line feed.
const THE_FILE: &str = "notes.txt";

/// The bytes of `THE_FILE` when it ends with no line feed at all.
const WITHOUT_A_LINE_FEED: &[u8] = b"one\ntwo";

/// The hint of a direct check:
///  a commit that takes the file corrects the ending,
///  and the person may correct it now without committing.
///  `src/native/policy/event_hints.rs`'s `finding_hint` holds this text.
const THE_HINT_OF_A_DIRECT_CHECK: &str = "Hint: if you are committing this file, a commit that takes it \
     corrects the ending for you while this repository's settings keep `final-newline` on; to correct it now without \
     committing, run `git monochromatic-git-policy-cli fix -- <path>`.";

/// The hint of an `add`:
///  the correction now and the staging that follows it lead,
///  and a plain commit's own correction follows with the selection that records the bytes instead.
///  `src/native/policy/event_hints.rs`'s `finding_hint` holds this text.
const THE_HINT_OF_AN_ADD: &str = "Hint: to correct the ending now, run `git monochromatic-git-policy-cli fix -- \
     <path>` and then stage the file again with `git add -- <path>`, because an index that already holds these bytes \
     keeps them until the file is staged once more; a plain commit, one that takes the whole file, also corrects the \
     ending while this repository's settings keep `final-newline` on, but a commit whose selection is read-only (`-p`, \
     `-i` or `--include`) records the staged bytes exactly as they are.";

/// The hint of a lifecycle whose bytes nothing corrects and which the index still holds.
///  `src/native/policy/event_hints.rs`'s `finding_hint` holds this text.
const THE_HINT_WHEN_THE_INDEX_HOLDS_THE_BYTES: &str = "Hint: if you are committing this file, edit it so it ends \
     with exactly one newline, then stage it again.";

/// The hint of a lifecycle whose bytes a commit already holds.
///  `src/native/policy/event_hints.rs`'s `finding_hint` holds this text.
const THE_HINT_WHEN_A_COMMIT_HOLDS_THE_BYTES: &str = "Hint: this file's bytes are already in a commit, so \
     nothing here corrects them; edit the file so it ends with exactly one newline, then commit the correction.";

/// The hint of a lifecycle that corrects the bytes itself.
///  `src/native/policy/event_hints.rs`'s `finding_hint` holds this text.
const THE_HINT_WHERE_THE_LIFECYCLE_CORRECTS: &str = "Hint: if you are committing or checking this file, a commit \
     corrects the ending for you; to correct it now without committing, run `git monochromatic-git-policy-cli fix -- \
     <path>`.";

/// The `code` and `message` of a finding of this policy,
///  which no hint may change (`SPEC.md`,
///  "Hints").
const THE_CODE_AND_MESSAGE: &str = "\"code\":\"final-newline/noncanonical-final-newline\",\"message\":\"Non-empty text file must end with exactly one LF byte.\"";

/// What:
///  One fixture repository,
///  its `HEAD` an ordinary commit of `base.txt`,
///  and `THE_FILE` in the worktree without its line feed.
///  `root` is the fixture folder the whole scene lives in,
///  and `repo` the repository inside it.
/// Why:
///  Every lifecycle of this file differs only in which state of the same repository it reads,
///  so one scene holds the repository and each control says only its own change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scene = { root: string; repo: string };
/// ```
struct Scene {
    /// The fixture folder this scene removes when its control ends.
    root: PathBuf,
    /// The repository whose file every control reports on.
    repo: PathBuf,
}

/// What:
///  `a_scene(name)` makes the repository one control reads,
///  with `base.txt` committed and `THE_FILE` written without its line feed.
/// Why:
///  A commit's candidates are the paths it changes against a commit that already exists,
///  so the scene has to start with one,
///  and a repository with an unborn branch would exercise none of the states this file is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aScene(name: string): Scene
/// ```
fn a_scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(repo.as_path(), &["add", "--", "base.txt"]);
    // The base commit is made by the real Git, never by this crate's commit path, so the file this scene writes
    //  afterwards reaches the index and the history exactly as written.
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join(THE_FILE), WITHOUT_A_LINE_FEED)
        .expect("the file without its line feed");
    return Scene {
        root,
        repo,
    };
}

/// What:
///  `the_commit_of_the_scene(scene)` is the object name of the base commit,
///  which every state this file stages or commits is compared against.
/// Why:
///  A `StagedAtBase` request names the commit its entries are read against,
///  and a candidate is what that commit does not already hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCommitOfTheScene(scene: Scene): string
/// ```
fn the_commit_of_the_scene(scene: &Scene) -> ObjectId {
    let name: String = git_text(scene.repo.as_path(), &["rev-parse", "HEAD"]);
    return parse_object_id(name.as_bytes()).expect("the base commit is an object name");
}

/// What:
///  `stage_the_file(scene)` stages `THE_FILE` as it stands,
///  which is the state an `add` leaves behind for a commit to read.
/// Why:
///  The bytes that commit holds are the bytes this control's finding is about,
///  and a lifecycle that corrects nothing must be given the bytes nobody corrected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stageTheFile(scene: Scene): void
/// ```
fn stage_the_file(scene: &Scene) {
    git(scene.repo.as_path(), &["add", "--", THE_FILE]);
}

/// What:
///  `commit_the_file(scene)` puts `THE_FILE` as it stands into a commit the real Git makes,
///  and answers that commit's object name.
/// Why:
///  A landed commit's own bytes and a pushed tree's bytes are what the post-commit gate and the
///  manual-push gate read,
///  and only a commit made outside the wrapper's commit path holds bytes no correction touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commitTheFile(scene: Scene): string
/// ```
fn commit_the_file(scene: &Scene) -> ObjectId {
    stage_the_file(scene);
    git(
        scene.repo.as_path(),
        &["commit", "--quiet", "--message=the file"],
    );
    return the_commit_of_the_scene(scene);
}

/// What:
///  `rendered_finding(scene, trigger, candidates, can_apply_patches, config)` runs the
///  `final-newline` policy through the stage for one lifecycle point and answers the event
///  lines the stage rendered.
///  `can_apply_patches` says whether this lifecycle may correct the bytes it reads.
/// Why:
///  The hint is chosen where the event is rendered,
///  so the answer has to be read from the events of a real pass,
///  never from a call to the policy alone;
///  and one policy at the stage keeps the lines of other policies out of the reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderedFinding(scene, trigger, candidates, canApplyPatches, config): string
/// ```
fn rendered_finding(
    scene: &Scene,
    trigger: Trigger,
    candidates: CandidateRequest,
    can_apply_patches: bool,
    config: &PolicyConfig,
) -> String {
    // What: the scripted facts prepare the candidates with the real Git, in the scene's repository.
    // Why:  the policy reads bytes Git hashed from the worktree or read out of the index, and a stand-in would
    //       answer about bytes no lifecycle of this file reads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const facts = scriptedFacts(); facts.candidatesRepository = scene.repo;
    // ```
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(scene.repo.clone());
    let mut checks = shipped_checks(
        facts,
        Vec::<OsString>::new(),
        LifecycleContent::Requested(candidates),
        Vec::<crate::effective_target::ToolCacheDirectory>::new(),
        &StartDirectory::at(scene.repo.as_path()),
    );
    checks.can_apply_patches = can_apply_patches;
    let request: StageRequest = StageRequest {
        trigger,
        config: config.clone(),
        controls: no_controls(),
        // `vec![...]` is an owned list of one: only the policy whose hint is read runs.
        selected: vec![PolicyId::FinalNewline],
    };
    let stage = run_policy_stage(&request, &[PolicyId::FinalNewline], &mut checks);
    return render_policy_events(0, stage.events.as_slice());
}

/// What:
///  `the_hint_in(rendered)` is the text of the `hint` field of the one finding line `rendered`.
/// Why:
///  The control compares the words the program printed,
///  and the value ends at the next quote because no hint text of this crate holds one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theHintIn(rendered: string): string
/// ```
fn the_hint_in(rendered: &str) -> String {
    let opening: &str = "\"hint\":\"";
    let start: usize = rendered
        .find(opening)
        .unwrap_or_else(|| panic!("the stage printed a finding with no hint: {rendered}"))
        + opening.len();
    let rest: &str = &rendered[start..];
    let end: usize = rest
        .find('"')
        .unwrap_or_else(|| panic!("the hint of this finding never ends: {rendered}"));
    return String::from(&rest[..end]);
}

/// What:
///  `assert_the_finding_keeps_its_fields(rendered, fix)` holds the finding line to the fields
///  no hint may change.
/// Why:
///  `SPEC.md`'s "Hints" says the hint never changes `message`,
///  `code` or `fix`,
///  so a control that reads the hint must also read what stayed the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertTheFindingKeepsItsFields(rendered: string, fix: 'none' | 'available'): void
/// ```
fn assert_the_finding_keeps_its_fields(
    rendered: &str,
    fix: &str,
) {
    assert!(rendered.contains(THE_CODE_AND_MESSAGE), "{rendered}");
    assert!(
        rendered.contains(format!("\"fix\":\"{fix}\"").as_str()),
        "{rendered}"
    );
    assert!(
        rendered.contains(format!("\"path\":\"{THE_FILE}\"").as_str()),
        "{rendered}"
    );
}

/// What:
///  `a_read_only_selection(words)` answers whether a commit whose own words are `words` leaves the
///  policies read-only,
///  read from the release code the commit path asks.
/// Why:
///  `-p`, `-i` and `--include` are the forms that record the bytes the index holds,
///  and the answer must come from parsing those words and deciding the selection,
///  never from a flag a control wrote,
///  or the control would agree with whatever the release code happened to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aReadOnlySelection(words: string[]): boolean
/// ```
fn a_read_only_selection(words: &[&str]) -> bool {
    let written: Vec<OsString> = words.iter().copied().map(OsString::from).collect();
    let region: CommitRegion = match parse_commit_region(written.as_slice(), &[]) {
        Ok(found) => found,
        Err(error) => panic!("the commit's own words {words:?} are refused: {error:?}"),
    };
    return decide_selection(&region, Conclusion::None).read_only;
}

/// What:
///  `the_config_with(severity)` is the settings of this repository with `final-newline` set to
///  `severity`.
/// Why:
///  The correction a commit makes follows whether the policy runs at all,
///  so a control of that correction states the severity it runs at instead of reading a default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theConfigWith(severity: 'off' | 'warn' | 'error'): PolicyConfig
/// ```
fn the_config_with(severity: Severity) -> PolicyConfig {
    let mut config: PolicyConfig = PolicyConfig::defaults();
    for setting in &mut config.settings {
        if setting.id == PolicyId::FinalNewline {
            setting.severity = severity;
        }
    }
    return config;
}

/// An `add` leads with the two steps that correct the file where it is refused,
///  says why staging follows the fix,
///  and names a plain commit's own correction with the read-only selection that records the bytes instead.
///  The hint is the same at `warn`, where the add runs, and at `error`, where it stages nothing.
#[test]
fn an_add_says_the_fix_then_staging_and_what_a_plain_commit_takes() {
    let scene: Scene = a_scene("final-newline-hint-add");
    for severity in [Severity::Warn, Severity::Error] {
        let rendered: String = rendered_finding(
            &scene,
            Trigger::PreForward,
            // The same request `git add -- notes.txt` makes of the policies.
            CandidateRequest::Add(vec![OsString::from("--"), OsString::from(THE_FILE)]),
            false,
            &the_config_with(severity),
        );
        assert_eq!(
            the_hint_in(rendered.as_str()),
            THE_HINT_OF_AN_ADD,
            "{severity:?}: {rendered}"
        );
        assert_the_finding_keeps_its_fields(rendered.as_str(), "none");
        assert!(
            rendered.contains("\"trigger\":\"pre-forward\""),
            "{rendered}"
        );
    }
    remove(scene.root.as_path());
}

/// A direct check reads the worktree bytes a commit would take,
///  so its hint says that commit
///  corrects them,
///  and offers the correction now.
#[test]
fn a_direct_check_says_the_commit_that_takes_the_file_corrects_it() {
    let scene: Scene = a_scene("final-newline-hint-check");
    let rendered: String = rendered_finding(
        &scene,
        Trigger::DirectCheck,
        CandidateRequest::Direct(vec![OsString::from(THE_FILE)]),
        false,
        &the_config_with(Severity::Warn),
    );
    assert_eq!(
        the_hint_in(rendered.as_str()),
        THE_HINT_OF_A_DIRECT_CHECK,
        "{rendered}"
    );
    assert_the_finding_keeps_its_fields(rendered.as_str(), "none");
    assert!(
        rendered.contains("\"trigger\":\"direct-check\""),
        "{rendered}"
    );
    remove(scene.root.as_path());
}

/// A commit whose selection is read-only records the staged bytes as they are,
///  so its hint tells the person to edit the file and stage it again,
///  and never that a commit will correct it.
///  The selection is the commit's own: each spelling is parsed and decided by the release code,
///  and the stage is handed the answer the commit path hands it.
#[test]
fn a_read_only_selection_of_the_commands_own_words_says_to_edit_and_stage() {
    let scene: Scene = a_scene("final-newline-hint-selection");
    // The staged copy is the state a patch or include selection commits, and it ends with no line feed.
    stage_the_file(&scene);
    for words in [vec!["--patch"], vec!["--interactive"], vec!["--include"]] {
        let read_only: bool = a_read_only_selection(words.as_slice());
        assert!(read_only, "{words:?} leaves the policies writable");
        let rendered: String = rendered_finding(
            &scene,
            Trigger::PreForward,
            crate::commit_transaction::staged_at_base(
                Some(the_commit_of_the_scene(&scene)),
                &[THE_FILE.as_bytes().to_vec()],
            ),
            !read_only,
            &the_config_with(Severity::Warn),
        );
        assert_eq!(
            the_hint_in(rendered.as_str()),
            THE_HINT_WHEN_THE_INDEX_HOLDS_THE_BYTES,
            "{words:?}: {rendered}"
        );
        assert_the_finding_keeps_its_fields(rendered.as_str(), "none");
    }
    remove(scene.root.as_path());
}

/// A commit that may correct the bytes it stages corrects them itself,
///  so its hint says so and offers no second correction,
///  whatever severity the repository sets and without a fix being applied by the person.
///  A commit that names no read-only selection is the positive case beside the three spellings above:
///  the same decision answers `false` for it, and the stage then reports the correction as its own.
#[test]
fn a_commit_that_corrects_the_file_says_so_at_either_severity() {
    let scene: Scene = a_scene("final-newline-hint-commit");
    stage_the_file(&scene);
    let writable: bool = !a_read_only_selection(&[]);
    assert!(
        writable,
        "a commit with no selection leaves the policies writable"
    );
    for severity in [Severity::Warn, Severity::Error] {
        let rendered: String = rendered_finding(
            &scene,
            Trigger::PreForward,
            crate::commit_transaction::staged_at_base(
                Some(the_commit_of_the_scene(&scene)),
                &[THE_FILE.as_bytes().to_vec()],
            ),
            writable,
            &the_config_with(severity),
        );
        assert_eq!(
            the_hint_in(rendered.as_str()),
            THE_HINT_WHERE_THE_LIFECYCLE_CORRECTS,
            "{severity:?}: {rendered}"
        );
        assert_the_finding_keeps_its_fields(rendered.as_str(), "available");
    }
    remove(scene.root.as_path());
}

/// A direct fix corrects the selected worktree bytes itself,
///  so its hint says so.
#[test]
fn a_direct_fix_says_the_correction_is_here() {
    let scene: Scene = a_scene("final-newline-hint-fix");
    let rendered: String = rendered_finding(
        &scene,
        Trigger::DirectFix,
        CandidateRequest::Direct(vec![OsString::from(THE_FILE)]),
        false,
        &the_config_with(Severity::Warn),
    );
    assert_eq!(
        the_hint_in(rendered.as_str()),
        THE_HINT_WHERE_THE_LIFECYCLE_CORRECTS,
        "{rendered}"
    );
    assert_the_finding_keeps_its_fields(rendered.as_str(), "available");
    remove(scene.root.as_path());
}

/// A landed commit's bytes are already recorded,
///  so nothing corrects them and its hint tells the person to edit the file and commit the correction.
#[test]
fn a_landed_commit_says_to_commit_the_correction() {
    let scene: Scene = a_scene("final-newline-hint-landed");
    let landed: ObjectId = commit_the_file(&scene);
    let rendered: String = rendered_finding(
        &scene,
        Trigger::PostCommit,
        CandidateRequest::Landed(landed),
        false,
        &the_config_with(Severity::Warn),
    );
    assert_eq!(
        the_hint_in(rendered.as_str()),
        THE_HINT_WHEN_A_COMMIT_HOLDS_THE_BYTES,
        "{rendered}"
    );
    assert_the_finding_keeps_its_fields(rendered.as_str(), "none");
    assert!(
        rendered.contains("\"trigger\":\"post-commit\""),
        "{rendered}"
    );
    remove(scene.root.as_path());
}

/// A pushed commit's bytes are already recorded,
///  so nothing corrects them and its hint names the step a person can take after the commit is made:
///  edit the file and commit the correction.
#[test]
fn a_manual_push_says_to_commit_the_correction() {
    let scene: Scene = a_scene("final-newline-hint-push");
    let pushed: ObjectId = commit_the_file(&scene);
    let rendered: String = rendered_finding(
        &scene,
        Trigger::ManualPush,
        CandidateRequest::Pushed(PushedScope {
            commits: vec![pushed],
            // `Vec::new()` is an empty owned list: this push publishes one commit, so no whole tree is read.
            trees: Vec::new(),
        }),
        false,
        &the_config_with(Severity::Warn),
    );
    assert_eq!(
        the_hint_in(rendered.as_str()),
        THE_HINT_WHEN_A_COMMIT_HOLDS_THE_BYTES,
        "{rendered}"
    );
    assert_the_finding_keeps_its_fields(rendered.as_str(), "none");
    assert!(
        rendered.contains("\"trigger\":\"manual-push\""),
        "{rendered}"
    );
    remove(scene.root.as_path());
}
