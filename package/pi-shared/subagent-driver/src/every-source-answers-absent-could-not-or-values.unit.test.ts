/**
 The class two walk: every source and every kept place in the table of roads is put through absent, unreadable in each of the ways
 the operating system can refuse, and readable, and the run's own answer is asserted each time. A place that could not be read never
 answers "nothing there", and never lets a run go on.

 Every value here is made up, in a disposable home, and every way of refusing is made with permissions this test sets.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { spawnSync, } from 'node:child_process';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  SecretEnvironmentUnreadableError,
  storeOfTheCheckout,
  valuesInPiCredentialFiles,
  valuesInTheAgeIdentity,
  valuesInShellStartFiles,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** The stand-in value a readable source declares. */
const SECRET = 'stand-in-source-value-0123456789';

/** The variable a shell start-up file exports it as. */
const SHELL_NAME = 'STAND_IN_SOURCE_TOKEN';

/**
 How a place can answer.
 */
type State = 'readable' | 'absent' | 'a-folder' | 'denied' | 'behind-a-denied-folder' | 'a-pipe';

/** Every state this walk puts each place through. */
const STATES: readonly State[] = ['readable', 'absent', 'a-folder', 'denied', 'behind-a-denied-folder', 'a-pipe',];

/**
 A home folder this test makes and removes.
 
 @returns its path and its disposer
 */
