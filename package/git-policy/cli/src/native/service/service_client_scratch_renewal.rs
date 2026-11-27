//! What:
//!  Keeps the modification time of the command's scratch directory fresh while a commit waits.
//! Why:
//!   A wait longer than the sweep's age must never have its directory swept,
//!  whatever the service sends;
//!  the directory is renewed at once the first time and then as often as the client run was configured with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (settings.keepFresh && (lastRenewed === undefined || elapsed(lastRenewed) >= settings.keepFresh.every)) { renew(settings.keepFresh.path); lastRenewed = now(); }
//! ```

use crate::service_client_settings_and_watches::ClientConfig;
use std::time::Instant;
/// What:
///  Renews the modification time of the command's scratch directory when it is due:
/// at once the first time,
/// then as often as
///  the config says.
/// Why:
///  A wait longer than the sweep's age must never have its directory swept,
/// whatever the service sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (settings.keepFresh && (lastRenewed === undefined || elapsed(lastRenewed) >= settings.keepFresh.every)) { renew(settings.keepFresh.path); lastRenewed = now(); }
/// ```
pub(crate) fn renew_scratch(
    settings: &ClientConfig,
    last_renewed: &mut Option<Instant>,
) {
    let Some(fresh) = settings.keep_fresh.as_ref() else {
        return;
    };
    let due: bool = match *last_renewed {
        Some(at) => at.elapsed() >= fresh.every,
        None => true,
    };
    if due {
        crate::scratch_location::renew(fresh.path.as_path());
        *last_renewed = Some(Instant::now());
    }
}
