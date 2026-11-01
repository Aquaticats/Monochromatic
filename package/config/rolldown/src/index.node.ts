import {
  defineConfig,
  type OutputOptions,
  type Plugin,
  type RolldownOptions,
  VERSION,
} from 'rolldown';
import { dts, } from 'rolldown-plugin-dts';

import { binShebangPlugin, } from './bin-shebang.ts';
import { browserslistTargets, } from './browserslist-targets.ts';
import { checkoutBundlerPlugin, } from './checkout-bundler.ts';
import { copiedAssetsPlugin, } from './copied-assets.ts';
import {
  anchoredExternal,
  packageExternals,
} from './package-externals.ts';
import {
  familyDependencies,
  familyEntrySpecifier,
} from './package-family.ts';
import { shebangExecutablePlugin, } from './shebang-executable.ts';

export {
  type CopiedAsset,
  CopiedAssetReadError,
  copiedAssetsPlugin,
} from './copied-assets.ts';

/**
 Resolved Browserslist targets shared by Node builds, via
 {@link browserslistTargets}.
 */
const target = await browserslistTargets({ runtime: 'node', },);

/**
 Bundle-inclusion patterns for Node builds:
 workspace source plus the ESM-only utility historically inlined
 so committed plugin bundles stay dependency-free.
 */
export const NODE_ALWAYS_BUNDLE: readonly string[] = [
  '@monochromatic-dev/**',
  'nano-spawn',
];

/**
 Bare names kept external even when undeclared:
 pi extension peers are provided by the pi runtime at load time,
 and bundling them duplicates the pi API causing CJS/ESM
 "exports is not defined" errors.
 */
export const NODE_ALWAYS_EXTERNAL: readonly string[] = [
  '@earendil-works/pi-coding-agent',
  'typebox',
  '@earendil-works/pi-ai',
];

/**
 Build the Node flavor external list for a custom bundle-inclusion set.
 
 Combines manifest-derived externals with the pi runtime peers kept
 external even when undeclared.
 
 @param alwaysBundle - Patterns whose matching dependencies must stay inline.
 
 @returns Regex list for rolldown's `external` input option.
 
 @example
 ```ts
 // TODO: deprecate Optique
 external: await nodeExternal({ alwaysBundle: ['\@monochromatic-dev/**', '\@optique/**',], },),
 ```
 */
export async function nodeExternal({ alwaysBundle, }: {
  readonly alwaysBundle: readonly string[];
},): Promise<RegExp[]> {
  return [
    ...await packageExternals({ alwaysBundle, },),
    ...NODE_ALWAYS_EXTERNAL.map(function toMatcher(name: string,): RegExp {
      return anchoredExternal(name,);
    },),
  ];
}

/**
 Plugins shared by every Node build:
 Oxc declarations,
 executable bits for shebang outputs,
 and Node shebang injection for bin-targeted chunks.
 */
const basePlugins = [
  dts({ generator: 'oxc', },),
  shebangExecutablePlugin(),
];

/**
 Shared raw-rolldown options for Node platform builds, without an input.
 
 Bundles workspace dependencies (`@monochromatic-dev/*`) into the output
 so built artifacts are self-contained and work outside the monorepo
 (e.g. Claude Code plugins installed via marketplace).
 
 Selects Oxc declaration generation explicitly;
 the tsgo backend cannot emit declarations for workspace source inlined from
 outside the entry package's tsconfig project
 (see `doc/troubleshooting/rolldown-plugin-dts-typescript-7-generator.md`).
 */
const baseOptions: RolldownOptions = {
  platform: 'node',
  external: await nodeExternal({ alwaysBundle: NODE_ALWAYS_BUNDLE, },),
  transform: { target: [...target,], },
  plugins: basePlugins,
};

/**
 Shared output options for Node builds, without a directory.
 
 `entryFileNames`/`chunkFileNames` force `.mjs` because raw rolldown has no
 `fixedExtension`; `rolldown-plugin-dts` derives `.d.mts` from the template.
 */
