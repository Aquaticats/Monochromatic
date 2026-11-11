import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialLines } from './adversarial-fixture.ts';
import {
  failureExitCode,
  releaseWorkflow,
  releaseWorkflowInputs,
} from './constants.ts';
import type { DispatchOptions } from './dispatch-options.ts';
import { dispatchRelease } from './dispatch-run.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';

/**
 Options of the fixture dispatch.
 */
const options: DispatchOptions = {
  root: '/repo',
  crate: 'forbidden-strings',
  from: '0.6.0',
  to: '0.6.1',
  note: 'Skip the generated rule sources.',
  dryRun: false,
};

/**
 Inputs every dispatch sends: the workflow file names them and a dry run adds one more.
 */
const requiredInputCount = Object.keys(releaseWorkflowInputs)
  .filter(function isRequired(name) {
    return name !== 'dryRun';
  }).length;

/**
 What the canned `git` answers for each subcommand the origin check runs; every field defaults to the answer of a
 checkout whose default branch origin holds completely.
 */
type GitReplies = {
  readonly lsRemote?: ProgramResult;
  readonly fetch?: ProgramResult;
  readonly isAncestor?: ProgramResult;
  readonly revList?: ProgramResult;
  readonly log?: ProgramResult;
};

/**
 The program a crate's dispatch gate runs in these tests, which is neither `git` nor `gh`, so its calls are told apart
 from both.
 */
const gateProgram = 'node';

/**
 The dispatch gate the tests give a crate: the command its release waits for.
 */
const gateCall: ProgramCall = {
  program: gateProgram,
  args: ['bin/host-controls.ts'],
  cwd: '/repo/package/gated/cli',
};

/**
 Default branch the canned remote names, which differs from `main` so a hard-coded branch name fails a test.
 */
const defaultBranch = 'trunk';

/**
 A finished program with the given status and output.

 @param status - exit status
 @param stdout - captured standard output
 @param stderr - captured standard error
 @returns the result
 */
function finished({
  status = 0,
  stdout = '',
  stderr = '',
}: {
  readonly status?: number;
  readonly stdout?: string;
  readonly stderr?: string;
} = {}): ProgramResult {
  return { status, stdout, stderr };
}

/**
 Answer one `git` call the way a checkout whose default branch origin holds completely would, unless overridden.

 @param call - the program call
 @param replies - overrides by subcommand
 @returns the canned result
 */
function answerGit({
  call,
  replies,
}: {
  readonly call: ProgramCall;
  readonly replies: GitReplies;
}): ProgramResult {
  /**
   Canned answers by the subcommand that asks.
   */
  const answers: Readonly<Record<string, ProgramResult>> = {
    'ls-remote': replies.lsRemote ?? finished({
      stdout: `ref: refs/heads/${defaultBranch}\tHEAD\n0123456789abcdef0123456789abcdef01234567\tHEAD\n`,
    }),
    'fetch': replies.fetch ?? finished(),
    'merge-base': replies.isAncestor ?? finished(),
    'rev-list': replies.revList ?? finished({ stdout: '1\n' }),
    'log': replies.log ?? finished({ stdout: 'abc1234\tAdd the thing\n' }),
  };
  /**
   The answer for this call's subcommand.
   */
  const answer = answers[call.args[0] ?? ''];
  if (answer === undefined)
    throw new Error(`Unexpected git call: ${call.args.join(' ')}`);
  return answer;
}

/**
 Run a dispatch against a canned `git` and a canned `gh`.

 @param dispatchOptions - options of the run
 @param result - what `gh` prints and exits with
 @param git - overrides of what `git` answers
 @returns the exit status, every call in order, the calls `gh` got and the printed lines
 */
