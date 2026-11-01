import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build with the public entry and the `cachyos-image` CLI. The CLI chunk is shebang-injected from the
 package `bin` mapping, so `dist/final/node/bin.mjs` runs directly under node.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    index: './src/index.ts',
    bin: './src/bin/cachyos-image.ts',
  },
},);

export default config;
