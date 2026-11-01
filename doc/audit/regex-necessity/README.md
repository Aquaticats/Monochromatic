# Regex-necessity audit harness

Instrument for the audit recorded in
[`../regex-necessity.md`](../regex-necessity.md).
Committed so the classification is reproducible from the repository instead of from one
machine's scratch directory.

This directory is the audit's own tooling,
 not product code.
It is excluded from the audit population:
 the harness parses patterns with patterns,
 so scanning it would report the instrument rather than the subject.

## Layout

- `build-mirror.mjs` mirrors every JS and TS file in the working tree,
  rewriting `oxlint-disable` to `oxlint-xisable` and `oxlint-enable` to `oxlint-xnable`.
  Both replacements are the same byte length,
  so positions survive.
- `scan.config.json` enables only `no-restricted-syntax/no-regex`,
  with empty `ignorePatterns`.
- `sites3.mjs` dedupes spans on `(offset, length)`,
  slices pattern text on UTF-8 byte offsets,
  and attributes each site to the directive range that covers it.
- `extract-html.mjs` extracts `<script>` blocks from tracked HTML,
  Vue,
  and Astro files,
  and enumerates the inline-YAML surface.
- `pipeline.mjs` classifies necessity,
  resolves package scopes from the nearest `mise.toml` and `package.json`,
  and finds enclosing declarations.
- `pattern-analysis.mjs` tokenizes pattern bodies and derives replacement shapes.
- `rebuttals.mjs` holds one answer per distinct recorded justification.
- `supplement.mjs` adds the Kotlin,
  HTML,
  and YAML sites,
  and the two sites reclassified on individual review.
- `generate.mjs` writes one issue payload per site.
- `reconcile.mjs` corrects already-filed issues in place,
  then files the rest.
- `fix-html-patterns.mjs` resolves each HTML script-block site to its own regex
  literal,
  slicing the extracted block on byte offsets.
- `repair-titles.mjs` re-titles and re-bodies the issues affected by corrections made
  after a filing round.
- `close-excluded.mjs` closes the issues filed for sites later excluded on
  provenance,
  with the evidence in the closing comment.
- `reapply-all.mjs` deterministically re-applies every current payload to its issue,
  which is the safe way to land a correction round:
  it adds labels without removing,
  and it is resume-safe through its own ledger.
- `spotcheck.mjs` reads filed issues back from the tracker and confirms each body
  names its site,
  quotes the extracted pattern,
  agrees with the repository source at that line,
  and carries the expected title and labels.
- `verify.mjs` reconciles the ledger against the tracker:
  duplicates,
  missing sites,
  and a tracker-side count by title.
- `repair-labels.mjs` re-applies issue labels after a pass that removed and added
  labels in one `gh issue edit` call,
  because GitHub applies the removals last.
- `index-update.mjs` builds the grouped issue list for the index issue from the ledger.
- `inject-scopes.mjs` writes the computed per-scope counts into the index issue body.
- `evidence.mjs` holds the first measurement round.
- `evidence2.mjs` supersedes two mis-constructed probes in `evidence.mjs`
  (the fence-strip input and the re-export matcher input) and adds the replacement-side
  measurement.

## Run order

```sh
# 1. Mirror the tree and scan it with the repo's own rule.
node build-mirror.mjs
<repo>/node_modules/.bin/oxlint --config scan.config.json --no-ignore \
  --format json "$WORK/mirror" > "$WORK/scan.json"

# 2. HTML script blocks, scanned the same way.
node extract-html.mjs
<repo>/node_modules/.bin/oxlint --config scan.config.json --no-ignore \
  --format json "$WORK/mirror-html" > "$WORK/scan-html.json"

# 3. Classify, supplement, generate payloads.
node sites3.mjs && node pipeline.mjs && node supplement.mjs && node generate.mjs

# 4. Evidence, run separately because timings are machine-specific.
node evidence.mjs && node evidence2.mjs
```

## Inputs the harness expects in its work directory

Two artifacts are produced by commands,
 not by the harness:

- `tasks.txt`,
  a full `mise tasks ls --all` dump.
  `generate.mjs` reads it so acceptance criteria name only tasks the owning package
  actually defines.
- `filed.jsonl`,
  the ledger of a previous filing run.
  `reconcile.mjs` reads it to pair already-created issues with corrected payloads.
  A fresh audit has no ledger and files everything as new.

## Machine-specific values

- The work directory defaults to `${HOME}/temp/agent/regex-audit` and honours
  `REGEX_AUDIT_WORK`.
- `scan.config.json` points `jsPlugins` at this clone's built plugin sidecar with an
  absolute `file://` URL.
  Rewrite that one URL for a different clone location;
  build it with
  `mise run //package/config/oxlint:build:js:node`.
