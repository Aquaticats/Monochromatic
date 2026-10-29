/**
 Tests for the environment a built command of this package gets from `runBuiltCommand` of the shared child fixture:
 the case's own variable wins over this process's, and a variable this process holds that the case does not name
 never reaches the command.

 THE PROOF IS A REAL CHILD. Each case points `TRANSLATION_REPAIR_RUNS_DIR` in this process at a scratch directory,
 runs `editor-standing-read` as built over another, and reads what it says about the directory it read. The command
 asks no model and never calls a provider. The suite runs one case at a time, since the variable is a process-wide
 write. The cases of the environment the fixture builds for any child are in `child-environment.unit.test.ts` of the
 adjudication package.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  runBuiltCommand,
  variablesSet,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: 'child environment',
  concurrency: 1,
  children: [
    it({
      name: 'RUNS A BUILT COMMAND with the case\'s variable winning over this process\'s and reports its exit '
        + 'code and both streams whole',
      fn: async () => {
        await using runs = await scratchDir({ prefix: 'child-environment-runs-', },);
        await using other = await scratchDir({ prefix: 'child-environment-other-', },);
        await mkdir(join(runs.path, 'artifacts',),);
        await writeFile(join(runs.path, 'artifacts', 'Mittens.json',), '{}', 'utf8',);
        using pointed = variablesSet({ values: { TRANSLATION_REPAIR_RUNS_DIR: other.path, }, },);
        void pointed;
        const run = await runBuiltCommand({
          command: 'editor-standing-read',
          packageEntry: BENCH_ENTRY,
          env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
        },);
        expect(run.code,).toBe(1,);
        expect(run.stderr,).toBe(
          `editor-standing-read: ${join(runs.path, 'artifacts', 'Mittens.json',)} refused, `
            + 'artifact parse failed at artifact.artifactSchemaVersion: expected a number.\n',
        );
        expect(run.stdout.split('\n',).at(0,),).toBe(
          'editor-standing-read: archives=0 artifacts=1 read=0 earlierRoster=0 earlierSchema=0 digestsJudged=0 '
            + 'digestsUnjudged=0 unjudgedDigestRounds=0',
        );
      },
    },),
    it({
      name: 'DROPS this process\'s runs directory from a built command that is given none',
      fn: async () => {
        await using runs = await scratchDir({ prefix: 'child-environment-runs-', },);
        await mkdir(join(runs.path, 'artifacts',),);
        await writeFile(join(runs.path, 'artifacts', 'Mittens.json',), '{}', 'utf8',);
        using pointed = variablesSet({ values: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, }, },);
        void pointed;
        const run = await runBuiltCommand({ command: 'editor-standing-read', packageEntry: BENCH_ENTRY, },);
        expect(run.stderr.includes(runs.path,),).toBe(false,);
      },
    },),
  ],
},);
