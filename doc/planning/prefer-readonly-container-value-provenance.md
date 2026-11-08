# Splitting a fresh container's identity from what it contains

Proposal,
 not a decision.
 Measured 2026-08-07 while working the argument side of
`prefer-readonly-parameter-types`,
 and written down because the reproduction is cheap and the sound fix
is not.

## What was measured

A local work-stack holding a parameter reports the parameter as mutated when the stack is mutated.
Nothing writes the parameter.

One variable per case,
 each printing the callable's own effect summary against a real program:

```ts
// const stack: Node[] = [root]; stack.pop();
// -> mutated=[0] referentMutated=[0] opaque=[]      wrong: nothing writes root

// const stack: number[] = []; stack.push(1); root.label.length
// -> clean

// const stack: Node[] = [root]; stack.push(root);
// -> mutated=[0] referentMutated=[0] opaque=[0]

// for (const child of root.children) void child;
// -> clean

// const seen = { total: 0 }; seen.total += root.label.length;
// -> clean

// const stack: Node[] = [root]; for (const held of stack) void held;
// -> clean

// const held: Node[] = [root]; held[0]?.label.length
// -> clean
```

Draining,
 accumulating and the loop itself are all refuted.
 The single cause is mutating a container that
holds the parameter.

## Why it matters here

`AGENTS.md` requires this shape.
 `ITR` forbids recursion over a structure and directs a work-stack instead,
so the analysis misreports the iteration idiom the repository mandates.
 `package/cli/markdown-lint/src/walk.ts`
is the instance:
 26 findings name `stack.push` and `stack.pop`,
 on a stack seeded with the parameter and
never escaping the function.

## The chain that produces the wrong fact

```text
expressionValueOrigins([root])   = { parameter 0 }
binding origins of stack         = { parameter 0 }
rootParameterOrigins(stack)      = { parameter 0 }
stack.pop()                      = mutation of parameter 0
```

The middle step is in `discoverAliasOrigins` in `effect-binding-origins.ts`.
 `stack` is a plain identifier
rather than an array binding pattern,
 so the declaration takes the `expressionOrigins` branch on its
initializer and registers those origins on the binding.
 The final step is `recordCollectionMemberEffect` in
`effect-collection-member-effect.ts`,
 which charges `rootParameterOrigins` of the receiver whenever the
structure claim answers mutated.

Neither step is wrong on its own terms.
 `expressionValueOrigins` answers "which caller parameters can this
expression's value reach",
 and `[root]` really can reach `root`;
 its own TSDoc says the literal's identity is
fresh and is not what is being credited.
 The consumer asks a different question,
 "which parameters does
mutating this receiver mutate",
 and reachability does not answer it.
 A container reaching the parameter is
not the parameter reaching the container.

## Why this was not simply a bug to fix

Over-attribution is a deliberate posture elsewhere in the same code.
 `containerElementReceiver` says so
directly:
 following only a declaration initializer "over-attributes rather than under-attributes:
 the write is
credited to a receiver whose elements the local may no longer hold,
 which costs precision and never an
offer".
 Charging the container's mutation to the parameter is the same trade,
 and reversing it moves toward
precision in the direction that can mint a wrong offer.

That is the guarded failure mode of this rule.

## Why the narrow fix is unsound

Making a fresh literal's value origins empty,
 by itself,
 loses real mutations.
 Reviewed against the source
by an external model and checked against the modules named here:

Array attribution stops working.
 `const stack = [root]; stack[0].label = x` writes the parameter through
the container,
 and `containerElementReceiver` cannot follow it:
 it recognises calls and identifier chains
that end in calls,
 and has no array-literal case.
 Its singular `Expression` return cannot represent
`[first, second]`,
 a spread,
 or a nested container at all.

Object attribution has no relation to fall back on.
 `const held = { inner: root }; held.inner.label = x`
needs property-value provenance,
 which does not exist;
 the element relation does not cover an object's
