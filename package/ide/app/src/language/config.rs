//! The language registry:
//!  Helix's built-in definitions plus the application's overrides,
//!  built
//! in code.
//!  No workspace or user Helix configuration is ever read,
//!  because a project-supplied
//! `languages.toml` could replace a server's command.

/// The launch seam decides what is spawned for each server.
use super::launch::{LaunchRequest, ServerLaunch};
/// What the toolchain provided for the project,
///  and when it was asked.
use super::toolchain::{self, ToolchainState};
/// Errors name the operation that failed.
use anyhow::{Context, Result};
/// What:
///  `ArcSwap` is a cell holding a shared pointer that can be replaced atomically.
/// Why:
///  `helix_lsp::Registry` requires its language registry in exactly this wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const loader = { current: new Loader(config) };
/// ```
use arc_swap::ArcSwap;
/// Helix's language registry and its typed configuration.
use helix_core::syntax::{Loader, config::Configuration};
/// What:
///  `HashMap` is a key-value table;
///  `Path`/`PathBuf` are borrowed and owned filesystem
///       paths;
///  `Arc` is a thread-safe shared pointer (siblings:
///  `Rc`,
///  `Box`).
/// Why:
///  Side tables are looked up by server or language name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unavailable = new Map<string, Unavailable>();
/// ```
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

/// rust-analyzer hears about file changes from the IDE instead of watching the project itself.
mod rust_analyzer;
/// The TypeScript family is served by the project's own TypeScript 7 server.
mod typescript;

/// The application's setup of the module.
mod setup;
/// The value that carries the application's setup.
pub use setup::LanguageSetup;

/// Whether a server's program exists,
///  and why not.
mod locate;
/// Why a configured server's program cannot be used.
pub(super) use locate::Unavailable;
/// The check of a server's real program,
///  never a wrapper placed in front of it.
use locate::locate;

/// How the registry uses the toolchain.
mod tools;

/// How many request timeouts a server may take to answer `initialize`.
///
/// helix-lsp sends `initialize` with the same timeout as every request,
///  and a client whose
/// `initialize` timed out can never be used:
///  the late answer is discarded and helix-lsp does not
/// ask again.
///  A server that is merely slow to start,
///  for example on a busy machine,
///  would then
/// stay failed until the next file is displayed.
///  Chosen,
///  not measured:
///  three request timeouts
/// keep a hung server reported as failed within a minute at Helix's default of 20 seconds.
const START_FACTOR: u64 = 3;

/// What the registry remembers about one server definition besides what Helix holds.
struct Definition {
    /// Command as configured,
    ///  before resolution and before the launch policy.
    command: String,
    /// Seconds a request may take,
    ///  as configured.
    ///  `u64` is an unsigned 64-bit integer.
    /// helix-lsp is given `START_FACTOR` times this,
    ///  the start allowance,
    ///  and the worker bounds
    /// every request it sends by this value itself.
    timeout: u64,
    /// Where the server's program was found,
    ///  before the launch policy;
    ///  absent when it was not found.
    ///  A change here is what makes the registry rebuild.
    resolved: Option<PathBuf>,
    /// The launch the policy produced;
    ///  absent for an unavailable server.
    launch: Option<ServerLaunch>,
}

/// The registry plus the side tables the worker reports states from.
pub(super) struct Languages {
    /// Shared with `helix_lsp::Registry`,
    ///  which reads server definitions from it at every start.
    pub(super) loader: Arc<ArcSwap<Loader>>,
    /// Canonical project root.
    root: PathBuf,
    /// The application's setup.
    setup: LanguageSetup,
    /// Server names per language,
    ///  in configured order,
    ///  before unavailable ones were removed.
    configured: HashMap<String, Vec<String>>,
    /// Servers that cannot be started,
    ///  with the reason.
    unavailable: HashMap<String, Unavailable>,
    /// Per-server facts kept outside Helix's registry.
    definitions: HashMap<String, Definition>,
    /// What the toolchain provided for the project,
    ///  and the clock reading of the question.
    toolchain: (ToolchainState, Instant),
}

