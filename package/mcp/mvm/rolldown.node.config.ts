import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/mcp-mvm`.

 Two entries. `index` is the `mvm-mcp` bin, which starts a stdio server loop when
 imported, so a test cannot load it. `exports-for-built-artifact-tests` exists only so
 the unit tests reach this package's tool entries and response builders through built
 output, as `test-import/require-eventual-artifact` requires, rather than through source.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'exports-for-built-artifact-tests': './src/exports-for-built-artifact-tests.ts',
  },
},);

export default config;
