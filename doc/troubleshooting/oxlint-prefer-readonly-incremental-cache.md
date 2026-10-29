# Oxlint 1.74.0: prefer-readonly cache rebuild makes a 13-file lint take 62.9 seconds

## Symptom

The package-scoped command:

```bash
mise run //package/config/oxlint:lint:oxlint
```

reported no diagnostics,
but spent 62.9 seconds on 13 files:

```text
Found 0 warnings and 0 errors.
Finished in 62.9s on 13 files with 479 rules using 16 threads.
```

The file count and thread count are misleading for this incident.
The run paid for one cold,
whole-project analysis in the project-owned
`prefer-readonly-parameter-type/prefer-readonly-parameter-types` JavaScript rule.
An unchanged repeat of the exact mise task in the disposable worktree finished in 1.0 seconds.

## Acceptance target

The maximum acceptable runtime for this exact 13-file package task is 10 seconds.
The cold or invalidated-cache case must satisfy the target because the stable warm path already finishes in 1.0 seconds.
A design that reaches the target only by preserving the current warm cache does not resolve this incident.

## Final catalog-free update

The completed architecture removes handwritten package,
ECMAScript,
DOM,
and Node effect catalogs.
It also removes `@mutates` as an opacity-discharge mechanism,
static plain-data exemptions,
and bodyless host authorities.
A reached call now requires exact implementation-derived effects,
verified runtime isolation,
or rejection.

Persistent cache schema 4 contains only mechanically derived summaries.
Process-local final indexes use TypeScript's immutable semantic `Project` snapshot as authority.
The former `effectProjectSourceSignatures()` configured-source metadata scan no longer runs before every process-index
lookup.
An active-overlay regression proves that a refreshed semantic snapshot writes a new final index without closing the
bridge.

`ForeignBorrowed` no longer triggers a complete source and callable scan.
A reached inferred candidate uses TypeScript 7.0.2's `Checker.getSignatureUsage()` and walks backwards through exact
callable owners.
Each usage must resolve to its own call edge.
Non-call escapes,
top-level or excluded callers,
unavailable queries,
and mismatched exact edges add an ordinary inbound and remove inferred provenance.

Final Linux x64 measurements over 13 files and 479 rules all reported zero diagnostics:

- cold empty persistent cache:
   838 milliseconds in Oxlint and 2.02 seconds wall;
- warm unchanged state:
   844 milliseconds in Oxlint and 1.41 seconds wall;
- changed source:
   835 milliseconds in Oxlint and 2.03 seconds wall;
- invalidated compiler options:
   824 milliseconds in Oxlint and 2.01 seconds wall.

The strict rule cannot soundly self-apply because its implementation necessarily calls bodyless TypeScript handles,
Oxlint host methods,
and ECMAScript collections.
`package/config/oxlint/src/overrides.ts` disables only this rule for its own package;
every other configured Oxlint rule remains active.
The strict unit and external-consumer boundaries still execute the rule.

The final package verification processes 91 files rather than the traversal phase's 143 because catalog implementation
and test files were deleted.
Build,
type lint,
all unit tests,
semantic-host lifecycle,
external consumer,
and package Oxlint pass.

## Schema 4 loses deliberate omission identities

Issue #427 found a correctness gap in the schema 4 cache.
A fresh summary scan can catch the TypeScript 7.0.2 tuple serializer panic,
omit only the affected callable,
and record its key in process-local `omittedCallableKeys`.
The completeness assertion receives that set and accepts the deliberate absence.

`PersistentEffectCacheHit` restores summaries,
`dependenciesResolved`,
and `directDependencies`.
It does not restore omitted callable keys.
The cache-hit branch in `effect-demand-index.ts` therefore returns without repopulating the set.
A warm completeness assertion sees a caller edge with no callee summary and no omission identity,
then throws `Owned effect edge lacks callee summary`.

A one-worker cold-to-warm control over `package/translation-repair` measured:

- cold:
  188 findings,
  no semantic-evidence failures,
  and 2 deliberate omissions;
