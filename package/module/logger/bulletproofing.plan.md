# Post-release verification track: module-logger

Status:
 proposal,
 not a release gate.
Rewritten on 2026-09-06 from the June plan after the owner reclassified that
plan as a historical reference by an unreliable narrator
(`doc/planning/module-logger-release.md`).
Nothing here blocks a publish;
 the release process is `doc/decision/npm-publishing.md`.

## What the June plan got wrong, so it is not repeated

- It predated the IndexedDB and localStorage sinks,
   the shared record buffer,
   and the bounded storage retries (2026-07-22),
   so its file map and sink inventory were stale.
- It proposed retiring a sink after ten consecutive failures.
   Every shipped sink swallows its own write errors,
   so the counter could never fire,
   and it reversed the 2026-06-14 decision in `DECISIONS.md` one day later.
   Dropped.
- It proposed four `createLogger` tuning options to serve a fuzz harness.
   Only `flushDeadlineMs` survived,
   because it has a consumer-facing reason (a slow but working backend).
- It claimed a single-argument `console.info` interprets `%s`.
   Measured false on the built artifact.
- It called the no-backend throw a defect at 102 unguarded call sites.
   The throw is reachable only through `createLogger` with sinks that all fail verification;
   the default logger always has the console sink.
   Kept,
   documented,
   tested.

## What already shipped (0.1.0)

- Console control-character neutralization (`src/sink/console-control-chars.ts`),
   with adversarial tests for malformed and partial sequences.
- `flush()` deadline via `flushDeadlineMs` (`DEFAULT_FLUSH_DEADLINE_MS`),
   with tests forcing the deadline for a wedged write,
   flush hook,
   and verify.
- Every unit test imports the built artifact through the package name;
   internals are exposed as underscore-prefixed entry exports.

## What shipped after 0.1.0 (pending release)

- Verify liveness (2026-09-06):
   every sink verifies concurrently under `verifyTimeoutMs` (`DEFAULT_VERIFY_TIMEOUT_MS`),
   with tests for a never-settling verify no longer starving later sinks and for the replay invariant under concurrency.
   The guard was shown to fail against the sequential,
   unbounded code before the fix was restored.
- Startup buffer bound (2026-09-06):
   pre-initialization records buffer under `STARTUP_BUFFER_CAP`,
   oldest dropped on overflow,
   one synthetic `warn` record naming the dropped count once every sink has answered.
   Measured on the built artifact:
   a full buffer holds about 1.6 MiB of heap and a burst one hundred times the cap settles in about 150 ms,
   so the drop path stays linear.
   The guard was shown to fail against the unbounded code before the fix was restored.
