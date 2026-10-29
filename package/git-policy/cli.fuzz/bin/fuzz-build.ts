/**
 Build the instrumented fuzz targets with a read-only compiler inside bounded containers.

 The compiler is a separate read-only input mounted only while compiling; fuzz execution never sees it.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed.
/// <reference types="node" />
import {
  copyFile,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  copyNextest,
  nextestCopyLines
} from '../../cli/bin/native-nextest-snapshot.mjs';
import {
  createContainer,
  NativeVerificationError,
  runCommand,
} from '../../cli/bin/native-verification-process.mjs';
import {
  baseImage,
  buildImage,
  limits,
} from './fuzz-images.mjs';
import {
  copyInputs,
  executable,
} from './fuzz-inputs.mjs';
import { targets } from './fuzz-options.mjs';

/**
 Run `cargo vendor` and map the vendored directory to its path inside the build image.

 @param {string} context - build context directory
 @returns {Promise<string>} the Cargo source configuration for the image
 */
async function vendoredConfig(context) {
  const vendor = await runCommand({
    command: 'cargo',
    args: [
      'vendor',
      '--locked',
      '--offline',
      '--versioned-dirs',
      join(
        context,
        'vendor',
      ),
    ],
    capture: true,
  });
  const config = vendor.stdout
    .replaceAll(
    JSON.stringify(join(
      context,
      'vendor',
    )),
    '"/work/vendor"',
    );
  if (!config.includes('directory = "/work/vendor"'))
    throw new NativeVerificationError('Cargo vendor did not emit the expected source directory mapping.');
  return config;
}

/**
 Write the Containerfile of the build image and build it.

 @param {string} context - build context directory holding the copied inputs
 */
async function buildImageFrom(context) {
  await writeFile(
    join(
      context,
      'Containerfile',
    ),
    [
      '# Installed Rust runtime image; the compiler is a separate read-only input.',
      `FROM ${baseImage}`,
      'COPY package /work/package',
      'COPY vendor /work/vendor',
      'COPY .cargo /work/.cargo',
      'COPY cargo-fuzz /usr/local/cargo/bin/cargo-fuzz',
      'RUN ["cargo-fuzz", "--version"]',
      ...nextestCopyLines(),
      'WORKDIR /work/package/git-policy/cli.fuzz',
      'ENV CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0',
      'ENV PATH="/toolchain/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"',
      'ENV RUSTC="/toolchain/bin/rustc" RUSTDOC="/toolchain/bin/rustdoc"',
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
      buildImage,
      context,
    ],
  });
}

/**
 Generator controls and Clippy prove the invariants are reached before any fuzzing result is trusted.

 @param {readonly string[]} compilerMount - `podman run` options that mount the read-only compiler
 */
async function proveGeneratorControls(compilerMount) {
  const prefix = [
    'run',
    '--rm',
    ...limits,
    ...compilerMount,
    buildImage,
    'cargo',
  ];
  await runCommand({
    command: 'podman',
    args: [
      ...prefix,
      'nextest',
      'run',
      '--lib',
      '--offline',
      '--locked',
    ],
  });
  await runCommand({
    command: 'podman',
    args: [
      ...prefix,
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
 Build the AddressSanitizer targets in a container and copy their binaries into the context.

 @param {{ context: string, evidence: string, compilerMount: readonly string[] }} request - the build context,
   the evidence directory receiving the container state, and the read-only compiler mount
 @throws {NativeVerificationError} when the instrumented build exits nonzero
 */
async function compileTargets({
  context,
  evidence,
  compilerMount,
}) {
  await using container = await createContainer({
    args: [
      'create',
      ...limits,
      ...compilerMount,
      buildImage,
      'cargo',
      'fuzz',
      'build',
      '--fuzz-dir',
      '.',
      '--sanitizer',
      'address',
      '--target',
      'x86_64-unknown-linux-gnu',
      '--target-dir',
      '/work/build',
    ],
  });
  const built = await runCommand({
    command: 'podman',
    args: [
      'start',
      '--attach',
      container.id,
    ],
    allowFailure: true,
  });
  const state = await runCommand({
    command: 'podman',
    args: [
      'inspect',
      '--format',
      '{{json .State}}',
      container.id,
    ],
    capture: true,
  });
  await writeFile(
    join(
      evidence,
      'build-state.json',
    ),
    state.stdout,
  );
  if (built.status !== 0)
    throw new NativeVerificationError(`Instrumented build exited ${String(built.status)}; container state retained in ${evidence}.`);
  await mkdir(
    join(
      context,
      'bin',
    ),
    { recursive: true },
  );
  await Promise.all(targets.map(function copyBinary(target) {
    return runCommand({
      command: 'podman',
      args: [
        'cp',
        `${container.id}:/work/build/x86_64-unknown-linux-gnu/release/${target.name}`,
        join(
          context,
          'bin',
          target.name,
        ),
      ],
    });
  }));
}

/**
 Copy the inputs, build the image, prove the controls and compile the instrumented binaries into `<context>/bin`.

 @param {{ source: string, context: string, evidence: string, compiler: string }} request - this package's
   directory, the build context, the evidence directory and the compiler sysroot to mount read-only
 */
export async function buildInstrumentedTargets({
  source,
  context,
  evidence,
  compiler,
}) {
  await copyInputs({
    source,
    context,
  });
  await mkdir(
    join(
      context,
      '.cargo',
    ),
    { recursive: true },
  );
  await writeFile(
    join(
      context,
      '.cargo/config.toml',
    ),
    await vendoredConfig(context),
  );
  await copyFile(
    await executable('cargo-fuzz'),
    join(
      context,
      'cargo-fuzz',
    ),
  );
  await copyNextest({ context });
  await buildImageFrom(context);
  // Compiler files retain their host labels. Do not relabel a shared Rust installation with :Z.
  // Only build containers disable label isolation; fuzz execution has no host mounts and keeps it enabled.
  const compilerMount = [
    '--security-opt',
    'label=disable',
    '--volume',
    `${compiler}:/toolchain:ro`,
  ];
  await proveGeneratorControls(compilerMount);
  await compileTargets({
    context,
    evidence,
    compilerMount,
  });
}
