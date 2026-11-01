import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { commitType, failureExitCode } from './constants.ts';
import { CrateReleaseError } from './error.ts';
import type { PushOptions } from './push-options.ts';
import {
  headParentsArguments,
  headPathsArguments,
  headSubjectArguments,
  pushRelease,
  requestFileName,
} from './push-run.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';

/**
 Options of the fixture push.
 */
const options: PushOptions = {
  root: '/work',
  repo: 'Aquaticats/Monochromatic',
  branch: 'main',
  scratchDirectory: '/runner/temp',
};

/**
 Object names of the head commit and its parent.
 */
const headName = 'b'.repeat(40);
const parentName = 'a'.repeat(40);

/**
 The subject the head commit carries, built the way a release builds it.
 */
const headline = `${commitType}(a): release 0.1.1: n`;

/**
 A canned answer of a program; a field left out takes the default answer's.
 */
type Answer = {
  readonly status?: number;
  readonly stdout?: string;
  readonly stderr?: string;
};

/**
 Canned answers by the first argument of the call.
 */
type Answers = Readonly<Record<string, Answer>>;

/**
 What a run left behind, for assertions.
 */
type Recording = {
  readonly status: number;
  readonly calls: readonly ProgramCall[];
  readonly written: readonly { readonly path: string; readonly text: string }[];
  readonly printed: readonly string[];
};

/**
 Run a push against canned answers.

 @param answers - what `git` and `gh` print, by the first argument of the call; a missing key answers success with no text
 @returns the recorded calls, writes and printed lines
 */
async function record(answers: Answers = {}): Promise<Recording> {
  const defaults: Record<string, ProgramResult> = {
    'rev-list': { status: 0, stdout: `${headName} ${parentName}\n`, stderr: '' },
    'log': { status: 0, stdout: `${headline}\n`, stderr: '' },
    'diff-tree': { status: 0, stdout: 'package/a/Cargo.toml\0package/a/Cargo.lock\0', stderr: '' },
    'api': {
      status: 0,
      stdout: '{"data":{"createCommitOnBranch":{"commit":{"oid":"cccc","url":"https://github.com/o/r/commit/cccc"}}}}',
      stderr: '',
    },
  };
  const calls: ProgramCall[] = [];
  const written: { path: string; text: string }[] = [];
  const printed: string[] = [];
  const status = await pushRelease({
    options,
    run: async function run(call) {
      calls.push(call);
      const key = call.args[0] ?? '';
      return { ...(defaults[key] ?? { status: 0, stdout: '', stderr: '' }), ...answers[key] };
    },
    readBytes: async function readBytes(path) {
      return new TextEncoder().encode(`contents of ${path}`);
    },
    writeText: async function writeText(file) {
      written.push(file);
    },
    print: function print(line) {
      printed.push(line);
    },
  });
  return { status, calls, written, printed };
}

/**
 Run a push that must throw and return what it threw.

 @param answers - canned answers, as for `record`
 @returns the thrown value
 */