/// Everything `build` produces before it is wrapped for sharing.
struct Built {
    /// Helix's registry,
    ///  without the unavailable servers.
    loader: Loader,
    /// See `Languages::configured`.
    configured: HashMap<String, Vec<String>>,
    /// See `Languages::unavailable`.
    unavailable: HashMap<String, Unavailable>,
    /// See `Languages::definitions`.
    definitions: HashMap<String, Definition>,
}

/// What:
///  Build the registry and its side tables.
///  `&Path` and `&LanguageSetup` are lent read-only.
/// Why:
///  Definitions are assembled in code from Helix's compiled-in defaults:
///  built-in table,
///      application overrides,
///  executable resolution,
///  then the launch policy.
///  Servers that are
///      missing or refused are removed from every language so Helix can never start them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build(root: string, setup: LanguageSetup): Built
/// ```
fn build(
    root: &Path,
    setup: &LanguageSetup,
    document: Option<&Path>,
    toolchain: &ToolchainState,
) -> Result<Built> {
    // `Instant::now` reads the monotonic clock; the phases below are logged with their durations.
    let started = Instant::now();
    // The built-in `languages.toml` of the pinned Helix revision, compiled into helix-loader.
    let defaults = helix_loader::config::default_lang_config();
    let merged = match &setup.extra_languages {
        Some(text) => {
            // What: `toml::from_str` parses text into a generic value; `context` attaches a message
            //       and the trailing `?` returns the error to the caller.
            // Why: Malformed application definitions must stop the module, not silently vanish.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const extra = parseToml(text);
            // ```
            let extra: toml::Value =
                toml::from_str(text).context("Cannot parse the extra language definitions")?;
            // Depth 3 is Helix's own merge depth: language entries merge by name.
            helix_loader::merge_toml_values(defaults, extra, 3)
        }
        None => defaults,
    };
    // What: `try_into` decodes the generic value into Helix's typed `Configuration`.
    // Why: Unknown keys and wrong types are rejected here by Helix's own schema.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const configuration = ConfigurationSchema.parse(merged);
    // ```
    let mut configuration: Configuration = merged
        .try_into()
        .context("Cannot decode the language definitions")?;
    // The TypeScript launcher is found once for this document and used by the definition and the check.
    let typescript_launcher = typescript::locate(root, document);
    typescript::apply(
        &mut configuration,
        root,
        typescript_launcher.as_deref().ok(),
    );
    rust_analyzer::watch_through_client(&mut configuration);
    // Helix gives servers its own spelling of the root, when it differs from the canonical one.
    let spellings = tools::spellings(root);
    let decoded = started.elapsed();
    let mut unavailable: HashMap<String, Unavailable> = HashMap::new();
    let mut definitions: HashMap<String, Definition> = HashMap::new();
    // `iter_mut` walks the table handing out modifiable borrows of each definition.
    for (name, definition) in configuration.language_server.iter_mut() {
        let command = definition.command.clone();
        let located = locate(name, &command, root, document, toolchain);
        let resolved = located.as_ref().ok().cloned();
        let outcome = match located {
            Ok(executable) => {
                let request = LaunchRequest {
                    server: name.clone(),
                    executable,
                    args: definition.args.clone(),
                    environment: tools::server_environment(&definition.environment, toolchain),
                    settings: definition.config.clone(),
                    project_root: root.to_path_buf(),
                    project_spellings: spellings.clone(),
                    read_only_binds: tools::read_only_binds(toolchain),
                    state_root: setup.state_root.clone(),
                };
                // Calling the function pointer held in `setup.launch`; a refusal becomes `Refused`.
                (setup.launch)(&request).map_err(Unavailable::Refused)
            }
            Err(reason) => Err(reason),
        };
        let launch = match outcome {
            Ok(launch) => {
                definition.command = launch.command.clone();
                definition.args = launch.args.clone();
                definition.environment = launch.environment.clone();
                definition.config = launch.settings.clone();
                Some(launch)
            }
            Err(reason) => {
                tracing::debug!(server = %name, ?reason, "language server is unavailable");
                unavailable.insert(name.clone(), reason);
                None
            }
        };
        definitions.insert(
            name.clone(),
            Definition {
                command,
                timeout: definition.timeout,
                resolved,
                launch,
            },
        );
        // What: `saturating_mul` multiplies and stops at the largest value instead of overflowing.
        // Why: helix-lsp applies its timeout to `initialize` and to every request alike; it gets
        //      the start allowance, and the worker applies the request timeout on its own.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // definition.timeout = definition.timeout * START_FACTOR;
        // ```
        definition.timeout = definition.timeout.saturating_mul(START_FACTOR);
    }
    let located = started.elapsed();
    let mut configured: HashMap<String, Vec<String>> = HashMap::new();
    for language in configuration.language.iter_mut() {
        let mut names = Vec::new();
        for features in &language.language_servers {
            names.push(features.name.clone());
        }
        configured.insert(language.language_id.clone(), names);
        // What: only the entries of servers that are available are kept, in their order.
        // Why: `Registry::get` starts every server a language lists; an unavailable one must not be listed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // language.languageServers = language.languageServers.filter(server => !unavailable.has(server.name));
        // ```
        let listed = std::mem::take(&mut language.language_servers);
        for features in listed {
            if !unavailable.contains_key(&features.name) {
                language.language_servers.push(features);
            }
        }
    }
    let loader = Loader::new(configuration).context("Cannot build the language registry")?;
    // What: `as_millis` gives whole milliseconds; `saturating_sub` cannot go below zero.
    // Why: Building the registry resolves every server program on PATH and compiles every
    //      language's patterns; the log shows which part a slow start spent its time in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // log.debug({ definitions, unavailable, decodeMs, locateMs, loaderMs }, 'built the language registry');
    // ```
    tracing::debug!(
        definitions = definitions.len(),
        unavailable = unavailable.len(),
        decode_ms = decoded.as_millis(),
        locate_ms = located.saturating_sub(decoded).as_millis(),
        loader_ms = started.elapsed().saturating_sub(located).as_millis(),
        "built the language registry"
    );
    // `Ok(...)` is the success variant of `Result`.
    return Ok(Built {
        loader,
        configured,
        unavailable,
        definitions,
    });
}

