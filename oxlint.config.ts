/**
 Root oxlint configuration for the Monochromatic monorepo.
 
 Spreads the shared config from `\@monochromatic-dev/config-oxlint/ts`.
 oxlint auto-discovers this file via `oxlint.config.ts` convention.

 The `/ts` subpath is the package's source entry: it points `jsPlugins` at the
 plugin TypeScript source, so a lint run reads live plugin code and never a
 `dist` bundle (rule ST3, `doc/decision/workspace-ts-source-imports.md`).
 
 Uses spread instead of `extends` because `extends` only merges rules:
 top-level fields like `categories`, `env`, `ignorePatterns`, `overrides`,
 and `plugins` are not inherited.
 */

import {
  defineConfig,
  type OxlintConfig,
} from 'oxlint';

import base from '@monochromatic-dev/config-oxlint/ts';

/**
 Root oxlint configuration with an explicit public export type for isolated declarations.
 */
const config: OxlintConfig = defineConfig({
  ...base,
},);

export default config;
