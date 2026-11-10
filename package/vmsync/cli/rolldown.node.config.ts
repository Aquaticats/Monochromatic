import {
  type NodeFlavorConfigs,
  perEntryNodeConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build inlining the argument-parsing stack alongside workspace source,
 so the bin stays runnable without install-time dependencies.
 
 Two entries. `src/index.ts` is the library barrel the built-artifact tests
 import, and `src/cli.ts` is the `vmsync` bin the manifest points at; each is
 its own self-contained bundle, so no shared chunk sits between them.
 */
const config: NodeFlavorConfigs = perEntryNodeConfig({
  entries: [
    './src/index.ts',
    './src/cli.ts',
  ],
},);

export default config;
