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
  recordedCargoPublishRunPassed,
  recordedReleaseRunFailed,
  recordedReleaseRunPassed,
  recordedRunStillGoing,
} from './fixture/recorded-run-answers.ts';
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
  followAttempts: 3,
  runLimit: 10,
  tagTemplate: '{crate}-v{version}',
  assetTemplates: ['{crate}-{version}-x86_64-unknown-linux-gnu.tar.gz'],
  userAgent: 'ua',
  maxTimeSeconds: 30,
};

/**
 Milliseconds the await sleeps between two looks, as the production poll computes it.
 */
const pollSleepMilliseconds = options.pollIntervalSeconds * millisecondsPerSecond;

/**
 How long the await may follow one run, in the words its lines use.
 */
const followBound = describeDuration(options.followAttempts * options.pollIntervalSeconds);

/**
 When the release run of the recorded pass was created.
 */
const releaseCreatedAt = String((recordedReleaseRunPassed as { createdAt: string }).createdAt);

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
 An answer of `gh run view` that reports the given text.

 @param body - the answer as JSON text
 @returns the finished call
 */
function answer(body: string): ProgramResult {
  return {
    status: 0,
    stdout: body,
    stderr: ''
  };
}

/**
 The recorded release run that completed with success, as an answer.

 @returns the finished call
 */
function releasePassedAnswer(): ProgramResult {
  return answer(JSON.stringify(recordedReleaseRunPassed));
}

/**
 The recorded release run that failed at its `pnpm install` step, as an answer.

 @returns the finished call
 */
function releaseFailedAnswer(): ProgramResult {
  return answer(JSON.stringify(recordedReleaseRunFailed));
}

/**
 The recorded `cargo-publish` run that built the archives, as an answer.

 @returns the finished call
 */
function publishPassedAnswer(): ProgramResult {
  return answer(JSON.stringify(recordedCargoPublishRunPassed));
}

/**
 The recorded run that was still going when it was captured, as an answer.

 @returns the finished call
 */
function stillGoingAnswer(): ProgramResult {
  return answer(JSON.stringify(recordedRunStillGoing));
}

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
  release: releasePassedAnswer(),
  list: answer(JSON.stringify([
    { databaseId: 902, headSha: publishSha, createdAt: '2026-11-11T01:00:30Z' },
    { databaseId: 901, headSha: olderSha, createdAt: '2026-10-01T09:00:00Z' },
  ])),
  publish: publishPassedAnswer(),
  curl: answer('{"version":{"created_at":"2026-11-11T01:30:00+00:00"}}'),
  releaseView: answer(JSON.stringify({
    assets: [{ name: expandTemplate({ template: options.assetTemplates[0] ?? '', crateName: options.crateName, version: options.version }) }],
  })),
};

/**
 Answers to give in place of the happy ones.
 */
