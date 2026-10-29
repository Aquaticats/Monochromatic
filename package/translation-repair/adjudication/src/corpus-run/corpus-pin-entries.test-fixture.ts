/**
 The suite every package that owns a command reading the corpus pin runs over its own commands: an unreadable
 `TRANSLATION_REPAIR_CORPUS_COMMIT` is a stated refusal at exit 6 naming the variable, with nothing on stdout.

 TEST SUPPORT, NOT PACKAGE SOURCE. The commands belong to the packages that build them, so each of those packages
 hands its own commands to `pinReadersSuite` from its own test file, with the entry of its own build; this
 package holds the shared suite and the unreadable setting, since it holds the fixture that removes every key from
 a child's environment. Every child runs through `runBuiltCommand`, which removes every variable whose name ends in
 `_API_KEY` and every variable whose name starts `TRANSLATION_REPAIR_` before the case adds the scratch locations it
 needs and the unreadable commit.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  type DescribeChild,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '../child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { scratchPlaces, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/keyless-built-command.test-fixture.ts';

/**
 Commit setting no reader accepts: a cat's name, not a full lowercase
 hexadecimal commit name.
 */
export const UNREADABLE_COMMIT = 'whiskers';

/**
 One built command that reads the corpus pin, with the entry of the package that owns it and arguments its command
 line accepts, so the run reaches the pin rather than a usage refusal.
 */
export type PinReader = {
  /**
   Name of the built command.
   */
  readonly command: string;

  /**
   URL of the built index of the package that owns the command, which the command's built file sits beside.
   */
  readonly packageEntry: string;

  /**
   Arguments the command's line accepts.
   */
  readonly args: readonly string[];
};

/**
 Runs each built command that reads the corpus pin with the commit setting unreadable and expects the stated
 refusal at exit 6.

 @param readers - commands to run, with the entry of the package that owns each

 @returns Suite of one case per command

 @example
 ```ts
 await describe({ name: '', concurrency: 1, children: [pinReadersSuite({ readers: PIN_READERS, },),], },);
 ```
 */
export function pinReadersSuite({ readers, }: { readonly readers: readonly PinReader[]; },): DescribeChild {
  return describe({
    name: 'corpus pin readers as built',
    concurrency: DEFAULT_CONCURRENCY,
    children: readers.map(function refusalCase({
      command,
      packageEntry,
      args,
    },) {
      return it({
        name: `REFUSES AN UNREADABLE CORPUS COMMIT in ${command} at exit 6 with its line alone and nothing on stdout`,
        fn: async function refusesUnreadableCommit(): Promise<void> {
          /**
           Scratch directory the case owns.
           */
          await using scratch = await scratchDir({ prefix: 'corpus-pin-entries-', },);

          /**
           What the command wrote with the commit setting unreadable.
           */
          const run = await runBuiltCommand({
            command,
            packageEntry,
            args,
            env: {
              ...scratchPlaces({
                runsDir: scratch.path,
                scratchDir: scratch.path,
              },),
              TRANSLATION_REPAIR_CORPUS_COMMIT: UNREADABLE_COMMIT,
            },
          },);

          expect(run,)
            .toEqual({
              code: 6,
              stdout: '',
              stderr: `${command}: TRANSLATION_REPAIR_CORPUS_COMMIT must be one full lowercase hexadecimal commit `
                + `name, and ${UNREADABLE_COMMIT} is not one\n`,
            },);
        },
      },);
    },),
  },);
}