/// Registry operations used by the worker.
impl Languages {
    /// Build the registry for one project root.
    pub(super) fn new(root: &Path, setup: LanguageSetup) -> Result<Self> {
        let toolchain = tools::resolve(root, &setup);
        let built = build(root, &setup, None, &toolchain)?;
        return Ok(Self {
            // `ArcSwap::from_pointee` wraps the registry for atomic replacement; `Arc::new` shares it.
            loader: Arc::new(ArcSwap::from_pointee(built.loader)),
            root: root.to_path_buf(),
            setup,
            configured: built.configured,
            unavailable: built.unavailable,
            definitions: built.definitions,
            toolchain: (toolchain, Instant::now()),
        });
    }

    /// What:
    ///  Server names configured for a language,
    ///  in order.
    ///  `&[String]` is a borrowed list.
    /// Why:
    ///  States are reported for every configured server,
    ///  including the ones Helix never sees.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// configured(language: string): readonly string[]
    /// ```
    pub(super) fn configured(&self, language: &str) -> &[String] {
        // An unknown language has an empty list.
        return match self.configured.get(language) {
            Some(names) => names.as_slice(),
            None => &[],
        };
    }

    /// Why a server cannot be started,
    ///  or nothing when it can.
    pub(super) fn unavailable(&self, server: &str) -> Option<&Unavailable> {
        return self.unavailable.get(server);
    }

