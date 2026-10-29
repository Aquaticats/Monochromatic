/**
 Tests that the `editor-standing-read` command names a failure rather than repeating what it said, run as an operator
 runs it, since the module exports nothing to call directly.

 A FILE THAT WILL NOT OPEN is the case that separates a sink that asks `refusalText` from one that prints a bare
 `error.message`. It used to arrive as an ordinary `Error` reading

   EACCES: permission denied, open '/tmp/attribution-read-XXXX/Basket.json'

 and under `artifacts/` a file's own stem is a person's entry id. So the case modes a fixture to `000` and pins
 what comes back. The case checks its own fixture first: a run as root opens a mode-`000` file regardless, which
 would leave every assertion testing the happy path while reporting a pass, so the helper reads the file back and
 refuses if it succeeded. The other sinks, which the probe package's own readers and the run package's lock reach,
 are read in `sink-names-only.unit.test.ts` of the probe package, through the same fixture.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  LEAKED_OPENING,
  namedRefusal,
  SOUND_ARTIFACT,
  UNREADABLE,
  unopenable,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/unopenable-file.test-fixture.ts';

//region Standing read sink tests

/**
 What a command left on its two streams.
 */
type CommandStreams = {
  /**
   Everything the command wrote to stdout.
   */
  readonly stdout: string;

  /**
   Everything the command wrote to stderr.
   */
  readonly stderr: string;
};

/**
 Runs one built command and returns both its streams, whatever it exited
 with.

 A NON-ZERO EXIT IS NOT A FAILURE TO RUN HERE. `editor-standing-read` exits 1
 on a fixture that recorded no judged rounds, which is its own verdict and has
 nothing to do with the refusal these cases read. The shared fixture reports an
 exit code rather than rejecting on one, which is why it is used instead of a
 promisified `execFile`: that one rejects on any non-zero exit and hides the streams
 on the rejection.

 @param command - built command to run

 @param packageEntry - URL of the built index of the package that owns the command, as `import.meta.resolve` gives it
 for that package's name

 @param args - arguments the command receives after its script

 @returns Both streams as the command left them

 @throws Error where the command never started at all

 @example
 ```ts
 const { stderr, } = await streamsOf({ command: STANDING_COMMAND, packageEntry: BENCH_ENTRY, args: [dir,], },);
 ```
 */
async function streamsOf(
  {
    command,
    packageEntry,
    args,
  }: {
    readonly command: string;
    readonly packageEntry: string;
    readonly args: readonly string[];
  },
): Promise<CommandStreams> {
  /**
   Command as it finished, or why it never started.
   */
  const finished = await runBuiltCommand({
    command,
    packageEntry,
    args,
  },);


  return {
    stdout: finished.stdout,
    stderr: finished.stderr,
  };
}

/**
 Command whose sink the CLI case exercises. Its module exports nothing, so
 its sink is reachable only by running it, which is also how an operator
 meets it.
 */
const STANDING_COMMAND = 'editor-standing-read';

await describe({
  name: 'a sink reporting a failure names it',
  children: [
    it({
      name: 'NAMES the code where a standing artifact will not open, run as an '
        + 'operator runs it, since the module exports nothing to call directly',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sink-standing-', },);

        /**
         Artifact the command will meet and fail to open.
         */
        const path = await unopenable({
          dir: scratch.path,
          name: UNREADABLE,
          body: SOUND_ARTIFACT,
        },);

        /**
         What the command printed, on both streams.
         */
        const { stderr, } = await streamsOf({
          command: STANDING_COMMAND,
          packageEntry: BENCH_ENTRY,
          args: [scratch.path,],
        },);

        expect(stderr.includes(
          `${STANDING_COMMAND}: ${path} refused, ${namedRefusal({ file: UNREADABLE, },)}`,
        ),).toBe(true,);
        expect(stderr.includes(LEAKED_OPENING,),).toBe(false,);
      },
    },),
  ],
  concurrency: 1,
},);

//endregion Standing read sink tests
