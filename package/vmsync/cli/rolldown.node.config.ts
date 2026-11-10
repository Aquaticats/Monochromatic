import {
  nodeConfig,
  nodeExternal,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/vmsync-cli`.

 Two entries. `index` is the `vmsync` bin, which parses argv and dispatches, so a
 test cannot load it. `testing` exists so the unit suites reach the memory grammar,
 the config helpers, the dd argument vector and the whitespace scanner through built
 output, as `test-import/require-eventual-artifact` requires, rather than through
 source. Workspace source and the argument-parsing stack are inlined, as the bin's
 build does, so the bundle loads with no install-time resolution of its own.

 @example
 ```ts
 import { parseMemoryToBytes } from '../dist/final/node/testing.mjs';
 ```
 */
const external = await nodeExternal({
  alwaysBundle: [
    '@monochromatic-dev/**',
    // TODO: deprecate Optique
    '@optique/**',
    'nano-spawn',
  ],
},);

/**
 Both builds of this package, the bin and the entry its unit suites import.

 @example
 ```ts
 export default config;
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'testing': './src/testing.ts',
  },
  external,
},);

export default config;
