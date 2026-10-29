# TypeSlayer 0.1.32 fails TypeScript 7.0.2 trace generation and rejects native trace events

## Symptom

Selecting any package in this workspace and asking TypeSlayer 0.1.32 to generate a trace fails before
TypeScript checks the selected project:

```text
Error [ERR_PACKAGE_PATH_NOT_EXPORTED]: Package subpath './bin/tsc' is not defined by "exports" in
/var/home/user/Monochromatic/node_modules/typescript/package.json
```

After the initial TypeScript 7 compatibility fork bypassed that error and normalized checker shards,
loading a trace for `package/fy/cli/tsconfig.json` exposed another failure:

```text
trace.json event[67] error: missing field `args`
```

The captured event is valid JSON emitted by TypeScript 7:

```json
{
  "pid": 1,
  "tid": 1,
  "ph": "X",
  "cat": "program",
  "ts": 8670.875,
  "name": "processTypeReferences",
  "dur": 1928.208
}
```

The observed environment was:

```text
TypeSlayer 0.1.32
TypeScript 7.0.2
Node.js 26.5.0
pnpm 11.15.1
```

`pnpm-workspace.yaml:143` selects TypeScript 7 for the workspace catalog:

```yaml
  'typescript': '>=7.0.2'
```

The failure applies to each selected active package because TypeSlayer deliberately runs the compiler that resolves
from the selected project.
A `createRequire()` survey from every `package/*/*/package.json` directory found that all 142 active package roots
resolve the same TypeScript 7.0.2 manifest.

## Root cause

This is a TypeSlayer 0.1.32 compatibility gap with TypeScript 7,
 not a bad `tsconfig.json` and not a missing compiler.
There are two independent incompatibilities.

### TypeSlayer requires a private TypeScript package subpath

TypeSlayer selects a package-manager-specific Node command in
`packages/typeslayer/src-tauri/src/app_data/mod.rs:201-223` at TypeSlayer commit
`436dfa8ddd10802ec41137c1d8d0f53628ca8fff`:

```rust
pub fn get_tsc_call(&self, user_flags: &str) -> TSCCommand {
    // ...
    match self.package_manager {
        PackageManager::Yarn => args.extend(["yarn", "node"]),
        PackageManager::PNPM => args.extend(["pnpm", "exec", "node"]),
        PackageManager::Bun => args.extend(["bun", "run", "node"]),
        PackageManager::NPM => args.extend(["node"]),
    };
```

The same function constructs a `require()` call to the compiler's internal `bin` path at
`packages/typeslayer/src-tauri/src/app_data/mod.rs:243-253`:

```rust
args.push("--eval");

let compiler_variant = self.settings.typescript_compiler_variant;
let compiler_require = format!(
    r#""require('{}/bin/{}')""#,
    compiler_variant.npm_package(),
    compiler_variant.as_str()
);
args.push(&compiler_require);
```

For the default compiler and this pnpm workspace,
 the resulting command begins with:

```text
pnpm exec node --eval "require('typescript/bin/tsc')" slay-gurrrl-slay
```

TypeScript 7.0.2 does contain an executable at `bin/tsc`.
Its published manifest maps the command name to that file at
`node_modules/typescript/package.json:34-36`:

```json
"bin": {
    "tsc": "./bin/tsc"
}
```

The `bin` field tells package managers which command shim to create.
 It does not make the same path importable.
TypeScript 7 also defines an export map at `node_modules/typescript/package.json:37-51`,
 and that map does not include
`./bin/tsc`:

```json
"exports": {
    "./package.json": "./package.json",
    ".": "./lib/version.cjs",
    "./unstable/sync": "./dist/api/sync/api.js",
    "./unstable/async": "./dist/api/async/api.js",
    "./unstable/fs": "./dist/api/fs.js"
}
```

Node's package-entry-point rules make unlisted subpaths private once `exports` exists.
 Node therefore rejects
TypeSlayer's package specifier before loading the compiler.
 This behavior has existed since Node 12,
 so Node 26 emits
the diagnostic but did not introduce the incompatibility.

TypeScript 6.0.2 has no `exports` field and permits the same deep import.
 This version boundary was verified directly.

### TypeScript 7 trace output does not match TypeSlayer's loader

Resolving the compiler executable by its absolute package-bin path bypasses the first failure and starts TypeScript 7.
That alone is not a complete fix.

TypeScript 7 creates one type tracer for each checker.
 At the exact 7.0.2 source tag
