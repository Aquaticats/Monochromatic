#!/usr/bin/env node
/**
 Move the installed wrapper to a released version and take one real commit sample with it, so the release's evidence is
 the released program and not a build of a worktree.

 This is what `native:install` runs, and it folds the coordinator's scratch script `~/temp/agent/upgrade-wrapper.ts` into
 the package. The steps are that script's: re-resolve the tool in `toolchain.lock.json`, print the lock diff, converge the
 installed tools, refuse unless `git --version` names the release asked for, commit the lock with the release's name, and
 print the newest timing line and the service journal of the sample commit.

 Three things are added to what the script did. Every step is bounded, because nothing here may wait forever; a lockfile
 that already names the release commits nothing and says so, instead of failing a commit with nothing to record; and
 `--dry-run` prints the same plan and changes nothing, so a person can read what an install would do first.

 What an install runs and the texts it reads are in `native-install-steps.ts`; this file is the command line and the order.
 */
import { join } from 'node:path';

import {
  deadlineMillisecondsOption,
  gateOptions,
} from './native-gate-arguments.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  printLine,
  runGateMain,
} from './native-gate-main.ts';
import {
  gateRepositoryOf,
  gitDeadlineMilliseconds,
  gitProgram,
  promptlessGitSettings,
} from './native-gate-paths.ts';
import {
  runGateCommand,
  runGateCommandOrThrow,
} from './native-gate-run.ts';
import {
  type InstallStep,
  commitTimingFileName,
  gitCommonDirectory,
  installSteps,
  journalIdentifiers,
  journalSince,
  lockFileName,
  newestTimingLine,
  readOrEmpty,
  stepNameOf,
  summaryLinesOf,
  timingReportLine,
  wrapperToolName,
  wrapperVersionLine,
} from './native-install-steps.ts';

/**
 Seconds one toolchain step is given when the caller names none.

 No recorded run of `toolchain:update` or `toolchain:install` was timed, so this bound is not a measurement: it is long
 enough for a download or a compile of one crate tool on a busy host, and `--deadline-seconds` overrides it for a host
 that needs longer. What it must not be is absent, because a step that never answers would hold the install.
 */
export const defaultToolchainStepDeadlineSeconds = 1_800;

/**
 Milliseconds the service journal of one sample commit is read in, which is a read of the local journal and answered in
 milliseconds on this host.
 */
const journalDeadlineMilliseconds = 60_000;

/**
 Milliseconds an install is counted from, before the sample commit, so the journal holds the commit's whole service work.
 */
const journalStartMarginMilliseconds = 2_000;

/**
 Option that names the checkout whose lock and Git directory the install writes.
 */
const repositoryOption = '--repository';

/**
 Option that prints what the install would do and changes nothing.
 */
const dryRunOption = '--dry-run';

/**
 Option that names how long one toolchain step may run.
 */
const deadlineOption = '--deadline-seconds';

/**
 How the command is called, printed by every refusal that reads the command line.
 */
const usage = 'usage: native:install [--repository <path>] [--deadline-seconds <count>] [--dry-run] <version>';

/**
 Runs one install step's command, bounded, and refuses with the whole step named and what it said.

 `runGateCommandOrThrow` names a command by its program and first argument, which for a task runner is the word `run`; a
 coordinator reading an install's refusal needs the task itself, so the step's own name is put in front of it.

 @param step - the step to run

 @param deadlineMilliseconds - how long it may run

 @returns what the step printed and how it ended

 @throws NativeGateError naming the whole step, when it could not run, ran past its deadline, or failed

 @example
 ```ts
 await runStepCommand({ step, deadlineMilliseconds: 1_800_000 });
 ```
 */
async function runStepCommand({
  step,
  deadlineMilliseconds,
}: {
  readonly step: InstallStep;
  readonly deadlineMilliseconds: number;
}): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string
}> {
  try {
    return await runGateCommandOrThrow({
      program: step.program,
      args: step.args,
      deadlineMilliseconds,
      environment: {
        ...process.env,
        ...promptlessGitSettings,
      },
      workingDirectory: step.repository,
    });
  }
  catch (error) {
    if (!(error instanceof NativeGateError))
      throw error;
    throw new NativeGateError(
      `the step ${stepNameOf({ step, })} did not succeed: ${error.message}`,
      { cause: error, }
    );
  }
}

