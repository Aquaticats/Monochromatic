# Replace `prefer-readonly-parameter-types` with a project rule

Status:
 replacement implementation,
 migration,
 and migration-specific verification complete.

Last updated:
 2026-07-14.

## Implementation progress

The user confirmed shared understanding and authorized implementation.

### Final verification record

The migration-specific acceptance gate is complete.

- Final single-worker root process `proc_367` ran `OXLINT_THREADS=1 mise run lint:oxlint` over 2,548 files in
  816.5 seconds.
  It reported 3,792 warnings and 665 errors from existing non-readonly workspace findings.
  Its captured output contains zero occurrences of the replacement rule ID,
  `SemanticBridgeError`,
  the omitted-owned-callable failure,
  or `context canceled`.
- Final root process `proc_368` ran `mise run test:unit` successfully from each test's nearest package task root.
  Repeated full-fanout processes `proc_331` and `proc_332` also passed after the lock and cache race fixes.
  Default discovery selects 508 active non-expensive tests.
  `--all` selects 512 active tests,
  including 4 expensive tests.
  The 64 inactive-tree tests leak into neither selection.
- Package type fanout checked 122 supported package tasks with no failures.
  The semantic-host and disposable external-consumer tasks passed in `proc_274` and `proc_273`.
- The semantic plugin's final build,
  type,
  unit,
  and Oxlint tasks passed in `proc_352` and `proc_362` to `proc_364`.
  Commits `65e7e5cdb` and `466ca8955` index 479 audited effects by owner and member before exact provenance matching,
  then verify every catalogue entry through that index.
  Repeated unchanged single-worker runs `proc_365` and `proc_366` completed in 9.3 and 9.4 seconds with zero findings.
  Commit `c0b403dc6` isolates the independent-process persistent-cache test from concurrently changing fixture projects.
- TypeScript synchronous API shutdown now guards both the main semantic client and demand-driven external-project
  clients.
  The final root sweep and package tests contain no native-child cancellation output.
  GitHub Actions run `29396066153` passed native lifecycle,
  platform path,
  and external-consumer verification on Ubuntu,
  macOS 15,
  and Windows.
- Done's self-contained browser bundle passed create and search flows against `DB_PATH=:memory:` with no console or page
  errors.
- File-enforcer now stages lock-owner metadata privately and publishes it by same-directory rename.
  Commit `d2240c4c5` adds a deterministic pause at the stage-to-publish boundary,
  where contenders observe no owner before publication and a live complete owner afterward.
  Commits `965d4cf32` and `b8a649536` use one-shot release operations
  and prevent a vanished predecessor observation from
  deleting its successor.
  Commit `4e28505f8` releases 8 writers together against one manifest lock.
  Commit `920bc0439` starts a real competing `acquireManifestLock()` while owner metadata remains privately staged,
  proves that acquisition stays pending,
  then proves it succeeds only after publication and predecessor release.
- Commit `764a3a766` guards the intended generated-policy boundary:
  canonical repository and forbidden-string policies retain the lazy-provider contract,
  while generated Git CLI mirrors omit the stale inlined contract.
  Final file-enforcer build,
  type,
  unit,
  and Oxlint tasks passed in `proc_327` to `proc_330`.
- Generated files are synchronized and idempotent.
  `git diff --check` passed,
  and working-tree review preserved unrelated desktop,
  Cargo,
  lockfile,
  Slopo,
  audit,
  and license changes.

Root dprint,
Stylelint,
Markdown,
and type-aware Oxlint still have pre-existing findings outside this migration.
They remain separate work rather than being hidden,
suppressed,
or rewritten as readonly acceptance failures.

### Current continuation state

The user corrected the TOML migration on 2026-07-13:
`ForeignBorrowed` is an ownership-boundary marker,
not a type annotation to repeat on every AST descendant or callback.
The correction is now recorded in the prefer-readonly guidance message
and `package/ownership-marker/foreign-borrowed/README.md`.

The implemented semantic rule applies that correction:

- exact marker identity is separated from provenance analysis;
- foreign provenance passes through property and element access,
  nested destructuring,
  aliases,
  owned calls,
  audited intrinsic callbacks,
  and synchronous `for...of` bindings;
- a callee parameter is foreign only when every owned inbound call supplies wholly foreign mutable state;
  an ordinary owned call removes that guarantee;
- object and array packages ignore primitive and soundly readonly siblings,
  but one foreign field cannot hide an owned mutable sibling;
- active nested-closure calls no longer create duplicate foreign inbound edges;
- direct callback invocation is tracked separately from referent mutation;
  unknown callback capabilities still require `@mutates`,
  while pure and throwing owned callbacks no longer make captured readonly values unsupported;
- focused fixtures cover callback,
  property,
  element,
  destructuring,
  synchronous iteration,
  pure callback,
  throwing callback,
  and mixed foreign/owned inbound paths;
- exact intrinsic evidence now includes Array `with` and `reduce`,
  proxy-capable Object reflection,
  mutating `Object.freeze`,
  and receiver-only `TextDecoder.decode`.

The completed TOML migration marks parser ingress,
retained AST storage,
and exported AST-emission seams,
then relies on provenance instead of descendant markers.
The durable mechanism and verification record is the
[foreign-provenance troubleshooting guide](../troubleshooting/oxlint-prefer-readonly-foreign-provenance.md).
The package now passes type lint,
Oxlint with no findings,
the unit suite,
TOML 1.0 and 1.1 conformance,
and the deterministic fuzz-coverage gate.
A campaign exposed the known `toml-eslint-parser` static-value prototype-setter limitation;
the oracle now classifies that unsupported input explicitly while regression-testing the package's prototype-safe
materializer.
The diagnosis is recorded in the
[`toml-eslint-parser` prototype-setter guide](../troubleshooting/toml-eslint-parser-static-value-prototype-setter.md).

Workspace packages must import one another through `/ts` source subpaths.
A working-tree `rg` scan over `.ts`,
`.mts`,
`.cts`,
and `.tsx` files found 74 marker imports using the package root and 105 already using `/ts`.
After correction,
the same scan finds 179 `/ts` imports and no marker root import.
Commit `300ccac29` records 58 standalone rewrites;
the remaining corrected sites belong to the current semantic-plugin and TOML changes.
`AGENTS.md` rule `ST3` records the workspace-wide convention,
and file-enforcer synchronized `CLAUDE.md`.

The current plugin passes `lint:types`,
`lint:oxlint`,
and `test:unit`.
Direct summaries use content-addressed persistent JSON keyed by analyzer implementation,
TypeScript version,
project graph,
lockfile,
compiler options,
source path,
and source contents.
Full nested payload validation,
entry-size limits,
atomic writes,
and age/count/byte maintenance make malformed or stale entries conservative misses.
A process-local final-index cache avoids repeated fixed-point propagation within one stable Oxlint input snapshot.
Closing the semantic bridge clears process caches before another lifecycle.
Focused tests cover independent-process hits,
project-dependency invalidation,
and corrupt nested payload rejection.
Invocation-time locked-package inference now resolves package exports,
shipped JavaScript or TypeScript,
runtime re-exports,
overloads,
source-map evidence,
object methods,
and static or instance class methods.
Missing implementations,
dynamic targets,
unknown transitive calls,
and unresolved callback data relations remain fail-closed.
Host intrinsic resolution now requires structured authority in addition to exact declaration identity.
ECMAScript and browser entries require exact standard commits,
authoring-source digests,
and algorithm identities.
Node entries require exact `@types/node` major,
runtime version,
embedded JavaScript module,
source digest,
and callable-definition marker.
Unknown host calls,
native-only implementations,
and authority drift remain fail-closed.
The durable evidence constraints are recorded in the
[host intrinsic evidence troubleshooting guide](../troubleshooting/oxlint-prefer-readonly-host-intrinsic-evidence.md).

The TSDoc contract phase is complete:

- commit `7c25431a6` specified parser and fixture behavior;
- commit `dbde47701` registered and validated `@mutates`;
- commit `0c7931fa0` documented external TSDoc registration behavior;
- commit `cac9e6a1d` added bounded cross-rule parsed-body reuse;
- commit `d6ea31e25` covered method,
  call,
  and ambient signatures.

The semantic-bridge foundation is complete:

- commits `9283e2e7a` and `de9578f10` declared and locked TypeScript 7 as a runtime dependency;
- commits `12071895f` and `39c2ac5e9` added tagged lifecycle logging;
- commit `73aecdf47` added configured-project discovery,
  virtual overlays,
  snapshot disposal,
  BOM-aware node mapping,
  and fail-closed errors;
- commit `32d84ac10` added exact owner,
  member,
  provenance,
  evidence,
  and package-major intrinsic effects;
