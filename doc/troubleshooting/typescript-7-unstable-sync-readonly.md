# TypeScript 7 semantic lint plugins need the sync API and an internal mapped-readonly flag

## Symptom

Code written for the TypeScript 6 compiler API cannot obtain a `Program` or `TypeChecker` from the repository's
installed `typescript@7.1.0-beta` package.
Two representative failures are:

- `import * as ts from 'typescript'` loads the version-only root export,
  so `ts.createProgram` is absent;
- importing the former `typescript/lib/typescript.js` implementation fails with
  `ERR_MODULE_NOT_FOUND` because TypeScript 7 does not ship that file.

TypeScript 7 does expose a synchronous semantic API at `typescript/sync`.
That API can support a synchronous Oxlint JavaScript rule,
but it does not export a named readonly-symbol query or `CheckFlags` enum.
Mapped readonly properties therefore require an explicitly version-tested adapter around the numeric
`Symbol.checkFlags` field.

Version context:
 the repository moved from `typescript@7.0.2` to `typescript@7.1.0-beta` (published 2026-10-20T16:45:47Z),
 which renamed the API entry points.
`typescript/unstable/*` no longer resolves (`ERR_PACKAGE_PATH_NOT_EXPORTED`);
 the same API now ships at `typescript/sync`,
`typescript/async`,
`typescript/fs`,
`typescript/path`,
 `typescript/proto`,
`typescript/ast`,
`typescript/ast/is`,
`typescript/ast/factory`,
`typescript/ast/utils`,
 `typescript/ast/scanner`,
`typescript/ast/visitor`,
and `typescript/ast/clone`.
The package root `typescript` stays version-only,
 and the rename changed no symbol names.
Evidence recorded before that move keeps its original text with an in-place rename note
 (the moved-identifier rule `WR6` in `AGENTS.md`),
 and `node_modules/.pnpm/typescript@7.0.2/` citations name 7.0.2 virtual-store paths that the move removed.

## Root cause

### The stable package root is not the compiler API

The `typescript@7.0.2/package.json:37-40` export map (2026-era evidence) exported the package root as `lib/version.cjs`
and the semantic API under an explicitly unstable subpath:

```json
"exports": {
    "./package.json": "./package.json",
    ".": "./lib/version.cjs",
    "./unstable/sync": "./dist/api/sync/api.js",
```

(Path renamed to `typescript/sync` in 7.1.0-beta,
2026-10-20;
 the `./unstable/` segment is gone from every entry and the root `.` export stays version-only.)

The [TypeScript 7.0 announcement][typescript-7-announcement] states that 7.0 does not ship a stable API and expects a
new API in 7.1.
The exported unstable subpath is therefore usable by explicit project choice,
not a compatibility continuation of `createProgram`.
The 7.1.0-beta rename moved the export paths only;
 it changed no symbol names (verified during the 2026-10-20 move).

### The synchronous API is a native-process client

The installed `typescript@7.0.2/dist/api/sync/client.js:11-36` starts the bundled executable with `--api` and wraps it
in `SyncRpcChannel`:

```javascript
const args = [
    "--api",
    "--cwd",
    cwd,
];
// ...
const channel = new SyncRpcChannel(resolveExePath(options), args, collectTiming);
```

Calls such as `Checker.getTypeAtLocation` are synchronous RPC requests.
`client.js:55-62` serializes the request and blocks for its response:

```javascript
apiRequest(method, params) {
    const encodedPayload = JSON.stringify(params);
    const start = performance.now();
    const result = this.channel.requestSync(method, encodedPayload);
```

This shape fits Oxlint's synchronous JavaScript-rule visitors,
but the client must be process-scoped and reused rather than recreated per parameter or source file.

### `getAliasedSymbol` accepts alias symbols only

The demand-driven effect work initially called `Checker.getAliasedSymbol` for every resolved symbol.
An ordinary value symbol caused the native TypeScript-Go process to panic with:

```text
panic: Should only get alias here
```

The installed `typescript@7.0.2` package records TypeScript-Go commit
`2bd066d87f5bafd315be9f40889d0a60b9e58e0b` in `gitHead`.
At that revision,
`internal/checker/checker.go:16176-16183` explicitly documents and enforces the precondition:

```go
// symbol. The function panics if the argument is not a symbol with an alias meaning.
func (c *Checker) resolveAlias(symbol *ast.Symbol) *ast.Symbol {
    if symbol.Flags&ast.SymbolFlagsAlias == 0 {
        panic("Should only get alias here")
    }
```

`internal/checker/checker.go:31924-31926` adds no guard in the public checker method:

```go
func (c *Checker) GetAliasedSymbol(symbol *ast.Symbol) *ast.Symbol {
    return c.resolveAlias(symbol)
}
```

