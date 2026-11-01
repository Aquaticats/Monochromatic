import {
  clientConfig,
  type ClientFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.client.ts';

/**
 Client-side browser bundle config for the Done h-css test variant.
 Bundles five page entry points into `dist/client/*.js`.
 No import-attributes plugin needed: h-css generates styles at runtime.
 */
const config: ClientFlavorConfig = clientConfig({
  input: [
    './src/client/inbox.ts',
    './src/client/in-progress.ts',
    './src/client/task-details.ts',
    './src/client/search.ts',
    './src/client/settings.ts',
  ],
},);
export default config;
