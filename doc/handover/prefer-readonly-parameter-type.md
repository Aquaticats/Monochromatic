# Handover: making `prefer-readonly-parameter-type` bulletproof

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/rss` is now `package/rss/app`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/kv-store` is now `package/kv-store/core`.

Live document.
 Updated as work lands,
 not at the end.

Scope:
 `package/oxlint-plugin/prefer-readonly-parameter-type`.
Goal in force:
 no false offers.
A false offer is the rule offering `readonly` for a parameter something writes through.
Withholding an offer is always safe and costs only precision.

## The bar that decides everything here

A defect is not a defect until it is falsified.
Falsification means all four of:

- the rule produces the annotation
- every suggestion in the file is applied,
   not just the one under test
- the result type-checks clean
- a driver observes the caller's state change
- **the escape comes from something the annotated callable does,
   not from something the caller
  hands it**

That last clause was implicit until it caught me out.
 A driver that supplies the escaping
 behaviour proves nothing about the callable it is driving:
 handing a value to a callback the
 caller wrote is handing it back to the caller,
 who already had it,
 and the callback relation
 defers that decision to the caller on purpose.
 The same invalid driver would equally "falsify"
 `returnRowDirectly`,
 which is kept offered as the policy control.

The third point needs a control in the same file,
 usually a callable carrying `@ts-expect-error` over a direct write,
 because `ReadonlyDeep` resolving to an error type makes every other function in the file
 type-check for the wrong reason.
Every falsification in this work compiles for one reason:
 TypeScript ignores `readonly` property modifiers in assignability,
 so a callee declaring `Row` launders a `ReadonlyDeep<Row>` straight back to mutable.

Summaries and boundary checks are not the bar.
A clean report and a disabled rule look identical,
 which is why a must-still-be-offered control belongs in every boundary check.
The rule is disabled for its own package by `readonlyEffectSelfHostingOverride`
 (`package/config/oxlint/src/overrides.ts`).

## How to work here

Build both artifacts,
 in order,
 or the measurement is of something else:

```bash
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:build
mise run //package/config/oxlint:build
```

The second produces the sidecar oxlint actually loads.
A sweep silently uses whichever sidecar is on disk.

Probe summaries with `probe.mjs` from the session scratch directory,
 copied into the plugin package to run,
 since TypeScript resolution fails from elsewhere:

```bash
PROBE=$(mktemp --tmpdir="$PWD" probe-XXXX.mjs)
cp <scratch>/probe.mjs "$PROBE"
node "$PROBE" <fixture-file> <exported-name>...
```

Falsify inside `package/module/kv-store`,
 which is the nearest package that both depends on `type-fest` and type-checks,
 then delete the files.
The fixture package has `type-fest` but its `lint:types` is deliberately absent.

Commit before mutation-checking.
`git checkout --` restores to HEAD and will delete an uncommitted fix rather than the mutant.

## Sweep discipline

While a sweep runs:
 no rebuilds,
 and no edits to any file it lints.
Docs,
 reads and probes of the built artifact are fine.

Record beside every capture:
 the commit,
 the plugin digest,
 the sidecar digest,
 **and the panic count**.
A capture without the first three cannot be told from one taken against a stale sidecar.

The panic count is the newer of the four.
 The upstream tuple panic in
 `doc/troubleshooting/typescript-go-tuple-type-panic.md` leaves each file it hits without readonly analysis
 for the run,
 so **every count in a capture that panicked is a lower bound**,
 and two captures whose panic
 counts differ are not comparable.

Both signals are greppable,
 and the second one is not phrased the way you would guess:

```text
panic: interface conversion            5 at both 851b0fd3f and 94af5da15
which the effect index omitted         2, both naming package/webapp-productivity/rss/src/index.ts
```

A first version of this section said no per-file warning was locatable.
 That was a search failure,
 not an
 absence:
 the warning reads `skipping <file>:<range>, which the effect index omitted`,
 and
 `effect-demand-index.ts` logs the **cause** beside it as `omitting <file> from the effect index: <error>`.

Reading that cause settles what a second version left open.
 All three omissions name the panic:

```text
package/webapp-productivity/rss/src/index.ts                      2 callables
node_modules/.../@optique/core@1.2.0/.../dist/facade.js           1 callable
```

So the panic is the cause,
 established rather than assumed,
 and five panics correspond to three omitted
 callables rather than to five files.

**The third line is new,
 and it exists because the external channel now runs.**
 The panic strikes inside a
 dependency's shipped implementation,
 not only in analysed source.
 That stays sound,
 since an omitted
 external callable answers `NO_EFFECT_SUMMARY`,
 the effect resolver reports unavailable,
 and the consumer's
 call falls to the unresolved boundary,
 which withholds.
 It does mean the upstream defect has a second
 surface now,
 and a capture should be read for omissions under `node_modules` as well as under `package`.

**Opacity is a merged channel,
 so no assertion on it can name which path wrote it.**
 The unresolved
 boundary and a resolved external retention both add to `opaqueParameterIndexes`,
 and nothing in the set
 records which.
 An assertion that a capture is charged is therefore satisfied by the resolution failing,
 which is the opposite of what such a test usually means to claim.

Measured,
 on the resolution fix for a package declaring no runtime entry:
 a first test handed that package
 a capturing closure and asserted the capture was charged.
 It passed,
 and **both** mutants survived it.
 The
 replacement asserts a **proven mutation** and its absence of opacity,
 which only a resolved
 implementation can produce,
 and both mutants then died.

So when a fix makes something resolve,
 assert a fact that only resolution writes:
 a proven mutation,
 or
 provenance naming the package.
 Not opacity,
 and not a charged capture.

Two ways of misreading a capture,
 both paid for:

-   **Never attribute a finding to a file by line proximity.**
     The interleaved panic traces put unrelated
     lines inside any context window,
     and a `nano-spawn` finding read that way was attributed to the wrong
     package entirely.
     Every message names its own subject,
     so grep for a parameter name from the message.
-   **Cold means the repository root.**
     The summary cache root resolves through `dependencyRoot`,
     which for a
     workspace package is the repo root,
     so clearing `package/<name>/node_modules/.cache` clears nothing that
     matters.
     A warm reading looks exactly like a measurement.

Registered criterion,
 in its reusable form:

- offers must not rise,
   which is the only soundness statement
- **read which way a mutation moves the offer count,
   not merely that it moved.**
   A withholding fix kills its
  mutant by raising the count;
   a precision recovery kills by lowering it.
   Both are correct kills in opposite
  directions,
   so "the number changed" is satisfied by either and confirms neither
