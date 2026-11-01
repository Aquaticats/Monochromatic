import {
  readWindowDial,
  resolveStragglerGraceMs,
  STRAGGLER_GRACE_VAR,
} from './grace-override.ts';

//region Writer grace override
// Gives the WRITER rounds a longer straggler window than the reader rounds,
// built in since 2026-09-06, and lets one invocation move it without a
// rebuild.
//
// A WRITER ROUND LOSES A CANDIDATE; A READER ROUND LOSES A BALLOT. The editor,
// refiner, translate and consolidate gathers ask each seat for text, and every
// voice the window cuts is one fewer candidate for the judges to choose among.
// THE ARCHIVE BLOCK REVIEW IS ONE TOO (ledger B713, 2026-10-21): each reviewer
// answers with the complete corrected block, so a `revise` reply cut by the
// window is a whole candidate lost, the very reason the writers wait longer. No decision had
// classified that round, and it ran under the reader window; this session
// classified it under the owner's rule, open to the owner's veto
// (`doc/decision/translation-repair-straggler-grace.md`, addendum of
// 2026-10-21). The selection round after it keeps the reader window.
// The critic, panel, judge, checker and probe gathers ask eight or so seats for
// a reading of text that already exists, and the last voice to arrive adds one
// ballot to seven. The two are not the same wait, and the 2026-09-01 seating
// showed why one window cannot serve both: the editor calibration seated the
// two slowest models as editors on their text, and the four-entry pass then
// ran under `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS=60000` for the reader
// rounds' sake, which cut the top editor mid-reply in the pass's first editor
// round. A window sized for the eight-wide rounds unseats the writers the
// three-wide rounds were seated on.
//
// A BUILT-IN OF ITS OWN SINCE 2026-09-06. The dial landed on 2026-09-02 as a
// launch-time setting so the round window's decision record stayed unchanged,
// and every page that shipped afterwards ran its writers at 180000 ms through
// it while the round window was the owner's 120000 ms of 2026-09-03. A
// production launch that depended on an operator remembering the variable was
// the same defect the pass overlap fallback had, so the owner made the figure
// the built-in (`doc/decision/translation-repair-straggler-grace.md`, decision
// of 2026-09-06). Writers never wait less than every other round: when the
// round window is the longer one, as under the editor calibration's 300000 ms,
// the writers follow it.
//
// UNSET AND BLANK BOTH MEAN "the built-in, or the round window when that is
// longer", and anything else unreadable refuses as a stated refusal, by the
// rule `readWindowDial` holds for both dials.

/**
 Environment variable overriding the writer rounds' straggler window, in
 milliseconds.
 */
export const WRITER_GRACE_VAR = 'TRANSLATION_REPAIR_WRITER_GRACE_MS';

/**
 Milliseconds a writer round keeps waiting on stragglers after quorum when no
 launch dial says otherwise and the round window is shorter.

 THE OWNER'S DECISION OF 2026-09-06, on the four pages of 2026-09-04 that
 shipped with their writers at this window through the launch dial while the
 reader rounds waited `STRAGGLER_GRACE_MS`: writer-round cuts were 4, 6, 11
 and 20 against 28, 35, 26 and 135 reader-round cuts. Whether the round window
 would serve the writers was not measured as a matched pair, and the owner
 chose the configuration every shipped page ran over an unmeasured shorter
 one. Record: `doc/decision/translation-repair-straggler-grace.md`.
 */
export const WRITER_GRACE_MS = 180_000;

/**
 Stage labels of the rounds the dial governs, as their log lines spell them.

 Listed here rather than read off the stages, which take the writer window as
 a value (`StageSettings`), because the list exists for the launch note only.
 */
export const WRITER_STAGE_LABELS: readonly string[] = [
  'editor',
  'refiner',
  'translate',
  'produceConsolidations',
  'archive-block-review',
];

/**
 Reads the writer dial's text.

 @param fallback - window the writers run under when the text is blank

 @param raw - override text as the launch's environment holds it, `''` for an
 absent variable, since unset and empty are alike here; REQUIRED, so the
 environment is read where an entry file reads it and in no stage it feeds

 @returns Milliseconds a writer round keeps waiting on stragglers after quorum

 @throws {@link StatedRefusalError} when the override is present and is not
 a positive finite number of milliseconds

 @example
 ```ts
 const writerMs = resolveWriterGraceMs({ fallback: WRITER_GRACE_MS, raw: env[WRITER_GRACE_VAR] ?? '', },);
 ```
 */
