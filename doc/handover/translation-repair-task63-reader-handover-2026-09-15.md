# Translation repair task63 persisted-input reader handover

Abandoned on 2026-09-16.
The persisted-input reader it hands over was removed in `cbedea357`:
no pass or probe entry reached it.
The owner authorized the teardown;
the decision is
[the teardown decision](../decision/translation-repair-provider-free-layer-teardown-2026-09-16.md).
This record stays as history of what was tried and why it was set aside.

## Purpose

This handover lets a fresh agent continue [task63](../planning/translation-repair-persisted-input-reader-2026-09-15.md),
the strict persisted preparation-input DTO reader,
without access to the prior session.
It records the verified state,
the one pending verification,
the remaining work in order,
and the standing constraints that produced that state.
The planning record and this handover are canonical;
no prior conversation is available.

Repository:
`/var/home/user/worktrees/translation-repair`,
branch `translation-repair-rebased`.
Package:
`package/module/translation-repair` (moved on 2026-10-18 to the packages under `package/translation-repair`).
Durable uncommitted evidence root:
`package/module/translation-repair/node_modules/.monochromatic/preparation-input-reader/`.
Agent scratch under `${HOME}/temp/agent` is ephemeral and reconstructable only.

Auto-push is enabled;
confirm `git status --short` and remote sync at session start rather than assuming it.
The owner `mise.lock` is intentionally dirty and must never be staged or reverted.
Its SHA-256 is
`6a3dcb5cbacd22f38feb141320f707ca41aa504b01df9197567ea4bf65ba4b12`.

## What task63 is

Build a strict decoder for the persisted `PreparationRootInputs` DTO
written by the Task47 provider-free input runner.
The reader accepts genuine bytes,
never caller object capabilities.
It validates every supported serialized field,
rebuilds a deeply frozen native DTO,
and checks only relations supported by represented data.
It is not corpus reconstruction,
plan review,
acquisition authority or writer approval.

The corrected input artifact is
2821531 bytes,
SHA-256 `475506126e6e990014b64ff2d4a694e099645e2b396efff690de7908449763a0`.
Locate it through
`node_modules/.monochromatic/preparation-root-order/native-proof-qTMylB/verification.json`
as `results[0].child` plus `/output/unqualified-inputs.json`.
The older 2821409-byte artifact with empty `freeOrder` objects is historical evidence only.

Task52 coordinates 63 to 64 to 65 to 66,
then 53 and 41.
Downstream tasks wait;
nothing else is authorized by this record.

## Verified completed state

Every DTO family decoder is implemented,
committed and checkpointed.
Each stage passed fresh normal build,
`bootstrap:seal`,
`lint:types`,
targeted unit tests through fresh built artifacts,
full-scope lint under the bounded one-thread profile,
and an actual-artifact native consumer
(unless noted).
The stage history and evidence live under the durable reader root:

- Bytes and strict compact JSON,
  closed values,
  selection projection,
  references,
  raw documents,
  exclusions.
  Actual-artifact consumer `json-native-consumer-GH3Qio`.
- Tagged selection runtime identities:
  the first artifact consumer refused
  `inputs.selection.selectionRuntimeDigest`
  because the reader assumed a bare SHA-256;
  the producer grammar is `sha256-tree-v1:<lowercase hex>`.
  The decoder and fixtures were corrected,
  not the artifact.
  Reference revalidation consumer `reference-native-uqYcb1`
  reads 40 parents,
  63 references,
  184 raw documents and three exclusions.
- Nodes,
  parent sides and protection metadata.
  `node-native-d9yTV7` covers 272 population parents,
  40 selected parents,
  5553 node appearances and 9 namespace nodes.
- Complete entries,
  original policy,
  retained archive lines,
  parser and alignment findings.
  Producer-traced rules:
  whole-page entries are excluded before entry emission,
  spans exist only for the `spans` classification,
  retained lines join with LF after the shared invisible-character fold,
  and notes carry producer folded spelling.
  `entry-native-XCVCvG` rebuilds 30 entries with 4531 frozen objects.
