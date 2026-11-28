//! What:
//!  Controls for the two failures of this file that print the policy stage's lines first:
//!  a repository location that cannot be read,
//!  and a configuration the command needs and cannot use.
//! Why:
//!  The alpha.51 release pass made what the stage printed lead both failures,
//!  as it leads the three slot refusals,
//!  and no control named either:
//!  an edit that dropped the stage's lines from one branch left every test green
//!  (the second pass of that release,
//!  its L1).
//!  Each control reads the text of a real finding of the real stage,
//!  so no sentence here is a stand-in for one the program prints.
//!  Neither state is reachable through `plan_invocation`:
//!  a command whose words need the configuration is stopped by
//!  `prepare_guarded_command` before the stage runs,
//!  and one whose words skip the configuration selects no policy that prints.
//!  So each control hands `forward_action` the text the stage rendered and a
//!  facts provider that answers as the failing command's does,
//!  which is the state this file's two lines decide.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const action = forwardAction(realGit, args, overlay, printed, env, false, checks);
//! // expect(action.stderr).toBe(printed + theFailure);
//! ```
#![cfg(unix)]

/// The planner,
///  its action and the failure status both of its branches end in.
use super::*;
/// The request the stage makes for the bytes of a file an `add` would stage.
use crate::candidate_prediction::CandidateRequest;
use crate::config_file::CONFIG_FILE_NAME;
/// The settings a stage runs with,
///  and the one constructor of the shipped checks.
use crate::config_schema::PolicyConfig;
use crate::invocation_config::{
    config_invalid_event,
    load_identity_config,
};
use crate::policy_checks::shipped_checks;
use crate::policy_content::LifecycleContent;
use crate::policy_engine::{
    StageRequest,
    run_policy_stage,
};
use crate::policy_events::render_policy_events;
/// The one policy each control runs,
///  and the severity that makes it print a finding instead of refusing.
use crate::policy_registry::{
    PolicyId,
    Severity,
};
use crate::policy_trigger::Trigger;
use crate::repo_read::the_switch_that_turns_the_answers_off::switch_off;
/// The facts provider of a real invocation,
///  which is what `forward_action` is written against.
use crate::repository_facts::git_facts;
use crate::start_directory::StartDirectory;
/// The fixture helpers every control of this crate shares.
use crate::test_support::{
    REAL_GIT,
    fixture,
    git,
    isolated_overlay,
    remove,
    repository,
};
use crate::wrapper_controls::no_controls;

/// The file a control writes without its final line feed,
///  which is the finding the stage prints.
const THE_FILE: &str = "notes.txt";

/// The bytes of `THE_FILE`:
///  one line feed at the end of the first line and none at the end.
const WITHOUT_A_LINE_FEED: &[u8] = b"one\ntwo";

/// The two words a control plans,
///  which write the index and so read this repository's configuration.
const THE_COMMAND: &[&str] = &["reset", "--hard"];

/// The settings file this crate reads,
///  which holds the invalid text a control writes.
const AN_INVALID_SETTINGS_FILE: &[u8] = b"{\"policies\": {\"no-such-policy\": \"warn\"}}";

/// One fixture folder,
///  and the repository inside it.
/// Why:
///  The finding and the failing command are about one repository,
///  and a control removes the folder it made however it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scene = { root: string; repo: string };
/// ```
struct Scene {
    /// The fixture folder this control removes when it ends.
    root: PathBuf,
    /// The repository whose file the stage reports on.
    repo: PathBuf,
}

/// What:
///  `a_scene(name)` makes one repository with `base.txt` committed and `THE_FILE` written
///  without its final line feed.
/// Why:
///  A finding of `final-newline` needs a text file with a line feed missing at its end,
///  and a repository whose `HEAD` exists is the shape every wrapped command runs in.
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
    // The base commit is made by the real Git, never by this crate's commit path, so the bytes this scene writes
    //  afterwards reach the worktree exactly as written.
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join(THE_FILE), WITHOUT_A_LINE_FEED)
        .expect("the file without its line feed");
    return Scene {
        root,
        repo,
    };
}

