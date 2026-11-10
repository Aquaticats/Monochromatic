import {
  failureExitCode,
  ghProgram,
  releaseWorkflow,
} from './constants.ts';
import type { DispatchOptions } from './dispatch-options.ts';
import { formatCommandLine } from './command-line.ts';
import {
  checkOriginContainment,
  refusalLines,
} from './origin-containment.ts';
import { findRunAddress } from './run-results.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';
import { buildReleaseDispatchArguments } from './service-arguments.ts';

/**
 Runs one program call to completion and reports how it ended.
 */
export type DispatchRunner = (call: ProgramCall) => Promise<ProgramResult>;

/**
 Print every non-empty line of a program's output.

 @param text - captured output

 @param print - prints one line

 @example
 ```ts
 printLines({ text: 'a\n\nb\n', print: console.log });
 ```
 */
function printLines({
  text,
  print,
}: {
  readonly text: string;
  readonly print: (line: string) => void;
}): void {
  for (const line of text.split('\n'))
    if (line.trim() !== '')
      print(line);
}

/**
 Start the release workflow for a crate and report where its run is.

 Before that, origin's default branch is fetched and the dispatch is refused unless it holds every commit of the local
 branch of the same name, or when that could not be checked.

 Then the crate's own release gate runs, when its registry entry names one: the command holds the checks that only this
 machine can make, so it is the last thing between the person and a run nothing can take back. A `--dry-run` skips it,
 because a dry run runs nothing that the gate's evidence would cover.

 Nothing else happens on this machine: the workflow does the bump, the check, the dry run, the commit, the push and the
 publish start. A refusal by `gh` is shown as `gh` wrote it, with the call that made it, so the person sees the cause.

 @param options - validated dispatch options

 @param run - runs a program call and reports its status and output

 @param print - prints one line

 @returns zero when the workflow was started, the failure status when origin lacks local commits, the check could not
 finish, the crate's release gate did not pass, or `gh` refused

 @example
 ```ts
 await dispatchRelease({ options, run, print: console.log });
 ```
 */
export async function dispatchRelease({
  options,
  run,
  print,
}: {
  readonly options: DispatchOptions;
  readonly run: DispatchRunner;
  readonly print: (line: string) => void;
}): Promise<number> {
  // The workflow builds origin's default branch, not this checkout, and a published crate version can never be taken
  // back, so nothing is dispatched until origin is known to hold every local commit of that branch.
  /**
   How the local default branch stands against origin's.
   */
  const containment = await checkOriginContainment({
    root: options.root,
    run,
    print,
  });
  if (containment.kind !== 'contained') {
    for (const line of refusalLines(containment))
      print(line);
    return failureExitCode;
  }
  // The crate's own gate is the last refusal before the dispatch: it is the expensive one, so every check that costs
  // nothing has already run, and nothing can be taken back once `gh` has started the workflow.
  if (options.dispatchGate !== undefined) {
    // A dry run starts the workflow's own dry run, so the gate's evidence would cover nothing it does.
    if (options.dryRun)
      print(`RELEASE DRY RUN: not running ${formatCommandLine(options.dispatchGate)}, the gate this crate names for a real release, so this dry run proves nothing about it.`);
    else {
      print(`Waiting for ${formatCommandLine(options.dispatchGate)}`);
      /**
       How the crate's gate ended.
       */
      const gate = await run(options.dispatchGate);
      printLines({
        text: gate.stdout,
        print
      });
      printLines({
        text: gate.stderr,
        print
      });
      if (gate.status !== 0) {
        print(`REFUSED ${releaseWorkflow} was not started (${formatCommandLine(options.dispatchGate)} exited ${String(gate.status)})`);
        return failureExitCode;
      }
    }
  }
  /**
   The call that starts the workflow.
   */
  const call: ProgramCall = {
    program: ghProgram,
    args: buildReleaseDispatchArguments({
      crate: options.crate,
      from: options.from,
      to: options.to,
      note: options.note,
      dryRun: options.dryRun,
      ...(options.repo === undefined ? {} : { repo: options.repo }),
    }),
    cwd: options.root,
  };
  print(`$ ${formatCommandLine(call)}`);
  /**
   How `gh` ended.
   */
  const result = await run(call);
  printLines({
    text: result.stdout,
    print
  });
  printLines({
    text: result.stderr,
    print
  });
  if (result.status !== 0) {
    print(`FAILED ${releaseWorkflow} was not started (gh exited ${String(result.status)})`);
    return failureExitCode;
  }
  /**
   The address of the created run, when `gh` printed one.
   */
  const address = findRunAddress(result.stdout);
  /**
   What the run does, for the line that reports it.
   */
  const kind = options.dryRun ? 'RELEASE DRY RUN STARTED' : 'RELEASE STARTED';
  if (address.found)
    print(`${kind} ${address.value}`);
  else
    print(`${kind}, but ${address.reason} List it with: gh run list --workflow=${releaseWorkflow} --limit=1`);
  return 0;
}
