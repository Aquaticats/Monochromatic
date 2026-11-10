import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/pi-plugin-terminal-title`.

 Two entries. `index` is the pi extension entry, which registers event handlers when
 it loads, so a test cannot import it. `testing` exists only so the title-builder
 suite reaches built output, as `test-import/require-eventual-artifact` requires,
 rather than sibling source.

 @example
 ```ts
 import { titleForTool } from '../dist/final/node/testing.mjs';
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'testing': './src/testing.ts',
  },
},);

export default config;
