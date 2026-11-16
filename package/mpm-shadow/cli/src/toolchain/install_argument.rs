//! What:
//! What an `install` argument names:
//! a tool id the plan already holds,
//! or a new tool a request describes,
//! with the route its prefix selects and the binaries that route's own source lists.
//! Why:
//! The owner's answer of 2026-11-13 is that the argument is "a request or an existing id":
//! `mpm i -g cargo:fastmod` adds and installs a new tool,
//! `mpm i -g tokei` installs the entry the plan holds under that id,
//! and a bare name that is neither is refused rather than guessed.
//! A route whose entry needs more than a request can say (a download address,
//! a checksum,
//! an archive layout,
//! the name of the crate inside a repository) cannot be added here,
//! and the refusal names the plan file and the README section that shows the entry's shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-argument.ts
//! export type Named = { kind: 'existing'; id: string } | { kind: 'new'; declaration: Declaration };
//! export function namedTools(plan: Plan, context: Context, arguments: string[]): Named[]
//! ```
//! The same file adds the entries it resolved, under the scope's OS lock, once the text is known.

/// Imports the JSON value the registries answer with.
use serde_json::Value;

/// Imports `GET` and JSON decoding from the resolver helpers,
/// and the registry name check.
use super::update_registry::registry_path_name;
use super::update_resolve::{ResolveError, get_json, text_at};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports the model types an entry is built from.
use super::model::{Declaration, Options, Request, Route};
/// Imports the plan the argument is looked up in,
/// and the reader that decodes the plan text.
use super::plan::{Plan, parse_plan};
/// Imports the splice into the plan's text.
use super::plan_insert::plan_with_entries;
/// Imports the guarded write of the plan.
use super::publish::{Revision, publish_plan, read_optional, revision_of};

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
    AddableRoute { prefix: "cargo:", route: Route::CargoBinstall, bins_from: BinsFrom::CrateRegistry, installs_with: None },
    AddableRoute { prefix: "npm:", route: Route::MpmPnpm, bins_from: BinsFrom::NpmRegistry, installs_with: Some("pnpm") },
    AddableRoute { prefix: "go:", route: Route::GoInstall, bins_from: BinsFrom::GoModulePath, installs_with: Some("go") },
];

/// The sentence for a plan that lacks the tool a route installs with.
fn missing_installer(argument: &str, installer: &str, label: &str) -> ToolchainError {
    return ToolchainError::config(format!(
        "{argument:?} was not added: its route installs with `{installer}`, and {label} holds no `{installer}` entry, so \
         the entry would be added and never install. Add the `{installer}` entry to {label} first (this repository's \
         plan holds one, in the shapes \"The plan\" shows), or run this command where the plan holds it"
    ));
}

/// The README section that shows every entry's shape,
/// named in every refusal so the person knows where to look.
const ENTRY_SHAPES: &str = "package/mpm-shadow/cli/README.md, section \"Adding a tool with install\"";

/// What:
/// What one argument names,
/// in the step before its entry is built.
/// Why:
/// A run that may not write the plan refuses at this point,
/// so it never asks a registry for the binaries of a tool it will not add.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    /// An id the plan holds.
    Existing(
        /// The tool id.
        String,
    ),
    /// A request whose route the command can add.
    Request(
        /// The request as the person wrote it,
        /// with `latest` for a range they left out.
        Request,
    ),
}

/// What one argument named.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Named {
    /// An id the plan holds.
    Existing(
        /// The tool id.
        String,
    ),
    /// A tool the plan does not hold yet,
    /// described by its request.
    New(
        /// The entry to add.
        Box<Declaration>,
    ),
}

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
    fn reason(self) -> &'static str {
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
/// What adding the arguments did.
/// Why:
/// `install` prints one line per added entry,
/// and a frozen run resolves the entries its own command line added,
/// so the ids travel back to the caller.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AddReport {
    /// The entries this run added to the plan,
    /// in the order given.
    pub added: Vec<Declaration>,
    /// The ids the plan already held,
    /// in the order given.
    pub existing: Vec<String>,
}