- the built overlay test exposed stale semantics when `openFiles` persisted;
  the adapter now uses it only for project discovery before switching to `openProjects` and `closeFiles`;
- the user corrected ambiguous virtual-filesystem callback names;
  commit `a53861211` names both callbacks and return types by overlay and delegation behavior,
  and commit `07d2c6934` records that naming standard in `XNC`.

The semantic-rule implementation and shared-configuration migration are complete:

- commits `d2d6b8521` and `c76f39219` added exact intrinsic lookup and recursive readonly classification;
- commit `353f2fd9a` added direct,
  intrinsic,
  cross-file,
  recursive,
  opaque,
  and immediate-callback effect summaries;
- commit `b29e44eec` moved `@mutates` extraction behind the shared plugin parser seam;
- commit `2909ca22c` traces direct,
  local-alias,
  and destructured parameter origins;
- commit `f02f16ea2` registered the replacement rule and its core diagnostics,
  covered bodyless source signatures,
  and failed closed on unresolved external calls;
- commit `b5afc9274` prohibited line,
  block,
  and mixed-list inline suppressions;
- commit `8258a37e9` exercised the core diagnostics through Oxlint;
- commit `d3fd789bc` retained opaque provenance and structurally verified local adapters;
- commit `39aeaea3c` kept ordinary fixes inert while exposing stale-contract removal as an explicit suggestion;
- commits `acd78d72a` and `ff09e0e69` fixed deep readonly collections and fail-closed callable capabilities;
- commit `55c849541` verifies implementation effects against bodyless overload-contract unions;
- commit `cb40ebe48` suggests `readonly T[]` only when element semantics prove the deep rewrite;
- commit `aadbab490` excludes dead nested closures while retaining invoked,
  returned,
  and directly passed deferred effects;
- commit `4901811d2` traces separately assigned aliases and excludes local parameter rebinding;
- commit `089c19f93` rejects adapter links that do not identify the opaque callable member;
- commit `81e53f211` propagates opaque provenance transitively through owned calls and callback relations,
  and exposes retained boundary names in diagnostics;
- commit `c8ddcf53d` activates passed and returned closures through local initializer aliases while retaining dead
  function-expression exclusion;
- commit `82acd6523` resolves aliased callback arguments for higher-order effect specialization;
- commit `609b3b824` offers suggestion-only `ReadonlyDeep` projections for capability-free structural data when a
  named `type-fest` import already resolves;
- commit `f10bc94a9` audits exact non-dispatching `Array.isArray` and `Object.is` calls while retaining
  `JSON.stringify` opacity;
- commit `9189aea99` propagates effects from closures in returned,
  passed,
  and caller-reachable stored containers while requiring active closure ancestry;
- commit `fdb6335ea` bounds overlays to the active file,
  caches configured roots instead of files,
  classifies created and deleted paths,
  and covers parser recovery,
  rename,
  symlink,
  and cache lifecycle;
- commit `a2ed792a6` packs a production staging manifest and exercises the installed artifact plus TypeScript 7 bridge
  from a disposable external consumer;
- commit `46ef12790` automates declaration preservation for function,
  overload,
  and call-signature mutation blocks through bundled re-exports;
- commit `677498da6` covers duplicate package majors,
  aliases,
  subpaths,
  and unresolved package metadata;
- commit `7efe445aa` adds exact standard-library `Array<T>` to `ReadonlyArray<T>` suggestions;
- commit `03a2a0618` caches exact-source direct effect scans after a workspace shadow run consumed CPU for more than ten
  minutes without completing;
- commit `b6dcba729` rejects inferred-project sentinels without poisoning later configured snapshots
and skips non-enforced
  or declaration inputs;
- commit `42478177d` catalogs exact observational String,
  identity-search,
  membership,
  `Error.isError`,
  and imported `node:path` calls;
- workflow run `29227040562` passes Linux x64,
  macOS 15 arm64,
  and Windows x64 host evidence,
  including native bridge lifecycle,
  platform path behavior,
  and external-consumer installation;
- a sustained local bridge probe sampled resident memory after forced collection across 2,000 post-warmup overlay opens.
  Cache stats remained at one overlay and one project root.
  Resident memory reached 102,727,680 bytes during the first sampled phase and rose by 1,310,720 bytes during the next
  1,000 opens,
  reaching 104,038,400 bytes rather than retaining each snapshot;
- commit `fb9fd3ab5` adds Windows-host acceptance for noncanonical source-path casing;
- commit `ccb5bfd95` enables replacement and no-disable rules in shared configuration,
  disables native rule explicitly,
  and adds agreed declaration and test exemptions;
- commit `2d0bb636a` removes retired native-rule directives from active source without disturbing mixed directives;
- commit `4eeb356b5` excludes primitive-only values from opaque caller-state effects;
- commit `8668d4363` adds explicit callback relations for Array and collection operations;
- commit `b4704a308` audits primitive-element Array `join` and exact Node `Socket.write` mutation;
- commit `d23f96871` rediscovers nested configured projects when cached containing project omits current source;
- commit `026f7ae08` introduces exact `ForeignBorrowed` ownership marker for externally dictated mutable handles,
  migrates plugin's foreign Oxlint boundaries,
  audits TypeScript 7 and Oxlint package operations,
  and propagates destructured object-literal call effects by declared mutation target;
- commit `dbb5616ee` moves `ForeignBorrowed` into shared plugin configuration for package-wide foreign-boundary use;
- commits `1c662eb1a` and `6a21274b0` migrate TSDoc and stylistic plugin foreign inputs and verified context effects;
- commits `f3c61a669` and `807cdcb2c` preserve current callable identities across direct-summary cache hits
and always index
  configured active source despite stale external-library metadata;
- a repeat one-worker workspace sweep reports zero semantic bridge failures across 1,315 replacement-rule diagnostics;
- commit `37e48c19b` removes caller-owned coercion hooks from `module-or-throw` failure formatting;
- unknown-call diagnostics now distinguish method receivers from other call inputs,
  explain that methods can change controlled state without assignment,
  name only affected destructured bindings,
  and enumerate every supported remediation;
- audited Pi `0.80.6` package effects record `ExtensionAPI.appendEntry`,
  `ExtensionAPI.on`,
  `ExtensionAPI.registerTool`,
  and `ExtensionAPI.setThinkingLevel` as exact receiver mutations;
  `ExtensionAPI.getThinkingLevel` is observational;
- commit `a74fe0ba3` excludes packaged primitive leaf values from opaque caller-state effects;
- commit `a9a93514f` audits `Array.prototype.toSorted` as observational only for primitive receiver elements;
- commit `49a81ddd7` replaces page-weight's object-capable `String(error)` fallback with noncoercing runtime-category
  formatting and proves caller `toString` is not invoked;
- commit `baa102b18` identifies exact global `String` conversion,
  accepts primitive inputs,
  and emits a dedicated object-coercion diagnostic naming every hook and supported remedy;
- commit `8054e3764` accepts deliberately coercing `unknown` through a complete `@mutates` hook contract,
  classifies `unknown` and `any` as opaque capabilities rather than readonly claims,
  and proves same-named external callables retain ordinary treatment;
- commits `51cbaea0d` and `1090b762d` prove noncoercing error formatting avoids ordinary methods,
  `Symbol.toPrimitive`,
  `valueOf`,
  accessors,
  and proxy property traps;
- commit `5a66e8b2f` proves incomplete global `String` mutation contracts remain unresolved diagnostics;
- commit `1f18c8edf` declares the real Voyage request serialization effect without changing caller assignability;
- commit `3b6b0685f` audits exact `TextEncoder.encode` as observational under the WHATWG algorithm;
- commit `8e7e6c8d9` distinguishes zero-effect intrinsic methods from mutating capabilities,
  preserves named type-alias package provenance,
  and audits `StdoutWriter.write` as receiver mutation;
- after the user rejected an absent `message` mutation contract as unsupported,
  commit `c6e026a82` moves MCP response serialization to ownership-known call sites
  and passes primitive text through the
  generic writer;
- after the user required a dedicated package seam,
  commit `092963df8` moves semantic rule source,
  focused tests,
  host acceptance,
  publication acceptance,
  namespace,
  and shared-config sidecar under `package/oxlint-plugin/prefer-readonly-parameter-type`;
- the same commit satisfies the user's narrow ubiquitous-dependency requirement through
  `package/ownership-marker/foreign-borrowed`,
  which contains one type declaration,
  no runtime code,
  and no runtime dependencies;