- no category other than argument opacity may move
- offers falling is expected,
   and each fall is sampled to its cause
- argument opacity rising is expected,
   and is **not** required to accompany an offer loss

That last clause was learned the hard way.
 A capture landing on a callable-typed parameter has
 no offer to lose,
 because that classification is an opaque capability.

A delta that disagrees with expectation is not evidence about the change until the change has
 been removed and the capture repeated.
A delta that survives removing the thing under test was never about it.

Wall clock is not an instrument for *finding* anything.
Identical findings have come back at 169s,
 543s,
 8m16s and 3m00s,
 and the transitive-capture walk came in faster while doing strictly more work.

It is still worth **recording**,
 for a different question.
 Cache state and machine load dominate short
 runs,
 so a single runtime proves nothing on its own;
 a series of full sweeps at the same thread count is
 comparable enough to answer whether a widened walk changed the cost class.
 Record it beside the finding
 deltas rather than as an aside,
 because #87 stayed a hunch for as long as it did precisely because
 nobody wrote the number down.

## What the baseline is, and what a zero means

Current baseline:
 `sweep-51-prefix.txt` at 1966 findings,
 digests recorded beside it.
It is the first capture in this work that can be checked afterwards rather than trusted.

The previous standing baseline of 1939 is **retired as unreproducible**.
Four runs spanning both code states and both cache states read 1966,
 including one restoring the exact pre-fix source with both artifacts rebuilt.
Forty-two files appeared in both the added and removed sets of that difference,
 meaning one finding renamed rather than findings gained and lost,
 which is the signature of a capture blended across two analyzer versions.
Task #71 holds the repair of the one substantive claim that rested on it.

A zero delta does **not** mean the shape is absent.
Of 1966 findings only 32 are offers,
 so 1934 parameters are already withheld for other reasons,
 and a shape occurring constantly on already-withheld parameters produces the identical zero.
Counting syntax rather than findings:
 25 stores of a function expression into a property target,
 37 returns of a function expression,
 356 identifier stores into property targets.
Those are lower bounds and were sampled as genuine.
The shapes are common here;
 the zeros describe the offer population.

## Channels now closed, with the mechanism for each

- bare closure stored outward:
   capture recorded at the store site
- closure handed to a retaining callee:
   captures carried per formal on the owned call edge
- closure stored or returned by name:
   resolve the callable,
   never test the syntax
- conditional and container held in a local:
   ask what a value can be,
   one walk
- closure reaching state only through a sibling call:
   the capture walk follows calls
- closure returned:
   opacity,
   because the benign-return precondition fails
- construction:
   opacity,
   since `NewExpression` was handled nowhere at all
- yield:
   opacity,
   since a yielded value reaches no returned set
- async return:
   **not** a withholding,
   a tracking repair,
   by making `await` transparent
- tagged template:
   opacity,
   since a tag is a call and a `TaggedTemplateExpression` is not one
- throw:
   opacity,
   since a handler outlives the throw and no returned record exists to track it
- callable inside a returned literal:
   descend the literal,
   reusing the result-site aggregate walk
- store of what a nested callable hands back:
   a nested callable has no summary to defer against
- destructuring default:
   scan binding-element defaults,
   where the declaration scan never looked
- call result as an argument,
   and through a pattern,
   logical assignment or parameter default
- conditional write target,
   and an element or property of an authored literal

## Design decisions taken, and why

The admission gate for a handed capture is the callee's own uncertainty about the formal,
 not the reason for it.
Gating on retention provenance is unsound:
 absent retention means call-caused or unknown,
 never proven non-retaining,
 so a callee forwarding its callback to something unresolved would pass.
Provenance is then copied unchanged,
 which is what makes one channel produce two messages,
 a silent store and a spoken forward,
 without a second decision.

Captures are kept beside ordinary origins on the edge,
 never folded into them.
An ordinary origin says the callee received the caller's value,
 so a write or a return it records is a fact about that value.
A capture says only that invoking the callable can reach the parameter.
Folding them together would also reach the unresolved boundary,
 where over-approximating withholds on ordinary `map` and `filter` code.

Captured origins must not go into `bindingOriginBySymbolId`.
That map asserts a value is parameter-derived,
 and a closure is not derived from what it
 captured;
 it carries a capability reaching it.
Overloading it turns silent store provenance into reportable call provenance.

A returned callable takes opacity rather than a returned origin.
A returned origin is a positive capability claim,
 and the available helper over-approximates by
 scanning nested callable bodies.
An over-approximation is safe on a channel that withholds and unsafe on one that claims.
Nothing discharges on a returned set today,
 so the reuse would state the wrong relation rather
 than break immediately.

The capture walk's source-file bound is a **cost** bound,
 not a soundness one.
Following a cross-file callee contributes nothing,
 because its body names symbols absent from
 the origin map,
 and loses nothing,
 because a callable able to capture those bindings is written
 inside the callable that owns them.
No assertion can defend it and none pretends to.

Provenance text must be true even where no reader sees it.
Three prefixes now exist,
 all silent,
 all recognised by `isRetentionProvenance`:
 `stored into `,
 `handed outward by `,
 `handed back as a callable capturing it at `.
Reusing one prefix for a different escape produced facts like
 `stored into a construction of RowKeeper`,
 which reads as a store into the construction.

## Mutation checking is the only instrument that finds an untested rule

Five mutants have survived a green suite in this work,
 and they did not mean the same thing.

Check first whether the mutant implements the defect it claims to.
 One here restored an early
 return only when origins had already been found,
 which never happens for the shape under test,
 so
 it behaved exactly like the fixed code and proved nothing.
 That reading is the cheapest to rule out
 and the easiest to mistake for the others.

- The retention-only gate survived every fixture and control.
  The design was right and nothing measured it.
  Resolution:
   a new fixture,
   `relayCallable`.
- Deleting the source-file bound survived.
  The code was right and my account of why was backwards.
  Resolution:
   correct the claim,
   and record that no assertion can defend it.
- Treating nullish coalescence as right-operand-only survived,
  because the coalescence fixture happened to put its capture on the right.
  Resolution:
   a fixture putting it on the left,
   built so the origin walk cannot reach it either.
- An activation fix through the possible-value walk changed nothing at all.
  Two shapes written to exercise it already passed without it.
  Resolution:
   revert.
   Landing a path no shape reaches,
   documented as a fix,
   is worse than
  leaving the defect recorded.

Assert on diagnostics,
 not on offer counts,
 wherever silence is the intended outcome.
A caller that says nothing and a caller that reports argument opacity naming its callee both
 lose their offer and both read the same opaque set.
Unique parameter names are what make a diagnostic count per callable rather than per file.