/// The methods of `AddReport`.
impl AddReport {
    /// The ids of the entries this run added.
    pub fn added_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for declaration in &self.added {
            ids.push(declaration.id.clone());
        }
        return ids;
    }
}

/// The addable route `source` selects,
/// or `None`.
fn addable_route(source: &str) -> Option<&'static AddableRoute> {
    for route in &ADDABLE_ROUTES {
        if source.starts_with(route.prefix) {
            return Some(route);
        }
    }
    return None;
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
fn neither(id: &str, label: &str) -> ToolchainError {
    let prefixes: Vec<&str> = ADDABLE_ROUTES.iter().map(|route| return route.prefix).collect();
    return ToolchainError::config(format!(
        "{id:?} is neither a tool id in {label} nor a request this command can add; a request starts with one of the \
         prefixes {}, each with an optional @<version> (the newest when none is given). To add a tool on any other route, \
         edit {label} by hand; the entry shapes are in {ENTRY_SHAPES}",
        prefixes.join(", ")
    ));
}

/// The sentence for a request whose route cannot be added from a request.
fn unaddable(argument: &str, source: &str, label: &str) -> ToolchainError {
    return ToolchainError::config(format!(
        "{argument:?} names `{source}`, which this command cannot add: {}. Edit {label} by hand for that tool; the entry \
         shapes are in {ENTRY_SHAPES}",
        unaddable_reason(source)
    ));
}

/// The registry answer as JSON,
/// or a failure naming the URL.
fn fetch(context: &Context, url: &str) -> Result<Value, ToolchainError> {
    return match get_json(context, url, &[]) {
        Ok(document) => Ok(document),
        Err(ResolveError::Failed(message)) => Err(ToolchainError::failure(message)),
    };
}

/// The texts of a JSON array field,
/// or an empty list when the field is absent,
/// null,
/// or not an array.
fn text_list(value: Option<&Value>) -> Vec<String> {
    let Some(entries) = value.and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        if let Some(name) = entry.as_str() {
            names.push(name.to_string());
        }
    }
    return names;
}

/// The `bin_names` of one version row in a crates.io crate document,
/// or `None` when the document does not hold that version.
fn crate_row_bin_names(document: &Value, version: &str) -> Option<Vec<String>> {
    let rows: &Vec<Value> = document.get("versions")?.as_array()?;
    for row in rows {
        if text_at(row, &["num"]) == Some(version) {
            return Some(text_list(row.get("bin_names")));
        }
    }
    return None;
}

/// The sorted,
/// de-duplicated list of `names`,
/// which is the order a new plan entry writes.
fn sorted(names: Vec<String>) -> Vec<String> {
    let mut ordered: Vec<String> = names;
    ordered.sort();
    ordered.dedup();
    return ordered;
}

/// What:
/// The binaries of a crate,
/// read from crates.io:
/// the row of the version the request asks for,
/// or of the newest stable one when it asks for `latest`.
/// Why:
/// The registry publishes a crate's `bin_names`,
/// so a person never lists them by hand;
/// a crate with no binary target has none,
/// and the caller refuses it rather than writing an entry that exposes nothing.
fn crate_bin_names(context: &Context, name: &str, range: &str) -> Result<Vec<String>, ToolchainError> {
    let path: String = match registry_path_name(name) {
        Ok(text) => text,
        Err(ResolveError::Failed(message)) => return Err(ToolchainError::config(format!("cargo:{name}: {message}"))),
    };
    let base: String = format!("https://crates.io/api/v1/crates/{path}");
    if range != "latest" {
        let version_path: String = match registry_path_name(range) {
            Ok(text) => text,
            Err(ResolveError::Failed(message)) => return Err(ToolchainError::config(format!("cargo:{name}@{range}: {message}"))),
        };
        let document: Value = fetch(context, &format!("{base}/{version_path}"))?;
        return Ok(sorted(text_list(document.get("version").and_then(|row| return row.get("bin_names")))));
    }
    let document: Value = fetch(context, &base)?;
    let Some(version) = text_at(&document, &["crate", "max_stable_version"]) else {
        return Err(ToolchainError::failure(format!(
            "{base}: crates.io reports no stable version for {name}"
        )));
    };
    if let Some(names) = crate_row_bin_names(&document, version) {
        return Ok(sorted(names));
    }
    let version_document: Value = fetch(context, &format!("{base}/{version}"))?;
    return Ok(sorted(text_list(version_document.get("version").and_then(|row| return row.get("bin_names")))));
}

