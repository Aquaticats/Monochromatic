/**
 Bundles a consumer of the neutral artifact for actual browser acceptance tests. @module
 */
import {
  clientConfig,
  type ClientFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.client.ts';

/**
 Browser fixture dependencies are bundled; lazy Node-only branches remain unreachable external imports.
 */
const config: ClientFlavorConfig = {
  ...clientConfig({
    input: ['./src/sinon-browser-fixture.ts',],
    platform: 'browser',
    external: [],
  },),
  external(id: string,): boolean {
    return id.startsWith('node:',);
  },
};

export default config;