## The method that is replacing the queue

Walking escape channels on purpose is cheaper than waiting for defects to surface,
 and it found
 three in one pass.

Write one file where **every** parameter genuinely leaks,
 each through a different channel,
 each
 with a unique parameter name,
 plus two controls that must still be offered.
Any offer the file draws is then a false offer by construction,
 and oxlint names them with no reasoning about the analyzer required.
Then apply the ordinary bar to each candidate.

First pass:
 fourteen channels,
 six offers,
 two of them the intended controls.
Three real defects,
 all now fixed.
The fourth candidate,
 a callback parameter retained by its driver,
 is carried forward.

Second pass staged in `hunt2.ts` in the session scratch directory:
 `Object.defineProperty`,
 `splice`,
 `Array.from`,
 a spread copy,
 `WeakMap.set`,
 `Reflect.set`,
 an optional call,
 a class field initialiser,
 a method reaching state through `this`,
 a tagged template,
 the retained callback parameter,
 an iterator object handed back,
 and two controls.

Task #73 carries this.
 Record the channels covered rather than the conclusion:
 a pass that finds nothing is the only available evidence that the shape space is closed,
 and it is weak evidence.

## Channels closed by working the queue as one pass

The queue had been filed one defect at a time and named one cause eight times:
 a call result
 reaching a use site the deferred relation did not cover.
Measuring the whole queue in a single file resolved it.

- an argument that is a call result,
   through an unresolved receiver and an owned one
- a binding through a destructuring pattern,
   a logical assignment,
   a parameter default
- a conditional write target
- a property of an authored literal,
   and an element of one
- a return of any of those

The fix is one idea applied in three places:
 ask where a value can have **come from** rather
 than what layer sits over it.
The value-source walk is reused,
 aggregate members are added on top of it only for this
 question,
 and widening can only add call sites,
 so every shape it reaches is a hole closed.

Four queue tasks needed no fix at all,
 and measuring them said so:
 a store through a destructuring pattern,
 an object literal property,
 a store from a parameter
 initializer,
 and an element property write on an array parameter.
Those four were filed from reading rather than probing.
**Probe before filing.**

Activation discovery is now gated on ancestry.
It had visited every node in the body,
 so a call inside a closure nothing runs activated its
 target and the target's body was read as though the enclosing callable had run it.
Two forms differ,
 and the first probe used the wrong one:
 a sibling bound to a `const` arrow does not reproduce it,
 a function declaration does.
Assert both halves of that fix,
 since the false fact must go **and** the offer must stay
 withheld.

## Where the work stands

**Every false offer this work found and falsified is closed.**
 Four escape-channel hunt passes drew
 44 channels;
 the fourth found nothing,
 and none of its channels had been fixed directly,
 so the
 fixes generalise past the shapes that motivated them.
 That is encouraging and still weak evidence:
 four passes are four samples of a space nobody has enumerated.
 Two of the three false offers found
 since that fourth pass came from a stronger reviewer reading the source rather than from a hunt
 pass,
 which says something about where the remaining ones are:
 in the relations between channels,
 not in the syntax any one channel scans.

Closed and swept:
 #51,
 #66,
 #67,
 #68,
 #69,
 #72,
 #74.
Every sweep but #69's came back at zero delta;
 #69's moved by one true finding,
 a closure capturing a promise `resolve` handed to `handle.once`.

Closed,
 falsified and mutation-checked,
 and **not yet swept**:
 #77,
 #65,
 #63,
 #78 and the
 write-through half of #78.
 Task #80 holds that sweep.
 #63 raises offers by construction,
 which is
 the first change here that must justify offer growth rather than only offer loss.

### The three things a callee can do with a callable a caller handed it

This is the shape of the last two defects,
 and it is worth stating on its own because the same
 relation went wrong twice in the same afternoon.
 A callee can:

-    **keep it**,
      which `propagateCapturedCapability` answered from the callee's `opaque` set,
      and
     which #69 closed
-    **hand back what invoking it produced**,
      which nothing answered,
      so `storeInvokedResult` stored
     a row aliasing the caller's own and was offered
-    **write through what invoking it produced**,
      which nothing answered either,
      so
     `handInlineToWriter` recorded no effect at all while the callee's `mutated=[0]` said exactly
     what it does

All three now read the same per-formal captures,
 and the choice of channel per case is deliberate:
 keeping speaks as opacity with the callee's own provenance,
 handing back speaks as a returned
 origin so the accepted return policy still applies,
 and writing speaks as a mutation because that
 is what happens.
 The precision control for all three is `readThroughCallable`,
 whose formal is
 neither opaque nor returned nor written because it keeps only a primitive,
 and whose caller
 `handCaptureToReader` keeps its offer.

### The fourth thing, which is not a callee at all

**#79** closes the same relation at the other kind of boundary.
 Captures lived on owned call edges
 only,
 so a call with no owned edge recorded none,
 and the reach of that is much wider than library
 functions:
 a possibly-overridden method is treated as unresolved on purpose,
 so **every instance
 method that keeps a callback** was losing the capture.
 Measured three ways,
 the same retainer
 written as an instance method recording nothing while the static and plain-function forms record
 `opaque=[0]`.
 Falsified twice.

The gate is narrow and its narrowness rests on a premise now measured in full:
 a closure handed as
 an argument is activated,
 and an activated closure's body is scanned inline as part of the
 enclosing callable,
 so every channel the enclosing callable has applies to the closure too.
 A
 store,
 an unresolved handoff,
 a construction,
 a throw,
 and a push into a container the callee
 itself supplied all record opacity for a closure that completes with nothing.
 The single channel
 that cannot apply is what invoking the closure hands back,
 because the uninspectable callee
 receives that value instead of the enclosing callable.
 So the gate asks only that,
 and
 `rows.map((row) => config.row.label,)` keeps its offer.

### Where a capture reaches an implementation, which is three positions rather than one

The inspection took a call's arguments alone.
 A capturing closure can also arrive as the
 **receiver**,
 which is what `.bind`,
 `.call`,
 `.apply` and any retaining method look like,
 or as the
 **callee**,
 which is what an unresolved invocation of a dynamically selected closure looks like.
 All
 three are inspected now,
 and the receiver form was falsified.

### And a declared type can lie about what a completion is

The gate that keeps the capture channel narrow asks whether a closure's completion can carry mutable
 state,
 and trusted the completion's static type.
 Two measured shapes abuse that trust:
 a local
 annotated `() => void` holding a row-returning callable,
 and a row asserted to a string.
 A completion
 is now judged by what it is,
 with assertions stripped and a call followed to its callable,
 and
 following stops at an external callee because an external declaration's return type is what this rule
 trusts everywhere else.

