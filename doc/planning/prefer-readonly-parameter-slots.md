# Attributing effects to parameter slots instead of parameter indexes

`prefer-readonly-parameter-type` keys every effect fact by parameter index.
A single destructured object parameter gives every binding it introduces the same index,
so a callee taking `{ named, unnamed }` records anything about either against index zero.
`ST9` requires that shape for every function taking more than one input,
which makes the collapse the ordinary case here.
`doc/decision/prefer-readonly-contract-name-narrowing.md` records the three defects it caused
and the precision it costs.

This plan replaces the index with a slot,
and records the design before any of it is built so the staging can be checked against it.

## What a slot is

A slot is a parameter index paired with at most one property name reached through it.
Slots are allocated per callable declaration,
as a pure function of the declaration alone:

- slots below `parameterCount` are the whole parameters,
   numbered exactly as parameter indexes are today,
   so every existing fact keeps its meaning;
- slots from `parameterCount` upward are one per statically canonical top-level property key an
   object pattern in a parameter reads.

The unit is the property key,
 not the binding.
`{ a: x, a: y }` reads one property twice and gets one slot that both symbols register against.
`{ a: { b } }` gives property `a` a slot and registers `b` against it,
which is sound because a write through `b` is a write through `a`,
and more precise than widening to the parameter.
Defining the unit as the binding instead would leave `a` with no slot in that second case,
which is the direction that loses writes.

Depth is capped at one property,
 so nothing below the top level gets its own slot.

Purity of the allocator is what lets a caller and a callee agree on a numbering without the
caller re-analyzing the callee's body.
`addOwnedCallEdge` holds the callee declaration,
so it computes the same table the callee's own summary computed.
Deriving slots from the body instead would break that agreement,
which is why a write through a plain identifier parameter,
 as in `function f(o) { o.p.x = 1 }`,
still records the whole parameter.
That is today's behaviour,
 so it is not a regression.

## Why numbers rather than string keys

A string slot key on the edge would let the caller name a callee property without knowing the
callee's numbering.
The hazard it guards against is a persisted caller edge outliving a change to the callee that
renumbers slots,
which would map an effect onto the wrong argument.

That hazard is already closed,
 and by content rather than by naming.
`PersistentEffectCacheEnvelope` in `effect-cache-envelope.ts` snapshots a content digest for
every non-declaration workspace file in the entry's transitive module-dependency closure,
and an entry revalidates only while every one of those digests still matches.
A callee edit changes the callee file's digest,
the callee is in the caller's closure,
so the caller's cached edges are discarded before they can be misread.
An entry whose module references did not resolve snapshots the whole indexed scope instead,
which fails in the same safe direction.

Numbers therefore keep the serialization,
 the hashing and the propagation loop unchanged in
shape,
and buy the safety a string key would have been paying for.
Two things still have to move with them:
`schema` is bumped because the payload gains a field,
and `isParameterIndexes` in `effect-summary-cache-validation.ts` bounds every stored index by
`parameterCount`,
which a property slot exceeds by construction.
Left alone it would reject valid payloads rather than accept invalid ones,
so it fails safe,
 but it makes the cache useless.

## The brand is the safety mechanism

Under this change `mutated.has(0)` stops meaning "parameter zero is mutated",
because a write to `named.label` records a property slot instead.
Every surviving `.has(parameterIndex)` becomes a dropped effect,
and a dropped effect is what offers `readonly` for written state.
TypeScript flags none of it,
since both sides stay `number`.

So `EffectSlot` and `ParameterIndex` become distinct brands before anything else moves,
following the repo idiom in `package/fs-id/core/src/types.ts`.
`ReadonlySet<EffectSlot>.has` then rejects a `ParameterIndex`,
and every conversion becomes a compile error that has to be answered rather than a grep that
has to be complete.
Fields whose meaning changes are renamed in the same pass,
so a consumer that never converts still fails to compile.

## The five formal-indexed arrays

`doc/planning/prefer-readonly-call-edge-shapes.md` records that `arguments`,
 `foreignArguments`,
 `directForeignArguments`,
 `callbackKeys` and `callbackFileNames` are read by formal index and must stay aligned.
Slots split them,
 so each is decided here:

- `arguments` becomes slot-indexed,
   holding caller slots.
   `propagateCalleeIndexes` reads it with a callee slot and needs no change.
