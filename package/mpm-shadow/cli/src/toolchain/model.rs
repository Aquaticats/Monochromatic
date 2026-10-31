//! What:     The data model the plan file and the lock file share: one tool's declaration (what a person writes in
//!           `toolchain.plan.jsonc`), the same declaration plus its resolved version and artifacts (a lock entry), and
//!           the decoding of both from JSON values with a message naming the path of every wrong field.
//! Why:      The lock carries every declaration field, so one decoder serves both files and "plan and lock agree"
//!           becomes plain equality of two `Declaration` values.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // model.ts
//! export type Declaration = { id: string; request: Request; route: Route; bins: string[]; options: Options; ... };
//! export type LockedTool = { declaration: Declaration; version: string; artifacts: Artifact[] };
//! export function decodeDeclaration(id: string, value: unknown): Declaration { ... }
//! ```

/// What:     Imports the JSON value types from `serde_json` (siblings: the standard library has no JSON). `Map` keeps
///           the key order of the input because the crate enables the `preserve_order` feature.
/// Why:      Both files are JSON documents; the decoder walks `Value`s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown; type Map = Record<string, unknown>;
/// ```
use serde_json::{Map, Value};

//region Request, route and OS family

/// What:     How a person asks for a tool's version: the newest (`Latest`), one exact version (`Pinned`), or a named
///           channel (`Channel`: an npm dist-tag, a rustup channel, a Java line).
/// Why:      `Pinned` is never moved by `update`; the other two are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Request = { track: 'latest' } | { pinned: string } | { channel: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Request {
    /// `{"track": "latest"}`.
    Latest,
    /// `{"pinned": "<version>"}`.
    Pinned(String),
    /// `{"channel": "<name>"}`.
    Channel(String),
}

/// What:     The install route of one tool: which provider installs it and which resolver finds its versions.
/// Why:      Ten values, the same ones the TypeScript plan used, so the committed lock keeps its `route` strings.
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
    /// Downloaded, verified against the lock's SHA-256 and extracted.
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

/// Every route with its lock spelling, in declaration order; `Route::parse` and `Route::as_str` read this one table.
const ROUTE_NAMES: [(Route, &str); 10] = [
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

    /// The route spelled `text`, or `None` for an unknown spelling.
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

impl OsFamily {
    /// The lock spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            OsFamily::Linux => return "linux",
            OsFamily::Macos => return "macos",
            OsFamily::Windows => return "windows",
        }
    }

    /// The family spelled `text`, or `None`.
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

/// A symlink made right after installs: `link_name` in `link_dir` pointing at the tool's `source_bin`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostinstallLink {
    /// Declared bin of the tool the link points at.
    pub source_bin: String,
    /// File name of the link.
    pub link_name: String,
    /// Directory holding the link, worktree-relative or absolute; `None` means the scope's own link directory.
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

/// One postinstall step: a link or a command (`when` is always `"always"`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Postinstall {
    /// An atomic symlink.
    Link(PostinstallLink),
    /// A command run from the scope root.
    Run(PostinstallRun),
}

/// Bootstrap metadata of the pinned `mpm` and `bin` prerequisites.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bootstrap {
    /// Tool that must be installed before this one.
    pub requires_tool: Option<String>,
    /// `"wheel"` or `"standalone-binary"`.
    pub distribution: String,
}

/// What:     The provider options of one tool; every field is optional and absent means "not set".
/// Why:      Typed fields replace string lookups in the providers; `Declaration` equality compares them.
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
    /// Git source URL of a `cargo-git` tool.
    pub cargo_git_url: Option<String>,
    /// Release-asset glob the GitHub resolver matches.
    pub asset_pattern: Option<String>,
    /// Packages whose install scripts `pnpm add --global` may run.
    pub allow_builds: Option<Vec<String>>,
    /// `sdkmanager` packages (part of the lock's frozen surface).
    pub android_packages: Option<Vec<String>>,
    /// The postinstall step.
    pub postinstall: Option<Postinstall>,
    /// Extra exposure names linking another declared bin: alias to primary.
    pub bin_aliases: Option<Vec<(String, String)>>,
    /// Bootstrap metadata.
    pub bootstrap: Option<Bootstrap>,
}

//endregion Options

//region Declaration and lock entry