A property read is not a call,
 so the reach walk missed a getter over caller state entirely.
 It now
 collects every callable an authored literal or class expression declares whenever a body reads a
 property off one.

Every spelling of that read now answers:
 element access,
 a destructuring pattern,
 a class declaration
 reached through the construction naming it,
 and both spread kinds.
 The two spread kinds were first
 justified by one shared reason that only one of them has.
 Measured:
 an object spread runs every getter,
 an array or argument spread runs `Symbol.iterator` and no getter at all.
 Both still belong,
 because both
 reach a callable the aggregate declares without writing a call,
 but only the object form reads getters.
 No fixture charges the array form yet,
 since the reachable shape needs a receiver declaring
 `Symbol.iterator`,
 which raises an untested question about whether a yield carries a returned fact.

### Commit before mutating, without exception

This is already stated under how to work here and it cost an implementation anyway,
 so the incident is
 attached to it.

The external formal-to-actual mapping fix was written,
 type-checked,
 linted and suite-green.
 The mutation
 check then showed that a mutant restoring the broken indexing survives the **entire** suite,
 because
 reaching that path through a diagnostic needs an installed package whose shipped implementation provably
 mutates a formal,
 invoked with a spread,
 and the corpus has no such call.
 The revert that followed deleted
 **the fix** rather than the mutant,
 because the fix had never been committed.

Two lessons,
 separated because only one is about git:

-    `git checkout --` restores to HEAD,
      so an uncommitted fix is what it removes.
      Commit first,
      always.
-    A fix whose mutant survives is not finished.
      Reverting it was correct,
      and landing it on the grounds
     that it reads correctly and is provably safe would have been the error this document already warns
     about,
      made knowingly rather than by accident.

### The external channel has never run on real input

Measured by instrumentation,
 not inferred:
 a temporary line on the resolver's success path,
 seven packages
 chosen for third-party usage including consumers of the two most-used runtime dependencies here,
 **zero
 successful resolutions**.

So `applyExternalEffect`,
 its four position kinds,
 its version-locking gate and its callback-relation mapping
 have never run on real input in this workspace.
 Zero findings carry package-version provenance,
 which is the
 only provenance that path emits.

Two consequences for anyone working here:

-    A fix to that channel cannot be pinned by the fixture corpus,
      and #100's capture wiring could not be
     pinned by exporting the unit either,
      because proving a capture is charged needs real
     `bindingOriginBySymbolId` origins the analyzer does not expose.
      It is reverted with its design recorded.
-    **Diagnosed.**
      One gate rejects nearly everything.
      Cold-run counts from a `valibot` consumer,
      with all
     four rejection paths instrumented:
      **call-identity 161,
      implementation 3,
      installed-package 0,
     version-lock 0.**
      So the resolver clears installed-package identity and version locking without a single
     rejection and is stopped at `packageCallIdentity` and its declaration-owner fallback for 161 of 164 calls.

     That is why no finding carries package-version provenance.
      It is a **defect in one gate**,
      not a coverage
     gap,
      and a fixture dependency would have produced one more call that also fails there.

     **Narrowed further,
      to one syntax.**
      Printing the callee's kind at the rejection:
      property-access callees
     156 and 133 across two packages,
      identifiers 4 and 17.
      And identity **does** work for an imported
     identifier,
      proven by the later gate naming `node:path join`,
      `node:fs/promises readFile`,
     `node:child_process spawn` and `ignore Ignore` on its way to failing because a builtin has no shipped
     implementation,
      which is correct.

     **Then reading the code flipped the conclusion.**
      `packageCallIdentity` *does* handle a property-access
     callee:
      it requires the receiver to be an identifier,
      resolves it through `importBinding`,
      and answers for
     a namespace import and an import specifier.
      So `import * as v from 'valibot'; v.safeParse(...)` is covered
     by design.

     The 156 rejections are member calls whose receiver is **not an import**:
      a global (`console.log`),
      a
     parameter (`signal.addEventListener`,
      `rows.map`),
      or not an identifier at all (`this.#kept.push`).
      **Every
     one is correctly rejected**,
      because the channel resolves *package export* calls and a method on an
     arbitrary value is not one,
      whatever its type came from.

     So the channel is narrower than "external calls":
      it is "calls to package exports with a shipped
     implementation",
      and this workspace makes very few.
      Saying it "has never run on real input" treated a fact
     about the workspace as a bug.
      One confirming measurement remains,
      printing the receiver's symbol kind for
     the 156;
      then this closes as a documentation correction rather than a code fix.

     **Consequence for #100 and #111:**
      their end-to-end coverage needs a fixture calling a **package export**
     whose shipped implementation mutates a formal,
      not merely any external call.
      A refuted hypothesis is recorded on #113 so
     it is not retried:
      `getSymbolAtLocation` versus `getResolvedSymbol` in `importBinding` changed the counts
     by nothing.

     Reproduction:
      **cold-run only.**
      The persistent cache makes a second lint of the same package emit
     nothing,
      because summaries are reused and the resolver never re-runs.

One method note that generalises beyond this channel.
 **An absence in a capture is evidence about what the
 rule said,
 never about what the rule did.**
 A silent channel and an absent channel produce identical output.
 That is the same instrument limit already recorded for store-caused withholding,
 reached from the opposite
 direction,
 and instrumentation is the only thing that separates them.

### Reading finds true statements about code and unreliable ones about behaviour

Four times in this effort a hole predicted by reading one module turned out closed by another.
 Each was a
 correct reading of the module in front of the reader and a wrong conclusion about the system:

-    a throw as a closure's completion,
      covered by the enclosing throw channel firing on the activated body
-    the external capture channel,
      whose obvious probe reaches the unresolved boundary instead
-    "descend through aggregates",
      which named two shapes where only one was open
-    a closure interpolated into a retaining tag,
      already charged

Against that,
 every defect that was real was found by **measuring a shape**:
 the void slot,
 the candidate
 list,
 the four handoffs,
 the two value-walk paths,
 the tag invocation.
 Probe before filing is already the
 rule here;
 this is the strongest evidence for it so far.

**And the mirror of that rule,
 learned on #113.**
 Measuring produces true numbers and unreliable
 interpretations.
 Three instrumented runs correctly localised the external blocker to one gate and one syntax
 kind,
 and the conclusion drawn from those numbers,
 that it was a defect,
 was wrong until the code producing
 them was read.
 Neither reading nor measuring is sufficient.
 The order that worked:
 **measure to localise,
 then
 read to interpret.**

