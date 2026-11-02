import { createHash } from 'node:crypto';
import {
  copyFile,
  mkdir,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  campaignContainerfile,
  campaignFailureMessage,
  evidenceJson,
  exitRecord,
  isContentAddressedId,
  skipBaselineArguments,
} from './campaign-files.ts';
import { prepareAttempt } from './campaign-attempt.ts';
import { scoredCount } from './campaign-ledger.ts';
import {
  composeReport,
  cumulativeStatus,
} from './campaign-report.ts';
import { startAttached } from './campaign-signals.ts';
import {
  buildLimitArguments,
  type ContainerLimits,
  runLimitArguments,
} from './container-limits.ts';
import { keepIdArguments } from './container-user.ts';
import { CargoMutantsToolingError } from './error.ts';
import { runPodman } from './podman.ts';

/**
 Facts about a finished set-up that a product's manifest records.
 */
export type ManifestFacts = {
  readonly baseImage: string;
  readonly campaignImage: string;
  readonly toolSha256: string;
  readonly container: string;
  /**
   The command the container runs, which has `--baseline skip` when the product proved the baseline.
   */
  readonly command: readonly string[];
  /**
   The product's proof that the tested image already passed the whole suite, when the baseline was skipped.
   */
  readonly baselineProof?: string;
};

/**
 What a product's hook before the image build receives.
 */
export type BeforeBuildContext = {
  readonly baseImage: string;
  /**
   Podman arguments that bound a container: no init process or network, the memory, the CPUs and the process cap.
   */
  readonly containerArguments: readonly string[];
  readonly contextPath: string;
  readonly evidence: string;
  /**
   Mutants that earlier attempts of this campaign scored: zero on the first attempt, more when the campaign resumes,
   which tells the hook that whatever it proves before scoring was proved by an earlier attempt.
   */
  readonly scoredMutants: number;
};

/**
 Everything that makes a product's campaign its own.
 */
export type CampaignDefinition = {
  /**
   Tag of the already tested image the campaign layers over.
   */
  readonly testImage: string;
  /**
   Tag given to the campaign image.
   */
  readonly mutationImage: string;
  /**
   Text appended to the message when the tested image does not resolve, such as the task that builds it.
   */
  readonly testImageHint: string;
  /**
   Directory under which the throwaway build context is created.
   */
  readonly scratchBase: string;
  /**
   Name prefix of the build-context directory.
   */
  readonly contextPrefix: string;
  /**
   Directory that keeps the evidence of every campaign.
   */
  readonly evidenceRoot: string;
  /**
   Name prefix of this campaign's evidence directory.
   */
  readonly evidencePrefix: string;
  readonly limits: ContainerLimits;
  /**
   Report directory inside the container.
   */
  readonly containerReport: string;
  /**
   The campaign command with its baseline run; it names the campaign, so its key does not change when the baseline is
   later skipped.
   */
  readonly command: readonly string[];
  /**
   Look for proof that the tested image already passed the whole suite, so the campaign's own baseline run adds nothing.
   It is called with the full image ID the campaign resolved, and answers a short description of the proof (where it
   was found) or nothing when there is none. With a proof the container runs `--baseline skip`; with none the baseline
   runs. The answer must never be a guess: a baseline that fails in the container makes every mutant look caught.
   */
  readonly baselineProof?: (context: { readonly baseImage: string }) => Promise<string | undefined>;
  /**
   Work a product does after the tested image is known and before the campaign image is built.
   */
  readonly beforeBuild?: (context: BeforeBuildContext) => Promise<void>;
  /**
   Record written as `manifest.json`.
   */
  readonly manifest: (facts: ManifestFacts) => unknown;
  /**
   Check a product runs on the evidence directory once the campaign passed and its report was copied.
   */
  readonly afterReport?: (evidence: string) => Promise<void>;
};

/**
 Path of the installed cargo-mutants executable.

 @returns `$CARGO_HOME/bin/cargo-mutants`, with the home directory's `.cargo` when the variable is unset
 
 @example
 ```ts
 installedToolPath();
 ```
 */
