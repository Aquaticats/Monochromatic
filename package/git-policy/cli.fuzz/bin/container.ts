#!/usr/bin/env node
/**
 Build instrumented targets with a read-only compiler, then fuzz without host mounts.
 */
import {
  cp,
  mkdir,
  mkdtemp,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  NativeVerificationError,
  runCommand,
} from '../../cli/bin/native-verification-process.ts';
import { buildInstrumentedTargets } from './fuzz-build.ts';
import {
  baseImage,
  limits,
  maxInputBytes,
  runImage,
} from './fuzz-images.ts';
import {
  type FuzzTarget,
  parseOptions,
  targets,
} from './fuzz-options.ts';
import {
  fuzzTarget,
  type FuzzOutcome,
} from './fuzz-run.ts';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
/**
 Parent of every scratch directory this campaign creates.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'git-policy-fuzz'
);
await mkdir(
  scratchBase,
  { recursive: true }
);

/**
 Build the run image holding only the instrumented binaries, corpus and dictionaries.

 @param source - this package's directory

 @param context - the build context
 */
async function buildRunImage({
  source,
  context,
}: {
  readonly source: string;
  readonly context: string;
}): Promise<void> {
  await cp(
    join(
      source,
      'seed',
    ),
    join(
      context,
      'corpus',
    ),
    { recursive: true },
  );
  await cp(
    join(
      source,
      'dictionary',
    ),
    join(
      context,
      'dictionary',
    ),
    { recursive: true },
  );
  await writeFile(
    join(
      context,
      'Run.Containerfile',
    ),
    [
      '# Fuzz execution has no compiler, registry, repository, home, or credential mounts.',
      `FROM ${baseImage}`,
      'COPY bin /fuzz/bin',
      'COPY corpus /fuzz/corpus',
      'COPY dictionary /fuzz/dictionary',
      `RUN ${JSON.stringify([
        'mkdir',
        '--parents',
        ...targets.map(function artifactDirectory(target) {
          return `/fuzz/artifacts/${target.name}`;
        }),
      ])}`,
      'WORKDIR /fuzz',
      '',
    ].join('\n'),
  );
  await runCommand({
    command: 'podman',
    args: [
      'build',
      '--network=none',
      '--http-proxy=false',
      '--pull=never',
      '--memory=2g',
      '--cpu-period=100000',
      '--cpu-quota=200000',
      '--tag',
      runImage,
      '--file',
      join(
        context,
        'Run.Containerfile',
      ),
      context,
    ],
  });
}

/**
 Fuzz each selected target in turn. A failed target is recorded and the campaign goes on,
 so one finding never hides the targets after it.

 @param selected - the targets

 @param evidence - the evidence directory

 @param secondsPerTarget - the seconds each runs

 @returns one outcome per target: what went wrong, or nothing
 */
async function* fuzzOutcomes({
  selected,
  evidence,
  secondsPerTarget,
}: {
  readonly selected: readonly FuzzTarget[];
  readonly evidence: string;
  readonly secondsPerTarget: number;
}): AsyncGenerator<FuzzOutcome> {
  for (const target of selected) {
    yield fuzzTarget({
      target,
      evidence,
      secondsPerTarget,
    });
  }
}

/**
 The validated command line: whether to stop after the build, the seconds per target and the targets to fuzz.
 */
const {
  buildOnly,
  secondsPerTarget,
  selectedTargets,
} = parseOptions(process.argv
  .slice(2));
/**
 This package's directory, where the task runs.
 */
const source = process.cwd();
/**
 The host Rust sysroot, mounted read-only into the build containers.
 */
const compiler = (await runCommand({
  command: 'rustc',
  args: [
    '--print',
    'sysroot',
  ],
  capture: true,
})).stdout
  .trim();
/**
 The compiler version, recorded in the manifest and required to be a nightly.
 */
const compilerVersion = (await runCommand({
  command: 'rustc',
  args: ['--version'],
  capture: true,
})).stdout
  .trim();
if (!compilerVersion.includes('nightly'))
  throw new NativeVerificationError('Fuzz instrumentation requires the repository-managed nightly Rust toolchain.');
/**
 Disposable build context that holds the copied inputs and the instrumented binaries.
 */
await using context = await mkdtempDisposable(join(
  scratchBase,
  'git-policy-cli-fuzz-',
));
/**
 Directory under the package's `target/` that keeps the evidence of every campaign.
 */
const outputRoot = join(
  source,
  'target',
  'verification',
);
await mkdir(
  outputRoot,
  { recursive: true },
);
/**
 Evidence directory of this campaign.
 */
const evidence = await mkdtemp(join(
  outputRoot,
  'campaign-',
));
await buildInstrumentedTargets({
  source,
  context: context.path,
  evidence,
  compiler,
});
await writeFile(
  join(
    evidence,
    'manifest.json',
  ),
  `${JSON.stringify(
    {
      baseImage,
      compilerVersion,
      targets: targets.map(function targetName(target) {
        return target.name;
      }),
      sanitizer: 'address',
      maxInputBytes,
      secondsPerTarget,
      memory: '2g',
      cpus: 2,
      pids: 128,
      compilerMount: 'read-only during compilation with container label isolation disabled; absent during fuzzing',
    },
    null,
    2,
  )}\n`,
);
if (buildOnly) {
  await cp(
    join(
      context.path,
      'bin',
    ),
    join(
      evidence,
      'bin',
    ),
    { recursive: true },
  );
  console.log(`Instrumented binaries: ${evidence}`);
}
else {
  await buildRunImage({
    source,
    context: context.path,
  });
  console.log(`Fuzz evidence: ${evidence}`);
  /**
   What went wrong for each target that failed.
   */
  const failedTargets: string[] = [];
  for await (const { failure, } of fuzzOutcomes({
    selected: selectedTargets,
    evidence,
    secondsPerTarget,
  })) {
    if (failure !== undefined)
      failedTargets.push(failure);
  }
  if (failedTargets.length > 0)
    throw new NativeVerificationError(`${failedTargets.join('; ')}; retained inputs and logs are in ${evidence}.`);
}
