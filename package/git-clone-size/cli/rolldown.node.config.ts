import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/git-clone-size-cli`.

 Two entries. `index` is the `git-clone-size` bin, which parses argv and measures when
 it loads, so a test cannot import it. `testing` exists only so the unit suites reach
 the estimators, assembly, exact measurement, and formatting through built output, as
 `test-import/require-eventual-artifact` requires, rather than through source.

 @example
 ```ts
 import { toSize } from '../dist/final/node/testing.mjs';
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'testing': './src/testing.ts',
  },
},);

export default config;
