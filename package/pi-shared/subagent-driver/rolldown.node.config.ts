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
    'start-pi-run-in-a-free-slot': './src/start-pi-run-in-a-free-slot.ts',
    'retry-start-pi-run-until-a-slot-frees': './src/retry-start-pi-run-until-a-slot-frees.ts',
    'start-a-batch-of-pi-runs': './src/start-a-batch-of-pi-runs.ts',
    'latest-pi-run-reports': './src/latest-pi-run-reports.ts',
    'list-recent-pi-run-ends': './src/list-recent-pi-run-ends.ts',
    'send-note-to-running-pi-runs': './src/send-note-to-running-pi-runs.ts',
    'stop-pi-runs-not-on-the-wrapper': './src/stop-pi-runs-not-on-the-wrapper.ts',
    'start-kimi-reviews': './src/start-kimi-reviews.ts',
  },
},);

export default config;
