/**
 Shared oxlint configuration for Monochromatic repositories (built Node entry).
 
 Spreads {@link base} and points `jsPlugins` at the co-located sidecar plugin
 bundles (`plugin-*.mjs`) emitted next to this file by `rolldown.node.config.ts`.
 Using relative `file://` URLs avoids any plugin resolution at lint time, which is
 the optimization tracked in issue #238. This is the package default export
 (`dist/final/node/index.mjs`).
 */

import {
  defineConfig,
  type OxlintConfig,
} from 'oxlint';

import { base, } from './config-base.ts';

/**
 Shared oxlint configuration pointing at prebuilt co-located plugin sidecars.
 
 oxlint 1.67.0 only accepts `jsPlugins` entries resolvable through `oxc_resolver`
 (`string | { name, specifier }`); a `file://` URL resolves to the sidecar path,
 while an in-memory plugin object is rejected.
 */
const config: OxlintConfig = defineConfig({
  ...base,
  jsPlugins: [
    new URL(
      'plugin-tsdoc.mjs',
      import.meta.url,
    ).href,
    new URL(
      'plugin-no-restricted-syntax.mjs',
      import.meta.url,
    ).href,
    new URL(
      'plugin-prefer-readonly-parameter-type.mjs',
      import.meta.url,
    ).href,
    new URL(
      'plugin-stylistic.mjs',
      import.meta.url,
    ).href,
    new URL(
      'plugin-test-import.mjs',
      import.meta.url,
    ).href,
  ],
},);

export default config;
