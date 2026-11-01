# @monochromatic-dev/translation-repair-document-structure

Parses a page into a document model the rest of the pipeline can anchor issues to.
It holds the MDX and Markdown parse,
front-matter splitting,
the footnote graph,
container masks and the heading and block alignment that pairs original and translation.
It is its own package because nearly every layer that builds on it needs the parse,
and it needs only `markup` below it.

## What it provides

`src/index.ts` re-exports each public name from the module that declares it.
The groups that follow name the main ones.

- MDX and Markdown parsing:
  parse a page body,
  split its front matter,
  and refuse text the parser cannot take.
  Names:
  `parseMdxBody`,
  `parseMarkdownBody`,
  `parseMarkdownMathBody`,
  `MdxParseError`,
  `MarkdownParseError`,
  `splitFrontMatter`,
  `FrontMatterParseError`,
  `parseSliceBody`.
  Declared in `parse-mdx.ts`,
  `parse-markdown-math.ts`,
  `front-matter.ts`,
  and `parse-slice-body.ts`.
- Document nodes and masks:
  build anchor-ready nodes from the top-level blocks of a page,
  mask comments,
  lone container tags and invisible lines to same-length whitespace before the parser reads them,
  and promote the children of containers to top-level blocks or widen block extents to hold container tags.
  Names:
  `buildDocumentNodes`,
  `maskHtmlComments`,
  `maskLoneContainerTags`,
  `maskInvisibleLines`,
  `flattenContainers`,
  `widenExtentsToContainers`.
  Declared in `document-node.ts`,
  `mask-html-comments.ts`,
  `mask-container-tags.ts`,
  `mask-invisible-lines.ts`,
  `unwrap-container.ts`,
  and `container-extents.ts`.
- Footnote graph:
  find footnote references and definitions,
  normalize their identifiers,
  and refuse a rename that cannot be proven syntax-preserving and injective.
  Names:
  `buildFootnoteGraph`,
  `footnoteMentions`,
  `normalizeFootnoteIdentifier`,
  `definitionStartsOf`,
  `FootnoteRewriteError`.
  Declared in `footnote-graph.ts`,
  `footnote-mentions.ts`,
  `footnote-identifier.ts`,
  `footnote-definition-starts.ts`,
  and `footnote-rewrite-error.ts`.
- Heading and block alignment:
  pair the headings and blocks of an original with those of its translation.
  Names:
  `alignBlocks`,
  `alignHeadingsForced`,
  `scanOptimalPaths`,
  `estimateExpansion`,
  `scorePairing`,
  `tokenize`.
  Declared in `align-blocks-walk.ts`,
  `align-headings-forced.ts`,
  `align-headings-optimal.ts`,
  and `align-blocks.ts`.
- Shared helpers and wording rules:
  concurrency that reports failures in input order,
  JSON record guards,
  Markdown fences,
  caught values rendered without quoted content,
  and the English usage rules the prompts quote.
  Names:
  `allInInputOrder`,
  `isJsonRecord`,
  `requireJsonSyntaxRefusal`,
  `fenceForMarkdown`,
  `refusalText`,
  `GRAMMATICAL_ENGLISH_RULE`,
  `IDIOMATIC_ENGLISH_RULE`.
  Declared in `all-in-input-order.ts`,
  `json-guard.ts`,
  `markdown-fence.ts`,
  `refusal-text.ts`,
  and `english-usage-policy.ts`.

## Family dependencies

Short names are directory names under `package/translation-repair`;
a package is named `@monochromatic-dev/translation-repair-<short name>`.

### Depends on

Production dependencies (`dependencies` in `package.json`):

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
- `shared-contract`
- `slice-cache`
- `slice-reading`
- `stage`
- `translate-floor`
- `translate-lane`
- `translate-stage`
- `typography`

Packages that list it only in `devDependencies`:

- `source-scan`

## Tests

Tests sit beside the sources as `*.unit.test.ts` and import built output:
this package through its own build,
`../dist/final/node/index.mjs`,
and the siblings they need through their package names,
which resolve to each sibling's built entry.

Build before testing:
`mise run //package/translation-repair/document-structure:build` builds this package,
and the library build keeps family siblings external,
so this package's built entry imports the siblings' built entries.
Its `build:js:node` task therefore declares `depends` on the `build:js:node` of every sibling listed in
`package.json`,
and the siblings build first.

Run the tests with `mise run //package/translation-repair/document-structure:test:unit`,
or build and test together with `mise run //package/translation-repair/document-structure:buildAndTest`.
Pass test file paths to run only those files.

Its `./ts/*` export also serves test fixtures that sibling tests import,
for example `corpus-run/generation-restamp.test-fixture.ts`,
`tag-name-spellings.test-fixture.ts`,
and `container-tag-ownership.test-fixture.ts`.

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
