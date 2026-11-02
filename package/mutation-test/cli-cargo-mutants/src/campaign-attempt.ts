import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';

import {
  campaignKey,
  createAttemptDirectory,
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
};

/**
 Find the campaign's evidence directory, take the right to run it and open the next attempt.

 A first attempt has no scored mutants and an empty report directory. A later attempt, after a kill, a reboot or a
 finished run, finds the earlier attempts' lists and seeds the names of every scored mutant, whatever its outcome,
 where cargo-mutants `--iterate` reads them.

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
  /**
   Names of the mutants earlier attempts scored.
   */
  const scoredBefore = await readScoredNames(evidence);
  /**
   Report directory of this attempt.
   */
  const attempt = await createAttemptDirectory(evidence);
  if (scoredCount(scoredBefore) > 0) {
    await seedScoredNames({
      attemptDirectory: attempt.path,
      names: scoredBefore,
    });
    console.log(`Resuming: attempt ${attempt.number} leaves out ${scoredCount(scoredBefore)} mutants that earlier attempts scored.`);
  }
  return {
    key,
    evidence,
    number: attempt.number,
    reportDirectory: attempt.path,
    scoredBefore,
    [Symbol.asyncDispose]: lock[Symbol.asyncDispose]
      .bind(lock),
  };
}