- immediate warm rerun:
  166 findings,
  126 semantic-evidence failures,
  and no restored omission records;
- repeated one-worker warm rerun:
  the same 166 findings and 126 failures;
- two isolated default-worker cycles:
  the same 188-finding cold fingerprint and 178-finding warm fingerprint in each cycle,
  while warm semantic failures varied from 73 to 71.

The warm cache therefore changes semantic coverage,
worker count changes the surviving diagnostic fingerprint,
and default-worker warning volume remains unstable.
The controlled matrix does not prove that a cache-write race
or scheduling alone caused earlier non-isolated diagnostic-count variation.
This is not a performance-only regression.
The plugin logger's warnings are outside Oxlint's diagnostic channel,
so the damaged 166-finding run still prints `Found 0 warnings`.

A complete fix must either persist and validate per-source omission identities with a schema bump,
or refuse to persist a source containing an omission.
The former retains narrow fail-closed behavior without recomputing the upstream panic in every process.
Acceptance requires exact diagnostic equality across cold and warm processes,
plus one-worker and default-worker equality.
The upstream panic and detailed reproduction are recorded in
[`typescript-go-tuple-type-panic.md`](typescript-go-tuple-type-panic.md).

## Traversal-phase resolution

The rule remains an Oxlint JavaScript rule and uses only Oxlint's released JavaScript-plugin boundary.
No Oxlint or `tsgolint` fork or upstream contribution is involved.

`effect-summaries.ts:148-268` now creates one mutable index for an exact TypeScript project snapshot.
Each Oxlint visitor adds its active source to that index.
`effect-demand-index.ts:124-422` scans active-source callables,
then follows only owned callee and callback source identities recorded on their call edges.
An unrelated configured source is not summarized merely because it belongs to the TypeScript project.

The persistent format is schema 3.
`effect-owned-call-edge.ts:131-158` stores exact callee and callback source paths beside callable keys.
`effect-reached-edge.ts:85-118` rejects an owned edge whose source is outside the indexed snapshot.
`effect-reached-edge.ts:131-156` also rejects an owned callable key whose loaded source lacks its summary.
These failures no longer reach Oxlint as diagnostics.
A failure inside the rule,
 the bridge,
 or the upstream API is not a fact about the file being
linted,
 so blaming that file with a lint issue misattributes it;
the rule logs a warning naming the file instead.
Nothing becomes an assumed-safe call either way,
 because the catch wraps the whole
verification and no offer is made from a partial state.

