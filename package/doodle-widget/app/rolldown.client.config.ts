import {
  clientConfig,
  type ClientFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.client.ts';

/**
 Client-side browser bundle config for the doodle widget.
 Bundles the drawing canvas, background management, and event handling
 modules into a single `dist/client/main.js` for HTML embedding.
 Minified via the shared client flavor since the output is inlined into a
 self-contained HTML file.
 */
const config: ClientFlavorConfig = clientConfig({ input: ['./src/client/main.ts',], },);
export default config;
