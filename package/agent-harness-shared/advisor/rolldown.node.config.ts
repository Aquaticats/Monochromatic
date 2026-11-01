import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build configuration: a single `src/index.ts` entry that re-exports
 every core module (ports, config, selection, context, completion, operation,
 rendering), so the package emits only `dist/final/node/index.mjs`. Consumers
 import source through the `./ts` subpath (ST3), so no per-module bundle
 entries exist. `pi-ai` and `typebox` stay external via the base config's
 `neverBundle`, provided by the consuming harness.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
  ],
},);

export default config;
