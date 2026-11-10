import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/fy-cli`.

 Two entries. `index` is the `cli-fy` bin, which parses `process.argv` and exits,
 so a test cannot load it. `testing` exists only so the unit tests reach the
 argument coercion and specifier resolution through built output, as
 `test-import/require-eventual-artifact` requires, rather than through source.

 @example
 ```ts
 import { coerceArg } from '../dist/final/node/testing.mjs';
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'testing': './src/testing.ts',
  },
},);

export default config;
