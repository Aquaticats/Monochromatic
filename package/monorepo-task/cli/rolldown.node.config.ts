import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for `dev-script-task-cli`.

 One entry: the `task-run` CLI. It lands under `dist/final/node/` as `.mjs`
 with its command-line shebang preserved. CI invokes the built file when
 present and the TypeScript source otherwise (node runs TypeScript directly),
 so a checkout with no built artifacts still runs the task graph.
 node-only (node:fs/node:path/node:child_process; no browser bindings).
 `package.json#bin` resolves to the built file for normal shell use.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/run.ts',
  ],
},);

export default config;
