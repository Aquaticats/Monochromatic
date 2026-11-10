import {
  perEntryNodeConfig,
  type NodeFlavorConfigs,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Separate library and shebang-bearing CLI bundles.
 */
const config: NodeFlavorConfigs = perEntryNodeConfig({
  entries: [
    './src/index.ts',
    './src/cli.ts',
  ],
},);

export default config;
