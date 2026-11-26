//! What:
//!  The handler of the clone requests at the design's healthy state:
//!  its two installed halves,
//!  its configuration,
//!  the readiness and results folders,
//!  ZED and service wiring,
//!  this boot's readiness record,
//!  and the requests a receipt answered.
//! Why:
//!  The handler's checks read one of these facts each,
//!  so the whole set is built here at the build's own paths and modes,
//!  and a control changes the one owner,
//!  mode or flag it judges.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const handler = { installation: healthyInstallation(), configuration: healthyConfiguration(), folders: healthyFolders(), wiring: healthyWiring() };
//! ```

use super::handler_checks::{
    FolderFacts,
    HandlerConfigurationFacts,
    HandlerInstallationFacts,
    HandlerWiringFacts,
    PathState,
    READINESS_DIRECTORY_MODE,
    RESULTS_DIRECTORY_MODE,
    ROOT_OWNER,
    ReadinessFacts,
    RequestFacts,
};
use super::route_test_facts_names::{
    CLONE_DATASET,
    LIVE_BOOT,
};
use super::route_test_facts_paths::{
    path_facts,
    root_directory,
    root_file,
};
use crate::worktree_clone_mount_protocol::ReadyState;

/// The handler's two installed halves at the build's own paths.
pub(crate) fn healthy_installation() -> HandlerInstallationFacts {
    return HandlerInstallationFacts {
        executable: root_file(super::handler_checks::ROOT_MODE_EXECUTABLE_PATH),
        zedlet: root_file(super::handler_checks::ZEDLET_EVENT_PATH),
    };
}

/// The handler's configuration at the build's own path.
pub(crate) fn healthy_configuration() -> HandlerConfigurationFacts {
    return HandlerConfigurationFacts {
        file: path_facts(
            crate::worktree_clone_mount_handler::CONFIGURATION_PATH,
            PathState::File {
                owner: ROOT_OWNER,
                mode: 0o644,
            },
        ),
        parse_error: None,
    };
}

/// The readiness and results folders at the design's modes.
pub(crate) fn healthy_folders() -> FolderFacts {
    return FolderFacts {
        readiness: root_directory(
            crate::worktree_clone_mount_protocol::READINESS_DIRECTORY,
            READINESS_DIRECTORY_MODE,
        ),
        results: root_directory(
            crate::worktree_clone_mount_protocol::RESULTS_DIRECTORY,
            RESULTS_DIRECTORY_MODE,
        ),
    };
}

/// ZED installed and enabled,
/// with its own other zedlet beside the route's.
pub(crate) fn healthy_wiring() -> HandlerWiringFacts {
    return HandlerWiringFacts {
        zed_unit: root_file("/usr/lib/systemd/system/zfs-zed.service"),
        zed_enabled: true,
        other_zedlets: vec![String::from("history_event-zfs-list-cacher.sh")],
    };
}

/// This boot's readiness record with every prerequisite ready.
pub(crate) fn healthy_readiness() -> ReadinessFacts {
    return ReadinessFacts::Read {
        boot: String::from(LIVE_BOOT),
        pool: ReadyState::Ready,
        checkout: ReadyState::Ready,
        sweep: String::from("finished"),
    };
}

/// One clone's request answered by a receipt of this boot.
pub(crate) fn healthy_request() -> RequestFacts {
    return RequestFacts {
        clone: String::from(CLONE_DATASET),
        intent_nonce: Some(String::from("0123456789abcdef0123456789abcdef")),
        receipt_nonce: Some(String::from("0123456789abcdef0123456789abcdef")),
        receipt_boot: Some(String::from(LIVE_BOOT)),
    };
}
