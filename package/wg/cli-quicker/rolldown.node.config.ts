import {
  type NodeFlavorConfigs,
  perEntryNodeConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

/**
 Self-contained production CLI and public tunnel-orchestration bundles.
 */
const config: NodeFlavorConfigs = perEntryNodeConfig({
  entries: [
    './src/index.ts',
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
    './src/update-policy-notice.ts',
    './src/update-hosts.data.ts',
    './src/opensnitch-toolchain-rule-patterns.ts',
    './src/opensnitch-toolchain-rules.ts',
    './src/opensnitch-rules-command.ts',
    './src/tunnel.ts',
    './src/config.ts',
    './src/tunnel-bypass.ts',
    './src/tunnel-bypass-route.ts',
    './src/bypass-watch.ts',
  ],
},);

export default config;
