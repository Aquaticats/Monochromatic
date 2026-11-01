//region Short-bench warning finder
// TEST SUPPORT, NOT PACKAGE SOURCE. Finds the warning a seat reading wrote when it seated a short bench, shared by the
// cases that read the seats of a phase over a scripted view.

import type { JudgeSeatPhase, } from '../../dist/final/node/index.mjs';

/**
 The warning a reading wrote when it seated a short bench.

 @param lines - what the reading logged

 @param phase - phase the seats were read for

 @returns Line, empty where the reading wrote no such warning (a case that expects one then fails on its content)

 @example
 ```ts
 const warning = shortWarningOf({ lines, phase: 'preparation', },);
 ```
 */
export function shortWarningOf(
  {
    lines,
    phase,
  }: {
    readonly lines: readonly string[];
    readonly phase: JudgeSeatPhase;
  },
): string {
  return lines.find(function isShortWarning(line,): boolean {
    return line.startsWith(`warn JUDGE SEATS phase=${phase}`,) && line.includes('short of quorum',);
  },) ?? '';
}

//endregion Short-bench warning finder
