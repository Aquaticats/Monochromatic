import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node-side rolldown build for `pi-spawn`.
 
 Builds the Pi extension (`src/index.ts`), the `spawn-pi` CLI (`src/cli.ts`), and the
 entry the unit suites import (`src/testing.ts`). `package.json#bin` points at
 `dist/final/node/cli.mjs` so command execution uses Node without relying on a
 TypeScript source loader. The testing entry exists because the extension entry
 registers handlers on load, so no suite can import it.
 
 @example
 ```typescript
 export default config;
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
    './src/cli.ts',
    './src/testing.ts',
  ],
},);

export default config;