The synchronous API handler also forwards the resolved symbol directly at
`internal/api/session.go:2552-2572`:

```go
// handleGetAliasedSymbol resolves an alias symbol to its target.
func (s *Session) handleGetAliasedSymbol(ctx context.Context, params *CheckerSymbolParams) (*SymbolResponse, error) {
    // ...
    aliased := setup.checker.GetAliasedSymbol(symbol)
```

The panic was therefore consumer misuse,
not a TypeScript-Go defect.
`effect-call-resolution.ts` now checks
`(symbol.flags & SymbolFlags.Alias) !== 0`
before making the synchronous RPC.
Ordinary symbols stay unchanged.

### Signature usages distinguish calls from value escapes

The catalog-free foreign-ownership fallback needs every inbound use of a reached callable without scanning every
callable declaration in the configured project.
The installed synchronous client exposes that operation at
`node_modules/.pnpm/typescript@7.0.2/node_modules/typescript/dist/api/sync/api.js:711-721`
(a 7.0.2 virtual-store path that the 7.1.0-beta move removed;
 the same relative file `dist/api/sync/api.js` ships in `typescript@7.1.0-beta`):

```javascript
getSignatureUsage(signatureDecl) {
    const data = this.client.apiRequest("getSignatureUsages", {
        snapshot: this.snapshotId,
        project: this.project.id,
        signatureDecl: getNodeId(signatureDecl),
    });
    return (data ?? []).map(entry => ({
        name: new NodeHandle(entry.name, this.project),
        call: entry.call ? new NodeHandle(entry.call, this.project) : undefined,
    }));
}
```

TypeScript-Go revision `2bd066d87f5bafd315be9f40889d0a60b9e58e0b` implements the query in
`internal/ls/findallreferences.go:1100-1158`.
It asks for references across `l.program.GetSourceFiles()` and associates a call only when the climbed reference is the
call expression's callee:

```go
sourceFiles := l.program.GetSourceFiles()
entries := l.GetReferencedSymbolsForNode(ctx, name.Pos(), name, sourceFiles)
// ...
called := ast.ClimbPastPropertyAccess(node)
var callExpr *ast.Node
if called.Parent != nil && ast.IsCallExpression(called.Parent) && called.Parent.Expression() == called {
    callExpr = called.Parent
}
```

The API session at `internal/api/session.go:3311-3350` resolves the declaration inside the requested snapshot,
runs this language-service query,
and returns both name and optional call handles.
A missing `call` is therefore useful fail-closed evidence:
the callable escaped through a value reference rather than being invoked at that exact usage.

A live probe over the configured 287-source fixture returned both calls to `readChild` in 6.3 milliseconds.
A second exact API query over the current 792-source package-config project completed in 6.0 milliseconds.
The implementation searches `program.GetSourceFiles()` per query,
so these measurements are empirical evidence rather than a project-size-independent bound.
An overload probe returned the same direct call plus alias and export value references for each overload declaration;
the value references had no call handle.
The production rule accepts only a usage that creates its own exact owned edge.
Non-call escapes,
top-level or excluded callers,
query failures,
and edge mismatches add an ordinary inbound and remove inferred `ForeignBorrowed` provenance.

### Unchanged files must reuse their immutable snapshot

TypeScript's installed sync adapter sends every `updateSnapshot` call through synchronous RPC.
It then retains decoded source files that the server reports as unchanged.
`node_modules/.pnpm/typescript@7.0.2/node_modules/typescript/dist/api/sync/api.js:66-76` contains both operations:

```js
updateSnapshot(params) {
    this.ensureInitialized();
    const requestParams = toUpdateSnapshotRequest(params);
    const data = this.client.apiRequest("updateSnapshot", requestParams);
    // Retain cached source files from previous snapshot for unchanged files
    if (this.latestSnapshot) {
        this.sourceFileCache.retainForSnapshot(data.snapshot, this.latestSnapshot.id, data.changes);
```

The first semantic adapter sent `fileChanges.changed` for every Oxlint source and called
`clearSourceFileCache()` immediately before every update.
One worker therefore paid one synchronous native request per source and discarded the decoded-source reuse that
TypeScript provides.
On the semantic plugin package,
three unchanged sequential package runs took 13,260 ms,
13,214 ms,
and 13,331 ms.

[`typescript-sync-adapter.ts:282-308`][adapter-reuse] now looks up the already materialized project and source.
When the source text is byte-for-byte equal and no prior path was deleted,
it returns handles over that immutable snapshot without clearing TypeScript's source cache or issuing another update.
Changed overlays,
created files,
deleted files,
and project discovery still take the update path.

