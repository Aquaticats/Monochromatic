# @monochromatic-dev/translation-repair-reference

Cited references for the critics:
looking up the page behind a citation,
attesting which details of the archive text the cited page states,
finding work titles,
and the on-disk caches those lookups keep.
It is its own package because the critics,
the translate lane and the probes all use the lookups and their caches,
and it sits above `provider` and `stage`,
which it calls through.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Cited reference lookup:
  find the URLs a page cites,
  fetch each,
  and write the reference block a prompt shows.
  Names:
  `citedReferenceUrlsOf`,
  `lookupCitedReference`,
  `citedReferenceBlock`,
  `referenceLineOf`,
  `numberedReferenceLines`.
  Declared in `cited-reference-scan.ts`,
  `cited-reference-lookup.ts`,
  and `reference-line-head.ts`.
- Attestation:
  ask the bench which details a cited reference states,
  check each quote word for word,
  keep the confirmed ones,
  and reject an addition claim whose archive-side quote overlaps an attested archive quote before the panel hears it.
  Names:
  `attestCitedReferences`,
  `buildReferenceAttestMessages`,
  `REFERENCE_ATTEST_RESPONSE_FORMAT`,
  `attestationVerdicts`,
  `keptAttestations`,
  `screenAttestedAdditions`.
  Declared in `reference-attest-stage.ts`,
  `reference-attest-wire.ts`,
  `reference-attest-verdict.ts`,
  and `reference-attest-claims.ts`.
- Work titles:
  pick the work titles out of a page and look each one up by web search.
  Names:
  `workTitlesOf`,
  `lookupQueryFor`,
  `lookupWorkTitle`,
  `searchWorkTitle`,
  `WorkTitleLookupError`.
  Declared in `work-title-scan.ts`,
  `work-title-lookup.ts`,
  and `work-title-search.ts`.
- Lookup caches:
  read and write the durable caches of lookups and references.
  Names:
  `readCachedLookup`,
  `writeCachedLookup`,
  `lookupCachePath`,
  `readCachedReference`,
  `writeCachedReference`,
  `referenceCachePath`,
  `CacheFileUnreadableError`.
  Declared in `lookup-cache.ts` and `reference-cache.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `glossary`
- `markup`
- `provider`
- `provider-stream`
- `rendering-audit`
- `shared-contract`
- `slice-reading`
- `stage`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `archive`
- `corpus-pass`
- `corpus-source`
- `coverage-census`
- `page-check`
- `probe`
- `repair`
- `translate-lane`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/reference:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/reference:test:unit`,
or build and test together with `mise run //package/translation-repair/reference:buildAndTest`.
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
