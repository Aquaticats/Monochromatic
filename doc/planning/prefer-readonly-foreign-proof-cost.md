# Narrowing which callables need a complete foreign-ownership proof

Working notes for task #34,
 fourth attempt.
Written before implementing,
and the reading below changes what the attempt should even be measured against.

## Two gates checked before writing code

### Unknown inbounds only remove provenance

`unknownInboundSummary` in `foreign-borrowed-complete-graph.ts` builds an edge whose
`directForeignByFormal` is all `false` and whose `foreignOriginsByFormal` is all empty.
In `inboundArgumentIsForeign` that fails the first test,
then fails `callerIndexes.length > 0`,
so the inbound reports not-foreign and the narrowing pass deletes the candidate.

So an unresolved caller,
 a non-call reference,
 a top-level call and a caller outside owned source all
subtract.
None can add.

### `closureFor` does answer for a file nothing loaded, with one trap

`edgesFor` falls through to `directModuleDependencies` resolved fresh from the program when a file
has no seeded or memoized edges,
so a forward closure can be taken for a marker-naming file the demand-driven index never touched.

The trap is the failure shape.
When any node's edges are unresolved,
`closureFor` returns `wholeScopeClosure()`,
whose `directDependencies` is empty while its meaning is "every indexed source".
Reading that as a reachable set gives the empty set for the case that means everything,
which is the unsound direction.
Any use of it here has to branch on `resolved` first.

A second limit:
an unloaded file's edges are module references only.
Semantic call edges enter through `includeDirectDependencies`,
 which runs inside `loadSource`.
Whether module reachability is a superset of semantic call reachability is not established,
and the skip is only sound if it is.

## What the reading turned up instead

`propagateForeignBorrowed` computes a greatest fixed point.
`initializeCandidates` seeds every parameter of any callable that has at least one inbound,
then narrowing deletes a parameter unless it is directly marked or every inbound argument is foreign.

A parameter therefore survives with no marker anywhere in the closure whenever the callable's
inbounds all pass a surviving parameter straight through.
Self-recursion is enough:
for `f(a) { return f(a); }` whose only in-scope usage is its own recursive call,
the single inbound has `foreignOriginsByFormal[0] = [0]`,
`candidates.get(f)` holds `0` from the optimistic seed,
so the test passes and the candidate is never deleted.

The result is a non-empty foreign set for a callable nothing marked.

### Which means the existing whole-scope gate already changes answers

`scopeNamesOwnershipMarker` skips the closure entirely for a scope whose text names no marker,
and its doc comment calls that "an equivalence rather than a trade".
For the case above it is not an equivalence.
A recursive callable in a marker-free scope is skipped and reported with no foreign parameters,
while the same callable in a scope that names a marker anywhere gets a closure that reports its
parameters foreign.

The direction is worth being precise about,
because it decides whether this is a defect or only an inconsistency.
Foreign ownership suppresses the read-only offer.
So the closure withholds an offer the skip emits.
Nothing is marked in either case,
so the parameter is not foreign,
so the offer is the correct answer and the withholding is the spurious one.

The gate is therefore more correct than the thing it skips,
and the inconsistency costs precision rather than soundness.

## What this does to the attempt

The planned per-callable reachability test extends the gate from whole-scope to per-callable.
If the reading above holds,
it will not leave the sweep unchanged:
it removes spurious withholding wherever a recursive callable sits in a marker-bearing scope,
which shows up as new offers.

Task #34's own bar,
"show the sweep unchanged at the current findings and offers rather than argue it",
was written against the three attempts that tried to reuse closure results,
where any diff was evidence of a defect.
It is the wrong bar for this attempt.
The right one:
every finding that changes must be a spurious withholding removed,
identified individually,
and no new offer may appear on a parameter anything writes.

## The attempt is refuted, and was not implemented

Three counterexamples,
 none of which a workspace-equivalence sweep would necessarily have contained.
That is the point:
the design fails deterministically,
 and measuring it could have passed.

### A marker need not be named in the file that applies it

This is the one that ends it.
`bindingContainsForeignBorrowed` and `expressionContainsForeignBorrowed` inspect checker types,
not source text.
A consumer can write

