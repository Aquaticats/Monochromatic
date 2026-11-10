import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for the unit tests.

 The package's own outputs are a client bundle and one generated HTML file, neither of
 which exposes a module a test can import. This build compiles `src/testing.ts`, the
 re-export surface the unit suites load, to `dist/final/node/testing.mjs` and
 `dist/final/node/testing.d.mts`: the directory the
 `test-import/require-eventual-artifact` rule reads as this package's built artifact.

 `sharp`, which the favicon renderer imports, is a devDependency here, so the shared
 externals list leaves it inside the bundle and its native binding then resolves
 against the bundle's own directory and fails. It is externalized beside the Node
 builtins, so it loads from this package's `node_modules` at run time.

 @example
 ```ts
 export default config;
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: ['./src/testing.ts',],
  external: [
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored protocol prefix test for node builtins; input is one bare import specifier; no repetition, no backtracking.
    /^node:/u,
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored exact package name; input is one bare import specifier; no repetition, no backtracking.
    /^sharp$/u,
  ],
},);

export default config;
