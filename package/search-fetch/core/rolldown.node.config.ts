import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build with the shared API plus the one-time credential migration entry.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    index: './src/index.ts',
    migrate: './src/migrate.ts',
  },
},);

export default config;