```ts
withForeign(function apply(value,) { return root(value,); },);
```

where the arrow's parameter is contextually typed by an API declaring `ForeignBorrowed<T>`,
and the consumer's own text names no marker at all.
The same holds for `root(getForeignValue(),)`,
where `directForeignByFormal` becomes true from the callee's return type.

So the set of files that can seed foreign provenance is not the set of files whose text names a
marker,
and any traversal rooted at marker-naming files starts in the wrong place.
Worse for the proposed direction:
the file declaring the marker is typically an outbound dependency of the caller,
so a forward walk from it never reaches the caller at all.

### Module reachability is not a superset of call reachability

The skip needed forward module closure to cover every call path.
It does not.
Two global script files can hold a caller and a callee with no module dependency between them,
and semantic call edges only enter the resolver through `seedEdges` and `includeDirectDependencies`,
both of which run inside `loadSource`.
A precomputed reachable set never sees edges that later loads add.

### Reflexive reachability

`closureFor(F)` does not put `F` in its own `dependencyDigests`,
so a directly marked root in a marker-naming file would have been missed by construction.

## What survives

The reframing does.
Markerless recursive components producing foreign candidates is confirmed independently,
and it means the shipped whole-scope gate already changes answers rather than being the equivalence
its comment claims.
That is a precision defect worth its own task,
 not part of this one.

The most promising untried direction is no longer about marker locality at all:
compute the foreign proof only when it can change a report.
Foreign ownership suppresses a read-only offer,
so a parameter already recorded as mutated,
 or already classified as something other than mutable,
has a verdict the foreign answer cannot move.
`createDemandDrivenEffectIndex.get` runs a closure for every callable asked about and hands the
result to `effectPublicSummary` whether or not any consumer reads it.
Deferring it until a verdict actually depends on it approximates nothing and skips whatever the
answer would not have changed.

## Design for the anchoring fix, task #36

Recorded here because it was worked out while this note was open,
and because it is the thing that has to land before any narrowing is measurable.

A plain least fixed point is not the fix,
and the reason is a concrete shape rather than a preference.
Take a marker-fed recursive helper:

```ts
function entry(marked: ForeignBorrowed<Held>,): void {
  helper(marked,);
}

function helper(value: Held,): void {
  helper(value,);
}
```

`helper`'s parameter is genuinely foreign,
and it has two inbounds:
one from `entry`,
 grounded in a marker,
and one from itself,
 grounded in nothing yet.
A least fixed point requires every inbound to be foreign at each step,
so the self-edge is never satisfied at the point the parameter would first enter,
and the parameter is lost.
That is the case the greatest fixed point exists to support,
which is why it is a greatest one.

The fix keeps the existing greatest fixed point and adds a grounding pass over its result:

-    Seeds.
     A candidate is seeded when the parameter is `directForeignBorrowed`,
     or when every inbound sets `directForeignByFormal` for it.
     Both are semantic facts,
      not text.

-    Support edges.
     For a surviving candidate,
      each inbound contributes an edge from the caller's contributing
     candidates to it.

-    Grounding.
     Keep a candidate only when it is seeded,
      or reachable in the support graph from a seeded one.

`recurse` from the counterexample has one inbound,
 itself,
 no seed,
 and no path to one,
so it drops.
`helper` reaches `entry`'s marked parameter through the inbound `entry` provides,
so it stays,
and its self-edge stops being the only thing holding it up.

The grounding pass is reachability over candidates that survived,
so it cannot add a candidate the current implementation would not have produced.
Every change it makes removes a suppression,
which is the direction that turns withheld offers into offers,
and each one has to be read.

### Measured, and falsified in both directions

The fixture is `readonly-recursive-ownership-invalid.ts`.

Before the change,
with the marker present:
one offer.
`markerlessPlain` was offered and `markerlessRecursion` was not,
though the two take the same parameter and read it the same way.
The only difference between them is the self-edge.

After the change,
with the marker present:
two offers.
`markerlessRecursion` joins its twin,
while `markerFedRecursion` and `markedRecursionEntry` stay suppressed.

With the marker removed from `markedRecursionEntry`:
four offers.
That is the falsification that matters,
because it establishes the two surviving suppressions are marker-driven rather than an artifact of
the helper being unexported.

