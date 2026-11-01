/**
 Scenarios and fixture content for the catalog-tighten e2e matrix.
 
 Each scenario installs a tiny fixture workspace under one pnpm layout, applies
 an optional post-install mutation (removing a file or directory, seeding a
 stale orphan), then asserts catalog-tighten behaves correctly: tightening the
 catalog floor to the active version, reporting a MISS or an UNDCL, or failing
 cleanly.
 The fixture pins {@link FIXTURE_PACKAGE} to {@link FIXTURE_ACTIVE} via an
 override so the expected tightened range is deterministic across layouts, and
 declares two consumer packages so "only some node_modules missing" is testable.
 */

//region Fixture constants

/**
 Catalog package the fixture pins and tightens; tiny, with several published versions.
 */
export const FIXTURE_PACKAGE = 'picomatch';

/**
 Catalog floor written into the fixture, below the pinned active version.
 */
export const FIXTURE_FLOOR = '4.0.0';

/**
 Active installed version the fixture pins via an override, so every layout resolves the same.
 */
export const FIXTURE_ACTIVE = '4.0.2';

/**
 Higher version seeded as a stale virtual-store orphan; the resolver must ignore it.
 */
export const FIXTURE_ORPHAN = '4.0.4';

/**
 Environment variable through which the orchestrator passes the monorepo's
 `pnpm@<version>` spec into each container, so a fixture always installs with
 the pnpm the repo actually runs (resolved at run time) rather than a hardcoded
 version that drifts from mise's floating `pnpm = "latest"`.
 */
export const FIXTURE_PNPM_ENV = 'FIXTURE_PNPM';

/**
 Relocatable content-addressable store path used by the remove-store scenario, so the mutation
 can delete a known directory and prove the store is irrelevant once files are in the virtual store.
 */
export const FIXTURE_STORE_DIR = '/tmp/ct-removable-store';

/**
 Consumer package directories under the `package/*\/*` glob, both depending on the catalog entry.
 */
export const CONSUMER_DIRS: readonly [
  string,
  string,
] = [
  'package/grp/consumer-a',
  'package/grp/consumer-b',
];

/**
 Expected catalog line after a successful tighten, asserted by the in-container run.
 */
export const EXPECTED_TIGHTENED: string = `${FIXTURE_PACKAGE}: >=${FIXTURE_FLOOR} -> >=${FIXTURE_ACTIVE}`;

//endregion Fixture constants

//region Scenarios

/**
 pnpm node-linker mode under test.
 */
type NodeLinker = 'isolated' | 'hoisted' | 'pnp';

/**
 Post-install mutation applied before running the tool.
 - `none`: install left intact.
 - `stale-orphan`: seed a higher orphan in the virtual store, with no symlink.
 - `remove-lockfile`: delete `pnpm-lock.yaml`.
 - `remove-workspace-yaml`: delete `pnpm-workspace.yaml`.
 - `remove-all-modules`: delete every `node_modules`.
 - `remove-some-modules`: delete one consumer's `node_modules`.
 - `orphan-store-copy`: undeclare the package in both consumers and delete their `node_modules`, keeping the
   root virtual store, so it is present in `.pnpm` but declared by no importer and reachable through no symlink.
 - `remove-virtual-store`: delete `node_modules/.pnpm`, leaving dangling symlinks.
 - `remove-store`: delete the relocated content-addressable store.
 - `remove-pnp-cjs`: delete `.pnp.cjs` under the pnp linker; pnpm's pnp is a hybrid that also keeps
   per-importer `node_modules` symlinks, so resolution survives via those (the tool still tightens).
 - `remove-pnpm`: delete the provisioned pnpm shim, simulating pnpm absent from PATH.
 */
type Mutation =
  | 'none'
    | 'stale-orphan'
    | 'remove-lockfile'
    | 'remove-workspace-yaml'
    | 'remove-all-modules'
    | 'remove-some-modules'
    | 'orphan-store-copy'
    | 'remove-virtual-store'
    | 'remove-store'
    | 'remove-pnp-cjs'
    | 'remove-pnpm';

/**
 Expected tool behaviour for a scenario.
 - `tighten`: exits zero and tightens the floor to the active version.
 - `miss`: exits zero, reports a MISS (absent from the workspace), and tightens nothing.
 - `undeclared`: exits zero, reports an UNDCL (present in the store, no importer symlink), and tightens nothing.
 - `error`: exits non-zero with a clear message.
 */
type Expectation = 'tighten' | 'miss' | 'undeclared' | 'error';

/**
 One matrix scenario: a pnpm layout, a mutation, and the expected tool behaviour.
 */
export type Scenario = {
  /**
   Human-readable label used in the test name and container diagnostics.
   */
  readonly label: string;
  /**
   pnpm `nodeLinker` mode for the install.
   */
  readonly nodeLinker: NodeLinker;
  /**
   pnpm `hoist` setting; ignored under the pnp linker.
   */
  readonly hoist: boolean;
  /**
   Extra `pnpm-workspace.yaml` lines for this scenario (store-relocating settings).
   */
  readonly extraSettings?: readonly string[];
  /**
   Post-install mutation applied before running the tool.
   */
  readonly mutation: Mutation;
  /**
   Expected tool behaviour.
   */
  readonly expect: Expectation;
};

/**
 Every {@link Scenario} the matrix runs: the node-linker and layout-settings
 coverage, plus the missing-X robustness cases.
 */