const baseOutput = {
  format: 'es' as const,
  entryFileNames: '[name].mjs',
  chunkFileNames: '[name]-[hash].mjs',
  minify: {
    compress: true,
    // Mangle breaks func.name and makes output difficult for users to audit.
    mangle: false,
    codegen: true,
  },
};

/**
 Narrow a flavor input union to its list form.
 
 @param value - Input paths as a list or a named-entry record.
 
 @returns Whether value is the list form.
 
 @example
 ```ts
 isInputList(['./src/index.ts']);
 ```
 */
function isInputList(
  value: readonly string[] | Readonly<Record<string, string>>,
): value is readonly string[] {
  return Array.isArray(value,);
}

/**
 Build one Node flavor config with overridable input and output directory.
 
 Bin-targeted entry chunks get `#!/usr/bin/env node` injected at byte 0
 through {@link binShebangPlugin}, so built CLIs cannot fall through to `/bin/sh`.
 
 @param input - Source input paths; defaults to the package index.
 
 @param outputDir - Output directory; committed Claude Code plugin bundles
   override this to `bundle/node` (see `doc/decision/gitignore-negations.md`).
 
 @param outputOverrides - Shallow output-option overrides for consumers
   needing e.g. `minify: false` or `codeSplitting: false`.
 
 @param external - Replacement external list from {@link nodeExternal} for
   consumers with custom bundle-inclusion patterns.
 
 @param extraPlugins - Plugins appended after the shared ones, such as
   {@link copiedAssetsPlugin} for data files the built output must carry.
 
 @returns Node flavor rolldown config for one self-contained build.
 
 @example
 ```ts
 // rolldown.node.config.ts
 import { nodeConfig, } from '\@monochromatic-dev/config-rolldown/.node.ts';
 export default nodeConfig({ outputDir: 'bundle/node', },);
 ```
 */
export function nodeConfig(
  {
    input = ['./src/index.ts',],
    outputDir = 'dist/final/node',
    outputOverrides = {},
    external,
    extraPlugins = [],
  }: {
    readonly input?: readonly string[] | Readonly<Record<string, string>>;
    readonly outputDir?: string;
    readonly outputOverrides?: OutputOptions;
    readonly external?: readonly RegExp[];
    readonly extraPlugins?: readonly Plugin[];
  } = {},
): RolldownOptions {
  return defineConfig({
    ...baseOptions,
    ...external === undefined ? {} : { external: [...external,], },
    input: isInputList(input,) ? [...input,] : { ...input, },
    plugins: [
      ...basePlugins,
      binShebangPlugin({ outputDir, },),
      ...extraPlugins,
    ],
    output: {
      ...baseOutput,
      dir: outputDir,
      cleanDir: true,
      ...outputOverrides,
    },
  },);
}

/**
 Build a library member of a package family: its family siblings stay
 imports, emitted against each sibling's built entry, while everything else
 follows {@link nodeConfig}.

 Family members check classes by identity across package boundaries, so a
 process must load each member from one file; see
 `src/package-family.ts` for why inlining siblings breaks that.

 The build refuses to run under a bundler other than the version this
 checkout installs; see `src/checkout-bundler.ts`.

 @param familyPrefix - package-name prefix every member of the family shares
 
 @param packageDir - directory holding the building package's package.json; defaults to the build cwd
 
 @param input - source input paths; defaults to the package index
 
 @param outputOverrides - shallow output-option overrides, as {@link nodeConfig} takes them

 @param extraPlugins - plugins appended after the shared ones, such as {@link copiedAssetsPlugin} for a data file the
   member's built module reads beside itself

 @returns Node flavor rolldown config for the library build

 @example
 ```ts
 // rolldown.node.config.ts
 export default await familyLibraryNodeConfig({ familyPrefix: '\@monochromatic-dev/translation-repair-', },);
 ```
 */
