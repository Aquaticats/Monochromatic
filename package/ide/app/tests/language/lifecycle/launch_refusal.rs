//! A launch policy that cannot produce a safe launch:
//!  the refusal reaches the display,
//!  and nothing is spawned in its place.

/// The scripted-server reader and the setup under test.
use crate::support::{
    self,
    PRODUCT_TIMEOUT,
    Probe,
    SERVER,
};
/// The state a refused server is in.
use ide_app::language::status::ServerState;
/// The setup the tests configure,
///  the request a policy is asked about,
///  and the direct launch the second policy starts from.
use ide_app::language::{
    config::LanguageSetup,
    launch::{
        LaunchRequest,
        ServerLaunch,
        launch_directly,
    },
};

/// A launch policy that cannot produce a safe launch.
fn refusing(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    return Err(format!("no confinement for {}", request.server));
}

/// A launch policy that needs a private directory below the project,
///  which the test configures as
/// the state root itself.
fn needing_state(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    let mut launch = launch_directly(request)?;
    launch
        .directories
        .push(request.project_root.join("state-inside-the-project"));
    return Ok(launch);
}

/// A refused or unpreparable launch is reported and nothing is spawned in its place.
#[test]
fn refused_launch_is_reported_and_nothing_is_spawned() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::launch_refusal::refused_launch_is_reported_and_nothing_is_spawned",
            support::standard,
        );
        return;
    };
    let refused = LanguageSetup {
        launch: refusing,
        state_root: None,
        extra_languages: Some(support::scripted(&root, &[], PRODUCT_TIMEOUT)),
        ..LanguageSetup::unconfined()
    };
    let mut probe = Probe::with_setup(&root, refused);
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the refused state");
    while !(matches!(probe.state(SERVER), Some(ServerState::LaunchRefused { .. }))) {
        probe.step();
    }
    assert_eq!(
        probe.state(SERVER),
        Some(&ServerState::LaunchRefused {
            reason: "no confinement for scripted-ls".to_string()
        })
    );
    drop(probe);
    // The project itself as the state root would make all of it writable, so preparation refuses it.
    let unprepared = LanguageSetup {
        launch: needing_state,
        state_root: Some(root.clone()),
        extra_languages: Some(support::scripted(&root, &[], PRODUCT_TIMEOUT)),
        ..LanguageSetup::unconfined()
    };
    let mut second = Probe::with_setup(&root, unprepared);
    second.open(&root.join("file.scripted"), "alpha\n");
    second.wait_for("the refused state");
    while !(matches!(
        second.state(SERVER),
        Some(ServerState::LaunchRefused { .. })
    )) {
        second.step();
    }
    let Some(ServerState::LaunchRefused {
        reason,
    }) = second.state(SERVER)
    else {
        panic!("unexpected state {:?}", second.state(SERVER));
    };
    assert!(reason.contains("contains the project"), "{reason}");
    assert!(
        !root.join("state-inside-the-project").exists(),
        "a private directory was created inside the project"
    );
    assert!(
        support::children().is_empty(),
        "a server was spawned although its launch was refused"
    );
    assert!(
        support::report(&root).is_empty(),
        "the server ran although its launch was refused"
    );
}
