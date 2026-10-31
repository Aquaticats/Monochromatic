//! What:
//!  `monochromatic-file-enforcer` is the engine behind the `file-enforcer` command:
//! it reads the JSONC config,
//! plans
//!  every rule against a virtual view of the worktree,
//! validates the whole run,
//! publishes generated files one by one,
//!  and serves all of that over a per-worktree Unix socket.
//! Why:
//!  The engine has no command-line parsing so that other tools (meow) can link it;
//! the `file-enforcer` binary in
//!  `package/file-enforcer/cli` is a thin client over it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { runRules, startServer } from '@monochromatic-dev/file-enforcer-core-rust';
//! ```

/// What:
///  `diagnostic` holds `Diagnostic`,
/// one failure or warning with a stable code,
/// a message,
/// an optional hint,
/// and the path and rule it concerns.
/// Why:
///  Every layer reports problems in this one shape,
/// and only the output boundary turns it into an event line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class Diagnostic extends Error { code: string; hint?: string; path?: string; rule?: string }
/// ```
pub mod diagnostic;

/// What:
///  `event` holds `Event`,
/// the JSON object that each output line is,
/// and the sinks that receive events.
/// Why:
///  Output is a stream of machine-readable lines,
/// so the type that fixes their fields and order is public for the CLI and for other tools that link the engine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type Event = { type: string; code: string; message: string; hint?: string; [extra: string]: unknown };
/// export interface EventSink { emit(event: Event): void }
/// ```
pub mod event;

/// What:
///  `config_types` holds the typed form of `file-enforcer.config.jsonc`:
/// `Config` and one `RuleConfig` variant per rule kind.
/// Why:
///  The hand-written decoder produces these types and the committed JSON Schema is derived from them,
/// so the two cannot describe different configs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type Config = { $schema?: string; rules: RuleConfig[] };
/// export type RuleConfig = ConcatRule | ModeOnlyRule | MirrorRule | /* ... */;
/// ```
pub mod config_types;

/// What:
///  `config_parts` holds the pieces that several rule kinds share:
/// value expressions and the small records inside `toml-keys`,
/// `package-licenses` and `jetbrains-harper`.
/// Why:
///  Sharing one definition keeps the schema and the decoder agreeing on what an expression such as `{ "file": "AGENTS.md" }` means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type ValueExpr = string | boolean | number | ValueExpr[] | { file: ValueExpr } | { var: string } | { concat: ValueExpr[] };
/// ```
pub mod config_parts;

/// What:
///  `config_reader` holds `Reader`,
/// field-by-field accessors over a JSON value that record every problem with its `file:line:col` and pointer.
/// Why:
///  A person fixing a config should see all the mistakes at once,
/// so the readers return a harmless default after recording one and decoding carries on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class Reader { errors: Diagnostic[]; text(object, key, pointer): string; array(object, key, pointer): unknown[] }
/// ```
pub mod config_reader;

/// What:
///  `config_decode_rules` decodes each rule kind's object into its `RuleConfig` variant;
/// it is private.
/// Why:
///  The per-kind field lists live in one place,
/// next to the corpus that proves them against the schema,
/// and callers only need `decode_config`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: decodeRule(reader, value, pointer): RuleConfig | undefined
/// ```
mod config_decode_rules;

/// What:
///  `config_decode` turns a loaded `ConfigDocument` into a typed `Config`,
/// checking ids,
/// scope and path shapes.
/// Why:
///  The repository config and the per-user config accept different kinds,
/// so the scope is an argument and one decoder serves both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function decodeConfig(document: ConfigDocument, scope: 'repository' | 'user', displayPath: string): Config; // throws Diagnostic[]
/// ```
pub mod config_decode;

/// What:
///  `config_load` reads a JSONC file into a `ConfigDocument`:
/// plain JSON plus where every node sits.
/// Why:
///  Syntax errors need `file:line:col`,
/// and later decode errors need the position of the node they concern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function loadConfigFile(path: string): ConfigDocument; // throws Diagnostic
/// ```
pub mod config_load;

/// What:
///  `config_locations` holds the table that maps a JSON pointer to a line and column in the config file.
/// Why:
///  `serde_json::Value` forgets positions,
/// so the table is kept beside it and the decoder asks it for the position of each problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class ConfigLocations { valueAt(pointer: string): Location; keyAt(pointer: string): Location }
/// ```
pub mod config_locations;

/// What:
///  `path_text` holds small operations on repo-relative paths kept as text with `/` separators.
/// Why:
///  Rules and the publisher agree on what the directory of a root-level file is (the empty string) by calling one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parentDirectory(path: string): string
/// ```
pub mod path_text;