/**
 Runs one install step, bounded, and prints the summary lines it wrote.

 @param step - the step to run

 @param deadlineMilliseconds - how long it may run

 @returns everything the step printed, on both streams

 @throws NativeGateError naming the step, when it could not run, ran past its deadline, or failed

 @example
 ```ts
 await runStep({ step, deadlineMilliseconds: 1_800_000 });
 ```
 */
async function runStep({
  step,
  deadlineMilliseconds,
}: {
  readonly step: InstallStep;
  readonly deadlineMilliseconds: number;
}): Promise<string> {
  /**
   What the step printed, which a failed step carries in its refusal.
   */
  const answered = await runStepCommand({
    step,
    deadlineMilliseconds,
  });
  printLine(`== ${stepNameOf({ step, })}: exit ${String(answered.status)}`);
  for (const line of summaryLinesOf({
    output: `${answered.stdout}\n${answered.stderr}`,
    markers: step.markers,
  }))
    printLine(line);
  return `${answered.stdout}\n${answered.stderr}`;
}

/**
 Runs one git query of the install, bounded and promptless.

 @param repository - the checkout to ask

 @param args - the arguments after the `-C <checkout>` pair

 @returns what git printed on standard output

 @throws NativeGateError when git cannot answer

 @example
 ```ts
 await askGit({ repository: '/repo', args: ['diff', '--', 'toolchain.lock.json'] });
 ```
 */
