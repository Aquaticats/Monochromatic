import {
  readFile,
  readdir,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  exitRecordOf,
  mutantEnvironment,
  mutants,
  onlyEvidence,
  reportLines,
  runFixtureCampaign,
  scoredLog,
  campaignScriptConcurrency,
} from './campaign-test-support.ts';
import { selectTestsArguments } from './campaign-files.ts';
import { overlayScoredNames } from './campaign-report.ts';
import { missedExitStatus } from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import { createFakePodman } from './fake-podman-fixture.ts';
import { isRecord } from './fs-guards.ts';

/**
 Environment that makes the fixture campaign a two-pass campaign over the five stand-in mutants.
 */
const twoPassEnvironment = {
  ...mutantEnvironment,
  FIXTURE_PASS_ONE: '1',
  FIXTURE_BASELINE_PROOF: 'gate evidence',
};

/**
 Mutants named by index, so a test reads as "pass 1 misses the second and fourth".
 */
const [first, second, third, fourth, fifth] = mutants;

/**
 Pick the container creations out of the recorded calls.

 @param calls - recorded calls of the stand-in `podman`, over every run of the fixture

 @returns the arguments of each `podman create`, in order

 @example
 ```ts
 createdContainers(run.calls);
 ```
 */
function createdContainers(calls: readonly (readonly string[])[]): readonly (readonly string[])[] {
  return calls.filter(function isCreate(call) {
    return call[0] === 'create';
  });
}

/**
 Check whether a container creation is pass 1 of a two-pass campaign, which selects tests by filterset.

 @param created - arguments of a `podman create`

 @returns true when the command carries the test selection

 @example
 ```ts
 isPassOne(createdContainers(run.calls)[0] ?? []);
 ```
 */
function isPassOne(created: readonly string[]): boolean {
  return created.some(function selectsTests(argument) {
    return argument.startsWith('--cargo-test-arg=--filterset=');
  });
}

/**
 Read a JSON object file of the evidence directory.

 @param file - path of the file

 @returns the parsed object

 @example
 ```ts
 await readRecord(join(evidence, 'mutants.out', 'outcomes.json'));
 ```
 */
async function readRecord(file: string): Promise<Readonly<Record<string, unknown>>> {
  /**
   Parsed file.
   */
  const parsed: unknown = JSON.parse(await readFile(
    file,
    'utf8',
  ));
  return isRecord(parsed) ? parsed : {};
}

/**
 Read the outcome entries of the composed report as `summary` per mutant name.

 @param evidence - the campaign's evidence directory

 @returns pairs of mutant name and summary, in report order

 @example
 ```ts
 await entrySummaries(evidence);
 ```
 */
async function entrySummaries(evidence: string): Promise<readonly (readonly [string, string])[]> {
  /**
   The entries of the composed outcomes.json.
   */
  const entries = (await readRecord(join(
    evidence,
    'mutants.out',
    'outcomes.json',
  ))).outcomes;
  return Array.isArray(entries)
    ? entries.map(function summaryOf(entry: unknown) {
      if ((!isRecord(entry)) || (!isRecord(entry.scenario)) || (!isRecord(entry.scenario.Mutant)))
        return ['?', '?'] as const;
      return [String(entry.scenario.Mutant.name), String(entry.summary)] as const;
    })
    : [];
}

