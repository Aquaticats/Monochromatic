#!/usr/bin/env node
/**
 Run cargo-mutants against an already tested, disposable container snapshot.
 Source mutation and compilation stay inside the container; only reports leave it.
 `--list <scope>` prints one scope's mutants on the host, and `--coverage` proves that the scopes together
 hold every mutant of the unscoped listing; both read the same scope table the campaigns use.
 The campaign flow, the listing and the coverage proof are the shared ones in
 `@monochromatic-dev/mutation-test-cli-cargo-mutants`; this script supplies the linter's names, scopes and options.
 */
import { join } from 'node:path';

import {
  campaignCommand,
  CargoMutantsToolingError,
  checkScopeCoverage,
  type ContainerLimits,
  defaultBuildTimeoutSeconds,
  listMutants,
  parseContainerLimitOptions,
  runCampaign,
  takeMutantRegex,
  takeShard,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  defaultTimeoutSeconds,
  type MutationScope,
  scopes,
  unscoped,
} from './mutation-scopes.ts';

/**
 Concurrent worktrees set MONOCHROMATIC_LINT_IMAGE_TAG so one snapshot never runs another's image.
 */
const imageTag = process.env
  .MONOCHROMATIC_LINT_IMAGE_TAG
  ?? 'development';

/**
 Report directory inside the container.
 */
const containerReport = '/work/mutation-report';

/**
 Directory that keeps the evidence of every campaign.
 */
const evidenceRoot = join(
  process.cwd(),
  'target',
  'verification',
);

/**
 Build and run the tool over an immutable input image, then retain its complete report.

 @param scope - the scope to mutate
 
 @param shard - `k/n` when only one shard of the scope is mutated
 
 @param examine - mutant-name regex when only matching mutants are tried
 
 @param limits - memory, CPUs and processes of the build and the container
 
 @throws CargoMutantsToolingError when a name regex would widen a scope that selects mutants by name, or the campaign fails
 
 @example
 ```ts
 await campaign({ scope: unscoped, limits });
 ```
 */
async function campaign({
  scope,
  shard,
  examine,
  limits,
}: {
  readonly scope: MutationScope;
  readonly shard?: string;
  readonly examine?: string;
  readonly limits: ContainerLimits;
}): Promise<void> {
  // cargo-mutants keeps a mutant that matches any `--re`, so a second one would widen a scope that has its own.
  if ((examine !== undefined)
    && scope.select
      .includes('--re'))
    throw new CargoMutantsToolingError('This scope already selects mutants by name; --re would widen it.');
  /**
   Per-mutant test limit of this scope.
   */
  const timeoutSeconds = scope.timeoutSeconds ?? defaultTimeoutSeconds;
  await runCampaign({
    testImage: `localhost/monochromatic-lint-test:${imageTag}`,
    mutationImage: `localhost/monochromatic-lint-mutation:${imageTag}`,
    testImageHint: '',
    // Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
    // whose backing device is unknowable.
    scratchBase: join(
      import.meta.dirname,
      '..',
      'node_modules',
      '.monochromatic',
      'monochromatic-lint',
    ),
    contextPrefix: 'monochromatic-lint-mutation-',
    evidenceRoot,
    evidencePrefix: 'mutation-',
    limits,
    containerReport,
    command: campaignCommand({
      outputDirectory: containerReport,
      timeoutSeconds,
      buildTimeoutSeconds: defaultBuildTimeoutSeconds,
      additionalExclusions: [],
      tail: [
        ...(shard === undefined ? [] : [
          '--shard',
          shard
        ]),
        ...scope.select,
        ...(examine === undefined ? [] : [
          '--re',
          examine
        ]),
        ...scope.run,
      ],
    }),
    manifest: function lintManifest({
      baseImage,
      campaignImage,
      toolSha256,
      container,
      command,
    }) {
      return {
        baseImage,
        campaignImage,
        shard: shard ?? null,
        examine: examine ?? null,
        toolSha256,
        container,
        command,
        limits: {
          memory: limits.memory,
          cpus: limits.cpus,
          pids: limits.pidsLimit,
          buildTimeoutSeconds: defaultBuildTimeoutSeconds,
          testTimeoutSeconds: timeoutSeconds,
        },
      };
    },
  });
}

/**
 Resolve one optional scope option to its table entry, or to the unscoped campaign when none is given.

 @param option - the scope option, or nothing
 
 @returns the scope
 
 @throws CargoMutantsToolingError when the option names no scope
 
 @example
 ```ts
 scopeFor('--core');
 ```
 */
function scopeFor(option?: string): MutationScope {
  if (option === undefined)
    return unscoped;
  /**
   The table entry of the option.
   */
  const scope = scopes.get(option);
  if (scope === undefined)
    throw new CargoMutantsToolingError(`Unknown scope ${option}; expected one of ${[...scopes.keys()].join(', ')}.`);
  return scope;
}

/**
 Dispatch: a campaign by default, optionally one `--shard k/n` of it or only the mutants whose names match
 `--re <regex>`, `--list [scope]` for one listing, and `--coverage` for the scope-union proof.

 @throws CargoMutantsToolingError when the options are not one of those forms
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Limits from `--memory`, `--cpus` and `--pids-limit`, which override the 2g, 2 CPU and 128 process bounds of the
   build and the container, and the other options.
   */
  const {
    limits,
    rest: options,
  } = parseContainerLimitOptions({
    options: process.argv
      .slice(2),
    acceptPidsLimit: true,
    leadingOnly: false,
    valuelessFlags: [
      ...scopes.keys(),
      '--list',
      '--coverage',
    ],
  });
  if (options[0] === '--coverage') {
    if (options.length !== 1)
      throw new CargoMutantsToolingError('--coverage takes no other option.');
    await checkScopeCoverage({
      scopes: Array.from(
        scopes.keys(),
        function selectOf(option) {
          return [
            option,
            scopeFor(option)
              .select
          ] as const;
        },
      ),
      unscopedSelect: unscoped.select,
      additionalExclusions: [],
      evidenceRoot,
    });
    return;
  }
  if (options[0] === '--list') {
    if (options.length > 2)
      throw new CargoMutantsToolingError('--list takes at most one scope option.');
    console.log((await listMutants({
      select: scopeFor(options[1])
        .select,
      additionalExclusions: [],
    })).join('\n'));
    return;
  }
  /**
   The shard, if any, and the other options.
   */
  const {
    shard,
    rest: withoutShard,
  } = takeShard(options);
  /**
   The mutant-name regex, if any, and the other options.
   */
  const {
    examine,
    rest,
  } = takeMutantRegex(withoutShard);
  if (rest.length > 1)
    throw new CargoMutantsToolingError(`Only one of ${[...scopes.keys()].join(', ')} is accepted.`);
  await campaign({
    scope: scopeFor(rest[0]),
    ...(shard === undefined ? {} : { shard }),
    ...(examine === undefined ? {} : { examine }),
    limits,
  });
}

await main();
