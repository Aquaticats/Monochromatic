import { fileURLToPath } from 'node:url';

import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';
import {
  defineConfig,
  type RolldownOptions,
} from 'rolldown';

/**
 Absolute path to the local `sax` stub aliased in place of the blocklisted package.
 */
const saxStub = fileURLToPath(new URL(
  'src/sax-stub.ts',
  import.meta.url
));

/**
 Node build for the key-helper daemon.
 
 Produces a single self-contained ESM bundle that Node SEA embeds. SEA cannot
 resolve filesystem modules at runtime, so every runtime dependency is bundled;
 only `node:` builtins are left external. `@homebridge/dbus-native` speaks to
 the path-style session bus over Node's `net`, so no native addon is bundled.
 No declarations: the SEA bundle has no type consumers.
 
 @example
 ```ts
 export default config;
 ```
 */
const daemonConfig: RolldownOptions = defineConfig({
  input: { index: './src/index.ts' },
  platform: 'node',
  transform: { target: 'node22', },
  resolve: { alias: { sax: saxStub }, },
  external: [
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored protocol prefix test for node builtins; input is one bare import specifier; no repetition, no backtracking.
    /^node:/u,
  ],
  output: {
    dir: 'dist/final/node',
    format: 'es',
    entryFileNames: '[name].mjs',
    chunkFileNames: '[name]-[hash].mjs',
    cleanDir: true,
    minify: {
      compress: true,
      // Mangle breaks func.name and makes output difficult for users to audit.
      mangle: false,
      codegen: true,
    },
  },
});

/**
 Build of `src/testing.ts`, the bundle the unit tests import.

 The tests drive the daemon's logic through built output rather than sibling
 source, as `test-import/require-eventual-artifact` requires, and the
 declarations this build emits are what the TypeScript compiler checks the
 tests against. Every dependency but the `node:` builtins stays bundled, so the
 bundle loads with no module resolution of its own, exactly as the daemon's
 does.

 `cleanDir` is off because raw rolldown cleans per config: this build runs
 after {@link daemonConfig}, and cleaning the shared output folder here would
 delete the daemon bundle.
 
 @example
 ```ts
 import { keysToEvdev } from '../dist/final/node/testing.mjs';
 ```
 */
const testingConfig: NodeFlavorConfig = nodeConfig({
  input: ['./src/testing.ts',],
  external: [
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored protocol prefix test for node builtins; input is one bare import specifier; no repetition, no backtracking.
    /^node:/u,
  ],
  outputOverrides: { cleanDir: false, },
});

/**
 Both builds of this package, in the order they must run: the daemon bundle
 first, because its config clears the shared output folder, then the testing
 bundle, whose config leaves that folder alone.

 @example
 ```ts
 export default config;
 ```
 */
const config: RolldownOptions[] = defineConfig([
  daemonConfig,
  testingConfig,
],);

export default config;
