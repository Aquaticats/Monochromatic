import {
  familyLibraryNodeConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Package-name prefix of the translation-repair family, whose members stay
 imports in a library build so a process loads each from one file.
 */
const FAMILY_PREFIX = '@monochromatic-dev/translation-repair-';

/**
 Library build: the package index, with family siblings left as imports.

 @example
 ```ts
 // Consumed by the ordinary package build task.
 ```
 */
const config: Awaited<ReturnType<typeof familyLibraryNodeConfig>> = await familyLibraryNodeConfig({
  familyPrefix: FAMILY_PREFIX,
},);

export default config;
