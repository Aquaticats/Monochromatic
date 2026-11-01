import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { AwaitOptions } from './await-options.ts';
import { awaitRelease } from './await-run.ts';
import {
  failureExitCode,
  millisecondsPerSecond,
  releaseWorkflow,
} from './constants.ts';
import {
  describeDuration,
  expandTemplate,
} from './publish-results.ts';
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
  manifestPath: 'package/forbidden-strings/cli/Cargo.toml',
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
 When the release run reports it was created.
 */
const releaseCreatedAt = '2026-11-01T10:00:00Z';

/**
 Milliseconds the await sleeps between two looks, as the production poll computes it.
 */
const pollSleepMilliseconds = options.pollIntervalSeconds * millisecondsPerSecond;

/**
 Commit of the publish run that the happy listing holds after the release run, and of the run older than it.
 */
const publishSha = 'b'.repeat(40);
const olderSha = 'a'.repeat(40);

/**
 Version that the commit of the crate holds before the release, which is not the released one.
 */
const previousVersion = '0.6.0';

/**
 Text of a crate manifest that holds a version.

 @param version - version the package table holds
 @returns the manifest text
 */
function manifestOf(version: string): string {
  return `[package]\nname = "forbidden-strings"\nversion = "${version}"\n`;
}

/**
 Version that each commit holds in the crate manifest; a commit absent from the table cannot be read.
 */
const happyVersions: ReadonlyMap<string, string> = new Map([
  [publishSha, options.version],
  [olderSha, previousVersion],
]);

/**
 Answers of the stubbed programs, in the order a successful await asks for them.
 */