/// The package's own name in an npm manifest,
/// the command name npm gives a `bin` written as one text.
fn npm_package_name(name: &str) -> String {
    return match name.rsplit('/').next() {
        Some(last) => last.to_string(),
        None => name.to_string(),
    };
}

/// What:
/// The binaries of an npm package,
/// read from the registry's manifest of the version or tag the request names
/// (`latest` when it names none).
/// Why:
/// npm publishes the command names in the package's own `bin` field,
/// so they are read rather than guessed,
/// and a package with no `bin` field has none.
fn npm_bin_names(context: &Context, name: &str, range: &str) -> Result<Vec<String>, ToolchainError> {
    let path: String = match registry_path_name(name) {
        Ok(text) => text,
        Err(ResolveError::Failed(message)) => return Err(ToolchainError::config(format!("npm:{name}: {message}"))),
    };
    let tag_path: String = match registry_path_name(range) {
        Ok(text) => text,
        Err(ResolveError::Failed(message)) => return Err(ToolchainError::config(format!("npm:{name}@{range}: {message}"))),
    };
    let manifest: Value = fetch(context, &format!("https://registry.npmjs.org/{path}/{tag_path}"))?;
    return match manifest.get("bin") {
        Some(Value::Object(entries)) => {
            let mut names: Vec<String> = Vec::new();
            for key in entries.keys() {
                names.push(key.clone());
            }
            Ok(sorted(names))
        }
        Some(Value::String(_)) => Ok(vec![npm_package_name(name)]),
        _ => Ok(Vec::new()),
    };
}

/// Whether `segment` is a Go major-version suffix (`v2`, `v10`),
/// which is a path element and not the name of a program.
fn is_major_version_suffix(segment: &str) -> bool {
    let Some(rest) = segment.strip_prefix('v') else {
        return false;
    };
    return !rest.is_empty() && rest.chars().all(|character| return character.is_ascii_digit());
}

/// Whether every character of `segment` may appear in a program name.
fn is_program_name(segment: &str) -> bool {
    if segment.is_empty() || segment.starts_with('-') || segment.contains("..") {
        return false;
    }
    return segment.chars().all(|character| return character.is_ascii_alphanumeric() || "._-".contains(character));
}

/// What:
/// The program name `go install <module>@<version>` writes:
/// the last segment of the module path,
/// as Go itself names it.
/// Why:
/// Go installs the package the path names into `GOBIN` under that last segment,
/// so it is read from the request and never looked up;
/// a path that ends in a major-version suffix names no program,
/// and the refusal asks for the package path inside the repository instead.
fn go_bin_name(module: &str) -> Result<String, ToolchainError> {
    let last: &str = match module.rsplit('/').next() {
        Some(segment) => segment,
        None => module,
    };
    if is_major_version_suffix(last) {
        return Err(ToolchainError::config(format!(
            "go:{module} ends with the major-version suffix {last:?}, which names no program; write the package inside the \
             repository instead, for example go:{module}/cmd/<program>"
        )));
    }
    if !is_program_name(last) {
        return Err(ToolchainError::config(format!(
            "go:{module} ends with {last:?}, which is not a program name; write the package path whose last segment is the \
             program `go install` writes"
        )));
    }
    return Ok(last.to_string());
}

