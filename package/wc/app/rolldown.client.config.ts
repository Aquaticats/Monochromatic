import {
  clientConfig,
  type ClientFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.client.ts';

/**
 Client-side browser bundle config for the wc text-stats tool.
 Bundles the input-handling and rendering logic into a single
 `dist/client/main.js` for HTML embedding.
 */
const config: ClientFlavorConfig = clientConfig({ input: ['./src/client/main.ts',], },);
export default config;
