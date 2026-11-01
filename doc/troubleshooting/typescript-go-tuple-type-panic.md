# typescript-go 7.0.2 tuple serialization panic plus cached omission metadata causes nondeterministic Oxlint coverage loss

TypeScript's sync API panics while serializing some instantiated tuple types.
The readonly analyzer catches a fresh panic and omits only that callable,
but its persistent cache does not store the omitted callable identity.
A warm run forgets why the summary is absent,
throws a secondary `Owned effect edge lacks callee summary` error,
and loses diagnostics according to rule and worker execution order.

The raw logger prints stacks but Oxlint still reports `Found 0 warnings`.
The affected run can therefore lose coverage without a counted analysis-integrity diagnostic.

## Symptom

During `mise run lint:oxlint` at the repository root:

```text
[error] [prefer-readonly-parameter-types] [Program] semantic rule failed: Error: panic: interface conversion: checker.TypeData is *checker.TypeReference, not *checker.TupleType
```

The message is followed by a Go stack and then `Error running tsgolint: "exit status: exit status: 2"`.
The report is attached to `package/rss/app/src/index.ts`,
 at `1:1`,
 which is the file whose analysis is lost.

The historical `package/test-harness/core/src/expect-matchers.ts` moved on 2026-10-04
 to `package/test-harness/core-expect/src/expect-matchers.ts`.

An earlier reading of mine named `package/test-harness/core/src/expect-matchers.ts` instead,
 inferred from diagnostics printed near the panic in an interleaved log.
That was wrong,
 and it is why the reproduction below looked unavailable for so long:
 every narrowing attempt probed the wrong package.

The panic is not caused by anything in this repository's rule changes.
It appears in all five workspace sweeps taken while investigating an unrelated soundness
question,
 the earliest of which predates every change in that series.

### Warm-cache variant

A fresh package run can log the upstream panic and continue with a deliberately omitted callable.
The unchanged warm run instead emits repeated records such as:

```text
[warn] [effect-contract] semantic evidence unavailable for .../splice-slices.unit.test.ts:
node-not-found: SemanticBridgeError: Owned effect edge lacks callee summary:
.../splice-slices.unit.test.ts:967:2384:263.
    at assertReachedCallSummaries (.../plugin-prefer-readonly-parameter-type.mjs:4:65412)
```

The callable key names `chunkAt`.
The stack is followed by bundled plugin frames,
while Oxlint's summary still says `Found 0 warnings`.
The exact warning and finding counts change with worker count.

### A second surface: a dependency's shipped implementation

The panic now also strikes inside code this repository does not own.
Measured at `94af5da15`,
 reading the cause `effect-demand-index.ts` logs beside each omission:

```text
omitting package/rss/app/src/index.ts:2646:3443:219 ... Error: panic: interface conversion
omitting package/rss/app/src/index.ts:3500:4182:263 ... Error: panic: interface conversion
omitting node_modules/.../@optique/core@1.2.0/.../dist/facade.js:12673:15456:263 ... Error: panic: interface conversion
```

Five panics,
 three omitted callables,
 so the counts do not correspond one to one.

The third is new,
 and it appeared only once the external channel began loading shipped implementations,
 which needed the worker-count defect fixed first.
It stays sound:
 an omitted external callable answers `NO_EFFECT_SUMMARY`,
 the effect resolver reports unavailable,
 and the consumer's call falls to the unresolved boundary,
 which withholds.

The practical consequence is for reading captures.
A sweep must be searched for omissions under `node_modules` as well as under `package`,
 which no capture before this one needed.

## Root cause

The stack names the failing frames:

```text
github.com/microsoft/typescript-go/internal/checker.(*Type).AsTupleType(...)
	github.com/microsoft/typescript-go/internal/checker/types.go:693
	github.com/microsoft/typescript-go/internal/api/proto.go:675 +0x9e5
github.com/microsoft/typescript-go/internal/api.checkerSetup.newTypeResponse(...)
	github.com/microsoft/typescript-go/internal/api/session.go:465
```

Line numbers in the stack come from the installed build,
 TypeScript 7.0.2.
The excerpts below come from a shallow clone of `microsoft/typescript-go` at commit
`f209df30`,
 so the same code sits at different line numbers.

The serializer decides a type is a tuple from that type's own object flags,
 then asserts
tuple data immediately,
 at `internal/api/proto.go:735-747`:

