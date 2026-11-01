# Node 26.8.2 root-input JSON omits definition-order Set members

## Symptom

The retained Task47 artifact contains `freeOrder: { "source": {}, "target": {} }`
for every queried registration.
The file is valid JSON,
so the byte comparison and byte/JSON reader report no serialization error.
Those checks do not establish that runtime collection contents survived persistence.

The measured artifact has 42 queried registrations.
Nine have nonempty definition-node inventories,
but their persisted order-exemption fields still contain empty objects.
Task67's transport repair qualifies a corrected artifact for task63's remaining DTO decoding work.
Do not interpret the original empty objects as empty exemption sets or grant plan authority to either artifact.
Frozen originals remain retained unchanged.

## Root cause

The repository excerpts in this section refer to pre-repair commit `081c05e14`.

`package/module/translation-repair/src/preparation-root-registration-model.ts:93` (removed before 2026-10-18)
uses the in-memory question type in a supposedly serializable DTO:

```ts
// package/module/translation-repair/src/preparation-root-registration-model.ts (removed before 2026-10-18)
readonly freeOrder: BlockPairingQuestion['freeOrder'];
```

The referenced `FreeOrderBlocks` at
`package/translation-repair/stage/src/pair-blocks-wire.ts:274` contains sets:

```ts
// package/translation-repair/stage/src/pair-blocks-wire.ts
readonly source: ReadonlySet<number>;
readonly target: ReadonlySet<number>;
```

`definitionIndexes` in `package/translation-repair/stage/src/pair-definition-order.ts:85`
constructs a set of local node positions.
The registration constructor at
`package/module/translation-repair/src/preparation-root-registration.ts:152` (removed before 2026-10-18)
retains those runtime objects directly:

```ts
// package/module/translation-repair/src/preparation-root-registration.ts (removed before 2026-10-18)
freeOrder: numbered.freeOrder,
```

The application at
`package/module/translation-repair/src/corpus-run/producer-prepare-app.ts:206` (removed before 2026-10-18)
passes `JSON.stringify(inputs)` to its output writer without a collection transformation.
Node does not encode Set members through ordinary JSON object properties.

The deciding source is Node `v26.8.2`,
commit `f2f2c2f246c36bd74f082cb43ecfe830657d81c9`,
read without modifying the source clone.
V8's `deps/v8/src/json/json-stringifier.cc:1379` emits an empty object when there are no own descriptors:

```cpp
// nodejs/node: deps/v8/src/json/json-stringifier.cc
if (map->NumberOfOwnDescriptors() == 0) {
  AppendCStringLiteral("{}");
  return SUCCESS;
}
```

Its slow object path at line 1466 obtains enumerable own string keys:

```cpp
// nodejs/node: deps/v8/src/json/json-stringifier.cc
KeyAccumulator::GetKeys(isolate_, object, KeyCollectionMode::kOwnOnly,
                       ENUMERABLE_STRINGS,
                       GetKeysConversion::kConvertToString)
```

The defect is the repository's DTO choice,
not a Node promise that sets serialize as arrays.

## Verification

The original artifact has 2821409 bytes and SHA-256
`12c4f304dad2a7039c5452ae7f3de5a84160f759e27968e05e47494c0f3eb75a`.
Its observed field shapes and definition domains are retained in package
`node_modules/.monochromatic/preparation-input-reader/free-order-observation/persisted-reading.json`.
The observed input collections are not a current acquisition receipt or reviewed plan.

Native Node `26.8.2` controls distinguish the representations:

```js
// Provider-free serialization control.
JSON.stringify({ source: new Set([1, 3]), target: new Set([2]) });
// Observed: {"source":{},"target":{}}
JSON.stringify({ source: [1, 3], target: [2] });
// Observed: {"source":[1,3],"target":[2]}
```

The exact control strings,
observation hash and retained V8 source are in the same private evidence directory.
The array control preserves members;
the set control loses them.
No provider call or corpus edit is involved.

## Repair ownership and limits

Keep `FreeOrderBlocks` as sets for existing in-memory pairing interpretation.
The root registration DTO must instead own explicit ordered numeric arrays,
materialized from those sets before JSON persistence.
This changes unpublished transport data,
not substantive pairing messages,
model identities,
provider order,
quorums or definition authority.

The existing responsibilities stay separate:

- `definitionIndexes` selects definition positions from native node zones.
- `blockPairingQuestion` owns in-memory order-exemption sets and the historical question key.
- Root registration owns the serializable projection.
- Task47 owns exclusive persistence and retained file identity.
- Task63 validates the corrected persisted DTO.
- Later pairing interpretation owns any array-to-set conversion it actually needs.

The JSON writer is a production consumer of the complete root DTO.
Source search finds no downstream production code semantically interpreting `registry[*].freeOrder` yet.
The existing replay and evidence-occurrence consumers use `BlockPairingQuestion.freeOrder`,
not the root DTO;
they must retain their current set behavior.

