import {
  defineConfig,
  type RolldownOptions,
} from 'rolldown';

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
  external: [
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored protocol prefix test for node builtins; input is one bare import specifier; no repetition, no backtracking.
    /^node:/u,
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored literal name test keeping the electron runtime external; input is one bare import specifier; no repetition, no backtracking.
    /^electron(\/|$)/u,
  ],
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