properties.
 Object destructuring depends on the same conflation today,
 so `const { inner } = { inner: root }`
would lose its origin as well.

Assignment does not propagate.
 `let held: Node[] = []; held = [root]` is reached by neither the
declaration-initializer walk nor the value-origin alias propagation,
 so a parallel element relation has to
accumulate assignment origins too.

Two concrete programs the narrow change would wrongly offer read-only:

```ts
function bump(box: Box,): void {
  const held = { inner: box, };
  held.inner.value += 1;
}

function bumpThroughArray(box: Box,): void {
  const held = [box,];
  const first = held[0];
  if (first !== undefined)
    first.value += 1;
}
```

## What a sound fix needs

A provenance split rather than an emptied branch.
 Three relations where there is now one:
 the value's own
identity,
 which is fresh for a literal;
 the elements it holds;
 and the properties it holds.
 Element and
property relations then have to survive declarations,
 assignments,
 destructuring,
 iteration,
 spreads,
 nested
containers,
 and the results of members that carry elements onward,
 including `pop`,
 `at`,
 `filter` and
`slice`.

The fallback matters as much as the relations.
 Where propagation cannot be resolved,
 including at the
existing alias hop limit,
 the answer has to become opaque rather than proven empty.
 Value-origin
conflation is what currently makes the unresolved case safe,
 so removing it without an opaque fallback
converts every gap into an offer.

## A narrower shape that dodges every wrong-offer case, and what it does not buy

Proposed after the redesign above was costed,
 because the wrong-offer cases all depend on emptying value
origins and this does not empty them.

Leave `expressionValueOrigins` exactly as it is,
 so every attribution that works today keeps working.
Record separately,
 alongside the origins,
 which bindings received theirs through a fresh container literal,
propagating that record through alias hops the same way origins already propagate.
 Consult it in one place
only:
 the structural-mutation charge in `recordCollectionMemberEffect`,
 which skips crediting the receiver's
origins when every one of them arrived that way.

Nothing is removed,
 so `const stack = [root,]; stack[0].label = x` keeps its attribution through the
element path,
 and `const held = { inner: root, }; held.inner.label = x` keeps its through the value path.
Both wrong-offer programs in `Why the narrow fix is unsound` depend on those paths going away,
 and under this they do not.

Three probes decide whether the record can be kept sound,
 and all three were run:

- `const stack: Node[] = [root,]; const alias = stack; alias.pop();` reports `referentMutated=[0]`
 exactly as the direct form does,
 so origins already survive one alias hop and the record has to survive it
too.
 A record that did not would discharge the aliased form while suppressing the direct one,
 which is the
same asymmetry between `find` and `at` that this session spent a commit fixing.
- `const inner = config.rows; inner.push(row,);` reports `referentMutated=[0]`,
 and must keep doing so.
 Its
binding arrives through a property step rather than a container literal,
 so the record is not set and the
charge stands.
- `const stack: Node[] = [root,]; stack.push(root,); stack[0].label = x;` reports
 `referentMutated=[0]`,
 through the element path rather than the structural charge,
 so suppressing the
charge does not lose it.

What it does not buy is most of the findings.
 The work-stack shape reports `referentMutated=[0]` and
`opaque=[0]` together,
 and this addresses only the first.
 The opacity comes from `stack.push(child,)`
handing parameter-derived state to a container whose later use nothing follows,
 which is the escape
question again and not this one.
 Measured:
 `const stack: Node[] = [root,]; stack.pop();` is already
`opaque=[]`,
 while adding the push makes it `opaque=[0]`.

So the sound description is that this removes a wrong fact rather than a report.
 That is still worth doing
on this repository's own standard,
 recorded in
`doc/decision/prefer-readonly-unpaired-view-membership.md`:
 a wrong inference is worse than an absent one.
But it should not be sold as clearing the 26 findings,
 because it will not.

## The narrow shape landed, 2026-08-07