`effect-dependency-closure.ts:162-185` merges semantic call edges with module dependency edges.
Fresh entries are published only after every reached source has contributed those edges.
A global-script call without an import therefore invalidates its caller when the callee source changes.
The regression lives in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/effect-summary-invalidation.unit.test.ts`.

`ForeignBorrowed` is declaration-wide rather than call-path-local.
When a reached summary contains explicit foreign provenance,
`effect-demand-index.ts:343-360` conservatively expands every owned source before propagating it.
The regression includes a foreign caller and an otherwise unreached ordinary caller of the same helper;
the helper is not incorrectly classified as wholly foreign.
This fallback preserves correctness but can cost more than an ordinary demand-only traversal.
Its marker-heavy worst case was not isolated by the 13-file acceptance benchmark.

Demand-driven scope applies to effect-summary construction,
not every cache-validation operation.
`effectProjectSourceSignatures()` still reads metadata for the configured project's source membership
before process-index reuse.
The measured 834-source semantic project satisfies the exact target,
but this result is not a general bound for larger projects.

Analysis has a 120-second project-wide runtime safety ceiling.
That ceiling fails closed for pathological graphs;
it is not the 10-second acceptance limit.
The 10-second result is an empirical gate on the exact command and workload.
`effect-analysis-budget.ts` raises `analysis-incomplete` when the ceiling is exhausted,
and a zero-budget regression proves no partial summary is returned.
The 10-second package-config acceptance target remains a stricter workload-specific gate.

### Traversal-phase acceptance measurements

The traversal-phase disposable-worktree measurements used commit `656444e0a`.
The cold command removed both
`node_modules/.cache/prefer-readonly-parameter-type`
and the generated Oxlint config before timing the exact user command.
No earlier lint run or generated summary was required for the result.

- Cold empty state:
  928 milliseconds reported by Oxlint,
  2.11 seconds wall time.
- Warm unchanged state:
  918 milliseconds reported by Oxlint,
  1.49 seconds wall time.
- Changed `package/config/oxlint/src/index.ts` source:
  924 milliseconds reported by Oxlint,
  2.11 seconds wall time.
- Invalidated compiler-option surface:
  922 milliseconds reported by Oxlint,
  2.16 seconds wall time.

Every case reported zero warnings and zero errors over 13 files with 479 rules.
The reproduced pre-change cold baseline on commit `6e5cfe99d` was 49.6 seconds in Oxlint.

The traversal-phase package verification also passed:

- package build;
- package type lint;
- every package unit test;
- semantic-bridge host lifecycle test;
- external-consumer host test;
- package Oxlint with zero warnings and zero errors over 143 files.

## Root cause

### The package task enables a project-owned semantic rule

`mise.toml:581-588` builds the bundled config when necessary,
then invokes Oxlint with type-aware linting:

```toml
[task_templates."lint:oxlint"]
description = "Lint with Oxlint"
shell = "node --input-type=module-typescript -e"
run = """
{{vars.dispatch_workspace_node}}
ensureOxlintConfig()
runWorkspaceNode('package/monorepo-task/core-exec', 'oxlint-wrapper', ['--type-aware'])
"""
```

Oxlint's type-aware and type-check path is not the dominant cost.
The project-owned JavaScript rule opens its own TypeScript 7 synchronous API.
For every linted TypeScript file,
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types.ts:154-181`
opens the semantic project and requests a whole-project effect index:

```typescript
Program(node: ForeignBorrowed<ESTree.Program>,): void {
  if (!isEnforcedTypeScriptSource(context.filename,))
    return;
  // ...
  const session = openSemanticFile({
    fileName: context.filename,
    sourceText: context.sourceCode.text,
    hasBOM: context.sourceCode.hasBOM,
  },);
  const effectIndex = buildEffectSummaryIndex({
    project: session.project,
    activeSourceFile: session.sourceFile,
  },);
```

The semantic project contained 834 source files in the reproduction.
Only 13 files were Oxlint targets.
`effect-summaries.ts:87-145` starts from every TypeScript project source and retains non-declaration
workspace dependencies,
so the semantic rule's workload is not bounded by Oxlint's displayed target count:

```typescript
const fileNames = [...new Set([
  ...project.program.getSourceFileNames(),
  activeSourceFile.fileName,
],),].toSorted();

const indexedSourceFiles = fileNames.flatMap(function retainIndexedSource(fileName,): SourceFile[] {
  // ...
  if (!project.program.isSourceFileFromExternalLibrary(sourceFile,))
    return [sourceFile,];
  if (isWorkspaceSourceFileName(fileName,))
    return [sourceFile,];
  // ...
},);
```

The current cache scope held 337 indexed source entries.
In the 62.9-second run,
the first `openSemanticFile` record appeared at `02:09:21.909Z`.
The second appeared at `02:10:23.792Z`.
The 61.883-second interval between those records strongly locates the delay in the first callback's project-index
initialization.
It does not attribute every millisecond exclusively to that source file.
The remaining file callbacks reused the process-local final index.

`effect-final-index-cache.ts:99-108` explains that within-process reuse:

```typescript
const cached = finalIndexByProject.get(projectKey,);
if ((cached === undefined)
  || (cached.fileListDigest !== fileListDigest)
  || (!sourceSignaturesEqual({
    left: cached.sourceSignatures,
    right: sourceSignatures,
  },)))
  return FINAL_EFFECT_INDEX_CACHE_MISS;
```

