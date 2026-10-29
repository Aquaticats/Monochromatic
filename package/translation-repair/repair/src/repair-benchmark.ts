import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  exchangeFailureLogText,
  exchangeFailureText,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  measureSeedRestoration,
  monotonicMs,
  type SeedRestoration,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { BenchmarkEntry, } from '@monochromatic-dev/translation-repair-archive/ts';
import type {
  RepairModels,
  RepairStatus,
  RepairTranslationResult,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  applySeededErrors,
  gradeSeedDetection,
  type SeedDetectionVerdict,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { repairTranslation, } from './repair-entry.ts';
import {
  runDerivabilityProbe,
  type SeedDerivability,
} from './derivability-probe.ts';
import {
  runRestorationJudge,
  type SeedJudgment,
} from './restoration-judge.ts';
import {
  computeRepairScorecard,
  type RepairScorecard,
} from './repair-scorecard.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Repair benchmark
// Milestone-two exit harness: plant omission seeds into real translations,
// run the whole repair loop, and grade how much of the deleted content came
// back. The PRIMARY grade is a bilingual ensemble judge anchored on the
// Chinese source (user directive): it rules whether each deleted sentence's
// meaning is present in the repaired translation and grounded in the
// original, tolerating terse-but-faithful rewording that a vocabulary-overlap
// grader under-credited. The lexical overlap grade is kept alongside as a
// cheap lower-bound signal, never the headline rate.

/**
 Logger root for the repair benchmark shell.
 */
const l = contextRoot({ tag: 'translation-repair-repair-benchmark', },);

/**
 Budget floor under which no new entry dispatches;
 a sliver of remaining budget cannot fit a whole repair run.
 */
export const MIN_REPAIR_DISPATCH_BUDGET_MS = 120_000;

/**
 One graded repair attempt over one entry.

 @example
 ```ts
 const record: RepairAttemptRecord = {
   entryId: 'whiskers',
   outcomeKind: 'ok',
   status: 'repaired',
   seedGrades: { 'seed/omission-0': grade, },
   issueCount: 3,
   resolvedIssueCount: 2,
 };
 ```
 */
export type RepairAttemptRecord = {
  /**
   Entry under repair.
   */
  readonly entryId: string;

  /**
   Whether the run dispatched, was cut by the budget, or threw.
   */
  readonly outcomeKind: 'ok' | 'skipped' | 'error';

  /**
   Pipeline completion status; absent when the run never produced one.
   */
  readonly status?: RepairStatus;

  /**
   Primary zh-anchored verdict per planted seed id from the bilingual
   judge ensemble.
   */
  readonly seedJudgments: Readonly<Record<string, SeedJudgment>>;

  /**
   Whether each planted seed was recoverable from the Chinese at all.

   Judged against the SOURCE and the deletion, never against the repaired
   text, so it is independent of what the pipeline did. A seed the source
   does not license cannot fairly count against detection.
   */
  readonly seedDerivability: Readonly<Record<string, SeedDerivability>>;

  /**
   Lexical overlap grade per planted seed id;
   a lower-bound signal kept for comparison, never the headline rate.
   */
  readonly seedGrades: Readonly<Record<string, SeedRestoration>>;

  /**
   How each planted seed fared at detection, separating detection failures
   from repair failures and from panel declines made on protective grounds.
   */
  readonly seedDetection: Readonly<Record<string, SeedDetectionVerdict>>;

  /**
   Adjudicated issues the run reported.
   */
  readonly issueCount: number;

  /**
   Issues the checkers confirmed fixed in the shipped text.
   */
  readonly resolvedIssueCount: number;

  /**
   Failure or skip detail in scorecard-stable wording.
   */
  readonly detail: string;

  /**
   Shipped candidate text; present only on dispatched attempts so
   saved run artifacts support post-run analysis of how partially
   restored seeds differ from their planted needles.
   */
  readonly repairedText?: string;
};


/**
 Whole repair benchmark result.

 @example
 ```ts
 const { records, scorecard, } = await runRepairBenchmark({ ... },);
 ```
 */
export type RepairBenchmarkResult = {
  /**
   Graded attempts in entry order.
   */
  readonly records: readonly RepairAttemptRecord[];

  /**
   Aggregate scorecard.
   */
  readonly scorecard: RepairScorecard;
};

/**
 Runs the milestone-two benchmark: every entry gets its seeds planted,
 the whole repair loop runs on the seeded pair, and restoration grades
 against the known deletions. Entries run sequentially inside the run
 budget; what the budget cannot fit records as skipped, and the
 scorecard reports the resulting coverage.

 @param client - injected model client

 @param entries - corpus entries with derived seeds

 @param models - role roster for every repair run

 @param signal - abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param runBudgetMs - real time the whole benchmark may take, counted on the
 monotonic clock so setting the system clock neither spends nor refunds it

 @param repair - repair driver seam; tests inject a scripted one

 @param judge - restoration-judge seam; tests inject a scripted one

 @param judgeModelIds - bilingual judge roster, from the caller's seated
 bench; no default, since a hand-listed one outlived the owner's cull
 (ledger P4)

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Graded attempts plus the aggregate scorecard

 @throws Whatever `signal.reason` carries once the caller aborts; an entry's
 fault under a live signal is recorded as that entry's error instead

 @example
 ```ts
 const { scorecard, } = await runRepairBenchmark({ client, entries, models, signal, perCallTimeoutMs, judgeModelIds, stageSettings, },);
 console.log(scorecard.seededRepairRate,);
 ```
 */
export async function runRepairBenchmark(
  {
    client,
    entries,
    models,
    signal,
    perCallTimeoutMs,
    runBudgetMs,
    repair = repairTranslation,
    judge = runRestorationJudge,
    derivability = runDerivabilityProbe,
    judgeModelIds,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly entries: readonly BenchmarkEntry[];
    readonly models: RepairModels;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly runBudgetMs?: number;
    readonly repair?: typeof repairTranslation;
    readonly judge?: typeof runRestorationJudge;
    readonly derivability?: typeof runDerivabilityProbe;
    readonly judgeModelIds: readonly RosterModelId[];
    readonly stageSettings: StageSettings;
  }>,
): Promise<RepairBenchmarkResult> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: runRepairBenchmark.name,
    l,
  },);

  /**
   Monotonic reading the run budget counts from (ledger B78).
   */
  const runStartedAt = monotonicMs();

  /**
   Graded attempts in entry order.
   */
  const records: RepairAttemptRecord[] = [];
  for (const entry of entries) {
    /**
     Run budget left at dispatch time; unbounded without a budget.
     */
    const remaining = runBudgetMs === undefined
      ? Number.POSITIVE_INFINITY
      : runBudgetMs - (monotonicMs() - runStartedAt);
    if (remaining < MIN_REPAIR_DISPATCH_BUDGET_MS) {
      rl.warn(`${entry.entryId}: run budget exhausted, skipping`,);
      records.push({
        entryId: entry.entryId,
        outcomeKind: 'skipped',
        seedJudgments: {},
        seedDerivability: {},
        seedGrades: {},
        seedDetection: {},
        issueCount: 0,
        resolvedIssueCount: 0,
        detail: 'run-budget-exhausted',
      },);
      continue;
    }

    /**
     Seeded pair for this entry, with planted regions for detection
     grading.
     */
    const {
      seededText,
      applications,
    } = applySeededErrors({
      text: entry.targetText,
      specs: entry.seeds,
    },);
    try {
      /* oxlint-disable no-await-in-loop -- sequential by design: each benchmark row mutates one seeded fixture and is recorded before next; provider capacity is not reason */
      /**
       Whole-pipeline result over the seeded pair.
       */
      const result: RepairTranslationResult = await repair({
        client,
        sourceText: entry.sourceText,
        targetText: seededText,
        models,
        signal,
        perCallTimeoutMs,
        stageSettings,
      },);
      /**
       Seeds this entry planted, as the judge and probe both address them.
       */
      const references = entry.seeds
        .map(function toReference(seed,) {
          return {
            seedId: seed.id,
            deletedText: seed.needle,
          };
        },);

      /**
       Zh-anchored judge verdicts over this entry's restored seeds.
       */
      const seedJudgments = await judge({
        client,
        judgeModelIds,
        sourceText: entry.sourceText,
        repairedText: result.repairedText,
        references,
        signal,
        perCallTimeoutMs,
        l: rl,
        stageSettings,
      },);

      /**
       Whether each deleted sentence was recoverable from the Chinese at all.

       Asked of the SOURCE and the deletion, never of the repaired text, so
       it is independent of whether the pipeline restored anything. A seed the
       source does not license cannot fairly count against detection: there is
       nothing to notice missing. The scorecard reports it as its own
       category rather than folding it into the headline rate.
       */
      const seedDerivability = await derivability({
        client,
        judgeModelIds,
        sourceText: entry.sourceText,
        references,
        signal,
        perCallTimeoutMs,
        l: rl,
        stageSettings,
      },);
      /* oxlint-enable no-await-in-loop */
      records.push({
        entryId: entry.entryId,
        outcomeKind: 'ok',
        status: result.status,
        seedJudgments,
        seedDerivability,
        seedGrades: Object.fromEntries(entry.seeds
          .map(function gradeSeed(seed,) {
            return [
              seed.id,
              measureSeedRestoration({
                needle: seed.needle,
                seededText,
                repairedText: result.repairedText,
              },),
            ];
          },),),
        seedDetection: gradeSeedDetection({
          sourceText: entry.sourceText,
          seededText,
          applications,
          issues: result.issues,
        },),
        issueCount: result.issues
          .length,
        resolvedIssueCount: result.issues
          .filter(function isResolved(record,) {
            return record.resolved;
          },)
          .length,
        detail: '',
        repairedText: result.repairedText,
      },);
    }
    catch (error) {
      // Aborts must always win so user steering can stop the benchmark, and
      // the run fails BECAUSE it was aborted: whichever torn-down exchange
      // surfaced, from the repair, the judge or the derivability probe, is a
      // symptom, and a caller tells an abort from a fault by identity alone
      // (as `repair-slice-buy.ts` and `translate-slice-attempt.ts` do).
      if (signal.aborted) {
        rl.warn(`${entry.entryId}: abandoned by the caller's abort (${exchangeFailureLogText({ error, },)})`,);
        throw signal.reason;
      }
      rl.warn(`${entry.entryId}: repair threw ${exchangeFailureLogText({ error, },)}`,);
      records.push({
        entryId: entry.entryId,
        outcomeKind: 'error',
        seedJudgments: {},
        seedDerivability: {},
        seedGrades: {},
        seedDetection: {},
        issueCount: 0,
        resolvedIssueCount: 0,
        detail: exchangeFailureText({ error, },),
      },);
    }
  }

  return {
    records,
    scorecard: computeRepairScorecard({ records, },),
  };
}

//endregion Repair benchmark
