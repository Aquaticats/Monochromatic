import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build of the widget's testable surface.

 One entry, `src/index.ts`, which re-exports the stroke state, drawing
 configuration, and eraser modules. Output lands in `dist/final/node/` as
 `.mjs` and `.d.mts`, the directory `test-import/require-eventual-artifact`
 reads as this package's built artifact.

 The widget itself keeps shipping from `node src/build.ts` (the single HTML
 file) and from the browser bundle in `rolldown.client.config.ts`; this build
 exists so unit tests exercise a bundle instead of sibling source.
 */
const config: NodeFlavorConfig = nodeConfig({},);

export default config;