- `callbackKeys` and `callbackFileNames` become slot-indexed,
   because `effect-invoked-capability.ts` reads them with an index drawn from
   `calleeSummary.invoked`,
   which now holds slots.
   Leaving them formal-indexed would misread a property slot as a formal.
- `foreignArguments` stays parameter-indexed and holds caller parameter indexes,
   because `foreign-borrowed-propagation.ts` compares its values against caller candidates that
   are parameter indexes,
   and `ForeignBorrowed` is a marker on a whole parameter rather than on a property.
   The conversion happens where the edge is built,
   not where it is read.
- `directForeignArguments` stays parameter-indexed,
   for the same reason.

## The caller-side fallback is what makes this sound

Propagation reads `edge.arguments[calleeSlot]` where `calleeSlot` comes from the callee's
effect set.
When the callee records a write on property slot `0.named`,
nothing ever consults the whole-parameter slot.
So a caller that cannot decompose its actual and fills only the whole slot turns a precise
callee fact into an empty mapping,
and the write disappears.

The rule is therefore the opposite of a widening:

- **Inside the callee**,
   an unsupported binding or effect attributes to the whole-parameter
   slot.
   This is a widening and it withholds offers.
- **At the caller**,
   an actual that cannot be decomposed contributes its origins to the whole
   slot **and to every property slot of that formal**.
   This is a broadcast,
   and skipping it
   loses writes.

The two are not symmetric and stating only the first is unsound.
The broadcast covers a non-literal actual,
 a call spread,
 a computed key,
 an object spread,
 a
conditional,
 an alias whose aggregate structure is unavailable,
 and every position past a
call spread.

A child effect is never added to the whole slot inside a summary.
That would erase the precision the whole change exists to recover,
because the whole slot maps to every origin the actual packages.

## Callback and observer effects must project before they are read

Two consumers index a summary with something that is not a slot in that summary:

- `effect-callback-relation.ts` tests `callbackSummary.mutated.has(relation.callbackArgumentIndex)`,
   where the index is an actual argument position of the callback invocation.
- `effect-element-application.ts` tests an observer summary with
   `ElementApplication.callbackParameterIndexes`.

A callback or observer that destructures its parameter records a property slot,
and both tests then miss it.
Both project the callback summary to parameters before applying the existing relation.
That keeps current soundness without claiming property precision through callbacks,
which needs `CallbackRelation` to describe a slot-to-slot mapping and is out of scope here.

## The external path needs projection, not slots

`applyExternalEffect` indexes a positional argument map with parameter indexes taken from an
external callable's public summary.
If that summary starts carrying slots,
 a property slot indexes past the end of the map and the
effect is dropped,
 which is the unsound direction.
External summaries are therefore projected to parameters before they are applied.

## Staging

Each stage is committed separately and the workspace sweep is compared by offer identity.

1.  Allocator,
     brands and threading,
     with the public summary projecting slots back to
    parameters.
     The sweep must be identical:
     same findings,
     same offers.
     A difference here
    means a `.has` site was missed,
     which is the cheapest possible signal.
2.  Caller-side property matching in `addOwnedCallEdge`.
     Numbers move.
     Every recovered offer
    is verified individually.
3.  Diagnostic naming narrowed to the affected bindings,
     so a finding stops naming primitives
    that cannot carry state.

## Every sweep comparison must run sequentially and single-threaded

Two separate reasons,
 discovered separately.

The parallel package fanout in the root `lint:oxlint` task runs one oxlint process per package
up to `availableParallelism()`,
each holding its own TypeScript program,
and sixteen of those exhausted host memory and took the machine down.
`fanout_packages` in `mise.toml` offers no way to bound it,
so a measurement sweep runs the package tasks one at a time instead.
That turned out to be faster in wall-clock terms as well as survivable,
because the parallel version was thrashing.

The thread count inside each process is the second reason,
 recorded next.

## Every sweep comparison must run single-threaded

Under oxlint's default thread count this rule does not produce the same findings twice.
Seventeen repeats of one package lint at one commit split seven to ten on whether a single
correct offer appears,
and ten single-threaded repeats of the same lint agreed unanimously.
`doc/troubleshooting/prefer-readonly-parameter-type-thread-nondeterminism.md` records the
measurement,
 rules out the persistent cache and the analysis budget,
 and names the workaround:
