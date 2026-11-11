import {
  perEntryNodeConfig,
  type NodeFlavorConfigs,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Single-entry extension build for the Qwen Token Plan Individual pi plugin package.
 */
const config: NodeFlavorConfigs = perEntryNodeConfig({
  entries: [
    './src/index.ts',
  ],
},);

export default config;