/// What:     One tool as a person declares it: everything except the resolved version and artifacts.
/// Why:      The plan holds a list of these; the lock holds the same values beside the resolution, so the two files
///           agree exactly when the lists are equal (apart from the resolution fields).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Declaration = { id: string; request: Request; route: Route; manager?: string; packageId?: string;
///   bins: string[]; os?: OsFamily[]; env: Record<string, string>; options: Options };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    /// Stable tool id (`node`, `cargo:fd-find`, `npm:wrangler`).
    pub id: String,
    /// What version is asked for.
    pub request: Request,
    /// How it is installed.
    pub route: Route,
    /// Manager identity (`cargo`, `uvx`, `rustup`), when one applies.
    pub manager: Option<String>,
    /// Package id under the manager (crate, npm or PyPI name, rustup channel), when it differs from the tool id.
    pub package_id: Option<String>,
    /// Binary names the exposure directory links.
    pub bins: Vec<String>,
    /// OS families the tool installs on; `None` means every family.
    pub os: Option<Vec<OsFamily>>,
    /// Environment variables the tool contributes (name to literal value), in written order.
    pub env: Vec<(String, String)>,
    /// Provider options.
    pub options: Options,
}

/// One platform's download: the URL and the SHA-256 of the file behind it (lowercase hex).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    /// Download URL.
    pub url: String,
    /// SHA-256 of the file, 64 lowercase hex characters.
    pub sha256: String,
}

/// What:     One lock entry: the declaration, the resolved version, and the per-platform artifact rows in written order.
/// Why:      `update` writes the version and artifacts; `lock` rewrites only the declaration half.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockedTool = { declaration: Declaration; version: string; artifacts: [platform: string, Artifact][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockedTool {
    /// The declaration half, equal to the plan's entry while the lock is current.
    pub declaration: Declaration,
    /// The resolved version the tool is installed at.
    pub version: String,
    /// Platform id (`linux-x64`) to artifact, in written order.
    pub artifacts: Vec<(String, Artifact)>,
}

//endregion Declaration and lock entry

//region Decoding

/// A decoding failure message that already names the path of the wrong field (`tools.node.bins[1]`).
pub type DecodeResult<T> = Result<T, String>;

/// The JSON kind name used in messages (`"an object"`).
fn kind_name(value: &Value) -> &'static str {
    match value {
        Value::Null => return "null",
        Value::Bool(_) => return "a boolean",
        Value::Number(_) => return "a number",
        Value::String(_) => return "a string",
        Value::Array(_) => return "an array",
        Value::Object(_) => return "an object",
    }
}

/// `value` as an object, or a message naming `path`.
fn as_object<'a>(value: &'a Value, path: &str) -> DecodeResult<&'a Map<String, Value>> {
    if let Some(map) = value.as_object() {
        return Ok(map);
    }
    return Err(format!("{path} must be an object, found {}", kind_name(value)));
}

/// `value` as text, or a message naming `path`.
fn as_text(value: &Value, path: &str) -> DecodeResult<String> {
    if let Some(text) = value.as_str() {
        return Ok(text.to_string());
    }
    return Err(format!("{path} must be a string, found {}", kind_name(value)));
}

/// `value` as a non-empty text, or a message naming `path`.
fn as_nonempty_text(value: &Value, path: &str) -> DecodeResult<String> {
    let text: String = as_text(value, path)?;
    if text.is_empty() {
        return Err(format!("{path} must not be empty"));
    }
    return Ok(text);
}

/// `value` as a list of texts, or a message naming `path` and the index of the first wrong element.
fn as_text_list(value: &Value, path: &str) -> DecodeResult<Vec<String>> {
    let Some(items) = value.as_array() else {
        return Err(format!("{path} must be an array of strings, found {}", kind_name(value)));
    };
    let mut texts: Vec<String> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        texts.push(as_nonempty_text(item, &format!("{path}[{index}]"))?);
    }
    return Ok(texts);
}

/// Fails when `map` has a key outside `allowed`, naming the key and the allowed ones.
fn reject_unknown_keys(map: &Map<String, Value>, allowed: &[&str], path: &str) -> DecodeResult<()> {
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{path}.{key} is not a known field; the known fields are {}", allowed.join(", ")));
        }
    }
    return Ok(());
}