type Answers = {
  /**
   Answers of `gh run view` for the release run, in order; the last one repeats.
   */
  readonly releaseViews?: readonly ProgramResult[];
  /**
   Answer of `gh run view` for every other run.
   */
  readonly publishView?: ProgramResult;
  readonly list?: ProgramResult;
  readonly versions?: ReadonlyMap<string, string>;
  readonly curl?: ProgramResult;
  readonly releaseView?: ProgramResult;
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
  const releaseViews = [...(merged.releaseViews ?? [merged.release])];
  const status = await awaitRelease({
    options: awaitOptions,
    run: async function run(call) {
      calls.push(call);
      const [first, second] = call.args;
      if (call.program === 'curl')
        return merged.curl;
      if ((first === 'run') && (second === 'view')) {
        // The release run is read first, so its answer is the one still left in the queue while the queue holds more.
        const next = releaseViews.length > 1 ? releaseViews.shift() : releaseViews[0];
        // The publish run is every run this await reads after the release run.
        const isReleaseRun = call.args[2] === options.releaseRunId;
        return isReleaseRun ? (next ?? merged.release) : merged.publish;
      }
      if ((first === 'run') && (second === 'list'))
        return merged.list;
      if (first === 'api')
        return manifestAnswer({ call, versions: merged.versions });
      return merged.releaseView;
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
          name: 'follows the release run, finds the publish run it started, follows that and checks crates.io and the release',
          fn: async () => {
            const result = await record();
            expect(result.status).toBe(0);
            expect(result.calls.map(function argsOf(call) {
              return call.args.slice(0, call.args[0] === 'api' ? 2 : 3).join(' ');
            })).toEqual([
              'run view 500',
              `run list --workflow=${options.workflow}`,
              `api repos/{owner}/{repo}/contents/${options.manifestPath}?ref=${publishSha}`,
              'run view 902',
              '--silent --show-error --max-time',
              `release view ${expandTemplate({ template: options.tagTemplate, crateName: options.crateName, version: options.version })}`,
            ]);
            expect(result.printed).toContain(`${releaseWorkflow} run ${options.releaseRunId}`);
            expect(result.printed).toContain(`${releaseWorkflow} run ${options.releaseRunId}: completed with success`);
            expect(result.printed).toContain(`publish run built from ${publishSha}`);
            expect(result.printed).toContain(`${options.workflow} run 902: completed with success`);
            expect(result.printed.at(-1)).toBe(
              `PUBLISHED ${expandTemplate({ template: options.tagTemplate, crateName: options.crateName, version: options.version })}`,
            );
          },
        }),
        it({
          name: 'asks for what the release run says about itself before it looks for any publish run',
          fn: async () => {
            const result = await record();
            expect(result.calls[0]?.args).toEqual([
              'run',
              'view',
              '500',
              '--json=createdAt,status,conclusion,jobs',
            ]);
            expect(result.calls[1]?.args).toContain('--event=workflow_dispatch');
          },
        }),
        it({
          name: 'picks the publish run created since the release run, not an older manual one',
          fn: async () => {
            const result = await record();
            expect(result.calls[3]?.args.slice(0, 3)).toEqual(['run', 'view', '902']);
            expect(result.sleeps).toEqual([]);
          },
        }),
        it({
          name: 'picks the release publish run, not an unrelated dispatch created after the release run but before it',
          fn: async () => {
            const unrelated = { databaseId: 903, headSha: 'c'.repeat(40), createdAt: '2026-11-11T00:56:00Z' };
            const own = { databaseId: 904, headSha: 'd'.repeat(40), createdAt: '2026-11-11T00:56:17Z' };
            const result = await record({
              answers: {
                list: answer(JSON.stringify([own, unrelated])),
                versions: new Map([
                  [unrelated.headSha, previousVersion],
                  [own.headSha, options.version],
                ]),
                releaseView: answer(JSON.stringify({
                  assets: [{ name: expandTemplate({ template: options.assetTemplates[0] ?? '', crateName: options.crateName, version: options.version }) }],
                })),
              },
            });
            expect(result.status).toBe(0);
            expect(result.printed).toContain(`publish run built from ${own.headSha}`);
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
            const unreadable = { databaseId: 903, headSha: 'c'.repeat(40), createdAt: '2026-11-11T00:56:00Z' };
            const own = { databaseId: 904, headSha: 'd'.repeat(40), createdAt: '2026-11-11T00:56:17Z' };
            const result = await record({
              answers: {
                list: answer(JSON.stringify([own, unreadable])),
                versions: new Map([[own.headSha, options.version]]),
                releaseView: answer(JSON.stringify({
                  assets: [{ name: expandTemplate({ template: options.assetTemplates[0] ?? '', crateName: options.crateName, version: options.version }) }],
                })),
              },
            });
            expect(result.status).toBe(0);
            expect(result.printed).toContain(`${options.workflow} run 904: completed with success`);
          },
        }),
        it({
          name: 'passes over a run whose commit is not a hash, without calling gh for it, and names why',
          fn: async () => {
            const odd = { databaseId: 903, headSha: 'main?x=1', createdAt: '2026-11-11T00:56:00Z' };
            const result = await record({
              answers: { list: answer(JSON.stringify([odd])) },
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed.at(-1)).toContain(`run ${String(odd.databaseId)}`);
            expect(result.printed.at(-1)).toContain('the commit');
            expect(result.calls.some(function readsManifest(call) {
              return call.args[0] === 'api';
            })).toBe(false);
          },
        }),
        it({
          name: 'finds no publish run when every run since the release holds another version, and names each run and why',
          fn: async () => {
            const result = await record({ answers: { versions: new Map([[publishSha, previousVersion]]) } });
            const lastResult = result.printed.at(-1) ?? '';
            expect(result.status).toBe(failureExitCode);
            expect(lastResult).toContain(`holds version ${JSON.stringify(previousVersion)} in ${options.manifestPath}, not ${JSON.stringify(options.version)}`);
            expect(lastResult).toContain(`run 902 (created 2026-11-11T01:00:30Z, commit ${publishSha})`);
            expect(lastResult).toContain(`run 901 (created 2026-10-01T09:00:00Z, commit ${olderSha}) was created before the release run (${releaseCreatedAt})`);
            expect(result.calls.some(function readsPublish(call) {
              return (call.args[0] === 'run') && (call.args[1] === 'view') && (call.args[2] === '902');
            })).toBe(false);
          },
        }),
        it({
          name: 'stops at a release run that failed, names the job and the step, and looks for no publish run',
          fn: async () => {
            const result = await record({ answers: { releaseViews: [releaseFailedAnswer()] } });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed).toContain(`${releaseWorkflow} run ${options.releaseRunId}: completed with failure`);
            expect(result.printed).toContain(
              'job release concluded failure at step "Install Node, pnpm and the workspace dependencies"',
            );
            expect(result.printed.at(-1)).toContain('RELEASE RUN FAILED: no publish was started');
            expect(result.printed.at(-1)).toContain(`gh run view ${options.releaseRunId} --log-failed`);
            expect(result.calls).toHaveLength(1);
          },
        }),
        it({
          name: 'reads the release run again after a look that failed, and passes a release whose run then completed with success',
          fn: async () => {
            const result = await record({
              answers: {
                releaseViews: [
                  { status: 1, stdout: '', stderr: 'HTTP 500: Server Error' },
                  releasePassedAnswer(),
                ],
              },
            });
            expect(result.status).toBe(0);
            expect(result.sleeps).toEqual([pollSleepMilliseconds]);
            expect(result.printed[0]).toContain(`${releaseWorkflow} run ${options.releaseRunId} could not be read (HTTP 500: Server Error)`);
            expect(result.printed).toContain(`${releaseWorkflow} run ${options.releaseRunId}: completed with success`);
            expect(result.printed.at(-1)).toContain('PUBLISHED ');
          },
        }),
        it({
          name: 'says a release run that never ended is still going, names what it waited for, and looks for no publish run',
          fn: async () => {
            const result = await record({ answers: { releaseViews: [stillGoingAnswer()] } });
            expect(result.status).toBe(failureExitCode);
            expect(result.sleeps).toEqual([pollSleepMilliseconds, pollSleepMilliseconds]);
            expect(result.printed[0]).toContain(`${releaseWorkflow} run ${options.releaseRunId} is still going: `);
            expect(result.printed[0]).toContain(`waiting up to ${followBound}`);
            // The line names the bound and what the wait was still waiting for.
            expect((result.printed.at(-1) ?? '').startsWith(
              `RELEASE RUN NOT FINISHED: The run was still going after ${followBound}, waiting for `,
            )).toBe(true);
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
            const printed: string[] = [];
            const status = await awaitRelease({
              options,
              run: async function run(call) {
                if (call.program === 'curl')
                  return happy.curl;
                if (call.args[1] === 'view')
                  return call.args[2] === options.releaseRunId ? happy.release : happy.publish;
                if (call.args[1] === 'list') {
                  lists += 1;
                  return lists < 3 ? answer('[]') : happy.list;
                }
                if (call.args[0] === 'api')
                  return manifestAnswer({ call, versions: happyVersions });
                return happy.releaseView;
              },
              sleep: async function sleep(milliseconds) {
                sleeps.push(milliseconds);
              },
              print: function printLine(line) {
                printed.push(line);
              },
            });
            expect(status).toBe(0);
            expect(lists).toBe(options.attempts);
            expect(sleeps).toEqual([pollSleepMilliseconds, pollSleepMilliseconds]);
          },
        }),
        it({
          name: 'gives up after the last look without sleeping after it, names the bound and the last reason',
          fn: async () => {
            const result = await record({ answers: { list: answer('[]') } });
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
          name: 'fails when the publish run failed, names its job and step, but still reports crates.io and the release',
          fn: async () => {
            const result = await record({
              answers: {
                publishView: answer(JSON.stringify({
                  createdAt: '2026-11-11T01:00:30Z',
                  status: 'completed',
                  conclusion: 'failure',
                  jobs: [{
                    name: 'gpc-publish-crate',
                    status: 'completed',
                    conclusion: 'failure',
                    steps: [{
                      name: 'Publish the crate to crates.io',
                      number: 3,
                      status: 'completed',
                      conclusion: 'failure'
                    }],
                  }],
                })),
                curl: answer('{"errors":[]}'),
                releaseView: answer(JSON.stringify({ assets: [] })),
              },
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed).toContain(
              'job gpc-publish-crate concluded failure at step "Publish the crate to crates.io"',
            );
            expect(result.printed.at(-1)).toContain('PUBLISH CHECK FAILED');
            expect(result.printed.some(function isCrates(line) {
              return line.startsWith(`crates.io ${options.version}: not found`);
            })).toBe(true);
          },
        }),
      ],
    }),
  ],
});
