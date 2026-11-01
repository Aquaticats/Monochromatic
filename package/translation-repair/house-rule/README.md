# @monochromatic-dev/translation-repair-house-rule

House policy for the memorial site:
the policy blocks and correction rules the prompts quote,
the identity and declared-name context of a page,
contributor names,
the candidate-selection model and its ledger,
and the grading and resolution sheets.
It is its own package because the critic,
repair,
translate and report layers all read the same policy text and it needs only `document-structure`,
`glossary`,
`markup`,
`shared-contract` and `typography` below it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- House policy and rules:
  the policy blocks the prompts quote and the rules about forms,
  names and bilingual lines.
  Names:
  `HOUSE_POLICY_BLOCK`,
  `JUDGE_POLICY_BLOCK`,
  `MEASUREMENT_POLICY_BLOCK`,
  `HOUSE_FORM_CORRECTIONS`,
  `POLISH_GATE_HOUSE_RULES`,
  `NAME_FORM_SCOPE_RULE`,
  `BILINGUAL_LINE_CLAUSE`.
  Declared in `house-policy.ts`,
  `house-form-corrections.ts`,
  `polish-gate-house-rules.ts`,
  `name-form-policy.ts`,
  and `bilingual-line-clause.ts`.
- Identity context and declared names:
  read who a page declares itself to be about,
  and check that a translation keeps the declared name forms.
  Names:
  `extractDeclaredIdentity`,
  `collectIdentityLines`,
  `declaredNameForms`,
  `findDroppedDeclaredNames`,
  `declaredNameRefusalReport`,
  `projectName`,
  `nameOccurrences`,
  `declaredNamePairs`.
  Declared in `identity-context.ts`,
  `declared-name-survival.ts`,
  `name-projection.ts`,
  and `linked-title-declared-name.ts`.
- Whole-name arrival:
  when a translation loses one declared form of the subject,
  decide whether another declared form arriving in its place stands as a whole name.
  It must not run on into a Latin letter or digit,
  or across a hyphen or an underscore;
  it must be spelled as declared,
  so `Li Li` does not stand for `Lili`;
  and no capitalised Latin word may stand beside it unless that word belongs to a declared form,
  is an ordinary opener,
  title,
  kin word or month,
  or stood beside the lost form in the base.
  Carriage is unchanged:
  `carriesName` still reads `Li Li` as carrying `Lili`,
  and only an arrival is held to this.
  Names:
  `arrivesAsWholeName`,
  `besideWordsOf`,
  `declaredWordsOf`,
  `excusedNeighboursOf`,
  `ORDINARY_CAPITALISED_WORDS`.
  Declared in `whole-name-arrival.ts` and `ordinary-capitalised-words.ts`.
- Contributor names:
  the name forms an archive credits and the findings raised when a translation departs from them.
  Names:
  `archiveContributorNameForms`,
  `contributorDeclarationLines`,
  `contributorAuthorityFindings`,
  `CONTRIBUTOR_AUTHORITY_FINDING`.
  Declared in `contributor-name-authority.ts` and `contributor-translation-guard.ts`.
- Candidate selection model and ledger:
  candidates,
  producers and ballot weights,
  merged duplicates,
  and the ledger a contest is recorded in.
  Names:
  `mergeIdenticalCandidates`,
  `mergeProducers`,
  `FULL_VOTE_WEIGHT`,
  `SELF_VOTE_WEIGHT`,
  `recordContest`,
  `readContestLedger`,
  `LEDGER_DIR`.
  Declared in `candidate-merge.ts`,
  `candidate-select-model.ts`,
  and `candidate-ledger.ts`.
- Grading and resolution sheets:
  draw a stratified sample for human grading,
  format the grading sheet,
  and render the issue blocks a resolution checker reads.
  Names:
  `drawStratifiedSample`,
  `formatGradingSheet`,
  `extractGradingCandidate`,
  `DEFAULT_SAMPLE_SIZE`,
  `resolutionIssueBlock`,
  `CHECKER_IDENTITY_RULE`,
  `wroteTextForIssue`.
  Declared in `sample-draw.ts`,
  `grading-sheet.ts`,
  `sample-grading.ts`,
  `resolution-sheet-evidence.ts`,
  and `resolution-authorship.ts`.
- Refusal shape:
  spot a model reply that refuses rather than answers.
  Names:
  `detectRefusalShape`,
  `REFUSAL_MARKERS`,
  `REFUSAL_SCAN_WINDOW`.
  Declared in `refusal.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `glossary`
- `markup`
- `shared-contract`
- `typography`

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
- `insertion`
- `lane-contest`
- `page-pass`
- `probe`
- `provider`
- `provider-stream`
- `rendering-audit`
- `repair`
- `run`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/house-rule:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/house-rule:test:unit`,
or build and test together with `mise run //package/translation-repair/house-rule:buildAndTest`.
Pass test file paths to run only those files.

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
