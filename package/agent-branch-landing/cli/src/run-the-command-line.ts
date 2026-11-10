/**
 What each subcommand does with what it was asked, and the status it ends with.

 @module
 */

import {
  type Bounds,
  parseCommandLine,
  parseSideToKeep,
} from './command-line.ts';
import {
  checkABranch,
  checkPassed,
} from './check-a-branch.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  GIT_OUTPUT_BYTES,
  FAILED_STATUS,
  MISUSED_STATUS,
  SUCCESS_STATUS,
  worktreeFolderOf,
} from './constants.ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  mainCheckoutOf,
  type Repository,
} from './git-commands.ts';
import {
  landAList,
  listLandedCompletely,
  worktreeRepositoryOf,
} from './land-a-list.ts';
import {
  landedCompletely,
  landABranch,
} from './land-a-branch.ts';
import { entriesOfFile, } from './landing-list.ts';
import { commitTheOpenMerge, } from './open-merge.ts';
import {
  reportOnAConflict,
  settleConflicts,
} from './resolve-conflicts.ts';
import { removeRemovableWorktrees, } from './remove-worktrees.ts';
import {
  reportToTheTerminal,
  type Report,
} from './report.ts';
import { runProgramWithChildProcess, } from './run-a-program.ts';
import {
  activeAgentUnits,
  type MachineState,
  workingFoldersOfProcesses,
} from './machine-state.ts';
import { reportOnWorktrees, } from './worktrees.ts';

/**
 Runs one command line and says what status it ends with.

 @param args - the words after the program's name

 @param workingFolder - the folder it runs in

 @returns 0 when it did what it was asked, 1 when it did not finish, and 2 when the caller is wrong

 @example
 ```ts
 const status = await runTheCommandLine({ args: ['land', '--branch=fix/a', '--message=m',], workingFolder: '/repository', },);
 ```
 */
export async function runTheCommandLine({
  args,
  workingFolder,
}: {
  readonly args: readonly string[];
  readonly workingFolder: string;
},): Promise<number> {
  try {
    return await runParsed({
      invocation: parseCommandLine({ args, },),
      workingFolder,
      report: reportToTheTerminal,
    },);
  }
  catch (caught) {
    if (caught instanceof LandingConfigurationError) {
      reportToTheTerminal.warning(caught.message,);
      return MISUSED_STATUS;
    }
    reportToTheTerminal.warning(caughtValueText(caught,),);
    return FAILED_STATUS;
  }
}

/**
 Runs one parsed invocation.

 @param invocation - what was asked

 @param workingFolder - the folder it runs in

 @param report - where to report

 @returns the status

 @example
 ```ts
 const status = await runParsed({ invocation, workingFolder, report, },);
 ```
 */
async function runParsed({
  invocation,
  workingFolder,
  report,
}: {
  readonly invocation: ReturnType<typeof parseCommandLine>;
  readonly workingFolder: string;
  readonly report: Report;
},): Promise<number> {
  if (invocation.command === 'help') {
    report.line(invocation.text,);
    return SUCCESS_STATUS;
  }
  /**
   The main checkout: the one the caller named, or the one this command runs in.
   */
  const mainCheckout = invocation.mainCheckout ?? await mainCheckoutOf({
    folder: workingFolder,
    run: runProgramWithChildProcess,
    timeoutMilliseconds: invocation.bounds
      .gitTimeoutMilliseconds,
  },);
  /**
   The main checkout, the way every sequence asks it questions.
   */
  const repository: Repository = {
    folder: mainCheckout,
    run: runProgramWithChildProcess,
    timeoutMilliseconds: invocation.bounds
      .gitTimeoutMilliseconds,
    outputBytes: GIT_OUTPUT_BYTES,
  };
  return runOf({
    invocation,
    repository,
    mainCheckout,
    report,
  },);
}

/**
 Runs one invocation that already knows its main checkout.

 @param invocation - what was asked

 @param repository - the checkout to ask

 @param mainCheckout - the main checkout

 @param report - where to report

 @returns the status

 @example
 ```ts
 const status = await runOf({ invocation, repository, mainCheckout, report, },);
 ```
 */
