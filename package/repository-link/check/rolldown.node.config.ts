import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/repository-link-check`.

 Entries are the library index the unit tests import and the three bins. Each
 bin chunk is one self-contained file (node built-ins only), so a copy of it
 outside the repository still runs when the repository is away.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'check': './src/check.ts',
    'run': './src/run.ts',
    'record': './src/record.ts',
  },
},);

export default config;
