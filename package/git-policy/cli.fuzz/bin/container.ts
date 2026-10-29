#!/usr/bin/env node
/** Build instrumented targets with a read-only compiler, then fuzz without host mounts. */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed.
/// <reference types="node" />
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
  parseOptions,
  targets,
} from './fuzz-options.ts';
import { fuzzTarget } from './fuzz-run.ts';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, '..', 'node_modules', '.monochromatic', 'git-policy-fuzz');
makeScratchBase(scratchBase, { recursive: true });


/**
 Build the run image holding only the instrumented binaries, corpus and dictionaries.

 @param {{ source: string, context: string }} request - this package's directory and the build context
 */
async function buildRunImage({
  source,
  context,
}) {
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

 @param {{ selected: readonly import('./fuzz-options.ts').FuzzTarget[], evidence: string,
   secondsPerTarget: number }} request - the targets, the evidence directory and the seconds each runs
 @returns {AsyncGenerator<string | undefined>} one outcome per target: what went wrong, or nothing
 */
async function* fuzzOutcomes({
  selected,
  evidence,
  secondsPerTarget,
}) {
  for (const target of selected) {
    yield fuzzTarget({
      target,
      evidence,
      secondsPerTarget,
    });
  }
}

const {
  buildOnly,
  secondsPerTarget,
  selectedTargets,
} = parseOptions(process.argv
  .slice(2));
const source = process.cwd();
const compiler = (await runCommand({
  command: 'rustc',
  args: [
    '--print',
    'sysroot',
  ],
  capture: true,
})).stdout
  .trim();
const compilerVersion = (await runCommand({
  command: 'rustc',
  args: ['--version'],
  capture: true,
})).stdout
  .trim();
if (!compilerVersion.includes('nightly'))
  throw new NativeVerificationError('Fuzz instrumentation requires the repository-managed nightly Rust toolchain.');
await using context = await mkdtempDisposable(join(
  scratchBase,
  'git-policy-cli-fuzz-',
));
const outputRoot = join(
  source,
  'target',
  'verification',
);
await mkdir(
  outputRoot,
  { recursive: true },
);
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
  /** @type {string[]} */
  const failedTargets = [];
  for await (const failure of fuzzOutcomes({
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
