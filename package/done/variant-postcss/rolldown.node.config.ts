import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for the Done postcss variant's server library.

 One entry, `src/index.ts`, which re-exports the request parsing and schema
 migration modules the unit tests load. Output lands in `dist/final/node/` as
 `.mjs` and `.d.mts`, the directory the `test-import/require-eventual-artifact`
 rule reads as this package's built artifact.

 The server keeps running from source (`node src/server.ts`), the postcss
 client bundle stays in `rolldown.client.config.ts`, and the stylesheet build
 stays in `mise.toml`; this build exists so unit tests exercise a bundle
 instead of sibling source.
 */
const config: NodeFlavorConfig = nodeConfig();

export default config;