Two corollaries worth carrying:

**An unseen invocation is an unscanned body.**
 Every channel that depends on a closure's body being read
 depends first on the invocation being seen,
 and one gate decides it:
 `invokedParts` in
 `effect-invoked-parts.ts`.
 Three invoking syntaxes existed and one was recognised.
 The tagged template and
 the construction each cost one clause there,
 and the second cost only one clause **because** the first was
 written as "what does this node invoke" rather than as a second `if`.

All four invoking syntaxes are now recognised:
 a call,
 a tagged template,
 a construction,
 and a decorator.
 `super()` and an optional call needed nothing,
 the first because recognising its enclosing construction
 sufficed and the second because it is already a call.

**But recognition is only the first of four steps**,
 and the decorator proved it.
 Recognition,
 activation,
 ancestry and the body scan all have to admit a shape.
 The decorator needed its clause **and** a repair to
 #70's ancestry gate,
 because a decorator is lexically inside the declaration it decorates and does not run
 inside it:
 it runs when the class is defined,
 whether or not the decorated member is ever called.
 Each change
 was a no-op alone and both halves die independently under mutation.

So the rule is one-directional.
 An unseen invocation is an unscanned body;
 **a seen invocation is not
 necessarily a scanned one.**

And one qualification to the do-not-land-a-no-op rule,
 learned by tripping over it here.
 **A no-op means the
 change alone is insufficient,
 not that the path is unreachable.**
 Reverting is still right;
 concluding
 "unreachable" is not.
 Record the weaker claim,
 which is what made resuming this cheap.

**Two relations that look like duplicates often are not.**
 Captures against ordinary origins,
 reach against
 the value walk,
 and the accessor reach walk against the value walk all share syntax and answer different
 questions,
 so a fix to one leaves the other silent.
 The tell is the same each time:
 the same syntax appears
 in both.

### Where soundness stands, stated precisely

**Every false offer this effort measured is closed.**
 The last one was #82,
 a callee binding filled by assignment,
 landed with a fixture pair and a mutant that dies at an exact delta.

What that claim does and does not cover,
 since the distinction is the whole value of stating it:

-    It covers every shape that was **probed and charged nothing while a control behaved differently**.
      Those are
     the measured false offers,
      and each is now falsified,
      fixed,
      pinned and mutation-checked.
-    It does **not** claim the shape space is exhausted.
      Four escape-channel hunt passes and one structural audit
     sampled it;
      nobody has enumerated it.
      The audit's own limit stands:
      it says which channels lack a capture
     channel,
      never which of those a real escape reaches.
-    It does **not** cover the published rule's consumers beyond this workspace.
      The decorator fix and the
     external channel's boundary are both cases where this repository cannot reach a shape a consumer can.

**The queue is now empty.**
 The three items that remained are all dispositioned,
 each on a measurement:

-    **#54**,
      precision only,
      **declined**.
      Instrumenting the store channel found the shape it would fix occurs
     **zero** times across three packages,
      so the offers it restores are two in a synthetic fixture.
      Its sound design
     is a new propagated summary fact,
      which is a large change for a shape measured at zero.
-    **#81**,
      an unmeasured cost,
      **established as zero here**.
      Instrumenting the ownership scan found **no** parameter
     carrying an initializer reaches its exclusion,
      in four packages including one whose parameters do carry defaults
     naming an owned function.
      The gap sits behind a graph entry condition ordinary callables never meet.
-    **#101**,
      documentation debt,
      **declined for this file** and split into #115.
      All 46 `doc/planning/` files carry
     the same line-break debt,
      so this file is the largest instance rather than an outlier,
      and conforming it alone
     buys no consistency while risking prose damage to the primary record.

**#100 is closed,
 and the prerequisite it was waiting on was the wrong one.**
 It was recorded as blocked on
 finding a dependency whose shipped implementation the resolver can load.
 The answer was to **write one**.
 A
 disposable consumer that owns its own lockfile passes every gate,
 the false offer reproduced end to end,
 and
 `6a98f371a` closes it.
 Recorded under the external-channel section of this file.

What that supersedes:
 the claim that neither verification route exists here.
 The seam route was correctly
 rejected,
 and the end-to-end route was rejected on a premise about installed packages rather than about
 authored ones.
 The reasoning that kept it open,
 which was that landing a path no test reaches is worse than
 leaving a finding recorded,
 still stands and is what forced the fixture to exist first.

Measuring it also found #117,
 closed the same session:
 the external channel failed with `spawn ENOMEM` at
 oxlint's default worker count,
 which is what an ordinary consumer runs.
 It does **not** follow that #113 was
 wrong,
 and a first version of this record claimed that.
 One worker avoids the failure,
 and the measurement
 log records a sweep invoked that way,
 so #113's question of whether installed dependencies reach the channel
 at all is still open on its own terms.

So what is left is #115 as a linter-scope decision for whoever owns markdown style,
 and the standing question
 below.

So the sound summary is that the soundness queue is empty and the precision,
 cost and documentation queue is
 not.
 Anyone resuming should read the sweep that answers whether any installed dependency reaches the external
 channel,
 since that is what decides whether #113's conclusion needs revisiting.

### Open

-    **#82**,
      two activation forms that still cannot reach a parameter default:
      a binding filled by
     assignment,
      whose fix needs the enclosing node universe that `closure-activity.ts` has and the
     value walk does not,
      and two more forms that are withheld today by other channels rather than by
     activation.
      Falsify the assignment-alias form before fixing.
-    **#81**,
      an owned call written only in a parameter default is invisible to the ownership scan.
     The comment claiming that cannot happen is corrected;
      the consequence is unmeasured.
-    **#87** is **closed as declined**,
      with a measurement rather than a hunch.
      Three cold full sweeps
     spanning the four walk-widening fixes came in at 7m58s,
      8m44s and 8m54s,
      so no cost class changed
     and there is nothing for memoisation to justify.
      Memo keys recorded in case a later change moves
     that series:
      `callableResultCanCarryState`,
      `transitiveCallableOrigins`,
      `packagedActualCallables`.
-    **#95**,
      tagged templates as invocations.
      Located by reading.
-    **#100**,
      a capture channel for external effect application.
      Confirmed reachable rather than
     theoretical:
      `applyExternalEffect` does handle callback relations and maps them only through
     ordinary argument origins,
      which are empty for a closure argument.
      Last instance of the early-return
     pattern.
-    **#81**,
      **#82**,
      **#87**,
      precision or cost rather than soundness.

### What landed since the capture-channel sweep