export function resolveWriterGraceMs(
  {
    fallback,
    raw,
  }: {
    readonly fallback: number;
    readonly raw: string;
  },
): number {
  return readWindowDial({
    variable: WRITER_GRACE_VAR,
    fallback,
    raw,
    unsetMeans: 'give writer rounds the built-in writer window, or the round window when that is longer',
  },);
}

/**
 Both windows of one invocation and where the writers' came from.

 @example
 ```ts
 const grace: WriterGrace = { writerMs: 180_000, roundMs: 60_000, source: 'writer-dial', };
 ```
 */
export type WriterGrace = {
  /**
   Milliseconds a writer round keeps waiting on stragglers after quorum.
   */
  readonly writerMs: number;

  /**
   Milliseconds every other round keeps waiting.
   */
  readonly roundMs: number;

  /**
   `writer-dial` when a launch set the writer variable; `built-in` when the
   writers wait `WRITER_GRACE_MS` because nothing set them and the round
   window is shorter; `round-window` when the writers follow every other
   round, because that window is at least as long as the built-in.

   CARRIED RATHER THAN INFERRED from the two numbers differing: a note that
   blamed the writer dial whenever the two windows disagree would name an
   override nobody made, since the built-in writer window differs from the
   round window with no dial set at all.
   */
  readonly source: 'built-in' | 'round-window' | 'writer-dial';
};

/**
 Reads both windows of this invocation, the round window first because the
 writer window is measured against it.

 READ ONCE, BY THE ENTRY FILE, from the environment it hands in, and handed
 down to every stage as a value (`StageSettings`). Every gather once read
 both dials off the process's environment for itself, so the calibration
 could adopt its own window by writing the round variable after launch; the
 calibration now passes its own round fallback here instead.

 @param env - environment of the launch, which the entry file hands in

 @param roundFallback - window every other round runs under when the round
 dial is blank: `STRAGGLER_GRACE_MS`, or the calibration's own
 `CALIBRATION_STRAGGLER_GRACE_MS`

 @returns Both windows and the writers' source

 @throws {@link StatedRefusalError} when either dial is set to something that
 is not a whole number of milliseconds a timer can hold; the round dial is
 read first, so a run with both wrong hears about the round dial first

 @example
 ```ts
 const { writerMs, roundMs, source, } = readWriterGrace({ env: process.env, roundFallback: STRAGGLER_GRACE_MS, },);
 ```
 */
export function readWriterGrace(
  {
    env,
    roundFallback,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly roundFallback: number;
  },
): WriterGrace {
  /**
   Window every other round runs under.
   */
  const roundMs = resolveStragglerGraceMs({
    fallback: roundFallback,
    raw: env[STRAGGLER_GRACE_VAR] ?? '',
  },);

  /**
   What the launch set for the writers, blank when it set nothing.
   */
  const written = env[WRITER_GRACE_VAR] ?? '';

  if (written.trim() === '') {
    if (roundMs >= WRITER_GRACE_MS) {
      return {
        writerMs: roundMs,
        roundMs,
        source: 'round-window',
      };
    }
    return {
      writerMs: WRITER_GRACE_MS,
      roundMs,
      source: 'built-in',
    };
  }
  return {
    writerMs: resolveWriterGraceMs({
      fallback: WRITER_GRACE_MS,
      raw: written,
    },),
    roundMs,
    source: 'writer-dial',
  };
}

/**
 Explains which window the writer rounds are under whenever it is not the
 round window, whether the built-in or a launch dial put them there.

 PRINTED BY THE DRIVERS, beside the round window's own note, for the reason
 `graceOverrideNote` gives: a run must never hide which window it ran under,
 and a writer round that lost nobody under a longer window is exactly the
 round the seating wants to hear about.

 @param grace - both windows and the writers' source

 @returns Note naming both windows and the stages, or nothing when the
 writers follow every other round

 @example
 ```ts
 const note = writerGraceOverrideNote({ grace: stageSettings.grace, },);
 ```
 */
export function writerGraceOverrideNote(
  { grace, }: { readonly grace: WriterGrace; },
): string {
  if (grace.source === 'round-window')
    return '';

  /**
   Stages and both windows, the part every note carries.
   */
  const windows = `writer rounds (${WRITER_STAGE_LABELS.join(', ',)}) abandon stragglers `
    + `${String(grace.writerMs,)}ms after quorum rather than the ${String(grace.roundMs,)}ms every other round waits`;

  if (grace.source === 'built-in') {
    return `WRITER GRACE built in: ${windows}, the owner's decision of 2026-09-06; `
      + `${WRITER_GRACE_VAR} moves it for one launch`;
  }
  return `WRITER GRACE OVERRIDDEN by ${WRITER_GRACE_VAR}: ${windows}`;
}

//endregion Writer grace override