export function installedToolPath(): string {
  return join(
    process.env
      .CARGO_HOME
      ?? join(
        homedir(),
        '.cargo',
      ),
    'bin',
    'cargo-mutants',
  );
}

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
   Proof from the product that this image already passed the whole suite, or nothing.
   */
  const baselineProof = await definition.baselineProof?.({ baseImage });
  /**
   The command the container runs: the campaign command, with the baseline skipped only on proof.
   */
  const containerCommand = baselineProof === undefined
    ? definition.command
    : skipBaselineArguments(definition.command);
  if (baselineProof !== undefined)
    console.log(`Baseline: skipped, because ${baselineProof} proves this image already passed the whole suite.`);
  else if (definition.baselineProof !== undefined)
    console.log('Baseline: run, because no gate evidence proves this image already passed the whole suite.');
  /**
   The campaign's evidence directory, its lock and this attempt's report directory, which the container writes through
   a bind mount so that every outcome reaches the host as it is scored.
   */
  await using attempt = await prepareAttempt({
    evidenceRoot: definition.evidenceRoot,
    evidencePrefix: definition.evidencePrefix,
    baseImage,
    command: definition.command,
  });
  /**
   Parts of the prepared attempt this function uses by name.
   */
  const {
    evidence,
    key,
    scoredBefore,
  } = attempt;
  await definition.beforeBuild?.({
    baseImage,
    containerArguments: [
      '--init',
      '--network=none',
      ...runLimitArguments({
        limits: definition.limits,
        withPidsLimit: true,
      }),
    ],
    contextPath: context.path,
    evidence,
    scoredMutants: scoredCount(scoredBefore),
  });
  /**
   Host path of the installed cargo-mutants executable.
   */
  const tool = installedToolPath();
  /**
   Digest that identifies the exact tool build in the manifest.
   */
  const toolSha256 = createHash('sha256')
    .update(await readFile(tool))
    .digest('hex');
  await copyFile(
    tool,
    join(
      context.path,
      'cargo-mutants',
    ),
  );
  await writeFile(
    join(
      context.path,
      'Containerfile',
    ),
    campaignContainerfile({
      baseImage,
      command: containerCommand,
    }),
  );
  /**
   File that receives the ID of the campaign image.
   */
  const imageIdFile = join(
    context.path,
    'image-id',
  );
  await runPodman({
    args: [
      'build',
      '--network=none',
      '--http-proxy=false',
      '--pull=never',
      ...buildLimitArguments(definition.limits),
      '--iidfile',
      imageIdFile,
      '--tag',
      definition.mutationImage,
      context.path,
    ],
    capture: false,
    allowFailure: false,
  });
  /**
   Content-addressed ID of the campaign image the build wrote.
   */
  const campaignImage = (await readFile(
    imageIdFile,
    'utf8',
  )).trim()
    .replace(
    'sha256:',
    '',
    );
  if (!isContentAddressedId(campaignImage))
    throw new CargoMutantsToolingError('The campaign image build did not report a full content-addressed image ID.');
  /**
   The user the campaign image runs as, which decides how the host user maps into the container.
   */
  const imageUser = (await runPodman({
    args: [
      'image',
      'inspect',
      campaignImage,
      '--format',
      '{{.Config.User}}'
    ],
    capture: true,
    allowFailure: false,
  })).stdout;
  /**
   Name of the campaign's container, so that one left behind by a killed run is found and removed.
   */
  const containerName = `cargo-mutants-${key}`;
  await runPodman({
    args: [
      'rm',
      '--force',
      '--ignore',
      containerName
    ],
    capture: true,
    allowFailure: false,
  });
  /**
   ID of the created campaign container.
   */
  const containerId = (await runPodman({
    args: [
      'create',
      '--name',
      containerName,
      '--init',
      '--network=none',
      // The host user stands for the image's user, so the report files the container writes belong to the host user.
      ...keepIdArguments(imageUser),
      // The label option lets the container write the host directory under SELinux.
      '--volume',
      `${attempt.reportDirectory}:${definition.containerReport}:z`,
      ...runLimitArguments({
        limits: definition.limits,
        withPidsLimit: true,
      }),
      campaignImage,
      ...containerCommand,
    ],
    capture: true,
    allowFailure: false,
  })).stdout
    .trim();
  /**
   The campaign container, removed when the campaign ends whether it passed or failed.
   */
  await using container = {
    id: containerId,
    async [Symbol.asyncDispose](): Promise<void> {
      await runPodman({
        args: [
          'rm',
          '--force',
          containerId
        ],
        capture: true,
        allowFailure: false,
      });
    },
  };
  if (!isContentAddressedId(container.id))
    throw new CargoMutantsToolingError('Container creation did not return a complete container ID.');
  /**
   Command the created container would run.
   */
  const created = (await runPodman({
    args: [
      'container',
      'inspect',
      container.id,
      '--format',
      '{{json .Config.Cmd}}'
    ],
    capture: true,
    allowFailure: false,
  })).stdout
    .trim();
  if (created !== JSON.stringify(containerCommand))
    throw new CargoMutantsToolingError(`The created container would run ${created}, not this campaign's command.`);
  await writeFile(
    join(
      evidence,
      'manifest.json',
    ),
    evidenceJson(definition.manifest({
      baseImage,
      campaignImage,
      toolSha256,
      container: container.id,
      command: containerCommand,
      ...(baselineProof === undefined ? {} : { baselineProof }),
    })),
  );
  /**
   Outcome of the campaign run, whose nonzero exit can mean missed mutants rather than a failure.
   */
  const result = await startAttached(container.id);
  // Preserve evidence even when missed mutants correctly make the campaign nonzero.
  /**
   Names of every mutant the campaign has scored, composed from all attempts into the report callers read.
   */
  const scoredAfter = await composeReport(evidence);
  /**
   Exit status of the whole campaign, not of this attempt alone.
   */
  const status = cumulativeStatus({
    attemptStatus: result.status,
    names: scoredAfter,
  });
  await writeFile(
    join(
      evidence,
      'exit.json',
    ),
    evidenceJson(exitRecord({
      status,
      signal: result.signal === '' ? undefined : result.signal,
      reportCopied: true,
      attempt: attempt.number,
      skippedMutants: scoredCount(scoredBefore),
    })),
  );
  if (status !== 0)
    throw new CargoMutantsToolingError(campaignFailureMessage({
      status,
      reportCopied: true,
      evidence,
    }));
  await definition.afterReport?.(evidence);
}