`typescript/v7.0.2`,
 commit `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`,
`internal/tracing/tracing.go:413-432` assigns each checker a separate file and records it in a legend:

```go
// NewTypeTracer creates a new tracer for a specific checker.
// The checkerIndex is used to create unique filenames for each checker's output.
func (tr *Tracing) NewTypeTracer(checkerIndex int) Tracer {
    // ...
    typesPath := tspath.CombinePaths(tr.traceDir, fmt.Sprintf("types_%d.json", checkerIndex))
    // ...
    tr.legend = append(tr.legend, TraceRecord{
        ConfigFilePath: tr.configFilePath,
        TracePath:      tr.tracePath,
        TypesPath:      typesPath,
        CheckerID:      checkerIndex,
    })
```

`internal/tracing/tracing.go:466-478` writes `legend.json`:

```go
// Sort legend entries by typesPath for deterministic output
slices.SortFunc(tr.legend, func(a, b TraceRecord) int {
    return strings.Compare(a.TypesPath, b.TypesPath)
})

// Write the legend file
legendPath := tspath.CombinePaths(tr.traceDir, "legend.json")
legendData, err := json.MarshalIndent(tr.legend, "", "  ")
// ...
if err := tr.fs.WriteFile(legendPath, string(legendData)); err != nil {
```

By contrast,
 TypeSlayer fixes its expected type filename to `types.json` in
`packages/typeslayer/src-tauri/src/validate/types_json.rs:9`:

```rust
pub const TYPES_JSON_FILENAME: &str = "types.json";
```

Its generation validator then reads that exact file at
`packages/typeslayer/src-tauri/src/commands/generate.rs:31-55`:

```rust
pub async fn validate_types_and_trace_async(
    outputs_dir: &str,
) -> Result<((TypesJsonSchema, usize), (Vec<TraceEvent>, usize)), String> {
    let types_path = Path::new(outputs_dir).join(TYPES_JSON_FILENAME.trim_start_matches('/'));
    // ...
    let (types_res, trace_res) = tokio::join!(
        load_types_json(types_path.clone()),
        load_trace_json(trace_path.clone())
    );
```

A TypeScript 7.0.2 trace of the fixture in the Verification section produced:

```text
legend.json
trace.json
types_0.json
types_1.json
types_2.json
types_3.json
```

Even `--singleThreaded` produced `legend.json`,
 `trace.json`,
 and `types_0.json`,
 not `types.json`.

The initial hypothesis that changing only the compiler entrypoint would restore TypeScript 7 support was therefore
incomplete.
 The package-bin resolver successfully printed `Version 7.0.2`,
 but a real trace exposed the second,
independent output-format mismatch.

### TypeScript 7 replaces the Node launcher process when possible

TypeScript 7's `node_modules/typescript/lib/tsc.js:8-19` uses `process.execve()` on non-Windows Node releases that
provide it:

```javascript
if (process.platform !== "win32" && typeof process.execve === "function") {
  try {
    process.execve(exe, [exe, ...process.argv.slice(2)]);
  } catch {
    // Fall through.
  }
}

execFileSync(exe, process.argv.slice(2), { stdio: "inherit" });
```

Successful `execve()` replaces the Node process,
 so normalization sequenced after `require()` never runs.
The fork implementation disables `process.execve` for trace generation,
 forcing the synchronous child-process fallback and preserving the parent process long enough to normalize outputs.
It also retains an exit hook for compiler launchers that call `process.exit()` directly.

### Native `processTypeReferences` events may omit `args`

TypeScript 7 intentionally starts automatic type-reference tracing with a nil argument map.
At TypeScript commit `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`,
`internal/compiler/filesparser.go:171-175` contains:

```go
func (t *parseTask) loadAutomaticTypeDirectives(loader *fileLoader) {
    if loader.opts.Tracing != nil {
        defer loader.opts.Tracing.Push(tracing.PhaseProgram, "processTypeReferences", nil, false)()
    }
```

The trace event's argument field uses zero-value omission in
`internal/tracing/tracing.go:80-89`:

```go
type traceEvent struct {
    PID  int            `json:"pid"`
    TID  int            `json:"tid"`
    PH   string         `json:"ph"`
    Cat  string         `json:"cat"`
    TS   float64        `json:"ts"`
    Name string         `json:"name,omitzero"`
    S    string         `json:"s,omitzero"`
    Dur  *float64       `json:"dur,omitzero"`
    Args map[string]any `json:"args,omitzero"`
}
```