Built as described,
 in `effect-container-literal-holder.ts`,
 recorded in `effect-body-bindings.ts`
beside the origins and the result sites,
 and consulted in exactly one place:
 the structural-mutation
charge in `recordCollectionMemberEffect`.

Three conditions decide the record,
 and each was added for a case rather than for symmetry.
 The bound
name must be a plain identifier,
 because a destructuring pattern binds what a container holds rather than
the container itself.
 The declaration must be `const`,
 which is the guard that makes the record safe to
consult at a call site instead of only at the declaration:
 `let stack = [root,]; stack = config.rows;
stack.pop();` holds a container this callable built when it is declared and one the caller owns by the time
the member runs,
 and a `const` cannot move like that.
 The initializer must be an array or object literal
written here,
 or an identifier already recorded,
 which is what carries the record across
`const alias = stack;` the way origins already cross it.

Verified against the probes this document set out,
 all four holding:

- `const stack: Node[] = [root,]; stack.pop();` reports `referentMutated=[]`,
 where it reported `[0]`.
- `const stack: Node[] = [root,]; const alias = stack; alias.pop();` reports `referentMutated=[]` too,
 so
the aliased form and the direct form agree.
 A record that stopped at the declaration would have discharged
one and suppressed the other,
 which is the `find` against `at` asymmetry this session spent a commit
closing.
- `const stack: Node[] = [root,]; stack.push(root,); stack[0].label = x;` still reports
 `referentMutated=[0]`,
 through the element path rather than the suppressed charge.
- `const inner = config.rows; inner.push(row,);` still reports `referentMutated=[0]`,
 and so does the
direct `config.rows.push(row,)`.
 Its binding arrives through a property step,
 so the record is not set.

The work-stack shape reports `referentMutated=[]` and still `opaque=[0]`,
 exactly as predicted:
 this
removes the wrong fact and leaves the report.
 Clearing the report is the escape question,
 which
`doc/planning/prefer-readonly-return-substitution.md` owns.

### What it measured

Workspace either side:
 2908 errors to 2907,
 1570 rule findings to 1569,
 warnings unchanged at 3902,
stale `@mutates` findings unchanged at 14.
 One finding cleared,
 which is what this document predicted:
the change removes a wrong fact rather than a report.

The read-only offer set is byte-identical either side,
 none appearing,
 disappearing or moving.
 That
was the number to watch,
 because withdrawing a mutation is exactly what makes a parameter offerable,
and an offer minted by a withdrawal that should not have happened is the failure this rule guards.

The one finding that cleared is `createExtensionContext` in
`package/pi-plugin/spawn/src/pi-test-harness.ts`,
 and it is the shape this was built for:
 a fresh array
built by spreading the parameter's own property,
 then pushed onto.
 The push restructures the copy,
 the
caller's array is untouched,
 and charging it to the parameter reported a write nothing performs.

Held to four fixture cases in `readonly-result-provenance-invalid.ts` rather than to the probes alone:
both wrong-offer programs this document names,
 the restructure the record exists for,
 and a caller-owned
container reached through one local hop.
 The restructure case also produces the record's visible
consequence,
 an offer where there was a report,
 and it is listed in the fixture's exhaustive offer list
so a future change that withdraws it has to say why.

## Recommendation

Not now,
 and not as part of the argument-side work that produced it.
 The stake is real:
 26 findings on one
file and the repository's mandated iteration idiom.
 The cost is a provenance redesign across
`effect-expression-provenance.ts`,
 `effect-binding-origins.ts`,
 `effect-container-element-origin.ts` and
`effect-collection-member-effect.ts`,
 whose failure mode is the one thing this rule must never do.

Taken on the same ground the typed arrays and `Set` were left undone in
`doc/decision/prefer-readonly-unpaired-view-membership.md` and
`doc/decision/prefer-readonly-result-provenance.md`:
 an increment whose cost is dominated by making its
own evidence trustworthy is not improved by doing it quickly.
