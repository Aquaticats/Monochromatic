import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/pi-plugin-statusline`.

 Two entries. `index` is the pi extension entry, which registers event handlers when
 it loads, so a test cannot import it. `testing` exists only so the rate-limit and
 usage-warning suites reach built output, as `test-import/require-eventual-artifact`
 requires, rather than sibling source.

 @example
 ```ts
 import { parseRateLimitSnapshots } from '../dist/final/node/testing.mjs';
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'testing': './src/testing.ts',
  },
},);

export default config;
