# A barrel `export *` drops an ambiguous name silently

FOUND 2026-08-17 while adding a reader to `@monochromatic-dev/module-translation-repair`.

## What happened

Two new functions were added to `src/corpus-run/rendering-audit-settled-input.ts`
and re-exported from `src/corpus-barrel.ts`,
which `src/index.ts` re-exports with `export * from './corpus-barrel.ts'`.

One of the two arrived in the built package.
The other did not.

```text
settled-ish exports: assertSettledRecordAgrees, buildSettledArtifact,
buildSettledArtifactV2, countSettledPerBand, parseSettledArtifact,
parseSettledArtifactV2, readSettledArchive, settledTallyLine
```

`readSettledArchive` is there.
`readSettledArtifact`,
added in the same commit,
in the same `export` clause,
from the same module,
is not.

That listing is quoted as it was printed and is not updated.
`#205` renamed the two-lane family off its version number on 2026-08-25,
so `buildSettledArtifactV2` and `parseSettledArtifactV2` no longer exist:
they are now `buildSettledTwoLaneArtifact` and `parseSettledTwoLaneArtifact`.
`parseSettledArtifact` still names the version-1 reader,
unchanged.

## Cause

`readSettledArtifact` already existed,
in `src/artifact-read.ts`,
reaching the package through `src/sheet-barrel.ts`.
`index.ts` star-exports both barrels.

The ECMAScript module semantics for this are explicit and quiet:
when two `export *` declarations would supply the SAME name from DIFFERENT modules,
the name is excluded from the resulting namespace.
Not an error,
not a warning,
not a last-one-wins.
It is simply absent.

## TS2308 fires for values too, corrected 2026-08-17

An earlier version of this document said types collide loudly and values collide
silently,
on the evidence that `readSettledArtifact` vanished while a type-check passed.
THAT RULE IS WRONG,
and a third collision is what prompted checking it:
a new `repeatBandOf` met the `bandOf` in `band-order.ts`,
both plain functions,
both arriving through `corpus-barrel.ts` against `sheet-barrel.ts`,
and it failed loudly.

Reduced to five files in a throwaway directory,
with two functions of the same name and no types involved at all:

```ts
// src/index.ts
export * from './barrel-one.ts'; // re-exports shared() from alpha.ts
export * from './barrel-two.ts'; // re-exports shared() from beta.ts
```

```text
src/index.ts(2,1): error TS2308: Module './barrel-one.ts' has already exported
a member named 'shared'. Consider explicitly re-exporting to resolve the
ambiguity.
```

So TypeScript reports the ambiguity for values as readily as for types.
The same reduction stays SILENT,
correctly,
when both paths reach the SAME declaration,
because a diamond is one export and not two.

What that means for the original incident:
the clean type-check recorded there did not prove TypeScript was quiet about it.
Far likelier that no type-check ran between the barrel line landing
and the import failure turning up,
since the built package comes from the bundler,
which emits regardless.

So the practical rule is the opposite of what was written here:
RUN THE TYPE-CHECK after touching a barrel,
because it does catch this,
and a collision reaching a test means the check was skipped rather than fooled.
The runtime half of this document stands unchanged and still bites,
because a bundle can be built and shipped without one.

## What the runtime actually does, measured

```text
namespace keys: onlyAlpha, onlyBeta
shared() threw: TypeError m.shared is not a function
```

Two modules star-exported into one,
each exporting `shared` plus one unique name.
The unique names both survive.
`shared` is not shadowed,
not last-one-wins,
not an error at import:
it is simply absent from the namespace,
and the failure lands at the CALL SITE of whoever expected it.

## Why nothing else caught it

-   `oxlint` passed.
    No rule looks across barrels for name collisions.
-   The bundler passed and emitted BOTH function bodies into `index.mjs`,
    so grepping the built file for the name finds it and proves nothing.
    Only the export list is affected.

The only symptom was an import failure in a test:

```text
SyntaxError: The requested module '../../dist/final/node/index.mjs'
does not provide an export named 'readSettledArtifact'
```

That message names the missing symbol and says nothing about why,
which reads exactly like a forgotten barrel line.
The barrel line was there.

## What to do about it

Rename.
`CRN` already forbids reusing an existing code name,
and this is the mechanism that punishes it:
the collision does not announce itself where it is created,
it removes an unrelated symbol somewhere downstream.
The new functions became `readArtifactSubjects` and `readArchiveSubjects`,
which also say what they return.

Do NOT add the old name to a forbidden-strings appendix.
`readSettledArtifact` is a legitimate,
still-live function in `artifact-read.ts`;
a deny-list entry would flag every honest use of it.
The rejected thing was the DUPLICATE,
not the name.

## How to check, since a type-check will not

Ask the built package what it actually exports:

```sh
node --input-type=module -e "
const m = await import('./dist/final/node/index.mjs');
console.log(Object.keys(m).filter((n) => n.includes('Settled')).join(', '));
"
```

Before adding a symbol to any barrel,
grep the whole package for the name first:

```sh
rg --count-matches '\bmyNewFunction\b' ./src
```

A hit outside the file being written is the collision,
and it is cheaper to find here than in a test three steps later.