One count separates all three states,
which is why the unit assertion is a count:
one before the fix,
two after,
four if grounding ever over-removes.

### Repo-wide: nothing changed at all

Swept at one thread against the fresh baseline taken at the commit before it.

-   1937 findings and 32 offers on both sides.
-   All 7201 finding locations identical,
     compared as sorted sets rather than as counts.
-   884.8s against 897.8s,
     which is 1.5 percent and inside the variation between runs.

So the shape does not occur in this workspace.
The defect is real and has a reproduction;
this repository simply contains no callable whose only in-scope inbound is its own recursive call
and whose parameter would otherwise be offered.

That is worth separating from the cache attempt recorded above,
which was also measured as no change and was reverted.
The difference is what each was for.
The cache was a performance change that bought no performance,
so no-change was its refutation.
This is a correctness change with a failing test before and a passing one after,
so no-change is only the statement that the workspace has no instances yet.
Reverting it would restore a known wrong answer with a known reproduction.

### And it unblocks the narrowing work

Task #34 was blocked on this on the grounds that fixing it would move the baseline any narrowing gate
had to be measured against.
It does not move the baseline.
The block is therefore lifted by measurement rather than by argument,
and the current numbers stand:
 1937 findings,
 32 offers,
 at one thread from the repository root.

Sol also recommends shadow mode for any future gate:
run every closure,
 assert the gate would have admitted every non-empty result,
and record the prospective skip ratio before trusting it.
That is the right shape for a change whose failure mode is silent.

## The fifth attempt, pre-registered before it is measured

Compute the foreign proof only when a verdict reads it.

### Why this one is not a gate

The four earlier attempts each invented a predicate that tried to guess whether the proof could
matter,
and each was refuted by a case the predicate got wrong.
This attempt has no predicate over the closure at all.
It moves the proof out of `CallableEffectSummary` and into a second `EffectSummaryIndex` method,
and the verifier's own control flow decides whether to ask.
`verifier.ts` reads the foreign answer in exactly four places,
so the question "can the answer change a report" has a mechanical answer per parameter:

-    `classification.kind === 'projected-readonly-capability'`.
-    `mutated && classification.kind === 'sound-readonly'`.
-    `(!mutated) && classification.kind === 'mutable'`.
-    `(!affected)` together with a parameter type the redundant-marker report would act on.

Everything else reaches no branch the answer can move:
an `opaque` parameter reports and returns before the read,
`opaque-capability` matches no branch,
and a `mutated` parameter whose type is already `mutable` has no offer left to suppress.
The fourth item is an equivalence rather than an approximation because
`reportRedundantForeignBorrowed` returns at its own first test unless the parameter's declared type
carries the marker;
testing that before asking is the report function's own control flow,
 hoisted.

### The one thing that has to be true, and why it is

Skipping a closure must not change any other callable's answer.
`completeForeignByCallable` is one map shared by every closure and written for every key each closure
returns,
which is exactly the shape attempt 2 died on,
so the question is whether a read ever returns an entry another root wrote.

It cannot,
and the reason is three lines rather than an argument about graph shape:
`initializeCandidates` calls `candidates.set(key, ...)` for every entry of `summaries` with no
condition,
`groundForeignCandidates` returns one entry per candidate,
and `completeForeignBorrowedGraph` seeds its root into `summaries` before enumerating anything.
So a root's own closure always returns an entry for its own key,
that write always lands last before the read,
and the `?? new Set()` fallback at the read site is dead code.

The comment above that fallback says the opposite:
"a callable the closure finds no inbound for is absent from the result rather than present and
empty".
That is false about this implementation and is corrected in the same commit.

The reshaping keeps only each root's own entry,
so the hazard stops being a fact that needs remembering and becomes one the type cannot express.
Sol's reason for insisting on that,
 which is the durable half of attempt 2's refutation:
`getSignatureUsage` enumerates references rather than call edges,
a lexical caller is inserted into `summaries` and queued before `isCallExpression` and edge
validation run,
and a caller that fails validation stays in the map with its edge replaced by a synthetic unknown
inbound.
Caller summaries therefore carry only the outbound edges the current root's walk discovered.
They are not the summaries a closure rooted at that caller would have built,
whatever the reachability sets say.

