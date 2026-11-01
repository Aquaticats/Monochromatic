#!/usr/bin/env node
/**
 Positive controls for the fuzz invariants.

 Plants one defect at a time in a copy of the linter and requires the generator controls, which call the same
 invariant functions as the fuzz targets, to fail.
 The worktree is only read; every edit and build happens in this package's ignored `target/planted` tree.
 */
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import { runCommand } from '../../../git-policy/cli/bin/native-verification-process.ts';
import {
  failedControls,
  passedControls,
} from './nextest-output.ts';
import {
  inputs,
  type Plant,
  plants,
} from './planted-definitions.ts';

/**
 Preserve the failed verification boundary rather than masking it with a cleanup error.
 */
class PlantedControlError extends Error {
  /**
   Stable class name that tells a failed planted control apart from an unrelated error in a log.
   */
  override name = 'PlantedControlError';
}

/**
 The generator controls' result in one copy of the linter.
 */
type Controls = {
  readonly status?: number;
  readonly failed: readonly string[];
  readonly passed: number;
  readonly output: string;
};

/**
 A plant with its position.
 */
type IndexedPlant = {
  readonly plant: Plant;
  readonly index: number;
};

/**
 What one plant showed; the fields appear in the results file in this order.
 */
type PlantResult = {
  readonly name: string;
  readonly file: string;
  readonly edits: Plant['edits'];
  readonly status?: number;
  readonly failed: readonly string[];
  readonly noticed: boolean;
};

/**
 Run the generator controls in the copied sidecar; report status and the failing control names.

 @param root - the copied tree

 @param targetDirectory - Cargo's target directory, kept between runs so dependency builds are reused

 @returns the exit status, the failing and passing controls, and the complete output

 @throws NativeVerificationError when Cargo cannot start

 @example
 ```ts
 const outcome = await controls({ root, targetDirectory });
 ```
 */
async function controls({
  root,
  targetDirectory,
}: {
  readonly root: string;
  readonly targetDirectory: string;
}): Promise<Controls> {
  /**
   What the nextest run left behind.
   */
  const outcome = await runCommand({
    command: 'cargo',
    args: [
      'nextest',
      'run',
      '--lib',
      '--offline',
      '--locked',
    ],
    cwd: join(
      root,
      'package/linter/monochromatic-lint.fuzz',
    ),
    env: {
      ...process.env,
      CARGO_TARGET_DIR: targetDirectory,
      CARGO_BUILD_JOBS: '2',
    },
    capture: true,
    allowFailure: true,
  });
  /**
   Standard output and error together.
   */
  const output = `${outcome.stdout}\n${outcome.stderr}`;
  return {
    ...(outcome.status === undefined ? {} : { status: outcome.status }),
    failed: failedControls(output),
    passed: passedControls(output),
    output,
  };
}

/**
 Apply one plant's edits to its file text, refusing an edit that does not match exactly once.

 @param plant - the defect to plant

 @param original - the text of the plant's file

 @returns the text with the defect

 @throws PlantedControlError when an edit does not match exactly once

 @example
 ```ts
 const text = planted({ plant, original });
 ```
 */
function planted({
  plant,
  original,
}: {
  readonly plant: Plant;
  readonly original: string;
}): string {
  return plant.edits
    .reduce(
    function applied(
      text,
      edit
    ) {
        /**
         How many times the edit's text occurs.
         */
      const occurrences = text.split(edit.from)
        .length
        - 1;
      if (occurrences !== 1)
        throw new PlantedControlError(`Planted control "${plant.name}" matches ${occurrences} times in ${plant.file}; it must match exactly once.`);
      return text.replace(
        edit.from,
        edit.to,
      );
    },
    original,
    );
}

/**
 Plant one defect, run the controls and restore the file.

 @param plant - the defect to plant

 @param index - the position of the plant, which names its log

 @param root - the copied tree

 @param targetDirectory - Cargo's target directory

 @param evidence - the evidence directory

 @returns what the plant showed

 @throws PlantedControlError when an edit does not match exactly once

 @example
 ```ts
 const result = await plantOne({ plant, index: 0, root, targetDirectory, evidence });
 ```
 */
