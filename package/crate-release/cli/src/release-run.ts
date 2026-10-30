import {
  join,
  posix,
} from 'node:path';

import {
  failureExitCode,
  manifestFileName,
} from './constants.ts';
import { formatCommandLine } from './command-line.ts';
import {
  type ManifestEdit,
  replaceDependencyPin,
  replacePackageVersion,
} from './manifest-edit.ts';
import type { ReleaseOptions } from './release-options.ts';
import {
  buildReleaseSteps,
  type ReleaseStep,
} from './release-steps.ts';

/**
 Runs one step and reports its exit status; a status other than zero is a failure the caller reports.
 */
export type ReleaseStepRunner = (step: ReleaseStep) => Promise<{ readonly status: number }>;

/**
 A file's new text, ready to be written.
 */
type PlannedWrite = {
  readonly path: string;
  readonly text: string;
};

/**
 A manifest edit with the file it belongs to.
 */
type PlannedEdit = {
  readonly path: string;
  readonly edit: ManifestEdit;
};

/**
 Compute every manifest edit of a release before any file is written, so a refused manifest leaves all files as they
 were.

 @param options - validated release options

 @param readText - reads a file by absolute path

 @param print - prints one line of progress

 @returns the files whose text changes; files that already hold the target are reported and left out

 @throws CrateReleaseError when a manifest does not hold the current version or an exact pin of it

 @example
 ```ts
 await planEdits({ options, readText, print });
 ```
 */
async function planEdits({
  options,
  readText,
  print,
}: {
  readonly options: ReleaseOptions;
  readonly readText: (path: string) => Promise<string>;
  readonly print: (line: string) => void;
}): Promise<readonly PlannedWrite[]> {
  /**
   Absolute path of the crate manifest.
   */
  const cratePath = join(
    options.root,
    options.crateDirectory,
    manifestFileName
  );
  /**
   Edits in the order they are reported: the crate first, then each pin file.
   */
  const edits: readonly PlannedEdit[] = [
    {
      path: cratePath,
      edit: replacePackageVersion({
        text: await readText(cratePath),
        from: options.from,
        to: options.to
      }),
    },
    ...await Promise.all(options.pinFiles
      .map(async function planPin(pinFile): Promise<PlannedEdit> {
      /**
       Absolute path of the pin file.
       */
        const path = join(
          options.root,
          ...pinFile.split(posix.sep)
        );
        return {
          path,
          edit: replaceDependencyPin({
            text: await readText(path),
            crateName: options.crateName,
            from: options.from,
            to: options.to,
          }),
        };
      })),
  ];
  for (const {
    path,
    edit
  } of edits)
    print(edit.changed ? `edited ${path}: ${options.from} to ${options.to}` : `${path} already holds ${options.to}`);
  return edits
    .filter(function isChanged({ edit }) {
      return edit.changed;
    })
    .map(function toWrite({
      path,
      edit,
    }): PlannedWrite {
      return {
        path,
        text: edit.text
      };
    });
}

/**
 Run the steps in order and stop at the first that fails.

 The steps are a short fixed chain (check, lock, dry run, commit), each depending on the one before, so the recursion is
 as deep as the plan and no deeper.

 @param steps - steps still to run

 @param runStep - runs one program call and returns its exit status

 @param print - prints one line

 @returns zero when every step succeeded, the failure status after the first failing step

 @example
 ```ts
 await runSteps({ steps, runStep, print });
 ```
 */
async function runSteps({
  steps,
  runStep,
  print,
}: {
  readonly steps: readonly ReleaseStep[];
  readonly runStep: ReleaseStepRunner;
  readonly print: (line: string) => void;
}): Promise<number> {
  /**
   The next step and the ones after it.
   */
  const [step, ...rest] = steps;
  if (step === undefined)
    return 0;
  print(`== ${step.label}`);
  print(`$ ${formatCommandLine({
    program: step.program,
    args: step.args
  })}`);
  /**
   How the step ended.
   */
  const outcome = await runStep(step);
  if (outcome.status !== 0) {
    print(`FAILED ${step.label} (${String(outcome.status)})`);
    return failureExitCode;
  }
  return await runSteps({
    steps: rest,
    runStep,
    print
  });
}

/**
 Run a release: edit the manifests, then run each step until one fails.

 Every edit is computed before the first file is written, a manifest that already holds the target is not rewritten,
 and the first failing step stops the run with its label and status printed. Nothing is published; the release ends in a
 local commit.

 @param options - validated release options

 @param readText - reads a file by absolute path

 @param writeText - writes a file by absolute path

 @param runStep - runs one program call and returns its exit status

 @param print - prints one line to the person running the release

 @returns zero after the commit, the failure status when a step failed

 @throws CrateReleaseError when a manifest does not hold the current version or an exact pin of it

 @example
 ```ts
 await runRelease({ options, readText, writeText, runStep, print: console.log });
 ```
 */
export async function runRelease({
  options,
  readText,
  writeText,
  runStep,
  print,
}: {
  readonly options: ReleaseOptions;
  readonly readText: (path: string) => Promise<string>;
  readonly writeText: (input: {
    readonly path: string;
    readonly text: string
  }) => Promise<void>;
  readonly runStep: ReleaseStepRunner;
  readonly print: (line: string) => void;
}): Promise<number> {
  await Promise.all((await planEdits({
    options,
    readText,
    print
  })).map(writeText));
  /**
   Exit status of the steps.
   */
  const status = await runSteps({
    steps: buildReleaseSteps(options),
    runStep,
    print
  });
  if (status === 0)
    print('RELEASE COMMITTED');
  return status;
}
