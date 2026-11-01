# `stylistic/comma-dangle` corrupts `;`-terminated type literals of destructured parameters

## Symptom

`oxlint --fix` with `stylistic/comma-dangle` enabled (oxlint 1.89.0,
`@monochromatic-dev/oxlint-plugin-stylistic` 0.0.1) leaves unparseable TypeScript wherever an
object-pattern parameter carries a type literal whose last member ends with `;`:

```ts
// input
export async function canaryUnder({ salt, }: { readonly salt: number; },): Promise<readonly number[]> {
// after one --fix pass
export async function canaryUnder({ salt, }: { readonly salt: number;, },): Promise<readonly number[]> {
```

The inserted `,` lands after the member's `;`,
producing `;,`.
oxlint then reports `Unexpected token` on the file and no further fixer applies to it.
The first corpus sweep recorded in `doc/troubleshooting/oxlint-typescript-indent-rule.md` broke 3,713
of 7,091 mirror files this way.

## Root cause

Trailing-comma insertion points are chosen in
`package/oxlint-plugin/stylistic/src/utility/comma-dangle.ts`
(`insertionTokenForList`,
`containerPenultimateToken` path).
An oxc binding-pattern node range includes its type annotation,
so the pattern's list brace is not the container's last token.
Before the fix,
`containerPenultimateToken` (`package/oxlint-plugin/stylistic/src/utility/comma-dangle.ts:506`)
resolved the close delimiter with:

```ts
const token = context.sourceCode
  .getLastToken(
    container,
    1,
  );
```

For `{ salt, }: { readonly salt: number; }` that "token before the close delimiter" is the member's
`;`,
and the fixer appends `,` after it.
The same mis-placement emits valid text when the last member has no separator,
which hides the bug behind `;`-terminated members,
and it swallows the pattern's own missing trailing comma,
which the wrong lookup never sees.

Measured trigger family (minimized repros in `~/temp/agent/indent-probe-2026-10-22/comma-repro/`):

- object-pattern parameter plus `;`-terminated type literal member:
  corrupted,
  in both function and arrow forms;
- identifier parameter plus the same type literal:
  never touched,
  even after repeated `--fix` passes;
- member without separator:
  trailing comma inserted,
  output parses;
- `,`-terminated member:
  untouched.

Hypotheses checked and rejected:

- "the stale `dist` bundle caused it":
  reproduced on a fresh rebuild;
  `comma-dangle.ts` has not changed since 2026-09-06;
- "the fixer appends a comma after any last member":
  identifier-parameter type literals are never flagged,
  so the insertion targets the wrong list's tail rather than the member;
- "function declarations only":
  arrow functions reproduce it too.

## Fix and workaround

A fix shipped in commit `0a49269a48`
(`fix(*): stop comma-dangle inserting inside type-annotated pattern annotations`).
`containerPenultimateToken` (`package/oxlint-plugin/stylistic/src/utility/comma-dangle.ts:506`)
now resolves the container's own close delimiter,
skipping a trailing type annotation that extends the node range:

```ts
const closeToken = isNode(typeAnnotation,)
  ? context.sourceCode.getTokenBefore(typeAnnotation,)
  : context.sourceCode.getLastToken(container,);
```

Measured behavior after the fix,
on the trigger family and on new fixtures:

- object-pattern parameter plus `;`-terminated type literal member:
  untouched,
  in both function and arrow forms;
- missing pattern comma:
  inserted at the pattern's tail (`{ salt }` becomes `{ salt, }`),
  where it belongs;
- array pattern plus tuple annotation:
  pattern and tuple commas each inserted at their own list tail;
- params-list comma:
  inserted after the annotation (`},)`),
  the correct tail of the parameter;
- identifier parameter plus the same type literal:
  still untouched.

Regression tests live in `package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts`
(`reports missing trailing commas on type-annotated patterns`,
`valid type-annotated pattern cases produce no violations`,
`--fix adds pattern commas and leaves type annotations alone`)
with fixtures in `package/test-fixture/oxlint-stylistic/src/{invalid,valid}/comma-dangle-pattern-annotation.ts`.
They were red first:
the pre-fix run corrupted the fixture into `Unexpected token`.

Recover an already corrupted file with `git checkout -- <file>` or by removing the stray `,` after the member separator by hand.

Two rejected quick fixes stay on record:
skipping insertion when the penultimate token is `;`
stops the corruption but leaves member separators unnormalized,
and replacing the `;` with `,`
matches dprint's `trailingCommas: always` output but rewrites more text.
The landed root fix needs neither tradeoff.

## Upstream filing

The emitter is this repository's own rule,
so no external upstream filing applies and the
`.out-of-scope/` constraint check does not gate anything here.
A duplicate search of `doc/troubleshooting/` found no earlier record of this failure mode.
The fix landed in `0a49269a48`;
the investigation handover
`doc/handover/issue-577-indent-rule-investigation.md` records the full run.