```go
case flags&checker.TypeFlagsObject != 0:
	resp.ObjectFlags = uint32(t.ObjectFlags())
	objectFlags := t.ObjectFlags()
	if objectFlags&checker.ObjectFlagsReference != 0 {
		var ref *checker.TypeReference
		if objectFlags&checker.ObjectFlagsTuple != 0 {
			tuple := t.AsTupleType()
			ref = tuple.AsTypeReference()
			resp.ElementFlags = tuple.ElementFlags()
```

`AsTupleType` is an unchecked type assertion,
 at `internal/checker/types.go:695`:

```go
func (t *Type) AsTupleType() *TupleType { return t.data.(*TupleType) }
```

The checker's own predicate for "is this a tuple" never does that.
It reads `Reference` from the type and `Tuple` from the type's target,
 at
`internal/checker/checker.go:23473-23475`:

```go
func isTupleType(t *Type) bool {
	return t.objectFlags&ObjectFlagsReference != 0 && t.Target().objectFlags&ObjectFlagsTuple != 0
}
```

The reason for that indirection is documented at `internal/checker/types.go:956-960`:

```go
// TupleType:
// ObjectFlagsReference|ObjectFlagsTuple: Originating generic tuple type (synthesized)

// TypeReference
// ObjectFlagsReference: Instantiated generic class, interface, or tuple type
```

So by the stated invariant the two flags together mark the synthesized originating tuple,
which does hold `*TupleType` data,
 and it is built in exactly one place,
`internal/checker/checker.go:24794`:

```go
t := c.newObjectType(ObjectFlagsTuple|ObjectFlagsReference, nil)
```

An instantiated tuple is a `*TypeReference` carrying `ObjectFlagsReference` alone,
 and
`internal/checker/checker.go:25091-25101` builds it without adding the tuple flag,
since `ObjectFlagsPropagatingFlags` at `internal/checker/types.go:618` covers only
`ContainsWideningType`,
 `ContainsObjectOrArrayLiteral` and `NonInferrableType`:

```go
t := c.newObjectType(ObjectFlagsReference|objectFlags|c.getPropagatingFlagsOfTypes(typeArguments, TypeFlagsNone), target.symbol)
d := t.AsTypeReference()
```

The panic is proof that some type reaching `newTypeResponse` carries `ObjectFlagsTuple`
while holding `*TypeReference` data,
 which the documented invariant says cannot happen.
Whether that type is produced by a path this trace did not find,
 or the invariant is
simply not enforced,
 is not established here.
What is established is that the serializer is the only reader that trusts the type's own
tuple flag to imply tuple data,
 and that the checker's own predicate does not,
 so the
serializer is where a mismatch becomes a crash rather than a wrong answer.

An earlier reading of mine was wrong and is recorded so it is not re-derived:
I first suspected this repository's own accessor-body scan in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/effect-packaged-callable-origins.ts`,
because that scan asked the checker about every name it walked,
 including names in type
positions.
Counting `semantic rule failed` across the sweeps disproved it:
 the panic is present in
the baseline sweep taken before that scan existed.
Keeping the scan out of type positions is still correct on its own terms and landed
separately,
 but it is not a fix for this.

## Verification

Version under test:
 TypeScript 7.0.2,
 resolved at
`node_modules/.pnpm/typescript@7.0.2/node_modules/typescript`.
Source excerpts:
 `microsoft/typescript-go` at commit `f209df30`.

Reproduces:

```bash
mise run lint:oxlint
```

Does not reproduce,
 each run completing with findings and no panic:

```bash
# Historical command; the file moved to package/test-harness/core-expect/src/expect-matchers.ts on 2026-10-04.
# 7 findings, no panic
node package/monorepo-task/core-exec/dist/final/node/oxlint-wrapper.mjs --type-aware package/test-harness/core/src/expect-matchers.ts

# 52 findings, no panic
node package/monorepo-task/core-exec/dist/final/node/oxlint-wrapper.mjs --type-aware package/test-harness/core

# 422 findings, no panic
node package/monorepo-task/core-exec/dist/final/node/oxlint-wrapper.mjs --type-aware package/module

# 52 findings, no panic
mise run //package/test-harness/core:lint:oxlint
```

The file the run is working through when it panics therefore does not trigger it on its
own,
 nor does its package,
 nor its whole package family.
The trigger depends on the wider program the repository-scoped run assembles,
 which is
consistent with an instantiated tuple type that only exists once the larger set of source
files is in one program.

Counting the failure across sweeps,
 each on a clean tree:

```bash
rg --count-matches 'semantic rule failed' -- <sweep-output>
```

returns exactly 1 for every sweep taken,
 including the earliest baseline.

## Upgrading does not fix it

