# Writer calibration unit scope: the paired-parent design and the production-envelope correction

The constraints on any repair of the unit scope,
the direction to investigate,
the whole-paired-parent prototype and its ranking against merging and context-only options,
and the measurement that rejected it in favour of frozen production-prepared groups.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Constraints

- Preserve the production archive floor and legitimate English expansion policy.
- Preserve forty calibration tasks and the existing producer,
  contributor,
  self-vote and pooled-null rules.
- Do not repair this with added generation rounds,
  a single-entry exception or post-outcome row deletion.
- Recompose both source and incumbent siblings together;
  merely adding neighboring source as context can duplicate or omit output obligations.
- Keep source,
  incumbent,
  ranges,
  parent identity and any coalesced membership auditable before model calls.
- Distinguish mechanical containment in an aligned parent
  from an unproven semantic guarantee about every archive sentence.

The proposed `AGENTS.md` `QIV` clarification in
[the reviewed fixture record](translation-repair-reviewed-calibration-fixtures-2026-09-11.md)
now includes consumer contracts and output obligations within each input unit.
It is not applied to `AGENTS.md`.
The expected action is being performed:
paid execution is blocked while the actual floor/scope mismatch is corrected.

## Direction to investigate

Trace the current sample through `alignDocumentSections` and `subdivideChunkPair`.
Prefer units closed over their archive-floor obligations rather than independently subdivided source/target fragments.
A coalesced parent can dissolve the mismatch;
if it needs subdivision,
source and incumbent must be split together only at independently closed boundaries.
Preserve production-relevant size limits instead of silently replacing slice work with unbounded whole documents.

The revised deterministic population must yield forty ordered units,
with explicit provenance and a pre-call reading.
Keep valid heading and media controls;
report non-informative contests separately rather than padding or dropping rows after outcomes.

Required checks include the windward regression,
the valid heading control,
complete paired source/target coverage,
no duplicate obligations,
and a removal test proving the coalescing or paired-boundary guard matters.
Freeze the revised plan and runtime before paid execution.

## Paired-parent prototype

The pinned census contains 275 nonempty deterministic section pairs across the corpus.
The source code does not simply zip independent chunks:
`subdivideChunkPair` uses a scorer-based monotone block alignment when no corroborated pairing is supplied.
The observed mismatch comes from using that uncorroborated fine-grained result as a calibration task.

A prototype that retains complete paired parents and applies the existing deterministic spread
produces forty tasks with 20117 source characters and 51710 incumbent characters.
The largest selected task has 1983 source characters and 5016 incumbent characters.
The whole population's largest target parent is 17851 characters;
the spread samples stratum midpoints rather than forcing that extreme into the draw.

Design comparison:

- Whole paired parents:
  preserves complete paired extents without claiming unproved fine-grained correspondence;
  increases the measured input distribution and still needs parent-alignment review.
- Selective sibling merging:
  could retain more of the old granularity;
  block counts alone cannot establish all cross-language dependencies,
  so missing proof would leave hidden mismatches.
- Neighboring source context only:
  adds evidence without changing the sample indexes;
  it leaves output obligations assigned to the wrong task and is not a remedy for this defect.

Ranking:
whole paired parents > selective merging > context-only.
Whole parents establish mechanical containment without the additional correspondence proof selective merging needs;
selective merging could correct output assignment with proof,
whereas context alone cannot.
Whole-parent selection was the implementation candidate at this checkpoint,
but the production-envelope experiment subsequently rejected it.
The larger input distribution and remaining deterministic parent-alignment assumptions must be disclosed,
and the selected forty parents still require a complete source/target reading before calibration.
No semantic correctness label is assigned to the incumbents.

The windward parent covers source offsets 2069 to 2988 and target offsets 11101 to 13840.
Its source includes the body missing from legacy `#14`.
Its incumbent still has a genuine missing-footnote issue,
which is distinct from task scope and must not be called a clean reference.
The new draw does not happen to select that parent;
the regression will exercise it directly rather than inserting it into the sample by hand.

Private prototype records are `parent-census.json` and `parent-prototype.json`
in the owned writer workspace.

## Production-envelope correction

`scope-envelope.json` records the next measured boundary.
The whole-parent sample contains `hulicaijia` source section 7,
whose parent predicate is false while a legacy child is line-structured.
The parent Boolean therefore cannot replace every child Boolean.
The forty parent tasks have median source length 393 and maximum 1983;
Mio12's actual recorded preparation has median 87 and maximum 390 across its seventeen slices.
This comparison is descriptive,
not a universal production maximum.

Keeping only parents that fit both existing node-group budgets leaves 43 candidates.
Its forty-task draw has median source length 89 and target length 171,
mostly single-node parents.
Allowing either atomic side adds only three candidates.
This is not evidence that such filtering preserves representative coverage.
The unrestricted and shape-filtered whole-parent proposals are rejected.
Only a provenance type scaffold was committed at `53b7c7f1e`;
no runtime sampler or writer was changed to use either proposal.
The unreferenced whole-parent scaffold was removed after this correction.
Prepared-group interfaces should reuse existing preparation shapes rather than retain that abandoned type.

The chosen implementation direction is frozen production-prepared groups:

- Freeze a deterministic bounded parent pool before any pairing calls.
- Use existing configured block-pairing electorate,
  quorum and stage policy without translation-generation calls.
- Account for preparation calls separately.
  Abort the plan on an uncorroborated fallback instead of dropping its parent afterward.
- Consume complete production pairing groups and existing subdivision,
  including insertion,
  unclaimed-target and child-level line-structure semantics.
- Draw forty prepared units deterministically,
  then persist and exit before writing.
- Review those exact units with their surrounding source and target parents.
  Incumbents remain fallible baselines,
  not gold labels.
- Require a distinct reviewed-plan digest before generation.
  Rebuild from the frozen recipe provider-free and refuse any mismatch before provider creation.
- Record corpus,
  build,
  configuration and population identities,
  ballots,
  recipes,
  exact ranges/hashes/node membership,
  line flags and actual serialized prompt sizes.

Independent review supported this direction after examining the new measurements.
It does not add writer rounds or change the writer standing rule.
The old launcher remains blocked.
A fresh budget reading is required before paid preparation and again before writer generation.
