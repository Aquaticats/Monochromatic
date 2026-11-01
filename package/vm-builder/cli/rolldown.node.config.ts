import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for `dev-script-vm-builder`.
 
 The package has no `src/index.ts`; its declared `bin` is
 `src/build-and-import.ts`, so the entry is overridden to that file. It
 emits `dist/final/node/build-and-import.mjs` (shebang preserved). Uses
 `node:child_process`/`node:fs` and shells out to sudo/podman/virsh, so
 the bundle is node-only. The auxiliary `import.ts`/`sign-and-push.ts`
 dev scripts are not bins and stay source-run via their mise tasks.

 `src/exec.ts` is a second entry only so that `dist/final/node/exec.mjs`
 exists for the unit tests, which import built output; the bin bundle
 runs sudo/podman at import time and cannot be imported by a test.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/build-and-import.ts',
    './src/exec.ts',
  ],
},);

export default config;
