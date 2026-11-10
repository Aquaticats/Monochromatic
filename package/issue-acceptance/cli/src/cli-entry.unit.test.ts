/**
 Tests for the command's entry: each subcommand end to end with fakes.

 @module
 */

import {
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  EXIT_FAILED,
  EXIT_OK,
  PlanError,
  runCli,
  type GitReader,
  type IssueTracker,
} from '../dist/final/node/index.mjs';

/**
 Returns the error an asynchronous call rejects with.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call resolves, which means it refused nothing.
 
 @example
 ```ts
 const refused = await rejectionOf(() => runCli({ ...request }));
 ```
 */
async function rejectionOf(action: () => Promise<unknown>,): Promise<unknown> {
  try {
    await action();
  }
  catch (error) {
    return error;
  }
  throw new Error('The call resolved where the test expects a refusal.',);
}

/**
 Streams that collect what the command printed.
 
 @returns The streams and the lines they hold.
 
 @example
 ```ts
 const { streams, lines } = collectingStreams();
 ```
 */
function collectingStreams(): {
  readonly streams: {
    readonly write: (line: string,) => void;
    readonly errorWrite: (line: string,) => void;
  };
  readonly lines: string[];
} {
  /**
   Lines the command printed.
   */
  const lines: string[] = [];
  return {
    lines,
    streams: {
      write: function write(line,) {
        lines.push(line,);
      },
      errorWrite: function errorWrite(line,) {
        lines.push(line,);
      },
    },
  };
}

/**
 Tracker that answers from the given listings and refuses every mutation.
 
 @param listings - Open issues the search reports.
 
 @returns A tracker for the test.
 
 @example
 ```ts
 const tracker = readOnlyTracker({ listings: [] });
 ```
 */
function readOnlyTracker({
  listings,
}: {
  readonly listings: readonly {
    readonly number: number;
    readonly title: string;
    readonly body: string;
  }[];
},): IssueTracker {
  return {
    state: async function state() {
      return 'OPEN';
    },
    close: async function close() {
      throw new Error('the test never closes an issue',);
    },
    comment: async function comment() {
      throw new Error('the test never comments on an issue',);
    },
    commentOnCommit: async function commentOnCommit() {
      throw new Error('the test never comments on a commit',);
    },
    read: async function read(number,) {
      return {
        number,
        title: 'An issue',
        state: 'OPEN',
        body: 'The body',
        comments: [],
      };
    },
    listOpenWithLabels: async function listOpenWithLabels() {
      return listings;
    },
  };
}

/**
 Branch reader that answers one presence answer for every path.
 
 @param present - Whether the branch tracks the path a check names.
 
 @returns A branch reader for the test.
 
 @example
 ```ts
 const reader = readerFor({ present: true });
 ```
 */
function readerFor({ present, }: { readonly present: boolean; },): GitReader {
  return {
    countMatches: async function countMatches() {
      return 0;
    },
    fileText: async function fileText() {
      return { kind: 'absent', };
    },
    pathExists: async function pathExists() {
      return present;
    },
    trackedPaths: async function trackedPaths() {
      return ['README.md',];
    },
    isOnBranch: async function isOnBranch() {
      return true;
    },
  };
}

/**
 Writes a plan holding one issue whose only check asks whether `README.md` is there.
 
 @param directory - Folder the plan lives in.
 
 @returns The plan file's path.
 
 @example
 ```ts
 const planPath = await writePlan('/tmp/x');
 ```
 */
async function writePlan(directory: string,): Promise<string> {
  /**
   Path the plan is written to.
   */
  const path = join(directory, 'plan.json',);
  await writeFile(path, JSON.stringify({
    repository: 'OWNER/NAME',
    checkout: '/repo',
    branch: 'main',
    issues: [{
      issue: 7,
      action: 'close',
      checks: [{ kind: 'exists', path: 'README.md', },],
      comment: 'Fixed on main.',
    },],
  },), 'utf8',);
  return path;
}