The earlier 29.4 and 39.0 second observations came from several package lint processes running concurrently.
`OXLINT_THREADS=1` limits each Oxlint process to one worker;
it does not serialize separate `mise run` processes or their native TypeScript children.

### Snapshots provide the project and unsaved-file lifecycle

The installed `typescript@7.0.2/dist/api/sync/api.d.ts:39` exposes `API.updateSnapshot`.
The resulting `Project.checker` exposes the required type operations at `api.d.ts:207-282`,
including:

```typescript
getTypeAtLocation(node: Node): Type | undefined;
getTypeFromTypeNode(node: TypeNode): Type | undefined;
isTypeAssignableTo(source: Type, target: Type): boolean;
getPropertiesOfType(type: Type): readonly Symbol[];
getIndexInfosOfType(type: Type): readonly IndexInfo[];
```

The API client's virtual filesystem callback lets an Oxlint rule overlay `context.sourceCode.text`.
Passing the path through `fileChanges.changed` produces a new snapshot without writing editor text to disk when the
configured project,
rather than a persistent API-open-file association,
owns later snapshots.

The production adapter initially used `openFiles` for every-file persistence.
A built-artifact test changed `SemanticFixtureBox<string>` to `Readonly<SemanticFixtureBox<string>>` in the virtual
overlay,
but the next snapshot still returned `SemanticFixtureBox<string>`.
`clearSourceFileCache()` did not change that result.

At the audited TypeScript Go revision,
`internal/project/projectcollectionbuilder.go:180-205` shows that `CloseFiles` decrements or deletes an API-open-file
association and `OpenFiles` increments it:

```go
if apiRequest.CloseFiles != nil {
    for path := range apiRequest.CloseFiles.Keys() {
        if entry, ok := b.apiState.openFiles[path]; ok {
            if entry.refCount > 1 {
                entry.refCount--
                b.apiState.openFiles[path] = entry
            } else {
                delete(b.apiState.openFiles, path)
            }
        }
    }
}

if apiRequest.OpenFiles != nil {
    for uri := range apiRequest.OpenFiles.Keys() {
        fileName := uri.FileName()
        path := b.toPath(fileName)
        // ...
        entry.refCount++
        b.apiState.openFiles[path] = entry
    }
}
```

The verified adapter now uses `openFiles` only to discover the configured project.
It then opens that `tsconfig.json`,
closes the temporary file association,
and sends later `fileChanges.changed` notifications to project-owned snapshots.
The same built test then returned `Readonly<SemanticFixtureBox<string>>` while confirming disk text was unchanged.

### Active source metadata can disagree with configured ownership

A one-worker workspace sweep repeatedly failed only on
`package/git-policy/cli/src/bin.ts` with:

```text
node-not-found: Effect summary index omitted owned callable declaration
/var/home/user/Monochromatic/package/git-policy/cli/src/bin.ts:3407:12786:263.
```

A package-local run indexed the same `runCliGit` function declaration.
The workspace process had already crossed many project snapshots.
Its current source remained available through `Project.program.getSourceFile`,
but program metadata could classify that decoded wrapper as external.
The effect index then skipped the active source before the verifier traversed it.

Configured-project discovery already proves that the current lint target belongs to selected project.
The effect index now receives the verifier's exact active `SourceFile`,
always indexes that source regardless of external-library metadata,
and applies external-library filtering only to other project sources.
Its direct-summary cache also rejects exact-text hits that omit any callable key present in current source wrapper.

A later process-local fixed-point cache reintroduced the same failure.
Its key covered the configured project and optional package-analysis root,
but not the active-source exception.
The root `tsconfig.json` decodes workspace package sources while classifying them as external libraries.
An index built for a root config source therefore excluded `bin.ts`.
The unchanged file list and source signatures then let the cache return that index when `bin.ts` became active.

Fixed-point cache validity now fingerprints the exact source set admitted by ownership,
external-library,
active-source,
and package-analysis policies.
Ordinary project sources retain cross-file index reuse because their admitted set is stable.
Making an external-classified source active changes that set and forces a new index
without retaining one full index per active path.
A regression creates an installed TypeScript package,
builds an index that excludes it,
then proves that making its source active builds a summary for its callable.

A repeat one-worker workspace sweep reported zero semantic bridge failures across 1,315 replacement-rule diagnostics.
Package type lint,
Oxlint,
and focused effect-summary tests also passed.

### Windows snapshots use case-insensitive source identity

The Windows x64 host run exposed a second stale-overlay path.
Oxlint supplied an absolute path with an uppercase drive letter and backslashes,
while TypeScript returned the source as a lowercase-drive path with forward slashes:

```text
D:\a\Monochromatic\Monochromatic\packages\...
d:/a/Monochromatic/Monochromatic/package/...
```

