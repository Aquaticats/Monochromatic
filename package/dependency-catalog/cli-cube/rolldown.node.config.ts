import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for `deps-cube`.
 
 Two entries: the library re-exports (`src/index.ts`) and the CLI
 (`src/cli.ts`). Both land under `dist/final/node/` as `.mjs` (via
 the shared config's `fixedExtension: true`). `package.json#bin`
 points at `dist/final/node/cli.mjs` so the bin runs without a
 source-time TypeScript loader.
 
 The browser-side `src/script/controller.ts` is NOT a rolldown entry:
 it is bundled at run time by rolldown inside `render-html.ts`,
 with the source path resolved from `PACKAGE_ROOT`. The package's
 `src/` is shipped via `files: ["src"]` so the runtime bundler can
 still see the controller source after a build.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
    './src/cli.ts',
  ],
},);

export default config;
