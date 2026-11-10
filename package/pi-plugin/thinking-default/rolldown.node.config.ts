import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/pi-plugin-thinking-default`.

 Two entries. `index` is the pi extension entry, which registers session handlers
 when it loads, so a test cannot import it. `testing` exists only so the model-policy,
 apply, and global-settings suites reach built output, as
 `test-import/require-eventual-artifact` requires, rather than sibling source.

 @example
 ```ts
 import { getThinkingDefaultForModel } from '../dist/final/node/testing.mjs';
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'testing': './src/testing.ts',
  },
},);

export default config;