`OXLINT_THREADS=1`.

This is why the stage-one checkpoint has to be read carefully.
Baseline and stage one both reported 1837 findings and 27 offers with identical offer sets,
but both were multi-threaded,
so that is two samples from a distribution rather than proof the refactor changed nothing.
A third multi-threaded sweep,
 at the stage-one correction commit,
 reported 1838 and 28,
and bisecting that difference is what surfaced the nondeterminism.

Every comparison from here runs with `OXLINT_THREADS=1`,
and the stage-one checkpoint is re-established that way before stage 2 is measured.

## What stage 2 must do, predicted before it is measured

Narrowing only ever removes caller origins from an `originsByCalleeSlot` entry,
propagation is a union-only least fixed point,
and the whole-parameter slot keeps its full unnarrowed origins,
so a caller's effect sets can only shrink.
That makes stage 2 falsifiable rather than merely observable.
Checked in this order,
 because each one is cheaper and sharper than the next:

- **Whole-slot edge rows are unchanged.**
   They are built from `originsByFormal` exactly as
   before,
   so any movement here is a bug in the plumbing rather than in the narrowing.
- **Every property-slot edge row is a subset of its old row.**
- **Every callable's effect sets are a subset of their old sets.**
- **No offer is withdrawn.**
   A withdrawn offer would mean narrowing created an effect,
   which
   the direction of the change forbids.
   New offers are the point of the change.

Two things that are deliberately not on that list.

Foreign-borrowed findings are compared as an observation rather than asserted unchanged.
`foreignOriginsByFormal` does derive from the unnarrowed formal origins,
but that only shows the edge field is untouched;
whether the diagnostic that consumes it also reads propagated effects is unproven,
and an invariant nobody has proven is a bug detector that fires on the wrong thing.

Total finding count is not an invariant either.
A parameter that loses `opaque` stops producing `opaqueEffectReport` and may produce
`shouldBeReadonly` in its place,
and one that loses `affected` can newly produce `staleMutatesTag`.
So the offer set is compared by identity,
while the finding delta has to be explained by those category shifts rather than by a number.

One legitimate way these predictions fail:
stage 2 reaches an origin the old whole-argument collector never found,
which makes an effect set grow for a reason that is a fix rather than a regression.
That has to be recognized by naming the newly found origin,
never by relaxing the prediction after the fact.

Recording all of this before the stage-2 sweep is the point:
a prediction written after seeing the result is not evidence of anything.

## Stage 2, measured

The acceptance criterion is met.
`narrowingPrecisionCostEffect` reports `mutated` projected to parameters as the first parameter
alone,
 where it read both,
and the rule now offers `readonly` for its `second` parameter,
which its callee only reads.
No offer was withdrawn anywhere in the plugin's own test suite.

Fourteen caller shapes in `readonly-slot-narrowing-invalid.ts` pin both directions,
and every one matched the prediction written before it was built:

- Narrowed to the written parameter alone:
   a plain key,
   shorthand keys,
   quoted keys,
   a callee
   binding the key under another name,
  `1e0` against a callee reading `1`,
   a row packaged one
   literal deeper,
   a row returned by a method the callee calls,
   and a spread placed before the
   exact key that shadows it.
- Refused:
   a spread placed after the key it can overwrite,
   a computed key,
   an accessor pair
   whose setter comes last,
   a getter reaching its row through `this`,
   and a rest formal whose
   property key names an array index.
- `prototypeKeyBroadcast` is the one that decides whether the `__proto__` rule was needed.
   Its
   literal defines no own `named`,
   its callee writes through the prototype's,
   and it reports the
   prototype-served row alone.
   Reading `__proto__` as an ordinary key reports nothing there,
   which is an offer for a row something mutates.

## The workspace sweep, and the five offers it recovered

Sequential and single-threaded on both sides,
 128 package tasks each.
The pre-stage-two side reported 1799 findings with 28 offers;
the stage-two side 1741 with 33.

The sweep's own reproducibility was tested before any of it was believed,
because nothing had established that fixing the thread count inside each process was enough once
128 processes ran in sequence against a shared on-disk cache.
Two consecutive runs of the shipped state reported 1741 findings and 33 offers with identical
finding and offer sets.
Comparing one run against one run was what the voided bisect did,
and it is not a thing to repeat.