const happy = {
  watchRelease: { status: 0, stdout: '', stderr: '' },
  view: { status: 0, stdout: `{"createdAt":"${releaseCreatedAt}","status":"completed"}`, stderr: '' },
  list: {
    status: 0,
    stdout: JSON.stringify([
      { databaseId: 902, headSha: publishSha, createdAt: '2026-11-01T10:05:00Z' },
      { databaseId: 901, headSha: olderSha, createdAt: '2026-11-01T09:00:00Z' },
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
  readonly versions?: ReadonlyMap<string, string>;
  readonly watchPublish?: ProgramResult;
  readonly curl?: ProgramResult;
  readonly release?: ProgramResult;
};

/**
 Answer of the stubbed `gh api` call that reads the manifest at the commit its endpoint names.

 @param call - the program call
 @param versions - version per commit
 @returns the manifest text, or a not found answer for a commit the table lacks
 */
function manifestAnswer({
  call,
  versions,
}: {
  readonly call: ProgramCall;
  readonly versions: ReadonlyMap<string, string>;
}): ProgramResult {
  const sha = call.args[1]?.split('?ref=')[1] ?? '';
  const held = versions.get(sha);
  return held === undefined
    ? { status: 1, stdout: '', stderr: `HTTP 404 for ${sha}` }
    : { status: 0, stdout: manifestOf(held), stderr: '' };
}

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
  const merged = { ...happy, versions: happyVersions, ...answers };
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
      if (first === 'api')
        return manifestAnswer({ call, versions: merged.versions });
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
              return call.args.slice(0, call.args[0] === 'api' ? 2 : 3).join(' ');
            })).toEqual([
              'run watch 500',
              'run view 500',
              `run list --workflow=${options.workflow}`,
              `api repos/{owner}/{repo}/contents/${options.manifestPath}?ref=${publishSha}`,
              'run watch 902',
              '--silent --show-error --max-time',
              `release view ${expandTemplate({ template: options.tagTemplate, crateName: options.crateName, version: options.version })}`,
            ]);
            expect(result.printed).toContain(`${releaseWorkflow} run ${options.releaseRunId}`);
            expect(result.printed).toContain(`publish run built from ${publishSha}`);
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
          name: 'picks the publish run created since the release run, not an older manual one',
          fn: async () => {
            const result = await record();
            expect(result.calls[4]?.args.slice(0, 3)).toEqual(['run', 'watch', '902']);
          },
        }),
        it({
          name: 'picks the release publish run, not an unrelated dispatch created after the release run but before it',
          fn: async () => {
            const unrelated = { databaseId: 903, headSha: 'c'.repeat(40), createdAt: '2026-11-01T10:04:00Z' };
            const own = { databaseId: 904, headSha: 'd'.repeat(40), createdAt: '2026-11-01T10:04:17Z' };
            const result = await record({
              answers: {
                list: { status: 0, stdout: JSON.stringify([own, unrelated]), stderr: '' },
                versions: new Map([
                  [unrelated.headSha, previousVersion],
                  [own.headSha, options.version],
                ]),
              },
            });
            expect(result.status).toBe(0);
            expect(result.printed).toContain(`publish run built from ${own.headSha}`);
            expect(result.printed).toContain(`run ${String(own.databaseId)}`);
            expect(result.calls.some(function watchesUnrelated(call) {
              return call.args.includes(String(unrelated.databaseId));
            })).toBe(false);
          },
        }),
        it({
          name: 'reads the manifest of the repository it was told about at the commit of each run it considers',
          fn: async () => {
            const result = await record({ awaitOptions: { ...options, repo: 'o/r' } });
            const reads = result.calls.filter(function isApi(call) {
              return call.args[0] === 'api';
            });
            expect(reads.map(function endpointOf(call) {
              return call.args[1];
            })).toEqual([`repos/o/r/contents/${options.manifestPath}?ref=${publishSha}`]);
            expect(reads.every(function runsInRoot(call) {
              return (call.program === 'gh') && (call.cwd === options.root);
            })).toBe(true);
          },
        }),
        it({
          name: 'skips a run whose commit cannot be read and takes a later run that holds the released version',
          fn: async () => {
            const unreadable = { databaseId: 903, headSha: 'c'.repeat(40), createdAt: '2026-11-01T10:04:00Z' };
            const own = { databaseId: 904, headSha: 'd'.repeat(40), createdAt: '2026-11-01T10:04:17Z' };
            const result = await record({
              answers: {
                list: { status: 0, stdout: JSON.stringify([own, unreadable]), stderr: '' },
                versions: new Map([[own.headSha, options.version]]),
              },
            });
            expect(result.status).toBe(0);
            expect(result.printed).toContain(`run ${String(own.databaseId)}`);
          },
        }),
        it({
          name: 'finds no publish run when every run since the release holds another version, and names each run and why',
          fn: async () => {
            const result = await record({ answers: { versions: new Map([[publishSha, previousVersion]]) } });
            const lastResult = result.printed.at(-1) ?? '';
            expect(result.status).toBe(failureExitCode);
            expect(lastResult).toContain(`holds version ${JSON.stringify(previousVersion)} in ${options.manifestPath}, not ${JSON.stringify(options.version)}`);
            expect(lastResult).toContain(`run 902 (created 2026-11-01T10:05:00Z, commit ${publishSha})`);
            expect(lastResult).toContain(`run 901 (created 2026-11-01T09:00:00Z, commit ${olderSha}) was created before the release run (${releaseCreatedAt})`);
            expect(result.calls.some(function watchesPublish(call) {
              return call.args.slice(0, 3).join(' ') === 'run watch 902';
            })).toBe(false);
          },
        }),
        it({
          name: 'stops at a failed release run, names its log, and looks for no publish run',
          fn: async () => {
            const result = await record({ answers: { watchRelease: { status: 1, stdout: '', stderr: '' } } });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed.at(-1)).toContain('RELEASE RUN FAILED: no publish was started');
            expect(result.printed.at(-1)).toContain('gh run view 500 --log-failed');
            expect(result.calls).toHaveLength(1);
          },
        }),
        it({
          name: 'stops when the release run cannot be read, with the reason',
          fn: async () => {
            const result = await record({ answers: { view: { status: 1, stdout: '', stderr: 'HTTP 404' } } });
            expect(result.status).toBe(failureExitCode);
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
                if (call.args[0] === 'api')
                  return manifestAnswer({ call, versions: happyVersions });
                return happy.release;
              },
              sleep: async function sleep(milliseconds) {
                sleeps.push(milliseconds);
              },
              print: function print() {},
            });
            expect(status).toBe(0);
            expect(lists).toBe(options.attempts);
            expect(sleeps).toEqual([pollSleepMilliseconds, pollSleepMilliseconds]);
          },
        }),
        it({
          name: 'gives up after the last look without sleeping after it, names the bound and the last reason',
          fn: async () => {
            const result = await record({ answers: { list: { status: 0, stdout: '[]', stderr: '' } } });
            expect(result.status).toBe(failureExitCode);
            expect(result.sleeps).toEqual([pollSleepMilliseconds, pollSleepMilliseconds]);
            expect(result.printed).toContain(
              `no ${options.workflow} run was started since ${releaseCreatedAt} within ${describeDuration(options.attempts * options.pollIntervalSeconds)}`,
            );
            expect(result.printed.at(-1)).toContain(`last result: No listed run was created at or after ${releaseCreatedAt}`);
            expect(result.printed.at(-1)).toContain('The listing holds no run.');
          },
        }),
        it({
          name: 'shows the gh error when listing the publish runs failed every time',
          fn: async () => {
            const result = await record({ answers: { list: { status: 1, stdout: '', stderr: 'rate limit exceeded' } } });
            expect(result.status).toBe(failureExitCode);
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
            // `gh api` has no repository option, so its endpoint carries the repository instead.
            expect(ghCalls.every(function hasRepo(call) {
              return call.args[0] === 'api'
                ? call.args[1]?.startsWith('repos/o/r/contents/') === true
                : call.args.includes('--repo=o/r');
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
            expect(result.status).toBe(failureExitCode);
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