The native-root controls in `devtest-vbkJNR` fail at two designated ordinary assertions
for asymmetric and empty definition indexes.
`b9108503a` implements the explicit array projection.
Its first fresh builds,
types and regression tests pass,
including whole-root JSON round-trip equality.
Lint then identifies the inline round trip as a deep-clone idiom;
the tests now write and reread actual JSON files rather than substitute `structuredClone`,
which would not test serialization.
R2 passes fresh builds,
bootstrap sealing,
types,
targeted tests,
full-scope lint and application sealing.
`devfull-KdsCj2` passes with 635 expected and observed Node `26.8.2` test-entry processes,
not 635 test cases.
`mutation-proof-1ogGxE` detects the source/target erasure,
side-swap,
runtime-set and empty-object mutations after successful builds;
baseline/restored tests pass with real source and `dist` unchanged.
Do not silently relabel the frozen artifact or derive a new expected digest from it.

## Current artifact qualification

Application runtime `frozen-runtime-uByTgd` contains 167 manifest-listed files.
Its 28930-byte manifest SHA-256 is
`fe4076e23fb763cb91baba1d871e79fd49852d2f6aab3ee8078666fdc024d1e5`.
The fixed `WWrgVV` bootstrap and platform profile remain unchanged.
The new 1550-byte base launch has SHA-256
`089666ae58251f5de92675fcab91810d512fecb9a0df86540225c99ebd326fce`.

`native-proof-qTMylB` retains matching and deliberately mismatched-reference runs.
Both freshly persisted artifacts contain 2821531 bytes with SHA-256
`475506126e6e990014b64ff2d4a694e099645e2b396efff690de7908449763a0`.
All queried records contain explicit order arrays;
the same nine definition-bearing parents contain nonempty index domains.
The mismatch retains the useful new output.
Both child container names are independently absent,
and captured stderr contains no bare shutdown-cancellation diagnostic.

The expected artifact is an explicit verification-only derivation from the old artifact's represented
population node order/zones,
definition-node IDs and numbered-block content hashes.
Only the `freeOrder` representation is replaced.
The actual new output matches those complete expected bytes.
This does not decode missing Set members,
reconstruct omitted bodies or grant semantic review.

`after-state-jDNGyO` rehashes the old `fOagX0` and R8 comparison runtimes,
the new runtime,
original and new artifacts,
frozen selection/support,
executable bindings and the owner lock after native and mutation runs.
Its 569 checked file identities match.
This after-state evidence supports preservation;
write-target selection alone would not.

The evidence roots are under package `node_modules/.monochromatic/preparation-root-order/`.
[Repository issue #541](https://github.com/Aquaticat/Monochromatic/issues/541) remains open;
the changes have not been reviewed.
Durable archive `preparation-root-order-retention/proof-HFMTuC` copies and hash-matches
7030 regular files totaling 116347762 bytes,
with 1590 directories,
566 recorded symlinks and no special entries.
It retains current source/artifact snapshots and original CID bytes.
At `2026-09-15T06:55:18.112Z`,
all twenty-seven selected stopped diagnostic containers were removed without force or prune;
all seventy-seven unselected containers remained present.
Final documentation/issue readbacks and their scoped check-container cleanup are supplemental records.
The transport qualification does not complete task63's remaining DTO field decoders.

## Rejected remedies

- A decoder that treats `{}` as an empty set invents absent interpretation data.
- A generic JSON replacer conceals the DTO's runtime-only field type and broadens serialization behavior.
- Reconstructing missing fields while calling them decoded data hides the persistence defect.
- Reusing the old file's byte-parity result as semantic validation overstates that proof.
- Editing frozen artifacts would destroy their bound evidence.

## Upstream filing decision

The `.out-of-scope/` topic inventory contains no matching Node JSON/Set entry.
No Node filing or patch is proposed.
The tracker search found the historical comparable
[Node #10965](https://github.com/nodejs/node/issues/10965),
where JSON-backed IPC omitted `Map` members.
That thread was read in full;
its IPC discussion is not evidence about the current preparation CLI or current IPC options.
The local representation fix adds nothing to that Node thread.

1.  Upstream fault:
    no;
    the consumer stores runtime sets in JSON-bound data.
2.  Upstream fixability:
    no Node change is needed for an explicit DTO projection.
3.  Supported use case:
    the measured native object serialization does not promise Set-member preservation.
4.  Contribution policy:
    not asserted because there is no proposed upstream contribution.
5.  Expected upstream action:
    not asserted;
    ordinary serialization behavior is not an upstream refusal incident.
6.  Prototype:
    the native array control and repository projection preserve indexes
    in the listed native and mutation checks.
    No upstream patch is needed.

Upstream filing artifact:
nothing to add to Node.
The repository defect is tracked separately with the changes explicitly unreviewed.
