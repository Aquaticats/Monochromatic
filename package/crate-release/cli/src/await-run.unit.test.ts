import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { AwaitOptions } from './await-options.ts';
import { awaitRelease } from './await-run.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';

/**
 Options of the fixture await.
 */
const options: AwaitOptions = {
  root: '/repo',
  crateName: 'forbidden-strings',
  releaseRunId: '500',
  version: '0.6.1',
  workflow: 'cargo-publish.yml',
  attempts: 3,
  pollIntervalSeconds: 20,
  runLimit: 10,
  tagTemplate: '{crate}-v{version}',
  assetTemplates: ['{crate}-{version}-x86_64-unknown-linux-gnu.tar.gz'],
  userAgent: 'ua',
  maxTimeSeconds: 30,
};

/**
 Answers of the stubbed programs, in the order a successful await asks for them.
 */
const happy = {
  watchRelease: { status: 0, stdout: '', stderr: '' },
  view: { status: 0, stdout: '{"createdAt":"2026-11-01T10:00:00Z","status":"completed"}', stderr: '' },
  list: {
    status: 0,
    stdout: JSON.stringify([
      { databaseId: 902, headSha: 'b'.repeat(40), createdAt: '2026-11-01T10:05:00Z' },
      { databaseId: 901, headSha: 'a'.repeat(40), createdAt: '2026-11-01T09:00:00Z' },
    ]),
    stderr: '',
  },
  watchPublish: { status: 0, stdout: '', stderr: '' },
  curl: { status: 0, stdout: '{"version":{"created_at":"2026-11-01T10:30:00+00:00"}}', stderr: '' },
  release: {
    status: 0,
    stdout: JSON.stringify({ assets: [{ name: 'forbidden-strings-0.6.1-x86_64-unknown-linux-gnu.tar.gz' }] }),
    stderr: '',
  },
};

/**
 Answers to give in place of the happy ones.
 */
type Answers = {
  readonly watchRelease?: ProgramResult;
  readonly view?: ProgramResult;
  readonly list?: ProgramResult;
  readonly watchPublish?: ProgramResult;
  readonly curl?: ProgramResult;
  readonly release?: ProgramResult;
};

/**
 Run an await against canned answers.

 @param answers - answers to give in place of the happy ones
 @param awaitOptions - options of the run
 @returns the exit status, the calls, the sleeps and the printed lines
 */
async function record({
  answers = {},
  awaitOptions = options,
}: {
  readonly answers?: Answers;
  readonly awaitOptions?: AwaitOptions;
} = {}) {
  const merged = { ...happy, ...answers };
  const calls: ProgramCall[] = [];
  const sleeps: number[] = [];
  const printed: string[] = [];
  const status = await awaitRelease({
    options: awaitOptions,
    run: async function run(call) {
      calls.push(call);
      const [first, second] = call.args;
      if (call.program === 'curl')
        return merged.curl;
      if ((first === 'run') && (second === 'watch')) {
        // The current call is already recorded, so the first watch is the release run's and the second the publish run's.
        const watches = calls.filter(function isWatch(recorded) {
          return (recorded.args[0] === 'run') && (recorded.args[1] === 'watch');
        }).length;
        return watches === 1 ? merged.watchRelease : merged.watchPublish;
      }
      if ((first === 'run') && (second === 'view'))
        return merged.view;
      if ((first === 'run') && (second === 'list'))
        return merged.list;
      return merged.release;
    },
    sleep: async function sleep(milliseconds) {
      sleeps.push(milliseconds);
    },
    print: function print(line) {
      printed.push(line);
    },
  });
  return { status, calls, sleeps, printed };
}

