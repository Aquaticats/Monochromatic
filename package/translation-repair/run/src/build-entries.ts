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
    'budget-sample',
    './src/corpus-run/budget-sample.ts',
  ],
  [
    'meter-report',
    './src/corpus-run/meter-report.ts',
  ],
  [
    'cap-census',
    './src/corpus-run/cap-census.ts',
  ],
  [
    'ledger-report',
    './src/corpus-run/ledger-report.ts',
  ],
  [
    'repeat-census',
    './src/corpus-run/repeat-census.ts',
  ],
  [
    'run-timing-report',
    './src/corpus-run/run-timing-report.ts',
  ],
  [
    'spend-report',
    './src/corpus-run/spend-report.ts',
  ],
  [
    'verify-published',
    './src/corpus-run/verify-published.ts',
  ],
],);

//endregion Build entries