async function askGit({
  repository,
  args,
}: {
  readonly repository: string;
  readonly args: readonly string[];
}): Promise<string> {
  return (await runGateCommandOrThrow({
    program: gitProgram,
    args: [
      '-C',
      repository,
      ...args,
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: {
      ...process.env,
      ...promptlessGitSettings,
    },
  })).stdout;
}

/**
 Prints the wrapper's own record of the sample commit and the service journal it was answered in.

 @param repository - the checkout the sample commit was made in

 @param started - the moment the sample commit began

 @example
 ```ts
 await printSampleEvidence({ repository: '/repo', started: new Date() });
 ```
 */
async function printSampleEvidence({
  repository,
  started,
}: {
  readonly repository: string;
  readonly started: Date;
}): Promise<void> {
  /**
   The wrapper's own timing record of that commit.
   */
  const timing = newestTimingLine({
    text: await readOrEmpty({
      path: join(
        await gitCommonDirectory({ repository, }),
        commitTimingFileName
      ),
    }),
  });
  printLine(timingReportLine({ timing, }));
  /**
   The service journal of the sample commit, bounded, so a service that answered nothing says so.
   */
  const journal = await runGateCommand({
    program: 'journalctl',
    args: [
      '--user',
      `--since=${journalSince({ moment: new Date(started.getTime() - journalStartMarginMilliseconds), })}`,
      '--no-pager',
      '--output=short-iso-precise',
      ...journalIdentifiers.flatMap(function identifier(name) {
        return [
          '--identifier',
          name,
        ];
      }),
    ],
    deadlineMilliseconds: journalDeadlineMilliseconds,
  });
  printLine('== service journal since the sample commit began');
  printLine(journal.timedOut
    ? `the journal did not answer within ${String(journalDeadlineMilliseconds)} ms`
    : `${journal.stdout}${journal.stderr}`.trimEnd());
}

/**
 Installs one release and samples one commit with it.

 @returns zero once the release is installed and the sample taken

 @throws NativeGateError when the command line, a step or the installed version refuses

 @example
 ```ts
 process.exitCode = await runGateMain({ main });
 ```
 */
async function main(): Promise<number> {
  /**
   The command line, whose one positional is the release to move to.
   */
  const options = gateOptions({
    argv: process.argv
      .slice(2),
    knownOptions: [
      repositoryOption,
      deadlineOption,
    ],
    flagOptions: [dryRunOption,],
  });
  /**
   The release to install.
   */
  const [version, ...rest] = options.positionals;
  if ((version === undefined) || (version === ''))
    throw new NativeGateError(`native:install needs the release to install, such as 0.6.1-alpha.10. ${usage}`);
  if (rest.length > 0)
    throw new NativeGateError(`native:install takes one release and "${rest.join(' ')}" is a second, so which one to install is unknown. ${usage}`);
  /**
   The checkout the command line named, when it named one.
   */
  const namedRepository = options.values
    .get(repositoryOption);
  /**
   The checkout the install writes in, which the option or the folder tree names.
   */
  const repository = await gateRepositoryOf({
    environment: {},
    ...(namedRepository === undefined ? {} : { named: namedRepository }),
  });
  /**
   How long one toolchain step may run.
   */
  const deadlineMilliseconds = deadlineMillisecondsOption({
    options,
    name: deadlineOption,
    fallbackSeconds: defaultToolchainStepDeadlineSeconds,
  });
  /**
   The steps of the install, whose commit is only run when the lock moved.
   */
  const [updateStep, installStep, commitStep] = installSteps({
    repository,
    version,
  });
  /**
   Every step in the order it runs, which the dry run prints.
   */
  const plan: readonly InstallStep[] = [
    updateStep,
    installStep,
    commitStep,
  ];
  if (options.values
    .has(dryRunOption)) {
    printLine(`native:install would, in ${repository}:`);
    for (const step of plan)
      printLine(`  ${stepNameOf({ step, })}`);
    printLine(`  then refuse unless what ${gitProgram} --version prints holds ${wrapperVersionLine({ version, })}`);
    printLine(`  commit ${lockFileName} when it moved, and print the newest line of ${commitTimingFileName} beside the common Git directory`);
    printLine('  and the service journal of the sample commit');
    return 0;
  }
  /**
   What the re-resolution printed, whose lock diff is printed next.
   */
  const updated = await runStep({
    step: updateStep,
    deadlineMilliseconds,
  });
  if (updated.trim() === '')
    printLine(`== ${stepNameOf({ step: updateStep, })} printed nothing`);
  /**
   The lockfile's diff, printed whole, because it is the review surface of a released version (rule `UPN`).
   */
  const diff = await askGit({
    repository,
    args: [
      'diff',
      '--',
      lockFileName,
    ],
  });
  printLine(diff.trim() === '' ? `${lockFileName} did not change` : diff.trimEnd());
  await runStep({
    step: installStep,
    deadlineMilliseconds,
  });
  /**
   What the installed `git` says it is.
   */
  const reported = (await runGateCommandOrThrow({
    program: gitProgram,
    args: ['--version',],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: {
      ...process.env,
      ...promptlessGitSettings,
    },
    workingDirectory: repository,
  })).stdout;
  printLine(reported.trimEnd());
  if (!reported.includes(wrapperVersionLine({ version, }))) {
    throw new NativeGateError(`${gitProgram} --version does not name ${version}, so the installed wrapper is not the release that was asked for. What it printed: ${reported.trim() || '(nothing)'}. Check that ${wrapperToolName} resolves to ${version} (mise run toolchain:install, or mise which ${wrapperToolName.slice('cargo:'.length)}) and that the wrapper's directory comes before any other ${gitProgram} on PATH.`);
  }
  /**
   Whether the lock moved, which is whether there is a version to record.
   */
  const lockMoved = diff.trim() !== '';
  if (lockMoved) {
    /**
     The moment the sample commit began, which the journal is read from.
     */
    const started = new Date();
    /**
     How many milliseconds the commit took, which is the sample.
     */
    const startedMilliseconds = Date.now();
    await runStep({
      step: commitStep,
      deadlineMilliseconds,
    });
    printLine(`== the sample commit took ${String(Date.now() - startedMilliseconds)} ms`);
    await printSampleEvidence({
      repository,
      started,
    });
  }
  printLine(lockMoved
    ? `installed ${version} and committed the lock`
    : `installed ${version}; ${lockFileName} already named it, so nothing was committed`);
  return 0;
}

if (import.meta.main)
  process.exitCode = await runGateMain({ main, });
