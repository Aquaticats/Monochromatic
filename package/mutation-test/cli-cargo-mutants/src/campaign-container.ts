import { createHash } from 'node:crypto';
import {
  copyFile,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import type { PreparedAttempt } from './campaign-attempt.ts';
import type { CampaignDefinition } from './campaign-definition.ts';
import {
  campaignContainerfile,
  evidenceJson,
  isContentAddressedId,
} from './campaign-files.ts';
import { startAttached } from './campaign-signals.ts';
import {
  buildLimitArguments,
  runLimitArguments,
} from './container-limits.ts';
import { containerStartedFile } from './constants.ts';
import { keepIdArguments } from './container-user.ts';
import { CargoMutantsToolingError } from './error.ts';
import {
  type PodmanResult,
  runPodman,
} from './podman.ts';

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
 Build the campaign image, run its container over one prepared attempt and remove the container.

 The image layers the installed cargo-mutants over the tested image. The container starts from the ID the build wrote,
 not from the shared tag, and is given its command explicitly, which is read back from the created container: two builds
 started together whose Containerfiles differ only in their default command can commit one image, so the image's own
 default command never decides what a campaign runs. The attempt's report directory is bind-mounted at the report path
 so every outcome reaches the host as cargo-mutants scores it.

 @param definition - the product's names, limits and hooks

 @param contextPath - throwaway build context directory

 @param baseImage - full content-addressed ID of the tested image

 @param baselineProof - the product's proof that the image passed the whole suite, which makes the baseline skipped

 @param command - the command the container runs, with its baseline setting applied

 @param attempt - the prepared attempt whose report directory the container writes

 @param containerName - name of the container, so that one left behind by a killed run is found and removed

 @param pass - the pass of a two-pass campaign, absent in a one-pass campaign

 @returns how the container ended; a nonzero status can mean missed mutants rather than a failure

 @throws CargoMutantsToolingError when an image or container ID is incomplete or the created container would run
 another command

 @example
 ```ts
 const result = await executePass({ definition, contextPath, baseImage, command, attempt, containerName: 'cargo-mutants-key' });
 ```
 */
export async function executePass({
  definition,
  contextPath,
  baseImage,
  baselineProof,
  command,
  attempt,
  containerName,
  pass,
}: {
  readonly definition: CampaignDefinition;
  readonly contextPath: string;
  readonly baseImage: string;
  readonly baselineProof?: string;
  readonly command: readonly string[];
  readonly attempt: Pick<PreparedAttempt, 'evidence' | 'reportDirectory' | 'seededMutants'>;
  readonly containerName: string;
  readonly pass?: 1 | 2;
}): Promise<PodmanResult> {
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
    contextPath,
    evidence: attempt.evidence,
    scoredMutants: attempt.seededMutants,
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
      contextPath,
      'cargo-mutants',
    ),
  );
  await writeFile(
    join(
      contextPath,
      'Containerfile',
    ),
    campaignContainerfile({
      baseImage,
      command,
    }),
  );
  /**
   File that receives the ID of the campaign image.
   */
  const imageIdFile = join(
    contextPath,
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
      contextPath,
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
      ...command,
    ],
    capture: true,
    allowFailure: false,
  })).stdout
    .trim();
  /**
   The campaign container, removed when the pass ends whether it passed or failed.
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
  if (created !== JSON.stringify(command))
    throw new CargoMutantsToolingError(`The created container would run ${created}, not this campaign's command.`);
  await writeFile(
    join(
      attempt.evidence,
      'manifest.json',
    ),
    evidenceJson(definition.manifest({
      baseImage,
      campaignImage,
      toolSha256,
      container: container.id,
      command,
      ...(baselineProof === undefined ? {} : { baselineProof }),
      ...(pass === undefined ? {} : { pass }),
    })),
  );
  // The container is about to run: this file, unlike the manifest or a report directory (which seeding creates too), is
  // the evidence that an attempt of this pass started one, for whoever finds the manifest gone.
  await writeFile(
    join(
      attempt.evidence,
      containerStartedFile,
    ),
    evidenceJson({ container: container.id }),
  );
  return await startAttached(container.id);
}
