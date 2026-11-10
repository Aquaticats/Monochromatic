import {
  defineConfig,
  type RolldownOptions,
} from 'rolldown';

import { runtimeExternals, } from './src/runtime-externals.ts';

/**
 Package names the Electron tools run against, which the host supplies.
 */
const EXTERNAL_TOOLING_PACKAGES = [
  'electron',
  '@electron/packager',
] as const;

/**
 Build config for package-local Node tools.
 
 @example
 ```ts
 console.log(toolConfig.output);
 ```
 */
const toolConfig: RolldownOptions = defineConfig({
  input: [
    './src/build-stage.ts',
    './src/distribute.ts',
    './src/wayland-boundary-test.ts',
  ],
  platform: 'node',
  transform: { target: 'node22', },
  external: runtimeExternals({ packageNames: EXTERNAL_TOOLING_PACKAGES, }),
  output: {
    dir: 'dist/tools',
    format: 'es',
    entryFileNames: '[name].mjs',
    chunkFileNames: '[name]-[hash].mjs',
    cleanDir: true,
    minify: {
      codegen: true,
      compress: false,
      mangle: false,
    },
  },
});

export default toolConfig;
