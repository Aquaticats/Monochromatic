#!/usr/bin/env node
/**
 Build instrumented targets with a read-only compiler, then fuzz without host mounts.
 */
import {
  cp,
  mkdir,
  mkdtempDisposable,
  mkdtemp,
} from 'node:fs/promises';
import { join } from 'node:path';

import { runCommand } from '../../../git-policy/cli/bin/native-verification-process.ts';
import { buildInstrumented } from './fuzz-build.ts';
import {
  FuzzVerificationError,
  scratchBase,
} from './fuzz-constants.ts';
import { prepareBuildContext } from './fuzz-context.ts';
import {
  buildRunImage,
  fuzzTargets,
} from './fuzz-run.ts';

/**
 Read one line of output from the host compiler.

 @param args - arguments of `rustc`

 @returns the trimmed standard output

 @throws NativeVerificationError when `rustc` exits nonzero

 @example
 ```ts
 const version = await rustc(['--version']);
 ```
 */
async function rustc(args: readonly string[]): Promise<string> {
  return (await runCommand({
    command: 'rustc',
    args,
    capture: true,
  })).stdout
    .trim();
}

/**
 Build and run within explicit container limits and retain discoveries before cleanup.

 @throws FuzzVerificationError when an option other than `--build-only` is given, the compiler is not nightly or a
 step fails

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  if (process.argv
    .slice(2)
    .some(function isUnknown(argument) {
      return argument !== '--build-only';
    }))
    throw new FuzzVerificationError('Only --build-only is accepted.');
  /**
   Whether to stop after the instrumented build.
   */
  const buildOnly = process.argv
    .includes('--build-only');
  /**
   The fuzz sidecar directory, which is the working directory of the task.
   */
  const source = process.cwd();
  /**
   The host compiler's sysroot.
   */
  const compiler = await rustc([
    '--print',
    'sysroot',
  ]);
  /**
   The host compiler's version text.
   */
  const compilerVersion = await rustc(['--version']);
  if (!compilerVersion.includes('nightly'))
    throw new FuzzVerificationError('Fuzz instrumentation requires the repository-managed nightly Rust toolchain.');
  await mkdir(
    scratchBase,
    { recursive: true },
  );
  /**
   The disposable build context.
   */
  await using context = await mkdtempDisposable(join(
    scratchBase,
    'monochromatic-lint-fuzz-',
  ));
  /**
   Directory that holds one evidence directory per campaign.
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
   The evidence directory of this campaign.
   */
  const evidence = await mkdtemp(join(
    outputRoot,
    'campaign-',
  ));
  await prepareBuildContext({
    source,
    context: context.path,
  });
  await buildInstrumented({
    context: context.path,
    evidence,
    compiler,
    compilerVersion,
  });
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
    return;
  }
  await buildRunImage({
    context: context.path,
    source,
  });
  console.log(`Fuzz evidence: ${evidence}`);
  await fuzzTargets(evidence);
}

await main();
