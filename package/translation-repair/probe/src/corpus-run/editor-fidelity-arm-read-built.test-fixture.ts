import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import type { ArmLog, } from '../../dist/final/node/index.mjs';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';

//region Editor fidelity read step as built
// The built `editor-fidelity-read` command run over arm logs written into an owned directory, each beside its exit file, for
// the cases that check what an operator would see. Cat-themed invention; no corpus content.

/**
 What a run of the built read step left behind.
 */
type BuiltRun = {
  /**
   Exit status, zero when every check held.
   */
  readonly code: number;

  /**
   Standard output, one entry per line.
   */
  readonly stdout: readonly string[];
};

/**
 Runs the built read step over logs written into an owned directory, each beside its exit file.

 @param arms - logs, in the order the arms ran

 @param prefix - name the owned directory starts with

 @returns Exit status and standard output

 @example
 ```ts
 const run = await readAsBuilt({ arms, prefix: 'editor-fidelity-passes-', },);
 ```
 */
export async function readAsBuilt(
  {
    arms,
    prefix,
  }: {
    readonly arms: readonly ArmLog[];
    readonly prefix: string;
  },
): Promise<BuiltRun> {
  /**
   The directory the logs are written into, removed when the run ends.
   */
  await using owned = await scratchDir({ prefix, },);
  await Promise.all(arms.flatMap(function filesOf(
    {
      arm,
      lines,
    },
  ): readonly Promise<void>[] {
    return [
      writeFile(
        join(
          owned.path,
          `${arm}.log`,
        ),
        `${lines.join('\n',)}\n`,
      ),
      writeFile(
        join(
          owned.path,
          `${arm}.exit`,
        ),
        '0\n',
      ),
    ];
  },),);
  /**
   What the built command did, whatever its exit code.
   */
  const run = await runBuiltCommand({
    command: 'editor-fidelity-read',
    packageEntry: PROBE_ENTRY,
    args: arms.map(function pathOf({ arm, },): string {
      return join(
        owned.path,
        `${arm}.log`,
      );
    },),
    cwd: owned.path,
    env: {},
  },);
  /**
   What the command printed to standard output.
   */
  const { stdout, } = run;
  /**
   The lines the command printed.
   */
  const printed = stdout.split('\n',);
  return {
    code: run.code,
    stdout: printed.filter(function present(line,): boolean {
      return line !== '';
    },),
  };
}

//endregion Editor fidelity read step as built