Eight items,
 each falsified at the five-clause bar,
 each pinned by a fixture group with controls,
 each
 with a mutant that died at an exact delta.
 Fixture offers moved 49 to 63 across them,
 and every arrival
 was a control or a helper's own parameter,
 never a subject.

-    **#88**,
      a store taking a defaulted producer's invoked result.
-    **#98**,
      the returned fact a concise arrow body carries.
      The direct scan recorded returned effects
     under `isReturnStatement` alone,
      so any callable with a concise body recorded an empty returned set.
     General rather than default-specific and the widest-reaching of the eight.
-    **#91**,
      a capture handed to a callback parameter.
-    **#93**,
      one shared callable-value resolver,
      at three sites.
      The third was found by measuring again
     after the task had been marked done on the strength of having changed the sites the investigation
     named.
-    **#89**,
      the two-edge collision,
      which #93 finally made demonstrable as an answer that flips with
     source order.
-    **#99**,
      argument retention at a callback call.
-    **#94**,
      every way source spells a property read.
-    **#96**,
      correcting the claim that a nested callable has no summary of its own.
-    **#90**,
      a declared `void` result answering for a slot rather than for a body.
      TypeScript permits
     assigning a value-returning function where a void-returning one is expected and permits no other
     such substitution,
      which was verified against the compiler with an expect-error control rather than
     recalled.
      The line the fix draws is body against slot:
      a declaration states `void` about its own
     implementation,
      while a parameter,
      a mutable local or a member signature names a slot the language
     permits to return something.
      Two costs are recorded rather than hidden,
      both awaiting a sweep for
     their price:
      a closure completing with `console.log` now withholds,
      since that name resolves to a
     member signature on a variable's type,
      and a member signature returning `void` is trusted nowhere.

-    **#92**,
      a candidate list standing in for a closed set.
      Nothing the resolver follows closes that set,
     so a nonempty list is evidence about what a callee can be and never proof of what it cannot be.
      The
     statement that made it obvious is a diff of two forwarders differing in one token sequence,
      where the
     one carrying a leaf-returning default **loses** a withholding the one without it has.
      The join is
     unconditional rather than gated on list completeness,
      which is less precise than the reviewed
     recommendation and was chosen on the discipline that closed #87:
      build the simpler thing,
      then measure
     whether it costs.
      Completeness would also have to be reported by the resolver rather than derived from
     `const` against `let`,
      because unresolvable candidates are dropped silently,
      so a `const` binding can
     look complete while its list is a strict subset.

     One line now carries both #90 and #92,
      which the mutation check showed rather than the code:
      removing
     the join restored two offers rather than one,
      because the empty-candidate case flows through the same
     expression.
### The pattern worth carrying forward

Three of those eight were one shape:
 a branch in `inspectEffectCall` classifies a call,
 answers its own
 question,
 and returns before something every call needs.
 An early return there is a claim that
 everything after it is irrelevant to that kind of call,
 and the claim has been wrong three times out of
 three.
 #100 is the remaining instance.

### What landed from the audit, and what the audit got wrong about itself

The four outward handoffs now share one capture recorder,
 which is one change rather than four because they
 differ in syntax and nothing else.
 Construction,
 yield and throw each charge their subject and spare their
 control;
 the tagged-template site joins on the same reasoning.
 Falsified at construction and yield,
 mutation-checked at 68 to 71.

Two implementation points are load-bearing and neither is obvious from the site being edited:

-    Construction asks **before** its per-argument classification.
      The `sound-readonly` discharge there
     proves no write reaches *through* a handed value,
      and says nothing about a value obtained by *invoking*
     a callable that value carries,
      so a discharge must never be able to skip the capture question.
-    The activation premise does not cover any of these.
      Activation covers what a closure body **performs**,
     so a direct write and a `throw config.row` inside an activated closure are already charged.
      Returning
     caller state is neither,
      and the consumer's later use of that returned value has no call edge.

The audit named five channels and its own framing was slightly wrong about two of them.

**The external channel is not reachable by the obvious probe.**
 Handing a capturing closure to `setTimeout`
 was already charged before any of this work,
 because a host global has no shipped implementation to resolve
 and the call goes to the unresolved boundary,
 which has had a capture channel since #79.
 #100 therefore
 needs a callee with a resolvable shipped implementation,
 and a probe against a host global would have
 confirmed a fix that never ran.

**"Descend through aggregates" named two shapes and only one was open.**
 A thrown object literal carrying a
 closure was already charged through the object-literal descent from #57.
 What stayed clean is a property
 read off such a literal,
 which is the value walk's gap and is #106.

Both corrections have the same moral as the throw-completion finding:
 a reading of one module predicts a
 hole that another module already closed,
 and the only way to tell is to measure the shape.

### The audit that replaces finding this class one at a time

Six defects in this work are one defect:
 a channel maps ordinary parameter origins and has no capture
 channel.
 #69,
 #79,
 #86,
 #91,
 #100 and #95 were each found and filed separately.

Captures are kept beside ordinary origins rather than folded into them,
 which stays right.
 The
 consequence nobody stated is that every channel written against ordinary origins has a capture hole
 until someone adds one,
 and nothing in the code says which channels have.

The audit is one grep against another,
 every caller of `parameterIndexes` against every site with a
 capture channel.
 Its result is sharper than expected:
 the five channels lacking one are every function
 in `effect-outward-handoff.ts` plus `applyExternalEffect`,
 and those are exactly the two modules among
 the candidates that never import `effect-callable-capture-closure.ts`.
 The boundary of the hole is a
 module import boundary,
 visible without reading a function body.

Generalised:
 **when a design deliberately keeps two kinds of fact apart,
 enumerate every consumer of the
 first and ask which consume the second.**
 Four hunt passes drew 44 channels and found none of these
 five,
 because a hunt pass samples shapes while this samples the code.

The limit travels with it.
 The audit says which channels lack the channel,
 not which of them a real
 escape reaches.
 #100's shape is confirmed reachable,
 #95's argued,
 #102's and #103's unmeasured,
 and
 each still needs its own falsification.

### What sweep five settled, and the one prediction it broke

Offers held at 31,
 no category but argument opacity moved,
 three findings added and two removed,
 runtime
 8m54s.
 Two of the additions are two of the removals with enriched call lists.

All five movers trace to one cause,
 a defaulted callable formal resolved to the callable its default
 names and followed into another file:
 `exists = generatedFileExists` and
 `watchDirectoryImpl = watchDirectory`.
 All are true at their own source,
 since a default runs whenever a
 caller omits the argument.

