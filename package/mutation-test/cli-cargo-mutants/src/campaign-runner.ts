import { createHash } from 'node:crypto';
import {
  copyFile,
  mkdir,
  mkdtemp,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  campaignContainerfile,
  evidenceJson,
  exitRecord,
  isContentAddressedId,
} from './campaign-files.ts';
import {
  buildLimitArguments,
  type ContainerLimits,
  runLimitArguments,
} from './container-limits.ts';
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
  readonly command: readonly string[];
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
  readonly command: readonly string[];
  /**
   Work a product does after the tested image is known and before the campaign image is built.
   */
  readonly beforeBuild?: (context: BeforeBuildContext) => Promise<void>;
  /**
   Record written as `manifest.json`.
   */
  readonly manifest: (facts: ManifestFacts) => unknown;
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
   Evidence directory of this campaign.
   */
  const evidence = await mkdtemp(join(
    definition.evidenceRoot,
    definition.evidencePrefix,
  ));
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
  console.log(`Mutation evidence: ${evidence}`);
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
      command: definition.command,
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
   ID of the created campaign container.
   */
  const containerId = (await runPodman({
    args: [
      'create',
      '--init',
      '--network=none',
      ...runLimitArguments({
        limits: definition.limits,
        withPidsLimit: true,
      }),
      campaignImage,
      ...definition.command,
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
  if (created !== JSON.stringify(definition.command))
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
      command: definition.command,
    })),
  );
  /**
   Outcome of the campaign run, whose nonzero exit can mean missed mutants rather than a failure.
   */
  const result = await runPodman({
    args: [
      'start',
      '--attach',
      container.id
    ],
    capture: false,
    allowFailure: true,
  });
  // Preserve evidence even when missed mutants correctly make the campaign nonzero.
  /**
   Outcome of copying the report out of the container.
   */
  const copy = await runPodman({
    args: [
      'cp',
      `${container.id}:${definition.containerReport}/.`,
      evidence
    ],
    capture: true,
    allowFailure: true,
  });
  await writeFile(
    join(
      evidence,
      'exit.json',
    ),
    evidenceJson(exitRecord({
      status: result.status,
      signal: result.signal === '' ? undefined : result.signal,
      reportCopied: copy.status === 0,
      ...(copy.status === 0 ? {} : { reportCopyError: copy.stderr }),
    })),
  );
  if (result.status !== 0)
    throw new CargoMutantsToolingError(`Mutation campaign exited ${result.status}; report ${copy.status === 0 ? 'copied' : 'not generated or not copied'}; inspect ${evidence}.`);
  if (copy.status !== 0)
    throw new CargoMutantsToolingError(`Mutation campaign passed but its report was not copied: ${copy.stderr}`);
}