/// What:
///  The global options that ask Git about `repo`,
///  which is what a command typed there carries.
/// Why:
///  The provider reads the repository layout from the options the caller wrote,
///  and a control passes no directory to a process-wide call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thePrefixFor = (repo: string): string[] => ['-C', repo];
/// ```
fn the_prefix_for(repo: &Path) -> Vec<OsString> {
    return vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
}

/// What:
///  The settings this crate runs `final-newline` at when a section says `warn`.
/// Why:
///  A `warn` is the severity a person sets to be told and still get the command,
///  and it is the finding each control holds beside the failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function the_config_with_a_warn() { const config = PolicyConfig.defaults(); config.setting('final-newline').severity = 'warn'; return config; }
/// ```
fn the_config_with_a_warn() -> PolicyConfig {
    let mut config: PolicyConfig = PolicyConfig::defaults();
    for setting in &mut config.settings {
        if setting.id == PolicyId::FinalNewline {
            setting.severity = Severity::Warn;
        }
    }
    return config;
}

/// What:
///  The event line the stage prints for `THE_FILE` in this scene,
///  read from the real stage and the real renderer.
/// Why:
///  The subject of both controls is what this file does with the text the stage printed,
///  so the text has to be the program's own,
///  and one policy selected keeps another policy's line out of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function the_printed_finding(checks): string {
///   const request = { trigger: 'pre-forward', config: theConfigWithAWarn(), controls: noControls(), selected: ['final-newline'] };
///   return renderPolicyEvents(0, runPolicyStage(request, ['final-newline'], checks).events);
/// }
/// ```
fn the_printed_finding(checks: &mut ShippedChecks<GitFacts>) -> String {
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: the_config_with_a_warn(),
        controls: no_controls(),
        selected: vec![PolicyId::FinalNewline],
    };
    // The same request `git add -- notes.txt` makes of the policies.
    checks.candidates = LifecycleContent::Requested(CandidateRequest::Add(vec![
        OsString::from("--"),
        OsString::from(THE_FILE),
    ]));
    let stage = run_policy_stage(&request, &[PolicyId::FinalNewline], checks);
    return render_policy_events(0, stage.events.as_slice());
}

/// What:
///  The checks one control plans with,
///  built the way `plan_with_real_git` builds them.
/// Why:
///  `forward_action` is written against this provider,
///  and the arguments and the start directory are the command's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function the_checks(facts: GitFacts, scene: Scene): ShippedChecks<GitFacts>
/// ```
fn the_checks(
    facts: GitFacts,
    scene: &Scene,
) -> ShippedChecks<GitFacts> {
    return shipped_checks(
        facts,
        THE_COMMAND
            .iter()
            .map(|word| return OsString::from(*word))
            .collect(),
        LifecycleContent::None,
        Vec::<crate::effective_target::ToolCacheDirectory>::new(),
        &StartDirectory::at(scene.repo.as_path()),
    );
}

/// What:
///  The words one control forwards,
///  as the wrapper resolved them.
/// Why:
///  `forward_action` plans the command it is given,
///  and a control states the same words twice:
///  once for the checks and once for the planner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const the_arguments = (): string[] => ['reset', '--hard'];
/// ```
fn the_arguments() -> Vec<OsString> {
    return THE_COMMAND
        .iter()
        .map(|word| return OsString::from(*word))
        .collect();
}

/// What:
///  The text one action writes to standard error,
///  held to the failure status this file's two branches end in.
/// Why:
///  A control reads the whole report,
///  and an action that is not a failure is a different ending than the branch under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function the_report_of(action: Action): string
/// ```
fn the_report_of(action: Action) -> String {
    match action {
        Action::Exit {
            code,
            stdout,
            stderr,
        } => {
            assert_eq!(code, ENGINE_FAILURE_EXIT_CODE, "{stderr}");
            assert!(
                stdout.is_empty(),
                "a failure writes nothing to standard output"
            );
            return stderr;
        }
        other => panic!("this file's two failures end the command: {other:?}"),
    }
}