- commit `d370083d4` severs caller origins when primitive elements are copied into a fresh array before an opaque call;
- commits `ddd47efeb`,
  `b1e4c55d6`,
  `b8088e3ca`,
  `9210628dd`,
  `52dde2957`,
  `9040ac96d`,
  `1bb39c3b5`,
  `77543359c`,
  `607374e91`,
  `df1877225`,
  and `ecfb3ec7f` migrate additional exact Pi,
  authoring,
  serialization,
  DOM,
  error,
  and ownership boundaries;
- commit `b72d5e979` adds `mise run lint:oxlint` as a single-worker repository sweep;
- commits `e3317a477`,
  `a576a9db9`,
  and `25ec90308` migrate catalog tightening,
  terminal-title registry handling,
  and git-clone-size ownership boundaries;
- commit `d12de9816` resolves exact ambient `AbortSignal.any` identity,
  records its dependent-signal mutation,
  and propagates sound contracts through git-clone-size;
- commit `3d90851aa` originally treated direct callback invocation as a known capability mutation,
  applies nonzero imported-callable catalog targets,
  and expands exact Pi effect evidence;
  the current continuation supersedes the function-object mutation model with a distinct invoked-capability effect;
- commit `fe9e0b99e` migrates Advisor and shared model-selection capability contracts;
- commit `3ee0ab774` limits imported callable effects to audited option fields
  and verifies each added Pi method against real Advisor source;
- commit `a717266a9` audits additional Pi shortcut,
  UI,
  command-context,
  and model-registry capabilities against real auto-mode source;
- commit `35f48181f` audits global timer and workspace helper boundaries
  and tracks caller-owned async iterator consumption;
- commit `4efbc821e` preserves nested origins for ordinary parameter contracts
  and audits Pi tool-event predicates through exact package provenance;
- commit `97d59cff8` migrates auto-mode Pi,
  provider-stream,
  callback,
  iterator,
  and canonical serialization boundaries;
- the 2026-07-13 sweep reported 1,014 replacement-rule diagnostics with no bridge-failure category:
  585 uncertain calls,
  272 readonly projections,
  108 unsupported declarations,
  48 missing contracts,
  and 1 stale contract;
  later package commits in this list postdate that baseline;
- a later sweep before the Advisor and direct-callback commits reported 965 diagnostics:
  533 uncertain calls,
  265 readonly projections,
  103 unsupported readonly declarations,
  63 missing contracts,
  and 1 stale contract,
  with no semantic bridge failure;
- the root sweep after commits `3d90851aa`,
  `fe9e0b99e`,
  and `3ee0ab774` reported 1,049 diagnostics:
  496 uncertain calls,
  246 readonly projections,
  181 missing contracts,
  125 unsupported readonly declarations,
  and 1 stale contract;
  direct callback invocation moved previously hidden effects into enforceable missing contracts,
  Advisor and shared model selection each reported zero diagnostics,
  and no semantic bridge failure was present;
- capability declarations were corrected before observational data contracts;
  mapped readonly projections were removed from callable and stateful capabilities,
  while genuine host,
  provider,
  parser,
  filesystem,
  and retained callback ingress uses exact `ForeignBorrowed` boundaries;
- observational inputs now use explicit readonly data structures or `ReadonlyDeep` only
when the complete reachable value
  is capability-free;
  exported hyperscript and Penpot option bags retain their mutable public declarations,
  and reflective `Object.entries`,
  JSON,
  and state inputs retain sound mutable effects;
- audited intrinsic result provenance now distinguishes pure receiver-value copies from mixed results:
  `slice`,
  `filter`,
  `find`,
  `findLast`,
  `toReversed`,
  and `toSorted` preserve receiver-value ownership,
  while `with` does not claim receiver-only provenance;
  aliases retain audited result origins;
- `toSorted` analyzes definitely callable comparators,
  preserves result ownership,
  and fails closed for absent,
  explicitly undefined,
  maybe-undefined,
  unresolved,
  or otherwise non-callable comparators when receiver elements are nonprimitive;
- the immutable root sweep recorded in process `proc_17` reported exactly 360 opaque-effect diagnostics and no readonly
  preference,
  unsupported declaration,
  missing contract,
  stale contract,
  semantic bridge failure,
  or omitted callable summary;
- commits `9df97a944` and `c4575ccf5` correct readonly classification for broad `object` and direct callable or
  constructable capabilities;
  these types can expose caller-defined behavior even without statically named mutable properties;
- the test-harness phase resolves its measured 35 opaque effects through exact Chai,
  Sinon,
  Promise-handler,
  reflection,
  coercion,
  and formatting contracts;
  `package/test-harness/core` now passes type lint,
  Oxlint with no findings,
  build,
  and unit tests;
- the serialization phase resolves every isolated `JSON.stringify` boundary through complete local contracts and sound
  mutable persistence DTOs;
  observational consumers retain readonly views,
  broad sink-only serializers accept `object`,
  callback boundaries were flattened where they only propagated serialization uncertainty,
  and no shallow mutable intersection is used to evade classification;
- terminal-title exhaustiveness retains its `never` proof and fixed fallback message instead of coercing an impossible
  runtime value;
- stable root process `proc_21` reports 263 opaque-effect diagnostics and no readonly preference,
  unsupported declaration,
  missing contract,
  stale contract,
  semantic bridge failure,
  or omitted callable summary;
  seven findings still mention serialization only because they share a boundary with `Response.json`,
  provider fetch,
  archive writes,
  or root configuration writes;
  the host/provider and residual phases own those mixed boundaries;
- the coercion phase centralizes arbitrary caught-value rendering in
  `@monochromatic-dev/caught-value-core`;
  `caughtValueText` preserves `Error.message` and exact JavaScript string conversion,
  while `caughtValueStack` preserves available stack detail;
  both expose conversion hooks through one authoritative `@mutates` contract;
- the audited workspace-package catalogue maps both caught-value helpers to their exact first-argument effect,
  so package-local semantic projects preserve fail-closed behavior without package-local formatter implementations;
- `no-restricted-syntax/prefer-caught-value-text` rejects duplicate Error-and-fallback formatters while accepting
  domain-specific alternate branches;
- stable root process `proc_26` reports 239 opaque-effect diagnostics and no readonly preference,
  unsupported declaration,
  missing contract,
  stale contract,
  semantic bridge failure,
  omitted callable summary,
  global `String` coercion finding,
  or duplicate caught-value formatter finding;
  the full Oxlint command still fails on remaining opaque findings and broader current workspace diagnostics,
  which belong to the host-provider,
  residual,
  and final-verification phases;
- the TypeScript sync adapter reuses byte-identical sources from its current immutable snapshot instead of clearing
  TypeScript's decoded-source cache and issuing one synchronous snapshot RPC per linted file;
  dependency-overlay regression coverage proves unchanged importers reuse the refreshed snapshot;
  final one-worker warm checks completed the semantic plugin package in 6,411 ms and 6,394 ms,
  `package/git-policy/cli` in 4,261 ms and 4,231 ms,
  and file-manager Electron in 6,887 ms and 6,896 ms;
  cold runs after analyzer or project source changes still rebuild content-addressed effect summaries;
- commits `1622488ad` and `619e0682d` audit ECMA-402 date formatting,
  migrate Feedsmith type names,
  and document RSS iterable scheduling effects;
  the RSS package passes build,
  type lint,
  and Oxlint;
- commits `3651f7eb0`,
  `f31f31953`,
  and `a045c0377` add pinned TypedArray and Node observation evidence,
  preserve callback `thisArg` uncertainty,
  and distinguish PostCSS mutation from observation or callback uncertainty;
- commit `9f2aa1660` audits Optique `1.1.1` parsing against authored and shipped source;
  readonly argument buffers are observational,
  while parser and option capabilities remain opaque;
- commits `f309c615c`,
  `0db781e4a`,
  `09da4a903`,
  `3f159cb1e`,
  `3fa5be6e1`,
  and `218da9e1e` complete Git policy CLI grouping,
  promise assimilation,
  Valibot validation,
  Error cause,
  and VFS or native write boundaries;
  the package passes type lint,
  Oxlint,
  build,
  and unit tests;
- commits `d9ae420f3` and `3cebfa881` propagate sound PostCSS clone,
  error,
  and stringifier uncertainty through the CSS mixin implementation;
  the CSS package passes type lint,
  Oxlint,
  build,
  and unit tests;
- commits `01cb2752f`,
  `db1b51186`,
  `f17220bcf`,
  `20d7d40d3`,
  and `9f537124b` audit Lezer and workspace TOML observations,
  remove unsupported watcher option projections,
  and propagate timer,
  async-context,
  rejection,
  and edit effects through file-enforcer;
  file-enforcer passes type lint,
  Oxlint,
  build,
  and unit tests;
