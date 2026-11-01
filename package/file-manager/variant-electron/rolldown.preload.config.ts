import {
  defineConfig,
  type RolldownOptions,
} from 'rolldown';

/**
 Preload bundle config: Electron's sandboxed preload runtime loads CommonJS
 only, so this is the one CJS island in the package. `main.ts` and
 `build-stage.ts` reference the output as `preload.cjs`, so the entry
 filename template pins that exact name.
 
 @example
 ```ts
 console.log(preloadConfig.output);
 ```
 */
const preloadConfig: RolldownOptions = defineConfig({
  input: ['./src/preload.ts',],
  platform: 'node',
  transform: { target: 'node22', },
  external: [
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored protocol prefix test for node builtins; input is one bare import specifier; no repetition, no backtracking.
    /^node:/,
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored literal name test keeping the electron runtime external; input is one bare import specifier; no repetition, no backtracking.
    /^electron(\/|$)/,
  ],
  output: {
    // Own subdir per bundling config, so a config-level clean can never
    // delete a sibling config's output; build:stage assembles dist/app.
    dir: 'dist/preload',
    format: 'cjs',
    entryFileNames: '[name].cjs',
    cleanDir: true,
    minify: {
      codegen: true,
      compress: false,
      mangle: false,
    },
  },
},);

export default preloadConfig;
