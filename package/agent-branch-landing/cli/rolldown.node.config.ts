import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/agent-branch-landing-cli`: the library index a test reads and the one bin.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'land-agent-branches': './src/land-agent-branches.ts',
  },
},);

export default config;
