import { join, } from 'node:path';

import {
  copiedAssetsPlugin,
  nodeConfig,
  type NodeFlavorConfig,
  nodeExternal,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Stylesheet the asset module reads beside itself once bundled.
 */
const CLIENT_CSS_SOURCE = join(
  import.meta.dirname,
  'src',
  'client.css',
);

/**
 Output-directory-relative name the copied stylesheet keeps.
 */
const CLIENT_CSS_FILE_NAME = 'client.css';

/**
 Node-side rolldown build for the module tests.

 The single `testing` entry re-exports the behavior the unit tests drive, so
 they load a built artifact instead of sibling source, as
 `test-import/require-eventual-artifact` requires.

 Every dependency stays external so a test and the bundle share one instance of
 each workspace package: the logger's default singleton must be the one the test
 flushes, and a class or schema checked across the boundary must be one object.

 The server keeps running from source (`node src/index.ts`), and the browser
 bundle stays in `rolldown.client.config.ts`.

 @example
 ```ts
 // Consumed by the ordinary build:js:node task.
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: ['./src/testing.ts',],
  external: await nodeExternal({ alwaysBundle: [], },),
  extraPlugins: [copiedAssetsPlugin({
    files: [{
      source: CLIENT_CSS_SOURCE,
      fileName: CLIENT_CSS_FILE_NAME,
    },],
  },),],
},);

export default config;
