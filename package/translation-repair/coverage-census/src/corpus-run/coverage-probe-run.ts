import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type CorpusPin,
  listCorpusPeople,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  listCoverageCandidates,
  type runCoverageStage,
} from '@monochromatic-dev/translation-repair-archive/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  askedAmong,
  type CommandLineOf,
} from '@monochromatic-dev/translation-repair-run/ts';
import { readCoverageProbeArguments, } from './coverage-probe-args.ts';
import { readPair, } from './coverage-probe-pair.ts';
import {
  answeredRow,
  failedRow,
  type ProbeRow,
  whereOf,
} from './coverage-probe-row.ts';
import { persistProbeRun, } from '@monochromatic-dev/translation-repair-probe/ts';
import type { RunnerClosure, } from '@monochromatic-dev/translation-repair-insertion/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Coverage probe run
// PROTOTYPE for question 28: can a roster tell a passage the translation merged
// from one it never rendered.
//
// WHAT IT MEASURES AND WHY IT IS WORTH QUOTA. It was established that neither
// aligner produces evidence of absence, at either scale, and that the
// deterministic substitutes are exhausted: heading Latin, section length and
// distinctive body tokens were each measured and none of them separates a
// translated passage from an absent one when the two sides share no characters.
// What is left is semantic. This asks the semantic question directly, over the
// passages the aligners actually refuse, and records what came back so the
// answer to question 28 rests on a measurement rather than on an expectation.
//
// IT DECIDES NOTHING. No slicing, no artifact and no lane reads its output.
//
// IT ASKS THE SHEET A PAGE WITH NO DECLARED NAMES GETS. It reads raw archives,
// not a prepared pair, so it has no declared identity to pass; production
// coverage has read the page's declared names since ledger B28, and a rerun
// measures the sheet without them.
//
// IT DOES KEEP ITS ANSWERS, which it did not always. It used to print rows to
// standard output and write nothing, on the reasoning that a caller would
// redirect them wherever the measurement was being kept. Nobody did, and both
// scales were probed on 2026-08-16 at real quota cost, and those numbers
// survived only in session transcripts. Every run now lands in
// the runs directory under `coverage-probe/` as well, carrying the corpus pin,
// the roster and the pipeline digest that produced it.
//
// AND THAT REDIRECT WOULD NOT HAVE WORKED ANYWAY, which is worth knowing
// because it is why the file is the answer rather than better discipline.
// Measured at the boundary on 2026-08-17: this module's tagged logger writes to
// STANDARD OUTPUT, the same stream the rows JSON goes to, and it logs a line
// per candidate as it goes. So `coverage-probe > rows.json` yields progress
// lines wrapped around a JSON document, which no parser accepts. The rows have
// never been recoverable that way for any run that actually probed anything.
//
// STANDARD OUTPUT IS OTHERWISE UNCHANGED and gains only the line saying where
// the file went, so nothing a caller does today breaks.

/**
 Asks the roster about every unpaired passage it is given, up to the cap.

 @param line - probe's command line, read whole by `reportingRefusals`

 @param pin - corpus clone and commit every read resolves against

 @param newClient - builds the client for every exchange, called after the
 command line is read, so a bad flag is refused before a key is asked for

 @param stage - roster's answer about one passage

 @param roster - models asked

 @param exchangeTimeoutMs - deadline per exchange

 @param readDigest - digest over built output, read at the start of the run,
 not the end: a rebuild mid-run would otherwise stamp a build that never ran

 @param readClosure - chunks the executing entry imports, read at the start
 for the same reason

 @param resolveRunsDir - directory the answers are kept under, resolved when
 they are kept

 @param now - present instant as ISO 8601, read when the run begins and
 when it ends

 @param log - logger every progress line goes to

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws StatedRefusalError when the command line is malformed, or when an
 entry named is no entry of the corpus at the pin

 @example
 ```ts
 await runCoverageProbe({ line, pin, newClient, stage, roster, exchangeTimeoutMs, readDigest, readClosure, resolveRunsDir, now, log, stageSettings, },);
 ```
 */
