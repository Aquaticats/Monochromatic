import { fileURLToPath, } from 'node:url';

import {
  copiedAssetsPlugin,
  type NodeFlavorConfigs,
  nodeConfig,
  perEntryNodeConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

import { CLEARNET_HOSTS_DATA_FILENAME, } from './src/clearnet-hosts-data-file.ts';

/**
 Bundle of the detached route watcher,
 which reads the clearnet host list again at every refresh and so needs a config of its own.
 */
const WATCHER_ENTRY = './src/bypass-watch.ts';

/**
 Source of the clearnet host lists,
 the one data file the build emits besides the bundles.
 */
const CLEARNET_HOSTS_DATA_SOURCE = fileURLToPath(new URL(
  `src/${CLEARNET_HOSTS_DATA_FILENAME}`,
  import.meta.url,
),);

/**
 Every self-contained production CLI and public tunnel-orchestration bundle.
 */
const ENTRIES: readonly string[] = [
  './src/index.ts',
  './src/errors.ts',
  './src/application-exemption.ts',
  './src/application-exemption-command.ts',
  './src/privilege-context-data.ts',
  './src/policy-routing-conflict.ts',
  './src/exempt-mark-reservation.ts',
  './src/opensnitch.ts',
  './src/opensnitch-config-tree.ts',
  './src/opensnitch-live.ts',
  './src/opensnitch-daemon-config.ts',
  './src/tunnel-bypass-hosts.ts',
  './src/tunnel-bypass-hosts-sync.ts',
  './src/bypass-host-refresh.ts',
  './src/bypass-watch-failure.ts',
  './src/run-subcommand.ts',
  './src/escape-text.ts',
  './src/notification-user.ts',
  './src/failure-repeat-limit.ts',
  './src/runtime-failure-report.ts',
  './src/host-answer-filter.ts',
  './src/host-rule-limits.data.ts',
  './src/map-in-chunks.ts',
  './src/tunnel-dependencies.ts',
  './src/tunnel-dns-lookup.ts',
  './src/update-policy-notice.ts',
  './src/update-hosts.data.ts',
  './src/provider-hosts.data.ts',
  './src/installed-clearnet-hosts.ts',
  './src/opensnitch-toolchain-rule-patterns.ts',
  './src/opensnitch-toolchain-rules.ts',
  './src/opensnitch-rules-command.ts',
  './src/tunnel.ts',
  './src/tunnel-up.ts',
  './src/config.ts',
  './src/tunnel-bypass.ts',
  './src/tunnel-bypass-route.ts',
  './src/tunnel-bypass-route-shape.ts',
  './src/bypass-watch.ts',
  './src/linux-process-identity.ts',
  './src/tunnel-bypass-watcher-sidecar.ts',
  './src/tunnel-bypass-watch-service.ts',
  './src/tunnel-bypass-watcher-control.ts',
  './src/tunnel-teardown.ts',
  './src/tunnel-traffic-block.ts',
  './src/tunnel-teardown-manual.ts',
  './src/interface-name.ts',
  './src/cli-args.ts',
  './src/shell-quote.ts',
  './src/tunnel-clearnet.ts',
  './src/runner.ts',
  './src/bypass-watch-sentinel-core.ts',
  './src/bypass-watch-sentinel.ts',
  './src/bypass-watch-sentinel-launch.ts',
  './src/bypass-watch-sentinel-marker.ts',
  './src/bypass-watch-sentinel-service.ts',
  './src/system-stop.ts',
];

/**
 Self-contained production CLI and public tunnel-orchestration bundles,
 each entry built as its own file.
 */
const entryBundles: NodeFlavorConfigs = perEntryNodeConfig({
  entries: ENTRIES.filter(function isNotWatcherEntry(entry: string,): boolean {
    return entry !== WATCHER_ENTRY;
  },),
},);

/**
 Self-contained production CLI and public tunnel-orchestration bundles.

 The route watcher is built through {@link nodeConfig} so its config can also emit the clearnet host data
 beside the chunks:
 the watcher reads that copy again at every refresh,
 so a list a newer build installs there reaches a running tunnel without a restart.
 */
const config: NodeFlavorConfigs = [
  ...entryBundles,
  nodeConfig({
    input: [WATCHER_ENTRY,],
    // The owning mise task pre-cleans the output directory once,
    // as every per-entry config expects.
    outputOverrides: { cleanDir: false, },
    extraPlugins: [copiedAssetsPlugin({
      files: [{
        source: CLEARNET_HOSTS_DATA_SOURCE,
        fileName: CLEARNET_HOSTS_DATA_FILENAME,
      },],
    },),],
  },),
];

export default config;
