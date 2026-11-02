import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';

import {
  campaignKey,
  createAttemptDirectory,
  mergeScoredNames,
  readScoredNames,
  recordCampaignIdentity,
  scoredCount,
  seedScoredNames,
  type ScoredNames,
} from './campaign-ledger.ts';
import { acquireCampaignLock } from './campaign-lock.ts';

/**
 What a campaign needs to know before its container starts.
 */
export type PreparedAttempt = AsyncDisposable & {
  /**
   Key that names the campaign.
   */
  readonly key: string;
  /**
   The campaign's evidence directory, the same for every attempt.
   */
  readonly evidence: string;
  /**
   Number of this attempt, counted from 1.
   */
  readonly number: number;
  /**
   Report directory of this attempt, which the container writes through a bind mount.
   */
  readonly reportDirectory: string;
  /**
   Mutants that earlier attempts scored, which this attempt leaves out.
   */
  readonly scoredBefore: ScoredNames;
  /**
   Mutants this attempt leaves out: those earlier attempts scored and, in pass 2 of a two-pass campaign, those pass 1
   caught or found unviable.
   */
  readonly seededMutants: number;
};

/**
 A campaign's evidence directory and the right to run it.
 */
export type OpenedCampaign = AsyncDisposable & {
  readonly key: string;
  readonly evidence: string;
};

/**
 Find the campaign's evidence directory and take the right to run it.

 @param evidenceRoot - directory that keeps the evidence of every campaign

 @param evidencePrefix - name prefix of this product's evidence directories

 @param baseImage - full content-addressed ID of the tested image

 @param command - the complete campaign command

 @returns the directory and its key; declare it with `await using` so the campaign lock is released when it ends

 @throws CargoMutantsToolingError when another run of the same campaign is alive or the directory belongs to another
 campaign

 @example
 ```ts
 await using opened = await openCampaign({ evidenceRoot, evidencePrefix: 'mutation-', baseImage, command });
 ```
 */
export async function openCampaign({
  evidenceRoot,
  evidencePrefix,
  baseImage,
  command,
}: {
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly baseImage: string;
  readonly command: readonly string[];
}): Promise<OpenedCampaign> {
  /**
   Key that names this campaign: the same image and command always find the same directory.
   */
  const key = campaignKey({
    baseImage,
    command,
  });
  /**
   Evidence directory of this campaign, kept across interruptions so a rerun resumes it.
   */
  const evidence = join(
    evidenceRoot,
    `${evidencePrefix}${key}`,
  );
  await mkdir(
    evidence,
    { recursive: true },
  );
  console.log(`Mutation evidence: ${evidence}`);
  await recordCampaignIdentity({
    evidence,
    baseImage,
    command,
  });
  /**
   Right to run this campaign, held until it ends.
   */
  const lock = await acquireCampaignLock(evidence);
  return {
    key,
    evidence,
    [Symbol.asyncDispose]: lock[Symbol.asyncDispose]
      .bind(lock),
  };
}

/**
 Open the next attempt of a campaign directory whose right to run is already held.

 A first attempt has no scored mutants and an empty report directory. A later attempt, after a kill, a reboot or a
 finished run, finds the earlier attempts' lists and seeds the names of every scored mutant, whatever its outcome,
 where cargo-mutants `--iterate` reads them. Names a caller inherits from another pass are seeded too.

 @param evidence - the campaign's evidence directory

 @param inherited - names scored by another pass that this attempt must leave out as well, absent for none

 @returns the attempt's number, report directory, and what it leaves out

 @example
 ```ts
 const attempt = await openAttempt({ evidence: '/e/native-mutation-0123456789ab' });
 ```
 */
export async function openAttempt({
  evidence,
  inherited,
}: {
  readonly evidence: string;
  readonly inherited?: ScoredNames;
}): Promise<Omit<PreparedAttempt, 'evidence' | 'key' | typeof Symbol.asyncDispose>> {
  /**
   Names of the mutants earlier attempts scored.
   */
  const scoredBefore = await readScoredNames(evidence);
  /**
   Names the new attempt leaves out.
   */
  const seeded = mergeScoredNames([
    scoredBefore,
    ...(inherited === undefined ? [] : [inherited])
  ]);
  /**
   Report directory of this attempt.
   */
  const attempt = await createAttemptDirectory(evidence);
  if (scoredCount(seeded) > 0) {
    await seedScoredNames({
      attemptDirectory: attempt.path,
      names: seeded,
    });
    console.log(`Resuming: attempt ${attempt.number} leaves out ${scoredCount(seeded)} mutants that earlier attempts${inherited === undefined ? '' : ' or pass 1'} scored.`);
  }
  return {
    number: attempt.number,
    reportDirectory: attempt.path,
    scoredBefore,
    seededMutants: scoredCount(seeded),
  };
}

/**
 Find the campaign's evidence directory, take the right to run it and open the next attempt.

 @param evidenceRoot - directory that keeps the evidence of every campaign

 @param evidencePrefix - name prefix of this product's evidence directories

 @param baseImage - full content-addressed ID of the tested image

 @param command - the complete campaign command

 @returns the prepared attempt; declare it with `await using` so the campaign lock is released when the campaign ends

 @throws CargoMutantsToolingError when another run of the same campaign is alive or the directory belongs to another
 campaign

 @example
 ```ts
 await using prepared = await prepareAttempt({ evidenceRoot, evidencePrefix: 'mutation-', baseImage, command });
 ```
 */
export async function prepareAttempt({
  evidenceRoot,
  evidencePrefix,
  baseImage,
  command,
}: {
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly baseImage: string;
  readonly command: readonly string[];
}): Promise<PreparedAttempt> {
  /**
   The campaign directory and its lock.
   */
  const opened = await openCampaign({
    evidenceRoot,
    evidencePrefix,
    baseImage,
    command,
  });
  try {
    return {
      key: opened.key,
      evidence: opened.evidence,
      ...(await openAttempt({ evidence: opened.evidence })),
      [Symbol.asyncDispose]: opened[Symbol.asyncDispose]
        .bind(opened),
    };
  }
  catch (error) {
    await opened[Symbol.asyncDispose]();
    throw error;
  }
}
