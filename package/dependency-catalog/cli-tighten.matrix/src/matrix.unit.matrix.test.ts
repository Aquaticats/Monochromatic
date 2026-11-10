/**
 Containerised matrix verifying catalog-tighten across pnpm install layouts.
 
 For every {@link LayoutCombo}, runs one podman container (monorepo and its
 store mounted read-only, fixture installed in a tmpfs) that installs a fixture
 under that layout and asserts catalog-tighten tightens the catalog floor to the
 active version. Results report through `describe`/`it` from
 `@monochromatic-dev/test-harness-core`.
 
 Needs podman and network (the fixture install pulls from the registry), so it
 lives behind the `test:matrix` task rather than the standard `test:unit` run.
 
 @module
 */

import {
  realpathSync,
} from 'node:fs';
import {
  homedir,
} from 'node:os';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn from 'nano-spawn';

import {
  SCENARIOS,
  type Scenario,
} from './combos.ts';
import {
  containerStoreDir,
} from './container-paths.ts';
import {
  resolvePnpmSpec,
  resolveStoreDir,
} from './host-pnpm.ts';
import {
  buildPodmanArgs,
} from './podman-args.ts';

//region Configuration

/**
 Maximum containers to run at once; each install is memory- and network-heavy.
 */
const MATRIX_CONCURRENCY = 2;

//endregion Configuration

//region Run

/**
 Runs one combination in a container and fails the test on a non-zero exit or
 a missing PASS marker.
 
 @param scenario - scenario to run
 
 @param pnpmSpec - monorepo `pnpm@<version>` spec passed into the container
 
 @param storeMount - pnpm store on the host, and the path inside the container its links name
 
 @throws Error when the container exits non-zero or does not report PASS
 
 @example
 ```ts
 await runScenario({ scenario: SCENARIOS[0], pnpmSpec: 'pnpm\@12.18.0', storeMount: { hostStoreDir: '/srv/store/v11', containerStoreDir: '/srv/store/v11' } });
 ```
 */
async function runScenario(
  {
    scenario,
    pnpmSpec,
    storeMount,
  }: {
    readonly scenario: Scenario;
    readonly pnpmSpec: string;
    readonly storeMount: {
      readonly hostStoreDir: string;
      readonly containerStoreDir: string;
    };
  },
): Promise<void> {
  /**
   Container result; nano-spawn rejects on a non-zero exit, surfacing a failed assertion.
   */
  const result = await spawn(
    'podman',
    buildPodmanArgs({
      scenario,
      pnpmSpec,
      storeMount,
    },),
  );
  if (!result.stdout
    .includes('PASS',)) {
    throw new Error(
      `[${scenario.label}] container did not report PASS:\n${result.stdout}\n${result.stderr}`,
    );
  }
}

//endregion Run

//region Main

/**
 Home directory as this process spells it, from `$HOME`.
 */
const logicalHome = homedir();

/**
 pnpm store on the host, from `pnpm store path`.
 */
const hostStoreDir = await resolveStoreDir();

/**
 Monorepo pnpm spec resolved once, passed into every container so fixtures
 install with the pnpm the repo actually runs.
 */
const pnpmSpec = await resolvePnpmSpec();

/**
 Store mount every container gets: the host directory pnpm reports, at the path
 the workspace's `node_modules` links spell.
 */
const storeMount = {
  hostStoreDir,
  containerStoreDir: containerStoreDir({
    hostStoreDir,
    homeDir: logicalHome,
    realHomeDir: realpathSync(logicalHome,),
  },),
};

await describe({
  name: 'catalog-tighten install-layout matrix',
  concurrency: MATRIX_CONCURRENCY,
  children: SCENARIOS.map(function scenarioCase(scenario,) {
    return it({
      name: scenario.label,
      fn: async () => {
        await runScenario({
          scenario,
          pnpmSpec,
          storeMount,
        },);
      },
    },);
  },),
},);

//endregion Main
