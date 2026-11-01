#!/usr/bin/env node
/**
 Positive controls for the fuzz invariants.
 Plants one defect at a time in a disposable copy of the subject and requires the generator controls,
 which call the same invariant functions as the fuzz targets, to fail.
 The worktree is only read; every edit and build happens under the operating system's temporary directory.
 */
import {
  cp,
  mkdir,
  mkdtemp,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  NativeVerificationError,
  runCommand,
} from '../../cli/bin/native-verification-process.ts';
import {
  type Plant,
  plants,
} from './planted-table.ts';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
/**
 Parent of every scratch directory this script creates.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'git-policy-fuzz'
);
await mkdir(
  scratchBase,
  { recursive: true }
);

/**
 What the generator controls reported for one run; `status` is absent when a signal ended the run.
 */
type ControlsOutcome = {
  readonly status?: number;
  readonly failed: readonly string[];
  readonly passed: number;
  readonly output: string;
};

/**
 What the controls observed for one planted defect.
 */
type PlantObservation = {
  readonly name: string;
  readonly file: string;
  readonly edits: Plant['edits'];
  readonly status?: number;
  readonly failed: readonly string[];
  readonly noticed: boolean;
};

/**
 Repository-relative inputs the helper library needs to build.
 */
const inputs = [
  'package/git-policy/cli/Cargo.toml',
  'package/git-policy/cli/Cargo.lock',
  'package/git-policy/cli/README.md',
  'package/git-policy/cli/src/native',
  'package/git-policy/cli.fuzz/Cargo.toml',
  'package/git-policy/cli.fuzz/Cargo.lock',
  'package/git-policy/cli.fuzz/src',
  'package/git-policy/cli.fuzz/fuzz_targets',
  'package/jsonc-edit/core-rust/Cargo.toml',
  'package/jsonc-edit/core-rust/Cargo.lock',
  'package/jsonc-edit/core-rust/src',
  'package/jsonc-edit/core-rust/fixtures',
  // The subject links the scanner; `build.rs` and `data` hold the embedded baseline its library compiles in.
  'package/forbidden-strings/cli/Cargo.toml',
  'package/forbidden-strings/cli/Cargo.lock',
  'package/forbidden-strings/cli/build.rs',
  'package/forbidden-strings/cli/src',
  'package/forbidden-strings/cli/data',
  'package/forbidden-strings/core/Cargo.toml',
  'package/forbidden-strings/core/Cargo.lock',
  'package/forbidden-strings/core/src',
];

/**
 The nextest result lines of one kind.

 nextest prints `FAIL [ 0.012s] ( 3/38) <binary id> <test name>`, or `FAIL + LEAK [` when the failed test left a
 child process behind, and repeats each failure in its closing recap; the test name ends the line.

 @param output - run's combined output

 @param keyword - result kind

 @returns trimmed lines that start with the keyword and hold a `[`
 */
function resultLines({
  output,
  keyword,
}: {
  readonly output: string;
  readonly keyword: 'FAIL' | 'PASS';
}): string[] {
  return output.split('\n')
    .map(function trimmed(line) {
      return line.trim();
    })
    .filter(function isResult(line) {
      return line.startsWith(`${keyword} `) && line.includes('[');
    });
}

/**
 Run the generator controls in the disposable sidecar; report status and the failing control names.

 @param root - disposable copy of the repository inputs

 @returns exit status,
   the distinct failing test names, the number of passing tests and the combined output
 */
async function controls({ root, }: {
  readonly root: string;
},): Promise<ControlsOutcome> {
  /**
   The nextest run of the generator controls, whose failure is evidence the caller inspects.
   */
  const run = await runCommand({
    command: 'cargo',
    args: [
      'nextest',
      'run',
      '--lib',
      '--offline',
      '--locked',
    ],
    capture: true,
    allowFailure: true,
    cwd: join(
      root,
      'package/git-policy/cli.fuzz',
    ),
    env: {
      ...process.env,
      CARGO_TARGET_DIR: join(
        root,
        'target',
      ),
    },
  });
  /**
   Both streams of the run.
   */
  const output = `${run.stdout}\n${run.stderr}`;
  /**
   Name of every failure line's test, repeated where nextest repeats a failure in its recap.
   */
  const failedNames = resultLines({
    output,
    keyword: 'FAIL',
  })
    .map(function testName(line) {
      return line.slice(line.lastIndexOf(' ') + 1);
    });
  return {
    ...(run.status === undefined ? {} : { status: run.status, }),
    failed: [...new Set(failedNames)],
    passed: resultLines({
      output,
      keyword: 'PASS',
    })
      .length,
    output,
  };
}