/// The `bins` of a new entry,
/// from the source its route owns.
fn bins_for(context: &Context, route: &AddableRoute, request: &Request) -> Result<Vec<String>, ToolchainError> {
    let range: &str = &request.range;
    let names: Vec<String> = match &route.bins_from {
        BinsFrom::CrateRegistry => match request.origin() {
            super::model::Origin::Cargo(name) => crate_bin_names(context, &name, range)?,
            _ => Vec::new(),
        },
        BinsFrom::NpmRegistry => match request.origin() {
            super::model::Origin::Npm(name) => npm_bin_names(context, &name, range)?,
            _ => Vec::new(),
        },
        BinsFrom::GoModulePath => match request.origin() {
            super::model::Origin::GoModule(module) => vec![go_bin_name(&module)?],
            _ => Vec::new(),
        },
    };
    if names.is_empty() {
        return Err(ToolchainError::failure(format!(
            "{} publishes no program this plan entry could expose, so no `bins` can be filled in and nothing was added; \
             a library package is installed by another tool, not by this one",
            request.text()
        )));
    }
    return Ok(names);
}

/// What:
/// The entry a request describes:
/// its id is the request's source text,
/// its route is the one its prefix selects,
/// and its binaries are read from that route's own source.
/// Why:
/// Nothing here names a person's tool for them:
/// every field comes from the request or from the registry that publishes the program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryFromRequest(context: Context, request: Request): Declaration  // throws
/// ```
fn entry_from_request(context: &Context, request: Request) -> Result<Declaration, ToolchainError> {
    let source: String = request.source.clone();
    let Some(route) = addable_route(&source) else {
        return Err(ToolchainError::config(format!("{source} selects no route this command can add")));
    };
    let bins: Vec<String> = bins_for(context, route, &request)?;
    return Ok(Declaration {
        id: request.source.clone(),
        route: route.route,
        request,
        manager: None,
        package_id: None,
        bins,
        os: None,
        env: Vec::new(),
        options: Options::default(),
    });
}

/// What:
/// What one argument names,
/// before any entry is built or looked up:
/// an id the plan holds,
/// or a request the command can add.
/// Why:
/// An id the plan holds is installed as it is,
/// so the plan is never edited for a tool that is already declared;
/// and a run that may not write refuses here,
/// before a registry is asked anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function target(plan: Plan, argument: string, label: string): { kind: 'existing'; id: string } | { kind: 'request'; request: Request }
/// ```
pub fn target(plan: &Plan, argument: &str, label: &str) -> Result<Target, ToolchainError> {
    for declaration in &plan.tools {
        if declaration.id == argument {
            return Ok(Target::Existing(argument.to_string()));
        }
    }
    let request: Request = match Request::parse(argument) {
        Ok(parsed) => parsed,
        Err(_) => return Err(neither(argument, label)),
    };
    if addable_route(&request.source).is_none() {
        return Err(unaddable(argument, &request.source, label));
    }
    return Ok(Target::Request(request));
}

/// What:
/// What one argument named once its entry is built,
/// or why it names nothing the command can act on.
/// Why:
/// The tests read the entry this returns to prove every field comes from the request or its route's source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function named(plan: Plan, context: Context, argument: string, label: string): Named
/// ```
pub fn named(plan: &Plan, context: &Context, argument: &str, label: &str) -> Result<Named, ToolchainError> {
    return match target(plan, argument, label)? {
        Target::Existing(id) => Ok(Named::Existing(id)),
        Target::Request(request) => Ok(Named::New(Box::new(entry_from_request(context, request)?))),
    };
}

