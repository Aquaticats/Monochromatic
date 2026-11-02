import { readFile } from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  exitRecordOf,
  fixtureImageUser,
  mutantEnvironment,
  mutants,
  onlyEvidence,
  reportLines,
  runFixtureCampaign,
  scoredBeforeKill,
  scoredLog,
  campaignScriptConcurrency,
} from './campaign-test-support.ts';
import { missedExitStatus } from './constants.ts';
import { keepIdArguments } from './container-user.ts';
import { createFakePodman } from './fake-podman-fixture.ts';

await describe({
  name: 'a resumable campaign',
  concurrency: campaignScriptConcurrency,
  children: [
    describe({
      name: 'a fresh campaign',
      children: [
        it({
          name: 'scores every mutant once in an attempt directory bind-mounted into the container and passes --iterate',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The campaign run.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FAKE_IMAGE_USER: fixtureImageUser,
              },
            });
            expect(run.status).toBe(0);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect(await scoredLog(fake)).toEqual(mutants.map(function firstStart(mutant) {
              return `1:${mutant}`;
            }));
            expect(await reportLines({
              evidence,
              name: 'caught.txt',
            })).toEqual([...mutants]);
            /**
             Arguments of the container creation.
             */
            const created = run.calls.find(function isCreate(call) {
              return call[0] === 'create';
            }) ?? [];
            expect(created).toContain('--iterate');
            expect(created).toContain(keepIdArguments(fixtureImageUser).join(''));
            expect(created.some(function isVolume(argument) {
              return argument.endsWith(`${basename(evidence)}/attempt-001:/report:z`);
            })).toBe(true);
            expect(await exitRecordOf(evidence)).toMatchObject({
              status: 0,
              attempt: 1,
              skippedMutants: 0,
            });
          },
          timeout: 60_000,
        }),
        it({
          name: 'puts the missed mutant in the report and fails the campaign with the missed-mutants status',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             Mutant that no test catches.
             */
            const missed = mutants.at(2) ?? '';
            /**
             The campaign run.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FAKE_MISSED: missed,
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
            })).toEqual([missed]);
            expect(await exitRecordOf(evidence)).toMatchObject({ status: missedExitStatus });
          },
          timeout: 60_000,
        }),
      ],
    }),
    describe({
      name: 'a campaign killed partway and run again',
      children: [
        it({
          name: 'scores each mutant exactly once across the kill and the resume, and the report holds all of them',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The killed run.
             */
            const killed = await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FAKE_KILL_AFTER: String(scoredBeforeKill),
              },
            });
            expect(killed.status).not.toBe(0);
            /**
             Evidence directory, the same one the resume finds.
             */
            const evidence = await onlyEvidence(fake);
            expect(await reportLines({
              evidence,
              name: 'caught.txt',
            })).toEqual(mutants.slice(
              0,
              scoredBeforeKill,
            ));
            /**
             The resumed run.
             */
            const resumed = await runFixtureCampaign({
              fake,
              environment: mutantEnvironment,
            });
            expect(resumed.status).toBe(0);
            expect(await onlyEvidence(fake)).toBe(evidence);
            expect(await scoredLog(fake)).toEqual(mutants.map(function startOf(
              mutant,
              index
            ) {
              return `${index < scoredBeforeKill ? 1 : 2}:${mutant}`;
            }));
            expect(await reportLines({
              evidence,
              name: 'caught.txt',
            })).toEqual([...mutants]);
            expect(await exitRecordOf(evidence)).toMatchObject({
              status: 0,
              attempt: 2,
              skippedMutants: scoredBeforeKill,
            });
          },
          timeout: 60_000,
        }),
        it({
          name: 'does not rescore a mutant that was missed before the kill, and fails with the missed status',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             Mutant scored, and missed, before the kill.
             */
            const [missedBeforeKill] = mutants;
            await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FAKE_MISSED: missedBeforeKill,
                FAKE_KILL_AFTER: String(scoredBeforeKill),
              },
            });
            /**
             The resumed run, whose own attempt scores only caught mutants and so exits 0.
             */
            const resumed = await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FAKE_MISSED: missedBeforeKill,
              },
            });
            expect(resumed.status).not.toBe(0);
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect((await scoredLog(fake)).filter(function isMissed(line) {
              return line.endsWith(`:${missedBeforeKill}`);
            })).toEqual([`1:${missedBeforeKill}`]);
            expect(await reportLines({
              evidence,
              name: 'missed.txt',
            })).toEqual([missedBeforeKill]);
            expect(await exitRecordOf(evidence)).toMatchObject({ status: missedExitStatus });
          },
          timeout: 60_000,
        }),
        it({
          name: 'takes over the lock a killed run left, and removes an orphaned container of the same campaign first',
          fn: async () => {
            await using fake = await createFakePodman();
            await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FAKE_KILL_AFTER: String(scoredBeforeKill),
              },
            });
            /**
             The resumed run.
             */
            const resumed = await runFixtureCampaign({
              fake,
              environment: mutantEnvironment,
            });
            /**
             Removal calls in order, each followed by the container creation they precede.
             */
            const removals = resumed.calls.filter(function isForcedRemoval(call) {
              return (call[0] === 'rm') && call.includes('--ignore');
            });
            expect(removals).toHaveLength(2);
            expect(new Set(removals.map(function nameOf(call) {
              return call.at(-1);
            }))
              .size).toBe(1);
          },
          timeout: 60_000,
        }),
      ],
    }),
    describe({
      name: 'a finished campaign run again',
      children: [
        it({
          name: 'scores no mutant again and keeps the earlier result in the report and the status',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             Mutant that no test catches.
             */
            const [, missed] = mutants;
            /**
             Environment of both runs.
             */
            const environment = {
              ...mutantEnvironment,
              FAKE_MISSED: missed,
            };
            await runFixtureCampaign({
              fake,
              environment,
            });
            /**
             Mutants scored by the first run.
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
            /**
             Evidence directory.
             */
            const evidence = await onlyEvidence(fake);
            expect(await reportLines({
              evidence,
              name: 'caught.txt',
            })).toEqual(mutants.filter(function isCaught(mutant) {
              return mutant !== missed;
            }));
            expect(await reportLines({
              evidence,
              name: 'missed.txt',
            })).toEqual([missed]);
            expect(await exitRecordOf(evidence)).toMatchObject({
              status: missedExitStatus,
              attempt: 2,
              skippedMutants: mutants.length,
            });
          },
          timeout: 60_000,
        }),
        it({
          name: 'counts every attempt in the composed outcomes.json',
          fn: async () => {
            await using fake = await createFakePodman();
            await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FAKE_KILL_AFTER: String(scoredBeforeKill),
              },
            });
            await runFixtureCampaign({
              fake,
              environment: mutantEnvironment,
            });
            expect(
              JSON.parse(await readFile(
                join(
                  await onlyEvidence(fake),
                  'mutants.out',
                  'outcomes.json',
                ),
                'utf8',
              )),
            ).toMatchObject({
              total_mutants: mutants.length,
              caught: mutants.length,
              missed: 0,
            });
          },
          timeout: 60_000,
        }),
      ],
    }),
  ],
});
