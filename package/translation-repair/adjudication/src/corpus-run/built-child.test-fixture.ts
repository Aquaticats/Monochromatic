/**
 A child process that runs a few lines against a built package, for a case
 that needs a first call of a memoised function in a process of its own.

 @module
 */

import { spawnKeyless, } from '../child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Runs lines against a built package in a fresh process and returns what it
 wrote to its standard output.

 @param packageEntry - URL of the built index of the package that declares the names the lines read, as `import.meta.resolve` gives it for that package's name

 @param body - module source run after `built`, the built package's namespace, is bound; it writes its answer with `process.stdout.write`

 @returns The child's standard output

 @throws SubprocessError when the child never started or exited with a failure

 @example
 ```ts
 const written = await runBuiltInChild({
   packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-glossary'),
   body: 'process.stdout.write(await built.resolveGit());',
 },);
 ```
 */
export async function runBuiltInChild(
  {
    packageEntry,
    body,
  }: {
    readonly packageEntry: string;
    readonly body: string;
  },
): Promise<string> {
  /**
   Directory the child runs in, so nothing it logs lands in the checkout.
   */
  await using scratch = await scratchDir({ prefix: 'built-child-', },);
  /**
   What the child wrote, a failed exit refusing instead.
   */
  const { stdout, } = await spawnKeyless({
    file: process.execPath,
    args: [
      '--input-type=module',
      '--eval',
      `const built = await import(${JSON.stringify(packageEntry,)});\n${body}`,
    ],
    cwd: scratch.path,
  },);
  return stdout;
}
