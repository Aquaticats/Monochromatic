# Strict persisted preparation input reader

Abandoned on 2026-09-16.
The reader this record specifies was removed in `cbedea357`:
no pass or probe entry reached it.
The owner authorized the teardown;
the decision is
[the teardown decision](../decision/translation-repair-provider-free-layer-teardown-2026-09-16.md).
This record stays as history of what was tried and why it was set aside.

## Ownership and sequence

Task52 is split into independently verifiable implementation owners:

- Task63 decodes the complete persisted input DTO.
- Task64 materializes finite root registrations and potential invocation slots.
- Task65 materializes initial and registered conditional post-relabel phases.
- Task66 owns exclusive persistence,
  separate review gates and independent reconstruction.

Task52 retains integration acceptance before task53 acquisition journals.
Task51 and task62 qualify the existing persisted-input comparison;
they do not approve root/phase contents or paid work.

## Transport repair prerequisite

Task67 supplies the corrected transport representation for the complete decoder.
The original retained Task47 file has empty objects in both `freeOrder` sides of all queried registrations.
Nine queried registrations retain definition-node IDs but no serialized order-index members.
The in-memory DTO incorrectly carries `ReadonlySet` values into plain JSON output.
The [serialization incident](../troubleshooting/translation-repair-root-order-serialization.md)
records native evidence and the transport-owner repair.

Do not reinterpret `{}` as empty sets or hide a reconstruction behind a decoding claim.
Byte equality and the successful byte/JSON stage remain their original bounded proofs,
not evidence that every runtime field survived serialization.
The original frozen artifact stays unchanged.
Task67's corrected artifact is separately qualified with explicit order arrays and SHA-256
`475506126e6e990014b64ff2d4a694e099645e2b396efff690de7908449763a0`.
Its native,
full-suite,
mutation and preservation records remain distinct from the unfinished complete DTO decoder.

## Reader shape

Use a DTO-specific closed decoder,
not a generic schema framework or a partial typed assertion.
Rebuild every supported `PreparationRootInputs` field from owned parsed values.
Freeze returned objects and arrays,
preserve array order,
and retain observed raw-byte extent and SHA-256 separately from the decoded evidence.
Neither field grants review or admission.

The byte-owning I/O caller must enforce its authorized extent before decoding.
`package/module/translation-repair/src/corpus-run/producer-input-bounds.ts` (removed before 2026-10-18) defines a
metadata ceiling,
not a root-input-body ceiling;
it cannot reject the retained root artifact merely because that artifact exceeds metadata size.
Do not invent an existing artifact limit.

The public reader must copy genuine byte views before interpreting them,
reject proxies/detached buffers through owned fixed errors,
use strict UTF-8 decoding,
and retain no parser excerpt or foreign error cause.
The native writer passes compact `JSON.stringify(inputs)` to the UTF-8 output writer:
`package/module/translation-repair/src/corpus-run/producer-prepare-app.ts:206` (removed before 2026-10-18)
and `package/module/translation-repair/src/corpus-run/producer-input-output.ts:84` (removed before 2026-10-18).
A serialization-grammar check must match those actual bytes,
not an assumed pretty-printed format.

Known evidence-only fields are still fully decoded.
Leaving them as `unknown` would let later consumers mistake unchecked evidence for a complete DTO.
Task64 may take an internal consumed-field projection after its owning entry decodes fresh bytes;
a previously returned typed object is not an authenticated certificate.
No additional persisted projection or general executor is needed.

## Relationship constraints from current source

`package/module/translation-repair/src/preparation-root-reference-model.ts` (removed before 2026-10-18) owns the closed
unversioned DTO.
Require its exact scope and key inventory;
do not add or infer a `version` field.
Scalar,
discriminant,
array,
digest,
coordinate and nested-record validation precede typed construction.

Validate unique identities and complete relationships where the DTO actually represents them.
Do not conflate these collections:

- `parents` contains the ordered frozen writer selection.
- `registry` also contains definition-only dependencies.
  A registry member need not belong to `parents`.
  Resolve parent identity through the represented population and entry domains,
  preserving its independent roles.
- `unalignedDefinitions` deliberately retains nodes outside aligned parents.
  It must not be required to resolve to a registered definition parent.
  The separate Xing namespace must remain valid without widening acquisition scope.
- `questionAliases` groups exact serialized questions in registry order,
  not merely equal digests.
  Reuse that substantive identity when checking alias membership.

These distinctions follow
`package/module/translation-repair/src/preparation-root-registration.ts:79` (removed before 2026-10-18),
`:99`,
`:173` and `:223`.
A scoped design review's suggestions to resolve all registry records through selected parents
and all unaligned definitions through registered definition parents are rejected by this source evidence.

