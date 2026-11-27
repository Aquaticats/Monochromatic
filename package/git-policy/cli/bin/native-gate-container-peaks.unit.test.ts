import { mkdir, mkdtempDisposable, readFile, rename, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { wait as waitUntil } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  evidenceDirectoryOf,
  processLabelOf,
  recordGateContainerPeaks,
} from './native-gate-container-peaks.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 An ID in the shape podman writes to a container's ID file.
 */
const suiteId = '3f2a9c1e'.repeat(8,);

/**
 The Clippy container's ID, which is another podman-generated one.
 */
const clippyId = '7b1d4e2f'.repeat(8,);

/**
 Milliseconds between two samples of a fixture run, short enough that a test's own deadlines stay small.
 */
const fixtureIntervalMilliseconds = 20;

/**
 Milliseconds a fixture run may take before the recorder stops by itself, which every test here ends before.
 */
const fixtureDeadlineMilliseconds = 20_000;

/**
 Write one cgroup file, making the directories above it.

 @param scope - the fixture scope's directory

 @param name - the file's name

 @param text - what the file holds

 @returns nothing, once the file is written
 */
async function writeScopeFile({
  scope,
  name,
  text,
}: {
  readonly scope: string;
  readonly name: string;
  readonly text: string;
}): Promise<void> {
  await mkdir(
    scope,
    { recursive: true },
  );
  await writeFile(
    join(scope, name),
    text,
  );
}

/**
 Write the files of one running container's fixture scope, with this test process's own process in it so the recorder's
 process naming has something to read.

 @param path - the scope's directory

 @param peakBytes - the peak this container's cgroup is to hold

 @returns nothing, once the files are written
 */
async function writeRunningScope({
  path,
  peakBytes,
}: {
  readonly path: string;
  readonly peakBytes: number;
}): Promise<void> {
  await writeScopeFile({
    scope: path,
    name: 'memory.current',
    text: '629145600\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.peak',
    text: `${String(peakBytes)}\n`,
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.max',
    text: '2147483648\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.stat',
    text: 'anon 71093452\nfile 481349632\nshmem 860160\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'memory.events',
    text: 'max 0\noom 0\noom_kill 0\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'pids.current',
    text: '23\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'pids.peak',
    text: '133\n',
  });
  await writeScopeFile({
    scope: path,
    name: 'cgroup.procs',
    text: `${String(process.pid)}\n`,
  });
}

