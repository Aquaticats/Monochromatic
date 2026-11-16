import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 The command line that runs one privileged test inside a machine, and the
 library its own unit tests read.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/cli.ts',
    './src/index.ts',
  ],
},);

export default config;
