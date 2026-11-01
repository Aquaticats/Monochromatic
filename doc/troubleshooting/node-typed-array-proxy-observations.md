# Node 26.8.2 byte-view proxies escape consumer input diagnostics

## Symptom

The translation-repair supporting-artifact reader initially allowed native or caller-controlled failures
instead of its fixed `PreparationRootError` input diagnostic.
The R6 regression suite at `e359a7d53` records ordinary assertion failures for these inputs:

- A byte-view proxy's prototype trap throws `Error: q7z9k2` during `instanceof`.
- A byte-view proxy's iterator getter throws the same private canary during byte copying.
- A revoked inventory or descriptor produces
  `TypeError: Cannot perform 'IsArray' on a proxy that has been revoked`.
- A revoked byte view produces
  `TypeError: Cannot perform 'getPrototypeOf' on a proxy that has been revoked`.

The canary is synthetic fixture data,
not corpus content or a credential.
This is a consumer diagnostic-boundary defect,
not a finding that Node should suppress caller exceptions.

## Root cause

At `e359a7d53`,
`package/module/translation-repair/src/read-preparation-selection-evidence.ts:33` (removed before 2026-10-18)
uses prototype-sensitive admission before its copy exception boundary:

```ts
// package/module/translation-repair/src/read-preparation-selection-evidence.ts (removed before 2026-10-18) at e359a7d53
if (!(content instanceof Uint8Array))
  throw new PreparationRootError({ kind: 'reference-content' });
try {
  return new Uint8Array(content);
}
catch (error) {
  if (!(error instanceof TypeError))
    throw error;
  throw new PreparationRootError({ kind: 'reference-content' });
}
```

Consequently,
throwing prototype traps bypass the catch,
and throwing iterator getters carrying an `Error` pass through its non-`TypeError` branch.
The same reader's inventory and entry `Array.isArray` observations were outside the descriptor-property catch.
R6 captures each distinct native operation in its failure stack.

Node source commit `f2f2c2f246c36bd74f082cb43ecfe830657d81c9`,
`lib/internal/util/types.js:18`,
provides a different observation:

```js
// Node lib/internal/util/types.js
function isUint8Array(value) {
  return TypedArrayPrototypeGetSymbolToStringTag(value) === 'Uint8Array';
}
```

The captured primordial getter is not a read of the input's own `Symbol.toStringTag` property.
At the same commit,
`deps/v8/src/builtins/builtins-typed-array-gen.cc:646`
defines `TypedArrayPrototypeToStringTag`.
Its heap-object branch dispatches on the engine's element kind,
with an undefined fallback:

```cpp
// Node deps/v8/src/builtins/builtins-typed-array-gen.cc
TNode<Int32T> elements_kind =
    Int32Sub(LoadElementsKind(receiver_heap_object),
             Int32Constant(FIRST_FIXED_TYPED_ARRAY_ELEMENTS_KIND));
Switch(elements_kind, &return_undefined, elements_kinds, elements_kind_labels,
       kTypedElementsKindCount);
```

The deciding Node API documentation,
`doc/api/util.md:3748`,
says `isUint8Array` returns true for a built-in `Uint8Array` instance.
The current installed declaration,
`node_modules/@types/node/util/types.d.ts:493`,
narrows `unknown` to `Uint8Array`.
The runtime probe verifies proxy behavior rather than inferring it from that declaration.

## Verification

Runtime:
Node `v26.8.2`.
The retained source clone is
`/var/home/user/temp/agent/node-json-diagnostic-20260912`,
at the commit named in Root cause.
It is sparse;
source inspection uses `git show HEAD:<path>`,
not missing working-tree files.

Retained evidence:

- `/var/home/user/temp/agent/native-byte-brand-probe-20260913.json`
- `/var/home/user/temp/agent/native-byte-brand-source-20260913.txt`
- `/var/home/user/temp/agent/preparation-supporting-bytes-focused-r6-20260913.out`
- `/var/home/user/temp/agent/preparation-supporting-bytes-runtime-r6-20260913.jsonl`

The probe script is
`/var/home/user/temp/agent/probe-native-byte-brand-20260913.mts`.
It requires fresh output paths and asserts runtime `v26.8.2`.
Its accepted catalog contains ordinary byte views with a throwing own tag getter,
`Buffer`,
shared-backed views,
subarrays and detached byte views.
Its refused catalog contains byte-view proxies,
revoked byte-view proxies,
clamped arrays,
`DataView` and null.
All brand observations match expectations;
no proxy or tag trap runs.
The regression suite separately proves the traps execute and throw the exact canary when directly observed.
Detached views pass the brand check but fail the subsequent copy,
which remains a separate fixed content failure.

The package regression file is
`package/module/translation-repair/src/preparation-selection-evidence.unit.test.ts` (removed before 2026-10-18).
It imports the built artifact and includes successful `Buffer`,
shared-view,
subarray and ordinary-byte ownership controls.
R6 rebuild and type checking pass before its designated regression failures.
R9 passes build,
the real supporting-artifact consumer,
types,
focused controls,
formatter,
rebuild,
lint,
the full unit suite and Markdown checks at `0d8460433`.
`unit exit 0` was read at line 10383 of
`/var/home/user/temp/agent/preparation-supporting-bytes-unit-r9-20260913.out`.
Its runtime run identity is `422d0eb0-7fb4-42d5-acb7-fe29b3351054`.
R6 remains red evidence,
not a final success record.

## Verified mechanism and consumer remedy

The direct Node probe verifies `isUint8Array` as a non-trapping brand observation for its catalog.
The consumer remedy uses it instead of `instanceof`,
then retains the ordinary native byte copy.
It narrowly catches inventory/entry shape-observation failures through
`preparationArtifactIsArray` in
`package/module/translation-repair/src/preparation-artifact-property.ts` (removed before 2026-10-18).
Hashing and the whole reader are not placed under a generic catch.

The tradeoff is explicit:
byte-view proxies are not accepted as byte containers,
even if they wrap a genuine view.
Genuine native views remain supported.
The API still requires caller-side size bounds and provides point-in-time matching,
not immutable certification.

## What does not work

- Prototype-sensitive `instanceof` admits proxy observation into validation.
- Catching only the byte constructor's `TypeError` does not cover earlier prototype traps
  or arbitrary errors from a proxy's iterator getter.
- Sanitizing only descriptor field reads misses revoked-proxy shape failures.
- TypeScript's `Readonly<Uint8Array>` does not establish a native runtime brand.
- A sparse clone's absent source path does not establish that Node lacks the source.
  Reading the committed tree resolves that inspection failure.

## Upstream filing decision

1.  Upstream fault:
    no.
    The observed failures expose incomplete consumer diagnostics around valid JavaScript operations.
2.  Upstream fixability:
    no upstream change is required;
    Node already supplies the observed brand predicate.
3.  Supported use case:
    the API documentation supports checking built-in byte views.
    Transparent proxy copying is not claimed by this report.
4.  Contribution policy:
    not reached because no upstream change or filing is proposed.
    This report makes no claim about contribution eligibility.
5.  Maintainer intent:
    not inferred.
    Read-only issue and PR searches for `isUint8Array proxy` in `nodejs/node`
    returned no matches on 2026-09-13;
    this is not evidence of a maintainer position.
6.  Prototype:
    the native probe and consumer regression establish the relevant boundary.
    No upstream patch was created or installed.

Nothing to add upstream.
No issue,
comment or external communication was sent.