The virtual overlay map used Node's platform `resolve()` result as its key.
The differently cased TypeScript filesystem callback missed that key and delegated to disk,
so the changed overlay still produced `SemanticFixtureBox<string>` and recovering syntax produced `any`.
The configured project itself remained discoverable,
which made this look like a parser or snapshot-refresh defect rather than a file-identity mismatch.

The adapter now case-folds absolute paths only on Windows before overlay,
active-file,
cache,
and `fileChanges` lookup.
Other hosts preserve path case.
The Windows host test then verifies overlay refresh,
parser recovery,
noncanonical filename casing,
and separator-neutral source identity through the native TypeScript process.

### Hidden disposable directories can fall into an inferred project

A stale-contract suggestion test initially created its disposable source under a dot-prefixed directory inside the
fixture `src/` tree.
`getDefaultProjectForFile` returned an inferred project whose config path was `/dev/null/inferred`,
and the configured-project snapshot then omitted that identity.
The semantic rule correctly failed closed with:

```text
project-not-found: TypeScript snapshot omitted configured project /dev/null/inferred.
```

Creating the same disposable source under a non-hidden directory inside the configured project selected the fixture
`tsconfig.json` and allowed Oxlint to apply the suggestion.
Semantic integration fixtures must therefore verify configured-project inclusion,
not infer inclusion merely from filesystem ancestry.

### Process `exit` cleanup races TypeScript's native child shutdown

The TypeScript sync channel registers its own process-exit listener at
`dist/api/syncChannel.js:31` to `40`:

```js
const liveChildren = new Set();
process.on("exit", () => {
    for (const child of liveChildren) {
        try {
            child.kill();
        }
        catch {
        }
    }
    liveChildren.clear();
});
```

The default `child.kill()` signal is `SIGTERM`.
The native server handles that signal by cancelling its context at
`cmd/tsgo/api.go:51` to `57`:

```go
ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
defer stop()

if err := s.Run(ctx); err != nil {
    fmt.Fprintln(os.Stderr, err)
    return 1
}
```

The resulting error text is the bare line:

```text
context canceled
```

The adapter originally registered another `exit` listener after creating the TypeScript client.
TypeScript's listener ran first,
sent `SIGTERM`,
and allowed the native server to print the cancellation error before adapter cleanup ran.
Accepting or filtering that line was incorrect because successful lint output
must not contain unexplained shutdown errors.

Moving adapter cleanup to `beforeExit` fixed natural one-process shutdown,
but parallel unit processes still reproduced the line intermittently.
TypeScript's explicit channel close at `dist/api/syncChannel.js:160` to `177` destroys both streams
and immediately sends the same default signal:

```js
close() {
    try {
        liveChildren.delete(this.child);
        this.child.stdout?.destroy();
        this.child.stdin?.destroy();
        this.child.kill();
        this.readFd = -1;
        this.writeFd = -1;
    }
    catch {
    }
}
```

Under concurrent process load,
the native child could handle `SIGTERM` before pipe EOF ended the server cleanly.
The deterministic adapter now guards the `API.client.channel.child` shape recorded from `typescript@7.0.2`
(2026-era evidence)
and replaces that child's `kill` operation with the same operation forced to `SIGKILL`.
TypeScript still owns stream cleanup,
child tracking,
and invocation timing.
The native server cannot handle `SIGKILL`,
so it cannot print a cancellation error.
The `beforeExit` hook remains so TypeScript's later `exit` listener finds no live child.

Demand-driven external package analysis creates separate `API` clients in
`external-implementation-project.ts`.
The first shutdown correction guarded only the main client in `typescript-sync-adapter.ts`,
so a final root sweep could still print one cancellation line when an external-project client exited.
Both construction sites now immediately pass their native child through
`configureNativeApiChildShutdown(nativeApiChild(api))`.
A repository search confirms those are the only `new API(...)` sites in the semantic package.

### Mapped readonly state is exposed only as an unnamed number

`typescript@7.0.2/dist/api/sync/api.d.ts:387` exposes this field:

```typescript
readonly checkFlags: number;
```

The matching native source at
`typescript-go@168e7015edf98244febc8f4ae450b673b5d195d7/internal/ast/checkflags.go:8-12` defines the hidden meaning:

```go
CheckFlagsNone              CheckFlags = 0
CheckFlagsInstantiated      CheckFlags = 1 << 0
CheckFlagsSyntheticProperty CheckFlags = 1 << 1
CheckFlagsSyntheticMethod   CheckFlags = 1 << 2
CheckFlagsReadonly          CheckFlags = 1 << 3
```