async function thrownBy(answers: Answers): Promise<unknown> {
  try {
    await record(answers);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  children: [
    describe({
      name: pushRelease.name,
      children: [
        it({
          name: 'builds the commit on the parent of the head from the head subject and files, and pushes it with gh',
          fn: async () => {
            const result = await record();
            expect(result.status).toBe(0);
            expect(result.written).toHaveLength(1);
            expect(result.written[0]?.path).toBe(join(options.scratchDirectory, requestFileName));
            const request = JSON.parse(result.written[0]?.text ?? '') as { variables: { input: Record<string, unknown> } };
            expect(request.variables.input).toEqual({
              branch: { repositoryNameWithOwner: 'Aquaticats/Monochromatic', branchName: 'main' },
              message: { headline },
              expectedHeadOid: parentName,
              fileChanges: {
                additions: [
                  { path: 'package/a/Cargo.toml', contents: Buffer.from('contents of /work/package/a/Cargo.toml').toString('base64') },
                  { path: 'package/a/Cargo.lock', contents: Buffer.from('contents of /work/package/a/Cargo.lock').toString('base64') },
                ],
              },
            });
            const gh = result.calls.find(function isGh(call) {
              return call.program === 'gh';
            });
            expect(gh).toEqual({
              program: 'gh',
              args: ['api', 'graphql', `--input=${join(options.scratchDirectory, requestFileName)}`],
              cwd: '/work',
            });
            expect(result.printed.at(-1)).toBe('PUSHED cccc https://github.com/o/r/commit/cccc');
          },
        }),
        it({
          name: 'runs git in the checkout and asks for the changed paths NUL-separated, so no path is split or quoted',
          fn: async () => {
            const result = await record();
            const git = result.calls.filter(function isGit(call) {
              return call.program === 'git';
            });
            expect(git.map(function argsOf(call) {
              return call.args.join(' ');
            }).toSorted()).toEqual([
              headPathsArguments.join(' '),
              headSubjectArguments.join(' '),
              headParentsArguments.join(' '),
            ].toSorted());
            expect(git.every(function inRoot(call) {
              return call.cwd === '/work';
            })).toBe(true);
          },
        }),
        it({
          name: 'reports a refusal by GitHub with its message and the way out, and returns the failure status',
          fn: async () => {
            const result = await record({
              api: {
                status: 1,
                stdout: String.raw`{"errors":[{"message":"Expected branch to point to \"aaa\" but it did not."}]}`,
                stderr: 'gh: Expected branch',
              },
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed.some(function isRefusal(line) {
              return line.startsWith('PUSH REFUSED: Expected branch to point to');
            })).toBe(true);
            expect(result.printed.join('\n')).toContain('dispatch the release again');
            expect(result.printed.join('\n')).toContain('gh exited 1: gh: Expected branch');
          },
        }),
        it({
          name: 'reports an answer that is not JSON as a refusal and still names gh status',
          fn: async () => {
            const result = await record({ api: { status: 4, stdout: '', stderr: 'gh: authentication required' } });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed.join('\n')).toContain('PUSH REFUSED: The text is not JSON');
            expect(result.printed.join('\n')).toContain('gh exited 4: gh: authentication required');
          },
        }),
        it({
          name: 'refuses a head that is a merge, because it has no single commit that was checked out',
          fn: async () => {
            const caught = await thrownBy({ 'rev-list': { stdout: `${headName} ${parentName} ${'c'.repeat(40)}\n` } });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('exactly one parent');
          },
        }),
        it({
          name: 'refuses a head with no parent',
          fn: async () => {
            const caught = await thrownBy({ 'rev-list': { stdout: `${headName}\n` } });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('has 0');
          },
        }),
        it({
          name: 'refuses a head that changes no file',
          fn: async () => {
            const caught = await thrownBy({ 'diff-tree': { stdout: '' } });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('nothing to push');
          },
        }),
        it({
          name: 'names the git call, its status and its error text when git fails',
          fn: async () => {
            const caught = await thrownBy({ log: { status: 128, stderr: 'fatal: bad object HEAD\n' } });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain(`git ${headSubjectArguments.join(' ')} failed with status 128`);
            expect((caught as Error).message).toContain('fatal: bad object HEAD');
          },
        }),
        it({
          name: 'sends nothing to GitHub when the head cannot be read',
          fn: async () => {
            const calls: ProgramCall[] = [];
            let caught: unknown;
            try {
              await pushRelease({
                options,
                run: async function run(call) {
                  calls.push(call);
                  return { status: 128, stdout: '', stderr: 'fatal' };
                },
                readBytes: async function readBytes() {
                  return new Uint8Array();
                },
                writeText: async function writeText() {},
                print: function print() {},
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect(calls.some(function isGh(call) {
              return call.program === 'gh';
            })).toBe(false);
          },
        }),
      ],
    }),
  ],
});
