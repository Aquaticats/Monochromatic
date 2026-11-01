# Writer calibration unit scope: the dependency closure census and the frozen-parent control

The provider-free census of the selected entries' section and definition dependencies,
which found one whole-entry section trigger,
and the consumer control that keeps the per-parent seam.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Measuring the owning journal's dependency closure

An independent design review proposed initial and post-footnote phases.
That phase count is not adopted before measuring whether the selected entries trigger native section pairing
or contain definition dependencies outside their selected parents.
The measured caller must determine the finite slot closure.
Model-plus-canonical-prompt uniqueness also applies across slots:
per-slot client caches must not become a way to buy the same model payload again.
Authorized aliases must be established by the root plan,
not inferred silently from a shared cache.

`preparation-dependency-census-20260912.mts` will re-read the pinned selected corpus entries,
verify the frozen selection and its referenced file hashes,
compare current selected-parent coordinates with the baseline,
intercept the actual `buySectionPairing` dispatch and inventory definition/media dependencies.
The section probe has an explicit required-question positive control and an equal-shape negative control.
It obtains no model response and grants no correspondence or final-unit admission.
The census runs with no network,
2 GiB memory,
2 CPUs and 512 PIDs,
read-only source/corpus/selection mounts and owned writable outputs.

Its current runtime is frozen at `node_modules/.frozen-dist-preparation-census-20260912` inside the package,
with digest `sha256-tree-v1:f7afc15c1b5cd1becb186fe3af1fad234caef1dd3963cb881bdbd432dc1f5f2a`.
`preparation-census-runtime-20260912.json` records the exact runtime and Node 26.8.2.
The first scratch freeze attempted an exclusive directory copy into its newly created destination and failed;
the inspected empty owned directory was then filled with exclusive per-file copies and both tree digests matched.
The first census validates the frozen references and section-dispatch controls,
then stops because its scratch code called the unexported `activeFootnoteMarkers` helper.
The probe now uses the existing public `documentLabels` and `referenceLabels` projections.
This is a probe-interface correction,
not an eligibility finding or a reason to reopen the frozen parent selection.
R2 completes the census.
R3 adds complete protection-state comparisons and a duplicate-question positive control.
`preparation-dependency-census-r3-20260912.out` and
`preparation-dependency-census-61BEKU/report.json` record the completed measurement:

- All 63 frozen reference files verify.
- All 40 selected parents retain their frozen source/target coordinates,
  node identities,
  hashes and protection state under the current runtime.
- Selected dispatch is 37 queried parents and 3 implicit parents,
  with no empty side.
  The implicit parents are `gaoyanger#0`,
  `xuewulihuameng#0` and `zheermao101#0`.
- The aligned definition inventory adds `Dethelly#5`,
  `Futajuhuacha#2`,
  `NIGHT81473140#9`,
  `Y1Ran#3` and `aiyysk#13`.
  The union has 45 parents,
  including 42 queried parents.
- No exact-question alias group is found;
  the synthetic duplicate-question positive control proves the grouping probe detects one.
- Native whole-entry section pairing triggers only for `XingZ60`:
  15 source sections and 13 target sections.
  Source sections 12 through 14 and target section 12 remain outside deterministic correspondence.
- Xing's source definition nodes `block/236` through `block/244` are unaligned.
  Its target has references but no definition nodes.
  This is deterministic source/namespace context,
  not source-to-target definition endorsement.
- Selected source-picture references occur in the 10 parent IDs listed by the report.
  This is dependency inventory,
  not image reading or corroboration.
- Fetch and provider request counts are zero.
  Peak container memory is 159240192 bytes,
  with no OOM or PID-limit event.

## Whole-entry section trigger and the frozen-parent control

The whole-entry section trigger is not automatic authorization for a new paid phase.
`prepareBlockPairing` accepts an already reconstructed parent;
the frozen-parent design need not inherit every operation of the whole-entry shell.
Conversely,
a nonempty explicit section pairing replaces the default correspondence and can remove a frozen selected pair.
A provider-free consumer/control check is next before fixing journal layout.
It must distinguish unchanged parent selection from newly reconstructed final children:
task 40 did not freeze final writer units,
and an authorized label rewrite must not be required to preserve stale child hashes or coordinates.

`frozen-parent-consumer-control-20260912.out` completes that mechanical control under the same bounded runtime.
`frozen-parent-consumer-c8Htkl/report.json` records:

- Xing parents 6,
  1 and 9 retain their registered source/target scopes without a section override.
- No source-to-target definition relation is invented from the nine source definitions and absent target definitions.
  The definition-only empty relation input produces no label rewrite.
- Native default grouping and `translateSliceInput` stay inside the selected parent node sets,
  with current indexes and hashes.
  These default groups are mechanism evidence,
  not qualified writer units or their future frozen identities.
- The independently valid partial section relation `source 12 → target 12` changes all three selected parent scopes.
  The scope checker requires abort/reopening,
  never substitution.
- Fetch and provider request counts are zero.
  Peak memory is 128495616 bytes.
