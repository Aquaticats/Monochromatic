/**
 Packaging, install, engines, type-checker-cost, and repo-internal rows of
 the historical-recall ledger. The sidecar consumes deepmerge-ts one way
 (Node ESM `import` of the npm build, and TypeScript `bundler` resolution
 through the `exports` types condition), so these rows ask whether that one
 way would have shown the bug; `./recall-packaging.ts` measures it.

 @module
 */

import type { SurfaceBug, } from './recall-ledger.ts';

/**
 Surface bugs in release order.
 */
export const SURFACE_BUGS: readonly SurfaceBug[] = [
  {
    broke: '`husky install` ran as a consumer `postinstall` script',
    buggy: '1.0.0',
    commit: '7102229',
    fixed: '1.0.1',
    id: 'postinstall-husky',
    issue: '',
    regressionTest: 'none',
    surface: 'install',
    version: '1.0.1',
  },
  {
    broke: 'no declarations for TypeScript before 4.1',
    buggy: '1.1.0',
    commit: 'c7e1019',
    fixed: '1.1.1',
    id: 'legacy-types-missing',
    issue: '#6',
    regressionTest: 'none',
    surface: 'packaging',
    version: '1.1.1',
  },
  {
    broke: 'current and legacy declarations were written to the same file, so one overwrote the other',
    buggy: '1.1.1',
    commit: 'a5f334b',
    fixed: '1.1.2',
    id: 'legacy-types-overwritten',
    issue: '#10',
    regressionTest: 'none',
    surface: 'packaging',
    version: '1.1.2',
  },
  {
    broke: 'the `exports` conditions were in an order that resolved the wrong build',
    buggy: '1.1.2',
    commit: '4117460',
    fixed: '1.1.3',
    id: 'exports-order',
    issue: '#12',
    regressionTest: 'none',
    surface: 'packaging',
    version: '1.1.3',
  },
  {
    broke: 'the Deno release was published broken',
    buggy: '1.1.4',
    commit: '4b8ca98',
    fixed: '1.1.5',
    id: 'deno-release',
    issue: '',
    regressionTest: 'none',
    surface: 'packaging',
    version: '1.1.5',
  },
  {
    broke: 'the Deno dist had an unresolvable import',
    buggy: '4.0.0',
    commit: '86faf2a',
    fixed: '4.0.1',
    id: 'deno-import',
    issue: '#84, #85',
    regressionTest: 'none',
    surface: 'packaging',
    version: '4.0.1',
  },
  {
    broke: '`tsc` hung on projects that import deepmerge-ts (HKT-returning functions without explicit return types)',
    buggy: '4.0.2',
    commit: 'eb4183e',
    fixed: '4.0.3',
    id: 'tsc-hang',
    issue: '#94',
    regressionTest: 'none',
    surface: 'type-checker',
    version: '4.0.3',
  },
  {
    broke: 'the `typesVersions` path for TypeScript 4.7+ named a missing file',
    buggy: '4.2.0',
    commit: 'b875711',
    fixed: '4.2.1',
    id: 'types-path',
    issue: '#145, #146',
    regressionTest: 'none',
    surface: 'packaging',
    version: '4.2.1',
  },
  {
    broke: 'a missing dev dependency (repo build only)',
    buggy: '4.3.0',
    commit: 'df4add2',
    fixed: '5.0.0',
    id: 'dev-dependency',
    issue: '',
    regressionTest: 'none',
    surface: 'repo-internal',
    version: '5.0.0',
  },
  {
    broke: 'an unneeded eslint disable comment (source comment only)',
    buggy: '4.3.0',
    commit: 'be28290',
    fixed: '5.0.0',
    id: 'eslint-disable',
    issue: '',
    regressionTest: 'none',
    surface: 'repo-internal',
    version: '5.0.0',
  },
  {
    broke: 'no `types` or `main` fallback, so `moduleResolution: node10` consumers found no declarations',
    buggy: '7.0.1',
    commit: '063675e',
    fixed: '7.0.2',
    id: 'node10-resolution',
    issue: '#480',
    regressionTest: 'none',
    surface: 'packaging',
    version: '7.0.2',
  },
  {
    broke: '`engines` allowed Node 16.0 to 16.8, which lack the `Object.hasOwn` the build calls',
    buggy: '8.0.1',
    commit: 'ef54ea6',
    fixed: '8.0.2',
    id: 'engines-hasown',
    issue: '',
    regressionTest: 'none',
    surface: 'engines',
    version: '8.0.2',
  },
];
