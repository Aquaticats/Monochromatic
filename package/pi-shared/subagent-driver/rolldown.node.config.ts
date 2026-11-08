import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/pi-shared-subagent-driver`: the library index and the bin.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'drive-pi-subagent': './src/drive-pi-subagent.ts',
  },
},);

export default config;