- Population rows and complete selected parents,
  including canonical identity,
  ordered disjoint protection membership,
  unclipped intersecting declarations,
  and text or node hashes only where prose is represented.
  `parent-native-Ni0uqi` checks 8364 frozen objects.
- The historical question-key encoder was extracted unchanged into
  `src/block-pairing-question-key.ts`
  so production and the reader share one owner.
  Pre-extraction goldens for eleven cases live in
  `question-baseline-QWeptI`;
  post-extraction parity consumer is `question-key-native-4zLMwl`.
- Numbered questions and protocol.
  `preparationInputQuestion` rebuilds zero-based contiguous blocks.
  `preparationInputProtocol` compares every supplied protocol value
  against the existing `blockPairingProtocol` constructor output
  through a private structural walk,
  then returns the frozen factory-owned protocol,
  never supplied subtrees.
  A sequential test intercepts `structuredClone` to reject
  native fields that would disappear from JSON.
  `question-native-AVY4wd` decodes all 42 questions,
  matching legacy keys and complete question digests,
  1555 frozen objects,
  zero fetch calls.
- Definition domains,
  corrected `freeOrder` index arrays,
  alias groups and nonempty unaligned namespaces.
  `domain-native-VakiX2` reads 45 domains,
  42 order records and one nine-node namespace;
  the actual alias collection is empty,
  so alias decoding has synthetic positive coverage only.
- Complete registration variants.
  Identity checks native role order and definition responsibility;
  the queried branch checks dispatch cardinality,
  shared key encoding,
  complete question digest and side-local index counts and bounds;
  structural `empty` and `implicit` records reject question or outcome fields.
  `registration-native-7rJcFl` retains 42 queried and three implicit records,
  1907 frozen objects.
- Protection geometry extraction.
  `src/sealed-node-ids.ts` owns pure sealing geometry,
  re-exported unchanged from `archive-original-note.ts`.
  `src/preparation-root-protection.ts` owns the complete projection;
  `preparation-root-parent.ts` delegates to it.
  Pre-extraction native outputs for twelve geometry cases
  are retained in `geometry-baseline-weNIba`
  with deciding source bytes and consumed entry identity.
  `src/preparation-root-protection.unit.test.ts`
  compares direct,
  delegated and population projections against those goldens.
- Barrel and seam changes:
  `DeepReadonlyData` is marked `@internal` and exported through the reader barrel;
  `preparationRootNode`,
  `preparationRootParent`,
  `preparationRootPopulationParent`
  and `preparationRootProtection`
  are exposed through `preparation-root-barrel.ts` under `@internal`,
  reachable from `dist/final/node/index.mjs`.

Scoped Advisor reviews corrected several assumptions mid-flight;
their accepted findings are folded into the planning record.
A scoped review is not whole-change acceptance.

## The one pending verification

`src/preparation-root-protection.unit.test.ts` is committed
but has not yet been executed through a package checkpoint.
Its imports resolve through the public barrel,
and the extraction moved logic verbatim,
but no build or test run has observed it.
Before any new reader work,
run a checkpoint that includes this file in the targeted test list.
The established pattern:

1. Copy the previous stage generator,
   `harness/prepare-registration-stage.mjs`,
   and adapt it,
   or write a new `prepare-geometry-test-stage.mjs`
   that derives its check worker from `harness/registration-worker.mjs`
   by inserting `'src/preparation-root-protection.unit.test.ts',`
   at the front of the `devtest` file list.
2. Generate `run-geometry-test-check.mjs` and a `verify-geometry-test-r1.mjs`
   from `verify-registration-stage-r2.mjs`
   by the same replacements used by every `prepare-*-stage.mjs`.
3. Execute with Node
   `/var/home/user/.local/share/mise/installs/node/26.8.2/bin/node`.
   The sequence is
   devbuild,
   native,
   devbootstrap,
   devtypes,
   devtest,
   devlint256go1.
   Expect roughly two minutes.
   Read the exact failure before changing anything if it refuses.

The generator scripts and per-stage result JSON files
under the reader root are the working examples;
do not invent a new harness shape.

## Remaining work in order

