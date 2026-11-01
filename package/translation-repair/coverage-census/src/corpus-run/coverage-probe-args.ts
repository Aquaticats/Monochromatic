import {
  type CommandLineOf,
  idListFlag,
  wholeNumberFlag,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Coverage probe arguments
// What the coverage probe is asked on its command line, kept apart from the
// probe so the reading can be tested without a subprocess, as
// `judge-fidelity-args.ts` and `rendering-audit-settled-args.ts` are. All
// three read their flag values through `command-flags.ts` (ledger B73), from
// a line already read whole against the probe's declaration (ledger B75).

/**
 How many candidates one invocation asks about by default.

 Small on purpose: the first run of anything that spends quota should be
 readable in full before a larger one is bought.

 COUNTED IN ATTEMPTS RATHER THAN IN ROWS, because a cap on successes lets a
 failing roster spend without bound and hides the failures from the count a
 reader checks.
 */
export const DEFAULT_CANDIDATE_CAP = 12;

/**
 Reads the entry filter and cap from the command line.

 @param line - the probe's command line, read whole by `reportingRefusals`
 and passed in so this is testable without a subprocess

 @returns Entry ids to probe, empty for every entry, and the candidate cap

 @throws StatedRefusalError when a cap is not a whole number written in
 digits or is below zero, or an entry filter names no entry

 @example
 ```ts
 const { onlyIds, cap, } = readCoverageProbeArguments({ line, },);
 ```
 */
export function readCoverageProbeArguments(
  { line, }: { readonly line: CommandLineOf<'coverage-probe'>; },
): {
  readonly onlyIds: readonly string[];
  readonly cap: number;
} {
  return {
    onlyIds: idListFlag({
      asked: line.flag('only',),
      naming: 'entry id',
    },),
    cap: wholeNumberFlag({
      asked: line.flag('cap',),
      unwritten: DEFAULT_CANDIDATE_CAP,
      leaveOffTo: `ask about the default of ${String(DEFAULT_CANDIDATE_CAP,)} candidates`,
    },),
  };
}

//endregion Coverage probe arguments
