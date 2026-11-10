/**
 Tests for the GitHub boundary, driven by a fake `gh` executable.

 @module
 */

import {
  chmod,
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
  ChildProcessError,
  createIssueTracker,
} from '../dist/final/node/index.mjs';

/**
 Returns the error an asynchronous call rejects with.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call resolves, which means it refused nothing.
 
 @example
 ```ts
 const refused = await rejectionOf(() => tracker.read(1));
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
 What the fake `gh` does, written so each test reads as one question.
 */
const FAKE_GH = String.raw`#!/usr/bin/env node
import { appendFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
appendFileSync(join(here, 'argv.log'), JSON.stringify(args) + '\n');
const number = args[2];

if (args.includes('--json') && args.includes('state')) {
  process.stdout.write(number === '2' ? 'CLOSED' : 'OPEN');
  process.exit(0);
}
if (args.includes('--json') && args.includes('number,title,state,body,comments')) {
  if (number === '9') {
    process.stdout.write('{"number":"9"}');
    process.exit(0);
  }
  if (number === '8') {
    process.stdout.write('{"number":8,"title":"t","state":"OPEN","body":"b","comments":"none"}');
    process.exit(0);
  }
  process.stdout.write(JSON.stringify({
    number: Number(number),
    title: 'An issue',
    state: 'OPEN',
    body: 'The body',
    comments: [{ body: 'first' }, { body: 'second' }],
  }));
  process.exit(0);
}
if (args[0] === 'issue' && args[1] === 'list') {
  process.stdout.write(JSON.stringify([
    { number: 4, title: 'package/a/x', body: '' },
    { number: 5, title: 'no package', body: 'package/b/y' },
  ]));
  process.exit(0);
}
if (args[0] === 'issue' && args[1] === 'close') {
  if (args.includes('refuse')) {
    process.stderr.write('gh: could not close');
    process.exit(1);
  }
  process.stdout.write('closed');
  process.exit(0);
}
if (args[0] === 'issue' && args[1] === 'comment') {
  process.stdout.write('commented');
  process.exit(0);
}
if (args[0] === 'api') {
  process.stdout.write('40112233');
  process.exit(0);
}
if (args.includes('--json') && args.includes('state')) {
  process.stdout.write('OPEN');
  process.exit(0);
}
process.stdout.write('{}');
process.exit(0);
`;

/**
 Writes the fake `gh` into a fresh folder.
 
 @param directory - Folder the fake lives in.
 
 @returns The fake's path.
 
 @example
 ```ts
 const executable = await writeFakeGh('/tmp/x');
 ```
 */
async function writeFakeGh(directory: string,): Promise<string> {
  /**
   Path the fake is written to.
   */
  const path = join(directory, 'gh',);
  await writeFile(path, FAKE_GH, 'utf8',);
  await chmod(path, 0o700,);
  return path;
}

/**
 Reads the argument vectors the fake `gh` was called with.
 
 @param directory - Folder the fake lives in.
 
 @returns One parsed vector per call.
 
 @example
 ```ts
 const calls = await recordedCalls('/tmp/x');
 ```
 */
async function recordedCalls(directory: string,): Promise<readonly (readonly string[])[]> {
  /**
   Log the fake appended to.
   */
  const text = await readFile(join(directory, 'argv.log',), 'utf8',);
  return text
    .split('\n',)
    .filter(function isCall(line,): boolean {
      return line !== '';
    },)
    .map(function toVector(line,): readonly string[] {
      return JSON.parse(line,) as readonly string[];
    },);
}

