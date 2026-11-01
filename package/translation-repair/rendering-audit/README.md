# @monochromatic-dev/translation-repair-rendering-audit

The rendering audit:
it asks a roster whether one rendering says what its original says,
screens every answer against both texts,
and reports what more than one voice found.
It also holds the rendering glossary of ordinary Chinese words and the page-name and heading readers the audit
leans on.
It is its own package because corpus preparation,
the passes,
the translate floors and the probes all read the same audit library and page-name glossary.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- Audit library:
  build the audit prompt,
  run it,
  screen and corroborate its findings,
  and anchor them to spans.
  Names:
  `runRenderingAudit`,
  `buildRenderingAuditMessages`,
  `RENDERING_AUDIT_RESPONSE_FORMAT`,
  `screenRenderingAudit`,
  `corroborate`,
  `anchorLocatedSpan`,
  `RENDERING_AUDIT_CATEGORIES`.
  Declared in `rendering-audit.ts`,
  `rendering-audit-prompt.ts`,
  `rendering-audit-wire.ts`,
  `rendering-audit-screen.ts`,
  `rendering-audit-corroborate.ts`,
  and `rendering-audit-anchor.ts`.
- Rendering glossary:
  the glossary of ordinary Chinese words whose word-for-word rendering reads badly in English,
  and the identity-context lines that show it to a model.
  Names:
  `RENDERING_GLOSSARY`,
  `renderingTermLines`.
  Declared in `rendering-glossary.ts`.
- Page names and headings:
  the names of linked pages and how the archive renders their links,
  repeated title spans the archive leaves unpaired,
  the footnote and comment notes of an entry,
  and the identity lines of a page.
  Names:
  `pageNameLines`,
  `pairedPageNames`,
  `linksOf`,
  `renderedLinks`,
  `repeatedTitleSpans`,
  `pageIdentityLines`,
  `entryNoteLines`.
  Declared in `page-name-glossary.ts`,
  `page-name-link-rendering.ts`,
  `page-title-spans.ts`,
  `page-identity-lines.ts`,
  and `entry-notes.ts`.
- Names the archive spells inside its sentences:
  pair a Han name of an original with the capitalised Latin words the archive spells it with,
  by the letters of the name's pinyin reading,
  list the pairs on the page-name block of every sheet,
  and name the spellings a candidate gives the same letters other than the archive's.
  Names:
  `proseNamePairs`,
  `respelledRendering`,
  `ProseNamePair`.
  Declared in `prose-name-pairs.ts`,
  `prose-name-han.ts`,
  and `prose-name-spelling.ts`.
- Attribution signatures:
  read the signer's name off a signature line,
  which opens with one or more dashes and carries the name and then a comma and the date,
  and list the signatures of a text.
  Names:
  `readSignature`,
  `signaturesOf`.
  Declared in `corpus-run/attribution-line.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

- `document-structure`
- `glossary`
- `house-rule`
- `markup`
- `provider`
- `shared-contract`
- `slice-reading`
- `stage`
- `typography`

### Used by

Packages that list it in `dependencies`:

- `adjudication`
- `artifact`
- `corpus-pass`
- `corpus-source`
- `page-check`
- `page-pass`
- `probe`
- `reference`
- `translate-floor`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/rendering-audit:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/rendering-audit:test:unit`,
or build and test together with `mise run //package/translation-repair/rendering-audit:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `rendering-glossary-seeded.test-fixture.ts`.

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
