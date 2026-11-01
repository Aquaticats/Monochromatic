import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  type CommandLineOf,
  idListFlag,
  wholeNumberFlag,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Settled rendering audit arguments
// The command line surface of `rendering-audit-settled.ts`, kept apart from the
// probe itself because they answer to different readers: this is what a person
// types, and that is what a roster is asked.
//
// The clone comes from the run's corpus pin, which the entry reads and hands
// in, and the COMMIT does not. Only the location of a checkout is a run-level
// fact; which commit a settled artifact was read at is a fact about that
// artifact, and it carries it.

/**
 Default archive, whose subdirectories are run sets.
 */
const DEFAULT_ARCHIVE_DIR = join(
  homedir(),
  'translation-repair-v2-archive',
);

/**
 Cap meaning "buy everything", which is what a run with no `--cap` asks for.
 */
const NO_CAP = -1;

/**
 What the command line asked for.

 @example
 ```ts
 const { archiveDir, cloneDir, onlyIds, cap, } = readAuditArguments({ line, },);
 ```
 */
export type AuditArguments = {
  /**
   Archive whose subdirectories are run sets.
   */
  readonly archiveDir: string;

  /**
   Corpus clone the artifacts' own commits are read from.
   */
  readonly cloneDir: string;

  /**
   Entries to audit, empty for every entry.
   */
  readonly onlyIds: readonly string[];

  /**
   How many subjects to buy, negative for all of them.

   ZERO IS MEANINGFUL and is half the reason this exists: it reads and
   verifies the whole archive, prints the population, and asks nobody
   anything. A run that cannot do that has a wiring fault, and finding one
   should not cost a roster.
   */
  readonly cap: number;
};

/**
 What the report was told to read and compare.

 @example
 ```ts
 const { run, against, } = readReportArguments({ line, },);
 ```
 */
export type ReportArguments = {
  /**
   Run file named with `--run`, in a one-element list, empty when the
   report should read the newest kept run.
   */
  readonly run: readonly string[];

  /**
   Earlier run file named with `--against`, in a one-element list, empty
   when no across-run band was asked for.
   */
  readonly against: readonly string[];
};

/**
 Reads the report's two flags with the same refusal the audit's flags get.

 SHARED RATHER THAN COPIED, because the report module had its own reader
 that collapsed absent and valueless into one empty string, which is exactly
 the defect this module records as fixed for `--cap` and `--only`: `--run`
 written last reported the newest run, and `--against` written last printed
 no across-run band, and neither said a word.

 @param line - the report's command line, read whole by `reportingRefusals`

 @returns Named files, each in a one-element list when written


 @example
 ```ts
 const { run, } = readReportArguments({ line, },);
 ```
 */
export function readReportArguments(
  { line, }: { readonly line: CommandLineOf<'rendering-audit-settled-report'>; },
): ReportArguments {
  /**
   What `--run` named.
   */
  const run = line.flag('run',);

  /**
   What `--against` named.
   */
  const against = line.flag('against',);
  return {
    run: (run.kind === 'written') ? [run.value,] : [],
    against: (against.kind === 'written') ? [against.value,] : [],
  };
}

/**
 Reads what the command line asked for.

 @param line - the audit's command line, read whole by `reportingRefusals`
 and passed in so this is testable without a subprocess

 @param runCloneDir - clone of the run's corpus pin, which the entry read
 inside its refusal boundary, used when `--clone` names none; until
 2026-10-10 this read a module constant resolved at load (ledger B302, B381)

 @returns Archive, clone, entry filter and cap

 @throws StatedRefusalError when a cap is not a whole number written in
 digits or is below zero, or an entry filter names no entry

 @example
 ```ts
 const asked = readAuditArguments({ line, runCloneDir: pin.cloneDir, },);
 ```
 */
export function readAuditArguments(
  {
    line,
    runCloneDir,
  }: {
    readonly line: CommandLineOf<'rendering-audit-settled'>;
    readonly runCloneDir: string;
  },
): AuditArguments {
  /**
   Archive as written, absent when none was named.
   */
  const archiveText = line.flag('archive',);

  /**
   Clone as written, absent when none was named.
   */
  const cloneText = line.flag('clone',);

  return {
    archiveDir: (archiveText.kind === 'written') ? archiveText.value : DEFAULT_ARCHIVE_DIR,
    cloneDir: (cloneText.kind === 'written') ? cloneText.value : runCloneDir,
    onlyIds: idListFlag({
      asked: line.flag('only',),
      naming: 'entry id',
    },),
    // A CAP THAT IS NOT A NUMBER USED TO BUY NOTHING IN SILENCE: `capped` in
    // `rendering-audit-settled-buy.ts` returns every subject for a negative cap
    // and `slice(0, cap)` otherwise, and `Number('once')` is `NaN`, so a
    // mistyped cap audited zero subjects and reported a clean run. A typed
    // sign is refused too, rather than reaching the `NO_CAP` sentinel and
    // auditing the whole archive.
    cap: wholeNumberFlag({
      asked: line.flag('cap',),
      unwritten: NO_CAP,
      leaveOffTo: 'audit every subject',
    },),
  };
}

//endregion Settled rendering audit arguments
