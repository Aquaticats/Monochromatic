import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for `cli-unused-export`.
 
 Two entries: library re-exports (`src/index.ts`) and the CLI
 (`src/cli.ts`). Both land under `dist/final/node/` as `.mjs`.
 `package.json#bin` points at `dist/final/node/cli.mjs` so the bin runs
 without a source-time TypeScript loader.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
    './src/cli.ts',
  ],
},);

export default config;
