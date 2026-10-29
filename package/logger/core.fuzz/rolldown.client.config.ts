import {
  CLIENT_ALWAYS_BUNDLE,
  clientConfig,
  clientExternalFor,
  type ClientFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.client.ts';

/**
 Browser bundle of the browser property layer: fast-check, the neutral
 logger artifact (resolved through the package's `default` export
 condition under the browser platform), and the properties, as one
 self-contained script the Playwright harness page loads. fast-check is
 forced inline because the bundle runs in a page with no module resolver.
 */
const config: ClientFlavorConfig = clientConfig({
  external: await clientExternalFor({
    alwaysBundle: [
      ...CLIENT_ALWAYS_BUNDLE,
      'fast-check',
    ],
  },),
  input: ['./src/browser/properties.ts',],
  platform: 'browser',
},);
export default config;
