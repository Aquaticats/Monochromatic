import {
  defineConfig,
  type RolldownOptions,
} from 'rolldown';

import { runtimeExternals, } from './src/runtime-externals.ts';

/**
 Package name of the Electron runtime that loads the bundle.
 */
const ELECTRON_PACKAGE = 'electron';

/**
 Main-process bundle config for Electron's Node-flavoured ESM runtime.
 
 @example
 ```ts
 console.log(mainProcessConfig.output);
 ```
 */
const mainProcessConfig: RolldownOptions = defineConfig({
  input: ['./src/main.ts',],
  platform: 'node',
  transform: { target: 'node22', },
  external: runtimeExternals({ packageNames: [ELECTRON_PACKAGE,], }),
  output: {
    dir: 'dist/app',
    format: 'es',
    entryFileNames: '[name].mjs',
    chunkFileNames: '[name]-[hash].mjs',
    // dist/app is shared with tsc-emitted app files assembled by
    // build:stage, so this config never cleans it.
    cleanDir: false,
    minify: {
      codegen: true,
      compress: false,
      mangle: false,
    },
  },
});

export default mainProcessConfig;