/// What:
///  The stage's finding leads the failure when the repository location cannot be read.
/// Why:
///  A person who turned a rule down to `warn` in order to be told reads that line,
///  and the failure that follows the stage used to report without it.
///  The provider here cannot start Git at all,
///  and its in-process layout reader is switched off so the read reaches the start,
///  which is the failure `failure_after` builds.
#[test]
fn the_stage_s_lines_lead_the_failure_when_the_location_cannot_be_read() {
    let scene: Scene = a_scene("forward-planning-unreadable-location");
    let reading: GitFacts = git_facts(
        Path::new(REAL_GIT),
        the_prefix_for(scene.repo.as_path()).as_slice(),
        isolated_overlay().as_slice(),
    );
    let mut finding_checks: ShippedChecks<GitFacts> = the_checks(reading, &scene);
    let printed: String = the_printed_finding(&mut finding_checks);
    assert!(
        printed.contains("\"code\":\"final-newline/noncanonical-final-newline\""),
        "the stage printed the finding this control holds: {printed}"
    );
    // The Git the command would ask is not there, and the reader that answers a plain layout in process is off, so
    //  the repository location is the fact that fails.
    let missing: PathBuf = scene.root.join("no-git");
    let mut checks: ShippedChecks<GitFacts> = the_checks(
        git_facts(missing.as_path(), &[], switch_off().as_slice()),
        &scene,
    );
    // The sentence is the provider's own, asked of the same reader the branch asks.
    let message: String = checks
        .facts
        .location()
        .expect_err("no Git to ask for the repository's location");
    let report: String = the_report_of(forward_action(
        PathBuf::from(REAL_GIT),
        the_arguments(),
        isolated_overlay(),
        printed.clone(),
        &[],
        false,
        &mut checks,
    ));
    assert_eq!(
        report,
        format!("{printed}monochromatic-git-policy-cli: {message}\n"),
        "the stage's lines lead the failure, and the failure is unchanged beside them"
    );
    remove(scene.root.as_path());
}

/// What:
///  The stage's finding leads the failure when the configuration cannot be read.
/// Why:
///  The same person reads the same finding,
///  and the branch that reports an invalid settings file printed its event alone.
///  The settings file is written after the stage ran,
///  so the failure is the one `forward_action` meets and never one an earlier read would have met.
#[test]
fn the_stage_s_lines_lead_the_failure_when_the_configuration_cannot_be_read() {
    let scene: Scene = a_scene("forward-planning-unreadable-configuration");
    let reading: GitFacts = git_facts(
        Path::new(REAL_GIT),
        the_prefix_for(scene.repo.as_path()).as_slice(),
        isolated_overlay().as_slice(),
    );
    let mut checks: ShippedChecks<GitFacts> = the_checks(reading, &scene);
    let printed: String = the_printed_finding(&mut checks);
    assert!(
        printed.contains("\"code\":\"final-newline/noncanonical-final-newline\""),
        "the stage printed the finding this control holds: {printed}"
    );
    std::fs::write(scene.repo.join(CONFIG_FILE_NAME), AN_INVALID_SETTINGS_FILE)
        .expect("an invalid settings file");
    // The event the person reads is the program's own, asked of the same reader the branch asks.
    let location: RepositoryLocation = checks
        .facts
        .location()
        .expect("the fixture repository's location");
    let error = load_identity_config(&location.identity)
        .expect_err("the settings file this control wrote is refused");
    let expected: String = format!("{printed}{}", config_invalid_event(&error));
    let report: String = the_report_of(forward_action(
        PathBuf::from(REAL_GIT),
        the_arguments(),
        isolated_overlay(),
        printed,
        &[],
        false,
        &mut checks,
    ));
    assert_eq!(
        report, expected,
        "the stage's lines stand before the configuration event and change nothing of it"
    );
    remove(scene.root.as_path());
}
