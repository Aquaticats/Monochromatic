import {
  nodeConfig,
  nodeExternal,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build inlining the argument-parsing stack alongside workspace source,
 so the bin stays runnable without install-time dependencies.
 */
const config: NodeFlavorConfig = nodeConfig({
  external: await nodeExternal({
    alwaysBundle: [
      '@monochromatic-dev/**',
      // TODO: deprecate Optique
      '@optique/**',
      'nano-spawn',
    ],
  },),
},);

export default config;
