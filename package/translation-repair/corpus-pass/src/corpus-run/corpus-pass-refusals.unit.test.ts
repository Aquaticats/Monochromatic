/**
 Tests for the refusals of `corpus-pass` that were wrong before its wiring
 moved out of its entry file, run as built: a limit that cannot be read, or a
 runs directory holding an artifact nothing can place, is a stated refusal at
 exit 6 with its message and no frames, and a ceiling of one minute is said in
 the singular.

 THE LIMITS: the cap and the spend ceiling were resolved while the file
 loaded, outside the refusal boundary, so an unreadable value exited 1 under
 Node's echo of the bundle's source line instead of exit 6; the corpus pin
 was resolved the same way until 2026-10-10 (ledger B302). THE ARTIFACT
 GUARDS: their errors are plain errors that forward their message, so the
 command printed an operator's refusal as a fault in the command, at exit 5,
 with frames. THE CAP LINE wrote "1 minutes".

 Every child runs through `runBuiltWithoutKeys`, which removes every variable
 whose name ends in `_API_KEY` and every variable whose name starts
 `TRANSLATION_REPAIR_` before the case adds the scratch locations it needs.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ENTRIES,
  NO_KEY_REFUSAL,
  pinLine,
  PROVIDERS_OFF_LINES,
  REFUSED_AS_STATED,
  runPass,
  startLine,
  WRITER_GRACE_BUILT_IN,
} from './corpus-pass-built.test-fixture.ts';
import { makeCorpusPassClone, } from './corpus-pass-clone.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'corpus-pass refusals as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES AN UNREADABLE HARD CAP at exit 6 with its line alone, nothing on stdout and no lock taken',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote for a cap that is not a number.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: { TRANSLATION_REPAIR_HARD_CAP_MINUTES: 'abc', },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'corpus-pass: TRANSLATION_REPAIR_HARD_CAP_MINUTES must be a positive number of minutes; received '
                + '"abc". This is the ceiling that stops one entry running away with a whole pass, so an '
                + 'unreadable value is refused rather than quietly replaced by the default: an operator who set it '
                + 'believes the run is bounded the way they asked for.\n',
            );
            /**
             Where the lock would stand had the pass claimed the directory.
             */
            const lockPath = join(
              runs.path,
              'pass.lock',
            );
            await expect(readFile(
              lockPath,
              'utf8',
            ),).rejects.toThrow('ENOENT',);
          },
        },),

        it({
          name: 'REFUSES AN UNREADABLE SPEND CEILING at exit 6 with its line alone and nothing on stdout',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote for a ceiling that is not a number.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: { TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD: 'abc', },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'corpus-pass: TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD must be a non-negative number of USD; '
                + 'received "abc". This is the ceiling that stops a run spending past its allowance, so an '
                + 'unreadable value is refused rather than quietly replaced by the default: an operator who set it '
                + 'believes the run is bounded the way they asked for.\n',
            );
          },
        },),

        it({
          name: 'REFUSES AN UNREADABLE CORPUS COMMIT at exit 6 with its line alone, nothing on stdout and no lock '
            + 'taken',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote for a commit that is no full commit name.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: { TRANSLATION_REPAIR_CORPUS_COMMIT: 'whiskers', },
            },);

            expect(run,).toEqual({
              code: REFUSED_AS_STATED,
              stdout: '',
              stderr: 'corpus-pass: TRANSLATION_REPAIR_CORPUS_COMMIT must be one full lowercase hexadecimal commit '
                + 'name, and whiskers is not one\n',
            },);
            await expect(readFile(
              join(
                runs.path,
                'pass.lock',
              ),
              'utf8',
            ),).rejects.toThrow('ENOENT',);
          },
        },),

        it({
          name: 'SAYS A CAP OF ONE MINUTE IN THE SINGULAR on the line naming the ceiling a run ran under',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote with the cap at one minute.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: { TRANSLATION_REPAIR_HARD_CAP_MINUTES: '1', },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe([
              await startLine({
                pending: 2,
                hardMs: 60_000,
              },),
              'CAP OVERRIDDEN by TRANSLATION_REPAIR_HARD_CAP_MINUTES: entries run under 1 minute rather than '
                + 'the built-in 420',
              WRITER_GRACE_BUILT_IN,
              pinLine({ clone, },),
              'CAP TOO TIGHT: an attempt runs 60000ms, which is not longer than the 460000ms one model '
                + 'exchange is allowed (360000ms by default, 460000ms for a Claude exchange). Attempts are cut before '
                + 'an exchange can return, so no slice caches, every '
                + 'attempt reports no progress, and the queue drops the entry as stalled after its second try. '
                + 'Raise the ceiling above one exchange to buy anything, or keep it here to exercise the stall '
                + 'path deliberately.',
              ...PROVIDERS_OFF_LINES,
              '',
            ].join('\n',),);
            expect(run.stderr,).toBe(NO_KEY_REFUSAL,);
          },
        },),

        it({
          name: 'ROUNDS A FRACTIONAL CAP TO THE WHOLE MILLISECOND at the one place minutes become milliseconds, so '
            + '4.35 minutes is 261000ms and no hold bound refuses it',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote with the cap at a decimal whose product with 60000 is not a whole number of
             floating-point milliseconds (4.35 * 60000 is 260999.99999999997).
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: { TRANSLATION_REPAIR_HARD_CAP_MINUTES: '4.35', },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe([
              await startLine({
                pending: 2,
                hardMs: 261_000,
              },),
              'CAP OVERRIDDEN by TRANSLATION_REPAIR_HARD_CAP_MINUTES: entries run under 4.35 minutes rather than '
                + 'the built-in 420',
              WRITER_GRACE_BUILT_IN,
              pinLine({ clone, },),
              'CAP TOO TIGHT: an attempt runs 261000ms, which is not longer than the 460000ms one model '
                + 'exchange is allowed (360000ms by default, 460000ms for a Claude exchange). Attempts are cut before '
                + 'an exchange can return, so no slice caches, every '
                + 'attempt reports no progress, and the queue drops the entry as stalled after its second try. '
                + 'Raise the ceiling above one exchange to buy anything, or keep it here to exercise '
                + 'the stall path deliberately.',
              ...PROVIDERS_OFF_LINES,
              '',
            ].join('\n',),);
            expect(run.stderr,).toBe(NO_KEY_REFUSAL,);
          },
        },),

        it({
          name: 'REFUSES A POSITIVE CAP THAT ROUNDS TO ZERO MILLISECONDS at exit 6 with its line alone, naming the '
            + 'figure and the smallest figure accepted',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote with the cap at 0.0000001 minutes, which is 0.006 milliseconds.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: { TRANSLATION_REPAIR_HARD_CAP_MINUTES: '0.0000001', },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'corpus-pass: TRANSLATION_REPAIR_HARD_CAP_MINUTES must be a positive number of minutes, and a ceiling '
                + 'is read in whole milliseconds; received "0.0000001", which rounds to 0 milliseconds. The smallest '
                + 'figure accepted is half a millisecond, written as 0.0000083334 minutes. This is the ceiling that '
                + 'stops one entry running away with a whole pass, so a figure it cannot state is refused rather '
                + 'than quietly replaced by another: an operator who set it believes the run is bounded the way '
                + 'they asked for.\n',
            );
          },
        },),

        it({
          name: 'REFUSES AN ARTIFACT NOTHING CAN PLACE at exit 6 with its message alone and no frames, since it is an '
            + 'operator\'s refusal and no fault of the command',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            // An artifact recording nothing, which the pool cannot place.
            await mkdir(join(
              runs.path,
              'artifacts',
            ),);
            await writeFile(
              join(
                runs.path,
                'artifacts',
                'tabby.json',
              ),
              '{}',
              'utf8',
            );

            /**
             What the command wrote against the directory holding it.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: {},
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(
              'POOL tabby.json records an id that is not its file name (absent); treating it as unplaceable\n',
            );
            expect(run.stderr,).toBe(
              'corpus-pass: 1 artifact in this directory records no readable pipeline:\n'
                + '  tabby\n'
                + '\n'
                + 'Each one is an entry that has SILENTLY CEASED TO EXIST. The scheduler\n'
                + 'counts every .json name as settled, so it will never be retried, and\n'
                + 'the pool filter excludes an artifact of unknown generation, so it is\n'
                + 'absent from every rate. Nothing else reports the gap.\n'
                + '\n'
                + 'Deleting the file is the whole remedy: the next pass re-runs that\n'
                + 'entry from scratch.\n'
                + '\n'
                + 'It is no longer ROUTINE maintenance, and it used to be. Artifacts are\n'
                + 'written to a temporary name and renamed once the entry completes, so a\n'
                + 'pass killed at its hard cap leaves a .partial rather than half a\n'
                + '.json. Reaching here now means a legacy artifact from before that, a\n'
                + 'concurrent writer, a permission fault, or storage trouble.\n',
            );
          },
        },),
      ],
    },),
  ],
},);
