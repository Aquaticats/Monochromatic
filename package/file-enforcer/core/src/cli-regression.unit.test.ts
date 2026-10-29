import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import spawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region CLI fixture helpers

/**
 Source CLI entry used by repo mise tasks.
 */
const CLI_PATH = join(
  import.meta.dirname,
  'cli.ts',
);

/**
 Source public entry imported by generated config fixtures so CLI context and
 builder context share one module instance.
 */
const SOURCE_INDEX_URL = pathToFileURL(join(
  import.meta.dirname,
  'index.ts',
),).href;

/**
 Creates an isolated temp directory for CLI regression tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(
    scratchRoot(),
    'file-enforcer-cli-regression-',
  ),);
}

/**
 Removes an isolated temp directory.
 
 @param tempDir - Directory returned by {@link setup}.
 
 @example
 ```ts
 await teardown(tempDir);
 ```
 */
async function teardown(tempDir: string,): Promise<void> {
  await rm(
    tempDir,
    {
      recursive: true,
      force: true,
    },
  );
}

/**
 Runs the source CLI in a fixture directory.
 
 @param cwd - Directory containing `file-enforcer.config.ts`.
 
 @example
 ```ts
 await runCli({ cwd: tempDir });
 ```
 */
async function runCli({ cwd, }: { readonly cwd: string; },): Promise<void> {
  await spawn(
    'node',
    [CLI_PATH,],
    { cwd, },
  );
}

/**
 Returns JSON text for a generated TypeScript string literal.
 
 @param value - Value to quote.
 
 @returns JavaScript string literal source.
 
 @example
 ```ts
 const literal = jsString('/tmp/path');
 ```
 */
function jsString(value: string,): string {
  return JSON.stringify(value,);
}

//endregion CLI fixture helpers

await describe({
  name: 'file-enforcer CLI regressions',
  concurrency: 1,
  children: [
    it({
      name: 'reruns config when untracked direct dependency changes',
      fn: async function rerunsConfigForUntrackedDependency(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const dependencyPath = join(
          tempDir,
          'dependency.txt',
        );
        const outputPath = join(
          tempDir,
          'output.txt',
        );
        const configPath = join(
          tempDir,
          'file-enforcer.config.ts',
        );
        await writeFile(
          dependencyPath,
          'alpha',
        );
        await writeFile(
          configPath,
          `
import { readFile } from 'node:fs/promises';
import { overwrite } from ${jsString(SOURCE_INDEX_URL,)};

await overwrite({
  dest: './output.txt',
  content: await readFile('./dependency.txt', 'utf8'),
});
`,
        );

        await runCli({ cwd: tempDir, },);
        await writeFile(
          dependencyPath,
          'bravo',
        );
        await runCli({ cwd: tempDir, },);

        expect(await readFile(outputPath, 'utf8',),).toBe('bravo',);
      },
    },),

    it({
      name: 'overwriteIfNotExists existing-file effects do not let CLI skip config',
      fn: async function existingSeededFileDoesNotHideConfigEffects(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const markerPath = join(
          tempDir,
          'runs.txt',
        );
        const seededPath = join(
          tempDir,
          'seeded.txt',
        );
        const configPath = join(
          tempDir,
          'file-enforcer.config.ts',
        );
        await writeFile(
          seededPath,
          'operator-owned',
        );
        await writeFile(
          configPath,
          `
import { appendFile } from 'node:fs/promises';
import { overwrite, overwriteIfNotExists } from ${jsString(SOURCE_INDEX_URL,)};

await appendFile('./runs.txt', 'run\\n');
await overwrite({
  dest: './stable.txt',
  content: 'stable',
});
await overwriteIfNotExists({
  dest: './seeded.txt',
  content: 'default',
});
`,
        );

        await runCli({ cwd: tempDir, },);
        await runCli({ cwd: tempDir, },);

        expect(await readFile(markerPath, 'utf8',),).toBe('run\nrun\n',);
        expect(await readFile(seededPath, 'utf8',),).toBe('operator-owned',);
      },
    },),
  ],
},);
