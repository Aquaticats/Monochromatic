import {
  perEntryNodeConfig,
  type NodeFlavorConfigs,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Single loadable Pi extension entry.
 */
const config: NodeFlavorConfigs = perEntryNodeConfig({ entries: ['./src/index.ts',], },);

export default config;
