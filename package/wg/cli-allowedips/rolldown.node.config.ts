import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Self-contained production CLI bundle.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: ['./src/index.ts',],
},);

export default config;