`typescript@7.1.0-dev.20260726.1` was installed across the whole workspace and swept.
The panic persists,
 with the same message and the same site.

The sweep produced 1859 findings and 23 offers,
 identical to the run on 7.0.2,
 and one `semantic rule failed` with the same
`interface conversion: checker.TypeData is *checker.TypeReference, not *checker.TupleType`.

The stack is what makes the correspondence exact.
On 7.0.2 it reads `internal/api/proto.go:675`;
 on the nightly it reads `internal/api/proto.go:741`,
 which is the line the clone at commit `f209df30` holds the unguarded `t.AsTupleType()` on.
So the nightly ships that code with no guard added.

The source agrees.
The newest upstream commit before that nightly's cutoff is `8d29e62f`,
 dated 2026-07-24,
 and `git show 8d29e62f:internal/api/proto.go` still has the unguarded branch.
`8d29e62f` is titled "Fix panic in variadic tuple relationship checking",
 which is a panic in the relater rather than in this serializer,
 so it is a different bug that happens to share the word tuple.

The catalog was returned to `'typescript': '>=7.0.2'` and reinstalled,
 because carrying a prerelease that fixes nothing is cost without benefit.
A future version is worth retesting the same way:
 pin the catalog entry,
 install,
 sweep,
 and grep the output for `semantic rule failed`.

## Fresh omission handling works, warm omission handling does not

Recorded 2026-08-07:
the first omission implementation was fail-closed only in propagation.
`assertReachedCallSummaries` rejected an intentionally missing callee and aborted the program.
Commit `32a648016` added process-local omission identities to the assertion,
which restored the intended fresh-process behavior.

The catch in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/effect-demand-index.ts:257`
records the omitted identity:

```ts
catch (error) {
  omittedCallableKeys.add(callableKey(declaration,),);
  dl.warn(
    `omitting ${callableKey(declaration,)} from the effect index: ${caughtValueStack(error,)}`,
  );
  return [];
}
```

The assertion receives that process-local set at `effect-demand-index.ts:360`:

```ts
assertReachedCallSummaries({
  summaries,
  omittedCallableKeys,
},);
```

That path works on a fresh cache.
The cache path loses the fact.
`effect-summary-persistent-cache.ts:104` declares everything a hit restores:

```ts
export type PersistentEffectCacheHit = {
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly dependenciesResolved: boolean;
  readonly directDependencies: readonly string[];
};
```

There is no omitted-callable field.
The cache-hit branch at `effect-demand-index.ts:208` returns summaries and dependencies only:

```ts
if (hit !== LAYERED_SUMMARY_CACHE_MISS) {
  closureResolver.seedEdges({
    fileName: sourceFile.fileName,
    edges: hit.edges,
  },);
  return {
    fileSummaries: hit.summaries,
    dependencies: reachedSourceFileNames({
      fileSummaries: hit.summaries,
      indexedFileNames,
    },),
  };
}
```

The warm completeness assertion consequently sees the caller edge
 but neither the callee summary nor its omission identity.
It throws the secondary `Owned effect edge lacks callee summary` error.

The fix needs either:

- persisted,
  validated per-source omitted identities plus a cache schema bump;
- or refusal to persist a source containing an omission.

Persisting the identities retains narrow fail-closed behavior without rerunning the upstream panic in every process.

## The remaining cost, and the shape of a narrower catch

On a fresh cache,
the panic costs each affected callable its summary.
Its callers take opacity through the absent-callee branch,
so that direction is conservative.
On a warm cache,
the missing omission metadata expands the loss beyond those callables and makes coverage depend on execution order.
The cold and warm states are therefore different soundness states,
not only different log volume.

The trigger is narrower than "a tuple".
 `findGerundInText` in
`package/claude-code-plugin/statusline/src/activity.ts` writes
`lowercaseValue.match(GERUND_PATTERN,) ?? []`,
 whose type is `RegExpMatchArray | []`,
 and `[]` is an empty
tuple.
 The serializer reads `Reference | Tuple` off that type's own object flags and immediately asserts
tuple data,
 which an instantiated type does not carry.
 So the shape to look for when this fires elsewhere is a `?? []` or `: []` fallback beside a non-tuple array type,
not a declared tuple.

This site no longer exists.
 On 2026-10-12 the statusline package replaced both of its regex sites with linear scanners,
 so `findGerundInText` moved to
`package/claude-code-plugin/statusline/src/gerund.ts`
 and takes its matches from a scanner that returns a plain array,
 with no `?? []` fallback beside a `RegExpMatchArray`.
`mise run //package/claude-code-plugin/statusline:lint` now reports no omitted callable summaries for that package,
 which is the same warning quoted from the run that found this trigger.
 The trigger shape is unchanged and is what to look for in other packages.

