import clearnetHosts from './clearnet-hosts.data.json' with { type: 'json', };

/**
 Hosts the toolchain's resolvers and installers fetch from,
 as named by the plan and lock (`toolchain.lock.json`),
 the toolchain resolvers,
 and `mpm-shadow`'s network client.

 They are recorded in `clearnet-hosts.data.json`,
 which the build also copies beside the route watcher so a running watcher reads a changed list again.
 `update-hosts.unit.test.ts` scans those sources and fails when a host appears there that this list lacks.
 */
export const PLAN_UPDATE_HOSTS: readonly string[] = clearnetHosts
  .updateHosts
  .plan;

/**
 Hosts the plan never names but its downloads reach.

 - GitHub release downloads redirect from `github.com` to `objects.githubusercontent.com`,
 and currently to `release-assets.githubusercontent.com`.
 - `cargo` and `cargo-binstall` read the sparse index from `index.crates.io` and download crates from
 `static.crates.io`.
 */
export const REDIRECT_UPDATE_HOSTS: readonly string[] = clearnetHosts
  .updateHosts
  .redirect;

/**
 Every host whose traffic the wg tools keep outside the tunnel for every program.
 */
export const UPDATE_HOSTS: readonly string[] = [
  ...PLAN_UPDATE_HOSTS,
  ...REDIRECT_UPDATE_HOSTS,
];

/**
 Hosts that appear in toolchain sources but are not fetched at update time:
 documentation links.
 `update-hosts.unit.test.ts` fails when an entry is no longer named there.
 */
export const NON_FETCH_HOSTS: readonly string[] = clearnetHosts
  .updateHosts
  .nonFetch;
