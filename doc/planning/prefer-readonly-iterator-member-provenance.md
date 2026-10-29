# Iterator members in `prefer-readonly-parameter-types`

Measured 2026-08-06 against `main` at `a4cf83e08`,
 from the workspace sweep recorded in
`doc/decision/prefer-readonly-result-provenance.md`.

## The recorded note was stale, and the real shape is different

`doc/decision/prefer-readonly-result-provenance.md` said under "Remaining work" that iterator members remain
separately unproven and named `summaries.values` as a cause of the `effect-fixed-point-propagation.ts:37`
finding.
 That finding no longer exists:
 the channel authority now claims iterator creation and drainage
together,
 which is what `ITERATOR_MEMBER_NAMES` and its probe were added for.

What remains is not the channel.
 It is the result.
 An iterator member's channel being narrow says the call
reaches nothing surprising;
 it says nothing about what the returned iterator carries,
 and every finding
below is about the second question.

## What the residue actually is

Parsing full diagnostic messages rather than their first lines,
 which matters because cause lists wrap and
a first-line count understates them by a factor of eight:

- 1689 findings from this rule across the workspace.
- 62 name at least one collection iterator member among their causes.
- 16 name nothing else,
   so these are the ones that could clear outright.

Of those 16:

- 14 are `entries`.
- 2 are `values`.
- 0 are `keys`.

Every one of the 14 is the same idiom,
 iterating with an index:

```ts
for (const [index, item,] of items.entries()) {
```

Representative cases:
 `package/git-clone-size/cli/src/async-queue.ts:51`,
`package/module/toml-edit/src/fuzz/arb-combinators.ts:33`,
 `package/module/kv-store/src/consensus.ts:53`,
`package/pi-plugin/goal/src/pi-runtime-verifier-provider.ts:174`.

The `values` pair is `package/module/zip-writer/src/serialize.ts:218` and
`package/desktop-app/file-manager-electron/src/session.ts:75`.

## Why `values` is the easy one and `entries` is not

`PROVENANCE_BY_OWNER` in `effect-result-provenance-authority.ts` describes a member's result with one of three
relations,
 and each entry is enforced by an identity probe in `effect-result-provenance.unit.test.ts` that puts
a sentinel in a real receiver and compares identity against the result.

`values()` fits `RESULT_RELATION_RECEIVER_ELEMENTS` exactly as `filter` and `slice` do:
 the elements the
iterator yields are the receiver's own values,
 by identity.

The probe shape does not pass unchanged,
 which this document first claimed and which is wrong.
 The container
half asserts `Array.isArray(result)`,
 and an iterator is not an array,
 so every `values` entry failed it.
 The
probe now drains a non-array result through `Symbol.iterator` and compares membership in what it yields,
 which
is the same claim the relation makes:
 the object handed back is fresh,
 and advancing it yields what the
receiver holds.
 A result that is neither an array nor iterable drains to a sentinel rather than to an empty
list,
 so a wrong shape fails as a wrong shape instead of reading as a missing sentinel.

`entries()` does not fit any of the three.
 The elements it yields are freshly allocated pairs which *contain*
a receiver element rather than alias one,
 so a sentinel placed in the receiver is never identical to any
element of the result and the existing probe shape cannot pass for it.
 That is not an accident of how the
probe is written;
 it is the sound reason `entries` was left out.

Marking `entries` as `RESULT_RELATION_RECEIVER_ELEMENTS` anyway would be sound,
 because everything reachable
through a pair can reach the receiver and over-attribution never loses a report,
 but it would be a claim the
authority's own standard says must be probed,
 and the probe would be measuring something the symbol does not
say.
 A fourth relation naming the position inside the yielded element is the sound representation.

## Proposed order

1.    Done.
   `values` on `Array`,
   `ReadonlyArray`,
   `Map` and `ReadonlyMap`,
   with the drained probe and both
   pinned counts moved from 18 to 22.
   `Set` and `ReadonlySet` are deliberately absent:
   `receiverHolding`
   builds a `Map` or an array and nothing else,
   so a `Set` entry could not be probed,
   and no finding in the
   residue involves one.

   It clears nothing on its own,
   measured rather than predicted.
   The two `values` findings survive for a
   reason the relation does not address:
   in `zip-writer` the iterator is passed straight into
   `computeOffsets(entries.values(),)`,
   so the result escapes into a call,
   and the callee is itself opaque at
   that parameter because it pushes each entry into an array it returns.
   In `file-manager-electron` the
   `values` cause sits beside `getBoundingClientRect`,
   `querySelectorAll` and `scrollIntoView`,
   which dominate
   it.
   Workspace matched pair either side of the one file:
   3028 errors,
   3903 warnings,
   1689 rule findings and
   4 semantic failures on both runs,
   with the finding sets identical line for line.

   Kept anyway,
   on the same ground as increments 1 to 4 of the result-provenance arc:
   the entry is a true,
   probed fact the later steps need,
   and landing it separately keeps the step that does change verdicts
   small enough to measure.