Syntactic `readonly` modifiers cover directly declared properties.
Recursively mapped types need the transient `CheckFlagsReadonly` bit because their resolved property declarations may
point back to mutable source declarations.
The missing named API is tracked by [typescript-go issue 4080][readonly-issue].

## Verification

Verified components (the recorded run used the 2026-era `typescript@7.0.2` package):

- repository package `typescript@7.0.2`;
- npm integrity recorded in `pnpm-lock.yaml`;
- `microsoft/typescript-go` source revision
  `168e7015edf98244febc8f4ae450b673b5d195d7`;
- Oxlint 1.73.0 JavaScript-plugin execution boundary;
- Linux x64 with Node 26.5.0.

A disposable Oxlint plugin imported `API` from `typescript/unstable/sync`
(path renamed to `typescript/sync` in 7.1.0-beta,
2026-10-20),
opened a fixture `tsconfig.json`,
and queried source nodes through the returned project.
Its corpus contained imported aliases,
generics,
unions,
overloads,
function types,
call signatures,
method signatures,
a recursive mapped `DeepReadonly`,
direct mutation,
cross-file mutation,
and callback-propagated mutation.

Run the semantic API independently with this minimal fixture.
It is recorded as run against `typescript@7.0.2` (2026-era evidence),
and its `typescript/unstable/*` specifiers were renamed `typescript/*` in 7.1.0-beta,
2026-10-20,
so a rerun today imports `typescript/sync` and `typescript/ast/is` instead:

```typescript
// /tmp/ts7-sync-probe/input.ts
export type Mutable<T> = { value: T };
export type DeepReadonly<T> = { readonly [Key in keyof T]: DeepReadonly<T[Key]> };
export function inspect(value: DeepReadonly<Mutable<string>>): string {
  return value.value;
}
```

```json
// /tmp/ts7-sync-probe/tsconfig.json
{
  "compilerOptions": { "strict": true, "noEmit": true },
  "files": ["input.ts"]
}
```

```javascript
// /tmp/ts7-sync-probe/probe.mjs
import { createRequire } from 'node:module';

const requireFromRepository = createRequire(`${process.cwd()}/package.json`);
const { API } = requireFromRepository('typescript/unstable/sync');
const { isFunctionDeclaration } = requireFromRepository('typescript/unstable/ast/is');
const root = '/tmp/ts7-sync-probe';
const config = `${root}/tsconfig.json`;
const input = `${root}/input.ts`;
const api = new API({ cwd: root });
try {
  const snapshot = api.updateSnapshot({ openProjects: [config] });
  const project = snapshot.getProject(config);
  const source = project.program.getSourceFile(input);
  const declaration = source.statements.find(isFunctionDeclaration);
  const type = project.checker.getTypeFromTypeNode(declaration.parameters[0].type);
  const property = project.checker.getPropertiesOfType(type)[0];
  console.log(project.checker.typeToString(type));
  console.log((property.checkFlags & (1 << 3)) !== 0);
  snapshot.dispose();
} finally {
  api.close();
}
```

Run from this repository so Node resolves its installed TypeScript package:

```sh
node /tmp/ts7-sync-probe/probe.mjs
```

Verified output:

```text
DeepReadonly<Mutable<string>>
true
```

The alias precondition probe used an ordinary value symbol.
It is likewise recorded as run against `typescript@7.0.2` (2026-era evidence),
and its `typescript/unstable/sync` specifier was renamed `typescript/sync` in 7.1.0-beta,
2026-10-20:

```typescript
// /tmp/ts7-alias-guard-probe/input.ts
export const ordinaryValue = 1;
```

```json
// /tmp/ts7-alias-guard-probe/tsconfig.json
{
  "compilerOptions": { "strict": true },
  "files": ["input.ts"]
}
```

```javascript
// /tmp/ts7-alias-guard-probe/probe.mjs
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const requireFromRepository = createRequire(`${process.cwd()}/package.json`);
const { SymbolFlags } = requireFromRepository('typescript/unstable/sync');
const builtEntry = pathToFileURL(join(
  process.cwd(),
  'package/oxlint-plugin/prefer-readonly-parameter-type/dist/final/node/index.mjs',
)).href;
const { closeSemanticBridge, openSemanticFile } = await import(builtEntry);
const fileName = '/tmp/ts7-alias-guard-probe/input.ts';
const sourceText = readFileSync(fileName, 'utf8');
const session = openSemanticFile({ fileName, sourceText, hasBOM: false });
const node = session.nodeAtOffset(sourceText.indexOf('ordinaryValue'));
const symbol = session.project.checker.getSymbolAtLocation(node);
if (symbol === undefined)
  throw new Error('Expected ordinary value symbol.');
console.log(JSON.stringify({
  flags: symbol.flags,
  alias: (symbol.flags & SymbolFlags.Alias) !== 0,
}));
const resolved = process.env.GUARD_ALIAS === '1'
  ? ((symbol.flags & SymbolFlags.Alias) !== 0
    ? session.project.checker.getAliasedSymbol(symbol)
    : symbol)
  : session.project.checker.getAliasedSymbol(symbol);
console.log(JSON.stringify({ resolvedFlags: resolved.flags }));
closeSemanticBridge();
```

