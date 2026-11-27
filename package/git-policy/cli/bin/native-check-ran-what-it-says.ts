/**
 The line a check that passed prints to say what it ran, and the refusal when its own output cannot say.

 This repository counts a green test run only when its output holds the expected suite and test names (`AGENTS.md`,
 rule `TRV`): a suite that is never awaited exits 0 having run nothing. `bin/native-ts-checks.ts` keeps a check's own
 lines only when it failed, so a passing check printed its status and nothing else, and a run that executed no test read
 the same as one that ran them all (issue 6613).

 What one `[PASS]` line stands for is the harness's own answer. Every named `describe` logs one `info` line when its
 children all passed, carrying the `PASS` tag and the names of the children it ran
 (`package/test-harness/core/src/describe.ts`, `runOnce`; the tag is `createVerdictLoggers` of
 `package/test-harness/core/src/verdict.ts`). A per-test `PASS` is logged at `debug`, which the console sink writes to
 standard error and hides unless `MONOCHROMATIC_VERBOSE` is set, so a line this module counts stands for one suite and
 names the tests that suite held, never for one test and never for one file: a file holds one or more suites.

 The count of unit test files a package holds is the harness's own discovery (`mise.toml`,
 `[task_templates."test:unit"]`): `**\/*.unit.test.ts` below the package, never a file under `node_modules`, and never a
 `*.expensive.*` file, which a run counts only when it is given `--all`.
 */
import { glob, } from 'node:fs/promises';
import { join, } from 'node:path';

import { countedText, } from './native-check-scope-bound-lines.ts';

/**
 The files a run of a package's unit tests must report on, which is the pattern the harness discovers with.
 */
export const unitTestFilePattern = '**/*.unit.test.ts';

/**
 Part of a path whose files the harness never runs.
 */
const nodeModulesPart = 'node_modules';

/**
 Part of a file name that marks a test the harness runs only when it is given `--all`, which this check never passes.
 */
const expensiveTestNamePart = '.expensive.';

/**
 The name of the task that runs a package's unit tests.
 */
const unitTestTaskName = 'test:unit';

/**
 The name of the task that type-checks a package.
 */
const typeCheckTaskName = 'lint:types';

/**
 The words a suite line opens with: the console sink's `info` level, then the stamp's own bracket.
 */
const suiteLineOpens = '[info] [';

/**
 The stamp's closing bracket and the space that opens the record's message.
 */
const stampCloses = '] ';

/**
 The tag the harness wraps a suite's pass in, which stands first in that suite's message.
 */
const passTag = ' [PASS] ';

/**
 What a check's own output says it ran, and whether the run counts as passed.

 An empty line list is the answer for a check this module has nothing to add about, which is every check but a package's
 `test:unit` and its `lint:types`.
 */
export type RanLines = {
  /**
   The lines the check prints under its status.
   */
  readonly lines: readonly string[];
  /**
   Whether the output cannot say what the check ran, which makes the run one that does not count as passed.
   */
  readonly refused: boolean;
};

/**
 A package's own unit tests, which must report on every unit test file that package holds.
 */
type PackageUnitTestsRan = {
  /**
   Marks the unit tests of one package.
   */
  readonly kind: 'packageUnitTests';
  /**
   The package's directory, which the harness's pattern is read below.
   */
  readonly packageDirectory: string;
};

/**
 A `test:unit` task that names no package, whose file count this module cannot derive.
 */
type UnitTestsOfNoPackageRan = {
  /**
   Marks a unit test run of no one package.
   */
  readonly kind: 'unitTestsOfNoPackage';
};

/**
 A type check, whose tool reports no count of the files it checked in this run.
 */
type TypeCheckRan = {
  /**
   Marks a type check.
   */
  readonly kind: 'typeCheck';
};

/**
 A check whose status alone is all its output can say.
 */
type UnstatedRan = {
  /**
   Marks a check this module adds nothing about.
   */
  readonly kind: 'unstated';
};

/**
 What one check's output has to say, read from the task's own name.

 The task names the package whose files it runs, so the count a unit test run is held against belongs to that package and
 never to a number written down here.
 */
type RanExpectation = PackageUnitTestsRan | UnitTestsOfNoPackageRan | TypeCheckRan | UnstatedRan;

/**
 One task name's path and task name, split where mise writes the colon.

 @param task - the task as the command line named it

 @returns the path inside the checkout, empty for the repository's own tasks, and the task name

 @example
 ```ts
 taskPartsOf({ task: '//package/a:test:unit' });
 ```
 */
function taskPartsOf({ task }: { readonly task: string; }): {
  readonly path: string;
  readonly name: string;
} {
  /**
   The words that open a path inside the checkout.
   */
  const checkoutPrefix = '//';
  if (!task.startsWith(checkoutPrefix))
    return {
      path: '',
      name: task,
    };
  /**
   Where the path ends and the task name begins.
   */
  const separator = task.indexOf(
    ':',
    checkoutPrefix.length,
  );
  if ((separator === (-1)))
    return {
      path: task.slice(checkoutPrefix.length,),
      name: '',
    };
  return {
    path: task.slice(
      checkoutPrefix.length,
      separator,
    ),
    name: task.slice(separator + 1,),
  };
}

