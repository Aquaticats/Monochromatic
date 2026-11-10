import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build with the testing entry only.

 The daemon's shipped artifact is the standalone binary the `build:binary` task
 compiles, and the module that binary is compiled from starts the loop on import,
 so it is not an entry a test can load. `src/testing.ts` re-exports the helpers
 the suites reach, and this build is what makes those exports the artifact under
 test rather than the source beside it.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: { testing: './src/testing.ts', },
},);

export default config;
