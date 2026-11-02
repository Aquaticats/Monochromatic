import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  openAttempt,
  openCampaign,
} from './campaign-attempt.ts';
import { executePass } from './campaign-container.ts';
import { type CampaignDefinition } from './campaign-definition.ts';
import {
  campaignFailureMessage,
  evidenceJson,
  exitRecord,
  skipBaselineArguments,
} from './campaign-files.ts';
import {
  readScoredNames,
  scoredCount,
  type ScoredNames,
} from './campaign-ledger.ts';
import {
  composeReport,
  cumulativeStatus,
  plannedMutantCount,
} from './campaign-report.ts';
import {
  missedExitStatus,
  timeoutExitStatus,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import { isRecord } from './fs-guards.ts';

/**
 Directory below the campaign's evidence directory that holds pass 1.
 */
export const passOneDirectoryName = 'pass-1';

/**
 Exit statuses with which cargo-mutants finishes scoring every mutant it was given: all caught, some missed, some timed out.
 */
const finishedStatuses: readonly number[] = [
  0,
  missedExitStatus,
  timeoutExitStatus,
];

/**
 Status recorded for a pass that exited as if it had finished but did not score every mutant it listed.
 */
const unfinishedPassStatus = 1;

/**
 What one pass leaves for the next.
 */
type PassOutcome = {
  /**
   Names the pass has scored, over all its attempts.
   */
  readonly names: ScoredNames;
  /**
   Number of the attempt that ran in this invocation, absent when the pass was already finished.
   */
  readonly attempt?: number;
  /**
   Why the pass cannot lead to the next, absent when it finished.
   */
  readonly failure?: {
    readonly status: number;
    readonly signal: unknown;
  };
};

/**
 Check whether pass 1 finished in an earlier invocation.

 It finished when its last `exit.json` records a finishing status and its attempts scored every mutant the first attempt
 listed. A pass cut off by a kill, a reboot or a stop has no such record or fewer scored mutants, and runs again.

 @param passOneEvidence - pass 1's evidence directory

 @returns true when running pass 1 again would score nothing

 @example
 ```ts
 await passOneFinished('/e/native-mutation-0123456789ab/pass-1');
 ```
 */
async function passOneFinished(passOneEvidence: string): Promise<boolean> {
  /**
   The status pass 1 last recorded, or nothing.
   */
  let status: unknown;
  try {
    /**
     The parsed record.
     */
    const parsed: unknown = JSON.parse(await readFile(
      join(
        passOneEvidence,
        'exit.json',
      ),
      'utf8',
    ));
    status = isRecord(parsed) ? parsed.status : undefined;
  }
  catch (error) {
    // No record yet, or a kill tore it: pass 1 did not finish.
    console.error(`No complete exit.json in ${passOneEvidence}: ${String(error)}`);
    return false;
  }
  /**
   Mutants the first attempt listed.
   */
  const planned = await plannedMutantCount(passOneEvidence);
  return finishedStatuses.includes(Number(status))
    && (planned !== undefined)
    && (scoredCount(await readScoredNames(passOneEvidence)) >= planned);
}

/**
 Run pass 1 unless an earlier invocation finished it.

 @param definition - the product's names, limits, command and hooks

 @param contextPath - throwaway build context directory

 @param baseImage - full content-addressed ID of the tested image

 @param containerCommand - the pass-1 command the container runs, with its baseline setting applied

 @param baselineProof - the product's proof that the image passed the whole suite, when the baseline is skipped

 @param key - the campaign's key, which names pass 1's container

 @param passOneEvidence - pass 1's evidence directory

 @returns the names pass 1 scored and, when it cannot lead to pass 2, why

 @example
 ```ts
 await runPassOne({ definition, contextPath, baseImage, containerCommand, baselineProof, key, passOneEvidence });
 ```
 */
async function runPassOne({
  definition,
  contextPath,
  baseImage,
  containerCommand,
  baselineProof,
  key,
  passOneEvidence,
}: {
  readonly definition: CampaignDefinition;
  readonly contextPath: string;
  readonly baseImage: string;
  readonly containerCommand: readonly string[];
  readonly baselineProof: string | undefined;
  readonly key: string;
  readonly passOneEvidence: string;
}): Promise<PassOutcome> {
  if (await passOneFinished(passOneEvidence)) {
    console.log('Pass 1: finished by an earlier run; it scores nothing again.');
    return { names: await readScoredNames(passOneEvidence) };
  }
  /**
   The attempt of pass 1 that this invocation runs.
   */
  const attempt = await openAttempt({ evidence: passOneEvidence });
  /**
   How the container ended.
   */
  const result = await executePass({
    definition,
    contextPath,
    baseImage,
    baselineProof,
    command: containerCommand,
    attempt: {
      evidence: passOneEvidence,
      reportDirectory: attempt.reportDirectory,
      seededMutants: attempt.seededMutants,
    },
    containerName: `cargo-mutants-${key}-${passOneDirectoryName}`,
    pass: 1,
  });
  // Preserve evidence even when missed mutants correctly make the pass nonzero.
  /**
   Names pass 1 has scored, composed from all its attempts.
   */
  const names = await composeReport(passOneEvidence);
  /**
   Planned mutant count of the composed report.
   */
  const planned = await plannedMutantCount(passOneEvidence);
  /**
   Exit status of pass 1 as a whole.
   */
  const status = cumulativeStatus({
    attemptStatus: result.status,
    names,
  });
  /**
   Whether pass 1 scored every mutant it listed.
   */
  const finished = finishedStatuses.includes(status)
    && (planned !== undefined)
    && (scoredCount(names) >= planned);
  /**
   Status to record: the finishing one, or the failing one.
   */
  const recorded = (finished || (!finishedStatuses.includes(status))) ? status : unfinishedPassStatus;
  await writeFile(
    join(
      passOneEvidence,
      'exit.json',
    ),
    evidenceJson(exitRecord({
      status: recorded,
      signal: result.signal === '' ? undefined : result.signal,
      reportCopied: true,
      attempt: attempt.number,
      skippedMutants: attempt.seededMutants,
    })),
  );
  return {
    names,
    attempt: attempt.number,
    ...(finished ? {} : {
      failure: {
        status: recorded,
        signal: result.signal,
      },
    }),
  };
}

/**
 Run a campaign in two passes and keep one report.

 Pass 1 runs `definition.passOne.command` in the `pass-1` directory of the campaign's evidence directory, as an ordinary
 resumable campaign of its own. When it has scored every mutant, pass 2 runs `definition.command` in the evidence
 directory itself, handed pass 1's caught and unviable mutants as names to leave out (the same `previously_caught.txt`
 seed a resume uses), so it scores exactly the mutants pass 1 missed or timed out, once each. The report in
 `mutants.out` is pass 1's with every mutant pass 2 scored replaced by pass 2's result, and the exit status follows that
 report.

 The campaign's key hashes the image and `definition.command` only, so it is the key a one-pass campaign of the same
 files has, and a different test selection in pass 1 (a changed source tree, say) does not start the campaign over: a
 mutant caught by any subset of the tests is caught.

 @param definition - the product's names, limits, command and hooks; `passOne` must be present

 @param contextPath - throwaway build context directory

 @param baseImage - full content-addressed ID of the tested image

 @param baselineProof - the product's proof that the image passed the whole suite, or nothing

 @throws CargoMutantsToolingError when a pass ends without scoring every mutant it listed, or the campaign ends with
 missed or timed-out mutants

 @example
 ```ts
 await runTwoPasses({ definition, contextPath, baseImage, baselineProof: undefined });
 ```
 */
export async function runTwoPasses({
  definition,
  contextPath,
  baseImage,
  baselineProof,
}: {
  readonly definition: CampaignDefinition;
  readonly contextPath: string;
  readonly baseImage: string;
  readonly baselineProof: string | undefined;
}): Promise<void> {
  if (definition.passOne === undefined)
    throw new CargoMutantsToolingError('runTwoPasses needs a pass-1 command in the campaign definition.');
  /**
   Applies the baseline decision to a pass's command.

   @param command - the pass's command with its baseline run

   @returns the command the container runs
   */
  function withBaseline(command: readonly string[]): readonly string[] {
    return baselineProof === undefined ? command : skipBaselineArguments(command);
  }
  /**
   The campaign's evidence directory and the right to run it, held through both passes.
   */
  await using campaign = await openCampaign({
    evidenceRoot: definition.evidenceRoot,
    evidencePrefix: definition.evidencePrefix,
    baseImage,
    command: definition.command,
  });
  /**
   Pass 1's evidence directory.
   */
  const passOneEvidence = join(
    campaign.evidence,
    passOneDirectoryName,
  );
  await mkdir(
    passOneEvidence,
    { recursive: true },
  );
  /**
   What pass 1 left.
   */
  const one = await runPassOne({
    definition,
    contextPath,
    baseImage,
    containerCommand: withBaseline(definition.passOne.command),
    baselineProof,
    key: campaign.key,
    passOneEvidence,
  });
  /**
   Mutants pass 1 missed or timed out, which pass 2 scores.
   */
  const survivors = [...one.names.missed, ...one.names.timeout];
  /**
   Mutants pass 2 scored in earlier invocations.
   */
  const twoBefore = await readScoredNames(campaign.evidence);
  /**
   Survivors no pass-2 attempt has scored yet.
   */
  const unscored = survivors.filter(function isUnscored(name) {
    return ![
      ...twoBefore.caught,
      ...twoBefore.missed,
      ...twoBefore.timeout,
      ...twoBefore.unviable
    ].includes(name);
  });
  console.log(`Pass 1: ${scoredCount(one.names)} mutants scored, ${survivors.length} missed or timed out; pass 2 scores ${one.failure === undefined ? unscored.length : 0} of them with the whole suite.`);
  /**
   How pass 2 ended, absent when it did not run.
   */
  let two: Awaited<ReturnType<typeof executePass>> | undefined;
  /**
   Number and seed size of the pass-2 attempt, absent when it did not run.
   */
  let attemptTwo: { readonly number: number; readonly seededMutants: number } | undefined;
  if ((one.failure === undefined) && (unscored.length > 0)) {
    /**
     The pass-2 attempt, which leaves out what pass 1 caught or found unviable.
     */
    const attempt = await openAttempt({
      evidence: campaign.evidence,
      inherited: {
        caught: one.names.caught,
        missed: [],
        timeout: [],
        unviable: one.names.unviable,
      },
    });
    attemptTwo = attempt;
    two = await executePass({
      definition,
      contextPath,
      baseImage,
      baselineProof,
      command: withBaseline(definition.command),
      attempt: {
        evidence: campaign.evidence,
        reportDirectory: attempt.reportDirectory,
        seededMutants: attempt.seededMutants,
      },
      containerName: `cargo-mutants-${campaign.key}`,
      pass: 2,
    });
  }
  // Preserve evidence even when missed mutants correctly make the campaign nonzero.
  /**
   Names of the whole campaign: pass 1's, each replaced by pass 2's where pass 2 scored the mutant.
   */
  const names = await composeReport(
    campaign.evidence,
    passOneEvidence,
  );
  /**
   Exit status of the whole campaign.
   */
  const status = one.failure === undefined
    ? cumulativeStatus({
      attemptStatus: two?.status ?? 0,
      names,
    })
    : one.failure.status;
  await writeFile(
    join(
      campaign.evidence,
      'exit.json',
    ),
    evidenceJson(exitRecord({
      status,
      signal: (two?.signal === undefined) || (two.signal === '') ? undefined : two.signal,
      reportCopied: true,
      ...(attemptTwo === undefined ? {} : {
        attempt: attemptTwo.number,
        skippedMutants: attemptTwo.seededMutants,
      }),
      passes: {
        passOne: {
          finished: one.failure === undefined,
          attempt: one.attempt ?? null,
          scored: scoredCount(one.names),
          survivors: survivors.length,
        },
        passTwo: {
          ran: two !== undefined,
          unscoredSurvivors: one.failure === undefined ? unscored.length : null,
        },
      },
    })),
  );
  if (status !== 0) {
    throw new CargoMutantsToolingError(one.failure === undefined
      ? campaignFailureMessage({
        status,
        reportCopied: true,
        evidence: campaign.evidence,
      })
      : `Pass 1 of the mutation campaign did not finish (status ${status}); ${campaignFailureMessage({
        status,
        reportCopied: true,
        evidence: passOneEvidence,
      })}`);
  }
  await definition.afterReport?.(campaign.evidence);
}
