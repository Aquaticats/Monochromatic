import {
  decimalDigits,
  usesOnly,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';

import { formatCommandLine, } from './command-line.ts';
import {
  ancestorStatuses,
  gitProgram,
  originRemote,
  releaseWorkflow,
  symbolicHeadPrefix,
} from './constants.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';
import { parseBranchName, } from './validate.ts';

//region Types

/**
 Outcome of comparing the local default branch with the one on origin.
 */
export type OriginContainment =
  | {
    readonly kind: 'contained';
  }
    | {
      readonly kind: 'lacking';
      readonly branch: string;
    /**
     Commits of the local branch that origin does not have.
     */
      readonly count: number;
      readonly newestHash: string;
      readonly newestSubject: string;
    }
    | {
      readonly kind: 'unchecked';
    /**
     The `git` command line that did not give an answer, as a person would paste it.
     */
      readonly command: string;
    /**
     Why the containment could not be checked: Git's own message when it printed one.
     */
      readonly reason: string;
    };

/**
 Runs one program call to completion and reports how it ended.
 */
export type ContainmentRunner = (call: ProgramCall) => Promise<ProgramResult>;

//endregion Types

//region Git calls

/**
 The call that asks origin which branch its `HEAD` names, which is the branch the release workflow builds when it is
 dispatched without a branch.

 @param root - absolute repository root

 @returns the call

 @example
 ```ts
 buildDefaultBranchCall('/repo');
 ```
 */
export function buildDefaultBranchCall(root: string,): ProgramCall {
  return {
    program: gitProgram,
    args: [
      'ls-remote',
      '--symref',
      originRemote,
      'HEAD',
    ],
    cwd: root,
  };
}

/**
 The call that brings the remote-tracking branch up to date with origin, replacing it even when origin moved it
 backwards, so the comparison never reads a stale copy.

 @param root - absolute repository root

 @param branch - validated branch name

 @returns the call

 @example
 ```ts
 buildFetchCall({ root: '/repo', branch: 'main' });
 ```
 */
export function buildFetchCall({
  root,
  branch,
}: {
  readonly root: string;
  readonly branch: string;
},): ProgramCall {
  return {
    program: gitProgram,
    args: [
      'fetch',
      '--no-tags',
      originRemote,
      `+refs/heads/${branch}:refs/remotes/${originRemote}/${branch}`,
    ],
    cwd: root,
  };
}

/**
 The call whose exit status tells whether origin holds every commit of the local branch.

 @param root - absolute repository root

 @param branch - validated branch name

 @returns the call

 @example
 ```ts
 buildContainmentCall({ root: '/repo', branch: 'main' });
 ```
 */
export function buildContainmentCall({
  root,
  branch,
}: {
  readonly root: string;
  readonly branch: string;
},): ProgramCall {
  return {
    program: gitProgram,
    args: [
      'merge-base',
      '--is-ancestor',
      `refs/heads/${branch}`,
      `refs/remotes/${originRemote}/${branch}`,
    ],
    cwd: root,
  };
}

/**
 The range of commits the local branch holds and origin does not.

 @param branch - validated branch name

 @returns the range in Git's `<excluded>..<included>` form
 */
function lackingRange(branch: string,): string {
  return `refs/remotes/${originRemote}/${branch}..refs/heads/${branch}`;
}

/**
 The call that counts the commits origin lacks.

 @param root - absolute repository root

 @param branch - validated branch name

 @returns the call

 @example
 ```ts
 buildLackingCountCall({ root: '/repo', branch: 'main' });
 ```
 */
export function buildLackingCountCall({
  root,
  branch,
}: {
  readonly root: string;
  readonly branch: string;
},): ProgramCall {
  return {
    program: gitProgram,
    args: [
      'rev-list',
      '--count',
      lackingRange(branch,),
    ],
    cwd: root,
  };
}

/**
 The call that prints the newest commit origin lacks as its short hash, a tab and its subject.

 @param root - absolute repository root

 @param branch - validated branch name

 @returns the call

 @example
 ```ts
 buildNewestLackingCall({ root: '/repo', branch: 'main' });
 ```
 */
export function buildNewestLackingCall({
  root,
  branch,
}: {
  readonly root: string;
  readonly branch: string;
},): ProgramCall {
  return {
    program: gitProgram,
    args: [
      'log',
      '--max-count=1',
      '--format=%h%x09%s',
      lackingRange(branch,),
    ],
    cwd: root,
  };
}

//endregion Git calls

//region Parsing

/**
 Read the default branch out of the answer of `git ls-remote --symref origin HEAD`.

 @param stdout - captured output, whose first line holds `ref: refs/heads/<branch>` and a tab and `HEAD`

 @returns the branch name, or a miss that says what origin printed instead

 @example
 ```ts
 parseDefaultBranch('ref: refs/heads/main\tHEAD\n0123456789abcdef\tHEAD\n');
 ```
 */
export function parseDefaultBranch(stdout: string,): Lookup<string> {
  /**
   The line that names the branch `HEAD` points to.
   */
  const symbolicLine = stdout
    .split('\n',)
    .find(function namesBranch(line,) {
      return line.startsWith(symbolicHeadPrefix,);
    },);
  if (symbolicLine === undefined)
    return missingValue(
      `${originRemote} printed no line that starts with ${JSON.stringify(symbolicHeadPrefix,)}, so it names no default branch.`,
    );
  /**
   The branch name, before the tab that separates it from `HEAD`.
   */
  const branch = symbolicLine
    .slice(symbolicHeadPrefix.length,)
    .split('\t',)[0]
    ?? '';
  try {
    return foundValue(parseBranchName({
      option: `the default branch of ${originRemote}`,
      value: branch,
    },),);
  }
  catch (error) {
    return missingValue(
      `${originRemote} names the default branch ${JSON.stringify(branch,)}, which is not a branch name this tool passes to Git${
        Error.isError(error,) ? `: ${error.message}` : ''
      }`,
    );
  }
}

/**
 Read the newest lacking commit out of `%h<TAB>%s` output.

 @param stdout - captured output of the log call

 @returns the short hash and subject, or a miss when the line has no tab
 */
function parseNewestLacking(stdout: string,): Lookup<{
  readonly hash: string;
  readonly subject: string;
}> {
  /**
   The first line, which is the only commit asked for.
   */
  const line = stdout.split('\n',)[0] ?? '';
  /**
   Position of the tab between hash and subject.
   */
  const tab = line.indexOf('\t',);
  if (tab <= 0)
    return missingValue(`git log printed ${JSON.stringify(line,)} instead of a short hash, a tab and a subject.`,);
  return foundValue({
    hash: line.slice(
      0,
      tab,
    ),
    subject: line.slice(tab + 1,),
  },);
}

//endregion Parsing

//region Check

/**
 Build the verdict for a step that gave no answer.

 @param call - the call
 
 @param result - how it ended
 
 @returns the verdict
 */
function uncheckedVerdict({
  call,
  result,
}: {
  readonly call: ProgramCall;
  readonly result: ProgramResult;
},): OriginContainment {
  /**
   Git's own message, trimmed.
   */
  const message = result.stderr
    .trim();
  return {
    kind: 'unchecked',
    command: formatCommandLine(call,),
    reason: `${message === '' ? 'git printed no message' : message} (git exited ${String(result.status,)})`,
  };
}

/**
 Compare the local default branch with origin's, step by step, and say how they stand.

 Each `git` call is printed as a `$` line before it runs. A step that fails is not an answer: the result is
 `unchecked` with Git's own message, never `contained`.

 @param root - absolute repository root, where `git` runs

 @param run - runs a program call and reports its status and output

 @param print - prints one line

 @returns `contained` when origin holds every local commit of its default branch, `lacking` with the count and the
 newest missing commit when it does not, `unchecked` when any step gave no answer

 @example
 ```ts
 await checkOriginContainment({ root: '/repo', run, print: console.log });
 ```
 */
export async function checkOriginContainment({
  root,
  run,
  print,
}: {
  readonly root: string;
  readonly run: ContainmentRunner;
  readonly print: (line: string,) => void;
},): Promise<OriginContainment> {
  /**
   Print a call, then run it.

   @param call - the call
   
   @returns how it ended
   */
  async function step(call: ProgramCall,): Promise<ProgramResult> {
    print(`$ ${formatCommandLine(call,)}`,);
    return await run(call,);
  }

  /**
   The call that names the default branch, and how it ended.
   */
  const branchCall = buildDefaultBranchCall(root,);
  /**
   How the call that names the default branch ended.
   */
  const branchResult = await step(branchCall,);
  if (branchResult.status !== 0)
    return uncheckedVerdict({
      call: branchCall,
      result: branchResult,
    },);
  /**
   The default branch, when origin named one.
   */
  const branch = parseDefaultBranch(branchResult.stdout,);
  if (!branch.found)
    return {
      kind: 'unchecked',
      command: formatCommandLine(branchCall,),
      reason: branch.reason,
    };
  /**
   The call that refreshes the remote-tracking copy of the default branch.
   */
  const fetchCall = buildFetchCall({
    root,
    branch: branch.value,
  },);
  /**
   How the fetch ended.
   */
  const fetchResult = await step(fetchCall,);
  if (fetchResult.status !== 0)
    return uncheckedVerdict({
      call: fetchCall,
      result: fetchResult,
    },);
  /**
   The call whose exit status says whether origin holds the local branch.
   */
  const containmentCall = buildContainmentCall({
    root,
    branch: branch.value,
  },);
  /**
   How the containment test ended.
   */
  const containmentResult = await step(containmentCall,);
  if (containmentResult.status === ancestorStatuses.contained)
    return { kind: 'contained', };
  if (containmentResult.status !== ancestorStatuses.notContained)
    return uncheckedVerdict({
      call: containmentCall,
      result: containmentResult,
    },);
  /**
   The call that counts the commits origin lacks.
   */
  const countCall = buildLackingCountCall({
    root,
    branch: branch.value,
  },);
  /**
   How the count ended.
   */
  const countResult = await step(countCall,);
  /**
   The count text, trimmed.
   */
  const countText = countResult.stdout
    .trim();
  if ((countResult.status !== 0) || (countText === '')
    || (!usesOnly({
      text: countText,
      allowed: decimalDigits,
    },)))
    return uncheckedVerdict({
      call: countCall,
      result: countResult,
    },);
  /**
   The call that prints the newest commit origin lacks.
   */
  const newestCall = buildNewestLackingCall({
    root,
    branch: branch.value,
  },);
  /**
   How the log call ended.
   */
  const newestResult = await step(newestCall,);
  if (newestResult.status !== 0)
    return uncheckedVerdict({
      call: newestCall,
      result: newestResult,
    },);
  /**
   The newest lacking commit, when git printed one.
   */
  const newest = parseNewestLacking(newestResult.stdout,);
  if (!newest.found)
    return {
      kind: 'unchecked',
      command: formatCommandLine(newestCall,),
      reason: newest.reason,
    };
  return {
    kind: 'lacking',
    branch: branch.value,
    count: Number(countText,),
    newestHash: newest.value
      .hash,
    newestSubject: newest.value
      .subject,
  };
}

//endregion Check

//region Refusal text

/**
 Write the lines of the refusal for a verdict that stops the dispatch.

 @param verdict - a verdict other than `contained`

 @returns the lines to print: what is wrong, why it matters and every way out

 @example
 ```ts
 refusalLines({ kind: 'unchecked', command: 'git fetch origin', reason: 'offline' });
 ```
 */
export function refusalLines(verdict: Exclude<OriginContainment, { readonly kind: 'contained'; }>,): readonly string[] {
  /**
   Why an unpublished source is a hazard, shared by both refusals.
   */
  const hazard = `${releaseWorkflow} builds the default branch of ${originRemote}, and a published crate version can never be taken back on crates.io.`;
  /**
   The way back to a release, shared by both refusals.
   */
  const rerun = 'then run the same release command again.';
  if (verdict.kind === 'unchecked')
    return [
      `REFUSED ${releaseWorkflow} was not started: it could not be checked whether ${originRemote} holds every commit of the local default branch.`,
      `The step that gave no answer is \`${verdict.command}\`: ${verdict.reason}`,
      `${hazard} Publishing without the check could release old source under the new version.`,
      `Fix the cause (network, credentials or the ${originRemote} remote), ${rerun}`,
    ];
  /**
   The count with its noun.
   */
  const commits = verdict.count === 1 ?
    `1 commit of local ${verdict.branch} that ${originRemote}/${verdict.branch} does not have` :
    `${String(verdict.count,)} commits of local ${verdict.branch} that ${originRemote}/${verdict.branch} do not have`;
  return [
    `REFUSED ${releaseWorkflow} was not started: ${commits}.`,
    `Newest commit that origin lacks: ${verdict.newestHash} ${verdict.newestSubject}`,
    `${hazard} Dispatching now would publish source without ${verdict.count === 1 ? 'it' : 'them'} under the new version.`,
    `Either wait for auto-push to send ${verdict.count === 1 ? 'it' : 'them'}, or run \`git push ${originRemote} ${verdict.branch}\`; ${rerun}`,
  ];
}

//endregion Refusal text
