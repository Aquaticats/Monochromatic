import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/test-harness-core`.

 Two entries. `index` is the package entry, shared with the neutral build, so it
 carries no `node:` import and exposes no scratch root. `testing` exists only so
 the tests of the node-only scratch root reach built output, as
 `test-import/require-eventual-artifact` requires, rather than sibling source.

 @example
 ```ts
 import { scratchRoot } from '../dist/final/node/testing.mjs';
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'testing': './src/testing.ts',
  },
},);

export default config;