    /// Seconds a request to the server may take;
    ///  Helix's default when the server is unknown.
    pub(super) fn timeout(&self, server: &str) -> u64 {
        return match self.definitions.get(server) {
            Some(definition) => definition.timeout,
            None => 20,
        };
    }

    /// Seconds a server may take to answer `initialize`:
    ///  `START_FACTOR` request timeouts.
    pub(super) fn start_timeout(&self, server: &str) -> u64 {
        return self.timeout(server).saturating_mul(START_FACTOR);
    }

    /// The launch the policy produced for a server,
    ///  when it is available.
    pub(super) fn launch(&self, server: &str) -> Option<&ServerLaunch> {
        // `and_then` continues into the inner `Option`; `as_ref` borrows instead of moving.
        let definition = self.definitions.get(server)?;
        return definition.launch.as_ref();
    }

    /// The application's private state directory,
    ///  when one is configured.
    pub(super) fn state_root(&self) -> Option<&Path> {
        return self.setup.state_root.as_deref();
    }

    /// The paragraphs of the notice about the toolchain,
    ///  empty unless its lookup could not serve the project.
    pub(super) fn toolchain_advisory(&self) -> Vec<String> {
        return self.toolchain.0.advisory(&self.root);
    }

    /// What:
    ///  Ask the toolchain again when `toolchain::due` says so;
    ///  returns true when its answer differs from the kept one.
    /// Why:
    ///  Asking takes tens to hundreds of milliseconds on the language thread,
    ///  so it is not done at every display.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// askToolchainAgain(language: string): boolean
    /// ```
    fn ask_toolchain_again(&mut self, language: &str) -> bool {
        let mut missing = false;
        for name in self.configured(language).iter() {
            if matches!(self.unavailable.get(name), Some(Unavailable::Missing(_))) {
                missing = true;
                break;
            }
        }
        if !toolchain::due(&self.toolchain.0, self.toolchain.1.elapsed(), missing) {
            return false;
        }
        let state = tools::resolve(&self.root, &self.setup);
        let differs = state != self.toolchain.0;
        tracing::info!(language, differs, "asked the toolchain again");
        self.toolchain = (state, Instant::now());
        return differs;
    }

    /// What:
    ///  Rebuild the registry when a language's server programs appeared,
    ///  disappeared,
    ///  or moved since it was built;
    ///  returns true when it was rebuilt.
    ///  `Option<&Path>` is the document about to be displayed.
    /// Why:
    ///  Programs are resolved when the registry is built,
    ///  so a server installed later (or
    ///      project dependencies installed after start) would otherwise stay "missing" until restart.
    ///  The TypeScript server is looked up from the document's folder,
    ///  so a document in another package can need another launcher.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refresh(language: string, document?: string): boolean
    /// ```
    pub(super) fn refresh(&mut self, language: &str, document: Option<&Path>) -> Result<bool> {
        // What the toolchain provides is asked again only when the answer may have changed: a failure
        // that may have been fixed, or a missing server that may have been installed.
        let mut changed = self.ask_toolchain_again(language);
        for name in self.configured(language) {
            let Some(definition) = self.definitions.get(name) else {
                continue;
            };
            let found = locate(
                name,
                &definition.command,
                &self.root,
                document,
                &self.toolchain.0,
            )
            .ok();
            if found != definition.resolved {
                changed = true;
            }
        }
        if !changed {
            return Ok(false);
        }
        tracing::info!(
            language,
            "rebuilding the language registry after a server program changed"
        );
        let built = build(&self.root, &self.setup, document, &self.toolchain.0)?;
        // `store` atomically replaces the registry Helix reads at its next server start.
        self.loader.store(Arc::new(built.loader));
        self.configured = built.configured;
        self.unavailable = built.unavailable;
        self.definitions = built.definitions;
        return Ok(true);
    }
}

/// Registry assembly,
///  the TypeScript override,
///  and availability are exercised on disposable projects.
#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