/**
 Apply one plant's edits to its file text, refusing an edit that does not match exactly once.

 @param plant - plant

 @param original - its file's text

 @returns the edited text

 @throws NativeVerificationError when an edit matches anything but exactly once
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
    function applyEdit(
      text,
      edit
    ) {
      /**
       How many times the edit's text occurs, which must be exactly once.
       */
      const occurrences = text.split(edit.from)
        .length
        - 1;
      if (occurrences !== 1)
        throw new NativeVerificationError(`Planted control "${plant.name}" matches ${occurrences} times in ${plant.file}; it must match exactly once.`);
      return text.replace(
        edit.from,
        edit.to,
      );
    },
    original,
    );
}

/**
 Plant one defect, run the controls, and put the file back.

 @param root - disposable copy

 @param evidence - evidence directory

 @param plant - defect

 @param index - defect's position, which names its log

 @returns what the controls observed
 */
async function runPlant({
  root,
  evidence,
  plant,
  index,
}: {
  readonly root: string;
  readonly evidence: string;
  readonly plant: Plant;
  readonly index: number;
}): Promise<PlantObservation> {
  /**
   The planted file inside the disposable copy.
   */
  const path = join(
    root,
    plant.file,
  );
  /**
   The file's text before the plant, written back afterwards.
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
   What the generator controls reported with the defect planted.
   */
  const run = await controls({ root });
  await writeFile(
    path,
    original,
  );
  // A compile error is not an observation: the plant must build and a named control must fail.
  /**
   Whether the controls failed and named at least one failing test.
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
    ...(run.status === undefined ? {} : { status: run.status, }),
    failed: run.failed,
    noticed,
  };
}

/**
 Plant each defect in table order. Each one starts after the previous one returned and restored its file,
 because every plant edits the same disposable copy.

 @param root - disposable copy

 @param evidence - evidence directory

 @returns one observation per defect
 */
async function* observations({
  root,
  evidence,
}: {
  readonly root: string;
  readonly evidence: string;
}): AsyncGenerator<PlantObservation> {
  for (const [
    index,
    plant,
  ] of plants.entries()) {
    yield runPlant({
      root,
      evidence,
      plant,
      index,
    });
  }
}

// Copy the inputs, prove the unplanted controls pass, then require every plant to fail a control.
/**
 The repository root, which the inputs are copied from.
 */
const repository = resolve(
  import.meta.dirname,
  '../../../..',
);
/**
 The disposable copy of the repository inputs that every plant edits.
 */
await using root = await mkdtempDisposable(join(
  scratchBase,
  'cli-git-fuzz-planted-',
));
/**
 Directory under the package's `target/` that keeps the evidence of every run.
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
 Evidence directory of this run.
 */
const evidence = await mkdtemp(join(
  evidenceRoot,
  'planted-',
));
await Promise.all(inputs.map(function copyInput(input) {
  return cp(
    join(
      repository,
      input,
    ),
    join(
      root.path,
      input,
    ),
    { recursive: true },
  );
}));
/**
 The controls on the unplanted copy, which must pass for a plant's failure to mean anything.
 */
const baseline = await controls({ root: root.path });
if ((baseline.status !== 0) || (baseline.passed === 0)) {
  await writeFile(
    join(
      evidence,
      'baseline.log',
    ),
    baseline.output,
  );
  throw new NativeVerificationError(`Unplanted generator controls did not pass; inspect ${evidence}.`);
}
console.log(`Unplanted controls: ${baseline.passed} passed`);
/**
 One observation per planted defect.
 */
const results: PlantObservation[] = [];
for await (const result of observations({
  root: root.path,
  evidence,
}))
  results.push(result);
await writeFile(
  join(
    evidence,
    'results.json',
  ),
  `${JSON.stringify(
    {
      baselinePassed: baseline.passed,
      results,
    },
    null,
    2,
  )}\n`,
);
console.log(`Planted-control evidence: ${evidence}`);
/**
 Defects that no control noticed.
 */
const unnoticed = results.filter(function isUnnoticed(result) {
  return !result.noticed;
});
if (unnoticed.length > 0) {
  throw new NativeVerificationError(`Planted defects were not noticed: ${unnoticed.map(function nameOf(result) {
    return result.name;
  })
    .join('; ')}.`);
}
