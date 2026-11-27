//! What:
//!  Variables that isolate a fixture's Git children from the person's own configuration,
//!  and the cache place the work those children belong to records its facts in.
//! Why:
//!   Production runners add an overlay to the inherited environment instead of clearing it,
//!  so a control passes these pairs as that overlay.
//!  The wrapper records what it learns about the real Git in a cache under `XDG_CACHE_HOME`
//!  or the home directory:
//!  the text `git version` prints,
//!  and where that Git's system configuration and attributes files are.
//!  A control's work must use disposable fixtures and never the person's
//!  (`AGENTS.md`'s THR and HOM),
//!  so the overlay names a cache place of the fixture's own.
//!  A build whose fixture named no place wrote `git-system-attributes-<digest>` under the
//!       person's home cache
//!  (issue 6571).
//! The code sits in a file of its own so `test_support.rs` stays under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const overlay = [['GIT_CONFIG_NOSYSTEM', '1'], ['GIT_CONFIG_GLOBAL', '/nonexistent-global-config'], ..., ['XDG_CACHE_HOME', cacheHome()]];
//! ```

use std::path::PathBuf;

/// What:
///  Variables that isolate a Git child started through the production runners from the
///       real system and global configuration,
///  with the fixed fixture identity,
///  and the cache place the work it belongs to records its facts in.
/// Why:
///   Production runners add an overlay to the inherited environment instead of clearing
///       it,
///  so tests pass these pairs as that overlay.
///  The wrapper records what it learns about the real Git in a cache under `XDG_CACHE_HOME`
///       or the home directory,
///  and a control's work must use disposable fixtures:
///  a build whose fixture named no place wrote `git-system-attributes-<digest>` in the
///       person's home cache,
///  which every session shares and which outlives the run
///  (issue 6571).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const overlay = [['GIT_CONFIG_NOSYSTEM', '1'], ['GIT_CONFIG_GLOBAL', '/nonexistent-global-config'], ..., ['XDG_CACHE_HOME', cacheHome()]];
/// ```
pub(crate) fn isolated_overlay() -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    // `mut` allows collecting the pairs one at a time.
    let mut pairs: Vec<(std::ffi::OsString, std::ffi::OsString)> = Vec::new();
    for (name, value) in [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ] {
        pairs.push((
            std::ffi::OsString::from(name),
            std::ffi::OsString::from(value),
        ));
    }
    pairs.push((
        std::ffi::OsString::from("XDG_CACHE_HOME"),
        cache_home().into_os_string(),
    ));
    return pairs;
}

thread_local! {
    /// The cache place this test thread's fixtures made.
    /// It is made on the first use.
    static CACHE_HOME: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  The cache place a test thread's fixtures hand the work they start:
///  one folder of that thread's own,
///  made on its first use.
/// Why:
///   The wrapper's cache is where it records facts about the real Git,
///  so a control's work must name a place of its fixtures rather than inherit the
///       person's home
///  (`AGENTS.md`'s THR and HOM).
///  One folder for the thread rather than one for each call keeps a control that
///       builds many contexts from making many folders,
///  and the fixtures' cleanup removes it when the test ends however it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cacheHome = () => (threadCacheHome ??= fixture('cache-home'));
/// ```
pub(crate) fn cache_home() -> PathBuf {
    return CACHE_HOME.with(|cell| {
        let mut place = cell.borrow_mut();
        return place
            .get_or_insert_with(|| return crate::test_support::fixture("cache-home"))
            .clone();
    });
}