export const SCENARIOS: readonly Scenario[] = [
  {
    label: 'isolated, hoist on',
    nodeLinker: 'isolated',
    hoist: true,
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'isolated, hoist off',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'hoisted, hoist on',
    nodeLinker: 'hoisted',
    hoist: true,
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'hoisted, hoist off',
    nodeLinker: 'hoisted',
    hoist: false,
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'pnp',
    nodeLinker: 'pnp',
    hoist: false,
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'modulesDir renamed',
    nodeLinker: 'isolated',
    hoist: false,
    extraSettings: ['modulesDir: node_modules_alt',],
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'virtualStoreDir relocated',
    nodeLinker: 'isolated',
    hoist: false,
    extraSettings: ['virtualStoreDir: .vstore',],
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'global virtual store',
    nodeLinker: 'isolated',
    hoist: false,
    extraSettings: ['enableGlobalVirtualStore: true',],
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'storeDir relocated',
    nodeLinker: 'isolated',
    hoist: false,
    extraSettings: [`storeDir: ${FIXTURE_STORE_DIR}`,],
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'stale orphan',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'stale-orphan',
    expect: 'tighten',
  },
  {
    label: 'missing lockfile',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'remove-lockfile',
    expect: 'tighten',
  },
  {
    label: 'missing store',
    nodeLinker: 'isolated',
    hoist: false,
    extraSettings: [`storeDir: ${FIXTURE_STORE_DIR}`,],
    mutation: 'remove-store',
    expect: 'tighten',
  },
  {
    label: 'missing some node_modules',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'remove-some-modules',
    expect: 'tighten',
  },
  {
    label: 'missing pnpm',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'remove-pnpm',
    expect: 'tighten',
  },
  {
    label: 'missing virtual store',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'remove-virtual-store',
    expect: 'miss',
  },
  {
    label: 'store-only orphan, undeclared',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'orphan-store-copy',
    expect: 'undeclared',
  },
  {
    label: 'missing all node_modules',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'remove-all-modules',
    expect: 'error',
  },
  {
    label: 'missing workspace yaml',
    nodeLinker: 'isolated',
    hoist: false,
    mutation: 'remove-workspace-yaml',
    expect: 'error',
  },
  {
    label: 'missing pnp cjs',
    nodeLinker: 'pnp',
    hoist: false,
    mutation: 'remove-pnp-cjs',
    expect: 'tighten',
  },
  {
    label: 'pnp, symlink off',
    nodeLinker: 'pnp',
    hoist: false,
    extraSettings: ['symlink: false',],
    mutation: 'none',
    expect: 'tighten',
  },
  {
    label: 'pnp, symlink off, missing pnp cjs',
    nodeLinker: 'pnp',
    hoist: false,
    extraSettings: ['symlink: false',],
    mutation: 'remove-pnp-cjs',
    expect: 'miss',
  },
];

//endregion Scenarios

//region Fixture files

/**
 Builds the `pnpm-workspace.yaml` for one scenario: the `package/*\/*` glob
 matching catalog-tighten's workspace discovery, the catalog floor for
 {@link FIXTURE_PACKAGE}, the scenario's linker, hoist, and extra settings,
 and an override pinning the active version. The `hoist:` line is omitted
 under pnp, where it does not apply.
 
 Under the pnp linker, pnpm keeps per-importer `node_modules` symlinks unless
 `symlink: false` is set (pnpm's recommended pnp config), which removes them and
 makes resolution depend on `.pnp.cjs` alone; both shapes are covered.
 
 @param scenario - scenario whose pnpm settings to encode
 
 @returns `pnpm-workspace.yaml` text for the fixture
 
 @example
 ```ts
 buildWorkspaceYaml(SCENARIOS[0])
 ```
 */
export function buildWorkspaceYaml(scenario: Scenario,): string {
  /**
   `hoist:` line, present only for node-modules linkers where the setting is meaningful.
   */
  const hoistLine = scenario.nodeLinker === 'pnp'
    ? ''
    : `hoist: ${String(scenario.hoist,)}\n`;
  /**
   Store-relocating settings for this scenario, each on its own line.
   */
  const extraLines = (scenario.extraSettings
    ?? [])
    .map(function toLine(setting,): string {
      return `${setting}\n`;
    },)
    .join('',);
  return [
    'packages:',
    '  - \'package/*/*\'',
    'catalog:',
    `  '${FIXTURE_PACKAGE}': '>=${FIXTURE_FLOOR}'`,
    `nodeLinker: ${scenario.nodeLinker}`,
    `${hoistLine}${extraLines}overrides:`,
    `  ${FIXTURE_PACKAGE}: ${FIXTURE_ACTIVE}`,
    '',
  ].join('\n',);
}

/**
 Builds the fixture root `package.json`, pinning `packageManager` to the
 monorepo pnpm spec the orchestrator resolved at run time, so corepack
 provisions exactly the pnpm the repo runs.
 
 @param pnpmSpec - `pnpm@<version>` spec derived from the monorepo's pnpm
 
 @returns root `package.json` text
 
 @example
 ```ts
 buildRootPackageJson("pnpm\@11.11.0")
 ```
 */
export function buildRootPackageJson(pnpmSpec: string,): string {
  return `${
    JSON.stringify(
      {
        name: 'catalog-tighten-fixture-root',
        private: true,
        version: '0.0.0',
        packageManager: pnpmSpec,
      },
      undefined,
      2,
    )
  }\n`;
}

/**
 Builds a consumer `package.json` depending on the {@link FIXTURE_PACKAGE} catalog entry.
 
 @param name - consumer package name
 
 @returns consumer `package.json` text
 
 @example
 ```ts
 consumerPackageJson('consumer-a')
 ```
 */
export function consumerPackageJson(name: string,): string {
  return `${JSON.stringify(
    {
      name,
      private: true,
      version: '0.0.0',
      dependencies: {
        [FIXTURE_PACKAGE]: 'catalog:',
      },
    },
    undefined,
    2,
  )}\n`;
}

//endregion Fixture files