1. Execute the geometry golden checkpoint described above.
2. Implement the whole-root reader.
   A module such as `src/preparation-input-read-root.ts` decodes the closed root keys
   `scope`,
   `selection`,
   `references`,
   `rawDocuments`,
   `listedEntryIds`,
   `population`,
   `excluded`,
   `entries`,
   `parents`,
   `obligations`,
   `registry`,
   `unalignedDefinitions`,
   `questionAliases`,
   `sectionPairing`,
   then checks cross-record relations.
   The reviewed relation plan is recorded in the planning record under
   "Whole-root relationship implementation boundary";
   honor every listed distinction,
   including:
   - Raw sides follow listed-entry order with source before archive,
     exactly two sides unless the unique exclusion is `missing-corpus-side`.
   - Registry order is selected parents in frozen selection order,
     then definition-bearing unselected parents of selected entries
     in native population order.
   - Registry roles,
     domains,
     dispatch,
     `freeOrder` and block correspondence derive from selection and population,
     never from registry claims alone.
   - Namespace nodes are absent by identity from aligned parents,
     not geometrically disjoint;
     do not infer inventory completeness without reparsing.
   - Alias groups derive from exact `JSON.stringify` of decoded questions
     in registry order,
     grouped before hashing,
     only where more than one occurrence exists;
     equal historical keys with different question bytes must not alias.
   - Reference bindings:
     the first three references carry their mandatory selection roles exactly once each,
     frames bind exactly once per selected entry,
     parent notes exactly once per selected parent,
     optional entry note zero or once;
     consumers stay within those identity sets.
   - The selection reference projection carries `path` and `hash` only.
     Never invent a per-reference `bytes` field.
   - Registry `sourceHash` and `targetHash` are complete-entry hashes,
     not parent-side hashes.
   - `sectionPairing` stays `'not-registered'`.
3. Add the public byte-reader result:
   bytes in,
   observed identity plus `DeepReadonlyData<PreparationRootInputs>` out,
   with a runtime deep-freeze walk as evidence.
   The annotation alone is not immutability proof.
4. Whole-root unit tests and an actual-artifact native consumer
   covering every root relation and the frozen result tree.
5. Full suite run,
   mutation guards with successful builds and designated assertions,
   import-closure and standalone-closure checks,
   and ordinary positive lint controls.
6. Documentation updates
   (the planning record,
   the package README where the reader becomes user-visible),
   exact evidence retention of the reader root,
   and removal of only owned stopped diagnostic containers,
   preserving every unselected container ID.
7. Close task63 only within that full checked scope,
   then proceed to task64.

## Standing constraints

- No paid model invocation,
  no corpus edits,
  no writer or plan approval,
  no live root or phase materialization.
  The corpus is UNLICENSED;
  do not edit or commit passages.
- Strict decoding only:
  values stay `unknown` until validated,
  closed key inventories,
  no unchecked DTO casts,
  no generic JSON-schema interpreter,
  no parallel protocol reconstruction.
- `PreparationRootError` uses fixed kinds
  `input-bytes | input-json | input-shape | input-relations`
  and never parser excerpts or causes.
  Tests use the `q7z9k2` needle convention to assert
  fixture text never leaks into diagnostics.
- Allocation bounds belong to the authorized I/O owner;
  the reader invents no body-size limits.
- Style:
  300 nonblank noncomment production-code-line budget per file,
  star-less TSDoc on declarations,
  `//` comments for statements,
  semantic Markdown line breaks,
  `concurrency: 1` for tests that replace global state.
- Tests import fresh built artifacts
  (`../dist/final/node/preparation-input-read.mjs`
  or `../dist/final/node/index.mjs`),
  never sibling source.
  Node drivers inside `node_modules` use `.mjs`.