await describe({
  name: createIssueTracker.name,
  children: [
    it({
      name: 'reads the state of one open issue and names the repository',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        expect(await tracker.state(7,),).toBe('OPEN',);
        /**
         Argument vectors the fake recorded.
         */
        const calls = await recordedCalls(directory.path,);
        expect(calls[0],).toEqual([
          'issue',
          'view',
          '7',
          '--repo',
          'OWNER/NAME',
          '--json',
          'state',
          '--jq',
          '.state',
        ],);
      },
    },),
    it({
      name: 'reads the state of one closed issue',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        expect(await tracker.state(2,),).toBe('CLOSED',);
      },
    },),
    it({
      name: 'closes an issue with its evidence as one comment',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        await tracker.close({
          number: 7,
          comment: 'Fixed on main.',
        },);
        /**
         Argument vectors the fake recorded.
         */
        const calls = await recordedCalls(directory.path,);
        expect(calls[0],).toEqual([
          'issue',
          'close',
          '7',
          '--repo',
          'OWNER/NAME',
          '--comment',
          'Fixed on main.',
        ],);
      },
    },),
    it({
      name: 'refuses a close that failed, naming the call and its error',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        /**
         Value the close threw.
         */
        const refused = await rejectionOf(() => tracker.close({
          number: 7,
          comment: 'refuse',
        },),);
        expect(refused,).toBeInstanceOf(ChildProcessError,);
        expect((refused as Error).message,).toContain('gh issue close failed',);
      },
    },),
    it({
      name: 'comments on an issue without closing it',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        await tracker.comment({
          number: 7,
          body: 'Still open for one part.',
        },);
        /**
         Argument vectors the fake recorded.
         */
        const calls = await recordedCalls(directory.path,);
        expect(calls[0],).toEqual([
          'issue',
          'comment',
          '7',
          '--repo',
          'OWNER/NAME',
          '--body',
          'Still open for one part.',
        ],);
      },
    },),
    it({
      name: 'records a correction on a commit and returns its id',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        expect(await tracker.commentOnCommit({
          commit: '2b309f8f25',
          body: 'Correction.',
        },),).toBe('40112233',);
        /**
         Argument vectors the fake recorded.
         */
        const calls = await recordedCalls(directory.path,);
        expect(calls[0],).toEqual([
          'api',
          '--method',
          'POST',
          'repos/OWNER/NAME/commits/2b309f8f25/comments',
          '--field',
          'body=Correction.',
          '--jq',
          '.id',
        ],);
      },
    },),
    it({
      name: 'reads one issue whole, including its comments',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        expect(await tracker.read(7,),).toEqual({
          number: 7,
          title: 'An issue',
          state: 'OPEN',
          body: 'The body',
          comments: ['first', 'second',],
        },);
      },
    },),
    it({
      name: 'refuses an issue whose number is not an integer',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        /**
         Value the read threw.
         */
        const refused = await rejectionOf(() => tracker.read(9,),);
        expect(refused,).toBeInstanceOf(ChildProcessError,);
        expect((refused as Error).message,).toContain('number is not an integer',);
      },
    },),
    it({
      name: 'refuses an issue whose comments are not a list',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        /**
         Value the read threw.
         */
        const refused = await rejectionOf(() => tracker.read(8,),);
        expect(refused,).toBeInstanceOf(ChildProcessError,);
        expect((refused as Error).message,).toContain('comments are not a list',);
      },
    },),
    it({
      name: 'lists open issues carrying every label, one flag per label',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: await writeFakeGh(directory.path,),
        },);
        expect(await tracker.listOpenWithLabels(['bug', 'ready-for-agent',],),).toEqual([
          {
            number: 4,
            title: 'package/a/x',
            body: '',
          },
          {
            number: 5,
            title: 'no package',
            body: 'package/b/y',
          },
        ],);
        /**
         Argument vectors the fake recorded.
         */
        const calls = await recordedCalls(directory.path,);
        expect(calls[0],).toEqual([
          'issue',
          'list',
          '--repo',
          'OWNER/NAME',
          '--state',
          'open',
          '--limit',
          '5000',
          '--label',
          'bug',
          '--label',
          'ready-for-agent',
          '--json',
          'number,title,body',
        ],);
      },
    },),
    it({
      name: 'refuses a search answer that is not a list',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Fake that answers one object where a list belongs.
         */
        const path = join(directory.path, 'gh',);
        await writeFile(path, FAKE_GH.replace(
          "if (args[0] === 'issue' && args[1] === 'list') {",
          "if (args[0] === 'issue' && args[1] === 'list') { process.stdout.write('{\"number\":1}'); process.exit(0);",
        ), 'utf8',);
        await chmod(path, 0o700,);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: path,
        },);
        /**
         Value the search threw.
         */
        const refused = await rejectionOf(() => tracker.listOpenWithLabels(['bug',],),);
        expect(refused,).toBeInstanceOf(ChildProcessError,);
        expect((refused as Error).message,).toContain('something other than a list of issues',);
      },
    },),
    it({
      name: 'refuses an answer that is not JSON',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-gh-',),);
        /**
         Fake that answers text where JSON belongs.
         */
        const path = join(directory.path, 'gh',);
        await writeFile(path, FAKE_GH.replace(
          "if (args[0] === 'issue' && args[1] === 'list') {",
          "if (args[0] === 'issue' && args[1] === 'list') { process.stdout.write('not json'); process.exit(0);",
        ), 'utf8',);
        await chmod(path, 0o700,);
        /**
         Tracker under test.
         */
        const tracker = createIssueTracker({
          repository: 'OWNER/NAME',
          executable: path,
        },);
        /**
         Value the search threw.
         */
        const refused = await rejectionOf(() => tracker.listOpenWithLabels(['bug',],),);
        expect(refused,).toBeInstanceOf(ChildProcessError,);
        expect((refused as Error).message,).toContain('not JSON',);
      },
    },),
  ],
},);