/// The name a plan entry declares twice,
/// or `None`:
/// the tool id every entry shares,
/// and each binary name.
fn collision(plan: &Plan, pending: &[Declaration]) -> Option<String> {
    for declaration in pending {
        if plan.tools.iter().any(|existing| return existing.id == declaration.id) {
            return Some(declaration.id.clone());
        }
        if let Some(existing) = plan.tools.iter().find(|existing| return existing.bins.iter().any(|bin| return declaration.bins.contains(bin))) {
            return Some(format!("{} (the binary names overlap with those of {})", declaration.id, existing.id));
        }
    }
    for (index, declaration) in pending.iter().enumerate() {
        for other in &pending[index + 1..] {
            if declaration.bins.iter().any(|bin| return other.bins.contains(bin)) {
                return Some(format!("{} (the binary names overlap with those of {})", other.id, declaration.id));
            }
        }
    }
    return None;
}

/// What:
/// The fault point the real command passes:
/// nothing stops the run between the plan's write and the lock's.
/// Why:
/// `add_named_tools` takes that step as a parameter so a test can stop the command exactly where a kill would,
/// and a run that carries on says so by naming this function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const nothingStopsTheRun = () => undefined;
/// ```
pub fn nothing_stops_the_run() -> Result<(), ToolchainError> {
    return Ok(());
}

/// What:
/// Adds every argument the plan does not hold yet to the plan file,
/// and returns what each one named.
/// Why:
/// This is the write `mpm i -g <tool>` does before the install:
/// the plan is read once,
/// the entries are appended as text with everything else kept,
/// and the file is published under the scope's OS lock only if no other mpm-shadow wrote it meanwhile.
/// A request that cannot be described,
/// or a run that may not write,
/// refuses here,
/// before the plan is touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addNamedTools(context: Context, arguments: string[], refusal: AddRefusal | undefined, afterPlanWrite: () => void): AddReport
/// ```
pub fn add_named_tools<F>(
    context: &Context,
    arguments: &[String],
    refusal: Option<AddRefusal>,
    after_plan_write: F,
) -> Result<AddReport, ToolchainError>
where
    F: FnOnce() -> Result<(), ToolchainError>,
{
    let scope: &super::scope::Scope = context.scope;
    let label: String = scope.plan_file.display().to_string();
    let bytes: Option<Vec<u8>> = read_optional(&scope.plan_file)?;
    let Some(content) = bytes else {
        return Err(ToolchainError::config(format!("the toolchain plan {label} does not exist")));
    };
    let text: String = match String::from_utf8(content.clone()) {
        Ok(decoded) => decoded,
        Err(error) => return Err(ToolchainError::config(format!("{label} is not valid UTF-8: {error}"))),
    };
    let plan: Plan = parse_plan(&text, &label)?;
    let mut report: AddReport = AddReport::default();
    for argument in arguments {
        match target(&plan, argument, &label)? {
            Target::Existing(id) => {
                if !report.existing.contains(&id) {
                    report.existing.push(id);
                }
            }
            Target::Request(request) => {
                if let Some(reason) = refusal {
                    return Err(ToolchainError::config(format!("{} was not added: {}", request.text(), reason.reason())));
                }
                if report.added.iter().any(|added| return added.id == request.source) {
                    continue;
                }
                if let Some(route) = addable_route(&request.source)
                    && let Some(installer) = route.installs_with
                    && !plan.tools.iter().any(|tool| return tool.id == installer)
                {
                    return Err(missing_installer(argument, installer, &label));
                }
                report.added.push(entry_from_request(context, request)?);
            }
        }
    }
    if report.added.is_empty() {
        return Ok(report);
    }
    if let Some(name) = collision(&plan, &report.added) {
        return Err(ToolchainError::config(format!(
            "{label}: a plan holds each tool id and each binary name once, and {name} is already there, so nothing was added"
        )));
    }
    let updated: String = plan_with_entries(&text, &label, &report.added)?;
    let expected: Revision = revision_of(&Some(content));
    publish_plan(scope, &expected, &updated)?;
    after_plan_write()?;
    return Ok(report);
}
