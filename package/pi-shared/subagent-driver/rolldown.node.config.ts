import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Node build for `@monochromatic-dev/pi-shared-subagent-driver`: the library index and the bin.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: {
    'index': './src/index.ts',
    'drive-pi-subagent': './src/drive-pi-subagent.ts',
    'watch-overdue-pi-runs': './src/watch-overdue-pi-runs.ts',
    'check-in-on-pi-runs': './src/check-in-on-pi-runs.ts',
    'start-pi-run-when-a-pool-has-room': './src/start-pi-run-when-a-pool-has-room.ts',
    'start-pi-run-in-a-worktree': './src/start-pi-run-in-a-worktree.ts',
    'retry-pi-run-start-until-a-pool-has-room': './src/retry-pi-run-start-until-a-pool-has-room.ts',
    'start-pi-runs-from-a-batch-list': './src/start-pi-runs-from-a-batch-list.ts',
  },
},);

export default config;