Refusing to decompose a literal that sets a prototype cost nothing measurable here:
the sweep is identical before and after that fix,
finding for finding,
so no argument literal in this workspace sets a prototype.
The precision it gives up is real but unexercised.

**No offer was withdrawn.**
That was the prediction that mattered,
and it is the one a narrowing cannot survive breaking.
All five added offers are in `package/toml-edit/core/src/emit-value.ts`,
and each is verified rather than assumed:

- That file contains no mutation at all.
   Measured:
   no member assignment and no mutating array or
   map method call anywhere in its 476 lines.
- The opacity that used to reach them is real and still reported.
  `assembleArrayParts` calls
   `parts.join`,
   an unresolved receiver call,
   and measures `opaque=[0]` after the change exactly
   as before it.
- What moved is where that opacity lands.
  `emitArray` calls
   `assembleArrayParts({ parts, options, depth, },)`,
   and the callee's opaque `parts` slot now
   receives only the origins filling the key `parts`,
   which is a local.
  `emitArray` measures
   `opaque=[]` where the broadcast had credited its `node`,
  `options` and `depth` alike.

So the effect was localized,
 not lost,
which is the distinction the whole model exists to draw.

Findings churn is larger and less interpretable:
 73 locations lost a finding and 15 gained one.
The gained ones are not new reports.
`client.ts:81:1` appears while `client.ts:83:12` disappears,
 and `exa-client.ts:108:1` while
`110:12` disappears,
which is one report re-anchoring from a parameter to its declaration as the set of parameters it
names changes.
Reading the finding count as a quality signal would have been wrong in both directions.

One methodological note worth keeping.
The first comparator classified over half of all findings as unlocated,
because oxlint prints the `,-[path]` line after every continuation line of the message and a
report's remediation list runs to more than a dozen of them.
Offers were unaffected,
 their messages being one line,
which is the only reason the withdrawal check was sound before the extractor was fixed.
A comparator that silently drops half its input is worse than no comparator,
so it now reports its own unlocated count and both sides read zero.

## What stage 3 must do, predicted before it is measured

Stage 3 changes message text and nothing else.
Only `verifier.ts` reads `opaqueBindingsByParameter`,
 only to build a subject,
and `affectedBindingNames` mutates nothing.
So the invariant is far sharper than stage 2's:

- **Offers are identical**,
   not merely unwithdrawn.
- **The set of finding locations is identical.**
- Only the message text of a report may differ.

Any offer or location that moves is a bug rather than a tradeoff.

One failure mode is specific enough to name in advance.
`usageSubject` has an empty-list branch emitting "The function input at this location is",
a subject that names nothing.
`parameterNames` falls back to every binding when narrowing empties the list,
but `inputMethodUsageSubject` applies its receiver filter after that fallback,
so a narrowed list can survive `parameterNames` and then filter to empty.
That is reachable rather than theoretical:
`everyBoundaryIsInputMethod` answers yes when every boundary is a method on some binding of the
parameter,
while `affectedBindingNames` includes a binding only when its own slot is opaque,
so opacity widened to the whole-parameter slot while the boundary names a property binding leaves
the receiver out of the narrowed set.
The check is a count of that phrase across both sweeps,
 and any increase is the regression.
The remedy is to apply the receiver filter first and fall back to the unnarrowed receiver list,
never to widen what counts as affected.

### Stage 3, measured

Every one of those held.
Offers are identical to the pre-stage-three sweep,
 all thirty-three of them.
Not one finding location moved,
 lost or gained.
Three hundred findings changed their message text and nothing else did.
The empty subject appears zero times on both sides,
so the named failure mode did not fire across 128 packages.

It is still reachable by construction,
 so the remedy landed anyway.
A path that can emit a subject naming nothing is a defect whether or not this workspace happens
to reach it,
and the sweep says only that no code here has an input whose whole-parameter slot took opacity
while a boundary named one of its properties.

## Rules decided in advance

- **Renamed binding** `{ a: b }`.
   The slot key is the property name `a`,
   since that is what a
   caller writes;
   the symbol registered is `b`'s.
   Diagnostics keep the authored name `b`.
- **Duplicate keys** `{ a: x, a: y }`.
   One slot for property `a`,
   both symbols registered
   against it.
