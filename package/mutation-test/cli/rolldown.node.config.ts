import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for `cli-mutation-test`.
 
 Two entries: library re-exports (`src/index.ts`) and the host CLI
 (`src/cli.ts`). Both land under `dist/final/node/` as `.mjs`.
 `package.json#bin` points at `dist/final/node/cli.mjs` so the bin runs
 without a source-time TypeScript loader. The container-side entrypoint
 is executed from baked source with plain Node type stripping, so it is
 intentionally not a bundle entry.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
    './src/cli.ts',
  ],
},);

export default config;