/// The request object at `path`.
fn decode_request(value: &Value, path: &str) -> DecodeResult<Request> {
    let map = as_object(value, path)?;
    if map.len() != 1 {
        return Err(format!("{path} must hold exactly one of track, pinned or channel"));
    }
    if let Some(track) = map.get("track") {
        if track.as_str() == Some("latest") {
            return Ok(Request::Latest);
        }
        return Err(format!("{path}.track must be \"latest\""));
    }
    if let Some(pinned) = map.get("pinned") {
        return Ok(Request::Pinned(as_nonempty_text(pinned, &format!("{path}.pinned"))?));
    }
    if let Some(channel) = map.get("channel") {
        return Ok(Request::Channel(as_nonempty_text(channel, &format!("{path}.channel"))?));
    }
    return Err(format!("{path} must hold exactly one of track, pinned or channel"));
}

/// The postinstall object at `path`.
fn decode_postinstall(value: &Value, path: &str) -> DecodeResult<Postinstall> {
    let map = as_object(value, path)?;
    reject_unknown_keys(map, &["when", "link", "run"], path)?;
    if map.get("when").and_then(Value::as_str) != Some("always") {
        return Err(format!("{path}.when must be \"always\""));
    }
    if let Some(link) = map.get("link") {
        let link_path: String = format!("{path}.link");
        let link_map = as_object(link, &link_path)?;
        reject_unknown_keys(link_map, &["sourceBin", "linkName", "linkDir"], &link_path)?;
        let Some(source) = link_map.get("sourceBin") else {
            return Err(format!("{link_path}.sourceBin is required"));
        };
        let Some(name) = link_map.get("linkName") else {
            return Err(format!("{link_path}.linkName is required"));
        };
        let link_dir: Option<String> = match link_map.get("linkDir") {
            Some(directory) => Some(as_nonempty_text(directory, &format!("{link_path}.linkDir"))?),
            None => None,
        };
        return Ok(Postinstall::Link(PostinstallLink {
            source_bin: as_nonempty_text(source, &format!("{link_path}.sourceBin"))?,
            link_name: as_nonempty_text(name, &format!("{link_path}.linkName"))?,
            link_dir,
        }));
    }
    if let Some(command) = map.get("run") {
        let run_path: String = format!("{path}.run");
        let run_map = as_object(command, &run_path)?;
        reject_unknown_keys(run_map, &["sourceBin", "args"], &run_path)?;
        let Some(source) = run_map.get("sourceBin") else {
            return Err(format!("{run_path}.sourceBin is required"));
        };
        let Some(arguments) = run_map.get("args") else {
            return Err(format!("{run_path}.args is required"));
        };
        // Arguments may be empty strings, so they are read with `as_text`, not `as_nonempty_text`.
        let mut args: Vec<String> = Vec::new();
        let Some(items) = arguments.as_array() else {
            return Err(format!("{run_path}.args must be an array of strings"));
        };
        for (index, item) in items.iter().enumerate() {
            args.push(as_text(item, &format!("{run_path}.args[{index}]"))?);
        }
        return Ok(Postinstall::Run(PostinstallRun {
            source_bin: as_nonempty_text(source, &format!("{run_path}.sourceBin"))?,
            args,
        }));
    }
    return Err(format!("{path} must hold a link or a run"));
}

