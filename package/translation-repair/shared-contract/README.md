# @monochromatic-dev/translation-repair-shared-contract

Small shared shapes that many layers of the pipeline read:
issue and claim models,
panel and adjudication shapes,
model cards and roster ids,
patch and document models,
and the provider catalogue pieces more than one client needs.
It mixes families on purpose,
because each module is imported from several layers,
and keeping them together lets the layers that build on them share one built copy without importing each other.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Issues and claims:
  the issue claim,
  its anchors and categories,
  and the clustering of claims from several critics.
  Names:
  `computeIssueClaimId`,
  `IssueClaim`,
  `SpanAnchor`,
  `validateIssueClaim`,
  `ISSUE_CATEGORIES`,
  `isIssueCategory`,
  `aggregateClaims`.
  Declared in `issue-model.ts`,
  `validate-issue.ts`,
  `issue-taxonomy.ts`,
  and `aggregate-claims.ts`.
- Panel and adjudication shapes:
  the vote states,
  ballots and tallies an adjudication panel produces.
  Names:
  `PANEL_VOTE_STATES`,
  `PanelBallot`,
  `VoteTally`,
  `AdjudicatedIssue`,
  `DEFAULT_ADJUDICATION_CONFIG`.
  Declared in `adjudicate-model.ts`.
- Model cards and roster ids:
  what is known about each model and which provider serves it under which id.
  Names:
  `MODEL_CARDS`,
  `ModelCard`,
  `ROSTER_MODEL_IDS`,
  `RosterModelId`,
  `ROSTER_CARDS`,
  `cardsServing`,
  `isDecisionSeat`.
  Declared in `model-cards.ts`,
  `model-card.ts`,
  `roster-id.ts`,
  and `model-card-derive.ts`.
- Provider catalogue pieces:
  the Hyper catalogue and credits reading,
  provider names and order,
  and the per-model limiters the clients share.
  Names:
  `HYPER_MODELS`,
  `parseHyperCredits`,
  `PROVIDER_ORDER`,
  `providerRecord`,
  `perModelLimiter`,
  `createSlotLedger`.
  Declared in `hyper-catalog.ts`,
  `hyper-credits.ts`,
  `provider-name.ts`,
  `per-model-limiter.ts`,
  and `provider-router-slots.ts`.
- Patch and document models:
  parse a document for repair,
  derive the envelopes a patch may edit,
  and place inserted chunks.
  Names:
  `parseDocument`,
  `RepairDocument`,
  `deriveEditableEnvelopes`,
  `EditableEnvelope`,
  `placeInsertions`,
  `deriveRefinableEnvelopes`,
  `selectRefinableParagraphs`.
  Declared in `parse-document.ts`,
  `patch-model.ts`,
  `chunk-insertion.ts`,
  `refine-envelope.ts`,
  and `refine-eligibility.ts`.
- Run-wide ledgers and caps:
  run spend accounting,
  the completion cap rule,
  run JSON reading and the page-title cache version.
  Names:
  `noteRunSpend`,
  `runSpendUsd`,
  `completionCapFor`,
  `COMPLETION_CAP`,
  `readRunJson`,
  `PAGE_TITLE_CACHE_VERSION`.
  Declared in `run-spend-meter.ts`,
  `completion-cap.ts`,
  `run-json-read.ts`,
  and `page-title-cache-version.ts`.
- Shared utility types:
  the pair type a callback gives its destructured parameter when it reads one entry of `Object.entries`,
  a map or a count table.
  Names:
  `ReadonlyEntry`.
  Declared in `readonly-entry.ts`.
  Every sibling package imports it from this package instead of declaring its own copy;
  `test-fixture` keeps one copy of its own because this package depends on it for tests.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `markup`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `artifact`
- `artifact-pool`
- `assembly`
- `bench`
- `consistency`
- `consolidate`
- `corpus-pass`
- `corpus-source`
- `coverage-census`
- `glossary`
- `house-rule`
- `insertion`
- `lane-contest`
- `page-check`
- `page-pass`
- `probe`
- `provider`
- `provider-stream`
- `reference`
- `rendering-audit`
- `repair`
- `run`
- `slice-cache`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/shared-contract:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/shared-contract:test:unit`,
or build and test together with `mise run //package/translation-repair/shared-contract:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `roster-buckets.test-fixture.ts`,
`accepted-cat-issue.test-fixture.ts`,
and `envelope-span.test-fixture.ts`.

Test helpers used across the family,
such as scratch directories and capturing loggers,
come from `test-fixture`,
imported as `@monochromatic-dev/translation-repair-test-fixture/ts/<file>`.

## Family docs

- [Family README](../README.md):
  the design,
  the package list and where the shared docs live.
- [Package split plan](../../../doc/planning/translation-repair-package-split.md):
  the split plan,
  with the package map and the rules the move keeps.