- host audits landed in commits `728fdfbde`,
  `43401eb56`,
  and `68a0e2f71`.
  They cover locale date formatting and byte concatenation,
  replace unsupported standard grouping calls
  with encounter-ordered grouping,
  and expose DOM,
  crypto,
  and logger retention effects in `Aquati.cat`;
  the package passes type lint,
  Oxlint,
  build,
  and unit tests;
- commits `4160b27a1`,
  `67592dce1`,
  `efb6b7b8f`,
  `99a536320`,
  and `def95c837` audit `DataView` reads and propagate `Buffer` conversion effects through Kiwi canvas and ZIP parsing;
  Kiwi passes type lint,
  Oxlint,
  and build;
  it has no package unit-test task;
- host and provider task 33 is complete;
- stable root process `proc_117` reports exactly 40 readonly semantic diagnostics,
  down from 168 in `proc_38`,
  with no semantic bridge failure,
  omitted callable summary,
  or duplicate caught-value formatter finding;
  the remaining Git,
  parser,
  archive,
  graphics,
  callback,
  response,
  and runtime boundaries belong to residual task 34;
  package-level host-provider migration continues before the next root sweep;
- residual task 34 is complete;
  commits from `afd5c7ba0` through `e21bac927` add exact effects for Canvas,
  DataView,
  regular expressions,
  reflection,
  `ignore`,
  workspace archive and file-enforcer calls,
  and Node buffer or process APIs;
  `20216fd16` applies arity conditions to imported and global calls as well as receiver methods;
- residual package migrations now cover Git policy CLI,
  repository and forbidden-string scanning,
  Git clone sizing,
  terminal execution,
  MVM,
  task utilities,
  TOML editing,
  watcher restart,
  dependency-cube rendering,
  Electron infrastructure,
  hall monitor,
  import-attribute transformation,
  Penpot,
  and Kiwi;
  their focused Oxlint and type tasks pass after the final source changes;
- root process `proc_191` reduced the replacement rule from 40 diagnostics in `proc_117` to 4:
  one root file-enforcer unsupported declaration,
  one Pi spawn readonly preference plus its stale contract,
  and one Git wrapper uncertainty;
  `518cdfb95`,
  `5f0151d13`,
  and the overload-sensitive catalog work resolve those measured cases;
  the next root sweep is the final zero-baseline gate;
- commits `238887716` through `5efde8492` add semantic argument-type conditions and behavioral tests;
  `Buffer.from(Uint8Array)` copies bytes while exposing proven conversion-property hooks,
  `Buffer.from(ArrayBuffer)` sharing remains fail-closed,
  `ZipWriter.add` distinguishes owned string encoding from retained byte views,
  and `overwriteEach` distinguishes eager file arrays from lazy callable builders;
- Kiwi no longer carries coarse `Buffer.from(Uint8Array)` uncertainty contracts;
  Penpot retains archive uncertainty only where caller-owned storage-object bytes are actually retained;
  two-argument and three-argument `child_process.spawn` behavior is covered by the semantic fixture;
- `3e1851bc7` and `a52f7ea13` remove the reverse development dependency from
  `module-caught-value` to `module-test`;
  caught-value consumer tests now live in `package/test-harness/core` and the caught-value package task delegates to those
  exact tests;
- commits `c3a01aa05` through `f815b434b` make `prefer-caught-value-text` resolve global `Error` and `String`
  bindings before reporting and cover block-bodied arrow formatters without exceeding the source-line limit;
- `package/monorepo-task/core-exec`,
  `package/oxlint-plugin/no-restricted-syntax`,
  `package/oxlint-plugin/prefer-readonly-parameter-type`,
  `package/oxlint-plugin/tsdoc`,
  `package/oxlint-plugin/stylistic`,
  `package/or-throw/core`,
  `package/page-weight/cli`,
  `package/llm/image-diff`,
  `package/mcp/stdio`,
  `package/mcp/mvm`,
  `package/zip-writer/core`,
  `package/dependency-catalog/core-pnpm`,
  `package/fs-id/core`,
  `package/dom/core`,
  `package/test-harness/core`,
  `package/jsonc-edit/core-ts`,
  `package/mutation-test/cli`,
  `package/dependency-catalog/cli-tighten`,
  `package/agent-harness-shared/terminal-title`,
  `package/git-clone-size/cli`,
  `package/pi-shared/model-selection`,
  `package/pi-plugin/advisor`,
  `package/pi-plugin/auto-mode`,
  `package/pi-plugin/current-time-context`,
  `package/pi-plugin/terminal-title`,
  `package/pi-plugin/agent-settled-notification`,
  `package/pi-plugin/thinking-default`,
  `package/agent-harness-shared/session-discovery`,
  `package/typeface/aquaticat`,
  `package/android-exempt-unused/cli`,
  `package/terminal-exec/cli`,
  and `package/wc/app` now pass package Oxlint under replacement rule with one JavaScript-plugin worker.

Final root semantic,
consumer,
publication,
browser,
performance,
and workspace verification gates are complete.

Verified package tasks:

- `mise run buildAndTest --` with TSDoc parser,
  cache,
  and integration tests;
- `mise run //package/oxlint-plugin/tsdoc:lint:types`;
- `mise run //package/oxlint-plugin/tsdoc:lint:oxlint`;
- `mise run //package/config/oxlint:lint:types`;
- `mise run //package/config/oxlint:lint:oxlint`;
- `mise run //package/monorepo-task/core-exec:lint:oxlint`;
- `mise run //package/oxlint-plugin/no-restricted-syntax:lint:types`;
- `mise run //package/oxlint-plugin/no-restricted-syntax:lint:oxlint`;
- `mise run //package/oxlint-plugin/no-restricted-syntax:test:unit`;
- `mise run //package/caught-value/core:buildAndTest`;
- built-package consumer import and calls from `package/page-weight/cli`;
- `mise run //package/oxlint-plugin/prefer-readonly-parameter-type:build:js:node`;
- `mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:types`;
- `mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:oxlint`;
- `mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit`;
- `mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:semantic-bridge-host`;
- `mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:external-consumer-host`;
- `mise run //package/pi-shared/model-selection:lint:oxlint`;
- `mise run //package/pi-shared/model-selection:test:unit`;
- `mise run //package/pi-plugin/advisor:lint:oxlint`;
- `mise run //package/pi-plugin/advisor:test:unit`;
- `mise run //package/pi-plugin/advisor:verify:extension`;
- `mise run //package/pi-plugin/auto-mode:lint:oxlint`;
- `mise run //package/pi-plugin/auto-mode:build:js:node`;
- `mise run //package/pi-plugin/auto-mode:test:unit`;
- `mise run //package/test-harness/core:lint:types`;
- `mise run //package/test-harness/core:lint:oxlint`;
- `mise run //package/test-harness/core:buildAndTest`;
- `mise run //package/mutation-test/cli:buildAndTest`;
- `mise run //package/jsonc-edit/core-ts:buildAndTest`;
- `mise run //package/git-policy/cli:build`;
- `mise run //package/git-policy/cli:test:unit`;
- `OXLINT_THREADS=1 mise run lint:oxlint` in stable root processes `proc_21` and `proc_26`.

Next action:
keep unrelated workspace lint baselines outside this completed migration.

## Continuity contract

This session is expected to cross repeated automatic context compactions.
This document is the canonical task state,
not a summary written only at the end.
Update it whenever research changes feasibility,
the user corrects an assumption,
a decision is settled,
a candidate exits,
a probe completes,
or the next action changes.

Keep the following recoverable from repository documents:

- user's actual request and subsequent scope corrections;
- measured repository facts and exact source locations;
- external candidates,
  versions,
  source revisions,
  evidence,
  and unresolved gates;
- discarded hypotheses and why they failed;
- settled decisions and any still-ordered question queue;
- verification commands,
  relevant outputs,
  changed files,
  and commit hashes;
- one explicit next action for a continuation session.

Compaction prompts do not change scope or authorize implementation.
After compaction,
resume from this document and the task list,
then re-derive the next action from the user's request and recorded decisions.

## Decision posture

The user set implementation time and money to unlimited for this replacement.
Do not rank an option lower because it takes longer,
requires more engineering,
uses more analysis passes,
or needs a wider migration.
Correctness,
semantic coverage,
explainability,
and durable maintenance decide between feasible designs.

When removing cost constraints leaves one option that strictly covers the others without weakening those qualities,
adopt and record it without asking.
Ask only when alternatives encode genuinely different policy or correctness outcomes.

## Goal

Retire Oxlint's type-aware `typescript/prefer-readonly-parameter-types` rule and replace its useful policy with a
project-owned JavaScript rule in dedicated
`@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type`.