1.    Done,
   and it was a defect rather than an increment.
   `callResultElementReceiver` read
   `receiverTypeArgumentIndex` on both sides of its type-identity comparison,
   so an element entry whose two
   positions differ could never verify.
   `Map.values` and `ReadonlyMap.values` were inert on arrival for exactly
   that reason:
   the receiver holds its values at position 1,
   `MapIterator` holds them at position 0,
   and
   the comparison read `undefined` against `V`.
   `MemberResultProvenance` is now a union giving the element
   relation its own `resultTypeArgumentIndex`.

   Measured:
   the same 3028 errors,
   3903 warnings,
   1689 rule findings and 4 semantic failures,
    with the
   finding sets identical line for line,
   so no verdict moved.
   What moved is the diagnosis:
   `serializeEntries` named `entries.values` before and names only `positioned.push` after,
   and no finding in
   the workspace names a `values` call any more.
   The remaining opacity there is the callee pushing each entry
   into an array it returns,
   which is a real escape and not an iterator question at all.

   The lesson generalises past this table:
   an authority entry can pass its runtime identity probe and still be
   unreachable through the static check that consumes it,
   so a probe alone does not prove an entry does
   anything.
1.    Done,
   and it does not clear the 14,
   which is the useful finding.
   `RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED`
   describes a result whose elements are tuples the member allocated,
   one recorded position of which is a
   receiver element,
   and `entries` is registered under it for `Array`,
   `ReadonlyArray`,
   `Map` and
   `ReadonlyMap`.
   The probe has three parts rather than the container probe's two:
   the result is not the
   receiver,
   no element of the result is the sentinel,
   and the recorded position inside an element is.
   The
   middle part is what separates this relation from the container one instead of restating it.

   The position proves the claim and does not bound it.
   Once a pair is known to carry a receiver element,
   everything reachable through that pair can reach the receiver,
   so flow covers the whole pair.
   That
   matters for `Map`,
   whose pairs hold a caller-owned key at position 0 and a caller-owned value at
   position 1:
   verifying either establishes the claim and flow then covers both.
   Reading the position as
   a bound would lose a write through the key of a `Map<Labelled, string>`.

   Measured:
   3028 errors,
   1689 rule findings and 4 semantic failures,
    finding sets identical line for line.

   The blocker is not the relation.
   `drawEach` still names `items.entries` with the relation verifying,
   because the iterator is consumed by `for...of`,
   and `for...of` is a spelling the escape walk does not
   attribute.
   An unattributed use reads as the result leaving the callable,
   and a container result that
   escapes is refused whatever relation it carries.
   The same blocker is already visible on `main` without
   any of this work:
   `iteratedContainerWriteEffect` and `spreadContainerWriteEffect` in
   `readonly-result-provenance-invalid.ts` report `rows.slice` for a container that never leaves,
   while
   `containerElementWriteEffect`,
   which consumes its container by element access,
   does not.

