import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for the pnpm workspace catalog reader.
 
 @example
 ```ts
 // The package task loads this config through rolldown.
 export default config;
 ```
 */
const config: NodeFlavorConfig = nodeConfig({
  input: ['./src/index.ts',],
},);

export default config;