The replacement must reduce false positives and configuration maintenance without claiming guarantees that its chosen
analysis pipeline has not proved.

## Measured baseline

A repository scan on 2026-07-12 found:

- 372 textual references to `prefer-readonly-parameter-types`;
- 154 `oxlint-disable` directives naming the rule across 113 files;
- 55 active files with those directives after excluding `package-paused/`;
- 809 lines across the four dedicated config and allow-list files;
- 363 allowed type names,
   comprising 190 TypeScript library names and 173 package type names;
- 122 uses of `ReadonlyDeep` across the repository;
- one output-level false-positive suppression in
  `package/monorepo-task/core-exec/src/oxlint-suppress.ts`.

The active disable reasons include external SDK types,
mutable-by-design accumulators and caches,
branded primitive intersections,
callback-bearing declarations,
and generic types whose caller-owned identity must be preserved.
The current rule is therefore enforcing several different concerns through one deep structural readonly test.

## Confirmed host constraint and chosen bridge

Oxlint 1.73 does not supply its JavaScript plugins with TypeScript type information.
The installed `@oxlint/plugins` declaration says parser services are unavailable,
and a disposable probe run with `--type-aware` observed an empty `context.sourceCode.parserServices` object.
Oxlint's source separately runs the regular linter,
 which hosts JavaScript plugins,
and the tsgolint type-aware linter.

The source trace and runnable probe are recorded in
[`doc/troubleshooting/oxlint-js-plugin-type-information.md`](../troubleshooting/oxlint-js-plugin-type-information.md).

This establishes only what the Oxlint host supplies.
It does not establish that a JavaScript plugin must remain syntax-only.
The plugin can load other parser,
semantic-analysis,
declaration-generation,
or TypeScript compiler components.

The selected independent bridge is TypeScript 7.0.2's `typescript/unstable/sync` API
(`typescript/unstable/sync` was renamed `typescript/sync` in 7.1.0-beta,
2026-10-20).
One project-owned adapter owns the native client,
project discovery,
snapshots,
virtual current-file overlays,
source-node lookup,
semantic queries,
and fail-closed bridge diagnostics.
TypeScript 6 fallbacks are prohibited.
The complete technology vet and remaining implementation acceptance gates are recorded in
[`doc/audit/tech-readonly-parameter-semantic-bridge-vet-2026-07-12.md`](../audit/tech-readonly-parameter-semantic-bridge-vet-2026-07-12.md).

## Architecture

The implementation is split at its runtime and policy boundaries:

- semantic rule implementation and tests:
  `package/oxlint-plugin/prefer-readonly-parameter-type/`;
- shared parser and mutation-contract model:
  `package/oxlint-plugin/shared/`;
- exact ownership marker:
  `package/ownership-marker/foreign-borrowed/`;
- shared-config sidecar and error-level policy:
  `package/config/oxlint/src/plugin-prefer-readonly-parameter-type.ts` and
  `package/config/oxlint/src/rule/restriction.ts`;
- bypass-prevention rule and fixtures:
  `package/oxlint-plugin/no-restricted-syntax/` and
  `package/test-fixture/oxlint-no-restricted-syntax/`;
- user-facing contracts:
  the dedicated plugin README,
  ownership-marker README,
  and TSDoc plugin README.

The semantic contract is the layered readonly-type and mutation-effect model recorded in the decision log.
The dedicated plugin owns TypeScript 7 runtime access,
package and host evidence,
and semantic caches.
Packages receiving `ReadonlyDeep` projections declare `type-fest` directly through the pnpm catalog.

## Implementation sequence after approval

### Build the TypeScript 7 semantic adapter test-first

Add contract tests around every unstable API operation before rule logic consumes it.
The adapter must prove configured-project discovery,
current-file virtual overlays,
BOM normalization,
source-span mapping,
snapshot reuse and disposal,
changed/deleted/renamed-file invalidation,
multiple package projects,
symlink and path-case behavior,
and fail-closed handling of missing or changed API capabilities.

Retain the disposable corpus for brands,
recursive and conditional types,
indexed access,
callable objects,
collections,
capabilities,
overloads,
bodyless signatures,
higher-order callbacks,
and Unicode spans.
Extend it for parser recovery,
dynamic dispatch,
closures,
deferred effects,
recursion,
and external callbacks.
Run platform artifact probes on Linux x64,
macOS arm64,
and Windows x64 before completion.
Add intrinsic-effect contract tests for exact ECMAScript,
DOM,
Node,
and package symbols;
package aliases and subpath exports;
supported and unsupported majors;
duplicate installed majors;
and unresolved package metadata.

### Define the rule contract

Record an explicit valid and invalid catalog before writing implementation code.
The catalog must cover declarations,
function expressions,
methods,
constructors,
callback signatures,
destructured parameters,
default values,
rest parameters,
generics,
external API callbacks,
and mutable-by-design state carriers where applicable.

State what the rule does not prove.
In particular,
a syntax-only or body-analysis rule must not be documented as deep type immutability.

### Build the rule test-first

Add invalid and valid fixture files under
`package/test-fixture/oxlint-no-restricted-syntax/src/`.
Extend the dedicated fixture config and integration test so the new diagnostic appears under
`no-restricted-syntax/<settled-rule-name>`.

Implement the rule as focused sibling modules for the host rule,
semantic adapter,
readonly classifier,
effect summaries,
TSDoc effect lookup,
and source suggestions.
Use TypeScript symbols and resolved signatures for semantic identity,
and use Oxlint scope/reference APIs for current-tree diagnostics.
Unknown calls or unsupported semantic states fail closed with `opaqueEffect` or a dedicated bridge diagnostic;
method-name lists cannot serve as effect proof.

### Implement the `@mutates` TSDoc contract

Register the custom block tag in the TSDoc parser configuration and expose parsed mutation targets through the shared
document-model seam.
Add dedicated validation for syntax,
known parameter targets,
duplicates,
descriptions,
overload consistency,
missing tags,
and stale tags.
Keep malformed-tag diagnostics in the TSDoc plugin and semantic effect diagnostics in the readonly rule.

Update the TSDoc plugin registration,
fixture config,
valid and invalid fixtures,
unit tests,
and README.
Verify that the shared parser performs one parse per comment for all participating rules.

### Switch shared configuration

Add the project rule to
`package/config/oxlint/src/rule/restriction.ts` at the agreed rollout severity.
Remove `typescript/prefer-readonly-parameter-types` from
`package/config/oxlint/src/rule/correctness.ts`.
Update overrides so test and external-signature behavior matches the settled policy rather than inheriting the old
rule's exemptions accidentally.

### Remove retired configuration

Delete the obsolete dedicated configuration files:

- `package/config/oxlint/src/rule/prefer-readonly-parameter-types.ts`;
- `package/config/oxlint/src/rule/prefer-readonly-parameter-types.allow-lib.ts`;
- `package/config/oxlint/src/rule/prefer-readonly-parameter-types.allow-pkg.ts`;
- `package/config/oxlint/src/rule/prefer-readonly-parameter-types.allow-pkg-unbash.ts`.

Remove the obsolete output suppression and its dedicated test cases from
`package/monorepo-task/core-exec/src/oxlint-suppress.ts` and
`package/monorepo-task/core-exec/src/oxlint-suppress.unit.test.ts`.

### Migrate source directives and explanatory types

Classify every active directive before deleting it:

- sites accepted by the new contract lose the directive;
- sites violating the new contract receive an sound `ReadonlyDeep` projection,
  an ownership-correct declaration improvement,
  an accurate `@mutates` contract,
  or a local adapter around external mutation;
- comments and helper types created only to placate the retired rule are removed or simplified;
- historical troubleshooting documents remain as historical evidence but gain a supersession note when their remedy is
  no longer current.

Keep `package-paused/`,
generated,
fixture,
invalid,
and build-output trees under their existing ignores.
Do not perform blind string replacement because old block disables may contain another still-active rule.
No inline suppression of the replacement rule is permitted.

### Verify the consumer boundary

Run the package unit tests,
fixture lint,
plugin type lint,
config build,
and config type lint through their `mise` tasks.

Build the shared Oxlint config,
then lint representative consuming packages through their normal package `lint:oxlint` task.
Verify that:

- intended new violations are reported under the project rule ID;
- accepted external and mutable-by-design boundaries no longer require old directives;
- old directives and config references are absent;
- the normal `--type-aware` wrapper still runs the remaining native type-aware rules;
- no output suppression hides replacement-rule diagnostics.