await describe({
  name: 'a two-pass campaign',
  concurrency: campaignScriptConcurrency,
  children: [
    describe({
      name: selectTestsArguments.name,
      children: [
        it({
          name: 'adds the filterset and the no-tests option as single arguments before the first bare --',
          fn: async () => {
            expect(selectTestsArguments({
              command: [
                'cargo',
                'mutants',
                '--file',
                'a.rs',
                '--',
                '--',
                '--skip',
                'x',
              ],
              filterset: 'kind(lib) & test(/^(?:a|b)::/)',
            })).toEqual([
              'cargo',
              'mutants',
              '--file',
              'a.rs',
              '--cargo-test-arg=--filterset=kind(lib) & test(/^(?:a|b)::/)',
              '--cargo-test-arg=--no-tests=pass',
              '--',
              '--',
              '--skip',
              'x',
            ]);
          },
        }),
        it({
          name: 'adds them at the end of a command without a bare --, and refuses an empty expression',
          fn: async () => {
            expect(selectTestsArguments({
              command: ['cargo', 'mutants'],
              filterset: 'all()',
            })).toEqual([
              'cargo',
              'mutants',
              '--cargo-test-arg=--filterset=all()',
              '--cargo-test-arg=--no-tests=pass',
            ]);
            expect(() => selectTestsArguments({
              command: ['cargo'],
              filterset: ' ',
            })).toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
    describe({
      name: overlayScoredNames.name,
      children: [
        it({
          name: 'lets the later pass replace each mutant it scored and keeps the earlier outcome of the rest',
          fn: async () => {
            expect(overlayScoredNames({
              earlier: {
                caught: ['c1'],
                missed: ['m1', 'm2'],
                timeout: ['t1'],
                unviable: ['u1'],
              },
              later: {
                caught: ['m1', 't1'],
                missed: [],
                timeout: [],
                unviable: [],
              },
            })).toEqual({
              caught: ['c1', 'm1', 't1'],
              missed: ['m2'],
              timeout: [],
              unviable: ['u1'],
            });
          },
        }),
      ],
    }),
    describe({
      name: 'a campaign that runs to the end',
      children: [
        it({
          name: 'scores every mutant once in pass 1, scores only the pass 1 survivors in pass 2, and reports one campaign',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The campaign run.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: {
                ...twoPassEnvironment,
                FAKE_PASS_ONE_MISSED: [second, fourth].join(','),
              },
            });
            expect(run.status).toBe(0);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect(await scoredLog(fake)).toEqual([
              ...mutants.map(function inPassOne(mutant) {
                return `1:${mutant}`;
              }),
              `2:${second}`,
              `2:${fourth}`,
            ]);
            /**
             The container creations, pass 1 first.
             */
            const created = createdContainers(run.calls);
            expect(created.map(isPassOne)).toEqual([true, false]);
            expect((await reportLines({
              evidence,
              name: 'caught.txt',
            })).toSorted()).toEqual(mutants.toSorted());
            expect(await reportLines({
              evidence,
              name: 'missed.txt',
            })).toEqual([]);
            expect(
              await readRecord(join(
                evidence,
                'mutants.out',
                'outcomes.json',
              )),
            ).toMatchObject({
              total_mutants: mutants.length,
              caught: mutants.length,
              missed: 0,
            });
            expect(await entrySummaries(evidence)).toHaveLength(mutants.length);
            expect((await entrySummaries(evidence)).filter(function isSecond([name]) {
              return name === second;
            })).toEqual([[second, 'caught']]);
            expect(await exitRecordOf(evidence)).toMatchObject({
              status: 0,
              attempt: 1,
            });
            expect((await exitRecordOf(evidence)).passes).toEqual({
              passOne: {
                finished: true,
                attempt: 1,
                scored: mutants.length,
                survivors: 2,
              },
              passTwo: {
                ran: true,
                unscoredSurvivors: 2,
              },
            });
          },
          timeout: 60_000,
        }),
        it({
          name: 'seeds pass 2 with the mutants pass 1 caught or found unviable, and not with the ones it missed or timed out',
          fn: async () => {
            await using fake = await createFakePodman();
            await runFixtureCampaign({
              fake,
              environment: {
                ...twoPassEnvironment,
                FAKE_PASS_ONE_MISSED: second,
                FAKE_PASS_ONE_TIMEOUT: third,
                FAKE_UNVIABLE: fifth,
              },
            });
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect((await readFile(
              join(
                evidence,
                'attempt-001',
                'mutants.out',
                'previously_caught.txt',
              ),
              'utf8',
            )).split('\n')
              .filter(Boolean)
              .toSorted()).toEqual([first, fourth, fifth].toSorted());
          },
          timeout: 60_000,
        }),
        it({
          name: 'keeps the mutant pass 2 misses as missed, replaces the one it catches, and ends with the missed-mutants status',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The campaign run.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: {
                ...twoPassEnvironment,
                FAKE_MISSED: second,
                FAKE_PASS_ONE_MISSED: fourth,
              },
            });
            expect(run.status).not.toBe(0);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect(await reportLines({
              evidence,
              name: 'missed.txt',
            })).toEqual([second]);
            expect(
              await readRecord(join(
                evidence,
                'mutants.out',
                'outcomes.json',
              )),
            ).toMatchObject({
              total_mutants: mutants.length,
              caught: mutants.length - 1,
              missed: 1,
            });
            expect((await entrySummaries(evidence)).filter(function isMissed([, summary]) {
              return summary === 'missed';
            })).toEqual([[second, 'missed']]);
            expect(await exitRecordOf(evidence)).toMatchObject({ status: missedExitStatus });
          },
          timeout: 60_000,
        }),
        it({
          name: 'runs no pass 2 container when pass 1 left no survivor',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The campaign run.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: twoPassEnvironment,
            });
            expect(run.status).toBe(0);
            expect(createdContainers(run.calls)).toHaveLength(1);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect((await reportLines({
              evidence,
              name: 'caught.txt',
            })).toSorted()).toEqual(mutants.toSorted());
            expect(await exitRecordOf(evidence)).toMatchObject({ status: 0 });
            expect((await exitRecordOf(evidence)).passes).toEqual({
              passOne: {
                finished: true,
                attempt: 1,
                scored: mutants.length,
                survivors: 0,
              },
              passTwo: {
                ran: false,
                unscoredSurvivors: 0,
              },
            });
          },
          timeout: 60_000,
        }),
        it({
          name: 'skips the baseline in both passes, because the proof covers the whole suite',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The campaign run.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: {
                ...twoPassEnvironment,
                FAKE_PASS_ONE_MISSED: second,
              },
            });
            expect(createdContainers(run.calls)
              .map(function strategyOf(created) {
                return created[created.indexOf('--baseline') + 1];
              })).toEqual(['skip', 'skip']);
          },
          timeout: 60_000,
        }),
        it({
          name: 'refuses to start without proof that the whole suite passed, because a filtered baseline cannot stand for it',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The run without proof.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FIXTURE_PASS_ONE: '1',
              },
            });
            expect(run.status).not.toBe(0);
            expect(run.stderr).toContain('proof that the whole suite already passed');
            expect(createdContainers(run.calls)).toHaveLength(0);
          },
          timeout: 60_000,
        }),
        it({
          name: 'names the evidence directory by the one-pass command, so the pass-1 test selection is not part of the key',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The run, whose own report is what tells why the directory looks as it does when it does not.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: {
                ...twoPassEnvironment,
                FAKE_PASS_ONE_MISSED: second,
              },
            });
            // A script that died shows its own message here, instead of a directory that merely looks unfinished.
            expect(run.status === 0 ? '' : run.stderr).toBe('');
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect((await readdir(evidence)).toSorted()).toEqual([
              'attempt-001',
              'campaign.json',
              'campaign.owner.1',
              'container-started.json',
              'exit.json',
              'manifest.json',
              'mutants.out',
              'pass-1',
            ]);
            expect(
              await readRecord(join(
                evidence,
                'manifest.json',
              )),
            ).toMatchObject({ pass: 2 });
            expect(
              await readRecord(join(
                evidence,
                'pass-1',
                'manifest.json',
              )),
            ).toMatchObject({ pass: 1 });
          },
          timeout: 60_000,
        }),
      ],
    }),
    describe({
      name: 'a campaign run in the other mode than before',
      children: [
        it({
          name: 'refuses a one-pass run over a two-pass campaign and scores nothing',
          fn: async () => {
            await using fake = await createFakePodman();
            await runFixtureCampaign({
              fake,
              environment: {
                ...twoPassEnvironment,
                FAKE_PASS_ONE_MISSED: second,
              },
            });
            /**
             Mutants scored by the two-pass run.
             */
            const before = await scoredLog(fake);
            /**
             The one-pass run of the same campaign.
             */
            const onePass = await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FIXTURE_BASELINE_PROOF: 'gate evidence',
              },
            });
            expect(onePass.status).not.toBe(0);
            expect(onePass.stderr).toContain('two-pass');
            expect(await scoredLog(fake)).toEqual(before);
          },
          timeout: 60_000,
        }),
        it({
          name: 'refuses a two-pass run over a one-pass campaign and scores nothing',
          fn: async () => {
            await using fake = await createFakePodman();
            await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FIXTURE_BASELINE_PROOF: 'gate evidence',
              },
            });
            /**
             Mutants scored by the one-pass run.
             */
            const before = await scoredLog(fake);
            /**
             The two-pass run of the same campaign.
             */
            const twoPass = await runFixtureCampaign({
              fake,
              environment: twoPassEnvironment,
            });
            expect(twoPass.status).not.toBe(0);
            expect(twoPass.stderr).toContain('one-pass');
            expect(await scoredLog(fake)).toEqual(before);
          },
          timeout: 60_000,
        }),
        it({
          name: 'lets a two-pass campaign run again in two passes',
          fn: async () => {
            await using fake = await createFakePodman();
            await runFixtureCampaign({
              fake,
              environment: twoPassEnvironment,
            });
            expect((await runFixtureCampaign({
              fake,
              environment: twoPassEnvironment,
            })).status).toBe(0);
          },
          timeout: 60_000,
        }),
      ],
    }),
    describe({
      name: 'a campaign killed and run again',
      children: [
        it({
          name: 'resumes pass 1 after a kill in pass 1 and scores no mutant twice',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             Environment of both runs.
             */
            const environment = {
              ...twoPassEnvironment,
              FAKE_PASS_ONE_MISSED: [second, fourth].join(','),
            };
            /**
             The run killed during pass 1.
             */
            const killed = await runFixtureCampaign({
              fake,
              environment: {
                ...environment,
                FAKE_KILL_AFTER: '2',
              },
            });
            expect(killed.status).not.toBe(0);
            expect(createdContainers(killed.calls)).toHaveLength(1);
            /**
             The resumed run.
             */
            const resumed = await runFixtureCampaign({
              fake,
              environment,
            });
            expect(resumed.status).toBe(0);
            expect(await scoredLog(fake)).toEqual([
              `1:${first}`,
              `1:${second}`,
              `2:${third}`,
              `2:${fourth}`,
              `2:${fifth}`,
              `3:${second}`,
              `3:${fourth}`,
            ]);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect((await reportLines({
              evidence,
              name: 'caught.txt',
            })).toSorted()).toEqual(mutants.toSorted());
          },
          timeout: 60_000,
        }),
        it({
          name: 'resumes pass 2 after a kill in pass 2 without running pass 1 again and scores no mutant twice',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             Environment of both runs.
             */
            const environment = {
              ...twoPassEnvironment,
              FAKE_PASS_ONE_MISSED: [second, fourth].join(','),
            };
            /**
             The run killed during pass 2, after it scored one survivor.
             */
            const killed = await runFixtureCampaign({
              fake,
              environment: {
                ...environment,
                FAKE_KILL_AFTER: '1',
                FAKE_KILL_START: '2',
              },
            });
            expect(killed.status).not.toBe(0);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect(await reportLines({
              evidence,
              name: 'missed.txt',
            })).toEqual([fourth]);
            /**
             The resumed run.
             */
            const resumed = await runFixtureCampaign({
              fake,
              environment,
            });
            expect(resumed.status).toBe(0);
            expect(createdContainers(resumed.calls)
              .map(isPassOne)).toEqual([true, false, false]);
            expect(await scoredLog(fake)).toEqual([
              ...mutants.map(function inPassOne(mutant) {
                return `1:${mutant}`;
              }),
              `2:${second}`,
              `3:${fourth}`,
            ]);
            expect(await reportLines({
              evidence,
              name: 'missed.txt',
            })).toEqual([]);
            expect(await exitRecordOf(evidence)).toMatchObject({
              status: 0,
              attempt: 2,
            });
          },
          timeout: 60_000,
        }),
        it({
          name: 'runs no container and scores nothing when the whole campaign is run again',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             Environment of both runs.
             */
            const environment = {
              ...twoPassEnvironment,
              FAKE_MISSED: second,
              FAKE_PASS_ONE_MISSED: fourth,
            };
            /**
             The first run.
             */
            const firstRun = await runFixtureCampaign({
              fake,
              environment,
            });
            /**
             The scored mutants after the first run.
             */
            const firstLog = await scoredLog(fake);
            /**
             The rerun.
             */
            const rerun = await runFixtureCampaign({
              fake,
              environment,
            });
            expect(rerun.status).not.toBe(0);
            expect(await scoredLog(fake)).toEqual(firstLog);
            expect(createdContainers(rerun.calls)).toHaveLength(createdContainers(firstRun.calls).length);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect(await reportLines({
              evidence,
              name: 'missed.txt',
            })).toEqual([second]);
            expect(await exitRecordOf(evidence)).toMatchObject({ status: missedExitStatus });
          },
          timeout: 60_000,
        }),
        it({
          name: 'stops without a pass 2 container when pass 1 ends in a failure status',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The run whose pass 1 is killed.
             */
            const killed = await runFixtureCampaign({
              fake,
              environment: {
                ...twoPassEnvironment,
                FAKE_PASS_ONE_MISSED: second,
                FAKE_KILL_AFTER: '3',
              },
            });
            expect(killed.status).not.toBe(0);
            expect(createdContainers(killed.calls)).toHaveLength(1);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect((await exitRecordOf(evidence)).passes).toEqual({
              passOne: {
                finished: false,
                attempt: 1,
                scored: 3,
                survivors: 1,
              },
              passTwo: {
                ran: false,
                unscoredSurvivors: null,
              },
            });
          },
          timeout: 60_000,
        }),
      ],
    }),
  ],
});