- **Equivalent keys** `{ 1: x }` against `{ "1": y }`.
   Keys are canonicalized to the property
   name the checker resolves,
   so equivalent spellings agree.
   The key is held in a map owned by
   the parameter rather than concatenated into one string,
   so no delimiter needs escaping.
- **Rest property** `{ a, ...rest }`.
  `rest` takes the whole-parameter slot.
   It names a
   complement set rather than a property,
   so no caller property key ever matches it.
- **Array pattern** `[first, second]`,
   elisions and array rest.
   Whole-parameter slot.
   Positional
   element keys are not modelled.
- **Default inside a pattern** `{ a = fallback }`.
   The binding takes property `a`'s slot,
   and
   the initializer's own origins are unioned into it.
  `BindingElement.initializer` is walked
   neither for origins nor for effects today,
   which is a gap this has to close rather than
   inherit.
- **Computed property name** `{ [key]: value }`.
   No slot.
   The binding takes the whole-parameter
   slot,
   and a caller's unknown computed key contributes to every property slot.
- **Caller literal,
   property order**.
   Resolved by walking the literal's properties in reverse
   for each target key:
   an exact match contributes its value and stops;
   a known different key
   is ignored;
   an unknown computed key contributes and continues;
   a spread contributes every
   origin of its source and continues.
   So `{ ...other, named: first }` attributes `named` to
   `first` alone,
   while `{ named: first, ...other }` attributes it to both.
- **Both sides of a key comparison canonicalize the same way**.
   The caller's literal keys go
   through `canonicalPropertyKey` in `effect-slot-identity.ts`,
   the same function the callee's
   pattern keys went through.
   That is what makes `callee({ 'named': x })` reach a callee written
   `{ named }`.
   Comparing `getText()` instead would make quoting and numeric spelling decide
   whether a write is seen,
   and earlier call-edge work was already bitten by exactly that.
   Agreeing is necessary and not sufficient:
   a shared key that is wrong makes a matching key
   look definitely different,
   which is the direction that drops an origin.
   Measured with
   `createScanner`,
   this AST's numeric token values are already the runtime property keys:
   `1e0` and `1.0` both read `1`,
  `0x10` reads `16`,
  `1_000` reads `1000`.
   String and template
   literals expose their cooked text,
   so escapes agree too.
- **Plain `__proto__: value`**.
   Not a key.
   That exact spelling sets the prototype rather than
   defining an own property,
   so every key the callee reads and the literal does not define is
   served through it.
   Treating it as a key leaves those keys with no origins,
   and
   `callee({ __proto__: source })` writing through `source.named` is then attributed to
   nothing.
   Verified at runtime:
   the write lands,
   and `Object.hasOwn` reports no own
   `__proto__`.
   The computed,
   shorthand and method spellings of the same name all create
   ordinary own properties and keep their key.
- **Any accessor in the literal**.
   No narrowing for that actual at all.
   Two shapes defeat
   anything weaker.
  `{ hidden: owned, get named() { return this.hidden; } }` reaches `owned`
   through `this`,
   which no scan of the accessor body finds,
   and
   `{ get named() { return owned; }, set named(value) {} }` puts the origin-free setter last,
   so a reverse walk that stops at the first exact match stops on the setter.
   A method is
   different and keeps its key:
   it defines a fresh function object and shadows what came
   before.
- **Property slots owned by a rest parameter**.
   Always broadcast.
   A rest formal is a
   synthesized array,
   so in `function callee(...{ 0: box })` the key `0` names the first
   actual rather than a property of it,
   and resolving `0` against a caller's
   `{ named: owned }` finds nothing and loses the write.
- **Wrappers that may be unwrapped to find the literal**.
   Parentheses and type-only wrappers
   only.
   Not an assignment,
   a sequence,
   an `await`,
   or a call:
   in
   `callee(argument = {}, Object.assign(argument, { named: owned },),)` the first actual is
   mutated by the evaluation of the second before the call happens.
   Not a spread element
   either,
   since `callee(...values)` fills formals from the elements rather than from the
   spread expression itself.
   No fixture asserts this one.
  `parameterIndexes` walks neither an assignment nor a call,
   so
   that actual carries no origins whether the rule unwraps it or not,
   and both the right rule
   and the wrong one report the same empty set against a true answer of the first parameter.
   Asserting it green would pin task #28's gap as the specification.
   It becomes testable when
   #28 lands and not before.