async function record({
  dispatchOptions = options,
  result = { status: 0, stdout: '', stderr: '' },
  gate = { status: 0, stdout: '', stderr: '' },
  git = {},
}: {
  readonly dispatchOptions?: DispatchOptions;
  readonly result?: ProgramResult;
  readonly gate?: ProgramResult;
  readonly git?: GitReplies;
} = {}) {
  const allCalls: ProgramCall[] = [];
  const printed: string[] = [];
  const status = await dispatchRelease({
    options: dispatchOptions,
    run: async function run(call) {
      allCalls.push(call);
      if (call.program === gateProgram)
        return gate;
      return call.program === 'git' ? answerGit({ call, replies: git }) : result;
    },
    print: function print(line) {
      printed.push(line);
    },
  });
  const calls = allCalls.filter(function isGh(call) {
    return call.program === 'gh';
  });
  const gateCalls = allCalls.filter(function isGate(call) {
    return call.program === gateProgram;
  });
  return {
    status,
    allCalls,
    calls,
    gateCalls,
    printed,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: dispatchRelease.name,
      children: [
        it({
          name: 'starts the workflow with gh in the repository root and prints the run address',
          fn: async () => {
            const result = await record({
              result: {
                status: 0,
                stdout: '✓ Created workflow_dispatch event for crate-release.yml at main\nhttps://github.com/o/r/actions/runs/42\n',
                stderr: '',
              },
            });
            expect(result.status).toBe(0);
            expect(result.calls).toEqual([
              {
                program: 'gh',
                args: [
                  'workflow',
                  'run',
                  releaseWorkflow,
                  `--raw-field=${releaseWorkflowInputs.crate}=${options.crate}`,
                  `--raw-field=${releaseWorkflowInputs.from}=${options.from}`,
                  `--raw-field=${releaseWorkflowInputs.to}=${options.to}`,
                  `--raw-field=${releaseWorkflowInputs.note}=${options.note}`,
                ],
                cwd: '/repo',
              },
            ]);
            expect(result.printed).toContain('https://github.com/o/r/actions/runs/42');
            expect(result.printed.at(-1)).toBe('RELEASE STARTED https://github.com/o/r/actions/runs/42');
          },
        }),
        it({
          name: 'prints a copyable command line before it starts the workflow',
          fn: async () => {
            const result = await record();
            expect(result.printed.find(function isGhLine(line) {
              return line.startsWith('$ gh ');
            })).toBe(
              `$ gh workflow run ${releaseWorkflow} --raw-field=${releaseWorkflowInputs.crate}=${options.crate} --raw-field=${releaseWorkflowInputs.from}=${options.from} --raw-field=${releaseWorkflowInputs.to}=${options.to} '--raw-field=${releaseWorkflowInputs.note}=${options.note}'`,
            );
          },
        }),
        it({
          name: 'names a dry run in the closing line and passes the input',
          fn: async () => {
            const result = await record({
              dispatchOptions: { ...options, dryRun: true },
              result: { status: 0, stdout: 'https://github.com/o/r/actions/runs/7\n', stderr: '' },
            });
            expect(result.calls[0]?.args).toContain(`--raw-field=${releaseWorkflowInputs.dryRun}=true`);
            expect(result.printed.at(-1)).toBe('RELEASE DRY RUN STARTED https://github.com/o/r/actions/runs/7');
          },
        }),
        it({
          name: 'passes the repository to gh when one is named',
          fn: async () => {
            const result = await record({ dispatchOptions: { ...options, repo: 'o/r' } });
            expect(result.calls[0]?.args).toContain('--repo=o/r');
          },
        }),
        it({
          name: 'says so and shows how to list the run when gh printed no address',
          fn: async () => {
            const result = await record({
              result: { status: 0, stdout: '✓ Created workflow_dispatch event for crate-release.yml at main\n', stderr: '' },
            });
            expect(result.status).toBe(0);
            expect(result.printed.at(-1)).toContain('RELEASE STARTED, but gh did not print the address of the created run.');
            expect(result.printed.at(-1)).toContain(`gh run list --workflow=${releaseWorkflow} --limit=1`);
          },
        }),
        it({
          name: 'shows what gh said, names the workflow and returns the failure status when gh refuses',
          fn: async () => {
            const result = await record({
              result: { status: 1, stdout: '', stderr: "could not find any workflows named crate-release.yml\n" },
            });
            expect(result.status).toBe(failureExitCode);
            expect(result.printed).toContain('could not find any workflows named crate-release.yml');
            expect(result.printed.at(-1)).toBe(`FAILED ${releaseWorkflow} was not started (gh exited 1)`);
            expect(result.printed.some(function isStarted(line) {
              return line.startsWith('RELEASE STARTED');
            })).toBe(false);
          },
        }),
        //region Origin containment before the dispatch

        describe({
          name: 'origin containment check',
          children: [
            it({
              name: 'dispatches when origin holds every commit of the local default branch, after fetching it',
              fn: async () => {
                const result = await record();
                expect(result.status).toBe(0);
                expect(result.calls).toHaveLength(1);
                expect(result.allCalls.map(function programAndSubcommand(call) {
                  return `${call.program} ${call.args[0]}`;
                })).toEqual([
                  'git ls-remote',
                  'git fetch',
                  'git merge-base',
                  'gh workflow',
                ]);
                expect(result.allCalls.every(function inRoot(call) {
                  return call.cwd === options.root;
                })).toBe(true);
              },
            }),
            it({
              name: 'names the default branch origin reports instead of assuming main',
              fn: async () => {
                const result = await record();
                const fetchCall = result.allCalls.find(function isFetch(call) {
                  return call.args[0] === 'fetch';
                });
                const ancestorCall = result.allCalls.find(function isAncestor(call) {
                  return call.args[0] === 'merge-base';
                });
                expect(fetchCall?.args.join(' ')).toContain(`refs/heads/${defaultBranch}:refs/remotes/origin/${defaultBranch}`);
                expect(fetchCall?.args.join(' ')).not.toContain('main');
                expect(ancestorCall?.args).toEqual([
                  'merge-base',
                  '--is-ancestor',
                  `refs/heads/${defaultBranch}`,
                  `refs/remotes/origin/${defaultBranch}`,
                ]);
              },
            }),
            it({
              name: 'refuses before gh when origin lacks one commit, naming it and both ways out',
              fn: async () => {
                const result = await record({
                  git: {
                    isAncestor: finished({ status: 1 }),
                    revList: finished({ stdout: '1\n' }),
                    log: finished({ stdout: 'abc1234\tFix the release note\n' }),
                  },
                });
                const text = result.printed.join('\n');
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
                expect(text).toContain(`REFUSED ${releaseWorkflow} was not started`);
                expect(text).toContain(`1 commit of local ${defaultBranch} that origin/${defaultBranch} does not have`);
                expect(text).toContain('abc1234 Fix the release note');
                expect(text).toContain('wait for auto-push');
                expect(text).toContain(`git push origin ${defaultBranch}`);
                expect(text).toContain('same release command again');
                expect(text).toContain('crates.io');
              },
            }),
            it({
              name: 'refuses before gh when origin lacks several commits, counting them and naming the newest',
              fn: async () => {
                const lacking = 3;
                const result = await record({
                  git: {
                    isAncestor: finished({ status: 1 }),
                    revList: finished({ stdout: `${String(lacking)}\n` }),
                    log: finished({ stdout: 'def5678\tNewest local commit\n' }),
                  },
                });
                const text = result.printed.join('\n');
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
                expect(text).toContain(`${String(lacking)} commits of local ${defaultBranch} that origin/${defaultBranch} do not have`);
                expect(text).toContain('def5678 Newest local commit');
              },
            }),
            it({
              name: 'refuses a dry run too, because its run also builds origin',
              fn: async () => {
                const result = await record({
                  dispatchOptions: { ...options, dryRun: true },
                  git: { isAncestor: finished({ status: 1 }) },
                });
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
              },
            }),
            it({
              name: 'refuses before gh and shows git\'s own message when the fetch fails',
              fn: async () => {
                const gitMessage = 'fatal: unable to access \'https://github.com/o/r.git/\': Could not resolve host: github.com';
                const result = await record({
                  git: { fetch: finished({ status: 128, stderr: `${gitMessage}\n` }) },
                });
                const text = result.printed.join('\n');
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
                expect(text).toContain(gitMessage);
                expect(text).toContain('could not be checked');
                expect(text).toContain('same release command again');
                expect(result.allCalls.some(function isAncestorCheck(call) {
                  return call.args[0] === 'merge-base';
                })).toBe(false);
              },
            }),
            it({
              name: 'refuses before gh and shows git\'s message when origin names no default branch',
              fn: async () => {
                const gitMessage = 'fatal: could not read from remote repository.';
                const result = await record({
                  git: { lsRemote: finished({ status: 128, stderr: `${gitMessage}\n` }) },
                });
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
                expect(result.printed.join('\n')).toContain(gitMessage);
                expect(result.printed.join('\n')).toContain('could not be checked');
              },
            }),
            it({
              name: 'refuses before gh when origin answers without a default branch line',
              fn: async () => {
                const result = await record({
                  git: { lsRemote: finished({ stdout: '' }) },
                });
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
                expect(result.printed.join('\n')).toContain('could not be checked');
              },
            }),
            it({
              name: 'refuses before gh when the containment test itself fails, as for a missing local branch',
              fn: async () => {
                const gitMessage = 'fatal: Not a valid object name refs/heads/trunk';
                const result = await record({
                  git: { isAncestor: finished({ status: 128, stderr: `${gitMessage}\n` }) },
                });
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
                expect(result.printed.join('\n')).toContain(gitMessage);
                expect(result.printed.join('\n')).toContain('could not be checked');
              },
            }),
            it({
              name: 'refuses before gh when the missing commits cannot be counted',
              fn: async () => {
                const gitMessage = 'fatal: bad revision';
                const result = await record({
                  git: {
                    isAncestor: finished({ status: 1 }),
                    revList: finished({ status: 128, stderr: `${gitMessage}\n` }),
                  },
                });
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
                expect(result.printed.join('\n')).toContain(gitMessage);
              },
            }),
          ],
        }),

        //endregion Origin containment before the dispatch

        //region The crate's own release gate

        describe({
          name: 'dispatch gate',
          children: [
            it({
              name: 'waits for the gate of the crate and names the command before the workflow is started',
              fn: async () => {
                const gateOutput = 'Waiting for the btrfs controls on this host: mise run //package/git-policy/cli:native:test:host:btrfs';
                const result = await record({
                  dispatchOptions: { ...options, dispatchGate: gateCall },
                  gate: { status: 0, stdout: `${gateOutput}\n`, stderr: '' },
                });
                expect(result.status).toBe(0);
                expect(result.gateCalls).toEqual([gateCall]);
                expect(result.allCalls.indexOf(gateCall)).toBeLessThan(result.allCalls.indexOf(result.calls[0] ?? gateCall));
                const text = result.printed.join('\n');
                expect(text).toContain(`Waiting for node bin/host-controls.ts`);
                expect(text).toContain(gateOutput);
                expect(result.printed.at(-1)).toContain('RELEASE STARTED');
              },
            }),
            it({
              name: 'refuses before gh when the gate exits nonzero, showing what the gate printed',
              fn: async () => {
                const gateRefusal = 'REFUSED the wrapper release: the btrfs controls on this host did not pass';
                const result = await record({
                  dispatchOptions: { ...options, dispatchGate: gateCall },
                  gate: { status: 1, stdout: '', stderr: `${gateRefusal}\n` },
                });
                expect(result.status).toBe(failureExitCode);
                expect(result.calls).toHaveLength(0);
                const text = result.printed.join('\n');
                expect(text).toContain(gateRefusal);
                expect(text).toContain(`REFUSED ${releaseWorkflow} was not started`);
                expect(text).toContain('node bin/host-controls.ts exited 1');
                expect(text).not.toContain('RELEASE STARTED');
              },
            }),
            it({
              name: 'skips the gate for a dry run and says the dry run proves nothing about it',
              fn: async () => {
                const result = await record({
                  dispatchOptions: {
                    ...options,
                    dryRun: true,
                    dispatchGate: gateCall,
                  },
                  result: { status: 0, stdout: 'https://github.com/o/r/actions/runs/7\n', stderr: '' },
                });
                expect(result.status).toBe(0);
                expect(result.gateCalls).toHaveLength(0);
                expect(result.calls).toHaveLength(1);
                expect(result.printed.join('\n')).toContain('RELEASE DRY RUN: not running node bin/host-controls.ts');
              },
            }),
            it({
              name: 'runs the gate through the streaming runner when the caller supplies one, so an hour of controls stays visible',
              fn: async () => {
                /**
                 The call the gate runner received, which is the gate's own command.
                 */
                const streamed: ProgramCall[] = [];
                const printed: string[] = [];
                const status = await dispatchRelease({
                  options: { ...options, dispatchGate: gateCall },
                  run: async function run(call) {
                    return call.program === 'git' ? answerGit({ call, replies: {} }) : { status: 0, stdout: '', stderr: '' };
                  },
                  runGate: async function runGate(call) {
                    streamed.push(call);
                    return { status: 0, stdout: '', stderr: '' };
                  },
                  print: function print(line) {
                    printed.push(line);
                  },
                });
                expect(status).toBe(0);
                expect(streamed).toEqual([gateCall]);
                expect(printed.at(-1)).toContain('RELEASE STARTED');
              },
            }),
            it({
              name: 'starts the workflow without a gate when the crate names none',
              fn: async () => {
                const result = await record();
                expect(result.status).toBe(0);
                expect(result.gateCalls).toHaveLength(0);
                expect(result.calls).toHaveLength(1);
              },
            }),
          ],
        }),

        //endregion The crate's own release gate

        ...adversarialLines.map(function noteCase(note) {
          return it({
            name: `passes the note ${JSON.stringify(note)} to gh as one argument`,
            fn: async () => {
              const result = await record({ dispatchOptions: { ...options, note } });
              expect(result.calls[0]?.args.at(-1)).toBe(`--raw-field=${releaseWorkflowInputs.note}=${note}`);
              expect(result.calls[0]?.args).toHaveLength(3 + requiredInputCount);
            },
          });
        }),
      ],
    }),
  ],
});
