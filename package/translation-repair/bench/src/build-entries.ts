//region Build entries
// Every RUNNER is a build entry, not just the library index.
//
// The tasks used to invoke `node src/corpus-run/<name>.ts` directly, so what
// executed was loose source while the tests imported the built bundle. That
// split is what made "which pipeline produced this artifact" hard to answer at
// all: there was no single artifact to point at, only a transitive closure of
// source files that had to be reconstructed to be identified.
//
// Building the runners makes the executed thing a FILE. Identity, staleness and
// reproducibility all become questions about `dist/final/node` rather than
// about an import graph.
//
// KEPT IN SOURCE rather than in the build config because two readers need it:
// the normal build and the coverage build (`rolldown.node.config.ts`,
// `rolldown.coverage.config.ts`), and the coverage census, which tells a
// runner's command-line main from library source by this list (ledger T8).

/**
 Named entries of the normal build and the coverage build, each a path from
 the package directory. A map, as every table keyed by a name is (ledger
 B77); the build configs hand rolldown its entries as an object.

 @example
 ```ts
 const runner = nodeEntries.get('coverage-census',); // './src/corpus-run/coverage-census.ts'
 ```
 */
export const nodeEntries: ReadonlyMap<string, string> = new Map([
  [
    'index',
    './src/index.ts',
  ],
  [
    'checker-sensitivity',
    './src/corpus-run/checker-sensitivity.ts',
  ],
  [
    'model-catalog',
    './src/corpus-run/model-catalog.ts',
  ],
  [
    'model-health',
    './src/corpus-run/model-health.ts',
  ],
  [
    'reader-measure',
    './src/corpus-run/reader-measure.ts',
  ],
  [
    'roster-card',
    './src/corpus-run/roster-card.ts',
  ],
  [
    'producer-calibrate',
    './src/corpus-run/producer-calibrate.ts',
  ],
  [
    'roster-bench',
    './src/corpus-run/roster-bench.ts',
  ],
  [
    'sentinel-probe',
    './src/corpus-run/sentinel-probe.ts',
  ],
  [
    'displacement-probe',
    './src/corpus-run/displacement-probe.ts',
  ],
  [
    'editor-calibrate',
    './src/corpus-run/editor-calibrate.ts',
  ],
  [
    'editor-standing-read',
    './src/corpus-run/editor-standing-read.ts',
  ],
  [
    'editor-width-probe',
    './src/corpus-run/editor-width-probe.ts',
  ],
  [
    'translate-probe',
    './src/corpus-run/translate-probe.ts',
  ],
  [
    'window-trial-probe',
    './src/corpus-run/window-trial-probe.ts',
  ],
],);

//endregion Build entries
