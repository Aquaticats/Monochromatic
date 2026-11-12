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
    'save-the-final-message-of-a-review-that-wrote-no-review-file':
      './src/save-the-final-message-of-a-review-that-wrote-no-review-file.ts',
    'list-processes-whose-command-line-holds-text': './src/list-processes-whose-command-line-holds-text.ts',
    'list-running-units-whose-worktree-is-gone': './src/list-running-units-whose-worktree-is-gone.ts',
    'print-the-final-report-of-the-newest-run-of-each-agent':
      './src/print-the-final-report-of-the-newest-run-of-each-agent.ts',
    'list-launchers-whose-unit-has-ended': './src/list-launchers-whose-unit-has-ended.ts',
    'remove-finished-worktrees': './src/remove-finished-worktrees.ts',
  },
},);

export default config;