A narrower catch would recover the 16.
 The panic surfaces when a type crosses the sync bridge,
so catching it at the type query rather than at the callable would lose one fact instead of one summary,
and the rule already treats an unresolved type as fail-closed nearly everywhere.
 The cost is that
`getTypeAtLocation` is called from about a dozen modules,
 so it means a helper plus a mechanical
replacement at every site,
 and every site then has to be checked for whether `undefined` really is its
fail-closed direction.
 Not attempted here:
 a wide mechanical change to recover precision is worth doing
deliberately rather than at the end of a long session.

## Warm-cache verification

Issue #427 supplied the first package-level reproduction with the extracted rules enabled.
A detached worktree at `da3f2f4f9710ffd353de90eef87f0114e3ded1fa` used TypeScript 7.0.2,
Oxlint 1.78.0,
and one Oxlint worker.
The plugin and shared config were rebuilt from that worktree.

After removing only cache entries whose payload named the disposable worktree,
the first run produced:

```text
188 findings
185 no-opaque-parameter-effects
3 prefer-readonly-parameter-types
0 semantic evidence unavailable
2 omitted callables after the upstream panic
```

The immediate unchanged rerun produced:

```text
166 findings
163 no-opaque-parameter-effects
3 prefer-readonly-parameter-types
126 semantic evidence unavailable
0 fresh omission records
```

A second one-worker warm run reproduced the same counts.
A later isolated default-worker matrix cleared the disposable project's cache before each cold-to-warm cycle:

- both cold runs produced the same 188-finding fingerprint and no semantic-evidence failures;
- both warm runs produced the same 178-finding fingerprint;
- warm semantic failures varied from 73 to 71 and changed category split.

The matrix proves cold-to-warm coverage loss,
one-worker versus default-worker diagnostic divergence,
and warning-count instability under default workers.
It does not prove that a cache-write race or scheduling alone caused earlier non-isolated diagnostic-count variation.

`readonly-rule-visitor.ts:108` catches the secondary error and logs `caughtValueStack(error)`:

```ts
catch (error) {
  rl.warn(
    `semantic evidence unavailable for ${context.filename}: ${
      error instanceof SemanticBridgeError
        ? `${error.reason}: ${caughtValueStack(error,)}`
        : caughtValueStack(error,)
    }`,
  );
}
```

This logger output is outside Oxlint's diagnostics.
The same warm run prints `Found 0 warnings and 166 errors` while stderr contains 128 logger warnings.
The minified stack is not an action path,
and the Oxlint summary does not disclose the missing analysis.

Until omission metadata round-trips,
extracted-rule acceptance cannot use warm output or compare worker counts.
The acceptance harness needs fresh-process versus warm-process diagnostic equality
 and one-worker versus default-worker equality.

## Minimal reproduction

Delta-debugging the affected file from 186 lines to 30 in 149 runs,
 then hand-reducing,
 gives a dependency-free case.
Every run took about a second,
 so the whole minimization cost a few minutes:

```ts
export function take<Fn extends (...args: never[]) => unknown,>(
  fn: Fn,
  args: Parameters<Fn>,
): void {
  void fn;
  void args;
}

export function use(): void {
  take(
    function render(): string {
      return '';
    },
    [],
  );
}
```

The essential ingredients are a generic parameter typed `Parameters<Fn>` and a call site
that instantiates it.
That matches the reading under "Root cause":
 an instantiated tuple is a `*TypeReference`,
 and the serializer branches on the type's own tuple flag.

None of these panic,
 which is what pins the generic instantiation as necessary:

```ts
export function take(args: [],): void {}
export function take(args: Parameters<() => string>,): void {}
export function take(args: [string, number,],): void {}
```

A variant returning the tuple-taking callable rather than accepting the tuple directly
panics as well,
 so the shape of the wrapper does not matter.

The original file reaches this through `memoizeAsync`,
 whose returned caller takes `{ args: Parameters<Fn> }`,
 called as `memoizedGetHtmlBody({ args: [], },)`.

## Why no workaround analyzes the construct

The panic happens while building the effect summary index,
 not while verifying,
 which a probe calling `buildEffectSummaryIndex` directly confirms.

More to the point,
 it happens inside the API's serialization of a type response.
Receiving the type object at all requires that response,
 so the rule cannot inspect the type's flags first and then decline:
 the first request that returns this type panics.
There is no sequence of API calls that reads the construct safely.