### This run had no reusable persistent entry set

All 337 cache entries for
`package/config/oxlint/tsconfig.json`
had modification times inside the 63.030-second logger interval.
They totalled 2,755,625 bytes.
The rule rebuilt the current schema-2 entry set instead of reading a reusable set.

The evidence proves a cold current identity,
but does not distinguish among these immediate triggers:

- analyzer digest rotation after semantic-plugin source changes;
- project membership or declaration-surface changes;
- compiler-option or lockfile changes;
- first run after cache deletion or schema rotation.

A disposable-worktree probe confirmed the invalidation mechanism.
Between cache populations,
the recorded `fileListDigest` changed and the next custom-rule run rebuilt in 64.3 seconds.
With no further source change in that worktree,
the same custom-rule run finished in 894 milliseconds,
and the exact mise task finished in 1.0 seconds.

### Cold analysis repeats synchronous manifest misses

The cold run also exposed a local hot path.
`intrinsic-effect-query.ts:69` caches parsed identities by discovered package root:

```typescript
const packageIdentityByRoot = new Map<string, PackageIdentity | typeof NO_PACKAGE_IDENTITY>();
```

However,
`intrinsic-effect-query.ts:115-139` walks from each declaration directory to a package manifest and logs every
missing candidate:

```typescript
if (markerIndex === (-1)) {
  for (const directory of ancestorDirectories(dirname(normalized,),)) {
    try {
      const parsed: unknown = JSON.parse(readFileSync(
        `${directory}/package.json`,
        'utf8',
      ),);
      if (hasPackageIdentityFields(parsed,)) {
        return {
          packageRoot: directory,
          packageName: parsed.name,
        };
      }
    }
    catch (error) {
      l.debug(`could not read workspace package identity in ${directory}: ${String(error,)}`,);
    }
  }
}
```

`intrinsic-effect-query.ts:189-203` performs this directory walk before consulting the root cache:

```typescript
const rootAndName = packageRootAndName(fileName,);
if (rootAndName === NO_PACKAGE_IDENTITY)
  return NO_PACKAGE_IDENTITY;
const cached = packageIdentityByRoot.get(rootAndName.packageRoot,);
```

The observed run wrote 21,616 `intrinsic-effect-query` records,
including 11,878 failed reads of a `src/package.json` path.
The file sink appends every record as JSONL at
`package/logger/core/src/sink/file.ts:274-283`:

```typescript
async function write(record: object,): Promise<void> {
  // ...
  await state.appendFile(
    state.filePath,
    `${JSON.stringify(record,)}\n`,
  );
}
```

Caching each visited directory's resolved package root or negative result is a direct local optimization target.
The diagnosis did not modify that code because the user requested an explanation,
not a performance fix.

### Sixteen Rust threads do not parallelize the semantic callback

The upstream source audit used Oxc tag `apps_v1.74.0`,
commit `2d4e8d20644e0e7446f0a381894b45ea339a0625`.
`apps/oxlint/src/js_plugins/external_linter.rs:151-198` says the callback runs on the main JavaScript thread,
while the calling Rust worker blocks on `rx.recv()`:

```rust
/// `ThreadsafeFunction` executes the callback
/// on main JS thread, and therefore it may have to wait for a previous `lintFile` call to complete.
/// Use an `mpsc::channel` to wait for the result from JS side, and block current thread until `lintFile`
/// completes execution.
// ...
if status == Status::Ok {
    match rx.recv() {
```

The displayed `using 16 threads` describes Oxlint's Rust pool.
It does not make this synchronous project index build 16-way parallel.
The run also logged 8 later `spawn ENOMEM` failures during external package-effect inference.
That allocator interaction is documented separately in
`doc/troubleshooting/oxlint-js-plugin-lazy-child-enomem.md`.
It is a secondary failure signal,
not the cause isolated by the rule-disable probe.

### The summary is Oxlint time, not mise wrapper time

