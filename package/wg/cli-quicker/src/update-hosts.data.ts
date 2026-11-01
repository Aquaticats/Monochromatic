/**
 Hosts the toolchain's resolvers and installers fetch from,
 as named by the plan and lock (`toolchain.lock.json`),
 the toolchain resolvers,
 and `mpm-shadow`'s network client.

 `update-hosts.unit.test.ts` scans those sources and fails when a host appears there that this list lacks.
 */
export const PLAN_UPDATE_HOSTS: readonly string[] = [
  'api.adoptium.net',
  'api.anaconda.org',
  'api.github.com',
  'awscli.amazonaws.com',
  'conda.anaconda.org',
  'crates.io',
  'dl.google.com',
  'files.pythonhosted.org',
  'github.com',
  'go.dev',
  'nodejs.org',
  'proxy.golang.org',
  'pypi.org',
  'registry.npmjs.org',
  'static.rust-lang.org',
  'unofficial-builds.nodejs.org',
  'ziglang.org',
];

/**
 Hosts the plan never names but its downloads reach.

 - GitHub release downloads redirect from `github.com` to `objects.githubusercontent.com`,
 and currently to `release-assets.githubusercontent.com`.
 - `cargo` and `cargo-binstall` read the sparse index from `index.crates.io` and download crates from
 `static.crates.io`.
 */
export const REDIRECT_UPDATE_HOSTS: readonly string[] = [
  'index.crates.io',
  'objects.githubusercontent.com',
  'release-assets.githubusercontent.com',
  'static.crates.io',
];

/**
 Every host whose traffic the wg tools keep outside the tunnel for every program.
 */
export const UPDATE_HOSTS: readonly string[] = [
  ...PLAN_UPDATE_HOSTS,
  ...REDIRECT_UPDATE_HOSTS,
];

/**
 Hosts that appear in toolchain sources but are not fetched at update time:
 documentation links and test or example placeholders.
 */
export const NON_FETCH_HOSTS: readonly string[] = [
  'pnpm.io',
  'example',
  'example.invalid',
  'example.test',
  'x',
];
