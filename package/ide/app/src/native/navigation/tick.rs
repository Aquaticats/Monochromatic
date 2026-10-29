//! Nonblocking navigation replies and lazy directory refresh scheduling.

/// Toolkit callbacks retain only UI-thread owners and immutable worker replies.
use super::{AppWindow, Navigation, State, open, present, search, watch};
/// Queue failures remain actionable diagnostics with their affected directory path.
use anyhow::Result;
/// Timed retry/refresh bounds avoid reading an inaccessible folder on every UI tick.
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

/// Consume the sole directory reply and retain its error until that same target recovers.
fn directory_reply(navigation: &mut Navigation) -> bool {
    if !navigation.reader_available {
        return false;
    }
    let was_busy = navigation.reader.is_busy();
    match navigation.reader.poll(&mut navigation.tree) {
        Ok(changed) => {
            // A changed `.gitignore` or exclude file can change the rows of every directory below it,
            // and nothing notifies those directories, so they are all listed again;
            // each listing decides its own ignored entries.
            if navigation.tree.take_ignore_sources_changed() {
                tracing::debug!("ignore rules changed; listing every shown directory again");
                navigation.directories.changed_all(&navigation.shown);
            }
            if was_busy && !navigation.reader.is_busy() {
                let completed = navigation.reading.take();
                let errored_there = match navigation.directory_error.as_ref() {
                    Some((path, _message)) => Some(path) == completed.as_ref(),
                    None => false,
                };
                if changed && errored_there {
                    navigation.directory_error = None;
                    return true;
                }
            }
            return changed && navigation.tree.rows() != navigation.rows;
        }
        Err(error) => {
            // A normal read failure consumed its reply; an unexpected disconnect leaves no usable reader.
            if navigation.reader.is_busy() || navigation.reading.is_none() {
                navigation.reader_available = false;
            }
            let path = match navigation.reading.take() {
                Some(reading) => reading,
                None => navigation.workspace.root().to_path_buf(),
            };
            let retained = if navigation.tree.has_listing(&path) {
                "The last directory snapshot is retained."
            } else {
                "No directory snapshot is available."
            };
            let message = format!(
                "{error:#}. {retained} Restore access to this directory or restart the application."
            );
            if navigation.directory_error.as_ref() == Some(&(path.clone(), message.clone())) {
                return false;
            }
            tracing::warn!(path = %path.display(), %error, "project directory read failed");
            navigation.directory_error = Some((path, message));
            return true;
        }
    }
}

/// Fill one available read slot:
///  first listings of expanded folders,
///  then notified folders,
/// then unwatched folders on the old 500 ms round robin,
///  then the safety sweep.
/// A folder whose first listing failed is retried at most every 500 ms.
fn schedule(navigation: &mut Navigation) -> Result<()> {
    if !navigation.reader_available || navigation.reader.is_busy() {
        return Ok(());
    }
    let cooling = match navigation.last_read {
        Some(time) => time.elapsed() < Duration::from_millis(500),
        None => false,
    };
    let missing = navigation.tree.missing_listings();
    let mut next_missing = None;
    for path in missing.iter() {
        let failed_there = match navigation.directory_error.as_ref() {
            Some((failed, _message)) => failed == path,
            None => false,
        };
        if !cooling || !failed_there {
            next_missing = Some(path);
            break;
        }
    }
    let target = if let Some(path) = next_missing {
        path.clone()
    } else {
        let next =
            navigation
                .directories
                .next(&navigation.shown, &navigation.watched, Instant::now());
        // Nothing is due: no notification, no unwatched folder on its timer, no sweep in progress.
        let Some(path) = next else {
            return Ok(());
        };
        path
    };
    if navigation.reader.request(&mut navigation.tree, &target)? {
        navigation.reading = Some(target);
        navigation.last_read = Some(Instant::now());
    }
    return Ok(());
}

/// Apply current results before scheduling more work,
///  keeping source selection independent of tree reads.
pub(super) fn update(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    shared: &Rc<RefCell<Navigation>>,
) {
    let mut navigation = shared.borrow_mut();
    let changed = directory_reply(&mut navigation);
    search::update(window, &mut navigation.search);
    if navigation.opener.has_pending() {
        match navigation.opener.poll() {
            Ok(Some(opened)) => {
                if let Err(error) = open::apply(window, source, &mut navigation, opened) {
                    open::failed(window, source, format!("{error:#}"));
                }
            }
            Ok(None) => {}
            Err(error) => {
                navigation.pending_line = None;
                open::failed(
                    window,
                    source,
                    format!("{error:#}. Choose a readable UTF-8 file inside this project."),
                );
            }
        }
    }
    // A missing reveal target waits for a directory change, not another identical model every 20ms.
    // Initial opens and explicit shortcuts already publish once at their action boundary.
    if changed {
        present::update(window, source, &mut navigation);
    }
    watch::update(source, &mut navigation);
    watch::scrolled(window, &mut navigation, Instant::now());
    if let Err(error) = schedule(&mut navigation) {
        tracing::warn!(%error, "cannot schedule project directory read");
        navigation.reader_available = false;
        navigation.directory_error = Some((
            navigation.workspace.root().to_path_buf(),
            format!("{error:#}. Restart the application to resume directory reads."),
        ));
        present::update(window, source, &mut navigation);
    }
}