/// What:
///  `config_convert` converts the parsed JSONC tree into plain JSON and builds the location table in the same walk.
/// Why:
///  Repeated keys,
/// lone surrogates and unrepresentable numbers cannot exist in a plain JSON value,
/// so they are rejected here with their position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function convertDocument(root: JsoncValue, spans: JsoncSpans, displayPath: string): [unknown, ConfigLocations]
/// ```
pub mod config_convert;

/// What:
///  `schema` derives the JSON Schema of the config from the config types and renders it as the committed file stores it.
/// Why:
///  Editors read the schema for completion,
/// and a golden test fails when the committed file is stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function configSchema(): object
/// ```
pub mod schema;

/// What:
///  `real_fs` reads the real filesystem without following symbolic links.
/// Why:
///  A generated file must never be written through a link into another tree,
/// so every read and every ancestor check uses `lstat`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function observeFile(root: string, rel: string): RealState // uses fs.lstat
/// ```
pub mod real_fs;

/// What:
///  `view` holds `View`,
/// the virtual worktree a rule plans against:
/// real files overlaid with what earlier rules planned.
/// Why:
///  Recording every read lets the engine revalidate a plan,
/// order rules,
/// answer `explain` and tell `watch` what to watch,
/// without rules declaring inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class View { readText(rel: string): string | undefined; list(rel: string): Entry[]; modeOf(rel: string): number | undefined }
/// ```
pub mod view;

/// What:
///  `view_types` holds the planned and observed state types that `View` stores.
/// Why:
///  Drift detection and revalidation compare these states,
/// so they are public beside the view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type Planned = { kind: 'file'; content: Uint8Array; mode?: number } | { kind: 'removed' };
/// ```
pub mod view_types;

/// What:
///  `view_glob` expands a glob such as `package/*/*/Cargo.toml` against the view one segment at a time.
/// Why:
///  Expansion lists only the directories the pattern names,
/// so it is bounded,
/// deterministic and recorded for `watch`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function expandGlob(view: View, pattern: string, want: EntryKind): string[]
/// ```
pub mod view_glob;

/// What:
///  `rule` defines the `Rule` and `RuleBuilder` traits and the `Change` a rule plans.
/// Why:
///  Other crates (the dependency catalog,
/// browserslist,
/// the JetBrains XML edit) plug in by implementing these traits,
/// and the engine never names them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export interface Rule { id(): string; kind(): string; composes(): boolean; plan(view: View): RulePlan }
/// export interface RuleBuilder { build(config: RuleConfig): Rule }
/// ```
pub mod rule;

/// What:
///  `value_eval` evaluates value expressions (`file`,
/// `var`,
/// `concat` and literals) against a view and a set of bound variables.
/// Why:
///  Three rule kinds share one expression language,
/// and an unbound variable is reported with the names the rule does bind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function evaluate(view: View, expr: ValueExpr, vars: Vars): Evaluated
/// ```
pub mod value_eval;

/// What:
///  `toml_ops` edits TOML text through `toml_edit` while keeping comments,
/// order and unowned keys;
/// it is private.
/// Why:
///  Only the `toml-keys` kind needs it,
/// and keeping it private lets its helpers change without an API change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: setKey(doc, path, value); appendBlock(doc, text)
/// ```
mod toml_ops;

/// What:
///  `kind_concat` implements the `concat` kind:
/// one file from literal text,
/// other files and variables.
/// Why:
///  It is the engine's simplest rule and the one most generated files use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class ConcatRule implements Rule { plan(view: View): RulePlan }
/// ```
pub mod kind_concat;

/// What:
///  `kind_mode_only` implements the `mode-only` kind:
/// set a mode on a file that exists,
/// never create it.
/// Why:
///  Some files are written by other tools and only need their permission bits enforced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class ModeOnlyRule implements Rule { plan(view: View): RulePlan }
/// ```
pub mod kind_mode_only;

/// What:
///  `kind_mise_path_list` implements the `mise-path-list` kind:
/// a Mise config that lists the existing,
/// non-empty directories matching globs.
/// Why:
///  Mise needs an explicit list,
/// and keeping it generated means a new package directory appears without a hand edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class MisePathListRule implements Rule { plan(view: View): RulePlan }
/// ```
pub mod kind_mise_path_list;

/// What:
///  `kind_package_licenses` implements the `package-licenses` kind:
/// copy license texts into each package according to its manifest's license expression.
/// Why:
///  Published packages must carry the texts their license names,
/// and the rule prunes only the texts it owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class PackageLicensesRule implements Rule { plan(view: View): RulePlan }
/// ```
pub mod kind_package_licenses;

