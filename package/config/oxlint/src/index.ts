/**
 Shared oxlint configuration for Monochromatic repositories (development entry).
 
 Spreads {@link base} and resolves `jsPlugins` to the in-repo plugin TypeScript
 source (`/ts` subpath), so development linting tracks live plugin source with no
 rebuild. The published default export is the prebuilt `index.node.ts`; this file
 is the `./ts` source export.
 
 @example
 ```typescript
 // oxlint.config.ts (monorepo root)
 import { defineConfig } from 'oxlint';
 import base from '\@monochromatic-dev/config-oxlint/ts';
 
 export default defineConfig({ ...base });
 ```

 The `/ts` subpath, not the bare package name: the bare name resolves to the built
 bundle, which is publishing output rather than the source a repository lint should
 track (rule ST3, `doc/decision/workspace-ts-source-imports.md`).
 */

import {
  defineConfig,
  type OxlintConfig,
} from 'oxlint';

import { base, } from './config-base.ts';

/**
 Shared oxlint configuration resolving plugins to TypeScript source.
 
 oxlint's Rust resolver does not understand pnpm workspace package names, so the
 plugin `/ts` source subpaths resolve to absolute paths via `import.meta.resolve()`
 at config evaluation time (Node.js handles workspace resolution).
 */
const config: OxlintConfig = defineConfig({
  ...base,

  // Language server still doesn't support js plugins.
  // Waiting for upstream: https://github.com/oxc-project/oxc/issues/14402 https://github.com/oxc-project/oxc/issues/14826
  jsPlugins: [
    // TSDoc validation rules adapted from eslint-plugin-jsdoc recommended config.
    new URL(import.meta.resolve('@monochromatic-dev/oxlint-plugin-tsdoc/ts',),).pathname,

    // Banned syntax patterns that oxlint's built-in rules can't express.
    new URL(import.meta.resolve('@monochromatic-dev/oxlint-plugin-no-restricted-syntax/ts',),)
      .pathname,

    // TypeScript semantic readonly types and caller-observable mutation contracts.
    new URL(import.meta.resolve('@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type/ts',),)
      .pathname,

    // TypeScript layout enforcement for per-line constructs, semicolons, and expression structure.
    new URL(import.meta.resolve('@monochromatic-dev/oxlint-plugin-stylistic/ts',),).pathname,

    // Tests must import the artifact their package ships, not its source.
    new URL(import.meta.resolve('@monochromatic-dev/oxlint-plugin-test-import/ts',),).pathname,
  ],
},);

export default config;
