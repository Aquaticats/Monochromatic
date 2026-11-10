import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/android-exempt-unused-cli`.

 Two entries. `index` is the CLI bin, which drives an interactive prompt when it
 loads, so a test cannot import it. `testing` exists only so the parser and diff
 suites reach built output, as `test-import/require-eventual-artifact` requires,
 rather than sibling source.

 @example
 ```ts
 import { parseDevices } from '../dist/final/node/testing.mjs';
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'testing': './src/testing.ts',
  },
},);

export default config;