The sampled event retains that nil map at `internal/tracing/tracing.go:318-321`:

```go
tid := tr.threadIDLocked(args)
tr.traceContent.WriteString(",\n")
tr.writeEvent(traceEvent{PID: 1, TID: tid, PH: "X", Cat: string(phase), TS: startMicros, Name: name, Dur: &dur, Args: args})
```

Upstream TypeSlayer 0.1.32 instead requires a count payload in
`packages/typeslayer/src-tauri/src/validate/trace_json.rs:513-521`:

```rust
#[serde(rename = "processTypeReferences")]
ProcessTypeReferences {
    #[serde(flatten)]
    common: EventCommon,
    cat: String,
    ph: EventPhase,
    dur: f64,
    args: CountArgs,
},
```

Serde therefore reports the missing field before TypeSlayer can ingest the trace.
The earlier hypothesis that normalization dropped `args` was wrong:
the TypeScript source passes nil,
the serializer omits zero-valued arguments,
and the captured JSON is complete.
Inventing a count would misrepresent compiler output.
The compatibility fix makes only this event's `CountArgs` optional,
which preserves TypeScript 6 counts when present and accepts TypeScript 7's omission.

## Verification

### Versions and source

- Installed TypeSlayer:
   npm package `typeslayer@0.1.32`,
   which was also the npm `latest` tag on 2026-07-25.
- TypeSlayer source:
   tag `typeslayer-v0.1.32`,
   commit
  `436dfa8ddd10802ec41137c1d8d0f53628ca8fff`.
- Installed TypeScript:
   npm package `typescript@7.0.2`,
   manifest `gitHead`
  `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`.
- TypeScript native source:
   tag `typescript/v7.0.2`,
   the same commit.
- Comparison compiler:
   npm package `typescript@6.0.2`.
- Runtime:
   `Node.js` 26.5.0.

### Source clone boundary

The initial source investigation used read-only third-party clones:

- `https://github.com/dimitropoulos/typeslayer.git`,
   checked out at tag `typeslayer-v0.1.32` under
  `$HOME/temp/agent/typeslayer-2026-07-25`;
- `https://github.com/microsoft/typescript-go.git`,
   checked out at tag `typescript/v7.0.2` under
  `$HOME/temp/agent/typescript-go-2026-07-25`;
- `https://github.com/jdx/mise.git`,
   checked out at tag `v2026.7.0`,
   commit `857b73f6a6b39a3bc90c44119a1e86ee11bd7273`,
   under `$HOME/temp/agent/mise-2026-07-25`.

The implementation used a separate clone of `https://github.com/Aquaticat/typeslayer.git` under
`$HOME/temp/agent/typeslayer-aquaticat-2026-07-25`.
That clone retained `dimitropoulos/typeslayer` as its read-only `upstream` remote;
changes were committed and pushed only to the authorized `Aquaticat/typeslayer` `origin`.

All clones were created with `gh repo clone` after preparing the private `$HOME/temp/agent` scratch root.
Fixture traces,
build artifacts,
and UI data were written under private scratch roots.

### Minimal resolver harness

From this repository:

```bash
node --eval "require('typescript/bin/tsc')"
```

Observed result with TypeScript 7.0.2:

```text
Error [ERR_PACKAGE_PATH_NOT_EXPORTED]: Package subpath './bin/tsc' is not defined by "exports"
```

The same check against a disposable TypeScript 6.0.2 installation succeeds:

```bash
npm install --ignore-scripts --save-dev --save-exact typescript@6.0.2
node --eval "require('typescript/bin/tsc')" slay-gurrrl-slay --version
```

Observed result:

```text
Version 6.0.2
```

### Trace-layout harness

Fixture source:

```typescript
// index.ts
export type Box<T> = { readonly value: T };
export const box: Box<string> = { value: "ok" };
```

Fixture configuration:

```json
// tsconfig.json
{
  "compilerOptions": {
    "noEmit": true,
    "strict": true
  },
  "include": ["index.ts"]
}
```

TypeScript 6.0.2,
 using TypeSlayer's original compiler-loading pattern:

```bash
node --eval "require('typescript/bin/tsc')" slay-gurrrl-slay \
  --noEmit --incremental false --noErrorTruncation \
  --generateTrace outputs --project tsconfig.json
find outputs -maxdepth 1 -type f -printf '%f\n' | sort
```

