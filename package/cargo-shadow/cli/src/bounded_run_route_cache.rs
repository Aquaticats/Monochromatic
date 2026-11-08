//! What:
//! The cache of the route to a delegated control group,
//! so discovery runs once per process.
//! Why:
//! Split out of `bounded_run.rs`,
//! which runs a child process within a time bound,
//! to keep that file under the code-line limit.
//! The items are re-exported from `bounded_run`,
//! so their paths there are unchanged.

/// The route override and the route types of the test-only override.
#[cfg(test)]
use crate::bounded_run::ROUTE_OVERRIDE;
/// The settings and the detection of the route.
use crate::control_group_route_discovery::{RouteSettings, detect_route};
/// The refusal text and the default discovery that the cache uses.
use crate::bounded_run::reason_of;


















/// Imports the control group pieces for this file.
use crate::control_group::{GroupUnavailable, Route};

/// The answer of the manager,
/// kept for the life of the process.
/// It is read and written only with `try_lock`:
/// a thread that finds the cache busy asks the manager itself instead of waiting for
/// the thread that holds it (contract V6:
/// an internal lock returns at once).
pub(crate) struct RouteCache {
    /// The last answer,
    /// a route or the reason there is none.
    slot: std::sync::Mutex<Option<Result<Route, String>>>,
}

/// The methods of the type above.
impl RouteCache {
    /// What:
    /// `pub(crate) const fn new() -> RouteCache` makes an empty cache.
    /// Why:
    /// The production cache is a `static`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static new(): RouteCache
    /// ```
    pub(crate) const fn new() -> RouteCache {
        return RouteCache { slot: std::sync::Mutex::new(None) };
    }

    /// What:
    /// `pub(crate) fn get_or_discover(&self, discover: &dyn Fn() -> Result<Route, GroupUnavailable>) -> Result<Route, GroupUnavailable>`
    /// returns the cached answer,
    /// or asks `discover` and keeps its answer.
    /// Why:
    /// A busy cache is never waited for:
    /// the caller discovers on its own and keeps the answer only if the cache is free by then.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// getOrDiscover(discover: () => Route): Route  // throws GroupUnavailable
    /// ```
    pub(crate) fn get_or_discover(&self, discover: &dyn Fn() -> Result<Route, GroupUnavailable>) -> Result<Route, GroupUnavailable> {
        if let Ok(held) = self.slot.try_lock()
            && let Some(answer) = held.as_ref()
        {
            return answer.clone().map_err(crate::control_group::unavailable);
        }
        let found: Result<Route, GroupUnavailable> = discover();
        if let Ok(mut held) = self.slot.try_lock() {
            *held = Some(found.as_ref().map(Clone::clone).map_err(reason_of));
        }
        return found;
    }

    /// What:
    /// `pub(crate) fn hold_for_test<T>(&self, work: impl FnOnce() -> T) -> T` runs `work` while this thread holds the cache's own mutex.
    /// Why:
    /// A test shows that a caller finds the busy cache and answers at once,
    /// with the actual mutex held,
    /// not a discovery in flight
    /// that happens to run outside it (certification,
    /// third pass,
    /// V6).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// holdForTest<T>(work: () => T): T
    /// ```
    #[cfg(test)]
    pub(crate) fn hold_for_test<T>(&self, work: impl FnOnce() -> T) -> T {
        let guard = self.slot.lock();
        let result: T = work();
        drop(guard);
        return result;
    }
}

/// What:
/// `fn find_route() -> Result<Route, GroupUnavailable>` finds out what this host can give.
/// Why:
/// The answer of the manager does not change while the process lives,
/// so it is asked once;
/// a test asks again,
/// with its own settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findRoute(): Route  // throws GroupUnavailable
/// ```
#[cfg(not(test))]
pub(crate) fn find_route() -> Result<Route, GroupUnavailable> {
    /// The route found once for the process,
    /// so later bounded runs do not detect it again.
    static ROUTE: RouteCache = RouteCache::new();
    return ROUTE.get_or_discover(&discover_default_route);
}

/// What:
/// `fn find_route() -> Result<Route, GroupUnavailable>` is the test build's version:
/// it asks again every time,
/// with the settings of the test.
/// Why:
/// A test makes the host look like one that lacks a capability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findRoute(): Route  // throws GroupUnavailable
/// ```
#[cfg(test)]
pub(crate) fn find_route() -> Result<Route, GroupUnavailable> {
    let settings: RouteSettings = ROUTE_OVERRIDE.with(|slot| { return slot.borrow().clone(); }).unwrap_or_default();
    return detect_route(&settings);
}

/// What:
/// `pub fn use_route_settings_for_test(settings: Option<RouteSettings>)` makes the runs of this thread discover their route with
/// `settings` (`None` restores the ordinary ones).
/// Why:
/// A host that lacks `systemd-run`,
/// a reachable manager or a cgroup v2 mount is made by pointing the settings at nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function useRouteSettingsForTest(settings?: RouteSettings): void
/// ```
#[cfg(test)]
pub(crate) fn use_route_settings_for_test(settings: Option<RouteSettings>) {
    ROUTE_OVERRIDE.with(|slot| { *slot.borrow_mut() = settings; });
}

/// What:
/// `fn discover_default_route() -> Result<Route, GroupUnavailable>` detects the route with the default settings.
/// Why:
/// `get_or_discover` takes it by path instead of a closure that captured nothing.
#[cfg(not(test))]
pub(crate) fn discover_default_route() -> Result<Route, GroupUnavailable> {
    return detect_route(&RouteSettings::default());
}