/// The options object at `path`.
fn decode_options(value: &Value, path: &str) -> DecodeResult<Options> {
    let map = as_object(value, path)?;
    reject_unknown_keys(
        map,
        &[
            "rustComponents",
            "rustTargets",
            "cargoFeatures",
            "cargoGit",
            "assetPattern",
            "allowBuilds",
            "androidPackages",
            "postinstall",
            "binAliases",
            "bootstrap",
        ],
        path,
    )?;
    let mut options: Options = Options::default();
    if let Some(list) = map.get("rustComponents") {
        options.rust_components = Some(as_text_list(list, &format!("{path}.rustComponents"))?);
    }
    if let Some(list) = map.get("rustTargets") {
        options.rust_targets = Some(as_text_list(list, &format!("{path}.rustTargets"))?);
    }
    if let Some(list) = map.get("cargoFeatures") {
        options.cargo_features = Some(as_text_list(list, &format!("{path}.cargoFeatures"))?);
    }
    if let Some(git) = map.get("cargoGit") {
        let git_path: String = format!("{path}.cargoGit");
        let git_map = as_object(git, &git_path)?;
        reject_unknown_keys(git_map, &["url"], &git_path)?;
        let Some(url) = git_map.get("url") else {
            return Err(format!("{git_path}.url is required"));
        };
        options.cargo_git_url = Some(as_nonempty_text(url, &format!("{git_path}.url"))?);
    }
    if let Some(pattern) = map.get("assetPattern") {
        options.asset_pattern = Some(as_nonempty_text(pattern, &format!("{path}.assetPattern"))?);
    }
    if let Some(list) = map.get("allowBuilds") {
        options.allow_builds = Some(as_text_list(list, &format!("{path}.allowBuilds"))?);
    }
    if let Some(list) = map.get("androidPackages") {
        options.android_packages = Some(as_text_list(list, &format!("{path}.androidPackages"))?);
    }
    if let Some(step) = map.get("postinstall") {
        options.postinstall = Some(decode_postinstall(step, &format!("{path}.postinstall"))?);
    }
    if let Some(aliases) = map.get("binAliases") {
        let alias_path: String = format!("{path}.binAliases");
        let alias_map = as_object(aliases, &alias_path)?;
        let mut pairs: Vec<(String, String)> = Vec::new();
        for (alias, primary) in alias_map {
            pairs.push((alias.clone(), as_nonempty_text(primary, &format!("{alias_path}.{alias}"))?));
        }
        options.bin_aliases = Some(pairs);
    }
    if let Some(bootstrap) = map.get("bootstrap") {
        let boot_path: String = format!("{path}.bootstrap");
        let boot_map = as_object(bootstrap, &boot_path)?;
        reject_unknown_keys(boot_map, &["requiresTool", "distribution"], &boot_path)?;
        let Some(distribution) = boot_map.get("distribution") else {
            return Err(format!("{boot_path}.distribution is required"));
        };
        let distribution_text: String = as_text(distribution, &format!("{boot_path}.distribution"))?;
        if distribution_text != "wheel" && distribution_text != "standalone-binary" {
            return Err(format!("{boot_path}.distribution must be \"wheel\" or \"standalone-binary\""));
        }
        let requires_tool: Option<String> = match boot_map.get("requiresTool") {
            Some(tool) => Some(as_nonempty_text(tool, &format!("{boot_path}.requiresTool"))?),
            None => None,
        };
        options.bootstrap = Some(Bootstrap { requires_tool, distribution: distribution_text });
    }
    return Ok(options);
}

/// The declaration fields of the tool `id` in `map`; `extra` lists the field names the lock adds on top.
fn decode_declaration_fields(id: &str, map: &Map<String, Value>, path: &str) -> DecodeResult<Declaration> {
    let Some(request_value) = map.get("request") else {
        return Err(format!("{path}.request is required"));
    };
    let request: Request = decode_request(request_value, &format!("{path}.request"))?;
    let Some(route_value) = map.get("route") else {
        return Err(format!("{path}.route is required"));
    };
    let route_text: String = as_text(route_value, &format!("{path}.route"))?;
    let Some(route) = Route::parse(&route_text) else {
        let names: Vec<&str> = ROUTE_NAMES.iter().map(|(_, name)| return *name).collect();
        return Err(format!("{path}.route {route_text:?} is not a route; the routes are {}", names.join(", ")));
    };
    let manager: Option<String> = match map.get("manager") {
        Some(value) => Some(as_nonempty_text(value, &format!("{path}.manager"))?),
        None => None,
    };
    let package_id: Option<String> = match map.get("packageId") {
        Some(value) => Some(as_nonempty_text(value, &format!("{path}.packageId"))?),
        None => None,
    };
    let bins: Vec<String> = match map.get("bins") {
        Some(value) => as_text_list(value, &format!("{path}.bins"))?,
        None => Vec::new(),
    };
    let os: Option<Vec<OsFamily>> = match map.get("os") {
        Some(value) => {
            let names: Vec<String> = as_text_list(value, &format!("{path}.os"))?;
            let mut families: Vec<OsFamily> = Vec::new();
            for name in names {
                let Some(family) = OsFamily::parse(&name) else {
                    return Err(format!("{path}.os {name:?} must be linux, macos or windows"));
                };
                families.push(family);
            }
            Some(families)
        }
        None => None,
    };
    let mut env: Vec<(String, String)> = Vec::new();
    if let Some(value) = map.get("env") {
        let env_path: String = format!("{path}.env");
        for (name, text) in as_object(value, &env_path)? {
            env.push((name.clone(), as_text(text, &format!("{env_path}.{name}"))?));
        }
    }
    let options: Options = match map.get("options") {
        Some(value) => decode_options(value, &format!("{path}.options"))?,
        None => Options::default(),
    };
    return Ok(Declaration {
        id: id.to_string(),
        request,
        route,
        manager,
        package_id,
        bins,
        os,
        env,
        options,
    });
}

