import {
  listCorpusPeople,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  graceOverrideNote,
  writerGraceOverrideNote,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  readStageSettings,
  STRAGGLER_GRACE_MS,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { fetchTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type CommandLineOf,
  createRunClient,
  HARD_CAP_VAR,
  readHeadSha,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveHardCapMinutes,
  resolveRunsDir,
  resolveSpendCeilingUsd,
  SPEND_CEILING_USD,
  SPEND_CEILING_VAR,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  HARD_CAP_MINUTES,
  PASS_MS_PER_MINUTE,
} from './corpus-pass-limits.ts';
import { runCorpusPassOver, } from './corpus-pass-run.ts';
import { readDriftOptIn, } from '@monochromatic-dev/translation-repair-slice-cache/ts';
import { settleEntry, } from './pass-entry-settle.ts';
import { outsideReadsFrom, } from './pass-outside-reads.ts';
import { readPassOverlap, } from './pass-overlap.ts';
import { RUN_PICTURE_SOURCES, } from './pass-visual-evidence.ts';

//region Corpus pass
// Runs the pipeline over every complete zh/en corpus pair at the pinned commit
// (`corpus-pass-run.ts`). This file is the wiring only: it reads what a process
// has (the environment, the clock, the git tip, the client) and hands it over.
// Run it with `mise run //package/translation-repair/corpus-pass:corpus-pass` (append
// `-- --plan` for a zero-quota setup check).

/**
 Reads the launch's overrides and runs the pass over them.

 EVERY OVERRIDE IS READ HERE, INSIDE THE REFUSAL BOUNDARY, in the order a
 launch's mistakes should surface: the per-entry ceiling, the spend allowance,
 both windows, then the slice overlap, then the corpus pin, then the drift
 opt-in, which refuses nothing, before the lock is claimed and before anything
 is read, so an unreadable value refuses the pass before it claims a
 directory or spends anything, as a stated refusal and not as a fault at load
 (ledger B302; the pin was still read at load until 2026-10-10). Every one of
 them is handed on as a value, and no library code reads the environment for
 them (`stage-settings.ts`, `cap-override.ts`, `spend-ceiling.ts`,
 `pass-generation-guard.ts`).

 @param line - the pass's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await runCorpusPass({ line, },);
 ```
 */
async function runCorpusPass({ line, }: { readonly line: CommandLineOf<'corpus-pass'>; },): Promise<void> {
  /**
   Hard ceiling in milliseconds, after any environment override.

   OVERRIDABLE so the re-attempt queue can be exercised against an entry that
   fits in one run: the queue only does anything to an entry the cap CUTS, and
   the shipped ceiling means the smallest such entry needs thirteen hours.
   `cap-override.ts` carries why an unreadable override throws.
   */
  const hardCapMs = resolveHardCapMinutes({
    fallback: HARD_CAP_MINUTES,
    raw: process.env[HARD_CAP_VAR] ?? '',
  },) * PASS_MS_PER_MINUTE;

  /**
   USD this run may spend on the provider that bills in USD before it stops
   starting entries, after any environment override (`spend-ceiling.ts`).
   */
  const spendCeilingUsd = resolveSpendCeilingUsd({
    fallback: SPEND_CEILING_USD,
    raw: process.env[SPEND_CEILING_VAR] ?? '',
  },);

  /**
   Both straggler windows and the contest ledger every stage of the pass runs
   under, read once.
   */
  const stageSettings = readStageSettings({ env: process.env, },);

  /**
   Note naming the straggler window when it is not the built-in one.
   */
  const graceNote = graceOverrideNote({
    effectiveMs: stageSettings
      .grace
      .roundMs,
    builtInMs: STRAGGLER_GRACE_MS,
  },);

  /**
   Note naming the writer rounds' window when a launch gave them their own.
   */
  const writerNote = writerGraceOverrideNote({ grace: stageSettings.grace, },);

  /**
   Slices each entry keeps in flight, read once for every entry of the pass.
   */
  const overlap = readPassOverlap({ env: process.env, },);

  /**
   Clone and commit the pass reads, after any environment override, beside
   where each half came from.
   */
  const pinSetting = readRunCorpusPinSetting({ env: process.env, },);

  /**
   Whether the launch asked to resume across a foreign build
   (`pass-generation-guard.ts`).
   */
  const driftAllowed = readDriftOptIn({ env: process.env, },);

  return runCorpusPassOver({
    line,
    runsDir: await resolveRunsDir(),
    graceNote,
    writerNote,
    pinSetting,
    hardCapMs,
    spendCeilingUsd,
    driftAllowed,
    pipelineDir: import.meta.dirname,
    readTip: readHeadSha,
    env: process.env,
    transport: fetchTransport,
    newClient: createRunClient,
    settle: settleEntry,
    outsideReads: outsideReadsFrom({
      env: process.env,
      fetchFn: fetch,
      pin: pinSetting.pin,
      listPeople: listCorpusPeople,
      readFile: readCorpusFile,
    },),
    pictureSources: RUN_PICTURE_SOURCES,
    now: monotonicMs,
    overlap,
    stageSettings,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'corpus-pass',
    argv: process.argv,
    env: process.env,
    run: runCorpusPass,
  },);

//endregion Corpus pass