export async function familyLibraryNodeConfig({
  familyPrefix,
  packageDir = process.cwd(),
  input = ['./src/index.ts',],
  outputOverrides = {},
  extraPlugins = [],
}: {
  readonly familyPrefix: string;
  readonly packageDir?: string;
  readonly input?: readonly string[] | Readonly<Record<string, string>>;
  readonly outputOverrides?: OutputOptions;
  readonly extraPlugins?: readonly Plugin[];
},): Promise<RolldownOptions> {
  /**
   Family members the building package depends on.
   */
  const family = new Set(await familyDependencies({
    packageDir,
    familyPrefix,
  },),);
  return nodeConfig({
    input,
    external: [
      ...await nodeExternal({ alwaysBundle: NODE_ALWAYS_BUNDLE, },),
      ...[...family,].map(function toMatcher(name: string,): RegExp {
        return anchoredExternal(name,);
      },),
    ],
    outputOverrides: {
      paths: function emittedSpecifier(id: string,): string {
        return familyEntrySpecifier({
          id,
          family,
        },);
      },
      ...outputOverrides,
    },
    // A FAMILY BUILD REFUSES ANOTHER CHECKOUT'S BUNDLER (`checkout-bundler.ts`):
    // family runners print a pipeline digest over their built files, which names
    // a reproducible build only when this checkout's own bundler made them. A
    // family config lists this library build first, and the bundler stops a
    // list at its first failed build, so the refusal also stops the runner
    // builds listed after it.
    extraPlugins: [
      checkoutBundlerPlugin({ installedVersion: VERSION, },),
      ...extraPlugins,
    ],
  },);
}

/**
 Default single-input Node build config via {@link nodeConfig}.
 
 Single-input builds are already self-contained: with one input there is
 nothing to hoist into a shared chunk. Multi-entry plugins must use
 {@link perEntryNodeConfig} instead so each entry stays self-contained.
 
 @example
 ```ts
 // rolldown.node.config.ts
 export { default, } from '\@monochromatic-dev/config-rolldown/.node.ts';
 ```
 */
const _default_1: RolldownOptions = nodeConfig();
export default _default_1;

/**
 Build each input as its own single-input bundle.
 
 A single build with multiple inputs hoists code shared between those
 inputs into a separate chunk whose filename carries a content hash.
 Because these dist bundles are committed and published, that hash churns
 the filename on every content change and leaks an internal chunk into the
 published plugin. Building one input per config removes the shared chunk
 entirely: the shared code inlines into each entry, so every output file is
 self-contained with a stable name.
 
 `cleanDir` stays off for every per-entry config because raw rolldown's
 clean is per-config and not watch-safe: any config cleaning the shared
 `dist/final/node` would delete sibling outputs. The owning mise task
 pre-cleans the directory once instead.
 
 @param entries - Source input paths, one self-contained bundle emitted per path.
 
 @param outputDir - Shared output directory; committed Claude Code plugin
   bundles override this to `bundle/node`.
 
 @returns One single-input rolldown config per entry.
 
 @example
 ```ts
 // rolldown.node.config.ts
 import { perEntryNodeConfig, } from '\@monochromatic-dev/config-rolldown/.node.ts';
 export default perEntryNodeConfig({ entries: ['./src/index.ts', './src/filter.ts',], },);
 ```
 */
export function perEntryNodeConfig(
  {
    entries,
    outputDir = 'dist/final/node',
  }: {
    readonly entries: readonly string[];
    readonly outputDir?: string;
  },
): RolldownOptions[] {
  return defineConfig(entries.map(function toSingleEntryConfig(entry: string,): RolldownOptions {
    return {
      ...baseOptions,
      input: [entry,],
      plugins: [
        ...basePlugins,
        binShebangPlugin({ outputDir, },),
      ],
      output: {
        ...baseOutput,
        dir: outputDir,
        cleanDir: false,
      },
    };
  },),);
}

/**
 Consumer-facing alias for Node flavor config values, so package configs
 can annotate their exported const without depending on rolldown directly.
 */
export type NodeFlavorConfig = RolldownOptions;

/**
 Consumer-facing alias for per-entry Node flavor config arrays.
 */
export type NodeFlavorConfigs = RolldownOptions[];
