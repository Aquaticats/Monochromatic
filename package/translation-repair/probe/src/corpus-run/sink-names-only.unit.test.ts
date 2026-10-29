/**
 Tests that every place reporting a caught failure names it rather than
 repeating what it said.

 SEPARATE FROM `message-names-only.unit.test.ts`, which asks the other half of
 the same question. That one reads the CLASSES and decides which may repeat
 their message. This one reads the SINKS: the four places that catch a failure
 while reading an artifact, a lock file or a run file, and print or record a
 reason for it. A marked class is only safe if the sink actually asks. The standing-read command, a sink the bench
 package owns, is read in `editor-standing-read-sink.unit.test.ts` of that package, through the same fixture.

 WHY AN UNREADABLE FILE AND NOT A MALFORMED ONE. A malformed file reaches
 every sink through `parseRunJson`, which already wraps it in a marked class,
 so `refusalText` and a bare `error.message` return the same string and no
 case here could tell them apart. Reverting all four sinks to the bare message
 and running the whole suite proved exactly that: 686 of 686 still passed.

 A file that will not OPEN is the case that separates them. It used to arrive
 as an ordinary `Error` reading

   EACCES: permission denied, open '/tmp/attribution-read-XXXX/Basket.json'

 and a run directory path names the run, while under `artifacts/` a file's own
 stem is a person's entry id. So these cases mode a fixture to `000` and pin
 what comes back.

 EACH CASE CHECKS ITS OWN FIXTURE FIRST. A run as root opens a mode-`000` file
 regardless, which would leave every assertion of the case testing the happy path
 while reporting a pass, so the helper reads the file back and refuses if it
 succeeded.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPlacement, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { lockRunsDir, } from '@monochromatic-dev/translation-repair-run';
import { gatherAttributionEntries, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  LEAKED_OPENING,
  namedRefusal,
  SOUND_ARTIFACT,
  UNREADABLE,
  unopenable,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/unopenable-file.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

//region Sink naming tests

/**
 Lock file name `lockRunsDir` competes for, which it does not take as input.
 */
const LOCK_FILE = 'pass.lock';

/**
 Console methods a sink may write through: `console.log` for a command's own
 stdout lines, and the three the tagged logger's console sink resolves by
 level at flush time.
 */
const CONSOLE_METHODS = [
  'log',
  'info',
  'warn',
  'error',
] as const;

/**
 Collects what would have gone to the console on any of those methods,
 restoring the real ones on disposal.

 THE LOGGER IS CAPTURED TOO, because the lock speaks through a tagged logger
 since provider-12, whose console sink resolves `console.warn` lazily and
 flushes on a microtask, so a wrapper installed before the call sees the line.

 @param lines - collector the caller reads afterwards

 @returns Collected lines, and the restore that disposal runs

 @example
 ```ts
 using printed = collectingLogs({ lines: [], },);
 ```
 */
function collectingLogs(
  { lines, }: { readonly lines: string[]; },
): { readonly lines: readonly string[]; } & Disposable {
  /**
   Real reporters, put back on disposal.
   */
  const reported = new Map(CONSOLE_METHODS.map(function keep(method,): [
    typeof method,
    (...parts: readonly unknown[]) => void,
  ] {
    return [
      method,
      console[method],
    ];
  },),);

  for (const method of CONSOLE_METHODS) {
    console[method] = (...parts: readonly unknown[]) => {
      lines.push(parts.map(String,)
        .join(' ',),);
    };
  }
  return {
    lines,
    [Symbol.dispose]: () => {
      for (const [method, real,] of reported) {
        console[method] = real;
      }
    },
  };
}

await describe({
  name: 'a sink reporting a failure names it',
  children: [
    it({
      name: 'NAMES the code where an artifact will not open, in the reason it '
        + 'hands its caller, since that reason travels into a report',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sink-attribution-', },);

        await writeFile(
          join(
            scratch.path,
            'Whiskers.json',
          ),
          SOUND_ARTIFACT,
          'utf8',
        );
        await unopenable({
          dir: scratch.path,
          name: UNREADABLE,
          body: SOUND_ARTIFACT,
        },);

        // The census prints its own POOL lines, which are not what this case
        // reads, and letting them through would bury the runner's output.
        using swallowed = collectingLogs({ lines: [], },);

        /**
         What the directory yielded.
         */
        const { malformed, } = await gatherAttributionEntries({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          artifactsDir: scratch.path,
        },);

        expect(malformed,).toHaveLength(1,);
        expect(malformed[0]?.name,).toBe(UNREADABLE,);
        expect(malformed[0]?.reason,).toBe(namedRefusal({ file: UNREADABLE, },),);
        expect(swallowed.lines.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'REFUSES to put the path in the reason, which is the whole point: a '
        + "run directory names the run and an artifact's stem is a person",
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sink-attribution-', },);

        await writeFile(
          join(
            scratch.path,
            'Whiskers.json',
          ),
          SOUND_ARTIFACT,
          'utf8',
        );
        await unopenable({
          dir: scratch.path,
          name: UNREADABLE,
          body: SOUND_ARTIFACT,
        },);

        using swallowed = collectingLogs({ lines: [], },);

        /**
         What the directory yielded.
         */
        const { malformed, } = await gatherAttributionEntries({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          artifactsDir: scratch.path,
        },);

        /**
         Everything the reader said, its own report included.
         */
        const said = [
          ...swallowed.lines,
          ...malformed.map(function reasonOf(one,): string {
            return one.reason;
          },),
        ].join('\n',);

        expect(said.includes(scratch.path,),).toBe(false,);
        expect(said.includes(LEAKED_OPENING,),).toBe(false,);
      },
    },),
    it({
      name: 'NAMES the code on the POOL line where a placement will not open, '
        + 'rather than printing the path the filesystem error carried',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sink-placement-', },);

        await unopenable({
          dir: scratch.path,
          name: UNREADABLE,
          body: SOUND_ARTIFACT,
        },);

        using printed = collectingLogs({ lines: [], },);

        /**
         How the unreadable artifact placed.
         */
        const placement = await readPlacement({
          artifactsDir: scratch.path,
          name: UNREADABLE,
        },);

        expect(placement.kind,).toBe('malformed',);
        expect(printed.lines,).toHaveLength(1,);
        expect(printed.lines[0],).toBe(
          `POOL malformed ${UNREADABLE}: ${namedRefusal({ file: UNREADABLE, },)}`,
        );
      },
    },),
    it({
      name: 'NAMES the code on the LOCK line where the lock file will not open, '
        + 'and still takes the lock over, since an unreadable lock holds nobody',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sink-lock-', },);

        /**
         Lock file naming a live holder that nothing can read.
         */
        const lockPath = await unopenable({
          dir: scratch.path,
          name: LOCK_FILE,
          body: JSON.stringify({
            pid: process.pid,
            startedAt: new Date().toISOString(),
          },),
        },);

        using printed = collectingLogs({ lines: [], },);

        await using _held = await lockRunsDir({ runsDir: scratch.path, },);

        /**
         Everything the claim said.
         */
        const said = printed.lines.join('\n',);

        expect(said.includes(
          `${lockPath} unreadable (${namedRefusal({ file: LOCK_FILE, },)})`,
        ),).toBe(true,);
        expect(said.includes(LEAKED_OPENING,),).toBe(false,);
      },
    },),
  ],
  concurrency: 1,
},);

//endregion Sink naming tests