### What is predicted, so that a null result is readable

-    The sweep reports exactly 1937 findings and 32 offers,
     and all 7201 finding locations are identical as sorted sets.
-    The named failure mode is not the branch gating,
      which is equivalent by construction.
     It is the analysis budget.
     `createEffectAnalysisBudget` defaults to 120000 ms per project index and every closure records a
     phase against it,
     so proving less spends less,
     and external implementation inference can now complete where it previously exhausted the budget
     and returned unavailable,
      which would remove consumer opacity reports.
     The baseline sweep contains zero occurrences of `analysis-incomplete` or `budget exhausted`,
     so this is predicted not to fire here.
     It stays a real semantic change and is stated rather than hidden.
-    A second budget consequence has no measurement to hide behind:
     a proof demanded mid-verification can throw after earlier parameters already reported,
     where today every active-file proof runs during `includeActiveSource` before any report.
     The verifier therefore computes every parameter fact first,
     demands the proof once for the whole callable,
      and only then reports.

### The bar this attempt is measured against

Wall time alone cannot say whether the direction worked,
because the current 884.8s baseline includes the proof and no current measurement of the floor
exists.
The 616s against 966s in the task description was taken at an older commit and is not comparable.
So the floor is measured too:
a throwaway build whose proof returns empty without running a closure.

Recovered fraction is then `(baseline - deferred) / (baseline - floor)`.
Registered before the runs:
under half is a refutation of this direction as a cost fix,
leaving only the type-shape improvement,
and that outcome gets recorded rather than argued away.

### Measured

Three sweeps,
 one thread,
 from the repository root,
 same tree.

-    Baseline,
      the proof eager for every callable asked about:
      884.8s.
-    Deferred,
      this change:
      511.1s.
-    Floor,
      the proof disabled outright and answering empty without a closure:
      537.9s.

The deferred run is not slower than the floor.
It is 26.8s faster,
which no amount of doing strictly more work explains.

That invites a claim these numbers cannot support,
and the first version of this section made it:
that the residual sits "under the variation between runs,
 bounded at about 5 percent".
There is one observation per variant,
so no variation was measured,
and wall time is not monotonic in one component's work anyway.
The three numbers support only this:
the deferred build finished in 511.1s against 884.8s eager,
and a build that never proves foreign ownership finished in 537.9s,
so whole-sweep timing does not resolve the deferred build's residual.

### So the residual was measured directly instead

A throwaway build timed every `completeForeignBorrowedGraph` call and logged each duration.
One sweep,
 one thread,
 whole workspace:

-    67 closures ran,
      across 128 packages.
-    9 more were skipped by the marker pre-scan,
      costing 0.04ms in total.
-    5301.7ms in closures altogether,
      a median of 13.8ms and a longest of 1084.0ms.

Five point three seconds,
 which is 1.04 percent of a 511s sweep.
That is the residual,
 without inferring anything from a wall-time difference between builds.

The eager build ran one closure per callable the rule asked about,
which is what the 373.7s difference is made of.
Sixty-seven is what "only when a verdict reads it" reduces that to,
and seven of those sixty-seven are the offers the mechanism exists to withhold.

The instrumented sweep itself took 561s,
 above both clean runs,
which is the logging plus the same run-to-run noise that made the wall-time reading unsafe.
It is reported here rather than omitted,
 because it is the evidence that the noise is real.

The prediction held exactly.
Deferred against baseline:
1196 opaque-call reports,
666 opaque-method reports,
32 offers,
37 projected-readonly-capability reports,
6 stale contracts on both sides;
3902 warnings and 3299 errors across every rule on both sides;
and 7201 diagnostics identical as message-plus-location pairs,
 compared with multiplicity.

Pairs rather than locations,
because a sorted location set proves nothing about what was said at each location,
and category counts can agree while two messages swap places.
A raw diff is not available as a stronger check:
package order varies between runs even at one thread,
so the same 7201 diagnostics arrive in a different order and the diff is 60805 lines of noise.
A second deferred run produced the identical set,
so the result is reproducible rather than a single sample.
`package/module/ts-morph-shim/src/visit-node.ts:167:60`,
the offer attempt 2 invented,
stays absent.