async function plantOne({
  plant,
  index,
  root,
  targetDirectory,
  evidence,
}: {
  readonly plant: Plant;
  readonly index: number;
  readonly root: string;
  readonly targetDirectory: string;
  readonly evidence: string;
}): Promise<PlantResult> {
  /**
   Location of the plant's file in the copied tree.
   */
  const path = join(
    root,
    plant.file,
  );
  /**
   The file before the defect.
   */
  const original = await readFile(
    path,
    'utf8',
  );
  await writeFile(
    path,
    planted({
      plant,
      original,
    }),
  );
  /**
   What the controls showed with the defect in place.
   */
  const run = await controls({
    root,
    targetDirectory,
  });
  await writeFile(
    path,
    original,
  );
  // A compile error is not an observation: the plant must build and a named control must fail.
  /**
   Whether the defect made a named control fail.
   */
  const noticed = (run.status !== 0) && (run.failed
    .length
    > 0);
  await writeFile(
    join(
      evidence,
      `planted-${index}.log`,
    ),
    run.output,
  );
  console.log(`Planted defect "${plant.name}": ${noticed ? `noticed by ${run.failed
    .join(', ')}` : 'NOT noticed'}`);
  return {
    name: plant.name,
    file: plant.file,
    edits: plant.edits,
    ...(run.status === undefined ? {} : { status: run.status }),
    failed: run.failed,
    noticed,
  };
}

/**
 Copy the inputs, prove the unplanted controls pass, then require every plant to fail a control.

 @throws PlantedControlError when the unplanted controls fail or a plant is not noticed

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Root of the repository.
   */
  const repository = resolve(
    import.meta.dirname,
    '../../../..',
  );
  // A fixed copy location keeps Cargo's dependency builds reusable between runs of this task.
  /**
   Directory of the copied tree and of Cargo's target directory.
   */
  const work = join(
    process.cwd(),
    'target',
    'planted',
  );
  /**
   The copied tree.
   */
  const root = join(
    work,
    'tree',
  );
  /**
   Cargo's target directory.
   */
  const targetDirectory = join(
    work,
    'target',
  );
  await rm(
    root,
    {
      recursive: true,
      force: true,
    },
  );
  /**
   Directory that holds one evidence directory per run.
   */
  const evidenceRoot = join(
    process.cwd(),
    'target',
    'verification',
  );
  await mkdir(
    evidenceRoot,
    { recursive: true },
  );
  /**
   The evidence directory of this run.
   */
  const evidence = await mkdtemp(join(
    evidenceRoot,
    'planted-',
  ));
  /**
   Removal of the copied tree when the run ends.
   */
  await using cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
  void cleanup;
  await mapInSequence({
    items: inputs,
    run: function copyOf(input) {
      return cp(
        join(
          repository,
          input,
        ),
        join(
          root,
          input,
        ),
        { recursive: true },
      );
    },
  });
  /**
   The controls without any defect.
   */
  const baseline = await controls({
    root,
    targetDirectory,
  });
  await writeFile(
    join(
      evidence,
      'baseline.log',
    ),
    baseline.output,
  );
  if ((baseline.status !== 0) || (baseline.passed === 0))
    throw new PlantedControlError(`Unplanted generator controls did not pass; inspect ${evidence}.`);
  console.log(`Unplanted controls: ${baseline.passed} passed`);
  /**
   What each plant showed.
   */
  const results = await mapInSequence({
    items: plants.map(function indexed(
      plant,
      index
    ): IndexedPlant {
      return {
        plant,
        index,
      };
    }),
    run: function plantOf(item) {
      return plantOne({
        plant: item.plant,
        index: item.index,
        root,
        targetDirectory,
        evidence,
      });
    },
  });
  await writeFile(
    join(
      evidence,
      'results.json',
    ),
    `${JSON.stringify(
      {
        baselinePassed: baseline.passed,
        results: results.map(function recordOf(result) {
          return {
            name: result.name,
            file: result.file,
            edits: result.edits,
            status: result.status ?? null,
            failed: result.failed,
            noticed: result.noticed,
          };
        }),
      },
      null,
      2,
    )}\n`,
  );
  console.log(`Planted-control evidence: ${evidence}`);
  /**
   The plants no control noticed.
   */
  const unnoticed = results.filter(function isUnnoticed(result) {
    return !result.noticed;
  });
  if (unnoticed.length > 0)
    throw new PlantedControlError(`Planted defects were not noticed: ${unnoticed.map(function nameOf(result) {
      return result.name;
    })
      .join('; ')}.`);
}

await main();