- Package tasks run through Mise with explicit paths,
  for example
  `mise run --no-deps --skip-tools //package/module/translation-repair:lint:types` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:lint:types`).
  Flags precede the task name;
  `--no-deps` and `--skip-deps` differ;
  `format:oxlint` takes no file filters.
- Container bounds:
  package checks 6 GiB RAM,
  8 GiB combined RAM and swap,
  two CPUs,
  512 PIDs,
  network disabled;
  actual-artifact native consumers 2 GiB RAM,
  3 GiB combined,
  one CPU,
  256 PIDs.
  State bounds explicitly for each run.
  Check `/proc/meminfo` headroom before starting.
- Verification uses existing main or repair read-only dependency overlays,
  never a fresh locked installation.
  Native consumers must not mutate source or dist;
  workers assert entry bytes unchanged.
- Commits:
  Conventional Commits with explicit scoped pathspecs,
  commit at the earliest opportunity,
  auto-push enabled,
  no `Closes #N` for issues that must stay open,
  no bulk staging,
  never stage `mise.lock`.
- Durable uncommitted evidence belongs under `node_modules/.monochromatic/`;
  scratch is disposable.
  When removing stopped diagnostic containers,
  retain hashes and CID bytes first,
  remove only exact owned IDs,
  and independently verify unselected IDs remain.

## Open issues and adjacent defects

- [#542](https://github.com/Aquaticat/Monochromatic/issues/542):
  the legacy block-pairing cache key aliases distinct NUL-bearing block boundaries.
  This is pre-hash delimiter ambiguity,
  not a SHA-256 collision.
  Open with `needs-triage`,
  unreviewed.
  The reader preserves the encoding unchanged;
  alias accounting uses complete question bytes.
  Evidence in `legacy-key-observation/`.
  The issue body received one editorial line-wrapping update;
  its immediate post-edit public response still showed the old body
  for an undiagnosed reason,
  and a fresh raw HTTP read matches the updated body exactly.
  Original submission and both responses are retained.
- [#541](https://github.com/Aquaticat/Monochromatic/issues/541):
  root-order transport defect,
  fixed and qualified but unreviewed.
- [#509](https://github.com/Aquaticat/Monochromatic/issues/509)
  and
  [#527](https://github.com/Aquaticat/Monochromatic/issues/527):
  comparison and cancellation issues,
  open and unreviewed.
- [#518](https://github.com/Aquaticat/Monochromatic/issues/518)
  and
  [#519](https://github.com/Aquaticat/Monochromatic/issues/519):
  separate unresolved defects.
- GitHub CLI 2.100.0 rewrites literal control escapes in JSON readbacks.
  The diagnosis and the raw-HTTP workaround are recorded in
  [`github-cli-json-control-escapes.md`](../troubleshooting/github-cli-json-control-escapes.md).
  For byte-sensitive issue readbacks use the retained
  `harness/read-key-issue-raw.mjs` pattern,
  not `gh`.

## Evidence index

Under `package/module/translation-repair/node_modules/.monochromatic/preparation-input-reader/`:

- `harness/`:
  stage generators `prepare-*-stage.mjs`,
  check runners `run-*-check.mjs`,
  native runners `run-*-native.mjs`,
  sequencers `verify-*-rN.mjs`,
  issue and raw-readback scripts.
- Per-stage results:
  `*-stage-rN-results.json` and `*-stage-rN-*.out`.
- Actual-artifact consumers named in the completed-state section above.
- `question-baseline-QWeptI`,
  `geometry-baseline-weNIba`:
  pre-extraction golden records with source bytes.
- `legacy-key-observation/`,
  `gh-json-observation/`:
  issue and readback evidence.
- `gh-doc-check-jDLrKG`:
  documentation validation proof.

Frozen comparison and Task67 evidence remain untouched under
`comparison-evidence/` and `preparation-root-order/`.
The reader's own stopped check containers await task63's final retention step;
they were intentionally preserved by earlier cleanups.

## Session-start checklist

1. Confirm `git status --short` shows only `mise.lock`
   and confirm the branch and remote state.
2. Re-read the planning record
   `doc/planning/translation-repair-persisted-input-reader-2026-09-15.md`
   and this handover.
3. Run the pending geometry golden checkpoint before new code.
4. Follow the remaining-work order without skipping qualification steps.
5. Record every checkpoint result in the planning record as it lands,
   including failures and their exact diagnostics.
