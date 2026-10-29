/**
 Tests for whether a capped entry earns another attempt inside one invocation.

 The largest entries were measured as unable to settle inside the 420-minute
 hard cap, and the way out ranked first needs a build that does not move
 between attempts. One invocation IS such a window, so the pass now re-queues
 an entry that made progress. What that costs if it is wrong is the whole
 three-day soft budget spent on one entry, so the stop condition is the part
 worth testing hardest.

 THE TWO CASES THAT MATTER ARE BOTH COUNTERINTUITIVE, and both are about a
 cache count that FELL. A settled entry discards its cache on the way out, so
 inferring progress from the count alone would read the one outcome the pass
 exists to reach as the sharpest possible stall. A reset cache, from an entry
 carrying slices of an older build, reads as negative progress while actually
 being a fresh generation's first attempt.

 `countCachedSlices` is tested against a real directory rather than a stub
 because the thing it has to get right is which files are slices, and the
 generation markers that must not be counted are real files with a real
 suffix sitting in the same directory.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  countCachedSlices,
  readAttemptOutcome,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readAttemptOutcome.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS settled when an artifact was written, whatever the cache '
            + 'did. A settled entry discards its slice cache on the way out, so '
            + 'this is the case where the count falls furthest and means the most',
          fn: async () => {
            expect(readAttemptOutcome({
              outcome: { kind: 'settled', },
              cachedBefore: 64,
              cachedAfter: 0,
            },),).toEqual({ kind: 'settled', },);
          },
        },),

        it({
          name: 'EARNS another attempt when the entry cached slices it did not '
            + 'have, since the next attempt then starts further along than this '
            + 'one did and the cap is the only thing that stopped it',
          fn: async () => {
            expect(readAttemptOutcome({
              outcome: { kind: 'resumable-failure', },
              cachedBefore: 45,
              cachedAfter: 64,
            },),).toEqual({
              kind: 'earned',
              gained: 19,
            },);
          },
        },),

        it({
          name: 'STOPS WHOLE-ENTRY RETRY when stage-local work remains despite cache growth',
          fn: async () => {
            expect(readAttemptOutcome({
              outcome: { kind: 'stopped', },
              cachedBefore: 0,
              cachedAfter: 13,
            },),).toEqual({ kind: 'stopped', },);
          },
        },),

        it({
          name: 'DECLINES WHOLE-ENTRY RETRY for an entry the pipeline declined, whatever the cache grew by',
          fn: async () => {
            expect(readAttemptOutcome({
              outcome: { kind: 'declined', },
              cachedBefore: 0,
              cachedAfter: 13,
            },),).toEqual({ kind: 'declined', },);
          },
        },),

        it({
          name: 'REFUSES another attempt when the count did not move, which is the '
            + 'stop condition: no progress guarantee holds, so an entry that '
            + 'bought nothing would repeat itself until the soft budget was gone',
          fn: async () => {
            expect(readAttemptOutcome({
              outcome: { kind: 'resumable-failure', },
              cachedBefore: 45,
              cachedAfter: 45,
            },),).toEqual({
              kind: 'stalled',
              cached: 45,
            },);
          },
        },),

        it({
          name: 'EARNS another attempt when the count FELL but slices remain, '
            + 'because a fall means the lane discarded an older build\'s cache and '
            + 'every slice left was bought by this attempt. Reading the plain '
            + 'difference would drop the entry exactly when it started paying for '
            + 'a fresh generation',
          fn: async () => {
            expect(readAttemptOutcome({
              outcome: { kind: 'resumable-failure', },
              cachedBefore: 65,
              cachedAfter: 10,
            },),).toEqual({
              kind: 'earned',
              gained: 10,
            },);
          },
        },),

        it({
          name: 'REFUSES another attempt when a reset left nothing at all, so an '
            + 'entry that cannot cache even one slice under this build stops '
            + 'rather than repeating a failure the cache cannot shorten',
          fn: async () => {
            expect(readAttemptOutcome({
              outcome: { kind: 'resumable-failure', },
              cachedBefore: 65,
              cachedAfter: 0,
            },),).toEqual({
              kind: 'stalled',
              cached: 0,
            },);
          },
        },),

        it({
          name: 'EARNS the first attempt on an entry with no cache at all, which '
            + 'is every large entry the first time it is seen',
          fn: async () => {
            expect(readAttemptOutcome({
              outcome: { kind: 'resumable-failure', },
              cachedBefore: 0,
              cachedAfter: 45,
            },),).toEqual({
              kind: 'earned',
              gained: 45,
            },);
          },
        },),
      ],
    },),

    describe({
      name: countCachedSlices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS slices of every lane and no generation marker, so progress '
            + 'in one lane counts while the `.txt` markers beside them do not',
          fn: async () => {
            /**
             Throwaway cache directory standing in for one entry's.
             */
            await using scratch = await scratchDir({ prefix: 'whiskers-cache-', },);
            const dir = scratch.path;

            await Promise.all([
              'generation.txt',
              'translate-generation.txt',
              'contest-generation.txt',
              '0-1-tabby.json',
              'translate.0-1-tabby.json',
              'contest.0-1-tabby.json',
            ].map(async function writeOne(name,): Promise<void> {
              await writeFile(
                join(
                  dir,
                  name,
                ),
                '{}\n',
              );
            },),);

            expect(await countCachedSlices({ dir, },),).toBe(3,);
          },
        },),

        it({
          name: 'REPORTS zero for a directory that does not exist, which is every '
            + 'entry before its first slice is bought',
          fn: async () => {
            expect(await countCachedSlices({
              dir: join(
                scratchRoot(),
                'whiskers-cache-that-was-never-created',
              ),
            },),).toBe(0,);
          },
        },),

        it({
          name: 'COUNTS ONLY FILES: a directory named like a slice was bought by no attempt, and a slice still '
            + 'being written under its `.partial` name is not yet one (ledger B65)',
          fn: async () => {
            /**
             Throwaway cache directory standing in for one entry's.
             */
            await using scratch = await scratchDir({ prefix: 'whiskers-cache-', },);
            const dir = scratch.path;

            await writeFile(
              join(
                dir,
                '0-1-tabby.json',
              ),
              '{}\n',
            );
            await writeFile(
              join(
                dir,
                'translate.0-2-tabby.json.4242.partial',
              ),
              '{',
            );
            await mkdir(join(
              dir,
              'contest.0-3-tabby.json',
            ),);

            expect(await countCachedSlices({ dir, },),).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
