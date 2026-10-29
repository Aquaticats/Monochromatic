import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for `cli-terminal-exec`.
 
 Two entries, inverted from the usual lib/bin split: the LIBRARY entry is
 `src/launch.ts` (spawns a detached terminal, returns) and the BIN entry
 is `src/index.ts` (replaces the process via execvp). They emit
 `dist/final/node/launch.mjs` (lib, the `.` export) and `index.mjs`
 (bin). Uses `node:child_process`, so the bundle is node-only.
 `package.json#bin` points at `dist/final/node/index.mjs`.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/launch.ts',
    './src/index.ts',
  ],
},);

export default config;