Selected-entry text can validate the hashes and coordinates it actually carries.
Raw-document and supporting-artifact identities do not supply omitted bodies.
Do not reconstruct those bodies,
open their locators,
relist the corpus or invoke another input runner from the reader.
Do not impose the current selection census as a general DTO schema constant;
the owning root plan separately binds the frozen selection.

## Verification contract

Use freshly built decoder artifacts and cat-authored schema fixtures.
Cover every field family,
including malformed fields not consumed by initial planning.
Cover extra/missing keys,
discriminant drift,
duplicate identities,
relationship drift,
UTF-8/JSON refusal,
caller mutation,
byte-view overrides and detached/proxy inputs.
Diagnostics must not expose input text or native parser details.

The real retained Task47 artifact must decode through the built interface.
Its reader import closure must not acquire files,
spawn processes,
construct providers or transmit requests.
Canonical checks,
types,
hashes and round trips remain syntax/consistency evidence,
not semantic review or acquisition approval.

## Implementation checkpoint

The initial internal byte/JSON stage copies native byte views,
uses strict UTF-8 decoding,
and compares reserialized JSON against original copied bytes.
The byte comparison rejects a stripped BOM as well as duplicate keys and non-native spellings.
A dedicated inert build entry keeps its acquisition-free import closure independently inspectable.
The parsed value deliberately remains `unknown`.
Complete DTO decoding,
relationship validation,
final immutable projection and their qualification remain task63 work;
this internal stage is not a completed root-input reader.

`54a8bca9b` introduces that stage and its built-artifact controls.
R1 fresh normal/bootstrap builds,
types and the JSON/root/message-vocabulary tests pass.
Full-scope lint then reports thirteen warnings and no errors,
covering layout,
shadowing and hexadecimal spelling.
`76105d223` addresses those findings.
R2 passes builds,
types and tests but reports one remaining chain-layout warning.
`0085cb002` names the canonical encoded bytes;
R3 passes fresh builds,
types,
targeted tests and full-scope lint.

The isolated `json-native-consumer-GH3Qio` consumes the actual 2821409-byte Task47 artifact
through the frozen built byte/JSON stage without repository or dependency mounts.
It preserves the recorded raw SHA-256 after caller-buffer mutation.
The parsed value remains unknown;
observing its scope and collection extents is not complete DTO validation.
The DTO-specific closed-record,
scalar and ordered-element helper layer passes R2 builds,
types,
targeted tests and full-scope lint after its recorded fixture/layout remediation.
The selection-projection decoder and its nonblank-text cases are the current checkpoint;
other DTO field families and complete relation validation remain absent.

A source-copy command timed out during that checkpoint.
The retained draft was intact while the new untracked target was observed empty;
the process snapshot contained no surviving `cp` or `rg`.
The target was restored through an explicit complete write,
then both files matched SHA-256
`db60fd53b166e5d9308b51f0cdd664ed87f760c0dec8274eb010394e2095cdbf`.
This records recovery,
not a diagnosed cause for the command timeout.
The proof roots and raw diagnostics are durable under package
`node_modules/.monochromatic/preparation-input-reader/`.
Following task67,
reference attribution,
raw-document identity and exclusion-field decoders are added with common entry/discriminant validation.
They preserve omitted-body identities as data and do not open their locators.
Their R1 builds,
types,
targeted tests and full-scope lint pass.
The actual-artifact consumer then refuses `inputs.selection.selectionRuntimeDigest`:
the new selection decoder incorrectly expected a bare SHA-256.
`package/module/translation-repair/src/read-frozen-preparation-selection.ts:50` (removed before 2026-10-18)
and lines 175 to 178 require `sha256-tree-v1:` followed by the digest.
The decoder and fixtures now preserve that original tagged grammar;
the valid retained artifact is not edited to satisfy the mistaken reader.
R2 rebuilds first and then passes the actual-artifact check before the remaining checks.
`reference-native-uqYcb1` preserves the selection,
reference,
raw-document and exclusion subtrees of the corrected artifact through the built decoders.
Types,
targeted tests and full-scope lint also pass.
The consumer uses a read-only dependency overlay;
it is not standalone import-closure or whole-DTO qualification.
Node-coordinate,
parent-side and protection R1 passes:
`node-native-d9yTV7` exercises every population and selected-parent side plus unaligned-definition nodes.
Fresh build,
bootstrap,
types,
targeted tests and full-scope lint pass.

Complete-entry,
policy,
retained-line and finding decoders are the next checkpoint.
Their relation checks use only represented texts:
separate content hashes,
shared invisible-character folding for retained origins,
line order and target reconstruction,
local evidence bounds and the producer's whole-page exclusion boundary.
They do not remove stubs again,
classify declaration prose,
reparse MDX or infer unaligned section coverage from registered parents.
Entry R1 passes the real artifact through all complete-entry fields:
`entry-native-0o4Al6` reconstructs 30 entries and checks 4531 frozen objects and collections.
Build,
bootstrap,
types and targeted tests pass;
Oxlint reports 12 warnings and one `tsdoc(require-returns-check)` error.
These are corrected before the next verification.

