/**
 Shared oxlint configuration without `jsPlugins`.

 Holds every field of the Monochromatic oxlint config except the plugin list:
 the development entry (`index.ts`) and the built Node entry (`index.node.ts`)
 each spread this base and attach their own `jsPlugins` (source-resolved vs
 co-located sidecar URLs). Keeping `jsPlugins` out of the base means the bundled
 `dist/final/node/index.mjs` carries no `import.meta.resolve` plugin resolution.
 */

import type { OxlintConfig, } from 'oxlint';

import { overrides, } from './overrides.ts';
import { correctnessRules, } from './rule/correctness.ts';
import { restrictionRules, } from './rule/restriction.ts';
import { styleRules, } from './rule/style.ts';
import { tsdocRules, } from './rule/tsdoc.ts';

/**
 Shared oxlint configuration shared by every entry, minus `jsPlugins`.
 */
export const base: OxlintConfig = {
  categories: {
    correctness: 'error',
    suspicious: 'warn',
    pedantic: 'warn',
    style: 'warn',
  },

  options: {
    denyWarnings: true,
    reportUnusedDisableDirectives: 'warn',
    typeAware: true,
    typeCheck: true,
  },

  plugins: [
    'unicorn',
    'typescript',
    'oxc',
    'import',
    'promise',
    'node',
  ],

  env: {
    // Required even with es2026 below. The oxc config reference says `builtin` is "equivalent to es2026";
    // that is wrong: `es2026` lists only post-ES5 additions, so without `builtin` the JS plugin scope
    // manager omits Array and Object and `isGlobalReference` returns false for them (measured on oxlint
    // 1.85.0). An explicit env also replaces oxlint's default `builtin: true` rather than merging with it.
    // Details: doc/troubleshooting/oxlint-js-plugin-global-reference-env.md.
    builtin: true,
    browser: true,
    node: true,
    es2026: true,
    serviceworker: true,
    webextensions: true,
    worker: true
  },

  settings: {},

  ignorePatterns: [
    '**/dist',
    // Committed Claude Code plugin bundles (tsdown output moved out of dist;
    // see doc/decision/gitignore-negations.md).
    '**/bundle',
    '**/node_modules',
    '**/logs',
    '**/coverage',
    '**/bak',
    '**/*.js',
    '**/*.cjs',
    '**/deprecated.*',
    '**/deprecated/**',
    // Paused and deprecated package trees are out of the lint/format scope; one
    // file in package-paused also has a two-rule autofix oscillation that
    // stalls `task-oxlint --fix` (see doc/troubleshooting/oxlint-multi-fix-convergence.md).
    '**/package-paused/**',
    '**/package-deprecated/**',
    '**/fixture/**',
    '**/fixtures/**',
    '**/invalid/**',
    // The repository's fixture packages under `package/test-fixture/`, named with their parent segment
    // so a package that is itself called `test-fixture` (`package/translation-repair/test-fixture`) is linted.
    '**/package/test-fixture/**',
    // Verbatim vendored upstream source the conf fork's parity fuzzer runs as its oracle; an autofix pass
    // once rewrote it and broke the oracle (package/upstream-fork/conf.fuzz/README.md). User-approved 2026-10-29.
    '**/upstream-fork/conf.fuzz/upstream-conf/**',
    '**/perf-test-data/**',
    '**/teto-generated/**',
    '**/sudoku-puzzles*',
    '**/perf-expected-output*',
    '**/*.astro',
    '**/i18n/i18n-types.ts',
    '**/i18n/i18n-util.ts',
    '**/i18n/i18n-util.sync.ts',
    '**/i18n/i18n-util.async.ts',
    '**/*.generated.ts',
  ],

  rules: {
    ...tsdocRules,
    ...correctnessRules,
    ...restrictionRules,
    ...styleRules,
  },

  overrides,
};
