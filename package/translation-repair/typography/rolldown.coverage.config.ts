import { join, } from 'node:path';

import {
  copiedAssetsPlugin,
  familyLibraryNodeConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

import { CONFIG_FILE_NAME, } from './src/semantic-wrap-binary-constants.ts';

/**
 Semantic-wrap config the built module reads beside itself.
 */
const WRAP_CONFIG_SOURCE = join(
  import.meta.dirname,
  'src',
  CONFIG_FILE_NAME,
);

/**
 Package-name prefix of the translation-repair family, whose members stay
 imports in a library build so a process loads each from one file.
 */
const FAMILY_PREFIX = '@monochromatic-dev/translation-repair-';

/**
 Library build with source maps and no minification, for the coverage census.

 @example
 ```ts
 // Consumed by the coverage-census:build task.
 ```
 */
const config: Awaited<ReturnType<typeof familyLibraryNodeConfig>> = await familyLibraryNodeConfig({
  familyPrefix: FAMILY_PREFIX,
  outputOverrides: {
    sourcemap: true,
    minify: false,
  },
  extraPlugins: [copiedAssetsPlugin({
    files: [{
      source: WRAP_CONFIG_SOURCE,
      fileName: CONFIG_FILE_NAME,
    },],
  },),],
},);

export default config;