await describe({
  name: '',
  children: [
    describe({
      name: awaitRelease.name,
      children: [
        it({
          name: 'watches the release run, finds the publish run it started, watches that and checks crates.io and the release',
          fn: async () => {
            const result = await record();
            expect(result.status).toBe(0);
            expect(result.calls.map(function argsOf(call) {
              return call.args.slice(0, 3).join(' ');
            })).toEqual([
              'run watch 500',
              'run view 500',
              'run list --workflow=cargo-publish.yml',
              'run watch 902',
              '--silent --show-error --max-time',
              'release view forbidden-strings-v0.6.1',
            ]);
            expect(result.printed).toContain('crate-release.yml run 500');
            expect(result.printed).toContain(`publish run built from ${'b'.repeat(40)}`);
            expect(result.printed).toContain('run 902');
            expect(result.printed.at(-1)).toBe('PUBLISHED');
          },
        }),
        it({
          name: 'asks for manually started publish runs only, and watches the release run with its exit status',
          fn: async () => {
            const result = await record();
            expect(result.calls[0]?.args).toEqual(['run', 'watch', '500', '--exit-status']);
            expect(result.calls[2]?.args).toContain('--event=workflow_dispatch');
          },
        }),
        it({
          name: 'picks the earliest publish run created since the release run, not an older manual one',
          fn: async () => {
            const result = await record();
            expect(result.calls[3]?.args.slice(0, 3)).toEqual(['run', 'watch', '902']);
          },
        }),
        it({
          name: 'stops at a failed release run, names its log, and looks for no publish run',
          fn: async () => {
            const result = await record({ answers: { watchRelease: { status: 1, stdout: '', stderr: '' } } });
            expect(result.status).toBe(1);
            expect(result.printed.at(-1)).toContain('RELEASE RUN FAILED: no publish was started');
            expect(result.printed.at(-1)).toContain('gh run view 500 --log-failed');
            expect(result.calls).toHaveLength(1);
          },
        }),
        it({
          name: 'stops when the release run cannot be read, with the reason',
          fn: async () => {
            const result = await record({ answers: { view: { status: 1, stdout: '', stderr: 'HTTP 404' } } });
            expect(result.status).toBe(1);
            expect(result.printed.at(-1)).toBe('RELEASE RUN UNREADABLE: HTTP 404');
            expect(result.calls.some(function isList(call) {
              return call.args[1] === 'list';
            })).toBe(false);
          },
        }),
        it({
          name: 'waits the poll interval between looks for the publish run until it appears',
          fn: async () => {
            let lists = 0;
            const sleeps: number[] = [];
            const status = await awaitRelease({
              options,
              run: async function run(call) {
                if (call.program === 'curl')
                  return happy.curl;
                if (call.args[1] === 'watch')
                  return happy.watchRelease;
                if ((call.args[1] === 'view') && (call.args[0] === 'run'))
                  return happy.view;
                if (call.args[1] === 'list') {
                  lists += 1;
                  return lists < 3 ? { status: 0, stdout: '[]', stderr: '' } : happy.list;
                }
                return happy.release;
              },
              sleep: async function sleep(milliseconds) {
                sleeps.push(milliseconds);
              },
              print: function print() {},
            });
            expect(status).toBe(0);
            expect(lists).toBe(3);
            expect(sleeps).toEqual([20_000, 20_000]);
          },
        }),
        it({
          name: 'gives up after the last look without sleeping after it, names the bound and the last reason',
          fn: async () => {
            const result = await record({ answers: { list: { status: 0, stdout: '[]', stderr: '' } } });
            expect(result.status).toBe(1);
            expect(result.sleeps).toEqual([20_000, 20_000]);
            expect(result.printed).toContain(
              'no cargo-publish.yml run was started since 2026-11-01T10:00:00Z within 1 minute',
            );
            expect(result.printed.at(-1)).toBe('last result: No listed run was created at or after 2026-11-01T10:00:00Z.');
          },
        }),
        it({
          name: 'shows the gh error when listing the publish runs failed every time',
          fn: async () => {
            const result = await record({ answers: { list: { status: 1, stdout: '', stderr: 'rate limit exceeded' } } });
            expect(result.status).toBe(1);
            expect(result.printed.at(-1)).toBe('last result: rate limit exceeded');
          },
        }),
        it({
          name: 'passes the repository to every gh call when one is named',
          fn: async () => {
            const result = await record({ awaitOptions: { ...options, repo: 'o/r' } });
            const ghCalls = result.calls.filter(function isGh(call) {
              return call.program === 'gh';
            });
            expect(ghCalls.length).toBeGreaterThan(0);
            expect(ghCalls.every(function hasRepo(call) {
              return call.args.includes('--repo=o/r');
            })).toBe(true);
          },
        }),
        it({
          name: 'fails when the publish run failed, but still reports crates.io and the release',
          fn: async () => {
            const result = await record({
              answers: {
                watchPublish: { status: 1, stdout: '', stderr: '' },
                curl: { status: 0, stdout: '{"errors":[]}', stderr: '' },
              },
            });
            expect(result.status).toBe(1);
            expect(result.printed.at(-1)).toContain('PUBLISH CHECK FAILED');
            expect(result.printed.some(function isCrates(line) {
              return line.startsWith('crates.io 0.6.1: not found');
            })).toBe(true);
          },
        }),
      ],
    }),
  ],
});