async function runOf({
  invocation,
  repository,
  mainCheckout,
  report,
}: {
  readonly invocation: Exclude<ReturnType<typeof parseCommandLine>, { readonly command: 'help' }>;
  readonly repository: Repository;
  readonly mainCheckout: string;
  readonly report: Report;
},): Promise<number> {
  if (invocation.command === 'land') {
    /**
     What the landing came to.
     */
    const outcome = await landABranch({
      repository,
      branch: invocation.branch,
      message: invocation.message,
      ...boundsOf(invocation.bounds,),
      report,
    },);
    return landedCompletely(outcome,) ? SUCCESS_STATUS : FAILED_STATUS;
  }
  if (invocation.command === 'check') {
    if (invocation.check === 'none') {
      throw new LandingConfigurationError('--check=none names nothing to check; a check runs the wrapper, a package, or tasks',);
    }
    /**
     What the check found.
     */
    const outcome = await checkABranch({
      repository: worktreeRepositoryOf({
        repository,
        name: invocation.worktree,
      },),
      kind: invocation.check,
      tasks: invocation.tasks,
      report,
    },);
    return checkPassed(outcome,) ? SUCCESS_STATUS : FAILED_STATUS;
  }
  if (invocation.command === 'list') return runAList({
    invocation,
    repository,
    report,
  },);
  if (invocation.command === 'commit-open-merge') {
    /**
     The checkout whose merge is committed, which is the main checkout unless the caller names a worktree.
     */
    const checkout = invocation.worktree === ''
      ? repository
      : worktreeRepositoryOf({
        repository,
        name: invocation.worktree,
      },);
    /**
     What committing came to.
     */
    const outcome = await commitTheOpenMerge({
      repository: checkout,
      message: invocation.message,
      paths: invocation.paths,
      tries: invocation.bounds
        .tries,
      pauseMilliseconds: invocation.bounds
        .pauseMilliseconds,
      commitTimeoutMilliseconds: invocation.bounds
        .commitTimeoutMilliseconds,
      report,
    },);
    return ((outcome.commit === 'committed') || (outcome.commit === 'no-merge-open'))
      ? SUCCESS_STATUS
      : FAILED_STATUS;
  }
  if (invocation.command === 'resolve-conflicts') return runResolve({
    invocation,
    repository,
    report,
  },);
  if (invocation.command === 'remove-worktrees') return runRemoval({
    invocation,
    repository,
    mainCheckout,
    report,
  },);
  return runWorktrees({
    repository,
    mainCheckout,
    report,
  },);
}

/**
 Turns the command line's bounds into the ones a landing sequence reads.

 @param bounds - the bounds from the command line

 @returns the same bounds under the names the sequences use

 @example
 ```ts
 const landingBounds = boundsOf(bounds,);
 ```
 */
function boundsOf(bounds: Bounds,): {
  readonly tries: number;
  readonly pauseMilliseconds: number;
  readonly commitTimeoutMilliseconds: number;
} {
  return {
    tries: bounds.tries,
    pauseMilliseconds: bounds.pauseMilliseconds,
    commitTimeoutMilliseconds: bounds.commitTimeoutMilliseconds,
  };
}

/**
 Lands the entries of a list file.

 @param invocation - what was asked

 @param repository - the checkout to ask

 @param report - where to report

 @returns the status

 @example
 ```ts
 const status = await runAList({ invocation, repository, report, },);
 ```
 */
async function runAList({
  invocation,
  repository,
  report,
}: {
  readonly invocation: Extract<ReturnType<typeof parseCommandLine>, { readonly command: 'list' }>;
  readonly repository: Repository;
  readonly report: Report;
},): Promise<number> {
  /**
   Every entry the file names.
   */
  const entries = await entriesOfFile({ path: invocation.list, },);
  /**
   The entries this run selects, which is all of them unless the caller names keys.
   */
  const selected = invocation.only
    .length
    === 0
    ? entries
    : entries.filter(function isSelected(entry) {
      return invocation.only
        .includes(entry.key,);
    },);
  if ((invocation.only
    .length
    > 0) && (selected.length === 0)) {
    throw new LandingConfigurationError(`${invocation.list}: no entry is named ${invocation.only
      .join(' ',)}`,);
  }
  /**
   What landing the list came to.
   */
  const outcome = await landAList({
    repository,
    entries: selected,
    ...boundsOf(invocation.bounds,),
    report,
  },);
  return listLandedCompletely(outcome,) ? SUCCESS_STATUS : FAILED_STATUS;
}

