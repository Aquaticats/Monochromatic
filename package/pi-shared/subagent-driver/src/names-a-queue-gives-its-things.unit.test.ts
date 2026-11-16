/**
 The names a queue gives its things: one escaped form for a file's name and for a unit's name, its inverse, and the names a person may
 write that would otherwise be a path, a hidden file, a unit systemd escapes again, or a name longer than the file system takes.

 The names here are the ones a review of this package tried by hand: a slash, two dots and a slash, two dots alone, a leading dot, a
 newline, a quote, a leading dash, a space, a percent sign, a dollar sign, a name outside ASCII, a name that differs from another only in
 case, a lone surrogate, and names at and past the length limit.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { dirname, join, normalize, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  LONGEST_ESCAPED_NAME,
  NO_NAME_IN_AN_ESCAPED_FORM,
  QueueNameError,
  aNameInAnEscapedName,
  aUnitNameOf,
  anEntryFileName,
  anEntryNameInAFileName,
  anEntryNameInAUnitName,
  anEscapedName,
  claimAnEntryOfAQueue,
  entryFileOf,
  claimFileOf,
  namesOfAnAttemptIn,
  putAQueueEntryIn,
  readQueueOfRuns,
  readTheEntryNamed,
  setAsideAnEntryOfAQueue,
  setAsideFileOf,
} from '../dist/final/node/index.mjs';

/**
 How many characters of a name the escaped form may take, which is the ceiling the names are measured against.
 */
const LONGEST = LONGEST_ESCAPED_NAME;

/**
 The names a person may write that a file name or a unit name does not take as they are.
 */