A scoped reviewer identified missing span-order and note-format checks plus regression gaps.
`archive-original-note.ts:259` bounds seals by the next heading,
and its span filter removes later covered intervals;
`entry-notes.ts:83` owns canonical folded-note spelling.
The decoder now rejects reversed,
overlapping or covered spans and noncanonical note whitespace without sorting or rewriting them.
Adjacent spans and an isolated zero-width end-of-document span remain valid.
New tests cover explicit retained stub-looking text,
non-reclassification of note meaning,
malformed MDX text,
and every current invisible-character fold without shifting line origins.
This scoped review is not whole-change acceptance.

Registration and whole-DTO relation work remain pending.
Full-root tests must preserve alignment attachments outside registered parents;
closure checks must exclude the stub remover,
MDX parser and original-note classifier.

Entry R2 passes build,
actual-artifact consumption,
bootstrap,
types and tests;
only five `unicorn(escape-case)` fixture warnings remain.
R3 corrects those spellings and passes the complete checkpoint,
including full-scope lint.
Its retained consumer is `entry-native-XCVCvG`.

Full population-parent and selected-parent decoders are the next checkpoint.
They preserve the source-index alias independently of combined pair position,
check canonical parent identity and ordered target-local protection membership,
and verify represented parent text and node hashes.
`preparation-root-parent.ts:83` retains whole overlapping declaration intervals,
not clipped intervals.
Empty target anchors therefore retain the producer's overlap predicate and never become all-sealed controls.
`document-node.ts:232` and `container-extents.ts:285` tie node hashes to exact text slices.
Population-only rows do not gain omitted-body reconstruction.
Parent R1 passes the actual artifact and all build,
bootstrap,
type and test stages;
its callback-layout warning is corrected before R2.
Parent R2 passes the full checkpoint.
`parent-native-Ni0uqi` decodes 272 population rows and 40 selected parents,
checking 8364 frozen objects and collections.

### Native question baseline and adjacent key encoding issue

`question-baseline-QWeptI` records eleven pre-extraction native key cases and their complete protocols.
The cases cover empty sides and blocks,
side reversal,
Unicode and astral text,
NUL,
quotes,
backslashes,
newlines and listing fences.
Repeated protocol construction is value-equal,
JSON-representable and object-disjoint;
each measured protocol graph contains fifteen objects or arrays.
No provider fetch occurs.

