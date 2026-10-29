import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build with the platform-neutral root entry plus the `./node` subpath
 entry that ships the fs helpers and package-root discovery. Rolldown's
 `node` platform resolves the `#posix-path` and `#root-filesystem` import
 map entries through the `node` condition, so the root entry inlines
 `node:path/posix` and `node:fs/promises` here and nowhere else.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
    './src/node.ts',
  ],
},);

export default config;