const AWKWARD_NAMES: readonly (readonly [string, string])[] = [
  ['a slash', 'a/b',],
  ['two dots and a slash', 'x/../../escape',],
  ['two dots alone', '..',],
  ['one dot alone', '.',],
  ['a leading dot', '.hidden',],
  ['a newline', 'new\nline',],
  ['a carriage return', 'carriage\rreturn',],
  ['a tab', 'a\ttab',],
  ['a quote', 'a"quote',],
  ['a single quote', "a'quote",],
  ['a leading dash', '-dash',],
  ['a dash alone', '-',],
  ['a space', 'sp ace',],
  ['a percent sign', '100%',],
  ['a dollar sign', '$HOME',],
  ['a backslash', String.raw`back\slash`,],
  ['a nul-like escape written out', String.raw`\x2f`,],
  ['text outside ASCII', 'é',],
  ['text of two bytes', 'é'.repeat(20,),],
  ['a name and its other case', 'Case',],
  ['the same name in lower case', 'case',],
  ['a combining mark and a precomposed letter', 'e\u0301',],
  ['a character that is no Unicode', '\uD800',],
  ['a name at the length limit written out', 'a'.repeat(LONGEST,),],
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
   The queue's folder.
   */
  readonly queueFolder: string;

  /**
   The runs folder.
   */
  readonly runsFolder: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a queue folder and a runs folder in a throwaway folder.

 @returns the folders

 @example
 ```ts
 await using folder = await aFolder();
 ```
 */
async function aFolder(): Promise<AThrowawayFolder> {
  /**
   The folder everything lives in.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-names-',),);
  /**
   The queue's folder.
   */
  const queueFolder = join(path, 'queue',);
  /**
   The runs folder.
   */
  const runsFolder = join(path, 'runs',);
  await mkdir(queueFolder, { recursive: true, },);
  await mkdir(runsFolder, { recursive: true, },);
  return {
    path,
    queueFolder,
    runsFolder,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Says whether one name would send a path out of the folder it is joined to.

 @param folder - the folder the path is built in

 @param path - the path

 @returns whether the path stays in the folder

 @example
 ```ts
 const inside = staysInside({ folder: '/runs', path: '/runs/a.json' });
 ```
 */
function staysInside({
  folder,
  path,
}: {
  readonly folder: string;
  readonly path: string;
},): boolean {
  return dirname(normalize(path,),) === normalize(folder,);
}

/**
 The names of the table a queue may write at all: the ones whose escaped form fits, and whose text is Unicode.

 @returns the labels and names

 @example
 ```ts
 const names = namesThisQueueWrites();
 ```
 */
function namesThisQueueWrites(): readonly (readonly [string, string])[] {
  return AWKWARD_NAMES.filter(function isOneThisQueueWrites([, name,],) {
    try {
      anEscapedName({ name, },);
      return true;
    }
    catch (error) {
      // The name is one this queue refuses, which is the answer this filter asks for.
      void error;
      return false;
    }
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the escaped form of a name',
      children: [
        it({
          name: 'turns every awkward name into text both a file name and a unit name take, and reads each back',
          fn: async () => {
            for (const [label, name,] of namesThisQueueWrites()) {
              /**
               The escaped form, for a name this queue may write at all.
               */
              const escaped = anEscapedName({ name, },);
              // Every character is one a file name and a systemd unit name both keep, or a `\xHH` systemd reads as one byte.
              /**
               The escaped form with every `\xHH` removed, which leaves only the characters kept as they are.
               */
              let kept = escaped;
              while (kept.includes('\\',)) {
                /**
                 Where the first escape begins.
                 */
                const at = kept.indexOf('\\',);
                kept = `${kept.slice(
                  0,
                  at,
                )}${kept.slice(
                  at + 4,
                )}`;
              }
              expect(`${label}: ${String(kept.split('',).every(function isKept(character,) {
                return 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.-'.includes(character,);
              },),)}`,).toBe(`${label}: true`,);
              expect(aNameInAnEscapedName({ escaped, },),).toBe(name,);
            }
          },
        },),
        it({
          name: 'keeps no name a path: no slash, no dot at its start, and no dash at the start of a file name',
          fn: async () => {
            for (const [label, name,] of namesThisQueueWrites()) {
              /**
               The escaped form.
               */
              const escaped = anEscapedName({ name, },);
              expect(`${label}: ${String(escaped.includes('/',),)}`,).toBe(`${label}: false`,);
              expect(escaped.startsWith('.',),).toBe(false,);
              expect(escaped.includes('\0',),).toBe(false,);
            }
          },
        },),
        it({
          name: 'refuses text that is no Unicode, naming the entry it was given for',
          fn: async () => {
            /**
             What escaping a lone surrogate threw.
             */
            let caught: unknown;
            try {
              anEscapedName({ name: 'a\uD800b', },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueNameError,);
            expect(String(caught,),).toContain('no Unicode',);
            expect(String(caught,),).toContain('give it a name that is text',);
          },
        },),
        it({
          name: 'refuses a name whose escaped form would outgrow the longest name a queue writes, naming the entry and the length',
          fn: async () => {
            // A name of spaces escapes to four characters each, so 31 of them are past the ceiling while 30 are not.
            expect(anEscapedName({ name: ' '.repeat(30,), },),).toHaveLength(30 * 4,);
            /**
             What escaping one space too many threw.
             */
            let caught: unknown;
            try {
              anEscapedName({ name: ' '.repeat(31,), },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueNameError,);
            expect(String(caught,),).toContain(String(LONGEST,),);
            expect(String(caught,),).toContain('give it a shorter name',);
          },
        },),
      ],
    },),
    describe({
      name: 'a file name and a unit name of one entry',
      children: [
        it({
          name: 'are one escaped form with different parts around it, and each reads back to the entry\'s name',
          fn: async () => {
            for (const [label, name,] of namesThisQueueWrites()) {
              /**
               The file's name.
               */
              const fileName = anEntryFileName({ name, },);
              /**
               The unit's name.
               */
              const unitName = aUnitNameOf({ name, },);
              expect(`${label}: ${String(anEntryNameInAFileName({ fileName, },),)}`,).toBe(`${label}: ${name}`,);
              expect(`${label}: ${String(anEntryNameInAUnitName({ unitName, },),)}`,).toBe(`${label}: ${name}`,);
              // The unit's name holds the file's name's escaped part, which is what lets a place be counted from a unit.
              expect(unitName,).toContain(fileName.slice(
                0,
                -'.json'.length,
              ),);
            }
          },
        },),
        it({
          name: 'answers nothing for a file or a unit this queue did not name',
          fn: async () => {
            expect(anEntryNameInAFileName({ fileName: 'notes.txt', },),).toBe(NO_NAME_IN_AN_ESCAPED_FORM,);
            expect(anEntryNameInAFileName({ fileName: '%zz.json', },),).toBe(NO_NAME_IN_AN_ESCAPED_FORM,);
            expect(anEntryNameInAUnitName({ unitName: 'pi-some-run', },),).toBe(NO_NAME_IN_AN_ESCAPED_FORM,);
          },
        },),
      ],
    },),
    describe({
      name: 'every path a queue builds from an entry\'s name',
      children: [
        it({
          name: 'stays inside the folder it belongs to, whatever the name holds',
          fn: async () => {
            await using folder = await aFolder();
            for (const [label, name,] of namesThisQueueWrites()) {
              expect(`${label}: ${staysInside({
                folder: folder.queueFolder,
                path: entryFileOf({
                  queueFolder: folder.queueFolder,
                  name,
                },),
              },)}`,).toBe(`${label}: true`,);
              expect(`${label}: ${staysInside({
                folder: join(
                  folder.queueFolder,
                  'claimed',
                ),
                path: claimFileOf({
                  queueFolder: folder.queueFolder,
                  name,
                },),
              },)}`,).toBe(`${label}: true`,);
              expect(`${label}: ${staysInside({
                folder: folder.runsFolder,
                path: setAsideFileOf({
                  runsFolder: folder.runsFolder,
                  name,
                },),
              },)}`,).toBe(`${label}: true`,);
              expect(`${label}: ${staysInside({
                folder: folder.runsFolder,
                path: namesOfAnAttemptIn({
                  runsFolder: folder.runsFolder,
                  name,
                  claim: {
                    token: 'a-token',
                    pid: 1,
                    at: '2026-11-16T00:00:00.000Z',
                  },
                },).batchFile,
              },)}`,).toBe(`${label}: true`,);
            }
          },
        },),
        it({
          name: 'writes nothing outside the queue\'s folders for the name that tries to escape it, end to end',
          fn: async () => {
            await using folder = await aFolder();
            /**
             The name that tries to leave the runs folder.
             */
            const name = 'x/../../escape';
            await putAQueueEntryIn({
              queueFolder: folder.queueFolder,
              entry: {
                name,
                branch: 'feat/escape',
              },
              position: 0,
            },);
            /**
             The entry as the queue holds it.
             */
            const held = await readTheEntryNamed({
              queueFolder: folder.queueFolder,
              name,
            },);
            if (((typeof held) !== 'object') || (held === null)) {
              throw new Error('the queue did not hold the entry it was given',);
            }
            /**
             What taking a claim on it answered.
             */
            const claimed = await claimAnEntryOfAQueue({
              queueFolder: folder.queueFolder,
              record: held,
              now: Date.now,
            },);
            if (claimed.kind !== 'claimed') {
              throw new Error(`the entry was not claimed: ${claimed.kind}`,);
            }
            /**
             The batch list the attempt would write.
             */
            const {batchFile} = namesOfAnAttemptIn({
              runsFolder: folder.runsFolder,
              name,
              claim: claimed.claim,
            },);
            await setAsideAnEntryOfAQueue({
              queueFolder: folder.queueFolder,
              runsFolder: folder.runsFolder,
              record: held,
              queueId: claimed.queueId,
              reason: 'the name tries to leave the folder',
              now: Date.now,
            },);
            // Nothing of the queue's is in the folder above the one the queue and the runs live in, except the two folders themselves.
            expect(batchFile.startsWith(`${folder.runsFolder}/`,),).toBe(true,);
            expect(staysInside({
              folder: folder.runsFolder,
              path: batchFile,
            },),).toBe(true,);
            expect((await readQueueOfRuns({ queueFolder: folder.queueFolder, },)).length,).toBe(0,);
            expect(staysInside({
              folder: folder.runsFolder,
              path: setAsideFileOf({
                runsFolder: folder.runsFolder,
                name,
              },),
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
