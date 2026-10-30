/**
 The instrumented build: image, generator controls, Clippy and the AddressSanitizer fuzz binaries.
 */
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  createContainer,
  runCommand,
} from '../../../git-policy/cli/bin/native-verification-process.ts';
import {
  buildBounds,
  buildImage,
  FuzzVerificationError,
  limits,
  targets,
} from './fuzz-constants.ts';

/**
 Compiler files retain their host labels. Do not relabel a shared Rust installation with :Z.
 Only build containers disable label isolation; fuzz execution has no host mounts and keeps it enabled.

 @param compiler - the host compiler's sysroot

 @returns the arguments that mount it read-only

 @example
 ```ts
 const mount = compilerMount('/usr/lib/rustlib');
 ```
 */
function compilerMount(compiler: string): readonly string[] {
  return [
    '--security-opt',
    'label=disable',
    '--volume',
    `${compiler}:/toolchain:ro`,
  ];
}

/**
 Build the build image and run the generator controls and Clippy in it.

 @param context - the build context with its Containerfile

 @param compiler - the host compiler's sysroot

 @throws NativeVerificationError when a step exits nonzero

 @example
 ```ts
 await checkInBuildImage({ context, compiler });
 ```
 */
async function checkInBuildImage({
  context,
  compiler,
}: {
  readonly context: string;
  readonly compiler: string;
}): Promise<void> {
  await runCommand({
    command: 'podman',
    args: [
      'build',
      ...buildBounds,
      '--tag',
      buildImage,
      context,
    ],
  });
  await runCommand({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...limits,
      ...compilerMount(compiler),
      buildImage,
      'cargo',
      'nextest',
      'run',
      '--lib',
      '--offline',
      '--locked',
    ],
  });
  // Type/lint the property helpers in the same bounded compiler context as their generator controls.
  await runCommand({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...limits,
      ...compilerMount(compiler),
      buildImage,
      'cargo',
      'clippy',
      '--lib',
      '--offline',
      '--locked',
      '--',
      '-D',
      'warnings',
    ],
  });
}

/**
 Build the instrumented fuzz binaries in a container and copy them into the context.

 @param context - the build context; the binaries land in its `bin`

 @param evidence - the evidence directory that receives the build state

 @param compiler - the host compiler's sysroot

 @throws FuzzVerificationError when the instrumented build exits nonzero

 @example
 ```ts
 await buildBinaries({ context, evidence, compiler });
 ```
 */
async function buildBinaries({
  context,
  evidence,
  compiler,
}: {
  readonly context: string;
  readonly evidence: string;
  readonly compiler: string;
}): Promise<void> {
  /**
   The build container, removed when this function ends.
   */
  await using buildContainer = await createContainer({
    args: [
      'create',
      ...limits,
      ...compilerMount(compiler),
      buildImage,
      'cargo',
      'fuzz',
      'build',
      '--fuzz-dir',
      '.',
      '--sanitizer',
      'address',
      '--codegen-units',
      '16',
      '--target',
      'x86_64-unknown-linux-gnu',
      '--target-dir',
      '/work/build',
    ],
  });
  /**
   How the instrumented build ended.
   */
  const built = await runCommand({
    command: 'podman',
    args: [
      'start',
      '--attach',
      buildContainer.id,
    ],
    allowFailure: true,
  });
  /**
   The container's state, kept as evidence.
   */
  const buildState = await runCommand({
    command: 'podman',
    args: [
      'inspect',
      '--format',
      '{{json .State}}',
      buildContainer.id,
    ],
    capture: true,
  });
  await writeFile(
    join(
      evidence,
      'build-state.json',
    ),
    buildState.stdout,
  );
  if (built.status !== 0)
    throw new FuzzVerificationError(`Instrumented build exited ${built.status ?? 'without a status'}; container state retained in ${evidence}.`);
  await mkdir(
    join(
      context,
      'bin',
    ),
    { recursive: true },
  );
  await mapInSequence({
    items: targets,
    run: function copyOf(target) {
      return runCommand({
        command: 'podman',
        args: [
          'cp',
          `${buildContainer.id}:/work/build/x86_64-unknown-linux-gnu/release/${target}`,
          join(
            context,
            'bin',
            target,
          ),
        ],
      });
    },
  });
}

/**
 Build the image, check the generators, build the instrumented binaries and record how they were built.

 @param context - the build context with its Containerfile

 @param evidence - the evidence directory

 @param compiler - the host compiler's sysroot

 @param compilerVersion - the compiler version text, recorded in the manifest

 @throws FuzzVerificationError when the instrumented build exits nonzero

 @example
 ```ts
 await buildInstrumented({ context, evidence, compiler, compilerVersion });
 ```
 */
export async function buildInstrumented({
  context,
  evidence,
  compiler,
  compilerVersion,
}: {
  readonly context: string;
  readonly evidence: string;
  readonly compiler: string;
  readonly compilerVersion: string;
}): Promise<void> {
  await checkInBuildImage({
    context,
    compiler,
  });
  await buildBinaries({
    context,
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
        compilerVersion,
        targets,
        sanitizer: 'address',
        buildJobs: 1,
        codegenUnits: 16,
        maxInputBytes: 8_192,
        secondsPerTarget: 30,
        memory: '2g',
        cpus: 2,
        compilerMount: 'read-only during compilation with container label isolation disabled; absent during fuzzing',
      },
      null,
      2,
    )}\n`,
  );
}