Observed result:

```text
trace.json
types.json
```

Both files parsed as JSON arrays.
 The fixture produced 614 trace events and 30,563 type entries.

TypeScript 7.0.2,
 invoked through its actual executable so the export-map failure cannot mask later behavior:

```bash
node node_modules/typescript/bin/tsc \
  --noEmit --incremental false --noErrorTruncation \
  --generateTrace outputs --project tsconfig.json
find outputs -maxdepth 1 -type f -printf '%f\n' | sort
```

Observed result:

```text
legend.json
trace.json
types_0.json
types_1.json
types_2.json
types_3.json
```

### Verified fork implementation

The production fix was committed and pushed to
[`Aquaticat/typeslayer`](https://github.com/Aquaticat/typeslayer) at
`0319832ff6d8bd5343371501a6b403e04acc33b2`.
The verified fork head is `ac48c6e074852fc1c4d8d6556029cf693ab6979b`,
which adds the retained-count regression test without changing production code.
The implementation:

- resolves each compiler through the executable declared in its package manifest rather than a private subpath;
- preserves legacy TypeScript 6 output unchanged;
- reads TypeScript 7's `legend.json` and checker-specific type shards;
- assigns each checker a contiguous global type-ID range and remaps every recorded type relationship;
- remaps checker-tagged trace type IDs and adapts TypeScript 7 event shapes to TypeSlayer's existing Rust schema;
- accepts a missing count on native `processTypeReferences` events while preserving counts emitted by older compilers;
- writes a synthetic `types.json` and normalized `trace.json` for the existing loader,
   analyzer,
   graph,
   raw-data,
   and query modules.

A built Tauri application was exercised through its real UI against disposable compiler fixtures:

- TypeScript 7.0.2 generated and rendered 35,333 types and 28,635 relations.
   Rust loaded 35,334 vector
  entries including its index-zero sentinel and 676 normalized trace events.
   `analyze-trace.json` and
  `type-graph.json` were generated successfully.
- TypeScript 6.0.2 generated and rendered 30,563 types and 25,479 relations.
   Its legacy `trace.json`,
  `types.json`,
   `tsc.cpuprofile`,
   `analyze-trace.json`,
   and `type-graph.json` artifacts all completed.

The captured missing-args event and its retained-count companion are regression fixtures in
`packages/typeslayer/src-tauri/src/validate/trace_json.rs:1287-1400`.
Before the schema fix,
this targeted command failed with `missing field 'args'`:

```bash
cargo test process_type_references_accepts_missing_args --jobs 1 -- --nocapture
```

After changing `ProcessTypeReferences.args` to `Option<CountArgs>`,
the targeted test passed.
A companion test supplies `{ "args": { "count": 4 } }`,
round-trips the typed event,
and confirms that the count remains four.
The complete Rust suite passed all 33 tests,
and the production Tauri release build completed.
Repository linting,
formatting,
TypeScript checks,
and all 14 JavaScript tests also passed.

The rebuilt globally linked application then loaded the exact trace that had failed at event 67.
Its log recorded `trace.json` as loaded rather than reporting a schema error.
A fresh UI generation against `package/fy/cli/tsconfig.json` loaded 341 type entries and 2,601 trace events,
generated analysis and graph artifacts,
and rendered 340 searchable types with 160 relationships.

TypeScript 7.0.2 accepts `--generateCpuProfile` but its native compiler produced no V8 profile file in direct and UI
verification.
The fork documents that SpeedScope still requires a JavaScript-based compiler;
trace analysis,
type search,
and the type graph work with TypeScript 7.

### Global fork installation through mise

Mise 2026.7.0's npm backend accepts registry version specifications,
not source URLs.
`src/backend/npm.rs:243-247` states this directly:

```rust
/// NPM installs packages from npm registry using version specs (e.g., eslint@8.0.0).
/// It doesn't support installing from direct URLs, so lockfile URLs are not applicable.
fn supports_lockfile_url(&self) -> bool {
    false
}
```

Every package-manager branch then constructs `package@version` rather than preserving a source specification.
For example,
the pnpm branch at `src/backend/npm.rs:418-424` uses:

```rust
let mut cmd = CmdLineRunner::new("pnpm")
    .arg("add")
    .arg("--global")
    .arg(format!("{}@{}", self.tool_name(), tv.version))
```

Mise's supported boundary for a locally compiled tool is `mise link`.
`src/cli/link.rs:11-13` describes that command:

```rust
/// Symlinks a tool version into mise
///
/// Use this for adding installs either custom compiled outside mise or built with a different tool.
```

After producing a release binary,
the verified global installation sequence is:

```bash
version=0.1.32-aquaticat.0319832
prefix="$HOME/.local/share/mise/linked/npm-typeslayer/$version"

mkdir --parents "$prefix/bin" "$prefix/share/typeslayer"
install --mode=0755 \
  --target-directory="$prefix/bin" \
  packages/typeslayer/src-tauri/target/release/typeslayer
printf '%s\n' 0319832ff6d8bd5343371501a6b403e04acc33b2 \
  > "$prefix/share/typeslayer/BUILD-COMMIT"

mise link --force "npm:typeslayer@$version" "$prefix"
mise config set \
  --file "$HOME/.config/mise/config.toml" \
  'tools.npm:typeslayer' \
  "$version"
mise which typeslayer
```

This keeps the registry-installed 0.1.32 directory and the earlier linked fork available for rollback while selecting
the repaired fork globally.
Verification selected the linked `0.1.32-aquaticat.0319832` version from the global mise config,
resolved `typeslayer` to that prefix,
and matched its SHA-256 digest,
`754a10a86c082edf3c0b88abd6dd9fe0c4439210f81ca97331e3b9b7aaee24df`,
to the release build.
The globally resolved command launched the production Tauri UI without a Vite development server.

The linked build is local rather than registry-reproducible:
upgrading requires rebuilding a newer fork commit,
staging it under a new version label,
and relinking it.

### Behavior catalog

Works cleanly:

- The built `Aquaticat/typeslayer` production fix at `0319832ff6d8bd5343371501a6b403e04acc33b2` with TypeScript 7.0.2
  and TypeScript 6.0.2.
- The test-only follow-up at `ac48c6e074852fc1c4d8d6556029cf693ab6979b`,
  whose complete Rust suite passes 33 tests.
- A native `processTypeReferences` event with no `args` field under the repaired Rust schema.
- TypeSlayer's `require('typescript/bin/tsc')` pattern with TypeScript 6.0.2.
- TypeScript 6.0.2 trace generation with the `trace.json` plus `types.json` layout TypeSlayer expects.
- Direct TypeScript 7.0.2 CLI invocation through `node_modules/typescript/bin/tsc`.

Fails with `missing field 'args'`:

- A TypeScript 7 trace containing a native `processTypeReferences` event without `args` under upstream 0.1.32 or the
  earlier compatibility fork at `3089dd964855c89eedf0505a3dac7a985ec51946`.

Fails with `ERR_PACKAGE_PATH_NOT_EXPORTED`:

- `require('typescript/bin/tsc')` with TypeScript 7.0.2.
- TypeSlayer 0.1.32 trace generation when its default `tsc` variant resolves TypeScript 7.0.2.

Runs the compiler but remains incompatible with TypeSlayer's loader:

- Resolving TypeScript 7's `bin.tsc` through its exported `package.json` and requiring the resulting absolute path.
- Direct TypeScript 7 trace generation,
   which writes checker-specific `types_N.json` files and `legend.json`.
- TypeScript 7 with `--singleThreaded`,
   which writes `types_0.json` rather than `types.json`.

## Verified workarounds

No end-to-end workaround for the unmodified upstream TypeSlayer 0.1.32 build was verified.
The fork implementation in the Verification section is an end-to-end verified replacement build.

### Verified compiler-process remediation, not an end-to-end workaround

Use a disposable analysis checkout or fixture whose `typescript` dependency is exactly 6.0.2:

```json
{
  "devDependencies": {
    "typescript": "6.0.2"
  }
}
```

After installing that fixture,
 the exact compiler command TypeSlayer constructs resolves and produces the two files its
loader expects.
The full compiler invocation and output layout were verified in the Trace-layout harness.
For the unmodified upstream build,
this verification stops at TypeSlayer's compiler-process and output-file boundary.
The later fork verification exercised the TypeScript 6 path through trace generation,
CPU profiling,
analysis,
graph generation,
and rendered type search.
This remains a boundary-level remediation for upstream 0.1.32 rather than an upstream end-to-end workaround.

Tradeoffs:

- The trace describes TypeScript 6 behavior and performance,
   not the native TypeScript 7 compiler.
- TypeScript 7-only configuration,
   syntax,
   or behavior can prevent the analysis checkout from compiling under 6.0.2.
- Replacing this workspace's catalog version would affect every TypeScript consumer,
   so a disposable analysis checkout
  is safer than changing the main workspace merely for TypeSlayer.

TypeSlayer's own FAQ documents importing an existing `trace.json` plus `types.json` pair through
`Raw Data | trace.json`,
followed by regenerating its analyzed trace and type graph.
That UI route was not exercised and is therefore not recorded as a verified workaround here.

## What does not work

- **Passing the fork's Git URL to mise's npm backend.
  ** Mise 2026.7.0 supports npm registry versions rather than direct source URLs.
   Build the fork and register its install prefix with `mise link` instead.
- **Selecting a different package in this monorepo.
  ** The selected packages still resolve the workspace's TypeScript
  7.0.2 catalog installation.
- **Reinstalling or updating the mise tool.
  ** TypeSlayer 0.1.32 was both the installed release and npm's current
  `latest` release during verification.
- **Using another TypeScript-7-compatible Node release.
  ** This was rejected from the deciding package rules rather than a local runtime matrix.
   TypeScript 7 requires Node 16.20 or newer,
   while Node has enforced unlisted `exports` subpaths since Node 12.
   No Node version that satisfies TypeScript 7's engine requirement makes `typescript/bin/tsc` public.
- **Changing the deep import to `typescript/lib/tsc.js`.
  ** TypeScript 7 does not export that subpath either.
- **Bypassing the export map with an absolute path.
  ** This starts TypeScript 7 but exposes the unsupported
  `legend.json` plus `types_N.json` layout.
- **Adding `--singleThreaded`.
  ** This reduces the trace to one checker shard,
   but its name remains `types_0.json` and
  TypeSlayer still asks for `types.json`.
   Renaming the file was not treated as a verified workaround because TypeSlayer's
  trace and type schemas have not been validated end to end against TypeScript 7 output.
- **Treating this as a pnpm layout bug.
  ** A plain Node `require()` in a minimal fixture reproduces the same export error.
- **Treating the missing `args` field as truncation or normalizer corruption.
  ** TypeScript 7 passes nil to the trace event and uses `omitzero` for the serialized field.
- **Adding a synthetic `{ "count": 0 }` argument in the normalizer.
  ** Zero is not present in the compiler output and would turn absence into an invented measurement.

## Upstream filing artifact

### Upstream filing decision

No matching `.out-of-scope/` exemption exists.
 The TypeScript project-reference and low-impact formatter exemptions are
unrelated to TypeSlayer compiler loading or trace formats.

The upstream tracker search inspected all 26 TypeSlayer issues and all 11 pull requests visible on 2026-07-25 for
`ERR_PACKAGE_PATH_NOT_EXPORTED`,
 `TypeScript 7`,
 `types_0.json`,
 and `legend.json`.
 It found no duplicate.
 Issue
[dimitropoulos/typeslayer#28](https://github.com/dimitropoulos/typeslayer/issues/28) mentions
`typescript/bin/tsc`,
 but it covers the distinct case where TypeScript is not installed.

The six filing constraints resolve as follows:

1. **Is it really upstream's fault?
   ** Yes as a TypeSlayer compatibility gap.
    TypeSlayer hardcodes an unexported compiler
   subpath and assumes the pre-TypeScript-7 trace layout.
    It is not a TypeScript defect because TypeScript's export map
   and checker-sharded trace files are deliberate package and compiler behavior.
2. **Can upstream fix it?
   ** Yes.
    TypeSlayer can invoke the package's declared executable rather than a private subpath,
   teach its loader and graph model to consume `legend.json` and checker-specific type files,
   and model omitted native event arguments as optional.
3. **Are they supporting this use case?
   ** No for the native compiler in this release.
    TypeSlayer 0.1.32 shipped on 2026-03-23,
    before TypeScript 7.0.2 shipped on 2026-07-08.
    More directly,
    `packages/typeslayer/src/components/customize-flags-dialog.tsx:213-220` warns that the native `tsgo` compiler in
   this TypeSlayer version cannot generate the required traces and asks users not to report that combination.
    TypeScript 7 later added trace generation under the standard `tsc` package name,
    but TypeSlayer 0.1.32 contains no corresponding claim or trace-format support.
    This constraint fails for version 0.1.32.
4. **Would the repo welcome our contribution?
   ** Yes.
    The README invites contributions and imposes only its commit-message
   convention.
    `CONTRIBUTING.md`,
    the repository templates,
    and recent tracker contents contain no ban on external or
   AI-assisted reports.
5. **Will they likely fix it?
   ** Soft yes.
    No decline or non-goal was found.
    There is also no positive implementation
   signal:
    no matching issue or pull request exists,
    and the relevant TypeSlayer source has not changed since 0.1.32.
6. **Have we prototyped a minimal fix compatible with their architecture?
   ** Yes.
    The `Aquaticat/typeslayer` fork resolves declared package executables,
   normalizes checker-sharded types and trace IDs into TypeSlayer's existing internal model,
   models the native event's count as optional,
   preserves TypeScript 6,
   and passed regression tests plus built-application UI verification.

Constraint 3 still fails for the unmodified upstream 0.1.32 release,
 so default policy remains not to file the draft as a
bug report.
The fork is implementation evidence that can support a future feature contribution if upstream begins accepting
TypeScript 7 compatibility work.

### Draft issue, do not file as-is

~~~md
Title: TypeScript 7 trace generation fails at `typescript/bin/tsc` and uses an unsupported sharded output layout

Labels: bug, TypeScript 7

## Description

TypeSlayer 0.1.32 cannot generate traces for `typescript@7.0.2`.

The first failure is:

```text
Error [ERR_PACKAGE_PATH_NOT_EXPORTED]: Package subpath './bin/tsc' is not defined by "exports"
```

TypeSlayer constructs `require('typescript/bin/tsc')` in
`packages/typeslayer/src-tauri/src/app_data/mod.rs:243-253`. TypeScript 7's published export map exposes
`./package.json`, the package root, and unstable API paths, but not `./bin/tsc`.

There is a second incompatibility after resolving the executable by absolute path. TypeScript 7 creates one
`types_N.json` file per checker and records those files in `legend.json`
(`microsoft/typescript-go` `internal/tracing/tracing.go:413-432,466-478`). TypeSlayer fixes
`TYPES_JSON_FILENAME` to `types.json` and reads that exact file in
`packages/typeslayer/src-tauri/src/commands/generate.rs:31-55`.

There is a third incompatibility after normalizing that layout.
TypeScript 7 calls `processTypeReferences` tracing with nil arguments in
`internal/compiler/filesparser.go:171-175`,
and its `traceEvent.Args` field uses `json:"args,omitzero"` in
`internal/tracing/tracing.go:80-89`.
TypeSlayer requires `CountArgs` for this event in
`packages/typeslayer/src-tauri/src/validate/trace_json.rs:513-521`,
so Serde rejects the valid native event with `missing field 'args'`.

## Reproduction

With `typescript@7.0.2` installed:

```bash
node --eval "require('typescript/bin/tsc')"
```

This throws `ERR_PACKAGE_PATH_NOT_EXPORTED`.

Bypass that first failure and generate a real trace:

```bash
node node_modules/typescript/bin/tsc \
  --noEmit --incremental false --noErrorTruncation \
  --generateTrace outputs --project tsconfig.json
find outputs -maxdepth 1 -type f -printf '%f\n' | sort
```

A default run produces:

```text
legend.json
trace.json
types_0.json
types_1.json
types_2.json
types_3.json
```

`--singleThreaded` still produces `types_0.json`, not `types.json`.

For comparison, TypeScript 6.0.2 accepts TypeSlayer's current deep import and emits:

```text
trace.json
types.json
```

## Suggested fix

1. Resolve and execute each compiler package's declared `bin` entry without importing the private `package/bin/name`
   subpath. Preserve the existing Node memory-option behavior and Yarn PnP path.
2. Detect `legend.json`, load every referenced `types_N.json` file, and preserve `checkerId` while correlating trace
   events with type IDs.
3. Make the `processTypeReferences` count optional rather than fabricating a value when TypeScript 7 omits `args`.
4. Add integration fixtures for TypeScript 6's single `types.json` layout and TypeScript 7's default multi-checker,
   `--singleThreaded`, and missing-event-argument layouts.

## Boundary-level fallback

Run TypeSlayer's compiler command against an actual `typescript@6.0.2` package in a disposable analysis checkout.
This resolves the reported package-path failure and emits the expected trace filenames,
but the TypeSlayer UI flow was not verified end to end.
It uses the classic compiler and therefore does not profile TypeScript 7's native performance.
~~~
