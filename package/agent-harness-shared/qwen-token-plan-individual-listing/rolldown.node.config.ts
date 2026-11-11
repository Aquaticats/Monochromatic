import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build configuration: a single `src/index.ts` entry that re-exports every
 core module (constants, thinking-level map, model facts and their data table,
 family keys, listing mapping, listing snapshot, listing fetch), so the package
 emits only `dist/final/node/index.mjs`. Consumers import source through the
 `./ts` subpath (ST3), so no per-module bundle entries exist. The pi-ai builtin
 catalog fixture stays unbundled: tests and the pi plugin reach it through the
 `./test-fixtures` subpath. This package has no Pi and no DSH dependencies.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
  ],
},);

export default config;
