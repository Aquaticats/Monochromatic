import {
  caseKey,
  casesBySalts,
  type FailedCase,
  type SweepOutcome,
} from './rotation-sweep-report.test-fixture.ts';

//region Rotation sweep refusal
// A FOUND FAILURE IS THE COMMAND'S ANSWER, NOT A FAULT OF THE COMMAND (ledger B701, and the class of the ledger
// "Operator refusals printed as faults"). The sweep ran to its end and found cases to fix, which is what the task is for;
// ending it in a thrown error printed the stack of the entry file and said nothing the report had not. A refusal names
// each case, its package, the salts that fail it and the command that reruns one salt, and the task exits with the code
// `reportingRefusals` gives a stated refusal. An error out of the sweep machinery (the canary, a package with no test
// file, a bad option) is a fault, and is thrown with its stack as before.

/**
 Exit code of a sweep that found cases to fix: 6, the code `reportingRefusals` (`run/src/corpus-run/cli-refusal.ts`) gives a
 command that declined in its own words, above the fault code 5 and clear of 1 to 3, which commands keep for their own verdicts.
 */
export const SWEEP_REFUSED_EXIT = 6;

/**
 The command an operator reruns the sweep with; its options follow `--`.
 */
const RERUN_COMMAND = 'mise run //package/translation-repair/source-scan:test:rotation-sweep --';

/**
 The options the task takes, said in every refusal of a command line.
 */
const OPTION_FORMS = 'the options are --package <directory> and --salt <n>, each repeatable';

/**
 A command line the sweep refuses: an option it does not take, a value it cannot use or none where one is needed.

 This is the operator's mistake and not a fault of the command, so `rotationSweepMain` prints its sentence alone and
 returns the stated-refusal code. The sentence is composed of authored words and the option and value the operator typed.
 */
export class SweepOptionRefusalError extends Error {
  /**
   @param reason - what is wrong with the command line, naming the option typed

   @example
   ```ts
   throw new SweepOptionRefusalError({ reason: '--nap is not an option', },);
   ```
   */
  constructor({ reason, }: { readonly reason: string; },) {
    super(`rotation sweep: refused, ${reason}; ${OPTION_FORMS}`,);
    this.name = 'SweepOptionRefusalError';
  }
}

/**
 The package directory a case's test file sits in.

 @param failed - case and its file, a path under `package/translation-repair`

 @returns The first segment of the path

 @example
 ```ts
 const directory = packageOf({ failed: { file: 'stage/src/a.unit.test.ts', name: '[nap]', }, },); // 'stage'
 ```
 */
function packageOf({ failed, }: { readonly failed: FailedCase; },): string {
  return failed.file
    .split('/',)
    .at(0,)
    ?? '';
}

/**
 Lines that refuse a sweep which found cases to fix: each case with its package, its salts and the command that reruns it.

 @param outcome - what the sweep found

 @returns Lines of the refusal, none when the sweep is clean

 @example
 ```ts
 const lines = refusalLines({ outcome: { results: [], redAtSaltZero: [], }, },); // []
 ```
 */
export function refusalLines({ outcome, }: { readonly outcome: SweepOutcome; },): readonly string[] {
  /**
   Cases a salt failed that salt 0 passed.
   */
  const shifted = casesBySalts({ outcome, },);
  if ((shifted.length === 0) && (outcome.redAtSaltZero
    .length
    === 0))
    return [];
  return [
    'rotation sweep: refused, because the sweep ran to the end and found cases to fix; this is a finding, not a fault of '
      + 'the command',
    ...shifted.flatMap(function shiftedLines({
      failed,
      firstSalt,
      salts,
    },): readonly string[] {
      return [
        `case: ${caseKey({ failed, },)}`,
        `  package: ${packageOf({ failed, },)}`,
        `  fails under salts: ${salts.join(', ',)} (salt 0 passes it)`,
        `  rerun one salt: ${RERUN_COMMAND} --package ${packageOf({ failed, },)} --salt ${String(firstSalt,)}`,
      ];
    },),
    ...outcome.redAtSaltZero
      .flatMap(function redLines(failed,): readonly string[] {
        return [
          `case: ${caseKey({ failed, },)}`,
          `  package: ${packageOf({ failed, },)}`,
          '  fails with no shift (salt 0), which cannot be read against the salts',
          `  rerun salt 0 alone: ${RERUN_COMMAND} --package ${packageOf({ failed, },)} --salt 0`,
        ];
      },),
  ];
}

//endregion Rotation sweep refusal
