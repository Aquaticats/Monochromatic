# TypeScript 7.1.0-beta's `Type.isTupleType()` answers true for tuple references
that 7.0.2 answered false,
unmasking readonly-preference offers for mutable tuples inside readonly containers

## Symptom

Moving the workspace from `typescript@7.0.2` to `typescript@7.1.0-beta` turned one
`prefer-readonly-parameter-type` fixture from fully silent into exactly one offer.
For
`package/test-fixture/oxlint-no-restricted-syntax/src/readonly-tuple-exposure-invalid.ts`
the rule began reporting:

```text
Parameter "rows" can be deeply readonly: `[number]` uses a mutable tuple.
Verified edit: Wrap the complete authored parameter type with `import('type-fest').ReadonlyDeep<...>`.
```

on the reader control `readStoredPair(rows: readonly StoredPair[])`,
while both writer controls stayed silent.
The offer is sound:
the reader iterates the same receiver and writes nothing,
and the fixture itself declares that offer allowed
("this one is allowed to be offered and the one above is not").
The 7.0.2-era silence came from the classifier never seeing the mutable element tuple at all.

## Root cause

### The classifier early-returns `deep-readonly` when the tuple predicate answers false

```ts
// package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/readonly-collection-classification.ts:174
function classifyTuple({
  checker,
  type,
  classify,
}: {
  readonly checker: Checker;
  readonly type: Type;
  readonly classify: NestedReadonlyClassifier;
}): ReadonlyClassification {
  if (!type.isTupleType())
    return { kind: 'deep-readonly', };
```

When `type.isTupleType()` answers false for a tuple's own representative,
the classifier never walks the tuple's element types,
so a mutable element tuple cannot surface as a cause and no preference offer can be produced.

### The predicate is version-stable; the representative the API hands out is not

`microsoft/typescript-go`'s tuple predicate is identical at tag `typescript/v7.0.2`
(commit `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`)
and on `main` (checked 2026-10-20):

```go
// internal/checker/checker.go:23397 (typescript/v7.0.2)
// internal/checker/checker.go:23603 (main)
func isTupleType(t *Type) bool {
	return t.objectFlags&ObjectFlagsReference != 0 && t.Target().objectFlags&ObjectFlagsTuple != 0
}
```

and the exported `Type.IsTupleType()` method only delegates to it
(`internal/checker/types.go` in both trees).
The predicate answers true for a tuple reference
(a type carrying `ObjectFlagsReference` whose target carries `ObjectFlagsTuple`)
and false for the tuple's own target type.
What the move changed is which representative the API returns for a tuple inside a container type:
7.0.2 handed the plugin the tuple target,
so `isTupleType()` answered false and `classifyTuple` fell through to `deep-readonly`;
7.1.0-beta hands the tuple reference,
so it answers true and the element walk runs.
The 7.1 answer also agrees with what the checker's own reference-aware predicate said all along,
which the migration recorded as an internal inconsistency on 7.0.2.

Measured,
not inferred:
the same fixture source,
the same built plugin logic,
and the same lint invocation produce full silence on 7.0.2 and exactly this one offer on
7.1.0-beta.
The offer's span lands on `readStoredPair`'s `rows` parameter
(asserted by exact source offset in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-type.unit.test.ts`,
test `keeps a receiver-held tuple out of the exposure discharge`).

## Verification

Versions under test:
`typescript@7.0.2` (recorded runs during the 2026-10-20 move) and `typescript@7.1.0-beta`
(npm `beta` dist-tag,
published 2026-10-20T16:45:47.402Z).
Predicate source under `microsoft/typescript-go`,
tag `typescript/v7.0.2` = `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`,
and `main` as checked out 2026-10-20.

Runnable harness on the pinned workspace:

```sh
# /var/home/user/Monochromatic
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit
```

Failing catalog,
as the 7.0.2-era expectation,
now the red state the move had to fix:

- `readStoredPair(rows: readonly StoredPair[])` with `type StoredPair = [string, string]`:
  no diagnostic,
  because `isTupleType()` answered false and the classifier returned `deep-readonly`;
- `rewriteMutableStoredPair(rows: StoredPair[])` and `rewriteStoredPair(rows: readonly StoredPair[])`:
  also silent,
  and here the silence is correct,
  kept by the write attribution pin
  (`referentMutated=[0]`,
  `package/oxlint-plugin/prefer-readonly-parameter-type/src/effect-summaries.unit.test.ts`,
  `keeps a write through an iterated tuple attributed to its parameter`).

Working catalog,
`typescript@7.1.0-beta`:

- `readStoredPair` receives exactly one sound preference offer,
  attributed by span to its `rows` parameter;
- both writers stay silent with the same attribution pin,
  because `pair[0] = 'rewritten'` reached through `rows.values()` mutates `rows`;
- the tuple element the classifier now walks reports as `[number]`-shaped mutable tuple state in the
  diagnostic text.

## Verified workarounds

### Accept the 7.1 answer and attribute offers by span (applied)

No suppression is needed:
the unmasked offer is the correct verdict for the reader,
and the writers' silence is what the attribution pin guarantees.
The fixture tests now assert one span-attributed offer instead of vacuous silence,
so an empty diagnostic list can never satisfy them again.
Tradeoff:
any test or downstream logic written against 7.0.2's silence must be re-read against
`isTupleType()` now being reference-aware.

### Ask the reference for its target when the old answer is required (not applied)

Code that genuinely wants the 7.0.2-era "is this the tuple target itself" reading can compare the
type against its reference target through the 7.1 API.
Tradeoff:
restoring the old early-return shape would again hide mutable tuples inside readonly containers,
which is exactly the unsound silence the move unmasked;
the plugin keeps no such special case.

## What does not work

- Treating the 7.0.2 silence as a soundness property:
  it was a fall-through of a predicate that answered false for tuple representatives,
  and the fixture's writers are withheld by write attribution,
  not by that silence.
- Forcing the 7.0.2 answer by special-casing `isTupleType()` in the classifier:
  measured,
  with the tuple positions recursed into and the early return bypassed,
  `rewriteMutableStoredPair` becomes offerable while its body runs `pair[0] = 'rewritten'`
  (recorded during the move;
  the fixture comment in `readonly-tuple-exposure-invalid.ts` keeps the warning).
- Reading the offer as a regression to remove:
  the offer's edit wraps the authored parameter type in `ReadonlyDeep`,
  which the reader's body supports.

## Upstream filing decision

`.out-of-scope/` holds no typescript-go or checker-API exemption
(checked `bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
and `typescript-project-references.md`).
Nothing to file and nothing to add:
the 7.1 answer matches the checker's own predicate,
and the observable change is a corrected representative rather than a defect.
No draft issue or comment is kept.

1.  Really upstream's fault?
    Not a defect.
    7.0.2's answer disagreed with the reference-aware predicate;
    7.1.0-beta agrees with it.
2.  Can upstream fix it?
    Not applicable;
    nothing needs fixing.
3.  Are they supporting this use case?
    Yes;
    the predicate and the tuple reference API are the documented shape of tuple types in the 7.1
    API surface.
4.  Would the repo welcome our contribution?
    Not evaluated,
    because nothing is proposed and opening an external issue needs the user's authorization.
5.  Will they likely fix it?
    Not applicable;
    the behavior is intended.
6.  Have we prototyped a minimal fix compatible with their architecture?
    Not applicable;
    consumer code changed instead,
    in `prefer-readonly-parameter-type`'s fixture expectations.