await describe({
  name: runCli.name,
  children: [
    it({
      name: 'prints the usage text for --help',
      fn: async () => {
        /**
         Streams collecting the output.
         */
        const { streams, lines } = collectingStreams();
        expect(await runCli({
          arguments: ['--help',],
          streams,
        },),).toBe(EXIT_OK,);
        expect(lines.join('',),).toContain('issue-acceptance settle --plan FILE',);
      },
    },),
    it({
      name: 'prints one issue through show',
      fn: async () => {
        /**
         Streams collecting the output.
         */
        const { streams, lines } = collectingStreams();
        expect(await runCli({
          arguments: [
            'show',
            '--repo',
            'OWNER/NAME',
            '7',
          ],
          streams,
          createTracker: function tracker() {
            return readOnlyTracker({ listings: [], },);
          },
        },),).toBe(EXIT_OK,);
        expect(lines.join('',),).toContain('#7 [OPEN] An issue',);
      },
    },),
    it({
      name: 'prints the ready-bug grouping and writes it when asked',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-cli-',),);
        /**
         File the grouping is also written to.
         */
        const outPath = join(directory.path, 'grouping.json',);
        /**
         Streams collecting the output.
         */
        const { streams, lines } = collectingStreams();
        expect(await runCli({
          arguments: [
            'ready-bugs',
            '--repo',
            'OWNER/NAME',
            '--out',
            outPath,
          ],
          streams,
          createTracker: function tracker() {
            return readOnlyTracker({
              listings: [{ number: 4, title: 'package/a/x', body: '', },],
            },);
          },
        },),).toBe(EXIT_OK,);
        expect(lines,).toContain('1\tpackage/a/x\t4\n',);
        expect(
          JSON.parse(await readFile(outPath, 'utf8',),),
        ).toEqual({
          considered: 1,
          paused: 0,
          groups: [{ packagePath: 'package/a/x', issues: [4,], },],
        },);
      },
    },),
    it({
      name: 'settles a plan without writing when apply is absent',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-cli-',),);
        /**
         Plan the run settles.
         */
        const planPath = await writePlan(directory.path,);
        /**
         Streams collecting the output.
         */
        const { streams, lines } = collectingStreams();
        expect(await runCli({
          arguments: [
            'settle',
            '--plan',
            planPath,
          ],
          streams,
          createReader: function reader() {
            return readerFor({ present: true, },);
          },
          createTracker: function tracker() {
            return readOnlyTracker({ listings: [], },);
          },
        },),).toBe(EXIT_OK,);
        expect(lines,).toContain('#7 holds on main: would close\n',);
      },
    },),
    it({
      name: 'reports a failed check with the failure status',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-cli-',),);
        /**
         Plan the run settles.
         */
        const planPath = await writePlan(directory.path,);
        /**
         Streams collecting the output.
         */
        const { streams, lines } = collectingStreams();
        expect(await runCli({
          arguments: [
            'settle',
            '--plan',
            planPath,
          ],
          streams,
          createReader: function reader() {
            return readerFor({ present: false, },);
          },
          createTracker: function tracker() {
            return readOnlyTracker({ listings: [], },);
          },
        },),).toBe(EXIT_FAILED,);
        expect(lines,).toContain('#7 DOES NOT HOLD: exists README.md answered false\n',);
      },
    },),
    it({
      name: 'refuses a plan file that is not there',
      fn: async () => {
        /**
         Streams collecting the output.
         */
        const { streams } = collectingStreams();
        /**
         Value the run threw.
         */
        const refused = await rejectionOf(() => runCli({
          arguments: [
            'settle',
            '--plan',
            join(scratchRoot(), 'issue-acceptance-absent', 'plan.json',),
          ],
          streams,
        },),);
        expect(refused,).toBeInstanceOf(PlanError,);
      },
    },),
  ],
},);
