import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build configuration: a single `src/index.ts` entry that re-exports every
 adapter module (catalog, provider stream, transcript, ranking policy, tool,
 command, plugin entry), so the package emits only `dist/final/node/index.mjs`.
 Consumers import source through the `./ts` subpath (ST3), so no per-module
 bundle entries exist. `@deepseek-ai/*` and `pi-ai` stay external via the base
 config's dependency-driven `packageExternals`, provided by the DSH runtime or
 the consuming harness.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
  ],
},);

export default config;
