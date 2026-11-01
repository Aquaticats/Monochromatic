import {
  defineConfig,
  type RolldownOptions,
} from 'rolldown';

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
  external: [
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored protocol prefix test for node builtins; input is one bare import specifier; no repetition, no backtracking.
    /^node:/u,
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored literal name test keeping the electron runtime and its packager external; input is one bare import specifier; no repetition, no backtracking.
    /^(electron|@electron\/packager)(\/|$)/u,
  ],
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