Run the unsafe and guarded variants from the repository root:

```sh
node /tmp/ts7-alias-guard-probe/probe.mjs
GUARD_ALIAS=1 node /tmp/ts7-alias-guard-probe/probe.mjs
```

The unsafe variant printed the non-alias identity and exited with the native panic:

```text
{"flags":2,"alias":false}
Error: panic: Should only get alias here
```

The guarded variant exited successfully:

```text
{"flags":2,"alias":false}
{"resolvedFlags":2}
```

The published-consumer regression opens the same API without calling `closeSemanticBridge` explicitly.
It asserts that natural process shutdown produces the semantic result on stdout and an empty stderr stream.
A full parallel package unit run after the forced-signal fix
produced no `context canceled` line on either captured stream.
A package Oxlint run captured after the fix also contained no cancellation line.
Final root process `proc_367` ran the single-worker workspace Oxlint task after both client sites were guarded.
Its stdout and stderr contain zero occurrences of `context canceled`,
`SemanticBridgeError`,
or the omitted-owned-callable failure.
GitHub Actions run `29396066153` passed the native lifecycle and disposable external-consumer tasks on Ubuntu,
macOS 15,
and Windows after the shutdown change.

A regression opens the same unchanged source twice and requires both sessions to expose the same decoded
`SourceFile` object.
The existing overlay,
rename,
nested-project,
and parser-recovery cases continue to verify that inputs requiring a new snapshot do not take the reuse path.
Warm one-thread package timing is verified with three sequential unchanged invocations of
`mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:oxlint`.
The final measured runs completed in 6,610 ms,
6,476 ms,
and 6,668 ms.

### Working catalog

- `API.updateSnapshot({ openProjects })` loads configured projects.
- `Snapshot.getProject` and `getDefaultProjectForFile` expose project ownership.
- `Program.getSourceFile` returns a traversable TypeScript 7 AST with source offsets.
- `Checker.getTypeFromTypeNode` preserves imported alias identity and generic instantiation.
- direct declaration modifiers classify ordinary readonly properties.
- `symbol.checkFlags & (1 << 3)` classifies the tested recursively mapped readonly property.
- virtual filesystem overlays plus configured-project snapshots and `fileChanges.changed` update semantic results;
- byte-identical sources reuse the current immutable snapshot and decoded `SourceFile` object;
- temporary `openFiles` discovery followed by `openProjects` and `closeFiles` avoids stale API-open-file semantics.
- TypeScript source offsets map to exact Oxlint diagnostic locations.
- `Checker.getAliasedSymbol` is called only when `symbol.flags` contains `SymbolFlags.Alias`;
- ordinary symbols bypass alias resolution without changing identity;
- `Checker.getSignatureUsage` returns project references with call handles only for exact callee positions;
- overload declarations expose the same call and value-escape family in the tested fixture;
- every main or demand-driven external `API` client immediately receives guarded TypeScript 7.0.2 child control;
- guarded child control forces channel-owned termination to `SIGKILL`;
- `beforeExit` bridge cleanup closes the native child before TypeScript's `exit` kill handler runs.

### Failing or unsupported catalog

- TypeScript 6 `createProgram` and `createLanguageService` entry points are absent from the TypeScript 7 package root.
- `typescript/lib/typescript.js` is absent.
- no exported `CheckFlags` enum names the mapped readonly bit.
- no stable TypeScript 7 API contract covers this use in 7.0.
- recreating `API` for every linted parameter starts unnecessary native clients and loses snapshot reuse;
- retaining `openFiles` across changed virtual overlays returned stale type text in the built adapter test;
- a dot-prefixed disposable source directory selected `/dev/null/inferred` rather than the expected configured project;
- calling `Checker.getAliasedSymbol` with an ordinary symbol panics the native child with
  `Should only get alias here`;
- treating every signature reference as a call is unsound because alias and export value escapes have no call handle.

## Verified workarounds

### Guard alias resolution by `SymbolFlags.Alias`

Before calling `Checker.getAliasedSymbol`,
require `(symbol.flags & SymbolFlags.Alias) !== 0`.
Use the original symbol otherwise.
The direct unsafe and guarded probes in the Verification section reproduced the panic and successful bypass.

Tradeoff:
callers must retain this precondition at every alias-resolution call site.
The sync API does not turn misuse into an ordinary JavaScript argument error.