At the audited Oxc tag,
`apps/oxlint/src/lint.rs:103` starts the timer before lint setup,
and `:508-556` executes `lint_files` before passing `now.elapsed()` to the output formatter.
The 62.9-second line therefore includes the synchronous JavaScript callback.
It cannot be explained by mise after Oxlint exits.

## Historical invalidation model (cache schema 2)

Schema 1 addressed entries by a whole-project content digest,
so editing any file invalidated every entry in the scope
and one changed line forced a full rebuild.

Schema 2 addresses entries by scope,
file path,
and source digest.
Each envelope revalidates against:

- recorded content digest of every non-declaration workspace file in the entry's transitive module-dependency
  closure;
- project membership through a file-list digest;
- declaration file contents for:
    - `.d.ts`
    - `.d.mts`
    - `.d.cts`
- global or module augmentations authored in non-declaration sources;
- resolved compiler options;
- governing lockfile content.

An edit therefore invalidates the edited file and files whose closure contains it.
Declaration,
compiler-option,
lockfile,
and membership changes still invalidate the whole scope.
Analyzer source changes rotate `analyzerDigest` and rebuild everything once.

### Dependency closure edges

Closure edges come from static `import` and `export ... from` declarations,
`import name = require(...)` external references,
literal dynamic `import()` arguments,
and literal `import('...')` type queries.

A runtime-variable dynamic `import()` is deliberately not an edge.
The checker types its result independently of any workspace file's content,
so no other file can change the importing file's summaries through that call.
Value flow through the call stays fail-closed inside effect analysis itself.

Before this refinement,
files such as `file-enforcer/src/cli.ts` (`await import(configPath)`) fell back to whole-scope closures.
Every transitive dependent of such a file was then invalidated by any edit:
39 of 276 file-enforcer-scope entries revalidated against the whole scope.

A non-literal `import('...')` type-query argument does shape checker semantics and still fails the file's closure
closed to the whole scope.
Regression coverage lives in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/effect-summary-invalidation.unit.test.ts`.

## Verification

### Versions and harness

- Oxlint:
  `1.74.0`.
- TypeScript synchronous API:
  `7.0.2`.
- Upstream source:
  Oxc tag `apps_v1.74.0`,
  commit `2d4e8d20644e0e7446f0a381894b45ea339a0625`.
- Target:
  13 files selected by `package/config/oxlint`.
- Semantic project:
  834 TypeScript source files,
  with 337 persistent effect-summary entries for the current scope.

The exact user-boundary harness was:

```bash
mise run //package/config/oxlint:lint:oxlint
```

The isolation harness used a disposable worktree and a temporary config that changed only one rule:

```javascript
import base from './package/config/oxlint/dist/final/node/index.mjs';