/**
 What a check's output has to say, read from the task's name and the checkout the check runs in.

 @param task - the check, as the command line named it

 @param worktree - the checkout the check runs in

 @returns what the check's output has to say

 @example
 ```ts
 expectationOfTask({ task: '//package/a:test:unit', worktree: '/repo' });
 ```
 */
function expectationOfTask({
  task,
  worktree,
}: {
  readonly task: string;
  readonly worktree: string;
},): RanExpectation {
  /**
   The task's own path and name.
   */
  const parts = taskPartsOf({ task, });
  if (parts.name === typeCheckTaskName)
    return { kind: 'typeCheck', };
  if (parts.name !== unitTestTaskName)
    return { kind: 'unstated', };
  if (parts.path === '')
    return { kind: 'unitTestsOfNoPackage', };
  return {
    kind: 'packageUnitTests',
    packageDirectory: join(
      worktree,
      parts.path,
    ),
  };
}

/**
 The unit test files a package holds, counted the way the harness discovers them.

 @param packageDirectory - the package's directory

 @returns the files' paths relative to the package, in code unit order

 @example
 ```ts
 await unitTestFilesOf({ packageDirectory: '/repo/package/a' });
 ```
 */
export async function unitTestFilesOf({
  packageDirectory,
}: {
  readonly packageDirectory: string;
},): Promise<readonly string[]> {
  /**
   The files the harness would run, which a file under `node_modules` and an expensive file are not.
   */
  const files: string[] = [];
  for await (const file of glob(
    unitTestFilePattern,
    {
      cwd: packageDirectory,
      exclude: function outsideNodeModules(name) {
        return name.includes(nodeModulesPart);
      },
    },
  )) {
    if (!file.includes(expensiveTestNamePart))
      files.push(file,);
  }
  return files.toSorted();
}

/**
 How many suites one check's output says reported `[PASS]`.

 A suite line is the console sink's `info` record whose message opens with the `PASS` tag: `formatRecord` writes the
 level and the ISO stamp (`package/logger/core/src/sink/console.ts:214`), and a named suite's pass is one `info` record
 (`package/test-harness/core/src/describe.ts`, `runOnce`; `createVerdictLoggers` in
 `package/test-harness/core/src/verdict.ts`). A per-test `PASS` is logged at `debug`, which the sink writes to standard
 error and hides, so none of those is counted here.

 @param output - everything the check printed, on both streams

 @returns the number of suite lines holding the harness's `PASS`

 @example
 ```ts
 suitePassCountOf({ output: '[info] [2026-11-27T17:18:33.609Z] [a] [PASS] b (13ms)\n' });
 ```
 */
export function suitePassCountOf({ output }: { readonly output: string; },): number {
  /**
   How many suite lines the output held so far.
   */
  let count = 0;
  for (const line of output.split('\n',)) {
    if (!line.startsWith(suiteLineOpens))
      continue;
    /**
     Where the stamp ends and the record's message begins.
     */
    const messageStarts = line.indexOf(
      stampCloses,
      suiteLineOpens.length,
    );
    if ((messageStarts === (-1)))
      continue;
    if (line.includes(
      passTag,
      messageStarts,
    ))
      count += 1;
  }
  return count;
}

/**
 The lines a check that passed prints when its output cannot say what it ran.

 @param lines - the lines the refusal prints

 @returns the lines, refused

 @example
 ```ts
 refusalOf({ lines: ['//package/a:b: exit 0, but its output holds no [PASS] suite line'] });
 ```
 */
function refusalOf({ lines }: { readonly lines: readonly string[]; },): RanLines {
  return {
    lines,
    refused: true,
  };
}

/**
 The lines a passing type check prints: what the tool reports about the files it checked.

 `tsc --build` prints one line when it passes, the `TSFILE: <path>` record that `listEmittedFiles` asks for
 (`package/config/typescript/tsconfig.options.json:132`), which names the build-info file it wrote and says nothing about
 the files it read. The count it can report is asked for with `--extendedDiagnostics`, which the repository's `lint:types`
 task does not pass; measured on 2026-11-27 in this package, `tsc --build --force --extendedDiagnostics` printed
 `Aggregate Files: 755` beside its `TSFILE:` line, and the same command without the flag printed the `TSFILE:` line alone.

 @param task - the check, as the command line named it

 @returns the lines that say what the type checker does and does not report

 @example
 ```ts
 typeCheckCountLines({ task: '//package/a:lint:types' });
 ```
 */