Build the published plugin artifact and load it from a disposable external consumer with no monorepo-root dependency
resolution.
Verify TypeScript 7 native artifact startup and rule diagnostics on Linux x64,
macOS arm64,
and Windows x64.
Build declarations through the installed Oxc and `rolldown-plugin-dts` path and assert every `@mutates` target and
description survives overloads,
call signatures,
and re-exports.

A full active-workspace CLI lint is the rollout gate after targeted consumer checks pass.
The CLI is authoritative;
editor diagnostics remain deferred while Oxlint's language server cannot load JavaScript plugins.

## Interview resolution

All discovered policy branches are resolved:

- preserve readonly type-contract semantics and add mutation effects;
- require universal,
  verified `@mutates` contracts for owned mutation;
- recognize ECMAScript,
  DOM,
  Node,
  and major-version-gated effects for packages present in the recorded pnpm lockfile baseline intrinsically,
  with local adapters as the fallback;
- infer higher-order relationships rather than adding a TSDoc relation language;
- enforce active production source while retaining test,
  benchmark,
  declaration,
  paused,
  generated,
  fixture,
  invalid,
  and build-output exemptions;
- prohibit inline suppression and enable the replacement at `error`;
- expose semantic rewrites only as suggestions;
- keep one rule identity with distinct diagnostic message IDs;
- preserve `@mutates` in published declarations;
- use TypeScript 7's unstable synchronous API without a TypeScript 6 fallback;
- treat the CLI as authoritative until Oxlint supports JavaScript plugins in its language server;
- author local projections with `type-fest`'s `ReadonlyDeep` while rejecting unsupported capability projections.

No policy question remains known.
A newly discovered genuine policy fork reopens one-question-at-a-time grilling;
implementation details that have a correctness-dominant answer do not.

## Research checkpoint

Current checkpoint:

- the technology vet selected TypeScript 7.0.2's `typescript/unstable/sync` API
  (`typescript/unstable/sync` was renamed `typescript/sync` in 7.1.0-beta,
  2026-10-20) and rejected every
  TypeScript 6 fallback;
- real Oxlint-boundary probes covered imported semantic types,
  mapped readonly state,
  representative direct and higher-order effects,
  current-file overlays,
  configured-project discovery,
  snapshot reuse,
  BOM and Unicode mapping,
  and native child cleanup;
- the type corpus confirmed that `ReadonlyDeep` handles the tested collections and recursive structures but cannot make
  retained capability methods sound;
- installed Oxc isolated declarations and `rolldown-plugin-dts` 0.27.4 preserved all tested `@mutates` blocks through a
  re-exporting declaration bundle;
- the advisor review converted unproved generalizations into explicit implementation acceptance gates;
- the user chose CLI-only authority while Oxlint's language server lacks JavaScript-plugin support;
- the user chose `type-fest`'s `ReadonlyDeep` instead of a project-owned duplicate or synthesized structural types;
- all discovered policy questions are resolved;
- no implementation code or dependency change is authorized.

Next action:
request the user's confirmation that this document reflects shared understanding.

## Decision log

### Settled before the interview

- Treat implementation time and money as unlimited.
- Never choose a narrower design merely because it is easier,
faster,
or cheaper to build.
- Resolve a decision without asking when removal of resource constraints leaves one strictly dominant option.
- Replace the native rule rather than adding another layer of allow-list entries.
- Implement the replacement in the existing `no-restricted-syntax` JavaScript plugin package.
- Keep this planning document current as decisions are made.
- Do not implement until the user confirms shared understanding.

### Corrected during investigation

The initial plan incorrectly treated absent Oxlint parser services as proof that a replacement could not reproduce
 type-aware semantics.
The verified fact is narrower:
Oxlint does not supply type information to JavaScript plugins.
Independent analysis was therefore investigated.
The completed vet selected TypeScript 7's synchronous unstable API;
the other candidates remain recorded as rejected alternatives or declaration-only components.

### Resolved from scope and the no-resource-constraint posture

The replacement remains a type-contract rule.
A behavior-only `no-param-reassign` variant is not a replacement for
`prefer-readonly-parameter-types` because it abandons the declared API contract.
No resource constraint justifies that loss.

The comprehensive design may analyze body writes,
aliases,
callee effects,
and capability use as supporting evidence.
That analysis helps distinguish accidental mutable exposure from a truthful mutable parameter contract;
it does not replace type analysis.

Existing active code proves that intentional parameter mutation is legitimate in this repository:
visited sets,
caches,
DOM transforms,
streams,
render sessions,
and assertion trackers all have documented mutable contracts.
The replacement must express these contracts without restoring a global type-name allow list or treating every method as
readonly.

### Chosen mutation-intent declaration

Intentional parameter mutation uses a verified custom TSDoc block tag.
Canonical proposed grammar:

```typescript
/**
 * Clears shared traversal state before reuse.
 *
 * @param visited - Shared cycle detector retained across calls.
 *
 * @mutates visited - Clears caller-owned traversal state.
 */
function clearVisited(visited: Set<string>,): void {
  visited.clear();
}
```

One `@mutates` block names one top-level parameter or destructured parameter property,
using the same naming rules as `@param`.
The description states why mutation belongs to the function's contract.
The semantic rule reports missing tags,
stale tags,
and mutation that reaches an undeclared parameter through aliases or callees.

### Confirmed TSDoc ripple

`@mutates` is not a standard TSDoc tag.
The repository's TSDoc plugin currently hardcodes standard tags and carries only `@yields` as a custom tag.
Supporting `@mutates` therefore requires coordinated changes rather than a one-line allow-list edit:

- `package/oxlint-plugin/tsdoc/src/rule/tag-names.ts`:
  recognize the custom tag and correct its standard-only documentation;
- `package/oxlint-plugin/tsdoc/src/tsdoc-blocks.ts`:
  terminate preceding blocks at `@mutates` and parse mutation blocks;
- `package/oxlint-plugin/tsdoc/src/tsdoc-doc-model.ts`:
  represent target names and descriptions;
- TSDoc parameter extraction:
  validate mutation targets against plain,
  rest,
  defaulted,
  and destructured parameters;
- dedicated TSDoc rules:
  reject missing names,
  unknown names,
  duplicate targets,
  and missing descriptions independently of semantic mutation analysis;
- `package/oxlint-plugin/tsdoc/src/index.ts` and
  `package/config/oxlint/src/rule/tsdoc.ts`:
  register and enable the new validation rules;
- `package/test-fixture/oxlint-tsdoc/` and TSDoc unit tests:
  add valid,
  malformed,
  duplicate,
  destructured,
  fenced-example,
  and unknown-tag cases;
- `package/oxlint-plugin/tsdoc/README.md` and shared config documentation:
  disclose the project-specific TSDoc extension and its grammar;
- the readonly rule:
  consume the same parsed mutation blocks rather than implementing a second comment scanner.

The TSDoc plugin owns tag grammar and signature-name validation.
The readonly rule owns effect verification against types,
body writes,
aliases,
and callee summaries.
Shared parsing primitives must move behind one dependency seam,
likely `@monochromatic-dev/oxlint-plugin-shared`,
so the sibling plugins cannot drift.

`tsdoc/tag-lines` already applies to every leading tag,
so `@mutates` automatically requires a preceding blank line.
`tsdoc/empty-tags` must not classify `@mutates` as a modifier because the new tag requires content.

### Chosen layered type and effect contract

For a nonmutating parameter,
require a deeply readonly TypeScript type whenever that type soundly represents the callable contract.
For external,
identity-sensitive,
callback-bearing,
branded,
or capability types where a readonly projection would misrepresent the usable API or break required assignability,
retain the original type only when whole-program effect analysis proves no mutation path.
`ForeignBorrowed` appears only where foreign ownership enters or is deliberately retained.
Descendants inherit that provenance through semantic dataflow;
they do not repeat the marker.
A parameter inherits foreign provenance only when every owned inbound call agrees,
so one owned call path restores ordinary readonly enforcement.

This is not a global type-name allow list.
The decision is made per parameter from resolved declarations,
provenance,
assignability,
used members,
and effect summaries.
A plain owned object that can become `Readonly<T>` remains a violation when declared mutable.
An opaque capability is not forced through a facade merely to satisfy syntax.

Readonlyness and mutation effects are transitive through reachable properties,
aliases,
destructuring,
closures,
and callee arguments.
Unknown external or dynamic effects fail closed:
the implementation must obtain an explicit effect summary or require complete `@mutates` documentation rather than
assuming safety.

The absence of `@mutates` is valid when analysis proves observation or a caller-observable effect.
Observation still requires an sound deep-readonly type where one is available.
Proven effects permit mutable types,
and a present contract must remain accurate.
Unresolved possible effects require complete contracts naming each uncertain boundary and affected input.