export default {
  ...base,
  rules: {
    ...base.rules,
    'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'off',
  },
};
```

A standalone probe run from the repository root separated semantic-project opening from summary-index construction:

```bash
node --input-type=module <<'EOF'
import { readFileSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
import {
  closeSemanticBridge,
  openSemanticFile,
} from './package/oxlint-plugin/prefer-readonly-parameter-type/dist/final/node/index.mjs';

const fileName = `${process.cwd()}/package/config/oxlint/src/index.ts`;
const sourceText = readFileSync(fileName, 'utf8');
const startedAt = performance.now();
openSemanticFile({ fileName, sourceText, hasBOM: false });
const openedAt = performance.now();
closeSemanticBridge();
const closedAt = performance.now();
process.stdout.write(`${JSON.stringify({
  openMilliseconds: Number((openedAt - startedAt).toFixed(1)),
  closeMilliseconds: Number((closedAt - openedAt).toFixed(1)),
  totalMilliseconds: Number((closedAt - startedAt).toFixed(1)),
})}\n`);
EOF
```

It reported:

```json
{"openMilliseconds":181.7,"closeMilliseconds":0.4,"totalMilliseconds":182.1}
```

This single probe does not establish a latency bound.
It does show that semantic-project opening alone did not consume the 10-second target on the measured host.
The pre-resolution whole-project summary index was the demonstrated dominant cold path.

### Working catalog

- Exact task in the disposable worktree,
  unchanged immediately after cache population:
  1.0 seconds,
  13 files,
  479 rules.
- Custom rule enabled,
  type-aware and type-check disabled,
  stable warm cache:
  894 milliseconds,
  13 files,
  428 rules.
- Custom rule disabled,
  type-aware and type-check enabled:
  396 milliseconds,
  13 files,
  478 rules.
- Custom rule disabled,
  type-aware and type-check disabled:
  234 milliseconds,
  13 files,
  427 rules.

### Cold or invalidated catalog

- Supplied exact task:
  62.9 seconds,
  13 files,
  479 rules.
- Custom rule enabled,
  type-aware and type-check disabled,
  after the observed file-list digest rotation:
  64.3 seconds,
  13 files,
  428 rules.
- The first semantic callback occupied 61.883 seconds of the supplied run.
- The cold logger contained 21,616 intrinsic-effect-query records and 8 `spawn ENOMEM` records.

The controlled comparison isolates one rule as the dominant cost.
Retaining Oxlint type-aware and type-check analysis while disabling only that rule changed 62.9 seconds to
396 milliseconds.
Conversely,
retaining the custom rule while disabling Oxlint type-aware and type-check analysis still took 64.3 seconds.
This comparison does not benchmark `oxlint-tsgolint` independently.

### Earlier measured behavior

The 2026-07-20 file-enforcer scope contained 276 indexed files.
The `buildEffectSummaryIndex` phase,
with a fresh bridge per run,
measured:

- cold empty cache:
  30 seconds for 6,614 callables;
- warm unchanged cache:
  0.4 seconds with 276 persistent hits and zero scans;
- warm after a one-line entry-point or leaf edit:
  0.4 to 0.9 seconds,
  with one file rescanned and 275 entries preserved.

Before schema 2,
the same edit cost 13.7 seconds and preserved no entries.

End-to-end `mise run //package/file-enforcer/core:lint:oxlint` measured:

- analyzer-digest rotation:
  24.5 seconds,
  compared with 45.1 seconds before the cache work;
- warm unchanged:
  3.2 seconds,
  with a 2.8-second rule-off floor;
- after a one-line edit:
  5.6 seconds.

## Verified workarounds

### Repeat after semantic inputs stabilize

In the disposable worktree,
the exact package task fell from the cold-run range to 1.0 second on an unchanged repeat.
This verifies the expected warm path for the observed cache identity,
not a universal time guarantee.

Tradeoff:
the first run after analyzer,
file-list,
declaration,
compiler-option,
or lockfile invalidation still pays the cold analysis cost.
Continuous edits on those surfaces can keep rotating the cache.

### Disable the project rule only for a narrow diagnostic run

The temporary config in the Verification section reduced the type-aware and type-check run to 396 milliseconds.
This is useful only to isolate another lint problem.

Tradeoff:
it removes readonly-parameter,
mutation-contract,
and external-effect enforcement for that run.
It is not an acceptable permanent package configuration.

### Preserve the persistent cache as operational guidance

Keep `node_modules/.cache/prefer-readonly-parameter-type` across ordinary lint invocations only when the run has no
deliberately omitted callable.
The cache is content-addressed and validates its dependency surfaces,
but schema 4 does not validate omission completeness.

Tradeoff:
stale identities remain until cache maintenance evicts them,
and intentional cache deletion forces the next run cold.
If an omission warning appears,
the next warm run is not an acceptable verification result until omission metadata round-trips.

## What does not work

- Removing `--type-aware` does not address this incident.
  The custom rule uses its own TypeScript API and still took 64.3 seconds without Oxlint type-aware or type-check
  analysis.
- Passing
  `--allow=prefer-readonly-parameter-type/prefer-readonly-parameter-types`
  did not disable the configured JavaScript rule in this setup.
  The measured command still ran 479 rules and took 62.2 seconds.
  An explicit config override to `off` was required for the isolation probe.
- `--debug=timings` does not attribute JavaScript-plugin rule time in Oxlint 1.74.0.
  The 64.3-second run's timing list showed only millisecond-scale native rules.
  Upstream issue
  [oxc-project/oxc#19745](https://github.com/oxc-project/oxc/issues/19745)
  already tracks JavaScript-plugin timing support.
- More Oxlint threads do not parallelize the callback.
  Upstream routes JavaScript rule execution through the main JavaScript thread.
- A cache directory containing files is not proof of a hit.
  Every envelope must match analyzer,
  project,
  declaration,
  compiler,
  lockfile,
  and dependency identities.

## When lint time regresses

Check these surfaces in order:

1.  Semantic integrity before timing.
    Compare cold and warm diagnostic fingerprints,
    search stderr for `semantic evidence unavailable`,
    and compare one-worker with default-worker output.
2.  Analyzer digest rotation.
    Any semantic-plugin source change rebuilds every scope once.
3.  Whole-scope surface churn.
    Lockfile,
    compiler-option,
    declaration,
    and project-membership changes legitimately invalidate whole scopes.
4.  Whole-scope closure fallback.
    Files whose module references cannot be statically resolved snapshot the whole indexed scope.
    Their transitive dependents inherit that fallback.
    Probe `directModuleDependencies` for `MODULE_DEPENDENCIES_UNRESOLVED`.
5.  Repeated manifest discovery.
    Count `could not read workspace package identity` records and cache ancestor-directory results when the count is
    high.
6.  Cold rebuild cost.
    Synchronous IPC to the TypeScript 7 Go child dominated the earlier profile.
    Incremental caching avoids repeated rebuilds,
    but does not reduce a legitimate cold rebuild.

Prior note from the 2026-07-20 investigation,
not reverified during this incident:
whole-repo serial warm lint above the 60-second goal was tracked in issue #374.
The current `mise.toml:831` still pins repository-wide lint fanout children with `OXLINT_THREADS = "1"`.

## Upstream filing decision

No `.out-of-scope/` entry matches Oxlint,
JavaScript plugins,
or this project-owned semantic rule.

The upstream duplicate search covered open and closed issues and pull requests for
`timings`,
`JS plugin`,
`custom rule timings`,
and `RuleTimingSource`.
It found open issue
[oxc-project/oxc#19745](https://github.com/oxc-project/oxc/issues/19745),
which already states that merged JavaScript visitors prevent per-rule attribution and requests a separate timing
path.
The issue has no comments.

The six filing constraints for the 62.9-second incident are:

### Is it really upstream's fault?

No.
Oxlint waited correctly for a project-owned synchronous rule.
The expensive whole-project algorithm,
cache identity,
repeated manifest probes,
and file logging belong to this repository.

### Can upstream fix it?

Oxlint can improve JavaScript-plugin timing visibility,
but it cannot make this custom effect analysis cheap without changing the rule's semantics or implementation.

### Are they supporting this use case?

Oxlint supports JavaScript plugins,
currently marked alpha.
Per-rule JavaScript timing is not present in 1.74.0 and is explicitly tracked as future work in issue #19745.

### Would the repository welcome a contribution?

Yes with review and disclosure.
Oxc `CONTRIBUTING.md:12-21` permits AI assistance,
requires disclosure,
and requires contributors to understand and test submissions.

### Will they likely fix it?

The timing capability is accepted as an open issue,
but there is no schedule or maintainer commitment in its empty thread.
The local cold-cache cost is not an upstream fix target.

### Have we prototyped a minimal upstream fix compatible with their architecture?

No.
Constraint 1 fails for the incident,
so the auto-prototype gate does not trigger.
The local consumer boundary already provides the cache and owns the remaining optimization targets.

Decision:
do not file a new upstream issue.
Issue #19745 already captures the only upstream capability gap encountered during diagnosis.
The project-specific reproduction confirms its stated consequence but adds no new architectural fact,
so there is no additive comment to post.
