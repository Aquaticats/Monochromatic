import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build configuration: a single `src/index.ts` entry that re-exports
 every feature module (core, scope, cost, budget, pi-coding-agent), so the
 package emits only `dist/final/node/index.mjs`. The former per-feature
 bundle entries were dropped when the export map collapsed to `.` + `./ts`
 (no consumer imports a feature subpath). pi peer deps stay external via the
 base config's `neverBundle`.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
  ],
},);

export default config;
