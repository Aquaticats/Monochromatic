import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { expect } from '@monochromatic-dev/test-harness-core/ts';

import { isRecord } from './fs-guards.ts';
import type { FakePodman } from './fake-podman-fixture.ts';
import { casesWithinOwnLimit } from './process-limits.ts';
import { runScript } from './run-fixture-program.ts';

/**
 Mutants the stand-in cargo-mutants scores, in scoring order.
 */
export const mutants = [
  'src/a.rs:1:1: replace a',
  'src/a.rs:2:1: replace b',
  'src/a.rs:3:1: replace c',
  'src/a.rs:4:1: replace d',
  'src/a.rs:5:1: replace e',
] as const;

/**
 Mutants scored before the kill in the kill-and-resume tests.
 */
export const scoredBeforeKill = 2;

/**
 The user the stand-in reports for the campaign image, which the container creation must map the host user onto.
 */
export const fixtureImageUser = '2000:2001';

/**
 Campaign scripts a test file starts at once when nothing limits it. Each script is a Node process that starts a Node process
 for every stand-in Podman call, and the whole unit-test run starts several test files in parallel, so a wide file
 oversubscribes a host or a capped scope: a few at once finish sooner than many that starve one another.
 */
const wantedCampaignScriptConcurrency = 4;

/**
 Campaign scripts a test file starts at once: the wanted width, narrowed so that the files the runner starts at once stay
 inside the process limit of the control group the run is in (a `TasksMax=` that counts threads as well as processes).
 */
export const campaignScriptConcurrency: number = await casesWithinOwnLimit(wantedCampaignScriptConcurrency);

/**
 Environment that gives the stand-in the mutants to score.
 */
export const mutantEnvironment: Readonly<Record<string, string>> = { FAKE_MUTANTS: mutants.join(',') };

/**
 Run the campaign script once in a fixture.

 @param fake - fixture whose working directory keeps the evidence between runs

 @param environment - stand-in settings of this run

 @returns what the script did

 @example
 ```ts
 await runFixtureCampaign({ fake, environment: mutantEnvironment });
 ```
 */
export function runFixtureCampaign({
  fake,
  environment,
}: {
  readonly fake: FakePodman;
  readonly environment: Readonly<Record<string, string>>;
}): ReturnType<typeof runScript> {
  return runScript({
    fake,
    script: join(
      import.meta.dirname,
      'campaign-fixture.ts',
    ),
    args: [],
    extraEnvironment: environment,
  });
}

/**
 Find the only evidence directory the campaigns of a fixture made.

 @param fake - fixture

 @returns its absolute path

 @example
 ```ts
 await onlyEvidence(fake);
 ```
 */
export async function onlyEvidence(fake: FakePodman): Promise<string> {
  /**
   Directory that holds every campaign's evidence.
   */
  const root = join(
    fake.workingDirectory,
    'target',
    'verification',
  );
  /**
   Names of the campaign directories.
   */
  const names = await readdir(root);
  expect(names)
    .toHaveLength(1);
  return join(
    root,
    names[0] ?? '',
  );
}

/**
 Read the lines of a file of the composed report.

 @param evidence - evidence directory

 @param name - file below `mutants.out`

 @returns its lines

 @example
 ```ts
 await reportLines({ evidence, name: 'caught.txt' });
 ```
 */
export async function reportLines({
  evidence,
  name,
}: {
  readonly evidence: string;
  readonly name: string;
}): Promise<readonly string[]> {
  return (await readFile(
    join(
      evidence,
      'mutants.out',
      name,
    ),
    'utf8',
  )).split('\n')
    .filter(function isLine(line) {
      return line !== '';
    });
}

/**
 Read the record of how the last attempt ended.

 @param evidence - evidence directory

 @returns the parsed `exit.json`

 @example
 ```ts
 await exitRecordOf(evidence);
 ```
 */
export async function exitRecordOf(evidence: string): Promise<Readonly<Record<string, unknown>>> {
  /**
   Parsed file.
   */
  const parsed: unknown = JSON.parse(await readFile(
    join(
      evidence,
      'exit.json',
    ),
    'utf8',
  ));
  return isRecord(parsed) ? parsed : {};
}

/**
 Read the mutants the stand-in scored, in order, each with the start that scored it.

 @param fake - fixture

 @returns lines `<start>:<mutant>`

 @example
 ```ts
 await scoredLog(fake);
 ```
 */
export async function scoredLog(fake: FakePodman): Promise<readonly string[]> {
  return (await readFile(
    join(
      fake.directory,
      'scored.log',
    ),
    'utf8',
  )).split('\n')
    .filter(function isLine(line) {
      return line !== '';
    });
}
