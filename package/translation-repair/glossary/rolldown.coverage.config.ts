import {
  familyLibraryNodeConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

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
},);

export default config;