The pre-registered prediction that offers would fall was wrong,
 and the reason matters more than the
 prediction.
 Offers are 31 findings out of 2005,
 and they are the parameters that already survived every
 other channel,
 so a new withholding reason lands on the already-withheld majority.
 **A sweep measures
 what a fix does to that majority and says almost nothing about the offered minority either way.**

So the criterion clause about sampling falling offers has gone untested three sweeps running and needs
 rewriting rather than repeating:
 an offer falling would be surprising,
 worth sampling hard when it
 happens,
 and its absence is not evidence that a fix withheld nothing.

One caveat against carrying that too far.
 Defaulted callable formals demonstrably occur in this
 workspace,
 since this sweep's own deltas named two,
 so #92 is the first fix whose target shape a sweep
 has actually found.
 An offer loss there is genuinely plausible.

### The instrument limit the null sweep established

A store-caused withholding is silent by design,
 so a sweep can only ever fail such a fix by raising an
 offer and can never confirm one.
 The sweep after #88,
 #98 and #91 moved not one finding in any category,
 with both artifact digests verified different from the previous run,
 and the explanation is the ratio:
 1282 argument-opacity findings against 31 offers means nearly every workspace parameter is already
 withheld for some other reason.
 The fixture group and the mutation check carry the whole weight.

## What the capture-channel sweep settled

Offers held at 31 and only argument-opacity moved,
 six added and one removed.
 The six additions are
 one true shape across three locales times two layers.
 The removal is a derivation replacing a
 conservative fallback,
 and it is precision-only on the independent ground that no offer was added or
 removed anywhere.
 Recorded in full in the planning doc.

Both artifact digests were recorded before the run and re-verified after it,
 and the two doc-only
 commits that landed while it ran left both unchanged.

## The external channel is reachable after all, and it was dead for a different reason

Two findings,
 measured together,
 and each one changes what the other means.

### Authoring the dependency works, and it closed #100

The blocker recorded against #100 was "find or add a dependency whose shipped implementation the resolver
 can load".
 The answer is to write one.
 A disposable consumer directory outside the repo satisfies every
 gate,
 because `packageVersionIsLocked` walks ancestors of `consumerProject.configFileName` rather than
 the repo root,
 so the fixture owns its own `pnpm-lock.yaml`.

What the gates actually require,
 read rather than guessed:

-   `workspace:*` cannot work.
     pnpm records workspace dependencies only as `link:package/...` under
     importers,
     never as a `name@version` package key,
     so `packageVersionIsLocked` answers false.
     This is
     the one form to not spend time on.
-   `lockfileLinePackageKey` is a section-unaware text scan,
     so any key at two-space indentation counts
     and a minimal authored lockfile suffices.
-   The generated external project already sets `allowJs`,
     `checkJs` and NodeNext,
     so a shipped `.js`
     implementation opens without special handling.
-   The fixture needs `type-fest` resolvable,
     since `typeFestResolvesFrom` gates the offer that the
     falsification depends on.

The false offer,
 through the real rule under oxlint,
 on a hand-authored package retaining a callback:

```text
consumer.ts:15: The function input named "config" is used by these calls:
  capture-retainer-probe@1.0.0 . retainCallback
consumer.ts:25: Parameter "config" should be readonly: property row is writable.
```

Those two lines are the whole falsification.
 Line 15 carries external provenance,
 which only
 `applyExternalEffect` writes,
 so the exact export resolved and applied.
 Line 25 offers `readonly` for a
 parameter whose row is handed out by a closure that same package retains.

After `6a98f371a` the same run reports
 `capture-retainer-probe@1.0.0 . retainCallback handed callable capture` on line 25 instead,
 while line 15
 is unchanged,
 so the channel that opened is the capture one rather than a wholesale loss of precision.

The fix charges **per formal**,
 not per call,
 which is the design `exposedCaptureOrigins` recorded in its own
 documentation before anything could be measured:
 only formals the external summary reports as invoked or
 opaque charge what their actual exposes,
 and a formal the mapping cannot place charges every argument.
 Charging unconditionally would also have been sound and would have thrown away the precision the early
 return exists for,
 since a summary that reports no fact about a formal is a proof the shipped
 implementation does not invoke,
 keep or write through it.

That precision is measured rather than argued,
 on an export that stamps its second formal and ignores its
 first:

```text
handRowProducerToIgnoredPosition   all sets empty      capture keeps its offer
stampThroughIgnoringExport         referentMutated=[0] same export resolved and applied
```

The pair is what makes the first line mean anything.
 Without the second,
 an empty result is
 indistinguishable from a gate that rejected the dependency,
 and it is also the pair that fails under the
 unconditional alternative.

Two branch facts,
 each pinned by its own case,
 because they arrive differently:
 an implementation that
 **keeps** a handed callable records opacity,
 and one that **calls** it records invocation.
 A third branch
 reading `callbackRelations` was removed rather than kept:
 no mutant could kill it,
 and reading it was
 asking about the wrong argument,
 since a relation names the formal fed **into** a callback and never the
 formal holding one.

The discriminator worth reusing:
 **prove the external path ran by which channel carried the fact,
 not by
 a log.**
 A consumer function passing the row directly reports opacity with package provenance;
 if any
 gate had failed,
 the same function would report ordinary unresolved-boundary opacity instead.
 Two
 outcomes that look alike in a count are distinguishable in the message.

One shape does not falsify and is worth keeping as a control.
 A closure that **writes** its capture is
 already charged,
 because the direct-write scan attributes it whatever the callee is.
 Only a closure that
 reads and hands back its capture reaches the hole,
 which is the void-slot asymmetry from #90 again:
 `() => Row` is assignable to `() => void`.

### Why every earlier reading said no implementation resolves

`openExternalImplementation` creates a second TypeScript API child during linting,
 and under oxlint that
 spawn fails:

```text
[external-callable-effect] external effect inference failed for
  capture-retainer-probe@1.0.0 ... retainCallback: Error: spawn ENOMEM
```

`initializeSemanticBridge` exists to prevent this failure for the first child,
 and its TSDoc records that
 oxlint reserves a multi-gigabyte virtual buffer per Rust worker when a JavaScript plugin is active.
 The
 external implementation project's child is created mid-lint instead,
 so it is structurally exposed.

**The ordering is not the whole explanation,
 and the sweep is what says so.**
 Reports,
 and how many of
 them carry external provenance,
 on a 16-core host with 63 GiB RAM and `/proc/sys/vm/overcommit_memory`
 at 0:

```text
threads=1    4 reports, 4 with provenance
threads=2    4 reports, 4 with provenance
threads=4    4 reports, 4 with provenance
threads=8    4 reports, 4 with provenance
threads=16   5 reports, 0 with provenance
```

