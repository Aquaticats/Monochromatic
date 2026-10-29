import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  copyFile,
  mkdir,
  mkdtempDisposable,
  symlink,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

//region Disposable consumer

/**
 Runs copied extension and helper artifacts without modifying installed files.

 @param scenario - startup dependency removed by fixture driver

 @returns consumer stdout after successful verification

 @example
 ```ts
 await runStartupFixture('normal');
 ```
 */
export async function runStartupFixture(scenario: string,): Promise<string> {
  /**
   Private fixture also owns copied runtime when testing unlinked executables.
   */
  await using directory = await mkdtempDisposable(join(
    scratchRoot(),
    'ask-startup-fixture-',
  ),);
  /**
   Consumer layout retains static artifact imports and exercises spaces in paths.
   */
  const packageDirectory = join(
    directory.path,
    'package with spaces',
  );
  /**
   Independently removable installation directory.
   */
  const bundleDirectory = join(
    packageDirectory,
    'dist/final/node',
  );
  /**
   Source driver keeps same relative artifact import as original test fixture.
   */
  const sourceDirectory = join(
    packageDirectory,
    'src',
  );
  await Promise.all([
    mkdir(
      bundleDirectory,
      { recursive: true, },
    ),
    mkdir(
      sourceDirectory,
      { recursive: true, },
    ),
  ],);
  await symlink(
    fileURLToPath(new URL(
      '../node_modules',
      import.meta.url,
    ),),
    join(
      directory.path,
      'node_modules',
    ),
    process.platform === 'win32' ? 'junction' : 'dir',
  );
  await Promise.all([
    'index.mjs',
    'answer-helper.mjs',
  ].map(async function copyArtifact(filename,) {
    await copyFile(
      new URL(
        `../dist/final/node/${filename}`,
        import.meta.url,
      ),
      join(
        bundleDirectory,
        filename,
      ),
    );
  },),);
  /**
   Copied driver executes only against disposable artifacts.
   */
  const driver = join(
    sourceDirectory,
    'driver.ts',
  );
  await copyFile(
    new URL(
      'request-startup-driver-fixture.ts',
      import.meta.url,
    ),
    driver,
  );
  /**
   Only runtime-removal scenario runs a disposable copy of Node.
   */
  const runtime = scenario === 'runtime-removed'
    ? join(
      directory.path,
      'node',
    )
    : process.execPath;
  if (scenario === 'runtime-removed')
    await copyFile(
      process.execPath,
      runtime,
    );
  /**
   Child owns test scenario; never removes repository artifacts.
   */
  const child = spawn(
    runtime,
    [
      driver,
      scenario,
    ],
    {
      cwd: directory.path,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Both streams retained so shutdown noise also fails verification.
   */
  const output = {
    stdout: '',
    stderr: '',
  };
  child.stdout
    .setEncoding('utf8',);
  child.stderr
    .setEncoding('utf8',);
  child.stdout
    .on(
    'data',
    function captureOutput(chunk: string,): void {
      output.stdout += chunk;
    },
    );
  child.stderr
    .on(
    'data',
    function captureError(chunk: string,): void {
      output.stderr += chunk;
    },
    );
  /**
   Close waits for all stderr before evaluating child outcome.
   */
  const exit: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   Exit code narrowed independently from untyped event tuple.
   */
  const [code,] = exit;
  if (code !== 0)
    throw new Error(`Startup fixture ${scenario} exited ${String(code,)}:\n${output.stderr}`,);
  if (output.stderr !== '')
    throw new Error(`Startup fixture ${scenario} wrote unexpected stderr:\n${output.stderr}`,);
  return output.stdout;
}

//endregion Disposable consumer