/**
 Settles or describes a worktree's conflicts.

 @param invocation - what was asked

 @param repository - the checkout to ask

 @param report - where to report

 @returns the status

 @example
 ```ts
 const status = await runResolve({ invocation, repository, report, },);
 ```
 */
async function runResolve({
  invocation,
  repository,
  report,
}: {
  readonly invocation: Extract<ReturnType<typeof parseCommandLine>, { readonly command: 'resolve-conflicts' }>;
  readonly repository: Repository;
  readonly report: Report;
},): Promise<number> {
  /**
   The worktree whose merge is being settled.
   */
  const worktree = worktreeRepositoryOf({
    repository,
    name: invocation.worktree,
  },);
  if (invocation.files
    .length
    === 0) {
    await reportOnAConflict({
      repository: worktree,
      report,
    },);
    return SUCCESS_STATUS;
  }
  /**
   What settling came to.
   */
  const outcome = await settleConflicts({
    repository: worktree,
    files: invocation.files
      .map(function fileOf(named: string,) {
      /**
       The path, which the entry holds as `path=side` when it names a side of its own.
       */
        const [path = named, side,] = named.split('=',);
        return {
          path,
          side: side === undefined ? invocation.side : parseSideToKeep(side,),
        };
      },),
    ...boundsOf(invocation.bounds,),
    report,
  },);
  return ((outcome.resolved === 'settled') && (outcome.commit
    .commit
    === 'committed'))
    ? SUCCESS_STATUS
    : FAILED_STATUS;
}

/**
 Reports on every worktree.

 @param repository - the checkout to ask

 @param mainCheckout - the main checkout

 @param report - where to report

 @returns the status

 @example
 ```ts
 const status = await runWorktrees({ repository, mainCheckout, report, },);
 ```
 */
async function runWorktrees({
  repository,
  mainCheckout,
  report,
}: {
  readonly repository: Repository;
  readonly mainCheckout: string;
  readonly report: Report;
},): Promise<number> {
  /**
   The worktrees and what is known of each.
   */
  const reports = await reportOnWorktrees({
    repository,
    machine: await machineStateOf(repository,),
    worktreesFolder: worktreeFolderOf(mainCheckout,),
  },);
  for (const each of reports) {
    report.line([
      each.name,
      `branch=${each.branch === '' ? '(detached)' : each.branch}`,
      `ahead=${each.commitsMainLacks === undefined ? '?' : String(each.commitsMainLacks,)}`,
      `running=${each.runningUnit ? 'yes' : 'no'}`,
      each.reasonToKeep === undefined ? 'SAFE' : `KEEP: ${each.reasonToKeep}`,
    ].join(' ',),);
  }
  report.line(`${String(reports.filter(function isSafe(each) {
    return each.reasonToKeep === undefined;
  },)
    .length,)} of ${String(reports.length,)} worktrees removable`,);
  return SUCCESS_STATUS;
}

/**
 Removes the worktrees nothing would be lost by removing.

 @param invocation - what was asked

 @param repository - the checkout to ask

 @param mainCheckout - the main checkout

 @param report - where to report

 @returns the status

 @example
 ```ts
 const status = await runRemoval({ invocation, repository, mainCheckout, report, },);
 ```
 */
async function runRemoval({
  invocation,
  repository,
  mainCheckout,
  report,
}: {
  readonly invocation: Extract<ReturnType<typeof parseCommandLine>, { readonly command: 'remove-worktrees' }>;
  readonly repository: Repository;
  readonly mainCheckout: string;
  readonly report: Report;
},): Promise<number> {
  if (!invocation.apply) {
    return runWorktrees({
      repository,
      mainCheckout,
      report,
    },);
  }
  await removeRemovableWorktrees({
    repository,
    machine: await machineStateOf(repository,),
    worktreesFolder: worktreeFolderOf(mainCheckout,),
    report,
  },);
  return SUCCESS_STATUS;
}

/**
 Reads what the machine says about a worktree right now.

 @param repository - the main checkout

 @returns the running units, the working folders, and the time of the judgement

 @example
 ```ts
 const machine = await machineStateOf(repository,);
 ```
 */
async function machineStateOf(repository: Repository,): Promise<MachineState> {
  return {
    activeUnits: await activeAgentUnits({ repository, },),
    workingFolders: await workingFoldersOfProcesses(),
    nowMilliseconds: Date.now(),
  };
}
