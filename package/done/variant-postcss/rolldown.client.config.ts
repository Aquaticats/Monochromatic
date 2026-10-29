import {
  clientConfig,
  type ClientFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.client.ts';
import {
  importAttributesPlugin,
} from '@monochromatic-dev/rolldown-plugin-import-attributes/ts';

/**
 Client-side browser bundle config for the Done productivity app.
 Bundles five page entry points into `dist/client/*.js`.
 Uses the import-attributes plugin so client code can import
 pre-built CSS via `with { type: 'text' }`. The CSS parsing stack
 (`@csstools/css-tokenizer` via workspace css-edit) is undeclared in
 dependencies, so it bundles by omission; declared server-only deps
 never enter the client graph.
 */
const config: ClientFlavorConfig = clientConfig({
  input: [
    './src/client/inbox.ts',
    './src/client/in-progress.ts',
    './src/client/task-details.ts',
    './src/client/search.ts',
    './src/client/settings.ts',
  ],
  platform: 'browser',
  extraPlugins: [importAttributesPlugin(),],
},);
export default config;
