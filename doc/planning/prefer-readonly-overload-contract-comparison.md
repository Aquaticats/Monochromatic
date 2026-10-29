# Overload contract disagreement in `package/pipe/core`

Working notes for task #17.
Read before the confirming measurement,
so the mechanism below is traced from source and is not yet measured end to end.

## What fires

`verifyOverloadConsistency` in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/overload-consistency.ts`
groups a symbol's declarations,
requires exactly one implementation and at least one bodyless overload,
and reports `inconsistentMutatesContract` when

-   `implementationSummary.mutatedParameterIndexes`

differs from the union over overloads of the same field.
The message reads "Mutation contracts disagree across callable signatures."

## The two sides are different kinds of fact

A bodyless overload gets its summary from `recordBodylessEffects` in `direct-bodyless-summary.ts`.
That function seeds `summary.mutated` from authored `@mutates` blocks and from nothing else.
An overload with no `@mutates` therefore contributes the empty set.

The implementation's side is measured from its body,
and `effect-public-summary.ts` builds the field being compared as

-   `mutatedParameterIndexes: new Set([...mutated, ...invoked,],)`

so it is the union of referent mutations with invoked capabilities.
The same module exposes `referentMutatedParameterIndexes`,
which is `mutated` alone.

An authored contract can only ever express a referent mutation.
So the comparison puts authored contracts on one side and a strictly larger vocabulary on the other,
and for any callable that invokes something reached through a parameter the two sides cannot agree
unless an author writes a contract that means something narrower than what made them differ.

## Why `runPipe` started differing

`package/pipe/core/src/run.ts:51` widens the argument object:

-   `const callableArgs = args as RunCallableArgs;`

The body then destructures `callableArgs` into `value` and `fn1` through `fn10` and invokes the step
functions.
`runPipe` writes nothing.

Before the provenance resolver followed runtime-transparent forms,
the `as` assertion stopped provenance,
so `callableArgs` carried no origin,
the step bindings carried none,
and invoking them recorded nothing.
`transparentOperand` in `effect-expression-provenance.ts` now unwraps an assertion,
correctly,
since `as` erases at runtime.
So the step bindings carry parameter zero's origin,
invoking them records an invoked capability on parameter zero,
and `mutatedParameterIndexes` becomes non-empty while every overload stays silent.

## Which hypothesis this favours

The task recorded two.
Neither is quite right as stated,
and the evidence points at a third reading.

Hypothesis (b) said bodyless overloads compute no effects and agreed only while the implementation
computed nothing.
True as far as it goes,
but it frames the fix as making bodyless overloads compute more,
which they cannot:
there is no authored form for an invoked capability.

Hypothesis (a) said the contract is genuinely incomplete.
Partly true.
Invoking a caller-supplied step is a real possible runtime effect,
and `JCH` does not forbid documenting it,
since a step function can reach whatever the caller packaged in `args`.
Writing `@mutates args` on all nine overloads would silence the finding soundly.

The reading the source supports is narrower than either:
the check compares a set that can contain invoked capabilities against a set that can only contain
authored referent mutations,
which is a category difference rather than a disagreement.
Comparing `referentMutatedParameterIndexes` instead would compare like with like.

Predicted effect of that change,
to be measured rather than assumed:

-   `readonly-overload-invalid.ts` keeps reporting.
    Its overload claims `@mutates controller` while the implementation transitions nothing,
    so the referent sets still differ.

-   The four `package/pipe/core` findings stop,
    because the implementations referent-mutate nothing and the overloads claim nothing.

-   Nothing else is lost.
    The invoked capability still reaches the ordinary parameter diagnostics on the implementation,
    which is where a reader is told about it.

## The asymmetry is structural, not incidental

Read in `direct-effect-summary.ts`:
an implementation's summary seeds `mutated` from `directMutated` and `invoked` from `directInvoked`,
both measured from the body,
and consults no authored contract while doing it.
Read in `direct-bodyless-summary.ts`:
an overload's summary seeds `mutated` from authored contract blocks and never touches `invoked` at
all.

So no authored form exists that puts a parameter into `invoked`.
Any callable whose body invokes something reached through a parameter therefore disagrees with its
own overloads permanently,
and the only way to silence it is to author a `@mutates` naming that parameter,
which claims a referent mutation in order to acknowledge an invocation.
That is a worse contract than silence,
and `JCH` is the rule it strains:
the contract would be describing the shape the comparison demands rather than the effect that exists.

## Settled by measurement

The four findings reproduce on the current tree,
at `pipe.ts:147`,
`piped.ts:156`,
`pipe-async.ts:147` and `piped-async.ts:156`,
found in the same whole-repo sweep that closed task #15.

The mechanism is `invoked`,
shown by a pair rather than by an argument.
`invokedStepOverload` in `readonly-overload-invalid.ts` declares one bodyless overload with no
contract,
and its implementation destructures a callable out of its parameter and calls it.
It writes nothing.
It was reported.
`invokedStepPlain` has the identical body with no overloads,
and was silent.
Same effect,
 same summary,
 different comparison.
That places the disagreement in the check.

## What landed

`verifyOverloadConsistency` now compares `referentMutatedParameterIndexes` on both sides.

Measured after:

-   `inconsistentReadonlyOverload` still reports.
    Its overload claims `@mutates controller` and the implementation transitions nothing,
    so the referent sets still differ,
    which is the case the check exists for.

-   `invokedStepOverload` stops reporting.

-   `package/pipe/core` goes from thirteen findings to nine,
    and the four that left are exactly the overload ones.
    The other nine are pre-existing opacity reports naming `steps.slice` and supplied callbacks.

Nothing is lost from the analysis.
The invoked capability is still recorded on the implementation's summary and still propagates to
callers,
so a caller passing mutable state into an overloaded higher-order callable is still refused an offer.
What stopped is reading that capability as a contract a signature failed to state.

## Which hypothesis held

Neither as written.
The task recorded (a) a genuine contract incompleteness and (b) a rule-side defect where bodyless
overloads compute no effects.

The answer is (b) in location and neither in mechanism.
Bodyless overloads computing nothing was never the problem:
they compute exactly what an author wrote,
which is the right amount.
The problem was the other side computing something no author can write.
