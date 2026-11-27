//! What:
//! The routes an `install` argument can add:
//! the request prefix that selects one,
//! the route the entry gets,
//! where that route's binaries come from,
//! and the plan entry whose installer the route runs.
//! Why:
//! This table is the whole answer to "which routes can an argument add";
//! a prefix outside it is refused with the reason a request cannot describe the route,
//! and a plan without the route's own installer is refused too,
//! because the entry would be added and never install.
//! A caller imports from `install_argument` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type BinsFrom = 'crate-registry' | 'npm-registry' | 'go-module-path';
//! export type AddableRoute = { prefix: string; route: Route; binsFrom: BinsFrom; installsWith?: string };
//! export const ADDABLE_ROUTES: AddableRoute[];
//! function addableRoute(source: string): AddableRoute | undefined;
//! ```

/// Imports the route a plan entry takes.
use super::super::model::Route;
/// Where the binaries of a tool a request describes come from.
/// Why:
/// `bins` is what the exposure directory links,
/// so a new entry without it exposes nothing;
/// each addable route reads the names from the source that owns them,
/// never from a guess about the tool's name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinsFrom {
    /// crates.io's own `bin_names` for the crate's version.
    CrateRegistry,
    /// npm's `bin` field of the package's manifest.
    NpmRegistry,
    /// The last segment of the Go module path,
    /// which is the name `go install` writes.
    GoModulePath,
}

/// What:
/// One route a request's prefix can add:
/// the prefix,
/// the route it selects,
/// where that route's binaries come from,
/// and the plan entry its installer needs.
/// Why:
/// This table is the whole answer to "which routes can an argument add";
/// a prefix outside it is refused with the reason a request cannot describe it,
/// and a plan without the route's own installer in it is refused too,
/// because the entry would be added and never install.
pub struct AddableRoute {
    /// The request prefix that selects this route (`cargo:`).
    pub prefix: &'static str,
    /// The route the entry gets.
    pub route: Route,
    /// Where the entry's `bins` come from.
    pub bins_from: BinsFrom,
    /// The plan entry whose installer this route runs,
    /// or `None` when the route starts the program the machine already has.
    pub installs_with: Option<&'static str>,
}

/// The routes a request's prefix can add,
/// in the order the refusal message lists them.
pub const ADDABLE_ROUTES: [AddableRoute; 3] = [
    // `cargo binstall` needs no plan entry: the crates it installs come from the `cargo` on the machine.
    AddableRoute {
        prefix: "cargo:",
        route: Route::CargoBinstall,
        bins_from: BinsFrom::CrateRegistry,
        installs_with: None,
    },
    AddableRoute {
        prefix: "npm:",
        route: Route::MpmPnpm,
        bins_from: BinsFrom::NpmRegistry,
        installs_with: Some("pnpm"),
    },
    AddableRoute {
        prefix: "go:",
        route: Route::GoInstall,
        bins_from: BinsFrom::GoModulePath,
        installs_with: Some("go"),
    },
];

/// The addable route `source` selects,
/// or `None`.
pub(super) fn addable_route(source: &str) -> Option<&'static AddableRoute> {
    for route in &ADDABLE_ROUTES {
        if source.starts_with(route.prefix) {
            return Some(route);
        }
    }
    return None;
}
