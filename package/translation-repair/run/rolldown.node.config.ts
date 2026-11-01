import { fileURLToPath, } from 'node:url';

import {
  copiedAssetsPlugin,
  familyLibraryNodeConfig,
  nodeConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';
import { CONFIG_FILE_NAME, } from '@monochromatic-dev/translation-repair-typography/ts/semantic-wrap-binary-constants.ts';

import { nodeEntries, } from './src/build-entries.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

/**
 Semantic-wrap config of the typography package, which the runner bundles inline and read beside themselves.
 */
const WRAP_CONFIG_SOURCE = fileURLToPath(import.meta.resolve(`@monochromatic-dev/translation-repair-typography/ts/${CONFIG_FILE_NAME}`,),);

/**
 Package-name prefix of the translation-repair family, whose members stay
 imports in a library build so a process loads each from one file.
 */
const FAMILY_PREFIX = '@monochromatic-dev/translation-repair-';

/**
 The library index first, with family siblings left as imports, then every runner with everything inlined, so a
 runner's pipeline digest over `dist/final/node` covers every byte it executes. Rolldown builds a list of configs in
 order, and only the first cleans the output directory.

 @example
 ```ts
 // Consumed by the ordinary package build task.
 ```
 */
const config: readonly ReturnType<typeof nodeConfig>[] = [
  await familyLibraryNodeConfig({
    familyPrefix: FAMILY_PREFIX,
  },),
  nodeConfig({
    input: Object.fromEntries([...nodeEntries,].filter(function isRunner([entry,]: ReadonlyEntry<string, string>,): boolean {
      return entry !== 'index';
    },),),
    outputOverrides: {
      cleanDir: false,
    },
    extraPlugins: [copiedAssetsPlugin({
      files: [{
        source: WRAP_CONFIG_SOURCE,
        fileName: CONFIG_FILE_NAME,
      },],
    },),],
  },),
];

export default config;