### Use the synchronous API with a narrow adapter

Import `typescript/sync`,
keep one `API` client per plugin process,
and hide all sync API calls behind a project-owned bridge module.
Keep the readonly bit in one named constant with a TypeScript-version contract test.

Tradeoff:
minor TypeScript updates may change the API or hidden flag meaning.
The adapter must fail closed and emit a bridge diagnostic instead of silently treating unknown types as readonly.

### Use snapshots and a virtual filesystem overlay

Use `openFiles` only for ancestor `tsconfig.json` discovery.
Open that configured project,
close the temporary file association,
and create replacement project snapshots with `fileChanges.changed` whenever current source changes.
Before updating,
compare current source text with the source already materialized in the immutable snapshot.
Reuse that project and source when they are identical and no deletion requires invalidation.
Only clear the decoded-source cache and send `fileChanges` when the source or project lifecycle actually changed.
Dispose superseded snapshots and close the API client during Node `beforeExit`
when the host provides no explicit lifecycle hook.
For the `typescript@7.0.2` sync client (2026-era evidence),
guard the native-child shape
and force TypeScript's channel-owned kill operation to use `SIGKILL` before exposing the API to callers.

Tradeoff:
the adapter depends on pinned TypeScript channel internals until upstream treats cancellation as clean shutdown.
Shape checks fail closed on a changed implementation.
Captured-stderr lifecycle and parallel-run tests must guard against noisy cleanup and leaked children.

### Keep syntax effects in the Oxlint traversal

Use TypeScript 7 for symbols,
types,
resolved signatures,
and cross-file identity.
Use the current Oxlint tree and source text for diagnostics and suggestions,
with source offsets as the mapping key.

Tradeoff:
the rule maintains a deliberate two-tree boundary and must test Unicode,
BOM,
comments,
and parser-recovery span cases.

## What does not work

- **Importing the package root as the old compiler API:
  ** it returns only version metadata in `typescript@7.0.2` and `typescript@7.1.0-beta`.
- **Importing `typescript/lib/typescript.js`:
  ** that TypeScript 6 implementation path is not shipped.
- **Using `@typescript-eslint/project-service`:
  ** its current peer range ends before TypeScript 7 and it wraps the old
  `tsserverlibrary` API.
- **Using `ts-morph`:
  ** current `ts-morph@28` bundles TypeScript 6 semantics and would not query the repository's
  installed TypeScript 7 compiler.
- **Using only declarations to detect mapped readonly state:
  ** mapped properties can resolve to mutable origin
  declarations;
   the transient readonly bit carries the projected state.
- **Using the asynchronous API at `typescript/async` inside an Oxlint visitor:
  ** Oxlint JavaScript rule callbacks are synchronous and
  cannot await semantic results before reporting.
- **Closing the bridge from a process `exit` listener:
  ** TypeScript's earlier listener sends `SIGTERM` first,
  and the native server prints `context canceled` to stderr.
- **Relying only on pipe destruction before TypeScript's default kill:
  ** parallel runs can deliver `SIGTERM` before the native server observes EOF.
- **Filtering `context canceled` from command output:
  ** it hides a lifecycle defect and can also hide a real cancellation failure.
- **Calling `clearSourceFileCache()` and `updateSnapshot()` for every unchanged lint source:
  ** it defeats TypeScript's decoded-source retention and adds one synchronous native request per source.
- **Treating `OXLINT_THREADS=1` as a global process limit:
  ** separately launched package tasks still compete through distinct Oxlint and TypeScript processes.
- **Calling `getAliasedSymbol` unconditionally:
  ** TypeScript-Go requires alias meaning and panics for an ordinary symbol.
  Catching the JavaScript error after the native panic does not restore that semantic request.

## Upstream filing decision

`.out-of-scope/` contains no matching exemption.
The existing [typescript-go issue 4080][readonly-issue] requests a supported readonly-symbol API,
so a new issue would be a duplicate.
The unchanged-snapshot performance defect was in this repository's adapter,
not TypeScript:
the installed API already retains decoded unchanged sources.
No upstream performance issue should be filed.

GitHub searches across open and closed TypeScript-Go issues and pull requests for
`"Should only get alias here"`
found issue 3256,
whose captured failures concern unrelated language-server crashes rather than the sync checker API precondition.
The alias panic investigated here came from this repository calling a documented alias-only operation
on an ordinary symbol.
Constraint 1 therefore fails:
there is no upstream issue or patch to file for this incident.

