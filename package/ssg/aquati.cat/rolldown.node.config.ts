import { join, } from 'node:path';

import {
  copiedAssetsPlugin,
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Codepoints table the icon module reads beside itself at import time.
 */
const ICON_CODEPOINTS_FILE = 'material-symbols-outlined.codepoints';

/**
 Absolute path of the icon codepoints file the bundle must find beside itself.
 */
const ICON_CODEPOINTS_SOURCE = join(
  import.meta.dirname,
  'src',
  'lib',
  'icon',
  ICON_CODEPOINTS_FILE,
);

/**
 Node-side rolldown build for the SSG library surface.

 One entry, `src/index.ts`, which re-exports the render, content, and style
 modules the unit tests load. Output lands in `dist/final/node/` as `.mjs` and
 `.d.mts`, the directory the `test-import/require-eventual-artifact` rule reads
 as this package's built artifact.

 The icon module reads its codepoints table from a path resolved against
 `import.meta.dirname`, so the table is copied beside the bundle; without it the
 artifact throws `ENOENT` on the first import.

 The site keeps running from source (`node src/build.ts`), and the browser
 bundle stays in `rolldown.client.config.ts`; this build exists so unit tests
 exercise a bundle instead of sibling source.
 */
const config: NodeFlavorConfig = nodeConfig({
  extraPlugins: [copiedAssetsPlugin({
    files: [{
      source: ICON_CODEPOINTS_SOURCE,
      fileName: ICON_CODEPOINTS_FILE,
    },],
  },),],
},);

export default config;