2.    Landed after a first pass was reverted on a misreading,
       which is worth recording because the
   misreading was mine and the evidence for it was available at the time.

   The change:
   `resultEscapesCallable` gained an `elementStepsAttributed` flag admitting the `for...of`
   expression position and a spread element,
   passed `true` from `effect-view-result-gate.ts` and
   `effect-collection-member-effect.ts` and `false` from `effect-call-analysis.ts`,
   where nothing walks
   the elements.
   That scoping is what keeps `formatUsageWarningStatus` intact,
   the control the attempt on
   issue #417 broke.

   Three count pins moved and I first read one of them,
   `readonly-tuple-exposure-invalid.ts` going from
   three reports to zero,
   as the unsoundness that fixture's test documents.
   It is not.
   That comment
   describes recursing into tuple positions via `checker.isTupleType`,
   which destroys the attribution;
   this change leaves it standing.
   Measured either side:
   `rewriteStoredPair` and `rewriteMutableStoredPair`
   both keep `referentMutated=[0]`,
   so no read-only offer can be made for a parameter they rewrite,
   and
   the fixture's own "offers nothing" assertion holds rather than passing vacuously.
   The guarantee moved
   to where it can be asserted:
   `effect-summaries.unit.test.ts` now pins those two attributions and the
   reader's empty one,
   so the silence in the diagnostics test rests on an assertion instead of on nothing.

   The other two moved for the reasons their own comments predicted.
   `readonly-member-channel-invalid.ts`
   went 7 to 5 by discharging `entries.values` and `entries.entries`,
   which that case says would clear
   "only under a relation describing a container whose elements are receiver state",
   and both loops only
   read,
   so nothing is left unattributed.
   `readonly-result-provenance-invalid.ts` went 12 to 11 by
   discharging `iteratedContainerWriteEffect`,
   whose write is recorded against `rows`.
   `spreadContainerWriteEffect` still reports,
   correctly:
   its spread builds a literal that is bound rather
   than passed,
    and a stored literal is where tracking genuinely ends.

   Workspace effect:
   3028 errors to 3023,
   1689 rule findings to 1685,
   4 semantic failures either side,
   and read-only offers unchanged at 33,
   which is the check that matters most since a false offer is
   the failure mode.
   Four findings cleared outright,
   including the `tasks.entries()` idiom in
   `package/git-clone-size/cli/src/async-queue.ts`.
   Several more kept reporting while their cause changed
   from the iterator to the real one:
   `blocks.entries` became `blocks.slice` and `blocks.with`,
   `calls.entries` became `output.content.push`,
   and `ctx.childrenByParent.get` became `ctx.shapes.set`.
   One parameter dropped off a shared finding,
   `strokes` in the doodle-widget export pages,
   which is
   `readonly StrokeData[]` and never written there.

   Superseded reading,
   kept because the error is instructive:

   The change was scoped by construction rather than global:
   `resultEscapesCallable` gained an
   `elementStepsAttributed` flag,
   admitting the `for...of` expression position and a spread element,
   passed `true` from `effect-view-result-gate.ts` and `effect-collection-member-effect.ts` and `false`
   from `effect-call-analysis.ts`.
   The scoping worked for what it was aimed at:
   `formatUsageWarningStatus` held,
   which is the control the earlier attempt on issue #417 broke.

   Three other pins did not hold,
   and one of them is not a count.
   `readonly-tuple-exposure-invalid.ts`
   went from three reports to zero,
   and that fixture's test already records what those three protect:
   with them discharged,
   `rewriteMutableStoredPair` is offered read-only while its body runs
   `pair[0] = 'rewritten'` on a tuple the array holds.
   A tuple is caller-owned state whatever its
   positions are,
   because the tuple itself is writable.

   So the widening trades a report for silence rather than for an attribution,
   which is precisely the
   failure `effect-collection-member-effect.ts` warns about in its own comment:
   the element step makes the
   container half true only while a write reaching through that step is actually attributed.
   For a write
   through an observer it is,
   pinned by `chainedContainerFoldWriteEffect`.
   For a write through a
   `for...of` binding over a `values()` iterator it is not,
   which is what the tuple fixture measures.

   The next attempt therefore cannot start at the escape walk.
   It has to start by making a write through
   an iterated binding attribute to the receiver's parameter,
   with `readonly-tuple-exposure-invalid.ts`
   as the control that must keep reporting throughout,
   and only then admit the position.
   Stated the
   other way round:
   admitting a position in the escape walk is sound exactly when something already
   walks that position,
   and this pair of steps was attempted in the wrong order.

3.    Attribute the element-step spellings in the escape walk:
   the `for...of` expression position and a
   spread element.
   This is the step that clears the 14,
   and it is the one to approach carefully.
   An
   earlier attempt at exactly this widening,
   recorded on issue #417,
   held every container control and still
   moved `formatUsageWarningStatus` in `package/pi-plugin/statusline/src/usage-warning.ts` from `opaque=[0]`
   to `[]`,
   which `effect-summaries.unit.test.ts` pins deliberately with the note that only deriving the
   `Object` readers should clear it.
   So the widening must reach the receiver resolution without discharging
   an argument-side obligation that arrives by propagation from a callee,
   and any attempt needs that
   function as a named control rather than as a surprise.