GitHub issue searches for `"context canceled" API`,
`SIGTERM sync API close`,
and `API shutdown cancellation` found no matching TypeScript Go report.
The shutdown noise is upstream behavior:
the shipped sync client sends `SIGTERM` during `close()`,
and the shipped server reports expected signal cancellation as an error on stderr.
Upstream can fix it by treating cancellation during an explicit API shutdown as successful termination or by giving the
client a quiet shutdown operation.
A focused upstream report is warranted,
but none was published because opening an external issue requires the user's authorization.
The repository workaround remains pinned and shape-guarded until that report or an upstream fix lands.

### Shutdown noise filing constraints

1.  **Is it really upstream's fault?
    ** Yes.
    Both signal delivery and cancellation-error printing are in the shipped TypeScript sync client and API server.
2.  **Can upstream fix it?
    ** Yes.
    Explicit shutdown can use a quiet protocol or treat expected shutdown cancellation as success.
3.  **Are they supporting this use case?
    ** The 7.0-era subpath was intentionally unstable
    (`typescript/unstable/sync`,
    renamed `typescript/sync` in 7.1.0-beta,
    2026-10-20),
    but TypeScript ships the client,
    server,
    and explicit `close()` operation together.
4.  **Would the repository welcome the report?
    ** `CONTRIBUTING.md` permits focused,
    human-reviewed bug reports and specifically disclosed AI assistance.
5.  **Will they fix it?
    ** Unknown.
    No matching issue or stated shutdown policy was found.
6.  **Was a minimal fix prototyped?
    ** No upstream patch was produced.
    The consumer-side forced-signal workaround isolates the emitting path and removes the diagnostic.

Suggested issue title
 (the `typescript/unstable/sync` path in it was renamed `typescript/sync` in 7.1.0-beta,
2026-10-20):
`typescript/unstable/sync API close can print context canceled to stderr`.

Suggested report body:

~~~md
With `typescript@7.0.2`, concurrent users of `typescript/unstable/sync` can emit a bare
`context canceled` line to stderr during otherwise successful shutdown.

`dist/api/syncChannel.js` destroys the channel streams and calls `child.kill()`,
which sends `SIGTERM`.
`cmd/tsgo/api.go` builds its server context with `signal.NotifyContext` and prints the resulting cancellation error to
stderr before returning status 1.
Under concurrent process load,
the child can observe `SIGTERM` before pipe EOF ends the server cleanly.

Expected:
closing a successfully used sync API client produces no stderr output.

Actual:
shutdown can print `context canceled` even though all semantic requests completed successfully.

A consumer workaround that forces the channel-owned child termination to `SIGKILL` removes the line,
but it depends on private unstable client fields.
Could explicit client shutdown use a quiet protocol operation,
or could the API server treat signal cancellation during expected shutdown as success?
~~~

### Readonly API gap filing constraints

1.  **Is it really upstream's fault?
    ** No for API instability,
    because TypeScript 7.0 explicitly does not promise a stable API.
    The missing readonly query is an acknowledged API gap.
2.  **Can upstream fix it?
    ** Yes,
    by exporting a readonly-symbol query or a supported flag.
3.  **Are they supporting this use case?
    ** Not as a stable TypeScript 7.0 API use case.
4.  **Would the repository welcome the contribution?
    ** Potentially.
    `CONTRIBUTING.md` permits specifically chosen AI-assisted work with human review and disclosure,
    while rejecting bulk autonomous submissions.
5.  **Will they likely fix it?
    ** Unknown.
    Issue 4080 is open in the `Post-7.0` milestone.
6.  **Was a minimal upstream fix prototyped?
    ** No.
    Constraints 1 and 3 fail,
    and the verified consumer adapter resolves this repository's immediate need.

Do not file the following draft as-is.
It is an additive comment candidate for issue 4080 only if a human decides that documenting this workaround
would help upstream.
The draft keeps its recorded `typescript@7.0.2` and `typescript/unstable/sync` wording
(path renamed to `typescript/sync` in 7.1.0-beta,
2026-10-20):

~~~md
TypeScript 7.0.2's `typescript/unstable/sync` API currently serializes `Symbol.checkFlags` as a number.
For a recursive mapped `DeepReadonly<T>` probe,
`checkFlags & (1 << 3)` matched the native `CheckFlagsReadonly` definition in
`internal/ast/checkflags.go` and distinguished the projected property from its mutable origin declaration.

That is enough for a version-pinned consumer workaround,
but it reinforces this issue's request for a supported query:
the enum meaning is not exported and the 7.0 API is explicitly unstable.
I verified the result through an Oxlint JavaScript plugin using snapshot updates and a virtual filesystem overlay.
This investigation and draft used AI assistance and was reviewed against the installed package and current Go source.
~~~

[adapter-reuse]:
  ../../package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/
[readonly-issue]: https://github.com/microsoft/typescript-go/issues/4080
[typescript-7-announcement]: https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/
