//! Controls for recovery failure messages.

use super::*;

/// The message is printed unchanged,
///  and filesystem failures name step,
///  path and reason.
#[test]
fn messages_name_the_step_and_path() {
    assert_eq!(
        RecoveryError(String::from("Unsafe registry")).to_string(),
        "Unsafe registry"
    );
    assert_eq!(
        io_failure(
            "listing",
            std::path::Path::new("/repo/.git/cli-git-transactions"),
            &std::io::Error::from(std::io::ErrorKind::PermissionDenied)
        )
        .to_string(),
        "listing /repo/.git/cli-git-transactions failed: permission denied"
    );
}