4.    Where the residue actually stands,
   measured after step 3.

   Iterator-only findings fell from 16 to 6,
   all `entries`,
   and every one of the six is the same
   shape:
   `[...collection.entries(),]` fed straight to `.map` or `.toSorted`.
   The obvious reading is
   that the spread is the blocker,
   since asking about a spread operand answers about a node no branch
   of `useEscapes` recognises.
   It is not.

   The reading was right and the first implementation of it was off by one node,
   which is worth
   recording because the measurement said "changed nothing" and that looked like evidence against
   the idea rather than against the code.
   Ascending from the node's parent found no spread and did
   nothing.
   `valueConsumer` already returns the spread element itself,
   so the ascent has to start at
   the node.

   Instrumenting settled it in one run,
   which is what the earlier round of reasoning should have been
   spent on.
   `receiverClaimAnswerable` was reached,
   the channel was narrow,
   the relation verified,
   and
   the refusal came from the escape test:
   the consumer was `...baseline.entries()`,
   whose parent is the
   array literal,
   so the literal branch asked "is this literal a call argument",
   and for a literal used
   as `.flatMap`'s receiver the answer is no.

   With the ascent starting at the node,
   `spreadCarrier` reaches the literal and its position as a
   receiver is already attributed.
   The redundant spread branch went with it,
   so one question is asked in
   one place.

   Measured:
   3023 errors to 3020,
   1685 rule findings to 1682,
   semantic failures and read-only offers
   unchanged at 4 and 33,
   three findings removed and none added.
   `spreadContainerWriteEffect` cleared
   too and its `referentMutated` still reads `[0]`,
   the same trade every other container case made,
   while `containerGrowthEffect` still reports at its row parameter because nothing attributes where
   that row ends up.

   The ascent covers array literals only,
   and that guard was checked rather than reasoned about.
   Probed
   over an overlay against an unresolvable sink:

   -    `unresolvedRowSink(...rows.values(),)`,
      a call-argument spread:
      `opaque=[0]`.
   -    `unresolvedSink({ ...rows.values(), },)`,
      an object-literal spread:
      `opaque=[0]`.
   -    `unresolvedSink([...rows.values(),],)`,
      an array-literal spread that itself escapes:
      `opaque=[0]`.
   -    `for (const row of [...rows.values(),])` reading only:
      `opaque=[]`.

   The third is the one that matters:
   ascending to the literal does not discharge it,
   because the
   literal's own position is an argument to a call nothing can read.
   The ascent moves the question rather
   than answering it,
   which is what keeps it from being a blanket discharge.
   `escapingLookupEffect` in
   `readonly-result-provenance-invalid.ts` is the standing fixture control for that shape and still reads
   `opaque=[0]`.

   **Iterator-only findings are now zero,
   from 16 when this arc started.
   ** Every remaining mention of an
   iterator member sits beside another cause that is doing the reporting,
   which is the sound state:
   the
   iterator question is answered and what is left is about something else.

5.    Done for `Map` and `ReadonlyMap`,
   after an experiment overturned a decision this document had
   already recorded against it.

   The recorded decision said `keys` must stay unregistered,
   on the grounds that
   `writeThroughMapKey(rows: Map<KeyRow, string>)` reads `referentMutated=[]` with `opaque=[0]`,
   so
   discharging the report would leave a false offer on a function that rewrites every key.
   The premise
   was measured and the conclusion did not follow.
   The attribution is empty *because* the relation is
   missing:
   with no relation the element walk cannot resolve `rows.keys()`,
   so the iterated binding
   carries no origins and the write lands on nothing.

   Registering it supplies both halves at once.
   Measured with the entry in place:
   `writeThroughMapKey`
   reads `referentMutated=[0]` and `opaque=[]`,
   and the attribution is what withholds the offer,
   while
   its reader control stays empty on both.
   That is the ordinary trade,
   not the failure mode.

   The lesson is narrower than "measure more" and worth stating exactly:
   an observation about the state
   before a change is not evidence about the state after it,
   and this document asserted the second from
   the first twice in one arc.

   The probe needed a receiver per member rather than per owner.
   A map holds caller-owned state on both
   sides of an entry,
   `get`,
   `values` and `entries` claim the value at position 1 and `keys` claims the
   key at position 0,
   so one receiver could only ever probe one of them and the other would pass or fail
   for reasons unrelated to its entry.
   `receiverHolding` now places the sentinel on the side the member
   names.

   Fixture pair `rewriteMapKey` and `readMapKey` in `readonly-tuple-exposure-invalid.ts`,
   with the
   attribution pinned in `effect-summaries.unit.test.ts` beside the tuple pair.
   Counts moved 26 to 28.

   Workspace:
   3020 errors,
   1682 rule findings,
   4 semantic failures and 33 offers,
   all unchanged with
   finding sets identical line for line.
   Nothing in this repository iterates a map's keys in a way that
   was reporting,
   so the entry buys correctness for the shape rather than a smaller number.

   The array half needs no entry at all.
   `sumIndexes(rows: readonly KeyRow[])` iterating `rows.keys()`
   is already fully silent,
   because the result carries indices and the exposure test answers before any
   relation is consulted.
   An entry there would be false as well as useless:
   the indices a key iterator
   yields are not the receiver's elements.

   Superseded proposal,
   kept because it named the right risk for the wrong member:
   its elements are indices,
   so the interesting claim is that it carries no
   receiver state at all,
   which is a different assertion from the other two and needs its own control
   proving a `Map` whose keys are objects is not swept in with an array's numeric ones.

Step 4 is the one to watch:
 `Map<object, V>.keys()` yields caller-owned objects,
 so a blanket "keys are
primitives" claim would be false for exactly the receiver type where it matters most.
