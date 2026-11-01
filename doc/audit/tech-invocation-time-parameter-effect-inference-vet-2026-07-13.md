# Invocation-time parameter effect inference technology vet

Status:
 implementation selection complete;
consumer and platform validation remain acceptance gates.

Lifecycle phase:
 adopted by the 2026-07-13 implementation request.

Subject:
 invocation-time parameter effect inference.

Decision scope:
 select an analyzer and cross-invocation cache architecture for external package,
Node,
ECMAScript,
and browser callable effects in the semantic readonly Oxlint plugin.

Started:
 2026-07-13.

Last updated:
 2026-07-13.

Governing skill commit:
 `3b6d1bd6ac0c6eb5704152ddb00e2b69ddcf653b`.

Governing skill SHA-256:
 `71c50a51d0f0086f789e350ef43824f8aead66435f9ab92d94aae751d16d8359`.

Compatibility fingerprint:
 `02360896073ee2f8ead5747ed02c62ab4bf676ce4db7aa7b21065c47c417bd9c`.

Active audit owner:
 current Pi session implementing the semantic readonly rule.

Prior compatible report:
 none found.
The earlier
[`tech-readonly-parameter-semantic-bridge-vet-2026-07-12.md`](./tech-readonly-parameter-semantic-bridge-vet-2026-07-12.md)
selected the TypeScript 7 bridge but did not decide external implementation inference or cross-process reuse.

## Context

The current plugin analyzes configured workspace TypeScript source with TypeScript 7 and caches direct summaries only in
one JavaScript process.
A repository scan of `pnpm-lock.yaml` found 748 package records,
477 snapshot identities,
and 124 workspace importers.
The requirement is not to enumerate every exported member in advance.
The plugin must resolve invoked callables on demand,
analyze inspectable implementations,
fail closed when proof is unavailable,
and avoid recomputing unchanged semantic summaries on each Oxlint process invocation.

Oxlint JavaScript plugins do not receive TypeScript parser services.
The plugin therefore retains the selected `typescript/unstable/sync` bridge
(`typescript/unstable/sync` was renamed `typescript/sync` in 7.1.0-beta,
2026-10-20).
Oxlint's language server does not execute JavaScript plugins,
so the performance boundary covered here is CLI lint and fix execution.

## Hard constraints

- TypeScript 7 `typescript/unstable/sync` remains the only semantic compiler API.
- Analysis runs synchronously inside one Oxlint JavaScript-plugin worker.
- Invoked package callables resolve by exact package version,
  declaration,
  implementation source,
  and callable identity.
- Observation is reported only when every reachable effect is resolved.
- Missing,
  dynamic,
  native,
  generated,
  or unmappable implementation remains uncertain.
- Lint execution is deterministic and offline.
- Cache hits survive separate Oxlint CLI processes.
- Cache validation is content-addressed and cannot trust executable cache content.
- Linux x64,
  macOS arm64,
  and Windows x64 remain supported.
- High-trust plugin code and every adopted dependency must be inspectable.
- No TypeScript 6 fallback is permitted.

Managed-service,
SaaS,
credential,
privacy,
residency,
and browser-runtime execution gates do not apply.
The browser overlay applies only to claims about Web API semantics and declaration identity.

## Frozen criteria

All soft criteria have weight 1 because no relative preference weight was supplied:

- sound parameter-reachable mutation and callback inference;
- exact TypeScript declaration-to-implementation mapping;
- support for ordinary installed JavaScript and TypeScript packages;
- deterministic fail-closed behavior for unsupported code;
- cross-invocation reuse;
- incremental invalidation correctness;
- synchronous Oxlint integration;
- source auditability;
- Linux,
  macOS,
  and Windows portability;
- bounded CPU,
  memory,
  process,
  descriptor,
  and disk use.

Hard-gate failures remain outside scoring.

## Frozen discovery schedule

### Package and ecosystem sources

- npm and broader registry search for `JavaScript static effect analysis mutation arguments`;
- npm and repository search for `TypeScript function parameter mutation analyzer`;
- npm and repository search for `JavaScript call graph static analyzer`;
- incumbent-plus-alternative search for `effect-analyzer alternative`;
- repository source for every discovered serious analyzer.

### Repository hosts

- GitHub search for `InferMutationAliasingEffects`;
- GitHub search for JavaScript points-to and call-graph analyzers;
- GitHub search for CodeQL JavaScript parameter data flow and library models;
- Oxlint source,
  docs,
  and tracker searches for JavaScript-plugin caching;