- Abandoned-write accounting (2026-09-06):
   two tests pin that a write the flush deadline abandoned may reject or resolve late without an unhandled rejection,
   with a late rejection reported once as a breadcrumb and the sink kept available for later writes.
   No code change was needed.
   The guard was shown to fail (the captured `late failure` rejection) when the tracked-write catch was stripped;
   that stripped run also exposed a harness gap,
   the file process dying on an older test before the new suite ran (#483).

## Open robustness items, in priority order

1.   Dynamic imports in the published artifacts.
     Both built entries carry `import('node:fs/promises')` and `import('node:path')` from the file sink's verify,
     and a browser consumer bundle inherits both,
     which is what users complain about.
     Lazy discovery would not remove them.
     Candidate:
     platform-split file sink selected through package `imports` conditions,
     guarded by a test that reads both artifacts and rejects any `import(`.
     See `DECISIONS.md`,
     "Open problem:
      import-time sink discovery".
2.   Deferred discovery for callers that never log:
     resolved on 2026-09-06 by the lazy default logger (issue #493);
     importing the package runs no discovery,
     so `package/ssg/aquati.cat/src/build/compress.ts` no longer needs its deferred import.

## Grill record: file sink platform split (2026-09-06)

Owner decisions so far,
 recorded as answered;
 the prototype lives in a throwaway worktree and nothing is implemented on main:

- Selection mechanism:
   `package.json` `imports` entry with `node` and `default` conditions (LogTape and chalk precedent).
- Runtime guard in the node branch:
   dropped;
   the condition already asserts Node.
- Conditions declared:
   `node` and `default` only;
   Bun and Deno resolve `node`,
   browsers fall to `default`.
- Guard test:
   one unit test reads both built artifacts and fails on any `import(`,
   on `node:` modules in the neutral artifact,
   and (pending round 2) on browser-only sink code in the node artifact.
- Changeset level:
   minor.
- Documentation:
   README runtime-support paragraph plus a DECISIONS.md entry superseding the open problem.
- Open after round 1:
   whether the file sink should exist in the neutral artifact at all (the owner leans to no),
   and the symmetric question for the browser-only IndexedDB and OPFS sinks in the node artifact,
   which the owner raised.
   Measured for that round:
   no explicit sink-factory call exists outside the logger package;
   no `customConditions` exists in the repository TypeScript configs,
   and TypeScript under `bundler` resolution matches only `types` and `import`/`require`.

### Round 2 (2026-09-06): platform code moves to subpaths

- The root entry stays platform-neutral:
   `logger`,
   `initPromise`,
   `createLogger`,
   `tagged`,
   the types,
   and a `sinks` namespace holding only the cross-platform factories (console,
   noop,
   sessionStorage,
   localStorage).
- `createFileSink` moves to the `./node` subpath;
   `createIndexedDbSink` and `createOpfsSink` move to the `./browser` subpath.
   No stub exists anywhere:
   the neutral artifact contains no file sink code and the node artifact contains no IndexedDB or OPFS code.
- The default sink list is selected through `#default-sinks`:
   `src/default-sinks.node.ts` (console,
   sessionStorage,
   localStorage,
   file) under `node`,
   `src/default-sinks.neutral.ts` (console,
   IndexedDB,
   sessionStorage,
   localStorage) under `default`,
   so zero-config keeps file logging under Node and IndexedDB in browsers.
- Source layout (owner deferred to the agent):
   flat siblings;
   `src/sink/file.ts`,
   `src/sink/indexed-db.ts`,
   `src/sink/opfs.ts` keep their names and become platform-only modules with static imports;
   `src/node.ts` and `src/browser.ts` are the subpath entries;
   the node rolldown config builds `index.ts` plus `node.ts`,
   the neutral config builds `index.ts` plus `browser.ts`.
- Guard test asserts four directions across every chunk of each build:
   no `import(` anywhere;
   no `node:` module in the neutral build;
   no `indexedDB` or `navigator.storage` reference in the node build;
   `createFileSink` absent from the neutral build and `createIndexedDbSink`/`createOpfsSink` absent from the node build.
- Reasoning recorded for the rejected shapes:
   a stub ships code whose only job is to answer no;
   per-condition `types` under `node`/`default` hands every `bundler`-resolution consumer the neutral types,
   because TypeScript matches only `types` and `import`/`require` there and this repository sets no `customConditions`.
- Accepted consequence:
   a Node consumer whose bundler resolves `default` gets no file logging and no message,
   the same as every package in the prior-art sample;
   the minor changelog line names it.

### Prototype 1 measurements (stub shape, worktree `logger-file-sink-split-20260906`, 2026-09-06)

The round-1 stub shape was built to validate the `imports`-field mechanism before the round-2 shape;
 every mechanism result carries over.

- Build,
   unit tests (23 suites),
   and `lint:types` pass.
   `lint:oxlint` exits 1 with 1719 warnings,
   every one `stylistic(require-asterisk-prefix)` in files the prototype did not touch:
   commit `4b09ccd18` (`style(config-oxlint): require starless tsdoc`) landed on main during the session,
   and the logger package has not been reformatted for it yet;
   `mise run //package/module/logger:format:oxlint` is the package-wide fix.
- `import(` count:
   node artifact 0 (was 2),
   neutral artifact 0 (was 2).
   The node artifact carries `from"node:fs/promises"` and `from"node:path"` statically;
   the neutral artifact mentions neither.
- Artifact bytes:
   node 25401 to 25351,
   neutral 26590 to 25501.
- Consumer bundles of the neutral artifact for the browser:
   rolldown 29.44 kB and esbuild 29.2 kB,
   no warnings,
   0 `import(` in each.
- End-user check on the node artifact:
   one `logger.info` plus `flush()` created
   `node_modules/.monochromatic/2026-09-06T18-35-55.672Z.log.jsonl`
   with the record.
- Survey:
   no workspace package both depends on the logger,
   builds with the neutral config,
   and is executed under Node directly.
   Transitive exposure exists:
   five libraries ship only a neutral build and inline the logger (`css-edit`,
   `fs-path`,
   `jsonc-edit`,
   `test`,
   `toml-edit`),
   and `kv-store` and `pipe` ship both builds.
   Today their inlined logger copy carries the dynamic import and writes files under Node;
   after the split the inlined copy follows the neutral default list and does not.
   Node-executed dependents that bundle those libraries through `default`:
   `build-tool/css`,
   `fy/cli`,
   `dev-script/deps-cube`,
   `dev-script/vm-builder` (through `fs-path`),
   `build-tool/css` and `dev-script/file-enforcer` (through `toml-edit` and `css-edit`).
   Only the inlined copy inside the library is affected;
   each dependent's own logger import still resolves the node artifact.
- Resolution facts read from rolldown's own type declarations:
   default `conditionNames` are `["import","node","default"]` for `platform: node`
   and `["import","default"]` for neutral.

### Prototype 2 measurements (decided shape, worktree `logger-platform-subpaths-20260906`, 2026-09-06)

The round-2 shape,
 built at `2739432b6`:
 root entry platform-neutral,
 `./node` ships `createFileSink`,
 `./browser` ships `createIndexedDbSink` and `createOpfsSink`,
 `#default-sinks` selects the default list per platform,
 rolldown inputs `index.ts` plus `node.ts` (node) and `index.ts` plus `browser.ts` (neutral).

- Build clean.
   Node build:
   `index.mjs` 18863 bytes,
   `node.mjs` 214,
   shared chunk `file-*.mjs` 2089 (the file sink with its static `node:fs/promises` and `node:path` imports),
   runtime chunk 260.
   Neutral build:
   `index.mjs` 18787,
   `browser.mjs` 1722,
   shared chunk `indexed-db-*.mjs` 4437,
   runtime chunk 260.
   Before,
   on main:
   node `index.mjs` 25401,
   neutral `index.mjs` 26590.
   The root `index.d.mts` files of both builds are byte-identical.
- Unit tests:
   every suite passes,
   including the six-case artifact guard with its two positive controls.
   `lint:types` passes.
   `lint:oxlint` reports only the repo-wide starless-TSDoc warnings (1791,
   against 1807 for the same files at HEAD);
   no other finding.
- `import(` count across all eight `.mjs` files:
   0.
   `node:fs` and `node:path` appear only in the node build's file chunk;
   `indexedDB` and `navigator.storage` appear nowhere in the node build.
- Browser consumer bundles of the neutral root:
   rolldown 26706 bytes and esbuild 27155 bytes,
   no warnings,
   0 `import(`,
   no `createFileSink`.
- Node end-user checks on the built node artifacts:
   the default logger wrote `node_modules/.monochromatic/2026-09-06T19-00-38.580Z.log.jsonl`;
   a logger built from `createFileSink` imported through `node.mjs` wrote a second file.
- Type honesty under `moduleResolution: bundler` with no custom conditions:
   importing `sinks` from the root,
   `createFileSink` from `./node`,
   and `createIndexedDbSink` from `./browser` type-checks;
   `sinks.createFileSink()` fails with `TS2339: Property 'createFileSink' does not exist`.
- `require-eventual-artifact` classifies subpath imports as unchecked
   (`package/oxlint-plugin/test-import/src/import-classification.ts`),
   so tests importing `@monochromatic-dev/module-logger/node`
   pass the rule without suppression.
- Survey matches prototype 1:
   no directly affected package;
   the five neutral-only libraries (`css-edit`,
   `fs-path`,
   `jsonc-edit`,
   `test`,
   `toml-edit`) inline the logger and their internal copy would follow the neutral default list under Node.
   `kv-store` and `pipe` already ship both builds with a `node` condition.
- Found in passing:
   `playwright/serve.ts` and `playwright.browser.config.ts` still reference `packages/`,
   the directory name before the singular rename,
   so the browser harness is stale at HEAD independent of this change.
   The prototype also had to teach the harness page to expose `browser.mjs` for the IndexedDB and OPFS browser tests.
- Not done in the prototype:
   README sections that still document the three moved factories on the root namespace,
   the `package.json` description,
   DECISIONS.md,
   the changeset.

### Round 3 (2026-09-06): landing plan

- The five neutral-only libraries keep the neutral default list in their inlined logger copy for now;
   an issue records that they should ship a node build with a `node` condition,
   as `kv-store` and `pipe` do.
- The starless-TSDoc reformat of module-logger lands as its own `style(module-logger)` commit before the split,
   so the package's lint gate is green and the architecture diff stays readable.
- The Playwright harness paths (`packages/` in `playwright/serve.ts` and `playwright.browser.config.ts`) are fixed in
   their own `fix(playwright)` commit,
   together with exposing `browser.mjs` on the harness page for the IndexedDB and OPFS browser tests.
- The `attw` types gate in the release workflow is deferred to a separate change.
- Landing order on main,
   pending the owner's go:
   TSDoc reformat commit;
   Playwright fix commit;
   split commit with guard test,
   test import updates,
   README,
   `package.json` description,
   DECISIONS.md entry superseding the open problem,
   and the minor changeset naming the `default`-condition consequence;
   guard-failure proof by stripping the `imports` map;
   the Q14 issue;
   removal of both throwaway worktrees.

### Landed on main (2026-09-06)

- `eac44c979` fix(playwright):
   harness paths after the singular rename.
- `8e6f494ef` feat(module-logger):
   platform-specific sinks behind `./node` and `./browser`,
   with the six-case artifact guard.
   Guard-failure proof:
   pointing the `default` condition of `#default-sinks` at the Node list fails the two neutral-side cases;
   restored,
   every suite passes.
- `ce38d07db` docs(module-logger):
   README,
   description,
   DECISIONS entry superseding the open problem,
   minor changeset.
- `25188f00f`,
   `f222e67c5`,
   `b6b6afdff`,
   `f06a9b05a`,
   `6e3ffeb76`:
   node builds with a `node` export condition for css-edit,
   fs-path,
   jsonc-edit,
   test,
   toml-edit (owner instruction on go;
   replaces the planned issue).
   Each builds,
   tests,
   and type-checks;
   Node resolves each self-reference to `dist/final/node`;
   the node builds inline the file sink and the neutral builds do not.
   Pre-existing `require-eventual-artifact` findings in fs-path (3)
   and toml-edit (99) test files are untouched by these commits.
- The TSDoc reformat step was unnecessary:
   `3d6c20c9f` (another session) had already swept the repository.
- Both prototype worktrees removed.

## Grill record: verification campaign (2026-09-06)

### Round 1

- Location:
   a sidecar package `package/module/logger.fuzz`,
   the css-edit and jsonc-edit convention,
   so fast-check stays out of the published package and no property file ships in the tarball.
   The owner also asked for the toml-edit campaign to migrate into a sidecar (`package/module/toml-edit.fuzz`).
- `doc/decision/logger-fuzzing.md` (June,
   Phase 0) is rewritten as the campaign decision record;
   a superseded section names each June decision the shipped code reversed
   (kept throw,
   5000 ms defaults,
   no retire threshold,
   neutralize-all console boundary) and points at `DECISIONS.md`.
- Interleaving generator:
   fast-check `scheduler()` over every fake-sink hook (`scheduleFunction`,
   `waitFor`,
   `waitIdle`);
   never-settling work is a task the scheduler never releases.
   The logger's own timers stay real,
   so deadline properties use short real deadlines and small bounded run counts.
- Fake-sink descriptor:
   per hook,
   a per-call behavior sequence with a repeating tail (resolve,
   resolve-false for verify,
   reject,
   throw synchronously,
   delayed,
   never),
   with a stable identity so a shrunk counterexample reads as `sink 2: write [reject, resolve*]`.
- Facts measured for this round:
   fast-check is in the catalog at `>=4.9.0`;
   the toml-edit workflow runs build,
   `lint:types`,
   `test:unit`,
   `fuzz --budget 3000`,
   conformance,
   and `fuzz:coverage`;
   Node 26 exposes `sessionStorage` without flags and `localStorage` only with `--localstorage-file`;
   each campaign copies its own coverage driver (547 differing lines between toml-edit and jsonc-edit),
   so the gate is target-specific code,
   not a shared library.

### Round 2

- Reference model scope:
   per sink the exact records received and availability,
   whether `flush()` settles within the deadline,
   the dropped-count marker record,
   and the `console.warn` breadcrumb count,
   so the "loud signal once,
   at a boundary" contract is a checked invariant.
   Properties that stub `console.warn` retain the sidecar's sequential setting.
   The runtime breadcrumb suites originally matched that setting;
   issue #481 now lets those suites run concurrently with context-owned replacements.
- CI and gate:
   `logger-fuzz.yml` mirrors `toml-edit-fuzz.yml` exactly (build,
   `lint:types`,
   `test:unit`,
   `fuzz --budget 3000`,
   `fuzz:coverage`),
   with a covered-line baseline committed in the sidecar and the gate measuring `package/module/logger/src`.
- The toml-edit sidecar migration lands after the logger campaign,
   copying the logger sidecar's layout.
- Boundary properties:
   the owner asked whether the browser backends (IndexedDB,
   OPFS,
   localStorage) should be driven through Playwright as well as the Node-reachable file and sessionStorage sinks;
   answered with the measured cost (no workflow runs the browser suite today;
   it is a local podman run):
   the Node campaign lands first and a Playwright property layer,
   one browser bundle of fast-check plus the neutral artifact and the properties loaded by the harness page,
   is the last deliverable.

### Landed on main (2026-09-06)

- Sidecar `package/module/logger.fuzz`:
   fake sinks,
   reference model,
   fuzz budget layers,
   and the scheduled orchestration property
  (guard-failure proof:
   startup replay removed,
   both properties fail).
- Sink boundary properties under Node (`5cad2a9a8` and the commits before it):
  console neutralizer against an independent reference,
   console sink against a reference of its grouped output,
  sessionStorage and file sinks reparsed from what they persisted;
   guard-failure proof with the neutralizer made identity and the file sink interpolating raw.
- Coverage-reachability gate (`587ee89c4`):
   driver over the `/ts` source,
   toml-edit's projector and baseline gate,
   first baseline 3376 of 3623 lines;
  guard-failure proof:
   dropping the flush-hook scenario fails the check on `create-logger.ts` (530 to 523).
- `.github/workflows/logger-fuzz.yml` mirroring `toml-edit-fuzz.yml`.
- Decision record rewritten as the campaign record:
   `doc/decision/logger-fuzzing.md`.

- Playwright browser property layer (`8aacfc672`,
   `c98454090`):
   localStorage and IndexedDB round-trips and OPFS acceptance in Chromium,
   Firefox,
   and WebKit inside podman;
  guard-failure proof with the record buffer dropping one record per batch.

Remaining:
 nothing on this track;
 mutation testing is the deferred follow-up.

## Verification campaign (the toml-edit bar)

`package/module/toml-edit` has a budgeted property campaign,
 a model oracle,
 a committed coverage baseline,
 and a green CI workflow.
The logger differs:
 its bugs are timing and interleaving bugs,
 not input bugs,
 so the campaign needs a scheduler,
 not a grammar.

Deliverables,
 each a separate change with its own tests:

- `src/fuzz/fake-sink.ts`:
   a `Sink` driven by a behavior descriptor per hook (`verify`:
   true,
   false,
   reject,
   delayed,
   never;
   `write` and `flush` likewise) with a stable identity so a shrunk counterexample reads as
   `sink 2: verify delayed-true, write never, flush resolve`.
- `src/fuzz/model.ts`:
   a reference model predicting,
   per sink,
   the exact records received and whether `flush()` settles,
   for a given operation sequence and schedule.
- `src/fuzz/*.property.unit.test.ts`:
   `fast-check` properties for exactly-once delivery,
   startup replay exactly once,
   dropout on failed verify,
   write resilience,
   flush totality under the deadline,
   and a scheduler-driven stateful model covering verify-before-log,
   log-before-verify,
   interleaved flush,
   and never-settling work.
- Sink boundary properties:
   JSONL sinks round-trip every adversarial message through `JSON.parse`;
   the console sink never emits a forbidden control.
- A coverage-reachability gate and a path-filtered CI workflow mirroring `toml-edit-fuzz.yml`,
   only after the properties exist.

Rejected:
 a differential oracle against pino or winston (formats differ by design);
 a hostile `toString` message family (the API is string-only).

## Definition of done for this track

- Each open robustness item is either implemented with a test that failed before the change,
   or closed with a recorded reason in `DECISIONS.md`.
- The property files run in the package unit task in bounded mode and in a budgeted campaign task.
- Every counterexample found is pinned as an example or a corpus seed.
