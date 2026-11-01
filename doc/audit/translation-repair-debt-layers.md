# Debt layers in the translation-repair package

Walked 2026-08-19 over 610 `.ts` files in `package/module/translation-repair/src` (moved on 2026-10-18 to
`package/translation-repair/*/src`),
zero quota,
read-only,
while the `#107` verification ran.
Layers per `EVL`,
and the point of walking them is that a layer-one-only reading is a guess.

## Inline annotations, the fourth layer

```text
TODO          0
FIXME         0
HACK          0
XXX           0
WORKAROUND    0
deprecated    0
"for now"     1
```

`EL4` reads zero as discipline only when the search is verified to have run,
so it was:
the same pass over the same 610 files returned non-zero for other patterns,
and the one `for now` was located and read.

THE SINGLE `for now` IS NOT DEBT.
It sits in `chunk-document.ts` explaining that a
misleading name is kept because the scorecard renders it into a finding string that 56
settled artifacts share,
so renaming it is a comparability change rather than a rename,
and it moves with `#99`.
That is a decision with its reason and its trigger recorded,
which is the opposite of a
loose end.

## Suppressions and escapes, the fifth layer

```text
oxlint-disable      52    of which carrying a `--` justification    52
ts-expect-error      0
@ts-ignore           0
eslint-disable       0
.skip(               0
as unknown as       48    in tests 48    in production source        0
as any               0    real casts; the 2 textual matches are English prose
```

`EL5` counts a justified suppression as healthy and a bare one as debt,
and every one of the 52 carries a reason after `--`.

THE `as any` COUNT IS ZERO AND THE FIRST READING SAID TWO.
A word-boundary search for `as any` matched two TSDoc sentences,
"reading its silence as any particular one" and "at least as wide as any period",
which are English rather than TypeScript.
Re-counted excluding comment lines:
0 in code,
2 in comments.
A grep for a type escape that also spells a common English phrase will always answer a
slightly different question than the one asked.

THE `as unknown as` SPLIT IS THE HEALTHY DIRECTION,
and the split is the whole finding.
All 48 are in tests,
none in production source.
That is what a suite obeying `THR` and `GFP` looks like:
a guard is only trusted once it has
been shown to reject something,
and building the thing it must reject means constructing a
value the types forbid.
The same 48 in production source would be the opposite finding.

## What this does not say

It does not say the package is finished.
`PKG` also wants tests covering every exported code path,
and these layers say nothing about
that;
an attempt to measure it the same afternoon produced a headline of 160 untested
exported functions that fell to 64 once the search recursed into `src/corpus-run`,
and the
64 turned out to include guards tested by message and helpers tested through their callers.
Coverage against implementation BRANCHES,
which is what `TC2` actually asks for,
has not been
measured and is not measured by counting mentions.

What it does say is that this package carries essentially no inline debt and no bare
suppressions,
which is a real and unusual result for a package of this size.