async function makeHome(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-source-states-',),);
  await chmod(path, 0o700,);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await reopen({ home: path, },);
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Puts one file into one of the states.
 
 @param path - where the file goes
 
 @param state - how it answers
 
 @param content - what a readable file holds
 */
async function put({ path, state, content, }: {
  readonly path: string;
  readonly state: State;
  readonly content: string;
},): Promise<void> {
  if (state === 'absent') {
    return;
  }
  await mkdir(join(path, '..',), { recursive: true, },);
  if (state === 'readable') {
    await writeFile(path, content,);
    return;
  }
  if (state === 'a-folder') {
    await mkdir(path, { recursive: true, },);
    return;
  }
  if (state === 'a-pipe') {
    spawnSync('mkfifo', [path,], { stdio: 'ignore', },);
    return;
  }
  if (state === 'denied') {
    await writeFile(path, content,);
    await chmod(path, 0o000,);
    return;
  }
  // Behind a folder that denies the way: the file is there and nothing may reach it.
  await writeFile(path, content,);
  await chmod(join(path, '..',), 0o000,);
}

/**
 Gives every folder its permissions back, so the home can be removed.
 
 @param home - the home folder
 */
async function reopen({ home, }: { readonly home: string },): Promise<void> {
  /* oxlint-disable eslint/no-await-in-loop -- one folder at a time, each a question of the file system rather than a stream. */
  for (const folder of ['', '.pi', '.pi/agent', 'checkout',]) {
    try {
      await chmod(join(home, folder,), 0o700,);
    }
    catch (error) {
      if (!Error.isError(error,)) {
        throw error;
      }
      // A folder that is not there needs no permission.
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}

//endregion Fixtures

//region The places

/**
 One place of the table of roads that holds values, and how a test puts it in a state.
 */
type Place = {
  readonly name: string;
  readonly make: (params: { readonly home: string; readonly state: State }) => Promise<void>;
  readonly read: (params: { readonly home: string }) => Promise<unknown>;
};

/**
 Every place beside the checkout's store that this walk knows: each is read by one function, and each answers absent, could not, or
 values.
 */
const PLACES: readonly Place[] = [
  {
    name: 'auth.json',
    make: async function make({ home, state, }) {
      await put({
        path: join(home, '.pi', 'agent', 'auth.json',),
        state,
        content: JSON.stringify({ example: { type: 'api_key', key: SECRET, }, },),
      },);
    },
    read: async function read({ home, }) {
      return await valuesInPiCredentialFiles({ home, },);
    },
  },
  {
    name: 'models.json',
    make: async function make({ home, state, }) {
      await put({
        path: join(home, '.pi', 'agent', 'models.json',),
        state,
        content: JSON.stringify({ providers: { example: { apiKey: SECRET, }, }, },),
      },);
    },
    read: async function read({ home, }) {
      return await valuesInPiCredentialFiles({ home, },);
    },
  },
  {
    name: '.bashrc',
    make: async function make({ home, state, }) {
      await put({
        path: join(home, '.bashrc',),
        state,
        content: `export ${SHELL_NAME}=${SECRET}\n`,
      },);
    },
    read: async function read({ home, }) {
      return await valuesInShellStartFiles({ home, environment: {}, },);
    },
  },
  {
    name: '.bash_profile',
    make: async function make({ home, state, }) {
      await put({
        path: join(home, '.bash_profile',),
        state,
        content: `export ${SHELL_NAME}=${SECRET}\n`,
      },);
    },
    read: async function read({ home, }) {
      return await valuesInShellStartFiles({ home, environment: {}, },);
    },
  },
];

/**
 The two places whose answer is a shape of their own rather than a list of values: the store a run declares, and the identity that
 opens it.
 */
const SHAPES: readonly {
  readonly name: string;
  readonly make: (params: { readonly home: string; readonly state: State }) => Promise<void>;
  readonly read: (params: { readonly home: string }) => Promise<{ readonly couldNot: boolean }>;

  /**
   Whether a file that is not there is a declaration this code cannot use, which is what an identity named by a variable is, and what
   a store named by a variable is not: a checkout with no store declares no secret environment.
   */
  readonly absentIsAStop: boolean;

  /**
   The states this place is walked through.
   */
  readonly states: readonly State[];
}[] = [
  {
    name: 'the checkout\'s store, named by the environment',
    make: async function make({ home, state, }) {
      await put({
        path: join(home, 'checkout', '.env.local.json',),
        state,
        content: JSON.stringify({ STAND_IN_KEY: 'ENC[AES256_GCM,data:x,iv:y,tag:z,type:str]', sops: { version: '3.9.0', }, },),
      },);
    },
    read: async function read({ home, }) {
      /**
       What the store's discovery answered.
       */
      const found = await storeOfTheCheckout({
        checkoutFolder: join(home, 'checkout',),
        environment: { ENV_LOCAL_JSON_PATH: join(home, 'checkout', '.env.local.json',), },
        home,
      },);
      return { couldNot: 'cause' in found, };
    },
    absentIsAStop: false,
    states: ['absent', 'a-folder', 'denied', 'behind-a-denied-folder', 'a-pipe',],
  },
  {
    name: 'the age identity the machine declares',
    make: async function make({ home, state, }) {
      await put({
        path: join(home, 'identity.txt',),
        state,
        // An identity file of the right shape that no store is encrypted to, built from parts so no file holds a key's spelling.
        content: `${['AGE', 'SECRET', 'KEY',].join('-',)}-1${'Q'.repeat(58,)}`.concat(String.fromCodePoint(10,),),
      },);
    },
    read: async function read({ home, }) {
      /**
       What the identity's own reader answered.
       */
      const identity = await valuesInTheAgeIdentity({
        environment: { SOPS_AGE_KEY_FILE: join(home, 'identity.txt',), },
        home,
      },);
      return { couldNot: 'unreadable' in identity, };
    },
    absentIsAStop: true,
    states: ['absent', 'a-folder', 'denied', 'behind-a-denied-folder', 'a-pipe',],
  },
  {
    // A machine that declares no identity has none to read, which is the one "nothing there" an identity has.
    name: 'the age identity no machine declares',
    make: async function make() {
      // No variable is set and no default file is written, which is a machine with no identity at all.
    },
    read: async function read({ home, }) {
      /**
       What the identity's own reader answered.
       */
      const identity = await valuesInTheAgeIdentity({
        environment: {},
        home,
      },);
      return { couldNot: 'unreadable' in identity, };
    },
    absentIsAStop: false,
    states: ['absent',],
  },
];

//endregion The places

/**
 Reads one place in one state, in a home of its own.
 
 @param place - the place

 @param state - how it answers

 @returns what the read gave, or the error it stopped the run with
 */
async function readInState({ place, state, }: {
  readonly place: Place;
  readonly state: State;
},): Promise<
  | { readonly values: unknown }
    | { readonly error: unknown }
  > {
  await using home = await makeHome();
  await place.make({ home: home.path, state, },);
  try {
    return { values: await place.read({ home: home.path, },), };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return { error, };
  }
}

/**
 Reads one place of a shape in one state, in a home of its own.
 
 @param shape - the place

 @param state - how it answers

 @returns what its own reader answered
 */
async function readShapeInState({ shape, state, }: {
  readonly shape: (typeof SHAPES)[number];
  readonly state: State;
},): Promise<{ readonly couldNot: boolean }> {
  await using home = await makeHome();
  await shape.make({ home: home.path, state, },);
  return await shape.read({ home: home.path, },);
}

await describe({
  name: 'every source answers absent, could-not, or values',
  children: [
    ...PLACES.map(function walkOne(place,) {
      return it({
        name: `${place.name} answers values when readable, nothing when absent, and a stop when it could not be read`,
        fn: async () => {
          /* oxlint-disable eslint/no-await-in-loop -- one state at a time, each in its own home, so a failure names the state. */
          for (const state of STATES) {
            const answered = await readInState({ place, state, },);
            if (state === 'readable') {
              if ('error' in answered) {
                throw new Error(`${place.name} readable was refused`,);
              }
              expect(JSON.stringify(answered.values,),).toContain(SECRET,);
              continue;
            }
            if (state === 'absent') {
              if ('error' in answered) {
                throw new Error(`${place.name} absent was refused instead of answering nothing`,);
              }
              expect(JSON.stringify(answered.values,),).not.toContain(SECRET,);
              continue;
            }
            // A folder, a denied file, a denied folder above it and a pipe are all "could not", never "not there".
            if (!('error' in answered)) {
              throw new Error(`${place.name} ${state} was read as absent instead of stopping the run`,);
            }
            expect(answered.error,).toBeInstanceOf(SecretEnvironmentUnreadableError,);
          }
          /* oxlint-enable eslint/no-await-in-loop */
        },
      },);
    },),
    ...SHAPES.map(function walkOneShape(shape,) {
      return it({
        name: `${shape.name} answers nothing when absent and a stop when it could not be read`,
        fn: async () => {
          /* oxlint-disable eslint/no-await-in-loop -- one state at a time, each in its own home, so a failure names the state. */
          for (const state of shape.states) {
            const answered = await readShapeInState({ shape, state, },);
            if (state === 'absent') {
              expect(answered.couldNot,).toBe(shape.absentIsAStop,);
              continue;
            }
            expect(answered.couldNot,).toBe(true,);
          }
          /* oxlint-enable eslint/no-await-in-loop */
        },
      },);
    },),
  ],
},);
