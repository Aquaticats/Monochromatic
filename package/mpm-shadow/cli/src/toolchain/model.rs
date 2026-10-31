//! What:
//! The data model the plan file and the lock file share:
//! one tool's declaration (what a person writes in
//!           `toolchain.plan.jsonc`),
//! the same declaration plus its resolved version and artifacts (a lock entry),
//! and
//!           the decoding of both from JSON values with a message naming the path of every wrong field.
//! Why:
//! The lock carries every declaration field,
//! so one decoder serves both files and "plan and lock agree"
//!           becomes plain equality of two `Declaration` values.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // model.ts
//! export type Declaration = { id: string; request: Request; route: Route; bins: string[]; options: Options; ... };
//! export type LockedTool = { declaration: Declaration; version: string; artifacts: Artifact[] };
//! export function decodeDeclaration(id: string, value: unknown): Declaration { ... }
//! ```

//region Request, route and OS family

/// Re-exports the request types,
/// so callers keep importing them from `model`.
pub use super::request::{Origin, Request};

/// What:
/// The install route of one tool:
/// which provider installs it and which resolver finds its versions.
/// Why:
/// Ten values,
/// the same ones the TypeScript plan used,
/// so the committed lock keeps its `route` strings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Route = 'bin-sourced' | 'archive' | 'cargo-binstall' | 'cargo-git' | 'mpm-pnpm' | 'mpm-uvx' | 'go-install'
///   | 'rustup-toolchain' | 'rustup-components' | 'android-sdk';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Route {
    /// Installed with the `bin` CLI from a GitHub release.
    BinSourced,
    /// Downloaded,
    /// verified against the lock's SHA-256 and extracted.
    Archive,
    /// `cargo binstall --version <locked> <crate>`.
    CargoBinstall,
    /// `cargo install --git <url> --rev <rev>`.
    CargoGit,
    /// `pnpm add --global <package>@<locked>`.
    MpmPnpm,
    /// `uv tool install` of a package or a verified wheel.
    MpmUvx,
    /// `go install <module>@<locked>` into a scoped `GOBIN`.
    GoInstall,
    /// `rustup toolchain install <locked channel>`.
    RustupToolchain,
    /// The component and target step of a Rust toolchain (not a route a plan entry normally names).
    RustupComponents,
    /// Android command-line tools plus `sdkmanager` packages.
    AndroidSdk,
}

/// Every route with its lock spelling,
/// in declaration order;
/// `Route::parse` and `Route::as_str` read this one table.
pub(crate) const ROUTE_NAMES: [(Route, &str); 10] = [
    (Route::BinSourced, "bin-sourced"),
    (Route::Archive, "archive"),
    (Route::CargoBinstall, "cargo-binstall"),
    (Route::CargoGit, "cargo-git"),
    (Route::MpmPnpm, "mpm-pnpm"),
    (Route::MpmUvx, "mpm-uvx"),
    (Route::GoInstall, "go-install"),
    (Route::RustupToolchain, "rustup-toolchain"),
    (Route::RustupComponents, "rustup-components"),
    (Route::AndroidSdk, "android-sdk"),
];

/// The methods of `Route`.
impl Route {
    /// The lock spelling of the route (`"bin-sourced"`).
    pub fn as_str(self) -> &'static str {
        for (route, name) in ROUTE_NAMES {
            if route == self {
                return name;
            }
        }
        // Every variant is in the table; the test `every_route_has_a_name` proves it.
        return "unknown";
    }

    /// The route spelled `text`,
    /// or `None` for an unknown spelling.
    pub fn parse(text: &str) -> Option<Route> {
        for (route, name) in ROUTE_NAMES {
            if name == text {
                return Some(route);
            }
        }
        return None;
    }
}

/// The three operating-system families a plan entry can be limited to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OsFamily {
    /// `"linux"`.
    Linux,
    /// `"macos"`.
    Macos,
    /// `"windows"`.
    Windows,
}

/// The methods of `OsFamily`.
impl OsFamily {
    /// The lock spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            OsFamily::Linux => return "linux",
            OsFamily::Macos => return "macos",
            OsFamily::Windows => return "windows",
        }
    }

    /// The family spelled `text`,
    /// or `None`.
    pub fn parse(text: &str) -> Option<OsFamily> {
        match text {
            "linux" => return Some(OsFamily::Linux),
            "macos" => return Some(OsFamily::Macos),
            "windows" => return Some(OsFamily::Windows),
            _ => return None,
        }
    }
}

//endregion Request, route and OS family

//region Options

/// A symlink made right after installs:
/// `link_name` in `link_dir` pointing at the tool's `source_bin`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostinstallLink {
    /// Declared bin of the tool the link points at.
    pub source_bin: String,
    /// File name of the link.
    pub link_name: String,
    /// Directory holding the link,
    /// worktree-relative or absolute;
    /// `None` means the scope's own link directory.
    pub link_dir: Option<String>,
}

/// A command run after every tool is installed and the exposure directory is rebuilt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostinstallRun {
    /// Declared bin of the tool that is started.
    pub source_bin: String,
    /// Arguments passed to it.
    pub args: Vec<String>,
}

/// One postinstall step:
/// a link or a command (`when` is always `"always"`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Postinstall {
    /// An atomic symlink.
    Link(
        /// The link to make.
        PostinstallLink,
    ),
    /// A command run from the scope root.
    Run(
        /// The command to run.
        PostinstallRun,
    ),
}

/// Bootstrap metadata of the pinned `mpm` and `bin` prerequisites.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bootstrap {
    /// Tool that must be installed before this one.
    pub requires_tool: Option<String>,
    /// `"wheel"` or `"standalone-binary"`.
    pub distribution: String,
}

