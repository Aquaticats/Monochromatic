/**
 Unit tests for the `podman run` argv the matrix builds per scenario.
 
 Pure host tests (no containers): pin the mounts the container needs to load
 the tool at all. The monorepo is mounted read-only, and so is the pnpm store
 its `node_modules` entries link into, at the path those links spell, because a
 relative link out of the repository resolves to nothing unless that path
 exists in the container.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  FIXTURE_PNPM_ENV,
  SCENARIOS,
  type Scenario,
} from './combos.ts';
import {
  HOST_REPO_ROOT,
  IN_CONTAINER_ENTRY,
  REPO_DIR,
} from './container-paths.ts';
import {
  buildPodmanArgs,
  NODE_IMAGE,
} from './podman-args.ts';

//region Fixtures

/**
 First scenario the matrix declares, before the emptiness check.
 */
const [firstScenario,] = SCENARIOS;
if (firstScenario === undefined)
  throw new Error('the matrix declares no scenario to build an argv for',);

/**
 Scenario every argv fixture is built for.
 */
const SCENARIO: Scenario = firstScenario;

/**
 Spec the orchestrator resolved from the pnpm on PATH.
 */
const PNPM_SPEC = 'pnpm@12.18.0';

/**
 Store directory on the host, as pnpm reports it.
 */
const HOST_STORE_DIR = '/var/canonical/home/user/.local/share/pnpm/store/v11';

/**
 Container path the workspace's `node_modules` links name for that store.
 */
const CONTAINER_STORE_DIR = '/home/user/.local/share/pnpm/store/v11';

//endregion Fixtures

//region Argv reading

/**
 Reads the value after every occurrence of one repeated flag.

 @param args - argv vector to read

 @param flag - flag whose values to collect, `--volume` for mounts

 @returns the values in argv order

 @throws Error when the argv ends on the flag with no value after it

 @example
 ```ts
 flagValues({ args: ['run', '--volume', '/a:/b:ro'], flag: '--volume' }) // ['/a:/b:ro']
 ```
 */
function flagValues(
  {
    args,
    flag,
  }: {
    readonly args: readonly string[];
    readonly flag: string;
  },
): readonly string[] {
  /**
   Values collected so far, in argv order.
   */
  const values: string[] = [];
  for (const [index, arg] of args.entries()) {
    if (arg !== flag)
      continue;
    /**
     Value that follows the flag.
     */
    const value = args[index + 1];
    if (value === undefined)
      throw new Error(`argv ends on ${flag} with no value after it`,);
    values.push(value,);
  }
  return values;
}

/**
 argv for {@link SCENARIO} under the fixture spec and store mount.

 @returns argument vector excluding the `podman` executable

 @example
 ```ts
 podmanArgs().at(0) // 'run'
 ```
 */
function podmanArgs(): readonly string[] {
  return buildPodmanArgs({
    scenario: SCENARIO,
    pnpmSpec: PNPM_SPEC,
    storeMount: {
      hostStoreDir: HOST_STORE_DIR,
      containerStoreDir: CONTAINER_STORE_DIR,
    },
  },);
}

//endregion Argv reading

await describe({
  name: 'podman argv',
  children: [
    //region Mounts

    it({
      name: 'mounts the monorepo read-only at the container repo dir',
      fn: async () => {
        expect(
          flagValues({
            args: podmanArgs(),
            flag: '--volume',
          },),
        ).toContain(`${HOST_REPO_ROOT}:${REPO_DIR}:ro`,);
      },
    },),

    it({
      name: 'mounts the resolved store read-only at the path the workspace links name',
      fn: async () => {
        expect(
          flagValues({
            args: podmanArgs(),
            flag: '--volume',
          },),
        ).toContain(`${HOST_STORE_DIR}:${CONTAINER_STORE_DIR}:ro`,);
      },
    },),

    it({
      name: 'gives every mount the read-only flag',
      fn: async () => {
        /**
         Mounts the argv passes, each `source:destination:mode`.
         */
        const mounts = flagValues({
          args: podmanArgs(),
          flag: '--volume',
        },);
        expect(mounts.length,).toBeGreaterThan(0,);
        for (const mount of mounts)
          expect(mount.endsWith(':ro',),).toBe(true,);
      },
    },),

    //endregion Mounts

    //region Fixture inputs

    it({
      name: 'passes the resolved pnpm spec through the fixture env var',
      fn: async () => {
        expect(
          flagValues({
            args: podmanArgs(),
            flag: '--env',
          },),
        ).toContain(`${FIXTURE_PNPM_ENV}=${PNPM_SPEC}`,);
      },
    },),

    it({
      name: 'runs this matrix entrypoint on the node image with the serialised scenario',
      fn: async () => {
        /**
         Trailing argv entries: image, interpreter, entrypoint, scenario JSON.
         */
        const tail = podmanArgs()
          .slice(-4,);
        expect(tail,).toStrictEqual([
          NODE_IMAGE,
          'node',
          IN_CONTAINER_ENTRY,
          JSON.stringify(SCENARIO,),
        ],);
      },
    },),

    //endregion Fixture inputs
  ],
},);
