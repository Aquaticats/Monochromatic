import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { readAskedCount, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { BenchSlice, } from './bench-sample.ts';
import {
  benchWidths,
  summarizeBench,
  type writeBenchReport,
} from './bench-report.ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import { runBenchRow, } from './roster-bench-round.ts';
import type { BenchRow, } from './roster-bench-row.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Roster bench run
// Runs the SAME slices at several producer-roster widths, to answer the one
// question about width the corpus cannot answer: whether more candidates make
// the judges converge less.
//
// Width is the INNER loop and the slice the outer one. Running width 2 for an
// hour and then width 6 for an hour would confound width with provider weather,
// and this provider is measured to degrade by the day and to shed bursts under
// load. Interleaving means every width meets the same conditions.
//
// One width is run TWICE per slice, so the report carries a run-to-run band. A
// difference between widths smaller than the band between two runs of the same
// width is noise, and without the band there is no way to say which is which.
//
// SPENDS QUOTA. Point `TRANSLATION_REPAIR_RUNS_DIR` at a throwaway directory.

/**
 Slices drawn when the caller names no count.
 */
const DEFAULT_SLICES = 10;

/**
 Runs the whole bench and writes its report.

 @param line - the bench's command line, read whole by `reportingRefusals`

 @param roster - models the widths are cut from, which also judge

 @param newClient - builds the client one row's exchanges go through; called
 once per row, since the client answers a repeated prompt from the stored
 reply of the same client, so a row must not meet the previous row's replies

 @param drawSlices - draws the slices every width sees; the pinned corpus in a
 run, a script in a case

 @param readHead - reads the pipeline commit the rows were produced by

 @param writeReport - keeps the rows so far, which the run calls after every
 row so a killed bench leaves everything it bought

 @param clock - source of the instants each row's duration is read from

 @param runsDir - runs directory the report is written under, which the entry
 resolves from its environment

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link StatedRefusalError} when the slice count is not a count of at
 least one, and its subclass `BenchReportError` when the roster is too narrow
 to vary, both before the corpus is drawn; and whatever building a row's
 client refuses with, such as a missing provider key

 @example
 ```ts
 await runRosterBench({
   line,
   roster,
   newClient: createProbeRunClient,
   drawSlices,
   readHead,
   writeReport: writeBenchReport,
   clock: performance,
   runsDir,
   stageSettings,
 },);
 ```
 */
export async function runRosterBench(
  {
    line,
    roster,
    newClient,
    drawSlices,
    readHead,
    writeReport,
    clock,
    runsDir,
    stageSettings,
  }: {
    readonly line: CommandLineOf<'roster-bench'>;
    readonly roster: readonly RosterModelId[];
    readonly newClient: () => ModelClient;
    readonly drawSlices: (input: { readonly count: number; },) => Promise<readonly BenchSlice[]>;
    readonly readHead: () => Promise<string>;
    readonly writeReport: typeof writeBenchReport;
    readonly clock: { readonly now: () => number; };
    readonly runsDir: string;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Slices asked for on the command line, or the default.
   */
  const wanted = readAskedCount({
    line,
    fallback: DEFAULT_SLICES,
    asks: 'slices',
  },);

  /**
   Widths this roster supports, and which of them is run twice; a roster too
   narrow to vary is refused here by `BenchReportError`, a stated refusal by
   class, before the corpus is drawn.
   */
  const {
    widths,
    repeated,
  } = benchWidths({ roster, },);

  /**
   Slices every width sees.
   */
  const sample = await drawSlices({ count: wanted, },);
  console.log(
    `BENCH ${String(sample.length,)} ${
      wordForCount({
        count: sample.length,
        one: 'slice',
        many: 'slices',
      },)
    }, widths ${
      widths.join(', ',)
    }, width ${String(repeated,)} run twice, roster of ${
      String(roster.length,)
    }`,
  );

  /**
   Every row, accumulated as they finish so a killed run still has a report.
   */
  const rows: BenchRow[] = [];

  /**
   Pipeline commit these rows were produced by.
   */
  const headSha = await readHead();

  /**
   Every run this bench will make, width inner so each width meets the same
   provider weather rather than its own hour of the night.
   */
  const runs = sample.flatMap(function toRuns(slice,) {
    return widths.flatMap(function toWidthRuns(width,) {
      return (width === repeated
        ? [
          1,
          2,
        ]
        : [1,])
        .map(function toRun(pass,) {
          return {
            slice,
            width,
            pass,
          };
        },);
    },);
  },);
  for (const run of runs) {
    /* oxlint-disable no-await-in-loop -- sequential by design: each benchmark row is written before the next starts so a killed bench keeps every complete purchase; provider capacity is not the reason */
    /**
     What this slice decided at this width.
     */
    const row = await runBenchRow({
      ...run,
      roster,
      client: newClient(),
      clock,
      stageSettings,
    },);
    rows.push(row,);
    console.log(
      `BENCH ${row.entryId}#${String(row.index,)} w${String(row.width,)}p${
        String(row.pass,)
      }: ${row.decision}, ${
        row.keptIncumbent ? 'kept' : 'replaced'
      }, weight ${String(row.voteWeight,)}, ${
        String(row.calls
          .length,)
      } ${
        wordForCount({
          count: row.calls
            .length,
          one: 'call',
          many: 'calls',
        },)
      }, ${String(Math.round(row.ms,),)}ms`,
    );
    await writeReport({
      runsDir,
      rows,
      headSha,
      widths,
      repeated,
      roster,
    },);
    /* oxlint-enable no-await-in-loop */
  }

  summarizeBench({ rows, },);
}

//endregion Roster bench run