What is available is a smaller blast radius on a fresh cache.
A callable whose summary cannot be built is omitted from the index with a warning naming the cause,
rather than aborting the run for its whole file.
Callers of that absent callee take opacity,
and the rule skips verifying the omitted callable.

The warm-cache regression invalidates the broader prior claim that omission is fail-closed on both sides.
That statement is true only while `omittedCallableKeys` remains in process memory.
Once a persisted summary is read,
the analyzer can abort category evidence,
continue with partially loaded state,
and lose findings.
The verified cold/warm differential under "Warm-cache verification" supersedes earlier workspace counts
 in this document.

## Verified workarounds

The upstream panic has no semantic workaround at the consumer boundary.
The rule cannot inspect a type response that the API aborts while serializing.

The current shared config keeps the extracted rules off.
That avoids invoking them across test files carrying the reproduced tuple shape,
while the preference rule continues to run on ordinary source.
The tradeoff is explicit:
mutation,
opacity,
and contract policy are not enforced until #423's acceptance gate passes.

For investigation only,
deleting cache entries belonging to a disposable project gives one complete cold run.
The tradeoff is that the panic is recomputed,
the next unchanged run is damaged again,
and clearing user cache is not an acceptable operational fix.

The earlier claim that every per-package task avoids the panic is superseded.
`mise run //package/translation-repair/<package>:lint:oxlint` reproduces both the fresh omission and warm-cache failure
 when the extracted rules are enabled.

## What does not work

- Treating one warm run as a rollout baseline.
  Repeated default-worker runs changed both finding and failure counts;
  one-worker warm runs were stable but consistently incomplete.
- Rebuilding only the shared config.
  It does not restore omission metadata absent from already persisted payloads.
- Treating stderr stacks as counted lint warnings.
  Oxlint reports zero warnings because the plugin logger is outside its diagnostic channel.
- Inferring that the warm run is conservative because the fresh omission path is conservative.
  The warm run lost findings and changed a surviving diagnostic's cause set.
- Attributing the panic to this repository's accessor-body scan over type-position names.
  Disproved by the baseline sweep,
  as recorded under "Root cause".
- Passing paths through the root mise task,
  as in `mise run lint:oxlint -- <path>`.
  That task has no usage spec,
  so mise splices the argument into the script body and the run dies with `ERR_INVALID_TYPESCRIPT_SYNTAX`
   before Oxlint starts.

## Upstream filing decision

The warm-cache omission defect is in this repository's persistence model,
not in TypeScript,
so it needs no upstream report.
The upstream decision in this section applies only to the tuple serialization panic.

Filing upstream is out of scope by the repository owner's standing instruction for this
work,
 which grants unlimited budget "except actually filing it upstream".
No `.out-of-scope/` entry covers typescript-go or this bug class;
 the closest,
`.out-of-scope/typescript-project-references.md`,
 is about not adopting project
references and does not apply.
The six constraints are still walked so a future session can act if that instruction
changes,
 and no fileable draft is kept.

1.  **Is it really upstream's fault?
    **
    Yes.
     The panic is raised inside `internal/api`,
     in upstream's own serialization of a
    type response,
     from an unchecked assertion in `internal/checker`.
     No consumer input
    can be malformed enough to make an unchecked cast the correct behavior.
2.  **Can upstream fix it?
    **
    Yes,
     and cheaply.
     The serializer needs to establish tuple data before asserting it,
    the way `isTupleType` establishes it from the target rather than from the type's own
    flag.
     That is a guard in one branch of `newTypeResponse`,
     plus whatever exported
    helper the `api` package needs,
     since `Type.data` is unexported.
3.  **Are they supporting this use case?
    **
    Yes.
     The whole `internal/api` package exists to serve external consumers such as
    tsgolint,
     and `newTypeResponse` is on that path.
4.  **Would the repo welcome our contribution?
    **
    Not checked.
     `CONTRIBUTING.md`,
     issue templates and AI-assistance policy in
    `microsoft/typescript-go` were not read,
     because the filing decision was already
    settled by the standing instruction.
5.  **Will they likely fix it?
    **
    Not checked.
     The upstream tracker was not searched for a duplicate,
     for the same
    reason.
     Any future session acting on this must run that search first,
     since a
    duplicate report is itself an incident.
6.  **Have we prototyped a minimal fix?
    **
    No. Not attempted,
     because constraints 4 and 5 were deliberately left unevaluated and
    the prototype exists to make a filing credible.

A future session that is authorized to file must complete constraints 4,
 5 and 6 before
drafting anything,
 starting from the duplicate search.