function typeCheckCountLines({ task }: { readonly task: string; },): readonly string[] {
  return [
    `${task}: the type checker reports no count of the files it checked`,
    `${task}: a build that passes prints one TSFILE line, which listEmittedFiles asks for, naming the build-info file it wrote`,
    `${task}: --extendedDiagnostics is the flag that reports the count as Aggregate Files, and lint:types passes none`,
  ];
}

/**
 The lines a passing `test:unit` of one package prints, and the refusal when its output reports too few suites.

 @param task - the check, as the command line named it

 @param output - everything the check printed, on both streams

 @param packageDirectory - the package's directory

 @returns the counts, or the refusal that names the count reported and the count expected

 @example
 ```ts
 await packageUnitTestLines({ task: '//package/a:test:unit', output, packageDirectory: '/repo/package/a' });
 ```
 */
async function packageUnitTestLines({
  task,
  output,
  packageDirectory,
}: {
  readonly task: string;
  readonly output: string;
  readonly packageDirectory: string;
},): Promise<RanLines> {
  /**
   The unit test files the package holds.
   */
  const files = await unitTestFilesOf({ packageDirectory, });
  /**
   The suites the check's own output says reported.
   */
  const suites = suitePassCountOf({ output, });
  /**
   How many files the package holds, said as a sentence needs it.
   */
  const expected = countedText({
    count: files.length,
    singular: 'unit test file',
    plural: 'unit test files',
  },);
  if (files.length === 0)
    return refusalOf({
      lines: [
        `${task}: exit 0, but the package holds no unit test file, so this check ran no test`,
        `${task}: this run does not count as passed`,
      ],
    },);
  /**
   What a refusal says a real run would have reported, which is the count the harness prints for one.
   */
  const expectation = `${task}: the harness prints a [PASS] line for each suite it runs, so a real run of ${expected} reports at least ${String(files.length)}, and this run does not count as passed`;
  if (suites === 0)
    return refusalOf({
      lines: [
        `${task}: exit 0, but its output holds no [PASS] suite line, where the package holds ${expected}`,
        expectation,
      ],
    },);
  if (suites < files.length)
    return refusalOf({
      lines: [
        `${task}: exit 0, but ${countedText({
          count: suites,
          singular: 'suite reported',
          plural: 'suites reported',
        },)} [PASS], where the package holds ${expected}`,
        expectation,
      ],
    },);
  return {
    lines: [`${task}: ${expected}, and ${countedText({
      count: suites,
      singular: 'suite',
      plural: 'suites',
    },)} reported [PASS]`,],
    refused: false,
  };
}

/**
 The lines a passing `test:unit` that names no package prints: the count of suites, and the file count this module cannot
 derive for it.

 @param task - the check, as the command line named it

 @param output - everything the check printed, on both streams

 @returns the count of suites, or the refusal that nothing reported at all

 @example
 ```ts
 unitTestsOfNoPackageLines({ task: '//:test:unit', output });
 ```
 */
function unitTestsOfNoPackageLines({
  task,
  output,
}: {
  readonly task: string;
  readonly output: string;
},): RanLines {
  /**
   The suites the check's own output says reported.
   */
  const suites = suitePassCountOf({ output, });
  if (suites === 0)
    return refusalOf({
      lines: [
        `${task}: exit 0, but its output holds no [PASS] suite line, so it checked nothing`,
        `${task}: this run does not count as passed`,
      ],
    },);
  return {
    lines: [`${task}: ${countedText({
      count: suites,
      singular: 'suite reported',
      plural: 'suites reported',
    },)} [PASS], and no count of unit test files is derived for a task that names no package`,],
    refused: false,
  };
}

/**
 The lines a check that passed prints to say what it ran, and the refusal when its own output cannot say.

 A check that failed keeps exactly what it printed before (`bin/native-ts-checks.ts` never asks this module about one),
 and a check this module knows nothing to ask of answers no lines.

 @param task - the check, as the command line named it

 @param output - everything the check printed, on both streams

 @param worktree - the checkout the check ran in

 @returns the lines to print under the check's status, and whether the run counts as passed

 @example
 ```ts
 const ran = await ranWhatItSaysOf({ task: '//package/a:test:unit', output, worktree: '/repo' });
 ```
 */
export async function ranWhatItSaysOf({
  task,
  output,
  worktree,
}: {
  readonly task: string;
  readonly output: string;
  readonly worktree: string;
},): Promise<RanLines> {
  /**
   What this check's output has to say, read from the task's own name.
   */
  const expectation = expectationOfTask({
    task,
    worktree,
  },);
  if (expectation.kind === 'unstated')
    return {
      lines: [],
      refused: false,
    };
  if (expectation.kind === 'typeCheck')
    return {
      lines: typeCheckCountLines({ task, }),
      refused: false,
    };
  if (expectation.kind === 'unitTestsOfNoPackage')
    return unitTestsOfNoPackageLines({
      task,
      output,
    },);
  return await packageUnitTestLines({
    task,
    output,
    packageDirectory: expectation.packageDirectory,
  },);
}
