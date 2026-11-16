/**
 A name that becomes a path: an entry's name is one path segment, and every file the package writes beside the runs for that name is
 inside the runs folder.

 A name arrives from a queue file, which anyone who can write that file decides. On 2026-11-16 the review of `main...35d724bdbe`
 showed an entry named `../outside` replacing a file beside the runs folder, so this asks the two questions the rule asks: at the
 door, where the entry is read, and again at the join, where the name becomes a path.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  readQueueOfRuns,
  setAsideAnEntryOfAQueue,
  setAsideFileOf,
  shapeOfAQueueEntry,
  startAnswerFileOf,
  watchedStartFileOf,
  writeQueueOfRuns,
} from '../dist/final/node/index.mjs';

/**
 The names that are no one path segment, each for a reason of its own.
 */
const NOT_ONE_SEGMENT: readonly { readonly name: string; readonly why: string }[] = [
  {
    name: '../outside',
    why: 'it climbs out of the folder',
  },
  {
    name: 'a/b',
    why: 'it is two segments',
  },
  {
    name: '/an-absolute-name',
    why: 'it is an absolute path',
  },
  {
    name: '',
    why: 'it is no name at all',
  },
  {
    name: '.',
    why: 'it is the folder itself',
  },
  {
    name: '..',
    why: 'it is the folder above',
  },
  {
    name: 'a\nb',
    why: 'it carries a newline',
  },
  {
    name: 'a\u0000b',
    why: 'it carries a null byte',
  },
];

/**
 The names that are one path segment, each for a reason of its own.
 */
const ONE_SEGMENT: readonly { readonly name: string; readonly why: string }[] = [
  {
    name: 'a',
    why: 'a plain name',
  },
  {
    name: 'a-name',
    why: 'a hyphen inside it',
  },
  {
    name: '-a-name',
    why: 'a leading dash is a segment the package writes beside the runs, never a word a program parses as an option',
  },
  {
    name: 'a.name.json',
    why: 'dots inside it',
  },
  {
    name: 'a name with spaces',
    why: 'spaces inside it',
  },
  {
    name: 'a-é',
    why: 'a character that is not ASCII',
  },
];

/**
 A folder a test may write in, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The folder the runs live in.
   */
  readonly runs: string;

  /**
   The queue's path.
   */
  readonly queueFile: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, removed when the test ends.

 @returns the folder

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-entry-name-path-',),);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  await mkdir(runs, { recursive: true, },);
  return {
    path,
    runs,
    queueFile: join(path, 'queue.json',),
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 How a call went: what it threw, or what it answered.

 @param call - the call to make

 @returns whether it threw, and what it threw or answered

 @example
 ```ts
 const answer = await answered(() => setAsideFileOf({ runsFolder: '/runs', name: 'a' }));
 ```
 */
async function answered(call: () => unknown,): Promise<{ readonly threw: boolean; readonly value?: unknown }> {
  try {
    return {
      threw: false,
      value: await call(),
    };
  }
  catch (error) {
    return {
      threw: true,
      value: error,
    };
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: setAsideFileOf.name,
      children: [
        ...NOT_ONE_SEGMENT.map(function refusesOne(bad,) {
          return it({
            name: `refuses the name "${bad.name.replaceAll('\n', String.raw`\n`,)}, which ${bad.why}`,
            fn: async () => {
              /**
               How the join went.
               */
              const join_ = await answered(function joinTheName() {
                return setAsideFileOf({
                  runsFolder: '/runs',
                  name: bad.name,
                },);
              },);
              expect(join_.threw,).toBe(true,);
              expect(String(join_.value,),).toContain('one path segment',);
            },
          },);
        },),
        ...ONE_SEGMENT.map(function joinsOne(good,) {
          return it({
            name: `joins the name "${good.name.replaceAll('\n', String.raw`\n`,)}", which is ${good.why}`,
            fn: async () => {
              expect(setAsideFileOf({
                runsFolder: '/runs',
                name: good.name,
              },),).toBe(`/runs/${good.name}-set-aside.json`,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: watchedStartFileOf.name,
      children: NOT_ONE_SEGMENT.map(function refusesOne(bad,) {
        return it({
          name: `refuses the name "${bad.name.replaceAll('\n', String.raw`\n`,)}, which ${bad.why}`,
          fn: async () => {
            expect((await answered(function joinTheName() {
              return watchedStartFileOf({
                runsFolder: '/runs',
                name: bad.name,
              },);
            },)).threw,).toBe(true,);
          },
        },);
      },),
    },),
    describe({
      name: startAnswerFileOf.name,
      children: NOT_ONE_SEGMENT.map(function refusesOne(bad,) {
        return it({
          name: `refuses the name "${bad.name.replaceAll('\n', String.raw`\n`,)}, which ${bad.why}`,
          fn: async () => {
            expect((await answered(function joinTheName() {
              return startAnswerFileOf({
                runsFolder: '/runs',
                name: bad.name,
              },);
            },)).threw,).toBe(true,);
          },
        },);
      },),
    },),
    describe({
      name: shapeOfAQueueEntry.name,
      children: NOT_ONE_SEGMENT.map(function refusesOne(bad,) {
        return it({
          name: `reads an entry named "${bad.name.replaceAll('\n', String.raw`\n`,)} as unreadable, naming the name`,
          fn: async () => {
              /**
               What the entry read as.
               */
            const shape = shapeOfAQueueEntry({
              record: {
                name: bad.name,
                branch: 'feat/a',
              },
            },);
            expect(shape.kind,).toBe('unreadable',);
            expect('reason' in shape ? shape.reason : '',).toContain('one path segment',);
          },
        },);
      },),
    },),
    describe({
      name: 'a record write',
      children: [
        it({
          name: 'never lands outside the runs folder when a link stands where its own being-written file would go',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A file beside the runs folder, which a set-aside must never touch.
             */
            const outside = join(folder.path, 'user-data.txt',);
            await writeFile(
              outside,
              'ORIGINAL USER DATA\n',
            );
            /**
             Where the record for the entry `a` would be written, and the temporary file beside it.
             */
            const record = setAsideFileOf({
              runsFolder: folder.runs,
              name: 'a',
            },);
            await symlink(outside, `${record}.being-written`,);
            await writeQueueOfRuns({
              queueFile: folder.queueFile,
              entries: [{ name: 'a', },],
            });
            await setAsideAnEntryOfAQueue({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              record: { name: 'a', },
              reason: 'it names no branch',
              now: () => 0,
            },);
            // The link's target is untouched, and the record itself is a file inside the runs folder.
            expect(await readFile(
              outside,
              'utf8',
            ),).toBe('ORIGINAL USER DATA\n',);
            expect(
              JSON.parse(await readFile(
                record,
                'utf8',
              ),),
            ).toMatchObject({
              name: 'a',
              reason: 'it names no branch',
            },);
            expect(await readQueueOfRuns({ queueFile: folder.queueFile, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
