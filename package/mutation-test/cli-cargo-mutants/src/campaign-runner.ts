import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { prepareAttempt } from './campaign-attempt.ts';
import { executePass } from './campaign-container.ts';
import type { CampaignDefinition } from './campaign-definition.ts';
import {
  campaignFailureMessage,
  evidenceJson,
  exitRecord,
  isContentAddressedId,
  skipBaselineArguments,
} from './campaign-files.ts';
import { scoredCount } from './campaign-ledger.ts';
import { runTwoPasses } from './campaign-passes.ts';
import {
  composeReport,
  cumulativeStatus,
} from './campaign-report.ts';
import { CargoMutantsToolingError } from './error.ts';
import { runPodman } from './podman.ts';

export { installedToolPath } from './campaign-container.ts';

/**
 Run one campaign over an immutable tested image and keep its complete report.

 The campaign image is built from a Containerfile that layers the installed cargo-mutants over the tested image.
 The container starts from the ID the build wrote, not from the shared tag, and is given its command explicitly, and
 that command is read back from the created container: two builds started together whose Containerfiles differ only in
 their default command can commit one image, so the image's own default command never decides what a campaign runs.
 The container is removed and the build context deleted when the campaign ends, whether it passed or failed.

 The campaign is resumable. Its evidence directory is named by a key of the tested image and the command, so a rerun
 finds the earlier attempts' `attempt-NNN` report directories, each bind-mounted into the container as it ran so that
 every outcome reaches the host as cargo-mutants scores it. The rerun leaves out every mutant those attempts scored
 (cargo-mutants `--iterate` reads their names from a seeded `previously_caught.txt`) and the final report
 `<evidence>/mutants.out` and the exit status cover all attempts.

 With `definition.passOne` the campaign runs in two passes (see `runTwoPasses`) and the same report and status cover both.

 @param definition - the product's names, limits, command and manifest

 @throws CargoMutantsToolingError when an image or container ID is incomplete, the created container would run
 another command, the campaign exits nonzero, or its report is not copied

 @example
 ```ts
 await runCampaign(definition);
 ```
 */
export async function runCampaign(definition: CampaignDefinition): Promise<void> {
  await mkdir(
    definition.scratchBase,
    { recursive: true },
  );
  /**
   Disposable build context that holds the Containerfile and the copied tool.
   */
  await using context = await mkdtempDisposable(join(
    definition.scratchBase,
    definition.contextPrefix,
  ));
  await mkdir(
    definition.evidenceRoot,
    { recursive: true },
  );
  /**
   Content-addressed ID of the tested image, which binds the campaign to one source snapshot.
   */
  const baseImage = (await runPodman({
    args: [
      'image',
      'inspect',
      definition.testImage,
      '--format',
      '{{.Id}}'
    ],
    capture: true,
    allowFailure: false,
  })).stdout
    .trim();
  if (!isContentAddressedId(baseImage))
    throw new CargoMutantsToolingError(`Test image did not resolve to a full content-addressed image ID${definition.testImageHint}.`);
  /**
   Proof from the product that this image already passed the whole suite, as the only element, or none.
   */
  const [baselineProof] = await definition.baselineProof?.({ baseImage }) ?? [];
  if (baselineProof !== undefined)
    console.log(`Baseline: skipped, because ${baselineProof} proves this image already passed the whole suite.`);
  else if (definition.baselineProof !== undefined)
    console.log('Baseline: run, because no gate evidence proves this image already passed the whole suite.');
  if ((definition.passOne !== undefined) && (baselineProof === undefined))
    throw new CargoMutantsToolingError(`A two-pass campaign needs proof that the whole suite already passed on this image, at limits no larger than this campaign's: pass 1 runs only a filtered baseline, and pass 2 may never run, so nothing else would check the whole suite${definition.testImageHint}.`);
  if (definition.passOne !== undefined) {
    await runTwoPasses({
      definition,
      contextPath: context.path,
      baseImage,
      ...(baselineProof === undefined ? {} : { baselineProof }),
    });
    return;
  }
  /**
   The campaign's evidence directory, its lock and this attempt's report directory, which the container writes through
   a bind mount so that every outcome reaches the host as it is scored.
   */
  await using attempt = await prepareAttempt({
    evidenceRoot: definition.evidenceRoot,
    evidencePrefix: definition.evidencePrefix,
    baseImage,
    command: definition.command,
    ...(definition.keyParts === undefined ? {} : { keyParts: definition.keyParts }),
  });
  /**
   Outcome of the campaign run, whose nonzero exit can mean missed mutants rather than a failure.
   */
  const result = await executePass({
    definition,
    contextPath: context.path,
    baseImage,
    ...(baselineProof === undefined ? {} : { baselineProof }),
    command: baselineProof === undefined ? definition.command : skipBaselineArguments(definition.command),
    attempt,
    containerName: `cargo-mutants-${attempt.key}`,
  });
  // Preserve evidence even when missed mutants correctly make the campaign nonzero.
  /**
   Names of every mutant the campaign has scored, composed from all attempts into the report callers read.
   */
  const scoredAfter = await composeReport({ evidence: attempt.evidence });
  /**
   Exit status of the whole campaign, not of this attempt alone.
   */
  const status = cumulativeStatus({
    attemptStatus: result.status,
    names: scoredAfter,
  });
  await writeFile(
    join(
      attempt.evidence,
      'exit.json',
    ),
    evidenceJson(exitRecord({
      status,
      signal: result.signal === '' ? undefined : result.signal,
      reportCopied: true,
      attempt: attempt.number,
      skippedMutants: scoredCount(attempt.scoredBefore),
    })),
  );
  if (status !== 0)
    throw new CargoMutantsToolingError(campaignFailureMessage({
      status,
      reportCopied: true,
      evidence: attempt.evidence,
    }));
  await definition.afterReport?.(attempt.evidence);
}
