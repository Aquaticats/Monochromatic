//! What:
//!  The words a waiting command is told about its own wait:
//! the place the service named,
//! or that it named none,
//! and that the command's own request is inside a write the service never ends.
//! Why:
//!  Which words are worth saying,
//! and when,
//! is one decision per wait,
//! and the client's send loop
//!  (`service_client.rs`) is the only reader of the clock that decides it.
//! Keeping the words apart lets the loop stay about the request:
//! its sends,
//! its answer,
//! and what it does when the service goes away.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! sayWhereItStands(stderr, config, parked, place, told); sayTheWriteItIsInside(stderr, config, parked, words, told);
//! ```

/// What:
///  `ClientConfig` is what a client run waits for and how long.
/// Why:
///   Both functions read the same wait
///   (`waiting_notice_after`) and neither reads anything else of the client.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { ClientConfig } from './service-client-settings-and-watches.ts';
/// ```
use crate::service_client_settings_and_watches::ClientConfig;
/// What:
///  The two wordings of a waiting commit's place.
/// Why:
///   The words belong beside the wait that decides they are due.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { positionNotice, unplacedNotice } from './service-messages.ts';
/// ```
use crate::service_messages::{position_notice, unplaced_notice};
use std::io::Write;
use std::time::Instant;

/// What:
///  The number of the place the service named,
/// which is the part of it the person is told.
/// Why:
///   A place says which commit it is (`number`) and how many the batch holds (`of`),
/// and every wording of it derives from the number alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placeNumber([3, 7]) // 3
/// ```
pub(crate) fn place_number(place: (usize, usize)) -> usize {
    let (number, _of) = place;
    return number;
}

/// What:
///  Says where a waiting commit stands once it has waited long enough to matter:
/// how many commits the service put before it,
/// or that the service has not said where it stands.
/// Why:
///   Two commits waited 62 and 32 seconds and said nothing about themselves,
/// and a person cannot tell a commit the service has not
///  reached from one it has lost.
/// The telling costs one clock read of the loop the client already runs and one line;
/// it waits for nothing,
///  takes no lock and writes no file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayWhereItStands(notices, config, parked, place, told)
/// ```
pub(crate) fn say_where_it_stands(notices: &mut dyn Write, config: &ClientConfig, parked: Option<Instant>, place: Option<(usize, usize)>, told: &mut Option<Option<usize>>) {
    let Some(since) = parked else {
        return;
    };
    if since.elapsed() < config.waiting_notice_after {
        return;
    }
    // What the person was last told: the place the service named, or nothing when it named none.
    let named: Option<usize> = place.map(place_number);
    if *told == Some(named) {
        return;
    }
    *told = Some(named);
    let words: String = match place {
        Some((number, of)) => position_notice(number, of),
        // What the client knows and no more: the service named no place, and how long the commit has waited.
        None => unplaced_notice(since.elapsed()),
    };
    let _written = writeln!(notices, "{words}");
}

/// What:
///  Says that the command's own request is inside a write of the service,
/// once the wait has mattered or the service's words about that write have changed.
/// Why:
///   The service says it the moment the write begins,
/// because a client whose ceiling falls inside the write would otherwise
///  miss it (sol-1);
/// a write that takes what writes take is over in milliseconds,
/// so a line said then reaches the person of a
///  command that lands at once and says nothing else.
/// The words therefore wait for the wait the place of a waiting commit waits for
///  (`waiting_notice_after`),
/// and every change of them past that wait is said,
/// so a write that has stayed stuck a new minute is told in its new words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayTheWriteItIsInside(notices, config, parked, words, told)
/// ```
pub(crate) fn say_the_write_it_is_inside(notices: &mut dyn Write, config: &ClientConfig, parked: Option<Instant>, words: Option<&str>, told: &mut Option<String>) {
    let Some(text) = words else {
        return;
    };
    if told.as_deref() == Some(text) {
        return;
    }
    let Some(since) = parked else {
        return;
    };
    if since.elapsed() < config.waiting_notice_after {
        return;
    }
    *told = Some(text.to_string());
    // Terminal output error: a notice is advisory; a failed write must not stop the commit.
    let _written = write!(notices, "{text}");
}