The two embedded-NUL cases have different block boundaries but the same legacy key.
Their complete question digests differ.
This is ambiguous pre-hash encoding,
not a SHA-256 collision.
[Issue #542](https://github.com/Aquaticat/Monochromatic/issues/542)
records the source-traced cache consumer and the missing cached-preparation replay.
It remains open and unreviewed;
no real corpus impact or cache-encoding repair has been demonstrated.
The reader's planned key-helper extraction preserves existing bytes,
while question alias accounting continues to use exact complete question bytes.

The issue was created successfully.
Its first CLI readback failed because GitHub CLI sanitizes response JSON,
including literal control escape text.
An independent raw public HTTP response matches the submitted body exactly.
The readback mismatch required no remote content repair or duplicate creation.
A separate issue-draft lint check requires digest-label line wrapping;
preserve original submission and response evidence apart from that editorial correction.
See [GitHub CLI JSON control escapes](../troubleshooting/github-cli-json-control-escapes.md).
Durable records remain in `legacy-key-observation/` and the separate `gh-json-observation/`
under the reader evidence root.

The shared `blockPairingQuestionKey` extraction is committed and used by production.
Its direct and delegated tests retain all pre-extraction golden outputs.
Key R1 passes build,
native parity,
bootstrap,
types and tests;
its fixture-local offset declaration is then scoped through a numeric fold.
Key R2 passes the complete checkpoint,
including full-scope lint.
`question-key-native-4zLMwl` matches all eleven original result records and protocols.
The legacy ambiguity is preserved rather than silently repaired.

`gh-doc-check-jDLrKG` validates the diagnosis,
hub,
then-current planning record and formatted issue draft.
The editorial issue update succeeds without a repeated mutation.
Its immediate public GET still contains the old body;
that response's cause is not diagnosed.
A subsequent fresh public read matches the formatted body exactly and confirms #542 remains open.
Both responses and the original submission remain retained.

Explicit numbered-block and complete-question decoders are now the next checkpoint.
The protocol reader walks supplied JSON against freshly generated native values,
checks closed object keys and array extent/order,
and freezes only the factory-owned result.
It returns no unknown supplied subtree and does not interpret JSON Schema or duplicate messages.
Question R1 passes build,
actual-artifact consumption,
bootstrap,
types and targeted tests.
`question-native-10Otpk` decodes all 42 queried questions,
checks their historical keys and complete question digests,
and observes 1555 frozen objects with zero provider fetches.
Full-scope lint reports nine warnings;
layout,
string spelling,
symbol description and an unknown-typed fixture traversal are corrected.
A sequential native-clone fault test is added to detect protocol fields that would disappear from JSON.
Question R2 passes the complete checkpoint,
including the native-clone fault control and full-scope lint.
`question-native-AVY4wd` repeats the actual-artifact question,
key,
digest and freezing checks.

Definition-domain,
corrected order-array,
question-alias and nonempty unaligned-namespace decoding is the next checkpoint.
Canonical `block/N` validation is shared with the existing node decoder rather than copied.
Definition IDs remain distinct from local numeric indexes;
source and target identity domains remain independent.
Alias membership and namespace coverage still require the whole-root relation checks.
Domain R1 passes build,
actual-artifact consumption,
bootstrap,
types,
focused tests and full-scope lint.
`domain-native-VakiX2` reads 45 definition domains,
42 order records and the nonempty namespace's nine nodes.
The actual alias collection is empty;
synthetic positive tests exercise alias-group decoding.

Complete registration variants are the next checkpoint.
Their identity reader checks native role order and definition responsibility,
while the queried branch checks native dispatch cardinality,
shared historical key encoding,
complete question digest,
and side-local definition-index counts and bounds.
Empty and implicit records reject question or outcome fields.
Selection membership,
complete-entry hashes and exact node-to-local-index correspondence remain whole-root checks.
Registration R1 passes build,
actual consumption of all 45 records,
bootstrap,
types and tests.
`registration-native-GyLXfS` retains 42 queried and three implicit records,
with 1907 frozen objects and zero provider fetches.
The synthetic suite covers empty dispatch.
The readonly-parameter lint finding identifies an unnecessarily broad question parameter
whose schema type remains writable.
The helper now receives only primitive source/target block counts,
not protocol data;
no mutation annotation or schema-type weakening is introduced.
Registration R2 passes the complete checkpoint.
`registration-native-7rJcFl` repeats the actual 45-record and deep-freezing checks.

### Whole-root relationship implementation boundary

The final byte-reader result will reuse `DeepReadonlyData` for its DTO view,
while retaining runtime freezing checks;
the annotation alone is not immutability evidence.
All root relationships are limited to represented data,
not independent corpus reconstruction or semantic review.

A scoped review confirms these required distinctions:

- Raw sides follow listed-entry order and source-before-archive order,
  with missing-side exclusions rather than fabricated empty documents.
- Selected entries and parents retain their different native and frozen ordering.
- Every represented-entry population parent,
  not merely selected parents,
  needs text,
  node-hash and protection consistency checks.
- Registry roles,
  domains,
  dispatch and local numbering derive from selection and population,
  never from registry claims alone.
- Unaligned nodes are absent by identity from aligned parents,
  not necessarily geometrically disjoint.
- Alias groups derive from exact question bytes before hashing.
- The selection reference projection contains `path` and `hash`,
  not per-reference `bytes`.
  The review suggestion to compare selection reference extents is therefore inapplicable;
  no absent field is invented.

Protection projection should have a shared pure geometry owner rather than importing the note classifier.
Existing production parent/node projection internals are exposed through their owned barrel under `@internal`
so native pre-extraction geometry outputs can be captured without a test-only API.
The actual selected entries have no original-policy spans;
only one unselected population parent has intersections,
and the observed population has no zero-width nodes.
Those measurements cannot substitute for synthetic positive geometry controls.
Geometry extraction is now implemented.
`sealed-node-ids.ts` owns the pure sealing geometry and re-exports it from
`archive-original-note.ts` so existing imports are unchanged.
`preparation-root-protection.ts` owns the complete protection projection,
and `preparation-root-parent.ts` delegates to it.
Pre-extraction native outputs for twelve geometry cases are retained in
`geometry-baseline-weNIba`,
including its deciding source bytes and consumed entry identity.
`preparation-root-protection.unit.test.ts` compares the direct projection,
the delegated parent constructor and the population projection against those goldens,
covering all-sealed,
partial,
boundary-touching,
zero-width-end,
overlapping-declaration,
empty-target and empty-anchor-overlap cases.
That golden test file is committed but has not yet been executed through a package checkpoint;
the next checkpoint must include it in the targeted test list before any further reader work.
Whole-root relationships and the public byte-reader result remain unimplemented.
No completed decoder,
semantic qualification or plan materialization is inferred from this checkpoint.