/// What:     Decodes one plan entry (`tools.<id>` in `toolchain.plan.jsonc`): the declaration fields only.
/// Why:      A plan entry that names `version` or `artifacts` is a hand edit of what `update` owns, so it is rejected
///           with the field named rather than silently ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function decodePlanEntry(id: string, value: unknown, path: string): Declaration
/// ```
pub fn decode_plan_entry(id: &str, value: &Value, path: &str) -> DecodeResult<Declaration> {
    let map = as_object(value, path)?;
    reject_unknown_keys(map, &["request", "route", "manager", "packageId", "bins", "os", "env", "options"], path)?;
    return decode_declaration_fields(id, map, path);
}

/// The artifact rows object at `path`.
fn decode_artifacts(value: &Value, path: &str) -> DecodeResult<Vec<(String, Artifact)>> {
    let map = as_object(value, path)?;
    let mut rows: Vec<(String, Artifact)> = Vec::new();
    for (platform, row) in map {
        let row_path: String = format!("{path}.{platform}");
        let row_map = as_object(row, &row_path)?;
        reject_unknown_keys(row_map, &["url", "sha256"], &row_path)?;
        let Some(url) = row_map.get("url") else {
            return Err(format!("{row_path}.url is required"));
        };
        let Some(sha256) = row_map.get("sha256") else {
            return Err(format!("{row_path}.sha256 is required"));
        };
        let sha256_text: String = as_text(sha256, &format!("{row_path}.sha256"))?;
        if !is_sha256_hex(&sha256_text) {
            return Err(format!("{row_path}.sha256 must be 64 lowercase hex characters"));
        }
        rows.push((platform.clone(), Artifact { url: as_nonempty_text(url, &format!("{row_path}.url"))?, sha256: sha256_text }));
    }
    return Ok(rows);
}

/// Whether `text` is 64 lowercase hex characters.
pub fn is_sha256_hex(text: &str) -> bool {
    return text.len() == 64 && text.bytes().all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
}

/// What:     Decodes one lock entry (`tools.<id>` in `toolchain.lock.json`): declaration fields, `version`, `artifacts`.
/// Why:      The lock is read by `lock --check`, `sync`, `status` and `update`; a malformed entry stops them all with
///           the field named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function decodeLockEntry(id: string, value: unknown, path: string): LockedTool
/// ```
pub fn decode_lock_entry(id: &str, value: &Value, path: &str) -> DecodeResult<LockedTool> {
    let map = as_object(value, path)?;
    reject_unknown_keys(
        map,
        &["request", "version", "route", "manager", "packageId", "bins", "os", "env", "options", "artifacts"],
        path,
    )?;
    let declaration: Declaration = decode_declaration_fields(id, map, path)?;
    let Some(version_value) = map.get("version") else {
        return Err(format!("{path}.version is required"));
    };
    let version: String = as_nonempty_text(version_value, &format!("{path}.version"))?;
    let artifacts: Vec<(String, Artifact)> = match map.get("artifacts") {
        Some(rows) => decode_artifacts(rows, &format!("{path}.artifacts"))?,
        None => Vec::new(),
    };
    return Ok(LockedTool { declaration, version, artifacts });
}

//endregion Decoding

//region Encoding

/// A JSON array of texts.
fn text_array(items: &[String]) -> Value {
    return Value::Array(items.iter().map(|item| return Value::String(item.clone())).collect());
}

/// The options as a JSON object with the fields in the fixed order the lock documents.
fn options_value(options: &Options) -> Value {
    let mut map: Map<String, Value> = Map::new();
    if let Some(list) = &options.rust_components {
        map.insert("rustComponents".to_string(), text_array(list));
    }
    if let Some(list) = &options.rust_targets {
        map.insert("rustTargets".to_string(), text_array(list));
    }
    if let Some(list) = &options.cargo_features {
        map.insert("cargoFeatures".to_string(), text_array(list));
    }
    if let Some(url) = &options.cargo_git_url {
        let mut git: Map<String, Value> = Map::new();
        git.insert("url".to_string(), Value::String(url.clone()));
        map.insert("cargoGit".to_string(), Value::Object(git));
    }
    if let Some(pattern) = &options.asset_pattern {
        map.insert("assetPattern".to_string(), Value::String(pattern.clone()));
    }
    if let Some(list) = &options.allow_builds {
        map.insert("allowBuilds".to_string(), text_array(list));
    }
    if let Some(list) = &options.android_packages {
        map.insert("androidPackages".to_string(), text_array(list));
    }
    if let Some(step) = &options.postinstall {
        map.insert("postinstall".to_string(), postinstall_value(step));
    }
    if let Some(aliases) = &options.bin_aliases {
        let mut alias_map: Map<String, Value> = Map::new();
        for (alias, primary) in aliases {
            alias_map.insert(alias.clone(), Value::String(primary.clone()));
        }
        map.insert("binAliases".to_string(), Value::Object(alias_map));
    }
    if let Some(bootstrap) = &options.bootstrap {
        let mut boot: Map<String, Value> = Map::new();
        if let Some(tool) = &bootstrap.requires_tool {
            boot.insert("requiresTool".to_string(), Value::String(tool.clone()));
        }
        boot.insert("distribution".to_string(), Value::String(bootstrap.distribution.clone()));
        map.insert("bootstrap".to_string(), Value::Object(boot));
    }
    return Value::Object(map);
}

