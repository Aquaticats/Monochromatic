import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/repository-move-cli`.

 Entries are the library index the unit tests import and the two bins the
 guided root script runs. Each bin chunk gets its shebang from the
 `package.json#bin` mapping.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'move-repository': './src/move-repository.ts',
    'list-uncommitted-work': './src/list-uncommitted-work.ts',
    'run-repository-move': './src/run-repository-move.ts',
  },
},);

export default config;