### What the proof buys, now that it has a price

The floor run answers a question no previous attempt could ask,
because none of them had a build with the proof switched off.
Disabling it adds exactly seven offers and nothing else:

-    `package/electron-infra/core/src/wayland-state.ts:166:3`
-    `package/toml-edit/core/src/emit-value.ts:66:24`,
     `:333:3`,
     `:423:3`
-    `package/toml-edit/core/src/toml-get-node.ts:110:3`,
     `:141:3`
-    `package/pi-plugin/goal/src/pi-runtime-verifier-provider.ts:244:22`

Every one is an offer withheld because a marker proves the caller owns the parameter,
and the first is the flaky one from
`doc/troubleshooting/prefer-readonly-parameter-type-thread-nondeterminism.md` whose
order-dependence started this whole line of work.
So the complete backwards closure exists to withhold seven offers in this workspace,
and after this change it costs no measurable time to do it.

An eighth location appears in the floor list and is an artifact:
`effect-demand-index.ts:487:27` is the `false ?` the throwaway floor build introduced,
which the linter reported against itself.
Recorded so the count is not read as eight.

### What the sweep does not prove, and what does

The floor run measures something beyond wall time,
and it narrows what the equivalence is worth.
Disabling the proof outright leaves `projectedReadonlyCapability` at 37 and `redundantForeignBorrowed` at 0,
exactly as both other runs report them.
Only offers move,
 and only by seven.

So across 128 packages the foreign answer is true-and-read in one verdict:
the offer suppression.
The other three have no instance anywhere in this workspace.
That makes "all 7201 locations identical" strong evidence about one branch and no evidence at all
about the other three,
including the one whose condition this change actually rewrote,
`foreignBorrowed && (!affected)` becoming `redundantMarkerPossible && foreignBorrowed`.

The proof for those is readable rather than measurable,
so it is written out here instead of being re-derived from a sweep that cannot show it.
`factsNeedForeignProof` returns false for a parameter that is `opaque` without
`acceptedHostOpacity`,
which is exactly the set that reports and returns before any verdict reads the answer.
For the rest,
 each clause stands for one reading verdict:

-    `kind === 'projected-readonly-capability'`,
      unconditional,
     covers the unsupported arm of the `mutated` verdict and the whole of the `(!mutated)` one.
     It is unconditional because both of those fire on that classification regardless of `mutated`.
-    `mutated && (kind === 'sound-readonly')` covers the sound arm of the `mutated` verdict.
-    `(!mutated) && (kind === 'mutable')` covers the offer suppression.
-    `redundantMarkerPossible` covers the redundant-marker report,
     and carries `(!affected)` inside itself together with everything that report decides from the
     declared type.

The union is exact rather than a superset in either direction.
Nothing satisfies a clause without some verdict reading the answer,
so no proof is demanded that could not have mattered;
and no verdict reads the answer without its clause holding,
which is the direction that would emit a report the eager build withheld.

The fixture suite is what covers the three branches the workspace never exercises.
`reports inert ForeignBorrowed markers over deeply readonly types` is the redundant-marker report,
and `admits exact contracted host capability only after ordinary inference fails` is the
`acceptedHostOpacity` path that keeps `opaque` from being usable as a skip on its own.
Both pass.

### Why deferral reaches the floor rather than approaching it

Worth stating,
 because "no measurable cost" invites the suspicion that the proof stopped running.
It did not:
the deferred build still withholds all seven offers,
 which requires proving all seven.
The reason the rest costs nothing is the diagnostic mix.
Of 1937 findings,
 1862 are opacity reports,
 and an opaque parameter reports and returns before the
foreign answer is read.
Most of the remainder are parameters this repository already types `readonly`,
which is `sound-readonly` and matches no branch the answer can move.
The closure was running once per callable regardless,
including for `includeActiveSource`,
 which asks for every callable in the active file purely to seed
the graph and reads no summary at all,
and for `verifyOverloadConsistency`,
 which reads the summary but never its foreign field.
Those two were paying the largest cost the rule has for answers nothing consumed.