Reservations exist at eight workers too,
 and the child spawns there,
 so "started after the reservations"
 cannot be the criterion.
 What decides it is the **aggregate reserved size at spawn time**,
 and the
 failing configuration is the core count,
 which is oxlint's default.
 Anyone designing against the
 ordering story alone will build the wrong fix.

Plain `node` driving `buildEffectSummaryIndex` succeeds,
 matching the single-threaded run,
 which is why
 the fixture test drives the summaries rather than oxlint.

### What fixed it, and the one thing still unpinned

One shared native child,
 started beside `initializeSemanticBridge` instead of one per generated project
 created on demand.
 `f4eae57f2`.
 At the default sixteen threads the fixture now yields four reports all
 carrying external provenance and no false offer,
 against five reports with no provenance before.

Sessions release only their snapshot now,
 since the client outlives them.
 `closeExternalImplementationProjects`
 closes it,
 which runs from `closeSemanticBridge` alone,
 so nothing closes it between linted files.
 The
 lazy fallback in `openExternalProject` still creates one on demand when the initializer never ran,
 which
 degrades to the old behaviour rather than failing,
 and that is deliberate.

Cost,
 measured rather than assumed:
 creating the child takes 1.6 ms to 4.7 ms,
 and it idles at 7.2 MiB
 resident against 1.19 GiB virtual.
 The virtual reservation is the Go runtime's,
 and it is the same kind of
 quantity that causes the failure,
 which is the argument for starting it early rather than never.

Pinned by `external-channel-workers.unit.test.ts`,
 which runs oxlint over an authored dependency and
 asserts external provenance in the reports.
 It passes **no** `--threads` flag on purpose:
 the failure
 depends on aggregate reserved size at spawn time,
 so choosing a count would pin a quantity that is not
 the shipped one.

The mutant settles which test was needed.
 Deleting the eager initializer **kills the oxlint test and
 leaves the summary-driven one passing**,
 which is the measured form of the claim that no existing shape
 could fail.
 Worth keeping in mind generally:
 a test that reaches the same code through a different host
 can be blind to a defect that only the shipped host exhibits.

Two consequences for measurement.
 A sweep at the default threads measured nothing about the external
 channel before this fix,
 so no earlier sweep in this record says anything about it.
 And a cold sweep now
 would be this channel's **first real measurement against installed dependencies**,
 which can move offers
 in either direction.

`externalCallableEffect` catches the throw,
 logs at debug and returns its generic unavailable sentinel,
 so
 the failure is indistinguishable from an unresolvable package.
 That is sound,
 since it withholds,
 and it
 is why this stayed invisible through four hunt passes.
 #117 holds it.

**A first reading of this went too far,
 and the configuration is why.**
 It was recorded here that every
 sweep in this record therefore ran with the channel dead,
 and that #113's conclusion rested on code that
 never executed.
 Neither follows.

`--threads` reaches oxlint only when `OXLINT_THREADS` is set,
 and that variable is set on the root `lint`
 task's own env block rather than on `lint:oxlint`.
 The measurement log records at least one sweep invoked
 as `OXLINT_THREADS=1 mise run lint:oxlint`,
 and at one worker this failure does not occur.
 So whether a
 given sweep had the channel alive depends on how that sweep was invoked,
 and the answer is not uniform.

What survives without qualification:
 the defect is real at the default worker count,
 which is what an
 ordinary consumer runs,
 and a silent fail-closed made it indistinguishable from an unresolvable package.
 What needs measuring rather than asserting is whether this workspace's installed dependencies reach the
 channel at all,
 which is what #113 was actually about and which one worker does not prevent.

## TypeScript 7.1.0-beta move, 2026-10-20

The workspace now pins `'typescript': '7.1.0-beta'`
(`pnpm-workspace.yaml`,
npm `beta` dist-tag,
published 2026-10-20T16:45:47.402Z).
Commits in order:
`27b1cf2eec` (catalog pin,
lockfile,
`minimumReleaseAgeExclude` entries),
`92b92380bc` (API entry-point imports),
`1ab344bd64` (docs),
`e4b2e1d72e` (API surface migration in `prefer-readonly-parameter-type`),
`787241da40` (fixture and harness test alignment),
`62ca73197b` (retired the `typescript-tuple-serialization-failed` category),
`ac0023ebe5` (omission tests driven from seeded records).

### Decisions taken this day

- The four omission-pathway tests rewrite to 7.1 as the only baseline
 (user direction:
 "pretend we were always on 7.1 beta+ and change what's there to match").
 They seed the persisted omission record as fixture data,
 so the persistence,
 restore,
 cross-process reporting and installed-package propagation contracts stay pinned end to end.
- The `minimumReleaseAgeExclude` entries stay until `7.1.0` stable
 (user choice,
 2026-10-20).
 They bypass the workspace's 24-hour `minimumReleaseAge` hold for `typescript@7.1.0-beta` and for
 the `@typescript/*` scope.

### What the move changed observably

- The 7.0.2 tuple-serialization checker panic is gone
 (`doc/troubleshooting/typescript-instantiated-tuple-serialization.md`).
- `Type.isTupleType()` now answers true for tuple references,
 unmasking one sound preference offer on `readStoredPair` in `readonly-tuple-exposure-invalid.ts`
 (`doc/troubleshooting/typescript-is-tuple-type-tuple-reference.md`).
 The writers stay silent on the `referentMutated=[0]` attribution pin.
- `semantic-overlay-filesystem.ts` returns `serverFS.useOS` on a cache miss,
 because 7.1 treats a `readFile` `undefined` answer as "absent" where 7.0.2 fell back to the OS.
- Inferred projects report `configFileName === undefined`
 (7.0.2 spelled `/dev/null/inferred`).

### Open items

- The `@typescript/*` `minimumReleaseAgeExclude` entry is broader than the seven platform binaries;
 tightening it to version-scoped entries is a follow-up candidate when the exclusions are revisited
 on `7.1.0` stable.
- `semantic-request-memo.ts`'s allow-list purity evidence is pinned to `typescript/v7.0.2`
 (`microsoft/typescript-go` `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`,
 `internal/api/session.go`);
 re-audit against the `7.1.0-beta` session surface is recommended.

### Next action

Nothing queued.
The suite is green
(`mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit` and `lint:types`),
and the open items wait on `7.1.0` stable or on a user go-ahead.

## Primary records

- `doc/planning/prefer-readonly-return-substitution.md`,
   the running measurement log
- `doc/decision/prefer-readonly-result-provenance.md`,
   the accepted result-provenance policy
- `doc/decision/prefer-readonly-contract-name-narrowing.md`