### Chosen callable coverage

Every callable participates in the same three-state analysis:
exported functions,
local helpers,
methods,
constructors,
getters or setters with parameters,
function expressions,
and inline callbacks.

The rule infers effects to decide whether readonly is sufficient,
to verify optional present tags,
and to propagate diagnostics.
Callback-capability invocation and referent mutation are separate effects.
Proven callback invocation permits a mutable capability without requiring `@mutates`.
Invoking an unknown callback requires an sound complete contract,
but invocation alone does not claim that the function object or a pure owned callback's captures were mutated.
Every call edge therefore has an inspectable effect summary or explicit uncertainty contract.
Moving a callable between local and exported scope does not change the analysis.

The TSDoc plugin already visits function expressions and arrow functions as documentable nodes.
The implementation must add mutation-tag fixtures for direct declarations,
methods,
and callback comment attachment so Oxlint's comment ownership cannot silently skip an effect contract.

### Chosen bodyless and external contract model

Every owned callable signature in enforced source carries its own effect tags,
including overload declarations,
interface and type-literal call signatures,
abstract methods,
source `declare` signatures in `.ts` files,
and the concrete implementation signature.
Signatures inside `.d.ts`,
`.d.mts`,
and `.d.cts` files remain exempt.

Overload effects may differ.
The implementation summary must cover the union of reachable overload effects,
with parameters mapped by resolved signature rather than assumed name equality.
The TSDoc plugin validates each signature independently;
the semantic rule validates cross-signature consistency.

The analyzer intrinsically recognizes effects for ECMAScript,
DOM,
Node,
and common ecosystem packages.
Package effects are keyed by resolved package identity and supported major version;
an effect entry for one major must never apply to another major implicitly.
A package is eligible for intrinsic coverage when its package name occurs in the current pnpm lockfile baseline,
`pnpm-lock.yaml` lockfile version 9.0 with SHA-256
`3912af5f960cef4c459f6dc99966dcdf9947507690f39969a4951404036cf76d`.
Transitive and direct packages use the same eligibility rule.
A later lockfile addition is not silently admitted to this baseline.

Eligibility does not itself assert an effect.
Each catalog entry still records exact callable symbols,
effect targets,
source provenance,
supported declaration versions,
and tests proving both accepted and rejected major-version matches.
Method names alone are never evidence.

An uncatalogued package,
unsupported major,
or unresolved symbol falls back to a locally owned,
documented adapter.
Adapter tags state the external effect at the repository boundary and must name the upstream callable in their
description or link.
This supersedes the earlier adapter-only design while retaining adapters as the fail-closed boundary.

`@mutates` means "may mutate,
"
not "always mutates.
"
The effect analyzer uses three outcomes:

- proven observation requires an sound deep-readonly type and rejects a stale tag;
- proven caller-observable effect permits a mutable type and makes an accurate tag optional;
- possible effect through an opaque external boundary reports `opaqueEffect` unless complete local documentation
  accounts for every affected input and uncertain boundary.

A local adapter is verified structurally:
every opaque effect must map to a parameter carrying `@mutates`,
and no opaque effect target may remain unaccounted for.
Its summary retains documented uncertainty separately from proven mutation,
plus upstream provenance for diagnostics and audit output.
A partial tag does not waive `opaqueEffect`.
This permits explicit external adapters without pretending their implementation was proved from unavailable source.

### Rejected higher-order TSDoc relation DSL

A proposed tag such as
`@propagates visitor.value to value`
is rejected.
It embeds parameter relations,
path syntax,
and directional keywords in unchecked comment text.
Typos and renames would depend on a second custom grammar,
and the authored contract would be harder to read than the code it describes.

Keep `@mutates` limited to one signature-local parameter target plus prose description.
The TSDoc plugin can validate that one target against the callable signature and report on the tag line,
like its existing `@param` name checks.
Do not expand TSDoc into a general effect-language syntax.

Higher-order propagation should instead come from machine-readable program structure:

- infer symbolic parameter-to-callback-argument relations from owned function bodies;
- resolve callback call signatures and their `@mutates` summaries through the type checker;
- specialize generated effect summaries at call sites when concrete callback effects are known;
- persist generated summaries for incremental and cross-package analysis rather than asking authors to write relation
  strings;
- use conservative signature-local `@mutates` effects for bodyless or opaque callables when specialization cannot be
  proved;
- prototype TypeScript-level effect metadata only if a bodyless generic API needs expressiveness that generated
summaries
  cannot carry.

### Chosen test and declaration exemptions

Tests and declaration files retain their exemptions from the readonly and mutation-effect contract.
The shared config must turn the replacement rule off for all test and benchmark filename forms already recognized across
config and TSDoc handling:

- `*.test.ts`,
  including unit,
  browser,
  and end-to-end variants;
- `*.spec.ts`;
- `*.bench.ts`;
- `*.d.ts`,
  `*.d.mts`,
  and `*.d.cts`.

The TSDoc `@mutates` validation rules use the same exemption predicate,
so exempt files do not gain tag requirements indirectly through the TSDoc plugin.
The dedicated plugin fixture configs remain allowed to enable the rules on fixture files because those files test the
rule itself rather than adopting production policy.

This preserves the current reason for the test override:
framework-owned mutable callbacks,
fixtures,
spies,
and mocks are not production API contracts.
Declaration files remain descriptive ambient shapes without bodies to verify.
Generated and published declarations are preservation-test inputs,
not source files subject to the replacement rule or `tsdoc/*` enforcement.

### Resolved bodyless generic feasibility boundary

Use generated symbolic summaries for owned higher-order implementations.
Bodyless generic callables expose only signature-local `@mutates` effects;
they cannot express an unchecked parameter-relation language.
When specialization cannot prove whether a callback propagates mutation to another parameter,
report `opaqueEffect` and require an owned wrapper with an analyzable body.

### Resolved rollout scope

Do not redefine repository lifecycle categories as part of a lint-rule replacement.
The replacement enforces the active production lint scope.
Existing global ignores remain for paused,
deprecated,
generated,
fixture,
invalid,
and build-output trees.

Migration still removes obsolete native-rule directives and misleading explanatory comments from ignored authored source
when that cleanup does not require making the source compile or pass the new rule.
Historical troubleshooting and audit documents retain factual references with a supersession note.
Generated artifacts are regenerated from their owner or left untouched when no owner is in scope;
never hand-edit them for textual cleanup.

### Resolved suppression and severity policy

The final replacement rule and `@mutates` validation rules are errors.
No inline suppression of either rule is allowed.
Add `no-disable-prefer-readonly-parameter-types.ts`,
register it in the plugin index,
enable it in shared and fixture configs,
and cover line,
block,
and list-style disable directives in fixtures and integration tests.
Opaque cases must gain an sound type,
verified effect summary,
local external adapter,
or existing file-class exemption instead of a comment bypass.
Retired native-rule directives must be parsed and edited token-by-token so mixed directives retain every other rule.

An implementation branch may use a temporary warning only while its own migration commit series is incomplete.
The merge-ready state has no active violations and error severity.

### Chosen remediation fix kind

Offer proven type rewrites as Oxlint suggestions.
Do not attach them as direct fixes,
so ordinary `--fix` and the repository's normal `format:oxlint` task cannot apply semantic signature changes.
Suggestions remain available through explicit `--fix-suggestions` or `--fix-dangerously` use.
They can become editor code actions only after Oxlint's language server supports JavaScript plugins.

Oxlint 1.73 JavaScript plugins cannot mark a fix dangerous directly.
Their protocol maps `Diagnostic.fix` to a normal fix and `Diagnostic.suggest` to a suggestion;
there is no JavaScript dangerous-fix field.
The verified source trace and disposable probe are recorded in
[`doc/troubleshooting/oxlint-js-plugin-fix-kinds.md`](../troubleshooting/oxlint-js-plugin-fix-kinds.md).

The rule sets `meta.hasSuggestions: true` and omits a direct `fix` for semantic rewrites.
A suggestion is emitted only when the semantic pipeline constructs one exact replacement and verifies the rewritten
program against the relevant TypeScript projects.
Ambiguous cases remain diagnostics without a suggestion.

### Chosen rule identity and taxonomy

Use one project rule:

```text
prefer-readonly-parameter-type/prefer-readonly-parameter-types
```

The recognizable basename preserves lineage from the retired native rule.
One semantic analysis owns the complete layered readonly and effect contract.
Distinct message IDs classify findings without independently configurable partial rules:

- `shouldBeReadonly`;
- `missingMutatesTag`;
- `staleMutatesTag`;
- `opaqueEffect`;
- `projectedReadonlyCapability`;
- `inconsistentMutatesContract`;
- `semanticBridgeUnavailable`.