await describe({
  name: '',
  children: [
    describe({
      name: evidenceDirectoryOf.name,
      children: [
        it({
          name: 'reads the evidence directory the gate printed, which every container\'s ID file is written to',
          fn: async () => {
            expect(evidenceDirectoryOf({
              logText: [
                'The image build is bounded at 8g, its own measured bound.',
                'Native wrapper verification evidence: /w/target/verification/native-HZPg8O',
                'gpc-gate-native-HZPg8O-suite runs as a container of this run',
              ].join('\n',),
            }),).toBe('/w/target/verification/native-HZPg8O',);
          },
        }),
        it({
          name: 'answers nothing while a log holds no such line yet, which is a gate still building its image',
          fn: async () => {
            expect(evidenceDirectoryOf({ logText: 'The image build is bounded at 8g.', },),).toBeUndefined();
          },
        }),
      ],
    }),
    describe({
      name: processLabelOf.name,
      children: [
        it({
          name: 'names the test a nextest process runs, from the exact filter that process was started with',
          fn: async () => {
            expect(processLabelOf({
              argv: [
                '/work/package/git-policy/cli/target/debug/build/monochromatic-git-policy-cli/85367b807f819bb5/out/git_policy_cli-85367b807f819bb5',
                '--exact',
                'repo_read::repo_read_config_typed::tests::an_empty_value_over_every_documented_key_declines_or_equals_git',
                '--nocapture',
              ],
            }),).toBe('repo_read::repo_read_config_typed::tests::an_empty_value_over_every_documented_key_declines_or_equals_git',);
          },
        }),
        it({
          name: 'names any other program by its own name and first argument, so a fixture\'s Git is told from a test',
          fn: async () => {
            expect(processLabelOf({
              argv: [
                '/usr/bin/git',
                'diff-tree',
                '--root',
                '--no-commit-id',
              ],
            }),).toBe('git diff-tree',);
          },
        }),
        it({
          name: 'answers nothing for a process whose command line is empty, which a zombie or a kernel thread is',
          fn: async () => {
            expect(processLabelOf({ argv: [], },),).toBeUndefined();
          },
        }),
      ],
    }),
    describe({
      name: recordGateContainerPeaks.name,
      children: [
        it({
          name: 'samples both containers of a run beside the tests holding their memory, and marks the role whose cgroup went',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-peaks-test-',
            ),);
            /**
             The run's evidence directory, whose ID files name the two containers.
             */
            const evidence = join(
              tree.path,
              'evidence',
              'native-HZPg8O',
            );
            await mkdir(
              evidence,
              { recursive: true },
            );
            await writeFile(
              join(evidence, 'suite.cid',),
              `${suiteId}\n`,
            );
            await writeFile(
              join(evidence, 'clippy.cid',),
              `${clippyId}\n`,
            );
            /**
             The gate log, which names the evidence directory the way the gate prints it.
             */
            const logPath = join(
              tree.path,
              'gate.log',
            );
            await writeFile(
              logPath,
              `Native wrapper verification evidence: ${evidence}\n`,
            );
            /**
             The cgroup root the fixture scopes live under, in the shape this host holds them.
             */
            const root = join(tree.path, 'cgroup',);
            /**
             The suite's scope, whose peak is the one the record asks about.
             */
            const suiteScope = join(
              root,
              'user.slice',
              'user-1000.slice',
              'user@1000.service',
              'user.slice',
              `libpod-${suiteId}.scope`,
            );
            /**
             Clippy's scope, which this test removes while the recorder watches it.
             */
            const clippyScope = join(
              root,
              'user.slice',
              'user-1000.slice',
              'user@1000.service',
              'user.slice',
              `libpod-${clippyId}.scope`,
            );
            await writeRunningScope({
              path: suiteScope,
              peakBytes: 1_589_641_216,
            });
            await writeRunningScope({
              path: clippyScope,
              peakBytes: 5_535_055_872,
            });
            /**
             Where the recorder writes its samples.
             */
            const outPath = join(
              tree.path,
              'peaks.jsonl',
            );
            // Clippy's container ends part way through, which is what the recorder has to notice and mark.
            const removed = (async function removedMidRun(): Promise<void> {
              await waitUntil(fixtureIntervalMilliseconds * 10,);
              await rename(
                clippyScope,
                join(
                  tree.path,
                  'clippy-scope-gone',
                ),
              );
            })();
            /**
             What the recorder found.
             */
            const outcome = await recordGateContainerPeaks({
              logPath,
              outPath,
              root,
              userId: 1_000,
              intervalMilliseconds: fixtureIntervalMilliseconds,
              processIntervalMilliseconds: fixtureIntervalMilliseconds,
              deadlineMilliseconds: fixtureIntervalMilliseconds * 50,
            });
            await removed;
            expect(outcome.rolesRead,).toEqual([
              'suite',
              'clippy',
            ],);
            expect(outcome.rolesStillRunning,).toEqual(['suite',],);
            expect(outcome.samples,).toBeGreaterThan(1,);
            /**
             Every line the recorder wrote, in the order it wrote them.
             */
            const samples = (await readFile(
              outPath,
              'utf8',
            ))
              .trim()
              .split('\n',)
              .map(function parsed(line) {
                return JSON.parse(line,) as Readonly<Record<string, unknown>>;
              });
            /**
             The suite's samples, which keep arriving while the recorder runs.
             */
            const suiteSamples = samples.filter(function ofSuite(sample) {
              return sample.role === 'suite';
            },);
            expect(suiteSamples.length,).toBeGreaterThan(0,);
            expect(suiteSamples[0]?.peakBytes,).toBe(1_589_641_216,);
            expect(suiteSamples[0]?.end,).toBe(false,);
            expect((suiteSamples[0]?.stat as Record<string, number> | undefined)?.shmem,).toBe(860_160,);
            expect((suiteSamples[0]?.processes as readonly Readonly<Record<string, unknown>>[] | undefined)?.length,).toBe(1,);
            /**
             Clippy's last line, which says its cgroup went.
             */
            const clippySamples = samples.filter(function ofClippy(sample) {
              return sample.role === 'clippy';
            },);
            expect(clippySamples.at(-1)?.end,).toBe(true,);
          },
          timeout: 60_000,
        }),
        it({
          name: 'reaches its deadline and says which containers it was still watching, so a wait never runs on',
          fn: async () => {
            await using tree = await mkdtempDisposable(join(
              scratchParent,
              'gate-peaks-deadline-',
            ),);
            /**
             The run's evidence directory, whose ID file names a container whose cgroup never appears.
             */
            const evidence = join(
              tree.path,
              'evidence',
              'native-HZPg8O',
            );
            await mkdir(
              evidence,
              { recursive: true },
            );
            await writeFile(
              join(evidence, 'suite.cid',),
              `${suiteId}\n`,
            );
            /**
             The gate log naming that directory.
             */
            const logPath = join(
              tree.path,
              'gate.log',
            );
            await writeFile(
              logPath,
              `Native wrapper verification evidence: ${evidence}\n`,
            );
            /**
             What the recorder found when its deadline passed.
             */
            const outcome = await recordGateContainerPeaks({
              logPath,
              outPath: join(
                tree.path,
                'peaks.jsonl',
              ),
              root: join(tree.path, 'cgroup',),
              userId: 1_000,
              intervalMilliseconds: fixtureIntervalMilliseconds,
              processIntervalMilliseconds: fixtureIntervalMilliseconds,
              deadlineMilliseconds: fixtureIntervalMilliseconds * 5,
            });
            expect(outcome.rolesRead,).toEqual([],);
            // A container whose ID file never appeared is one the recorder was still watching when its deadline came.
            expect(outcome.rolesStillRunning,).toEqual([
              'suite',
              'clippy',
            ],);
            expect(outcome.samples,).toBe(0,);
          },
          timeout: 60_000,
        }),
      ],
    }),
  ],
});