/// What:
///  `kind_mirror` implements the `mirror` kind:
/// make a directory an exact copy of another.
/// Why:
///  Skill and agent directories are copies of one source,
/// and the target is owned outright so stale files disappear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class MirrorRule implements Rule { plan(view: View): RulePlan }
/// ```
pub mod kind_mirror;

/// What:
///  `kind_toml_keys` implements the `toml-keys` kind:
/// guarded key assignments and append-if-absent blocks in `Cargo.toml`.
/// Why:
///  Package manifests share a policy block,
/// and the rule composes with other rules that edit the same manifest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class TomlKeysRule implements Rule { composes(): true; plan(view: View): RulePlan }
/// ```
pub mod kind_toml_keys;

/// What:
///  `registry` maps a rule kind name to the builder that makes its rules,
/// and supplies `NotWiredBuilder` for kinds no crate has linked yet.
/// Why:
///  The CLI and the server take a `RuleRegistry` by value,
/// so other crates register their kinds without the engine knowing them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export class RuleRegistry { register(kind: string, builder: RuleBuilder): void; build(config: Config): Rule[] }
/// ```
pub mod registry;

/// What:
///  `plan` runs every rule in config order against a view and validates the whole run.
/// Why:
///  A failing rule,
/// two writers that do not compose,
/// or a rule reading a later rule's output must stop the run before anything is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planRun(root: string, rules: Rule[]): PlanReport
/// ```
pub mod plan;

/// What:
///  `drift` compares the plan with the real files and names each difference.
/// Why:
///  `check` reports the differences and `write` publishes exactly them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function computeDrift(report: PlanReport): Drift[]
/// ```
pub mod drift;

/// What:
///  `publish` applies drift to disk:
/// write a hidden staging file,
/// set its mode,
/// sync,
/// rename over the target.
/// Why:
///  Each file is replaced atomically and never through a symbolic link,
/// and a run killed midway reconverges on the next run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function publish(root: string, report: PlanReport, drifts: Drift[]): Outcome
/// ```
pub mod publish;

/// What:
///  `expected_dir` writes the paired `actual` and `expected` trees that `check --expected-dir` offers to diff tools.
/// Why:
///  `diff -ru actual expected` then shows exactly what `write` would change,
/// and the directory is checked to be empty and outside the worktree first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function validateExpectedDir(dir: string, root: string): string
/// ```
pub mod expected_dir;

/// What:
///  `explain` answers which rules write or read a path.
/// Why:
///  A person facing a generated file needs to find the rule and the source that produce it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function explain(report: PlanReport, path: string): Event[]
/// ```
pub mod explain;

/// What:
///  `run` executes one command:
/// load the config,
/// plan,
/// replan while the files read change,
/// then check,
/// explain or write.
/// Why:
///  The in-process engine and the server both call `run`,
/// so a command behaves the same however it is reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function run(request: RunRequest, registry: RuleRegistry, sink: EventSink): number
/// ```
pub mod run;

/// What:
///  `run_finish` holds the last step of `check` and of `write`;
/// it is private.
/// Why:
///  It keeps `run` short,
/// and callers only need the two entry points in `run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: finishCheck(...); finishWrite(...)
/// ```
mod run_finish;

/// What:
///  `json_build` builds JSON objects from key and value pairs without the `json!` macro.
/// Why:
///  The repository's lint configuration disallows the macro's expansion,
/// and field order matters for the output lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function object(pairs: [string, unknown][]): Record<string, unknown>
/// ```
pub mod json_build;

/// What:
///  `rpc_protocol` builds and names the JSON-RPC 2.0 lines that the client and the server exchange.
/// Why:
///  Both sides share one definition of the protocol version and the error codes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const PROTOCOL_VERSION = 1; export function requestLine(id, method, params): string
/// ```
pub mod rpc_protocol;

/// What:
///  `runtime_dir` chooses the directory that holds server sockets and names each socket.
/// Why:
///  A socket must live in an owner-only directory and below the kernel's path-length limit,
/// and the name must be stable per worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runtimeDirectory(xdg: string | undefined, cache: string): string
/// ```
pub mod runtime_dir;

/// What:
///  `identity` computes the build identity and the worktree identity that a client and a server compare in the handshake.
/// Why:
///  A server built from another binary,
/// or serving a deleted-and-recreated directory,
/// must refuse and be replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function buildIdentity(): string; export function worktreeIdentity(root: string): { path: string; dev: number; ino: number }
/// ```
pub mod identity;

