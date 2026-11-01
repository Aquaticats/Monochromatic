import {
  type NodeFlavorConfigs,
  perEntryNodeConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Self-contained generation, injected-lookup, and ASN database bundles.
 */
const config: NodeFlavorConfigs = perEntryNodeConfig({
  entries: [
    './src/generate.ts',
    './src/generate-with-lookup.ts',
    './src/asn-network.ts',
    './src/asn-networks.ts',
  ],
},);

export default config;