Malformed tag names,
duplicate targets,
missing descriptions,
and unknown parameter targets remain `tsdoc/*` diagnostics because they are documentation-grammar failures.

The rule has no semantic-relaxation options.
The public package may expose the fixed project policy,
but consumers cannot configure away depth,
effect verification,
or opaque-boundary handling.

### Resolved suggestion coverage

Offer a suggestion only when the analyzer can generate a complete remediation without inventing human rationale:

- replace a mutable type with one verified sound readonly form;
- remove a stale `@mutates` block only after closed-world proof of no effect;
- perform exact mechanical syntax normalization around an otherwise complete tag.

Do not synthesize missing `@mutates` descriptions,
external adapters,
or effect rationales.
Those diagnostics require authored design because project TSDoc requires comments to explain why.

### Chosen mutation boundary

`@mutates parameterName` means the callable may cause any caller-observable state change through state reachable from
that parameter at entry.
It covers:

- property assignment,
  update,
  and deletion;
- transitive writes through nested objects and aliases;
- collection mutators;
- stream,
  iterator,
  cancellation,
  event,
  DOM,
  and other capability operations that change receiver state or external state represented by the receiver;
- synchronous,
  asynchronous,
  deferred,
  and closure-captured effects;
- effects propagated through callees and callbacks.

Local rebinding of the parameter variable does not mutate the caller's referent and is outside this rule.
A separate parameter-reassignment policy could govern that syntax later.
Do not introduce separate `@consumes`,
`@writes`,
or `@cancels` tags;
the prose description explains the domain-specific transition while the machine-readable effect remains one concept.

### Chosen ownership-aware remediation

Use whole-program ownership evidence to choose the remediation location.

When every valid use treats a repository-owned type as immutable,
make the type declaration deeply readonly and let consumers retain the canonical name.
When lifecycle owners legitimately mutate the type,
keep that declaration mutable and apply an sound readonly projection only at nonmutating parameter boundaries.
External capability types retain their original form only under the selected effect proof.

Type-owner suggestions originate when the owner file is linted,
not as cross-file edits attached to a consumer diagnostic.
Oxlint JavaScript fix payloads carry ranges for the current file rather than workspace edits.
The semantic pipeline may coordinate findings across files,
but each suggestion must be independently valid in its own file and pre-verified against all affected TypeScript
projects.

For mixed ownership,
do not automatically split one domain type into mutable and immutable sibling types.
Prefer a local projection unless an independently meaningful domain distinction already exists
or is separately designed.

### Chosen declaration publication

Preserve `@mutates` blocks in emitted and bundled declaration files.
Mutation effects are part of the published API contract,
not repository-only lint metadata.
Do not strip them or translate them into prose-only `@remarks` blocks.

The custom tag documentation must explain how external TSDoc consumers register it.
Declaration verification must exercise Oxc isolated declaration output,
`rolldown-plugin-dts` bundling,
overloads,
interface call signatures,
re-exports,
and declaration comments to prove targets and descriptions survive intact.
Published rule and config packages must expose the custom-tag contract in their READMEs.

A disposable probe against the repository-installed Rolldown 1.1.5 and `rolldown-plugin-dts` 0.27.4 verified the
selected path.
Oxc isolated declarations preserved three `@mutates` blocks on a function,
an overload,
and a call signature with zero transform errors.
Bundling those declarations through a re-exporting entry preserved all three blocks and their descriptions.
The implementation suite must retain this corpus and add package-level README and external-consumer checks.

### Chosen semantic bridge

Use TypeScript 7's project-native synchronous API through `typescript/sync`.
Do not install or fall back to TypeScript 6.
The repository accepts the API's unstable compatibility status in exchange for matching its installed TypeScript 7
compiler semantics
(the 7.1.0-beta export rename,
2026-10-20,
dropped the `unstable/` path segment and changed no symbol names).

The rule will keep one process-scoped `API` client,
open projects through snapshots,
provide unsaved current-file text through virtual filesystem callbacks,
and invalidate changed files with `snapshot.update` and its `fileNotifications`.
Oxlint's synchronous JavaScript-rule visitors require the synchronous TypeScript API rather than its asynchronous
counterpart.

The disposable Oxlint-boundary prototype proved representative feasibility with installed TypeScript 7.0.2
(2026-era evidence):

- one fixture covering imported aliases,
  generics,
  unions,
  overloads,
  function types,
  call signatures,
  and method signatures resolved from one TypeScript project;
- TypeScript source offsets produced the expected Oxlint spans on ASCII LF source;
- explicitly readonly properties and one recursive mapped `DeepReadonly` projection were distinguished from mutable
  properties;
- one direct mutation,
  one cross-file call,
  and one immediate generic callback-invocation shape propagated to the owning parameter;
- one virtual filesystem overlay changed a queried parameter type in the next snapshot without writing source to disk.

Follow-up probes added these bounded results:

- `openFiles` discovered the distinct package `tsconfig.json` files for the no-restricted-syntax and JSONC-edit
  packages and returned zero semantic diagnostics for each queried source file;
- one Oxlint run configured with 16 threads invoked the JavaScript plugin in one process;
  a cache revision reused one snapshot across ordinary disk-backed files,
  while the BOM fixture exposed a required `sourceCode.hasBOM` normalization before overlay comparison;
- a BOM plus CRLF fixture with an astral character and a combining sequence before the parameter mapped to the expected
  line,
  column,
  and UTF-8 Oxlint output offset;
- brands,
  recursive types,
  callable objects,
  conditional types,
  indexed access,
  arrays,
  maps,
  sets,
  weak maps,
  typed arrays,
  and an `AbortController` capability all produced queryable TypeScript 7 type structures;
- `ReadonlyDeep<Map<...>>` and `ReadonlyDeep<Set<...>>` produced readonly mapped projections,
  while `ReadonlyDeep<AbortController>` retained the mutating `abort` capability and therefore demonstrated why
  structural readonly alone cannot satisfy `projectedReadonlyCapability` detection.

These results select the primary bridge but do not complete its acceptance suite.
The remaining effect,
parser-recovery span,
lifecycle,
cache,
packaging,
and platform probes remain mandatory before the replacement can be enabled and declared complete.

Readonly detection must isolate the unstable detail behind a tested adapter.
TypeScript 7.0.2 (2026-era evidence) exposes transient mapped-property readonly state through `Symbol.checkFlags`,
whose upstream `CheckFlagsReadonly` value is `1 << 3` but whose enum is not exported.
Pin adapter contract tests to the installed TypeScript version and fail closed with a dedicated semantic-bridge
diagnostic when the expected capability disappears.
Do not misreport unavailable readonly semantics as `opaqueEffect`,
which is reserved for unresolved mutation effects.

The published `no-restricted-syntax` package must declare TypeScript 7 as a runtime dependency and prove that its built
artifact resolves `typescript/sync` in a disposable external consumer.
It may not rely on the monorepo root's development dependency.

Oxlint's language server currently does not support JavaScript plugins.
The CLI is the sole authority for this rule until Oxlint adds JavaScript-plugin language-server support.
Do not retain the incumbent rule for editor-only approximation,
build a separate editor integration,
or block CLI rollout on the upstream capability.
Live editor diagnostics and code actions for this rule are therefore explicitly deferred.

### Chosen readonly projection authoring

Use `ReadonlyDeep` from `type-fest` for parameter-local deep-readonly projections.
Do not create a project-owned duplicate or synthesize inline structural readonly types.
Each package that imports `ReadonlyDeep` must declare `type-fest` through the pnpm catalog rather than relying on a
transitive or root dependency.

`ReadonlyDeep` is authoring syntax,
not proof that the resulting capability is soundly immutable.
The semantic rule must still inspect the resolved projection and its reachable effects.
The probe showed that `ReadonlyDeep<Map<...>>` and `ReadonlyDeep<Set<...>>` produce useful collection projections,
while `ReadonlyDeep<AbortController>` retains `abort()` and must produce `projectedReadonlyCapability`
when used as a supposedly
nonmutating contract.

Suggestions may introduce `ReadonlyDeep<T>` only when the target package already declares `type-fest` or when the
migration explicitly adds that dependency.
A suggestion cannot edit `package.json`,
so it must not offer an import that would leave the source unresolved.

### Grilling status

The intrinsic-effect boundary includes ECMAScript,
DOM,
Node,
and package names present in the recorded current `pnpm-lock.yaml` baseline.
Package entries remain gated by resolved major version and source audit.
Host entries remain gated by audited standards revisions or exact Node embedded-source identities.
All currently known policy questions are resolved,
and the user confirmed the plan before implementation began.