/// What:
/// The provider options of one tool;
/// every field is optional and absent means "not set".
/// Why:
/// Typed fields replace string lookups in the providers;
/// `Declaration` equality compares them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Options = { rustComponents?: string[]; rustTargets?: string[]; cargoFeatures?: string[]; ... };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Options {
    /// `rustup component add` names.
    pub rust_components: Option<Vec<String>>,
    /// `rustup target add` triples.
    pub rust_targets: Option<Vec<String>>,
    /// `cargo binstall --features` values.
    pub cargo_features: Option<Vec<String>>,
    /// Release-asset glob that picks one asset when a repository's asset names are ambiguous.
    pub asset: Option<String>,
    /// Packages whose install scripts `pnpm add --global` may run.
    pub allow_builds: Option<Vec<String>>,
    /// `sdkmanager` packages (part of the lock's frozen surface).
    pub android_packages: Option<Vec<String>>,
    /// The postinstall step.
    pub postinstall: Option<Postinstall>,
    /// Extra exposure names linking another declared bin:
    /// alias to primary.
    pub bin_aliases: Option<Vec<(String, String)>>,
    /// Bootstrap metadata.
    pub bootstrap: Option<Bootstrap>,
}

/// The methods of `Options`.
impl Options {
    /// Whether the tool is a wheel bootstrap (installed from a verified wheel rather than by name).
    pub fn is_wheel(&self) -> bool {
        return match &self.bootstrap {
            Some(bootstrap) => bootstrap.distribution == "wheel",
            None => false,
        };
    }
}

//endregion Options

//region Declaration and lock entry

/// What:
/// One tool as a person declares it:
/// everything except the resolved version and artifacts.
/// Why:
/// The plan holds a list of these;
/// the lock holds the same values beside the resolution,
/// so the two files
///           agree exactly when the lists are equal (apart from the resolution fields).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Declaration = { id: string; request: Request; route: Route; manager?: string; packageId?: string;
///   bins: string[]; os?: OsFamily[]; env: Record<string, string>; options: Options };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    /// Stable tool id (`node`,
    /// `cargo:fd-find`,
    /// `npm:wrangler`).
    pub id: String,
    /// What version is asked for.
    pub request: Request,
    /// How it is installed.
    pub route: Route,
    /// Manager identity (`cargo`,
    /// `uvx`,
    /// `rustup`),
    /// when one applies.
    pub manager: Option<String>,
    /// Package id under the manager (crate,
    /// npm or PyPI name,
    /// rustup channel),
    /// when it differs from the tool id.
    pub package_id: Option<String>,
    /// Binary names the exposure directory links.
    pub bins: Vec<String>,
    /// OS families the tool installs on;
    /// `None` means every family.
    pub os: Option<Vec<OsFamily>>,
    /// Environment variables the tool contributes (name to literal value),
    /// in written order.
    pub env: Vec<(String, String)>,
    /// Provider options.
    pub options: Options,
}

/// One platform's download:
/// the URL and the SHA-256 of the file behind it (lowercase hex).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    /// Download URL.
    pub url: String,
    /// SHA-256 of the file,
    /// 64 lowercase hex characters.
    pub sha256: String,
}

/// What:
/// One lock entry:
/// the declaration,
/// the resolved version,
/// and the per-platform artifact rows in written order.
/// Why:
/// `update` writes the version and artifacts;
/// `lock` rewrites only the declaration half.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockedTool = { declaration: Declaration; version: string; artifacts: [platform: string, Artifact][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockedTool {
    /// The declaration half,
    /// equal to the plan's entry while the lock is current.
    pub declaration: Declaration,
    /// The resolved version the tool is installed at.
    pub version: String,
    /// Platform id (`linux-x64`) to artifact,
    /// in written order.
    pub artifacts: Vec<(String, Artifact)>,
}

//endregion Declaration and lock entry


/// Re-exports the decoding functions,
/// so callers keep importing them from `model`.
pub use super::model_decode_support::DecodeResult;

/// Re-exports the decoders.
pub use super::model_decode::{decode_lock_entry, decode_plan_entry, is_sha256_hex};

/// Re-exports the encoding functions,
/// so callers keep importing them from `model`.
pub use super::model_encode::{lock_entry_value, request_value};


//region Version placeholder

/// The text a plan writes where the tool's resolved version belongs inside a field that must carry it.
pub const VERSION_PLACEHOLDER: &str = "{version}";

/// What:
/// A copy of the declaration with `{version}` replaced by `version` in `packageId` and in every `env` value.
/// Why:
/// A Rust channel's package id (`nightly-2026-10-30`) and a tool's home variable (`android-sdk/23.0`) equal the
///           resolved version,
/// which changes on every update.
/// The plan writes `{version}` once,
/// the lock holds the
///           expanded text,
/// and comparing a plan with a lock expands the plan with the lock's version first,
/// so an
///           update never makes the two files disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function expandVersion(declaration: Declaration, version: string): Declaration
/// ```
pub fn expand_version(declaration: &Declaration, version: &str) -> Declaration {
    let mut expanded: Declaration = declaration.clone();
    if let Some(text) = &declaration.package_id {
        expanded.package_id = Some(text.replace(VERSION_PLACEHOLDER, version));
    }
    let mut env: Vec<(String, String)> = Vec::new();
    for (name, text) in &declaration.env {
        env.push((name.clone(), text.replace(VERSION_PLACEHOLDER, version)));
    }
    expanded.env = env;
    return expanded;
}

//endregion Version placeholder
