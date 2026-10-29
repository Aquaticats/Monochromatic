import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build with the platform-neutral root entry plus the `./node` subpath
 entry that ships the file sink. Code both entries share (the sink and its
 static `node:fs/promises` import) lands in a hashed chunk. Rolldown's
 `node` platform resolves the `#default-sinks` import map entry through the
 `node` condition, so the root entry inlines the Node default sink list.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
    './src/node.ts',
  ],
},);

export default config;
