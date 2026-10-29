# A root parent walk that threw, and the callable it deleted

Scope:
 `package/oxlint-plugin/prefer-readonly-parameter-type`.
Found while investigating why a callable storing a member result into a module binding
 drew no diagnostic at all,
 neither an offer nor a report.

## Symptom

At the summary level the callable had no entry in the effect index whatsoever.
Not empty facts,
 which is what an uninteresting callable has,
 but nothing:

```ts
// package/test-fixture/oxlint-no-restricted-syntax/src/, probe
let escaped: Row | undefined;

export function storeMemberResult(rows: Row[],): void {
  escaped = rows.at(0,);
}
```

`buildEffectSummaryIndex` returned `NO_EFFECT_SUMMARY` for it,
 while the near-identical `escaped = rows[0]` had an ordinary summary with empty facts.
At the user boundary the difference showed as silence for the member form and an offer of
 `readonly Row[]` for the index form.

The asymmetry is the clue.
Only the member form reaches the escape test,
 because only a verified member call has receiver opacity to discharge.

## Cause

`Node.parent` is declared present by the TypeScript unstable AST and is absent at a source
 file.
Measured rather than assumed,
 by ascending from a declaration name and printing each step:

```text
79 -> 263 -> 307 -> PARENT_UNDEFINED
```

Identifier,
 then function declaration,
 then source file,
 then nothing.

`targetIsCallableLocal` asks whether an assignment target's declaration sits inside the
 callable body,
 and for a module binding the answer requires walking past the source file.
The walk guarded only against a self-referential root:

```ts
while (cursor.current !== container) {
  if (cursor.current.parent === cursor.current)
    return false;
  cursor.current = cursor.current.parent;
}
```

`undefined` is not equal to the cursor,
 so the guard did not fire,
 the cursor became `undefined`,
 and the next iteration dereferenced it.

The throw was then caught by the demand index,
 which omits one callable rather than losing a whole file,
 and logs a warning at a level the ordinary run does not print.
So a crash presented as an absence.

## Why it mattered more than a missing offer

The omission is fail-closed,
 since callers of an absent callee take opacity and the callable itself is never offered.
Nothing unsound shipped.

What it cost is the classification the escape test was built for.
Property and element stores were covered,
 and the module-binding store,
 the one target kind that most obviously leaves the callable,
 could not be reached at all:
 the code that would have classified it threw before answering.

## Fix

Both root walks in the package now stop on an absent parent as well as a self-referential
 one,
 through a shared `isPresentNode` in `effect-value-consumer.ts`.
That predicate uses `Object.is` rather than a comparison against `undefined`,
 because writing the comparison needs a nullish union to compare through and the
 `no-nullish-union` rule forbids one.
The question it asks is whether the runtime handed a value back,
 not whether the declared type admits absence.

`enclosedByNestedCallable` in `effect-result-escape.ts` carried the same assumption and was
 corrected with it.
Every caller passes a node inside the body,
 so that walk should meet its boundary first and the guard should be unreachable there,
 which is a reason to write it rather than a reason to omit it.

Pinned by `storeIntoModuleBinding` in `readonly-assignment-store-invalid.ts` and its
 assertion in `effect-summaries.unit.test.ts`.
Restoring the self-reference-only guard reproduces the original symptom exactly,
 as `Expected an effect summary for storeIntoModuleBinding`,
 which is how the diagnosis was confirmed rather than argued.

## What to check when a callable seems to have no verdict

Silence and empty facts look identical from outside and are not the same state.
Read the summary directly before reasoning about the rule's judgement:
 `NO_EFFECT_SUMMARY` means the callable was omitted,
 and omission means something threw.

## What else the crash was doing: a boundary reported by the wrong name

The same walk was corrupting a diagnostic in a package that stores nothing into a module
 binding,
 which is how a crash in one predicate reached code with no relation to it.

`package/done/app/src/server.ts:224` calls `serveStatic` from `h3`.
Before the fix the report named the boundary as authored text and a source position;
 after it,
 by package identity:

```text
before   serveStatic [/var/home/user/Monochromatic/package/done/app/src/server.ts:225]
after    h3@2.0.1-rc.26 . serveStatic
```

Same parameter,
 same reason,
 different name for the thing that caused it.
The package form is written in `external-callable-effect.ts` and requires
 `externalCallableEffect` to return a proven effect;
 the authored form is the fallback `recordOpaqueBoundary` writes when it does not.

The route runs through h3 rather than through the workspace.
`serveStatic` calls `resolveDotSegments`,
 which begins by assigning to its own parameter:

```js
function resolveDotSegments(path, opts) {
	if (path[0] !== "/" || path[1] === "/" || path[1] === "\\") path = "/" + path;
```

`targetIsCallableLocal` resolves `path` to its parameter declaration,
 which sits beside the body rather than inside it,
 so `nodeWithin` ascends past the function to the source file and off the end.
The exact over-narrowness already recorded against `targetIsCallableLocal` is what
 supplied a node outside the container for the walk to fall off.

Confirmed by reverting the guard alone,
 rebuilding,
 and linting the one package:
 the boundary name flips back,
 and restoring the guard flips it forward again.
Both readings take seconds,
 so this needs no sweep.

That retires an earlier hypothesis worth naming because it was wrong in an instructive
 direction.
The flip had been attributed to `DEFAULT_ANALYSIS_BUDGET_MILLISECONDS` being wall-clock,
 making diagnostic text depend on machine load.
It is deterministic and depends on one predicate.
A plausible mechanism that explains a symptom is not evidence for that mechanism.

One thing the reproduction shows and does not explain.
The isolated single-package run prints no omission warning in either state,
 while the full sweep prints one for `resolveDotSegments` in the failing state.
`externalCallableEffect` catches at debug level and returns its unavailable sentinel,
 and the demand index catches at warn level and omits;
 both end in a boundary the rule cannot see through.
Which catch received the throw in which run is not established by this measurement.
What follows for anyone counting omission warnings to size the problem:
 that count is a floor,
 because one of the two channels is silent at ordinary verbosity.
