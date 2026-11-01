import {
  CLIENT_ALWAYS_BUNDLE,
  clientConfig,
  clientExternalFor,
  type ClientFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.client.ts';

/**
 Client-side browser bundle config for the syllable break demo.
 Bundles the hyphenation library and UI logic into a single `dist/client/main.js`
 for HTML embedding. The `hyphen**` pattern keeps every hyphenation package
 (`hyphen`, `hyphen-en`, and siblings) inline.
 */
const config: ClientFlavorConfig = clientConfig({
  input: ['./src/client/main.ts',],
  external: await clientExternalFor({
    alwaysBundle: [
      ...CLIENT_ALWAYS_BUNDLE,
      'hyphen**',
    ],
  },),
},);
export default config;
