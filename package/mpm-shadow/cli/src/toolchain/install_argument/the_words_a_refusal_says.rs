//! What:
//! The sentences a person reads when `install` refuses what the arguments name:
//! the plan entry a route installs with but the plan lacks,
//! the switch that forbids writing the plan at all,
//! the route the command cannot describe from a request,
//! and the argument that is neither an id the plan holds nor a request.
//! Why:
//! Every refusal names the input the person wrote and the plan file and README section that hold the shape the
//! entry needs,
//! so the message can be acted on without reading this code.
//! A caller imports from `install_argument` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type AddRefusal = 'frozen-check' | 'offline';
//! function missingInstaller(argument: string, installer: string, label: string): Error;
//! function unaddableReason(source: string): string;
//! function neither(id: string, label: string): Error;
//! function unaddable(argument: string, source: string, label: string): Error;
//! ```

/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;

/// Imports the routes whose prefixes a refusal lists.
use super::the_routes_a_request_can_add::ADDABLE_ROUTES;
/// The sentence for a plan that lacks the tool a route installs with.
pub(super) fn missing_installer(
    argument: &str,
    installer: &str,
    label: &str,
) -> ToolchainError {
    return ToolchainError::config(format!(
        "{argument:?} was not added: its route installs with `{installer}`, and {label} holds no `{installer}` entry, so \
         the entry would be added and never install. Add the `{installer}` entry to {label} first (this repository's \
         plan holds one, in the shapes \"The plan\" shows), or run this command where the plan holds it"
    ));
}

/// The README section that shows every entry's shape,
/// named in every refusal so the person knows where to look.
const ENTRY_SHAPES: &str =
    "package/mpm-shadow/cli/README.md, section \"Adding a tool with install\"";

/// What:
/// Why a new entry cannot be added at all,
/// when the rest of the command line forbids writing one.
/// Why:
/// Both cases are refused before the plan is touched,
/// so a run that writes nothing writes nothing anywhere.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddRefusal {
    /// `install --lockfile-only --frozen-lockfile`:
    /// the CI drift check,
    /// which never changes a file.
    FrozenCheck,
    /// `install --lockfile-only --offline`:
    /// a run that resolves nothing,
    /// so a new entry would sit in the plan with no lock row.
    Offline,
}

/// The methods of `AddRefusal`.
impl AddRefusal {
    /// The sentence that says why the entry was not added and what to do instead.
    pub(super) fn reason(self) -> &'static str {
        return match self {
            AddRefusal::FrozenCheck => {
                "`--frozen-lockfile` with `--lockfile-only` is the check that writes nothing, so it cannot add a tool; \
                 run the same command without `--frozen-lockfile`, or add the entry to the plan and run `mpm-shadow update <id>`"
            }
            AddRefusal::Offline => {
                "`--offline` resolves nothing, so it cannot add a tool; run the same command without `--offline` to look the \
                 tool's version up, or add the entry to the plan by hand and run `mpm-shadow install --lockfile-only --offline`"
            }
        };
    }
}

/// What:
/// Why a request that did parse cannot be added.
/// Why:
/// The reason is the whole difference between "a prefix the command does not know" and
/// "a route the command cannot describe from a request",
/// and only the second is a design limit worth explaining.
fn unaddable_reason(source: &str) -> String {
    if source.starts_with("github:") {
        return "the `github:` request leaves two routes open (`archive` and `bin-sourced`), it names a release asset's \
                shape or an archive layout, and it cannot name the binaries"
            .to_string();
    }
    if source.starts_with("pipx:") {
        return "the `mpm-uvx` route exposes the entry points the package declares, which PyPI's metadata does not list"
            .to_string();
    }
    if source.starts_with("git:") {
        return "the `cargo-git` route also needs `packageId`, the crate to build inside the repository".to_string();
    }
    if source.starts_with("conda:") {
        return "the conda route's entry carries `options` a request cannot name".to_string();
    }
    return format!(
        "`{source}` names a product of one of the named resolvers (`node`, `go`, `zig`, `rustup`, `aws`, `rust`, \
         `android-sdk`, `java:temurin`), whose entries carry fields and options a request cannot name; `go:<module>` is \
         the addable Go form"
    );
}

/// The sentence for an argument that is neither an id the plan holds nor a request the command can add.
pub(super) fn neither(
    id: &str,
    label: &str,
) -> ToolchainError {
    let prefixes: Vec<&str> = ADDABLE_ROUTES
        .iter()
        .map(|route| return route.prefix)
        .collect();
    return ToolchainError::config(format!(
        "{id:?} is neither a tool id in {label} nor a request this command can add; a request starts with one of the \
         prefixes {}, each with an optional @<version> (the newest when none is given). To add a tool on any other route, \
         edit {label} by hand; the entry shapes are in {ENTRY_SHAPES}",
        prefixes.join(", ")
    ));
}

/// The sentence for a request whose route cannot be added from a request.
pub(super) fn unaddable(
    argument: &str,
    source: &str,
    label: &str,
) -> ToolchainError {
    return ToolchainError::config(format!(
        "{argument:?} names `{source}`, which this command cannot add: {}. Edit {label} by hand for that tool; the entry \
         shapes are in {ENTRY_SHAPES}",
        unaddable_reason(source)
    ));
}