- TypeScript Go source and installed package searches for API sessions,
  sockets,
  snapshots,
  and persistence.

### Standards and runtime sources

- WHATWG Web IDL operations and extended attributes;
- Node API JSON documentation and Node source mapping;
- ECMAScript and host declaration provenance already audited by the existing intrinsic catalogs.

### Repository evidence

- current direct-summary cache and bridge lifecycle;
- current package,
  Node,
  ECMAScript,
  DOM,
  and Pi effect catalogs;
- existing external-call diagnostics and adapter contracts;
- current lockfile package identities;
- prior semantic-bridge vet and troubleshooting documents.

The expansion round added Jelly,
React Compiler mutation aliasing,
CodeQL JavaScript data flow,
Oxlint issue `#21672`,
and TypeScript LSP API sessions.
No later taxonomy term produced another synchronous,
sound,
TypeScript 7-compatible effect analyzer.
Discovery is saturated with one implementation survivor.

## Candidate ledger

### Extend the current TypeScript 7 analyzer

Discovery source:
 current plugin and prior semantic-bridge decision.

Base category:
 custom composition of inspectable open-source local technology.

Overlays:
 incumbent extension,
high-trust plugin execution,
and multi-platform support.

Lifecycle:
 selected for implementation with validation gates.

The current analyzer already resolves exact TypeScript signatures,
tracks parameter origins,
handles direct and transitive mutations,
callbacks,
closures,
owned call edges,
and exact audited host identities.
Extending the same call-resolution seam to shipped package implementations avoids a second AST and preserves current
diagnostics.

### Jelly

Discovery source:
 GitHub and npm static-analysis searches.

Base category:
 inspectable open-source local analyzer.

Lifecycle:
 hard-gate exit.

The Jelly README explicitly says its ECMAScript model is intentionally not fully sound and treats the Node standard
library as unknown code.
It analyzes complete entry graphs,
warns that dependency analysis can take a long time,
and documents memory tuning up to multi-gigabyte heaps.
Those properties fail the sound-observation and synchronous-plugin constraints.
Its call graph research remains useful comparative evidence,
but Jelly cannot be the effect authority.

