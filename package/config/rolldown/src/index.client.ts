import { join, } from 'node:path';

import {
  defineConfig,
  type RolldownOptions,
} from 'rolldown';

import { browserslistTargets, } from './browserslist-targets.ts';
import { packageExternals, } from './package-externals.ts';

/**
 Resolved Browserslist targets shared by client bundles, via
 {@link browserslistTargets}.
 */
const target = await browserslistTargets({ runtime: 'browser', },);

/**
 Bundle-inclusion patterns for client bundles:
 workspace source plus the heavy browser libraries that must ride inside
 the self-contained script.
 */
export const CLIENT_ALWAYS_BUNDLE: readonly string[] = [
  '@monochromatic-dev/**',
  '@lezer/**',
  'lezer-**',
  'jspdf',
];

/**
 Build the client flavor external list for a custom bundle-inclusion set.
 
 @param alwaysBundle - Patterns whose matching dependencies must stay inline.
 
 @returns Regex list for rolldown's `external` input option.
 
 @example
 ```ts
 external: await clientExternalFor({ alwaysBundle: [...CLIENT_ALWAYS_BUNDLE, 'hyphen**',], },),
 ```
 */
export async function clientExternalFor({ alwaysBundle, }: {
  readonly alwaysBundle: readonly string[];
},): Promise<RegExp[]> {
  return await packageExternals({ alwaysBundle, },);
}

/**
 Externals shared by every client bundle produced from the consuming
 package's manifest at import time.
 */
const clientExternal = await clientExternalFor({ alwaysBundle: CLIENT_ALWAYS_BUNDLE, },);

/**
 Build one client bundle config with overridable inputs.
 
 Bundles everything reachable (including undeclared transitives) into
 self-contained browser scripts served as `<script type="module">` tags.
 Output goes to `dist/client/` with plain `.js` extensions matching
 existing HTML references. No declarations: client bundles have no type
 consumers.
 
 @param input - Client entry paths; defaults to the package client index.
 
 @param platform - Resolution platform; browser apps override neutral.
 
 @param extraPlugins - Additional rolldown plugins, e.g. import-attributes
   for `with { type: 'text' }` CSS imports.
 
 @param external - Replacement external list from {@link clientExternalFor}
   for consumers with custom bundle-inclusion patterns.
 
 @returns Client flavor rolldown config producing self-contained scripts.
 
 @example
 ```ts
 // rolldown.client.config.ts
 import { clientConfig, } from '\@monochromatic-dev/config-rolldown/.client.ts';
 export default clientConfig({ input: ['./src/client/main.ts',], },);
 ```
 */
export function clientConfig(
  {
    input = ['./src/client.ts',],
    platform = 'neutral',
    extraPlugins = [],
    external,
  }: {
    readonly input?: readonly string[];
    readonly platform?: 'neutral' | 'browser';
    readonly extraPlugins?: RolldownOptions['plugins'];
    readonly external?: readonly RegExp[];
  } = {},
): RolldownOptions {
  return defineConfig({
    input: [...input,],
    plugins: extraPlugins,
    platform,
    resolve: {
      mainFields: [
        'module',
        'main',
      ],
      alias: {
        // jspdf lazy-imports canvg inside addSvgAsImage(); client apps never
        // call it, so resolve the dynamic import to the empty stub instead of
        // bundling canvg and its custom-licensed rgbcolor dependency.
        canvg: join(
          import.meta.dirname,
          'stub',
          'canvg.ts',
        ),
      },
    },
    external: external === undefined ? clientExternal : [...external,],
    transform: { target: [...target,], },
    output: {
      dir: 'dist/client',
      format: 'es',
      entryFileNames: '[name].js',
      chunkFileNames: '[name]-[hash].js',
      cleanDir: true,
      minify: {
        compress: true,
        // Mangle breaks func.name and makes output difficult for users to audit.
        mangle: false,
        codegen: true,
      },
    },
  },);
}

/**
 Default single-input client bundle config via {@link clientConfig}.
 */
const _default_1: RolldownOptions = clientConfig();
export default _default_1;

/**
 Consumer-facing alias for client flavor config values, so package configs
 can annotate their exported const without depending on rolldown directly.
 */
export type ClientFlavorConfig = RolldownOptions;
