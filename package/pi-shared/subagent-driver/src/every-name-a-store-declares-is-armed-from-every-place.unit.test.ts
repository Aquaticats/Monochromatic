/**
 The class the fourth review found: the set of names to arm is what the store declares, and each name's value is looked for in every
 place a value for that name can come from. Taking the names from what one reader happened to return dropped a declared name that the
 environment already supplied, and dropped the store's own value when the environment shadowed it.

 Every row below is one declared name placed in one combination of places, held against a real sops store and a disposable home.

 @module
 */

import { createSopsFixture, } from '@monochromatic-dev/env-local-json-core/ts/fixture.ts';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { readSecretValuesOfThisMachine, } from '../dist/final/node/index.mjs';

/** A stand-in value of the store, long enough to be masked. */
const IN_THE_STORE = 'stand-in-store-only-value-0123456789';

/** A stand-in value of the store that this code does not mask, because it is shorter than a maskable value. */
const SHORT = 'st';

/** A stand-in value of the driver's own environment, long enough to be masked. */
const IN_THE_ENVIRONMENT = 'stand-in-environment-only-0123456789';

/**
 One declared name, and what each place holds for it. A value shorter than this code masks is one the store declares and neither
 place can arm, so the name's own value comes from the other place.
 */
type Place = {
  /**
   The name the store declares.
   */
  readonly name: string;

  /**
   The value the store holds, when it holds one.
   */
  readonly store: string;

  /**
   The value the driver's environment holds, when it holds one.
   */
  readonly environment?: string;
};

/**
 The rows: each one declared name placed in one combination of places.
 */
const ROWS: readonly {
  /**
   What the row is called.
   */
  readonly called: string;

  /**
   The names and what each place holds for them.
   */
  readonly places: readonly Place[];

  /**
   Each value that must be armed.
   */
  readonly armed: readonly string[];
}[] = [
  {
    // A name the generic credential-name rule does not match, which is why the declared set is what decides.
    called: 'a declared name only the store holds',
    places: [{ name: 'DEPLOYMENT', store: IN_THE_STORE, },],
    armed: [IN_THE_STORE,],
  },
  {
    called: 'a declared name whose store value is too short to arm and whose environment value is not',
    places: [{ name: 'DEPLOYMENT', store: SHORT, environment: IN_THE_ENVIRONMENT, },],
    armed: [IN_THE_ENVIRONMENT,],
  },
  {
    called: 'a declared name both places hold the same value for',
    places: [{ name: 'DEPLOYMENT', store: IN_THE_STORE, environment: IN_THE_STORE, },],
    armed: [IN_THE_STORE,],
  },
  {
    // The environment shadows the store in the reader this code asks, so the store's own value is reachable only by asking with
    // the name absent. Both are armed: a tool can print either.
    called: 'a declared name both places hold different values for',
    places: [{ name: 'DEPLOYMENT', store: IN_THE_STORE, environment: IN_THE_ENVIRONMENT, },],
    armed: [IN_THE_STORE, IN_THE_ENVIRONMENT,],
  },
  {
    // Two names, each in a different place, so a set taken from one reader's answer cannot cover both.
    called: 'two declared names, one in each place',
    places: [
      { name: 'DEPLOYMENT', store: IN_THE_STORE, },
      { name: 'RELEASE', store: SHORT, environment: IN_THE_ENVIRONMENT, },
    ],
    armed: [IN_THE_STORE, IN_THE_ENVIRONMENT,],
  },
];

/**
 Reads one row: creates its store and home, reads this machine's secret environment for that checkout, and answers the values armed
 by the name they were armed under.

 @param row - the names and where each place holds them

 @returns every armed value, by its name
 */
async function armedFor({ row, }: { readonly row: (typeof ROWS)[number] },): Promise<ReadonlyMap<string, string>> {
  await using store = await createSopsFixture({
    values: Object.fromEntries(row.places.map(function held(place,) {
      return [place.name, place.store,];
    },),),
  },);
  await using folder = await makeFolder();
  /**
   The environment the driver was started with, which holds what each place of this row says it holds.
   */
  const environment: Record<string, string> = {
    PATH: process.env.PATH ?? '',
    ENV_LOCAL_JSON_PATH: store.storePath,
    SOPS_AGE_KEY_FILE: store.identityPath,
  };
  for (const place of row.places) {
    if (place.environment !== undefined) {
      environment[place.name] = place.environment;
    }
  }
  /**
   Everything this machine's secret environment holds for that checkout.
   */
  const read = await readSecretValuesOfThisMachine({
    environment,
    home: folder.path,
    checkoutFolder: folder.path,
  },);
  // No source's own text holds a value.
  expect(JSON.stringify(read.sources,),).not.toContain(IN_THE_STORE,);
  return new Map(read.values.map(function pair(value,) {
    return [value.value, value.name,] as const;
  },),);
}

await describe({
  name: 'every name a store declares is armed from every place it holds a value',
  children: [
    it({
      name: 'arms each declared name from the store, from the environment, or from both, in every combination',
      fn: async () => {
        // One row at a time: each creates a sops store and a home of its own, and a store is read by spawning sops, which must not
        // race another row's.
        /* oxlint-disable eslint/no-await-in-loop -- one row at a time, each with its own store and home. */
        for (const row of ROWS) {
          /**
           Every armed value of this row, by its name.
           */
          const armedByName = await armedFor({ row, },);
          for (const value of row.armed) {
            expect([...armedByName.keys(),],).toContain(value,);
          }
          // Every armed value is named by a declared name of this row, which is the point: the names come from the declarations.
          for (const place of row.places) {
            expect(armedByName.get(place.store,) ?? place.name,).toBe(place.name,);
          }
        }
        /* oxlint-enable eslint/no-await-in-loop */
      },
    },),
  ],
},);

/**
 Makes a disposable folder.

 @returns the folder and the way to remove it
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-store-names-',),);
  await mkdir(path, { recursive: true, },);

  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}
