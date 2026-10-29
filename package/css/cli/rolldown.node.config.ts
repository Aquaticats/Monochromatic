import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for `build-tool-css`.
 
 Two entries: the library (`src/index.ts`, emitted as `index.mjs`, the `.`
 export) and the CLI (`src/cli.ts`, emitted as `cli.mjs`, the `build-css`
 bin, shebang preserved). Package `build:css` tasks use inline `node -e`
 dispatch (runWorkspaceNode) to call the built file directly when present,
 otherwise `src/cli.ts`, so bootstrap does not depend on pnpm linking the
 `build-css` shim.
 node-only (optique, node fs via module-fs-path; parsing via the
 workspace css-edit CST).
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
    './src/cli.ts',
  ],
},);

export default config;
