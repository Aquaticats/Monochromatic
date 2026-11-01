# TypeScript 7.0.1-rc: isolatedDeclarations rejects exported consts initialized from other consts (TS9010) even when the type is trivially number

Under `isolatedDeclarations: true`,
 an exported `const` whose initializer is an expression
(not a literal) must carry an explicit type annotation,
 because declaration emit must be
derivable from the single file without type inference across expressions.

Found while building `package/file-manager/variant-electron`'s shared constants.

## Symptom

```txt
src/constants.ts(54,14): error TS9010: Variable must have an explicit type annotation with --isolatedDeclarations.
```

for:

```ts
export const ROW_STRIDE = PANE_HEIGHT + PANE_GAP;
```

while sibling literal consts (`export const PANE_GAP = 12;`) pass,
 since a literal's
declaration type is syntactically evident.

## Root cause

`isolatedDeclarations` (TypeScript 5.5+) restricts declaration emit to what a per-file tool
can produce without the checker;
 an arithmetic expression over imported-or-earlier consts
requires inference,
 so the compiler demands the annotation.
Documented behavior of the flag;
 the repo's `@monochromatic-dev/config-typescript/dom` build
config enables it for the tsc-emitted browser artifacts.

## Verification

Environment:
 typescript 7.0.1-rc (repo catalog),
 `tsconfig.build.json` extending
`@monochromatic-dev/config-typescript/dom` with `isolatedDeclarations: true`.

- `export const ROW_STRIDE = PANE_HEIGHT + PANE_GAP;` fails with TS9010.
- `export const ROW_STRIDE: number = PANE_HEIGHT + PANE_GAP;` compiles;
   shipped in
  `file-manager-electron/src/constants.ts`.

## Verified workarounds

- Annotate the exported const (`: number`).
   Tradeoff:
   the emitted declaration widens from the
  literal arithmetic result to `number`;
   irrelevant here,
   worth knowing where literal types
  matter (annotate with the literal type or restate the literal in that case).

## What does not work

- `as const` on the expression:
   TS9010 is about annotation presence for emit,
   not widening.

## Upstream filing decision

`.out-of-scope/` has TypeScript-related entries (`typescript-project-references.md`,
`low-impact-typescript-formatting.md`);
 neither exempts this,
 but no filing is warranted
anyway:

1. Really upstream's fault?
    No;
    this is the flag's documented contract.
2. Can upstream fix it?
    Nothing to fix.
3. Supported use case?
    Yes,
    with the exact remedy the error message names.
4. Would the repo welcome the contribution?
    Not evaluated;
    constraint 1 fails.
5. Will they likely fix it?
    Nothing to fix.
6. Prototyped minimal fix?
    Not applicable;
    one-line annotation recorded above.

Decision:
 nothing to file.