Primary source:
[`cs-au-dk/jelly`](https://github.com/cs-au-dk/jelly),
README sections `Jelly`,
`Usage`,
and `Approximate interpretation`,
accessed 2026-07-13.

### React Compiler mutation and aliasing inference

Discovery source:
 repository search for JavaScript mutation inference.

Base category:
 inspectable open-source compiler subsystem.

Lifecycle:
 category mismatch and performance hard-gate exit.

The inference implementation operates on React Compiler's own HIR and is coupled to React-specific compiler semantics.
The current source file
`compiler/packages/babel-plugin-react-compiler/src/Inference/InferMutationAliasingEffects.ts`
is 2,975 lines before its supporting HIR,
range,
and validation modules.
Oxlint issue `#21672` records that a plugin invoking React Compiler reparses each file and changed one reported
3,500-file lint from 4.8 seconds to 23.8 seconds.
It is not a reusable package-call effect API and adds the duplicate parse the current bridge avoids.

Primary sources:

- [React Compiler inference source](https://github.com/facebook/react/blob/main/compiler/packages/babel-plugin-react-compiler/src/Inference/InferMutationAliasingEffects.ts),
  accessed 2026-07-13;
- [Oxlint issue `#21672`](https://github.com/oxc-project/oxc/issues/21672),
  opened 2026-04-23 and still open when checked 2026-07-13.

### CodeQL JavaScript data flow

Discovery source:
 CodeQL JavaScript data-flow search.

Base category:
 inspectable query libraries plus a prebuilt database extractor and CLI.

Lifecycle:
 integration and execution hard-gate exit.

CodeQL provides whole-program local and global data-flow libraries and supports custom library models.
It requires extracting a CodeQL database and running queries outside the synchronous rule visitor.
The extraction,
prebuilt CLI,
database lifecycle,
and query runtime cannot satisfy per-invocation Oxlint diagnostics or the current source-overlay boundary.
Its library models also move unknown package semantics into a separate maintained model set rather than inferring them.

Primary sources:

- [Analyzing data flow in JavaScript and TypeScript](https://codeql.github.com/docs/codeql-language-guides/analyzing-data-flow-in-javascript-and-typescript/),
  accessed 2026-07-13;
- [Customizing JavaScript library models](https://codeql.github.com/docs/codeql-language-guides/customizing-library-models-for-javascript/),
  accessed 2026-07-13.

### `effect-analyzer`

Discovery source:
 npm and prior semantic-bridge audit.

Base category:
 inspectable open-source TypeScript analyzer.

Lifecycle:
 category mismatch and TypeScript-version hard-gate exit.

The project analyzes Effect-TS program structure,
service dependencies,
errors,
and diagrams through `ts-morph`.
It is not a general parameter-reachable mutation analyzer and retains the TypeScript 6 stack rejected by the governing
requirements.

Primary source:
[`jagreehal/effect-analyzer`](https://github.com/jagreehal/effect-analyzer),
README and package source,
accessed 2026-07-13.

### Oxlint-owned persistent lint cache

Discovery source:
 Oxlint documentation and tracker.

Base category:
 incumbent host capability.

Lifecycle:
 unavailable.

Oxlint issue `#21672` specifically requests caching for expensive JavaScript plugins and remains open.
The current JavaScript-plugin documentation describes `createOnce`,
`before`,
and `after` process-local lifecycle hooks but no cross-process result cache.
The plugin cannot delegate persistence to Oxlint 1.73.

### Persistent TypeScript API session

Discovery source:
 TypeScript 7 installed declarations and TypeScript Go source.

Base category:
 incumbent compiler capability.

Lifecycle:
 synchronous-client hard-gate exit for current release.

`typescript/unstable/sync` exposes `API.fromLSPConnection`,
but the installed synchronous client rejects socket options before connecting:

```js
// node_modules/.pnpm/typescript@7.0.2/node_modules/typescript/dist/api/sync/client.js:10-13
if (!isSpawnOptions(options)) {
  throw new Error("Socket connections are not yet supported in the sync client");
}
```

The TypeScript LSP can create API sessions,
but this plugin cannot use them through the required synchronous client in version 7.0.2.
A custom daemon would add process ownership,
stale socket,
crash recovery,
and cross-platform named-pipe policy that the disk cache avoids.
This option becomes eligible when the supported synchronous client can connect to an existing session.

## Host metadata boundaries

### Web IDL

Web IDL provides API surface and JavaScript binding rules,
not complete API-specific effects.
The 2026-07-03 living standard says API-specific details are specified in prose:

> specifications describe their interfaces using Web IDL,
>  and then use prose to specify API-specific details.

Web IDL can identify callbacks,
dictionaries,
iterables,
maplike and setlike operations,
and binding-level conversion hooks.
It cannot prove whether an arbitrary Web API method mutates its receiver,
an argument,
hidden platform state,
or a dependent object.

Primary source:
[WHATWG Web IDL commit snapshot `fad9b4ce`](https://webidl.spec.whatwg.org/commit-snapshots/fad9b4ce284fd034b719c1c8576e1c692bc97de3/),
accessed 2026-07-13.

### Node API JSON

Node 26.5.0 publishes `all.json` with names,
versions,
stability,
parameters,
returns,
source document paths,
and prose descriptions.
The documented schema does not define parameter-mutation or observation fields.
The JSON is useful for exact version and source links,
not as proof of absence.
Node's contributing guide also states a one-to-one relationship between many documentation files and `lib/<module>.js`,
which can guide source resolution for JavaScript-backed built-ins.
Native bindings still require source evidence or remain uncertain.

Primary sources:

- [Node 26.5.0 `all.json`](https://nodejs.org/dist/v26.5.0/docs/api/all.json),
  accessed 2026-07-13;
- [Node API documentation tooling](https://github.com/nodejs/node/blob/main/doc/contributing/api-documentation.md),
  accessed 2026-07-13.

## Selected architecture

### Invocation-time inference

For each unresolved invoked package callable:

1. Resolve exact declaration and package identity from the active TypeScript project.
2. Resolve the package export's runtime file and shipped source map or declaration map.
3. Map the declaration owner and member to one implementation callable.
4. Analyze only that implementation and transitively reached callables.
5. Reuse the existing origin,
   callback,
   closure,
   mutation,
   and uncertainty summaries.
6. Treat dynamic exports,
   unresolved aliases,
   native binaries,
   generated code without source mapping,
   and ambiguous declaration-to-runtime mappings as uncertain.

Uninvoked package exports are not enumerated or analyzed.
The lockfile bounds accepted package identities and cache invalidation;
it is not an API work queue.

### Two-level cache

The selected cache has two layers:

- process-local project snapshots and final fixed-point indexes for subsequent files in one Oxlint run;
- content-addressed serialized direct summaries for reuse by later Oxlint processes.

Persistent entries contain data only,
never executable code.
Readers validate schema,
source digest,
analyzer digest,
TypeScript version,
platform-sensitive resolution identity,
package version,
config identity,
and callable spans before use.
Corrupt or partial entries are cache misses.
Writers use sibling temporary files and atomic rename.

The cache stores direct summaries rather than TypeScript object handles or symbol IDs.
Fixed-point state is cloned before propagation.
Changed source invalidates its own direct summary;
reverse call dependencies determine which final summaries need propagation again.
A full content digest check is allowed because hashing source bytes is not semantic recomputation.

The cache root follows the repository's existing cache convention under `node_modules/.cache` and supports an explicit
test override.
No cache entry is committed or published.

## Alternatives and ranking

### Content-addressed disk summaries plus process-local final indexes

Pros:

- survives independent Oxlint processes;
- preserves synchronous operation;
- stores inspectable data;
- invalidates from exact content and analyzer identity;
- requires no daemon ownership or network protocol;
- supports package-source summaries with the same representation.

Cons:

- TypeScript project startup still occurs once per Oxlint process;
- source digest checks still read changed or candidate files;
- schema and corruption validation are required;
- first use of a callable remains a full analysis miss.

### Persistent analyzer daemon

Pros:

- can retain TypeScript projects and decoded ASTs across CLI invocations;
- can avoid repeated compiler startup.

Cons:

- TypeScript 7.0.2's synchronous client cannot connect to an existing API socket;
- requires cross-platform process,
  socket,
  crash,
  version,
  and stale-state management;
- expands the trusted execution and denial-of-service boundary;
- duplicates language-server lifecycle responsibilities.

### Oxlint-owned cache only

Pros:

- host could skip unchanged files before entering JavaScript;
- no plugin-owned storage format.

Cons:

- unavailable in Oxlint 1.73;
- open upstream issue has no adopted interface;
- a file-only result cache would still need project-dependency invalidation for transitive effects.

Ranking:
 content-addressed summaries > persistent daemon > Oxlint-only cache.
The disk design outranks the daemon because it is available through the current synchronous API and has a narrower
failure boundary.
The daemon outranks waiting for Oxlint because it is technically implementable,
while the host cache does not exist.

## Scoring and sensitivity

Only the content-addressed extension survives every current hard gate,
so comparative soft scoring is not applicable.
The result is not a tie.
Changing any equal soft weight cannot promote a hard-gate failure.

The recommendation changes if either hard fact changes:

- TypeScript's synchronous client supports an existing API session;
- Oxlint provides a dependency-aware persistent JavaScript-plugin cache.

Those are explicit revisit triggers rather than low-confidence score ranges.

## Validation gates

Implementation is incomplete until it proves:

- a second independent Oxlint process reads unchanged direct summaries from disk;
- changed implementation source invalidates the exact source and transitive callers;
- changed analyzer source or package version invalidates old entries;
- corrupt,
  truncated,
  unknown-schema,
  and path-mismatched entries become misses;
- concurrent writers cannot expose torn JSON;
- deleted and renamed files are pruned without trusting stale state;
- cache growth has a measured bound and pruning policy;
- package source mapping accepts audited ESM,
  CommonJS,
  source-mapped,
  and TypeScript-source cases;
- dynamic exports and native modules remain uncertain;
- warm-run timing shows semantic scans are not repeated;
- Linux,
  macOS,
  and Windows workflows exercise cache read and invalidation;
- published external consumers can create and reuse the cache outside monorepo ancestry.

## Recommendation

Adopt invocation-time package-source inference on the current TypeScript 7 effect engine.
Add process-local final-index reuse and content-addressed persistent direct-summary caching before expanding package
lint migration.
Retain exact audited host evidence only where implementation source or machine-readable effect metadata cannot prove
the call.
Do not adopt Jelly,
React Compiler,
CodeQL,
`effect-analyzer`,
or a custom persistent daemon for this rule.

## Evidence limits

No external analyzer reached finalist execution because each failed a documented category,
soundness,
version,
or synchronous-integration hard gate before execution.
The selected custom extension has not yet passed the validation gates in this report.
The report must be updated with commands,
logs,
elapsed timings,
cache hit counts,
and platform results as implementation lands.