/// The postinstall step as the JSON object the lock stores.
fn postinstall_value(step: &Postinstall) -> Value {
    let mut map: Map<String, Value> = Map::new();
    map.insert("when".to_string(), Value::String("always".to_string()));
    match step {
        Postinstall::Link(link) => {
            let mut inner: Map<String, Value> = Map::new();
            inner.insert("sourceBin".to_string(), Value::String(link.source_bin.clone()));
            inner.insert("linkName".to_string(), Value::String(link.link_name.clone()));
            if let Some(directory) = &link.link_dir {
                inner.insert("linkDir".to_string(), Value::String(directory.clone()));
            }
            map.insert("link".to_string(), Value::Object(inner));
        }
        Postinstall::Run(command) => {
            let mut inner: Map<String, Value> = Map::new();
            inner.insert("sourceBin".to_string(), Value::String(command.source_bin.clone()));
            inner.insert("args".to_string(), text_array(&command.args));
            map.insert("run".to_string(), Value::Object(inner));
        }
    }
    return Value::Object(map);
}

/// The request as the JSON object the plan and lock store.
pub fn request_value(request: &Request) -> Value {
    let mut map: Map<String, Value> = Map::new();
    match request {
        Request::Latest => {
            map.insert("track".to_string(), Value::String("latest".to_string()));
        }
        Request::Pinned(version) => {
            map.insert("pinned".to_string(), Value::String(version.clone()));
        }
        Request::Channel(channel) => {
            map.insert("channel".to_string(), Value::String(channel.clone()));
        }
    }
    return Value::Object(map);
}

/// What:     One lock entry as a JSON object with the keys in the fixed order `request, version, route, manager,
///           packageId, bins, os, env, options, artifacts`; absent optional fields are omitted, never written as null.
/// Why:      The order keeps the committed file's diffs small; omission matches the TypeScript lock's owner decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lockEntryValue(tool: LockedTool): Record<string, unknown>
/// ```
pub fn lock_entry_value(tool: &LockedTool) -> Value {
    let declaration: &Declaration = &tool.declaration;
    let mut map: Map<String, Value> = Map::new();
    map.insert("request".to_string(), request_value(&declaration.request));
    map.insert("version".to_string(), Value::String(tool.version.clone()));
    map.insert("route".to_string(), Value::String(declaration.route.as_str().to_string()));
    if let Some(manager) = &declaration.manager {
        map.insert("manager".to_string(), Value::String(manager.clone()));
    }
    if let Some(package_id) = &declaration.package_id {
        map.insert("packageId".to_string(), Value::String(package_id.clone()));
    }
    map.insert("bins".to_string(), text_array(&declaration.bins));
    if let Some(families) = &declaration.os {
        let names: Vec<String> = families.iter().map(|family| return family.as_str().to_string()).collect();
        map.insert("os".to_string(), text_array(&names));
    }
    let mut env: Map<String, Value> = Map::new();
    for (name, text) in &declaration.env {
        env.insert(name.clone(), Value::String(text.clone()));
    }
    map.insert("env".to_string(), Value::Object(env));
    map.insert("options".to_string(), options_value(&declaration.options));
    let mut rows: Map<String, Value> = Map::new();
    for (platform, artifact) in &tool.artifacts {
        let mut row: Map<String, Value> = Map::new();
        row.insert("url".to_string(), Value::String(artifact.url.clone()));
        row.insert("sha256".to_string(), Value::String(artifact.sha256.clone()));
        rows.insert(platform.clone(), Value::Object(row));
    }
    map.insert("artifacts".to_string(), Value::Object(rows));
    return Value::Object(map);
}

//endregion Encoding
