import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type CoverageVerdict,
  runCoverageStage,
} from '@monochromatic-dev/translation-repair-archive/ts';
import {
  type AnchorTarget,
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { decoyCut, } from './coverage-control-decoy.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Coverage control
// Whether the coverage roster can vote absence AT ALL, asked before its
// unanimity is read as evidence of anything.
//
// One entry's ninety-six block-scale answers carried not one vote for
// absence. TWO DIFFERENT THINGS PRODUCE EXACTLY THAT READING: a translation
// that genuinely carries every passage, and a wire whose evidence rule admits
// any non-empty quote, so nothing can ever be found missing. Nothing recorded
// separates them, and the second would make the null an artifact of the
// instrument rather than a fact about the corpus.
//
// THE DAMAGE IS THE ROSTER'S OWN EVIDENCE. Coverage candidates are by
// construction the passages the aligners REFUSE to pair, so nothing outside the
// roster's own answer says which target text renders one. Asking first and then
// deleting exactly the spans it pointed at is the only cut known to remove the
// rendering, rather than some text that happened to sit near it.
//
// SPENDS QUOTA: two roster rounds per case.
//
// IT ASKS THE SHEET A PAGE WITH NO DECLARED NAMES GETS: its cases are cut from
// raw archives with no prepared identity, while production coverage has read
// the page's declared names since ledger B28.

/**
 Cases the control is tried on.

 Several rather than one, because a single case answered by a coin is
 indistinguishable from a wire that works; few rather than many, because this
 gates a reading rather than being the measurement itself.
 */
const CONTROL_CASES = 3;

/**
 One passage, with the translation it is asked about.
 */
export type CoverageControlCase = {
  /**
   Where the passage sits, in the terms the probe's rows print.
   */
  readonly where: string;

  /**
   Original-side text whose coverage is in question.
   */
  readonly sourcePassage: string;

  /**
   Whole translation, undamaged, as the first round sees it.
   */
  readonly translation: AnchorTarget;
};

/**
 What deleting one passage's rendering did to its verdict.
 */
export type CoverageControlRow = {
  /**
   Where the passage sits.
   */
  readonly where: string;

  /**
   Verdict before the rendering was deleted, always `carried`, since the
   control skips cases it cannot damage.
   */
  readonly before: CoverageVerdict['kind'];

  /**
   Verdict once the rendering was gone.
   */
  readonly after: CoverageVerdict['kind'];

  /**
   Voices reporting nothing rendered the passage, before the deletion.
   */
  readonly absentBefore: number;

  /**
   Voices reporting nothing rendered the passage, after it.

   THIS IS THE NUMBER THE CONTROL TURNS ON, rather than the verdict kind: the
   recorded null is about ballots, not about how they were rolled up, so a
   wire that produces a single absence vote it did not produce before has
   shown the vote to be reachable.
   */
  readonly absentAfter: number;

  /**
   Verdict when an EQUALLY LARGE cut was taken where the roster did not point,
   or `no-room` when the page had nowhere to take one clear of the anchored
   spans.

   A sound wire keeps saying `carried` here: the rendering is untouched, so
   nothing about the passage changed. This is what separates a wire that reads
   the passage from one that answers `absent` to any damaged document.
   */
  readonly decoy: CoverageVerdict['kind'] | 'no-room';

  /**
   Voices reporting nothing rendered the passage, after the decoy cut.
   */
  readonly absentAfterDecoy: number;

  /**
   Offset the decoy cut was taken at, or `-1` when there was no room.

   KEPT so a decoy that does move the verdict can be diagnosed rather than
   guessed at: a cut landing on a title or a frontmatter block is structural
   damage of a different kind, and its offset is what says so.
   */
  readonly decoyAt: number;

  /**
   Spans deleted from the translation.
   */
  readonly removedSpans: number;

  /**
   Characters the deletion took out, so a cut that removed almost nothing
   cannot be mistaken for one that removed a passage.
   */
  readonly removedChars: number;
};

/**
 Why a case could not be damaged.

 TWO DIFFERENT MEANINGS used to print as one line saying "not damageable", and
 the difference is the whole question. A roster that did not say `carried` is
 a roster VOTING ABSENCE on undamaged corpus text, which is the strongest form
 of the thing this control was built to look for. A cut that leaves nothing of
 the page is a page with no second question in it, and says nothing about
 coverage.
 */
export type CoverageControlRefusal = {
  /**
   Where the passage sits.
   */
  readonly where: string;

  /**
   `not-carried` when the roster declined to call it covered before any damage
   was done; `cut-left-nothing` when it did, and deleting the spans it anchored
   on deleted every character of the translation, so no page was left to ask
   it about a second time.

   THE SPANS OF A CARRIED VERDICT ARE ALWAYS FOUND, so there is no reason for
   spans that could not be: a carried verdict's evidence is the page's own
   text for each region a full claim anchored (`judgeCoverage`), which
   `tryCase` checks before it cuts. Until 2026-10-05 the emptied page was
   reported as `evidence-not-locatable`, the name of that unreachable state,
   when every span it offered had been found.
   */
  readonly reason: 'not-carried' | 'cut-left-nothing';

  /**
   Verdict the roster reached on the undamaged page.
   */
  readonly verdict: CoverageVerdict['kind'];

  /**
   Voices reporting nothing rendered the passage, undamaged.
   */
  readonly absent: number;

  /**
   Spans the roster offered as evidence.
   */
  readonly offeredSpans: number;
};

/**
 Decoy round reduced to what the row records.

 The no-room case is given the shape of a verdict so the row does not have to
 branch twice over one condition, once per field.
 */
type DecoyReading = {
  /**
   Verdict kind, or `no-room` when the page had nowhere to take the cut.
   */
  readonly kind: CoverageVerdict['kind'] | 'no-room';

  /**
   Voices reporting nothing rendered the passage.
   */
  readonly absent: number;
};

/**
 Whether the wire proved able to see the damage, with its working.
 */
export type CoverageControlResult = {
  /**
   Whether a majority of tried cases voted absence once their rendering was
   deleted, at least one equally large cut was taken elsewhere, and no more
   than half of the cuts taken elsewhere drew the vote too.
   */
  readonly held: boolean;

  /**
   Cases where deleting the anchored spans produced absence votes that were
   not there before.
   */
  readonly sawAbsenceOnTarget: number;

  /**
   Cases where the equally large cut taken ELSEWHERE also produced them,
   which a sound wire keeps at zero.
   */
  readonly sawAbsenceOnDecoy: number;

  /**
   Cases an equally large cut was taken on at all.

   A page with no room for one clear of the anchored spans takes none and
   records `no-room`, which carries no absence vote and so reads like a decoy
   that stayed quiet. The decoy half is therefore judged over this count
   rather than over the rows, and a control where it is zero never ran its
   decoy half and does not hold whatever its targeted cuts showed.
   */
  readonly decoysTaken: number;

  /**
   Every case the control managed to damage and re-ask.
   */
  readonly rows: readonly CoverageControlRow[];

  /**
   Cases it could not damage, each carrying why.

   NOT A FAILURE LOG. A page whose passages the roster never calls covered
   produces nothing to damage and is reported entirely here, and that is a
   result rather than an empty run.
   */
  readonly refusals: readonly CoverageControlRefusal[];
};

/**
 Deletes named spans from a document, or reports that none were there.

 Exported so the cut can be tested directly. This decides what the control is
 actually asking about, and a version that quietly returned the text unchanged
 would turn the whole gate into a formality that passes whatever it is handed.

 BLANK ANSWERS TWO STATES: no span was present, and the spans were the whole
 document. A caller that has to tell them apart checks that its spans are
 present before it cuts, as `tryCase` does.

 A SPAN THAT SHOWS A READER NOTHING IS NO SPAN, asked of `rendersAsNothing`
 (ledger B40) as an empty one is: cut as a span, one space took every space
 of the document with it, and a zero-width space or a Hangul filler counted
 as rendering deleted where no reader saw any.

 @internal

 @param text - document to cut from

 @param spans - exact document text of each region to remove

 @returns Text with every span gone; blank when no span was present, and
 blank too when the spans were all of it

 @example
 ```ts
 const damaged = withoutSpans({ text, spans, },);
 ```
 */
export function withoutSpans(
  {
    text,
    spans,
  }: {
    readonly text: string;
    readonly spans: readonly string[];
  },
): string {
  /**
   Spans worth cutting, longest first.

   ORDER MATTERS: a short span sitting inside a longer one would be gone
   already by the time its own turn came, and the count of what was removed
   would then depend on which order the roster happened to answer in.
   */
  const ordered = spans
    .filter(function isCuttable(span,): boolean {
      return !rendersAsNothing({ text: span, },);
    },)
    .toSorted(function longestFirst(
      left,
      right,
    ): number {
      return right.length - left.length;
    },);

  /**
   Document with every span taken out, all occurrences of each.
   */
  const cut = ordered.reduce(
    function without(
      standing,
      span,
    ): string {
      return standing
        .split(span,)
        .join('',);
    },
    text,
  );

  if (cut === text)
    return '';

  return cut;
}

/**
 Asks one case before and after its rendering is deleted.

 @param client - injected model client

 @param probe - passage and the translation it is asked about

 @param modelIds - roster asked, the same one the reading under test used

 @param signal - cancellation

 @param exchangeTimeoutMs - deadline per exchange

 @param l - logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Row for this case, or the refusal saying why it could not be damaged

 @throws Error when a carried verdict offers no span, or one the translation
 does not hold, which `judgeCoverage` cannot produce

 @example
 ```ts
 const row = await tryCase({ client, probe, modelIds, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
async function tryCase(
  {
    client,
    probe,
    modelIds,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly probe: CoverageControlCase;
    readonly modelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<CoverageControlRow | CoverageControlRefusal> {
  /**
   What the roster says about the passage as it stands.
   */
  const before = await runCoverageStage({
    client,
    modelIds,
    sourcePassage: probe.sourcePassage,
    translation: probe.translation,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   That answer's verdict, read out once so no expression walks two members of
   it at a time.
   */
  const { verdict: beforeVerdict, } = before;

  /**
   Spans the roster anchored on, which are exactly what the cut removes.
   */
  const { evidence, } = beforeVerdict;

  // Only a `carried` case can have its rendering deleted, because only that
  // verdict claims there is one. Anything else is the wire ALREADY declining to
  // say the passage is covered, with no damage done to it, which is reported
  // rather than skipped.
  if (beforeVerdict.kind !== 'carried')
    return {
      where: probe.where,
      reason: 'not-carried',
      verdict: beforeVerdict.kind,
      absent: beforeVerdict.absent,
      offeredSpans: evidence.length,
    };

  /**
   Translation as it stands, kept so the size of the cut can be reported.
   */
  const { text: standingText, } = probe.translation;

  /**
   Whether every span the roster anchored on is text this translation holds.

   A CARRIED VERDICT ALWAYS OFFERS SUCH SPANS. `judgeCoverage` calls a passage
   carried only on at least one full claim it could anchor, and fills
   `evidence` with the document's own text from the first anchor of each such
   claim to its last. The document it was handed is this translation
   (`runCoverageStage` passes it through), and an anchor is a located quote's
   share of one block, which `locateQuote` never makes from a quote that
   shows a reader nothing. Blank here is asked as `withoutSpans` asks it.
   */
  const everySpanPresent = (evidence.length > 0)
    && evidence.every(function isPresent(span,): boolean {
      return (!rendersAsNothing({ text: span, },)) && standingText.includes(span,);
    },);
  if (!everySpanPresent)
    throw new Error(
      'unreachable: a carried verdict offered no span, or a span that is blank or is not in the translation it '
        + 'was judged against; judgeCoverage fills evidence with that translation\'s own text for each full '
        + 'claim it anchored, so every span can be found and cut',
    );

  /**
   Translation with every span the roster anchored on taken out.
   */
  const damagedText = withoutSpans({
    text: standingText,
    spans: evidence,
  },);

  // BLANK HERE MEANS EVERY CHARACTER A READER SEES WENT. `withoutSpans` answers
  // blank for a document no span was present in as well, which
  // `everySpanPresent` has ruled out, so this is a page the anchored spans
  // covered whole: nothing of it remains to ask the roster about a second
  // time. What remains is asked of `rendersAsNothing` (ledger B40), since a
  // page ends in a line break no span covers: asked of an empty-string
  // comparison, a one-sentence page whose sentence the roster quoted left
  // `\n`, and the roster was asked a second round about it.
  if (rendersAsNothing({ text: damagedText, },))
    return {
      where: probe.where,
      reason: 'cut-left-nothing',
      verdict: beforeVerdict.kind,
      absent: beforeVerdict.absent,
      offeredSpans: evidence.length,
    };

  /**
   What the roster says once the rendering it pointed at is gone.
   */
  const after = await runCoverageStage({
    client,
    modelIds,
    sourcePassage: probe.sourcePassage,
    translation: parseDocument({ text: damagedText, },),
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   Verdict once the rendering was gone.
   */
  const { verdict: afterVerdict, } = after;

  /**
   Characters the targeted cut took, which the decoy cut matches exactly.
   */
  const removedChars = standingText.length - damagedText.length;

  /**
   Where an equally large cut can be taken clear of the anchored spans.
   */
  const {
    span: decoySpan,
    at: decoyAt,
  } = decoyCut({
    text: standingText,
    avoid: evidence,
    chars: removedChars,
  },);

  /**
   What the roster says with that unrelated cut made instead.

   SPLICED BY OFFSET rather than by text, so exactly as many characters go as
   the targeted cut took. Deleting by span would take every occurrence and the
   two cuts would stop being the same size.
   */
  const decoyAnswer = (decoySpan === '')
    ? 'no-room' as const
    : await runCoverageStage({
      client,
      modelIds,
      sourcePassage: probe.sourcePassage,
      translation: parseDocument({
        text: standingText.slice(
          0,
          decoyAt,
        ) + standingText.slice(decoyAt + decoySpan.length,),
      },),
      signal,
      exchangeTimeoutMs,
      l,
      stageSettings,
    },);

  /**
   That answer read as a verdict.
   */
  const decoyVerdict = (function readDecoy(): DecoyReading {
    if (decoyAnswer === 'no-room')
      return {
        kind: 'no-room',
        absent: 0,
      };

    /**
     Verdict the decoy round returned.
     */
    const { verdict, } = decoyAnswer;

    return {
      kind: verdict.kind,
      absent: verdict.absent,
    };
  })();

  return {
    where: probe.where,
    before: beforeVerdict.kind,
    after: afterVerdict.kind,
    absentBefore: beforeVerdict.absent,
    absentAfter: afterVerdict.absent,
    decoy: decoyVerdict.kind,
    absentAfterDecoy: decoyVerdict.absent,
    decoyAt,
    removedSpans: evidence.length,
    removedChars,
  };
}

/**
 Asks whether deleting a passage's rendering changes what the roster votes.

 @param client - injected model client

 @param cases - passages to try, of which the first few damageable ones are used

 @param modelIds - roster asked

 @param signal - cancellation

 @param exchangeTimeoutMs - deadline per exchange

 @param l - logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Whether the wire voted absence once the rendering was gone

 @example
 ```ts
 const control = await coverageControlHolds({ client, cases, modelIds, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function coverageControlHolds(
  {
    client,
    cases,
    modelIds,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly cases: readonly CoverageControlCase[];
    readonly modelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<CoverageControlResult> {
  /**
   Cases that were damaged and re-asked.
   */
  const rows: CoverageControlRow[] = [];

  /**
   Cases that could not be damaged, with why.
   */
  const refusals: CoverageControlRefusal[] = [];

  for (const probe of cases) {
    if (rows.length >= CONTROL_CASES)
      break;

    /* oxlint-disable eslint/no-await-in-loop -- sequential by design: this gate decides whether a reading may be trusted, and the cases must meet the same provider conditions as each other for their agreement to mean anything */
    /**
     This case's before and after, or a refusal.
     */
    const row = await tryCase({
      client,
      probe,
      modelIds,
      signal,
      exchangeTimeoutMs,
      l,
      stageSettings,
    },);
    /* oxlint-enable eslint/no-await-in-loop */

    if (!('after' in row)) {
      refusals.push(row,);
      console.log(
        `COVERAGE control ${row.where}: ${row.reason} (undamaged verdict ${row.verdict}, ${
          String(row.absent,)
        } absence ${
          wordForCount({
            count: row.absent,
            one: 'vote',
            many: 'votes',
          },)
        }, ${String(row.offeredSpans,)} ${
          wordForCount({
            count: row.offeredSpans,
            one: 'span',
            many: 'spans',
          },)
        } offered)`,
      );
      continue;
    }

    rows.push(row,);
    console.log(
      `COVERAGE control ${row.where}: ${row.before} -> ${row.after}, absence votes ${
        String(row.absentBefore,)
      } -> ${String(row.absentAfter,)}, cut ${String(row.removedSpans,)} ${
        wordForCount({
          count: row.removedSpans,
          one: 'span',
          many: 'spans',
        },)
      } of ${
        String(row.removedChars,)
      } ${
        wordForCount({
          count: row.removedChars,
          one: 'char',
          many: 'chars',
        },)
      }; DECOY of the same size at ${String(row.decoyAt,)}: ${row.decoy}, absence votes ${
        String(row.absentAfterDecoy,)
      }`,
    );
  }

  /**
   Cases where deleting the rendering produced absence votes that were not
   there before.
   */
  const sawAbsenceOnTarget = rows
    .filter(function noticed(row,): boolean {
      return row.absentAfter > row.absentBefore;
    },)
    .length;

  /**
   Cases where the cut taken ELSEWHERE produced them too.
   */
  const sawAbsenceOnDecoy = rows
    .filter(function criedWolf(row,): boolean {
      return row.absentAfterDecoy > row.absentBefore;
    },)
    .length;

  /**
   Cases a decoy cut was taken on: every row but those whose page had no room
   for one.
   */
  const decoysTaken = rows
    .filter(function tookDecoy(row,): boolean {
      return row.decoy !== 'no-room';
    },)
    .length;

  // SAID WHERE THE ROWS ARE PRINTED, since a reader of this run otherwise sees
  // only that the control did not hold, beside targeted cuts that all moved.
  if ((rows.length > 0) && (decoysTaken === 0))
    console.log(
      `COVERAGE control CANNOT HOLD: a decoy cut was taken on none of ${String(rows.length,)} damaged ${
        wordForCount({
          count: rows.length,
          one: 'case',
          many: 'cases',
        },)
      }, no page having room for one clear of the spans the roster anchored on, so nothing shows whether an `
        + 'unrelated cut of the same size moves the vote too',
    );

  // A MAJORITY RATHER THAN UNANIMITY on the targeted cut, for the same reason
  // the editor width control uses one: a passage may be genuinely paraphrased
  // somewhere else in the document, and demanding a clean sweep would fail a
  // working wire on it.
  //
  // BOTH HALVES ARE REQUIRED. A wire that votes absence on the targeted cut has
  // only shown the vote reachable; one that votes it on the decoy too is
  // answering the damage rather than the question, and its absence votes carry
  // no information about coverage either way.
  //
  // A DECOY HALF THAT NEVER RAN IS NOT A QUIET ONE. A row whose page had no
  // room for a decoy cut records `no-room` and no absence vote, and until
  // 2026-10-05 the decoy half was taken over every row, so such a row counted
  // exactly as a decoy that was taken and moved nothing: a control made only
  // of such rows held, and so did one whose single decoy drew the vote beside
  // two rows with no room. The decoy half is now taken over the decoys that
  // ran, and at least one has to have run.
  //
  // NO ROWS CANNOT HOLD. A page offering nothing to damage has not shown the
  // absence vote reachable UNDER DAMAGE, whatever its refusals say on their own,
  // and reporting that as a held control would be the empty-run claim this
  // whole family exists to avoid.
  return {
    held: (rows.length > 0)
      && ((sawAbsenceOnTarget * 2) > rows.length)
      && (decoysTaken > 0)
      && ((sawAbsenceOnDecoy * 2) <= decoysTaken),
    sawAbsenceOnTarget,
    sawAbsenceOnDecoy,
    decoysTaken,
    rows,
    refusals,
  };
}

//endregion Coverage control