/// What:
///  `endpoint` resolves the socket and root of the per-worktree server and of the per-user server.
/// Why:
///  The two scopes get separate servers,
/// and each is keyed by the path it serves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function repositoryEndpoint(root: string): Endpoint; export function userEndpoint(config: string): Endpoint
/// ```
pub mod endpoint;

/// What:
///  `launch` starts a server if none is listening,
/// under a startup lock,
/// and removes a socket proven stale.
/// Why:
///  Several clients may start at once,
/// and exactly one must win and spawn the server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function startServerIfNeeded(socket: string, launch: ServerLaunch, deadline: number): StartOutcome
/// ```
pub mod launch;

/// What:
///  `server` listens on the socket,
/// hands each connection to its own thread,
/// and exits when idle,
/// when its worktree disappears,
/// or when told to.
/// Why:
///  One server per worktree serializes every write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function serve(options: ServerOptions, registry: RuleRegistry): void
/// ```
pub mod server;

/// What:
///  `server_connection` handles one client connection:
/// handshake,
/// `run`,
/// `watch` and `shutdown`.
/// Why:
///  Keeping the per-connection protocol apart from the accept loop keeps both short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function handleConnection(shared: Shared, stream: Socket): void
/// ```
pub mod server_connection;

/// What:
///  `watch` is the server side of `watch`:
/// it watches every path the last run read and re-runs when something changes.
/// Why:
///  Generated files stay current while a person edits their sources,
/// and hand edits to generated files are warned about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function watchConnection(shared: Shared, sink: RpcSink, configPath: string): void
/// ```
pub mod watch;

/// What:
///  `client` connects to a server (starting one when needed),
/// performs the handshake and sends requests.
/// Why:
///  A command is a client of the server,
/// and a connection lost mid-request is reported and never replayed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function connect(options: ClientOptions): Connection
/// ```
pub mod client;

/// What:
///  `test_support` holds scratch directories and run helpers shared by the unit tests.
/// Why:
///  Every test needs a disposable worktree and a way to run a config against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // test helpers: scratchDir(), writeFiles(root, files), runConfig(root, config, mode)
/// ```
#[cfg(test)]
mod test_support;

/// What:
///  `config_tests` checks config decoding and syntax errors.
/// Why:
///  The decoder's messages are part of the tool's contract.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './config.test';
/// ```
#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;

/// What:
///  `config_position_tests` checks that every config problem names `file:line:col` and the JSON pointer.
/// Why:
///  A wrong column or a message without a position sends a person to the wrong place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './configPosition.test';
/// ```
#[cfg(test)]
#[path = "config_position_tests.rs"]
mod config_position_tests;

/// What:
///  `kinds_tests` checks each built-in rule kind against disposable worktrees.
/// Why:
///  Each kind has branches (create,
/// update,
/// prune,
/// refuse) that only a real directory exercises.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './kinds.test';
/// ```
#[cfg(test)]
#[path = "kinds_tests.rs"]
mod kinds_tests;

/// What:
///  `run_tests` checks whole runs:
/// validation,
/// ordering,
/// replanning,
/// staging and `explain`.
/// Why:
///  These are the engine's guarantees about what is and is not written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './run.test';
/// ```
#[cfg(test)]
#[path = "run_tests.rs"]
mod run_tests;

/// What:
///  `run_failure_tests` checks the failure and edge branches of a run.
/// Why:
///  A snapshot that keeps changing,
/// an unbound variable,
/// `explain` on a directory and a non-empty `--expected-dir` each have their own code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './runFailure.test';
/// ```
#[cfg(test)]
#[path = "run_failure_tests.rs"]
mod run_failure_tests;

/// What:
///  `schema_tests` holds the golden test of the committed schema and the corpus check of the decoder against it.
/// Why:
///  The schema is derived and the decoder is hand-written,
/// so a test must catch them drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './schema.test';
/// ```
#[cfg(test)]
#[path = "schema_tests.rs"]
mod schema_tests;

/// What:
///  `server_tests` checks the server's handshake,
/// idle exit,
/// stale-socket recovery and runtime directory.
/// Why:
///  The server lifecycle only exists with a real socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './server.test';
/// ```
#[cfg(test)]
#[path = "server_tests.rs"]
mod server_tests;

/// What:
///  `server_watch_tests` checks `watch` and a server that hangs up mid-request.
/// Why:
///  Both depend on timing and on a peer that misbehaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './serverWatch.test';
/// ```
#[cfg(test)]
#[path = "server_watch_tests.rs"]
mod server_watch_tests;
