import {
  familyLibraryNodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build that keeps the store package an import instead of inlining it.

 `memoize` and `memoizeAsync` compare store reads against the `ABSENT` sentinel by
 identity, so a caller passing a store built by `@monochromatic-dev/kv-store-core`
 must load that one module instance. Inlining the store package would give the
 artifact a second copy, whose miss sentinel never equals the caller's, and every
 miss would read as a cached value. The family library build keeps the sibling as
 an import emitted against its built entry, which is also the entry the unit tests
 load beside the artifact.

 @example
 ```ts
 // Consumed by the build:js:node task.
 ```
 */
const config: NodeFlavorConfig = await familyLibraryNodeConfig({
  familyPrefix: '@monochromatic-dev/kv-store-',
},);

export default config;
