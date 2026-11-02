import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mutantEnvironment,
  mutants,
  onlyEvidence,
  runFixtureCampaign,
  scoredBeforeKill,
  scoredLog,
} from './campaign-test-support.ts';
import { skipBaselineArguments } from './campaign-files.ts';
import { CargoMutantsToolingError } from './error.ts';
import { createFakePodman } from './fake-podman-fixture.ts';
import { isRecord } from './fs-guards.ts';

/**
 What the fixture's product offers as proof that its tested image already passed the whole suite.
 */
const proofText = 'gate evidence native-abc123';

/**
 Read the container creation of a fixture run.

 @param calls - recorded calls of the stand-in `podman`

 @returns the arguments of `podman create`

 @example
 ```ts
 createdOf(run.calls);
 ```
 */
function createdOf(calls: readonly (readonly string[])[]): readonly string[] {
  return calls.find(function isCreate(call) {
    return call[0] === 'create';
  }) ?? [];
}

/**
 Read the value after `--baseline` in a campaign command.

 @param command - arguments of the container creation

 @returns the strategy word, or nothing when the option is absent

 @example
 ```ts
 baselineStrategyOf(['cargo', 'mutants', '--baseline', 'run']);
 ```
 */
function baselineStrategyOf(command: readonly string[]): string | undefined {
  return command[command.indexOf('--baseline') + 1];
}

await describe({
  name: 'the baseline of a campaign',
  children: [
    describe({
      name: skipBaselineArguments.name,
      children: [
        it({
          name: 'turns the baseline run into a baseline skip and leaves every other argument alone',
          fn: async () => {
            expect(skipBaselineArguments([
              'cargo',
              'mutants',
              '--baseline',
              'run',
              '--iterate',
              '--file',
              'run',
            ])).toEqual([
              'cargo',
              'mutants',
              '--baseline',
              'skip',
              '--iterate',
              '--file',
              'run',
            ]);
          },
        }),
        it({
          name: 'refuses a command that does not run the baseline, because a skip must replace a decision the command made',
          fn: async () => {
            expect(() => skipBaselineArguments([
              'cargo',
              'mutants',
              '--baseline',
              'skip',
            ])).toThrow(CargoMutantsToolingError);
            expect(() => skipBaselineArguments(['cargo', 'mutants'])).toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
    describe({
      name: 'a campaign whose product offers no proof',
      children: [
        it({
          name: 'runs the baseline and records no proof',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The campaign run.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: mutantEnvironment,
            });
            expect(run.status).toBe(0);
            expect(baselineStrategyOf(createdOf(run.calls))).toBe('run');
            /**
             Parsed manifest of the campaign.
             */
            const manifest: unknown = JSON.parse(await readFile(
              join(
                await onlyEvidence(fake),
                'manifest.json',
              ),
              'utf8',
            ));
            expect(isRecord(manifest) && ('baselineProof' in manifest)).toBe(false);
          },
          timeout: 60_000,
        }),
      ],
    }),
    describe({
      name: 'a campaign whose product proves the tested image passed the whole suite',
      children: [
        it({
          name: 'skips the baseline in the container command and records the proof in the manifest',
          fn: async () => {
            await using fake = await createFakePodman();
            /**
             The campaign run.
             */
            const run = await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FIXTURE_BASELINE_PROOF: proofText,
              },
            });
            expect(run.status).toBe(0);
            expect(baselineStrategyOf(createdOf(run.calls))).toBe('skip');
            /**
             Parsed manifest of the campaign.
             */
            const manifest: unknown = JSON.parse(await readFile(
              join(
                await onlyEvidence(fake),
                'manifest.json',
              ),
              'utf8',
            ));
            expect(manifest).toMatchObject({ baselineProof: proofText });
            expect(run.stdout).toContain(proofText);
          },
          timeout: 60_000,
        }),
        it({
          name: 'finds the same campaign directory and rescores nothing when the proof appears only on the rerun',
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
             Evidence directory of the run that had no proof.
             */
            const evidence = await onlyEvidence(fake);
            /**
             The rerun, whose product now has the proof.
             */
            const rerun = await runFixtureCampaign({
              fake,
              environment: {
                ...mutantEnvironment,
                FIXTURE_BASELINE_PROOF: proofText,
              },
            });
            expect(rerun.status).toBe(0);
            expect(baselineStrategyOf(createdOf(rerun.calls))).toBe('skip');
            expect(await onlyEvidence(fake)).toBe(evidence);
            expect(await scoredLog(fake)).toEqual(mutants.map(function startOf(
              mutant,
              index
            ) {
              return `${index < scoredBeforeKill ? 1 : 2}:${mutant}`;
            }));
          },
          timeout: 60_000,
        }),
      ],
    }),
  ],
});