- **Non-literal actual**.
   Broadcast:
   every property slot of the formal receives the actual's
   origins,
   which is what happens today.
- **Overloads**.
  `overload-consistency.ts` compares two different declarations with two
   different slot tables,
   so both sides project to parameters before comparing.
- **Propagation bound**.
  `effect-fixed-point-propagation.ts` counts
   `parameterCount * EFFECT_DIMENSION_COUNT`.
   It becomes slot-count-based,
   otherwise
   `EffectPropagationError` throws on a program that is converging normally.

## Two structural requirements the current code does not meet

`addOwnedCallEdge` receives `allArgumentIndexes`,
 which is already flattened:
 each actual has
been reduced to a list of caller origins with no record of which property contributed which.
Property matching cannot be reconstructed from it,
so the edge has to be built from structured argument provenance,
which means the builder also needs `bindingOriginBySymbolId`.

`registerBindingOrigin` is used both to seed parameters and to register local aliases,
and it recurses through any pattern it is handed.
Only the parameter seeding may allocate property slots.
An alias destructured from an unrelated local must not invent one,
so the two uses become separate operations.

## What this fixes, beyond precision

The index collapse was recorded as a precision cost.
Three live unsoundnesses were measured on a throwaway fixture while designing this,
each offering `readonly` for a parameter something writes:

- A callback and its argument packaged into one destructured parameter.
   `probeInvoke({ callback, value })` invoking `callback(value)` reads `referentMutated=[]`
   while the callback writes through what it receives.
   The same call written positionally
   reads `[0]`,
   which isolates the cause to the destructured parameter.
   `propagateCallbackRelations` looks up `edge.callbackKeys[relation.callbackParameterIndex]`,
   finds the object literal at that position is not a callable,
   and `continue`s.
   Tracked as
   task #27.
- A callee reached through a local holding an object literal.
   `const options = { named: first }; callee(options)` reads `referentMutated=[]` where the
   direct literal reads `[0]`,
   because `provenanceSuccessors` has no case for an aggregate
   literal.
   Independent of slots.
   Tracked as task #28.
- The precision loss itself,
   which withholds offers rather than making wrong ones and is the
   one the acceptance criterion measures.

## What landed, and what the sweep pair actually brackets

Commits,
 in order:
`4a39a445e` and `f69c83303` planned it;
`4883d16df` and `7ab855cb8` built stage one;
`54a1d06d4` corrected it;
`6a0122661` and `263b0d099` recorded the nondeterminism and revised the predictions;
`0f787856f` and `7776ba61a` built and measured stage two;
`5bfedf489` fixed the prototype unsoundness in it;
`13b60c154`,
`63c7d8c6d` and `a238f02ea` recorded the measurements;
`d33ba1870` built stage three;
`03c18be6b` pre-registered its invariant;
`49896479f` closed the empty-subject path.

The shipped state measures 1741 findings and 33 offers,
identical across the stage-three sweep and the confirmation sweep after the last fix,
which is what an unreached path changing nothing looks like.

### Stage one is settled

The sweep pair above brackets stages two and three only,
because its reference side already had slots allocated and threaded.
Stage one was measured separately and afterwards,
by checking the pre-slot plugin sources into the current tree,
 sweeping,
 then doing the same for
the post-slot ones:

Both sides report 1796 findings and 28 offers,
with identical finding sets and identical offer sets,
zero locations lost and zero gained.
Keying effects by slot rather than by parameter index changed nothing at workspace scale,
which is what stage one claimed and what nothing had previously checked with a measurement worth
trusting.

The first attempt at this comparison reused the older `263b0d099` sweep log rather than
re-sweeping,
and reported nine moved report locations.
Every one was `package/config/tofu`,
 which concurrent work had edited twice in between.
Line numbers shifted by a constant while message text stayed identical,
 which is the signature,
and `git log --since` over the named files confirmed it.
Both sides of a sweep pair have to be swept against the same tree.

## Acceptance

`narrowingPrecisionCostEffect` reads `mutated` projected to parameters as the first parameter
alone,
 where it reads both today.
Adversarial fixtures cover every shape the rules above widen or broadcast,
and each must retain the write even where that means both caller parameters stay affected.
Fixture assertions state both levels,
parameter-level for the invariants that must not move and slot-level for the new precision,
so a regression in the projection cannot hide behind a passing parameter-level assertion.