export async function runCoverageProbe(
  {
    line,
    pin,
    newClient,
    stage,
    roster,
    exchangeTimeoutMs,
    readDigest,
    readClosure,
    resolveRunsDir,
    now,
    log,
    stageSettings,
  }: {
    readonly line: CommandLineOf<'coverage-probe'>;
    readonly pin: CorpusPin;
    readonly newClient: () => ModelClient;
    readonly stage: typeof runCoverageStage;
    readonly roster: readonly RosterModelId[];
    readonly exchangeTimeoutMs: number;
    readonly readDigest: () => Promise<string>;
    readonly readClosure: () => Promise<RunnerClosure>;
    readonly resolveRunsDir: () => Promise<string>;
    readonly now: () => string;
    readonly log: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   When this run began, read before any work so the record dates the run
   rather than the moment it happened to finish.
   */
  const startedAt = now();

  /**
   Digest over built output, which is the only identity that moves when the
   code moves but the commit does not.

   READ AT THE START, not at the end. A long run gives a developer plenty of
   time to rebuild, and `rendering-audit-settled` was caught doing exactly
   that: `dist` was rebuilt while a run was in flight, so the digest it was
   about to stamp described a build that had never probed anything. Node loads
   the code once, at startup; the identity that answers for a run is the one
   present THEN.
   */
  const pipelineDigest = await readDigest();

  /**
   Chunks this entry imports, read from the executing file at run START for
   the same reason the digest is: a rebuild mid-run would otherwise stamp a
   build that never ran.
   */
  const runnerClosure = await readClosure();

  /**
   Entry filter and candidate cap.
   */
  const {
    onlyIds,
    cap,
  } = readCoverageProbeArguments({ line, },);

  /**
   Client for every exchange.
   */
  const client = newClient();

  /**
   Abort shared by every call, never fired: each exchange has its own deadline.
   */
  const controller = new AbortController();

  /**
   Rows accumulated across candidates, one per ATTEMPT: a candidate whose call
   failed keeps a row saying so, since a measurement that drops its failures
   reports a success rate of one.
   */
  const rows: ProbeRow[] = [];

  /**
   Entries to walk, filtered when the caller named some.
   */
  const entryIds = askedAmong({
    asked: onlyIds,
    known: await listCorpusPeople({ pin, },),
    source: '--only',
    within: 'the corpus at the pin',
  },);
  /* oxlint-disable no-await-in-loop -- Sequential on purpose: this probe exists
     to be read while it runs, and a fan-out over entries would interleave the
     progress of several documents into one stream. */
  for (const entryId of entryIds) {
    if (rows.length >= cap)
      break;

    /**
     Both sides at the pin, or nothing when this entry lacks one.
     */
    const texts = await readPair({
      pin,
      entryId,
      log,
    },);
    if (texts.kind === 'missing')
      continue;

    /**
     Translation, parsed once and used both as the searched text and as what
     every quote is anchored against.
     */
    const target = parseDocument({ text: texts.target, },);

    /**
     Passages this entry's aligners refuse.
     */
    const candidates = listCoverageCandidates({
      source: parseDocument({ text: texts.source, },),
      target,
    },);
    if (candidates.length === 0)
      continue;

    log.info(`${entryId}: ${String(candidates.length,)} unpaired ${
      wordForCount({
        count: candidates.length,
        one: 'passage',
        many: 'passages',
      },)
    }`,);
    for (const candidate of candidates) {
      if (rows.length >= cap)
        break;

      /**
       Where this passage sits, in the same terms the censuses print.
       */
      const where = whereOf({ candidate, },);
      try {
        /**
         What the roster concluded about it.
         */
        const answer = await stage({
          client,
          modelIds: roster,
          sourcePassage: candidate.sourceText,
          translation: target,
          signal: controller.signal,
          exchangeTimeoutMs,
          l: log,
          stageSettings,
        },);
        rows.push(answeredRow({
          entryId,
          candidate,
          answer,
        },),);
        /**
         Verdict of this candidate, read once for the progress line.
         */
        const { verdict, } = answer;
        log.info(
          `${entryId} ${where}: ${verdict.kind} (full ${String(verdict.anchoredFull,)}, `
            + `partial ${String(verdict.anchoredPartial,)}, absent ${String(verdict.absent,)}, `
            + `unanchored ${String(verdict.unanchored,)}, `
            + `heard ${String(verdict.heard,)} of ${String(verdict.asked,)})`,
        );
      }
      catch (error) {
        // Reported rather than fatal, the same way the translate probe learned
        // to be: one slow call must not cost every later candidate.
        rows.push(failedRow({
          entryId,
          candidate,
          asked: roster.length,
          error,
        },),);
        log.info(`${entryId} ${where}: FAILED ${exchangeFailureLogText({ error, },)}`,);
      }
    }
  }
  /* oxlint-enable no-await-in-loop */


  /**
   Where this run was kept, said out loud so the answers are findable without
   searching a runs directory for them.
   */
  const keptAt = await persistProbeRun({
    runsDir: await resolveRunsDir(),
    probeName: 'coverage-probe',
    run: {
      startedAt,
      finishedAt: now(),
      pipelineDigest,
      runnerClosure,
      roster,
      subject: {
        // THE COMMIT, not the whole pin: the pin also carries a local clone
        // directory, which names this machine rather than the corpus and is
        // worth nothing to a later reader holding the file.
        corpusPin: pin.commitSha,
        entriesWalked: entryIds,
        entriesRequested: onlyIds,
        candidateCap: cap,
      },
      rows,
    },
  },);
  log.info(`kept ${String(rows.length,)} ${
    wordForCount({
      count: rows.length,
      one: 'row',
      many: 'rows',
    },)
  } at ${keptAt}`,);

  // STANDARD OUTPUT STAYS. Redirecting it is the workflow this probe shipped
  // with, and removing it would trade one lost measurement for another.
  console.log(JSON.stringify(
    { rows, },
    undefined,
    2,
  ),);
}

//endregion Coverage probe run
