import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build with the public entry, the internal testing seam, and the edit
 CLI. The CLI chunk is shebang-injected from the package `bin` mapping, so
 `dist/final/node/bin.mjs` runs directly under node.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    index: './src/index.ts',
    testing: './src/testing.ts',
    bin: './src/bin/env-local-json.ts',
  },
},);

export default config;
