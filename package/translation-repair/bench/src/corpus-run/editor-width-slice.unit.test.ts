/**
 Tests for one slice run at both editor widths.

 THREE ARMS RUN, NOT TWO, and that is the property the whole width probe rests
 on. The third arm repeats the NARROW roster so the comparison has a null band
 of its own: without it, the headline number, how often widening changed the
 shipped text, cannot be told apart from the lane simply disagreeing with
 itself run to run. A version that ran two arms and reported
 `narrowRepeatAgreed: true` would not fail loudly. It would produce a confident
 number from a comparison with no band under it, and every reading of the width
 question would tilt toward width mattering.

 SO THE COUNT IS THE ASSERTION. Both fixtures in this file count editor calls per
 seat, and the seats are asymmetric on purpose: two narrow against three wide,
 so three arms cost seven calls where two would cost five. A count is the only
 evidence that separates them, because both shapes return a well-formed row.

 THE STUB CLIENT COSTS NO QUOTA. `runWidthSlice` takes its client, so every
 call here is answered in-process, and the replies are validated against the
 live request's own wire guard rather than assumed to fit it.

 A PANEL THAT ALWAYS PICKS THE FIRST SEAT IS REPORTED AS `position-decided`,
 never as a win. That is the anti-laundering guard: the head-to-head runs both
 seating orders precisely so a preference for the seat cannot be published as a
 preference for the text, and the churning fixture drives exactly that case.

 Fixtures are cat-themed invention, and the model identifiers are real catalog
 seats because a roster is checked against the catalog.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type {
  AdjudicatedIssue,
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  runWidthSlice,
  type WidthProbeInput,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Editor width slice tests

/**
 Logger the arms write to.
 */
const l = tagged({ tag: 'editor-width-slice-test', },);

/**
 Translation the editors repair.
 */
const TARGET_TEXT = 'The cat is doing the sleeping on the windowsill.';

/**
 Original the edits answer to, in the Simplified Chinese the corpus uses.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。';

/**
 Seats both arms share.

 The narrow arm IS these two, so the repeat arm calls exactly these seats
 again and the wide arm calls them plus one more.
 */
const SHARED_EDITORS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Seat only the wide arm adds, which is the variable under test.
 */
const WIDE_ONLY_EDITOR = SEAT_HYPER_OPENROUTER_UNMEASURED;

/**
 Narrow roster.
 */
const NARROW_EDITORS: readonly RosterModelId[] = SHARED_EDITORS;

/**
 Wide roster, one seat larger so three arms and two arms cost different
 numbers of calls.
 */
const WIDE_EDITORS: readonly RosterModelId[] = [
  ...SHARED_EDITORS,
  WIDE_ONLY_EDITOR,
];

/**
 Panel, held fixed across every arm, and disinterested: no seat here edits.
 */
const JUDGES: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Editor calls three arms cost at these rosters.

 Spelled as the sum it is so the reader can see which arm each term pays for,
 rather than checking a bare seven.
 */
const THREE_ARM_EDITOR_CALLS = NARROW_EDITORS.length
  + NARROW_EDITORS.length
  + WIDE_EDITORS.length;

/**
 Single envelope covering the whole slice.
 */
const ENVELOPES: readonly EditableEnvelope[] = [
  {
    envelopeId: 'envelope/0',
    startOffset: 0,
    endOffset: TARGET_TEXT.length,
    baseText: TARGET_TEXT,
    baseHash: hashContent({ content: TARGET_TEXT, },),
    issueIds: ['adjudicated/tense',],
  },
];

/**
 Issues the slice carries: one the panel accepted and one it did not.

 BOTH ARE HERE so the row's `acceptedIssues` column has something to be wrong
 about. That column names the work available to the editors, and counting every
 issue instead would report a slice as having twice the work it had.
 */
const ISSUES: readonly AdjudicatedIssue[] = [
  {
    issueId: 'adjudicated/tense',
    status: 'accepted' as const,
    severity: 'major' as const,
    claims: [],
    tallies: {},
  },
  {
    issueId: 'adjudicated/hearsay',
    status: 'rejected' as const,
    severity: 'minor' as const,
    claims: [],
    tallies: {},
  },
];

/**
 Slice the probe runs, assembled once because no case varies it.
 */
const INPUT: WidthProbeInput = {
  entryId: 'whiskers',
  sliceIndex: 3,
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT,
  issues: ISSUES,
  envelopes: ENVELOPES,
  findings: [],
};

/**
 Repairs the shared seats propose, one per time each is asked.

 KEYED BY HOW OFTEN A SEAT HAS BEEN ASKED, not by arm, so the schedule the
 runner happens to use cannot decide the fixture. Each shared seat is asked
 exactly once per arm, so the narrow arm reads the first entry, the repeat arm
 the second, and the wide arm the third: the lane disagrees with itself every
 time, which is what the null band exists to catch.

 Each is a single-word swap, well inside what the preservation gate allows
 without a licensed quote.
 */
const ROUND_TEXTS: readonly string[] = [
  'The cat is doing the napping on the windowsill.',
  'The cat is doing the dozing on the windowsill.',
  'The cat is doing the drowsing on the windowsill.',
];

/**
 What a stub client recorded while the arms ran.
 */
type CallLog = {
  /**
   Calls answered by an editor seat.
   */
  editors: number;

  /**
   Calls answered by a panel seat.
   */
  judges: number;

  /**
   How often each seat has been asked, which drives the churning replies.
   */
  readonly perSeat: Map<string, number>;
};

/**
 Opens an empty call log.

 @returns Fresh log

 @example
 ```ts
 const log = freshLog();
 ```
 */
function freshLog(): CallLog {
  return {
    editors: 0,
    judges: 0,
    perSeat: new Map<string, number>(),
  };
}

/**
 Validates a scripted reply against the live request's wire guard and wraps it
 as an outcome.

 @param reply - scripted reply for this call

 @param request - live request, whose guard the reply must satisfy

 @returns Outcome carrying the validated reply

 @throws {@link Error} when the fixture itself fails the guard it is meant to
 satisfy, which is a defect in the fixture rather than in the code under test

 @example
 ```ts
 return replyWith({ reply: { edits: [], }, request, },);
 ```
 */
function replyWith<ValueT,>(
  {
    reply,
    request,
  }: {
    readonly reply: unknown;
    readonly request: ChatJsonRequest<ValueT>;
  },
): ChatJsonOutcome<ValueT> {
  if (!request.validate(reply,))
    throw new Error('scripted reply failed the wire guard',);
  return {
    kind: 'ok',
    value: reply,
    rawText: JSON.stringify(reply,),
  };
}

/**
 Whether a seat is one of the panel's.

 @param modelId - seat that was asked

 @returns Whether the panel holds it

 @example
 ```ts
 const judging = isJudge({ modelId: request.modelId, },);
 ```
 */
function isJudge({ modelId, }: { readonly modelId: RosterModelId; },): boolean {
  return JUDGES.includes(modelId,);
}

/**
 Builds a client whose editors answer from a script and whose panel always
 names the FIRST candidate on whatever slate it is shown.

 A first-seat panel is deliberate rather than lazy: it is the reading the
 head-to-head has to survive, since a preference for the seat must never be
 published as a preference for the text.

 @param editorReply - decides what a shared editor seat proposes, given how
 often that seat has already been asked

 @param log - call log this client writes to

 @returns Client honoring that script

 @example
 ```ts
 const client = scriptedClient({ editorReply: () => ({ edits: [], }), log, },);
 ```
 */
function scriptedClient(
  {
    editorReply,
    log,
  }: {
    readonly editorReply: (args: { readonly asked: number; },) => unknown;
    readonly log: CallLog;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (isJudge({ modelId: request.modelId, },)) {
        log.judges += 1;
        return replyWith({
          reply: {
            best: 1,
            assessment: 'fixture panel always names the first seat',
          },
          request,
        },);
      }

      log.editors += 1;

      /**
       How often this seat has been asked, counting this call.
       */
      const asked = (log.perSeat.get(request.modelId,) ?? 0) + 1;

      log.perSeat.set(
        request.modelId,
        asked,
      );

      // The wide arm's extra seat proposes nothing, so the text the wide arm
      // ships is still written by the shared seats and differs from the narrow
      // arm's only because the shared seats churn.
      if (request.modelId === WIDE_ONLY_EDITOR)
        return replyWith({
          reply: { edits: [], },
          request,
        },);

      return replyWith({
        reply: editorReply({ asked, },),
        request,
      },);
    },
  };
}

/**
 Editor script where every seat declines every time.

 @returns Empty edit list

 @example
 ```ts
 const client = scriptedClient({ editorReply: decliningReply, log, },);
 ```
 */
function decliningReply(): unknown {
  return { edits: [], };
}

/**
 Editor script where a seat proposes different wording every time it is asked.

 @param asked - how often this seat has been asked, counting this call

 @returns Edit replacing the whole envelope with that round's wording

 @throws {@link Error} when a seat is asked more times than the script has
 rounds, which would mean the runner changed how many arms it runs

 @example
 ```ts
 const reply = churningReply({ asked: 1, },);
 ```
 */
function churningReply({ asked, }: { readonly asked: number; },): unknown {
  /**
   Wording for this round.
   */
  const newText = ROUND_TEXTS[asked - 1];
  if (newText === undefined)
    throw new Error(
      `a seat was asked ${String(asked,)} times, more than the ${
        String(ROUND_TEXTS.length,)
      } rounds this script writes`,
    );

  return {
    edits: [
      {
        region: 1,
        newText,
      },
    ],
  };
}

await describe({
  name: runWidthSlice.name,
  children: [
    it({
      name: 'RUNS THREE EDITOR ARMS, repeating the narrow roster, which is what gives the '
        + 'comparison a null band of its own',
      fn: async () => {
        /**
         What the arms asked for.
         */
        const log = freshLog();

        await runWidthSlice({
          client: scriptedClient({
            editorReply: decliningReply,
            log,
          },),
          input: INPUT,
          narrowEditorIds: NARROW_EDITORS,
          wideEditorIds: WIDE_EDITORS,
          judgeModelIds: JUDGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // Two arms would cost five calls at these rosters, three cost seven, so
        // the count is what separates a probe with a band from one without.
        expect(log.editors,).toBe(THREE_ARM_EDITOR_CALLS,);
        for (const seat of SHARED_EDITORS) {
          expect(log.perSeat.get(seat,),).toBe(3,);
        }
        expect(log.perSeat.get(WIDE_ONLY_EDITOR,),).toBe(1,);
      },
    },),
    it({
      name: 'READS two arms that shipped nothing as `nothing-shipped`, and buys no judging '
        + 'for a slice where the editors declined',
      fn: async () => {
        /**
         What the arms asked for.
         */
        const log = freshLog();

        /**
         Row a slice both rosters declined contributed.
         */
        const row = await runWidthSlice({
          client: scriptedClient({
            editorReply: decliningReply,
            log,
          },),
          input: INPUT,
          narrowEditorIds: NARROW_EDITORS,
          wideEditorIds: WIDE_EDITORS,
          judgeModelIds: JUDGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(row.comparison,).toBe('nothing-shipped',);
        expect(row.narrowShipped,).toBe(false,);
        expect(row.wideShipped,).toBe(false,);
        expect(log.judges,).toBe(0,);
      },
    },),
    it({
      name: 'EARNS no head-to-head where the arms did not differ, rather than spending twelve '
        + 'ballots to learn that twice',
      fn: async () => {
        /**
         Row a slice both rosters declined contributed.
         */
        const row = await runWidthSlice({
          client: scriptedClient({
            editorReply: decliningReply,
            log: freshLog(),
          },),
          input: INPUT,
          narrowEditorIds: NARROW_EDITORS,
          wideEditorIds: WIDE_EDITORS,
          judgeModelIds: JUDGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(row.verdict,).toBe('not-run',);
        expect(row.usableBallots,).toBe(0,);
      },
    },),
    it({
      name: 'REPORTS the narrow repeat as DISAGREEING when the lane changed its own mind, '
        + 'which is the band every width reading is measured against',
      fn: async () => {
        // The positive control for the "READS two arms that shipped nothing"
        // and "EARNS no head-to-head where the arms did not differ" cases: a
        // runner that ran the narrow arm once and reported agreement would pass
        // those and fail this.
        /**
         Row a churning lane contributed.
         */
        const row = await runWidthSlice({
          client: scriptedClient({
            editorReply: churningReply,
            log: freshLog(),
          },),
          input: INPUT,
          narrowEditorIds: NARROW_EDITORS,
          wideEditorIds: WIDE_EDITORS,
          judgeModelIds: JUDGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(row.narrowRepeatAgreed,).toBe(false,);
        expect(row.comparison,).toBe('differs',);
        expect(row.narrowShipped,).toBe(true,);
        expect(row.wideShipped,).toBe(true,);
      },
    },),
    it({
      name: 'CALLS a panel that always names the first seat POSITION-DECIDED, never a win '
        + 'for whichever arm happened to sit first',
      fn: async () => {
        // Both orders are run precisely so this cannot be laundered into a
        // quality result. The fixture panel names seat one in both orders, so it
        // prefers the narrow text once and the wide text once, which is the
        // panel telling us about the slate rather than about the repair.
        /**
         What the arms and the head-to-head asked for.
         */
        const log = freshLog();

        /**
         Row a churning lane contributed.
         */
        const row = await runWidthSlice({
          client: scriptedClient({
            editorReply: churningReply,
            log,
          },),
          input: INPUT,
          narrowEditorIds: NARROW_EDITORS,
          wideEditorIds: WIDE_EDITORS,
          judgeModelIds: JUDGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(row.verdict,).toBe('position-decided',);
        expect(row.usableBallots,).toBeGreaterThan(0,);
        expect(log.judges,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'COUNTS the ACCEPTED issues as the work available, not every issue the slice carried',
      fn: async () => {
        /**
         Row a slice carrying one accepted and one rejected issue contributed.
         */
        const row = await runWidthSlice({
          client: scriptedClient({
            editorReply: decliningReply,
            log: freshLog(),
          },),
          input: INPUT,
          narrowEditorIds: NARROW_EDITORS,
          wideEditorIds: WIDE_EDITORS,
          judgeModelIds: JUDGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(ISSUES.length,).toBe(2,);
        expect(row.acceptedIssues,).toBe(1,);
      },
    },),
    it({
      name: 'STAMPS the row with the slice it describes, so a report can join it back',
      fn: async () => {
        /**
         Row a slice both rosters declined contributed.
         */
        const row = await runWidthSlice({
          client: scriptedClient({
            editorReply: decliningReply,
            log: freshLog(),
          },),
          input: INPUT,
          narrowEditorIds: NARROW_EDITORS,
          wideEditorIds: WIDE_EDITORS,
          judgeModelIds: JUDGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(row.entryId,).toBe(INPUT.entryId,);
        expect(row.sliceIndex,).toBe(INPUT.sliceIndex,);
        expect(row.heardNarrow,).toBe(NARROW_EDITORS.length,);
        expect(row.heardWide,).toBe(WIDE_EDITORS.length,);
      },
    },),
    it({
      name: 'NAMES the seats that wrote the shipped text, so who the extra seats bought is '
        + 'readable at roster level',
      fn: async () => {
        /**
         Row a churning lane contributed.
         */
        const row = await runWidthSlice({
          client: scriptedClient({
            editorReply: churningReply,
            log: freshLog(),
          },),
          input: INPUT,
          narrowEditorIds: NARROW_EDITORS,
          wideEditorIds: WIDE_EDITORS,
          judgeModelIds: JUDGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // Both shared seats proposed identical wording, so the candidate they
        // collapsed into carries both stakes and neither may be lost.
        for (const seat of SHARED_EDITORS) {
          expect(row.narrowProducers.includes(seat,),).toBe(true,);
          expect(row.wideProducers.includes(seat,),).toBe(true,);
        }

        // The wide-only seat proposed nothing, so it wrote nothing, and naming
        // it here would credit a seat for text it did not write.
        expect(row.wideProducers.includes(WIDE_ONLY_EDITOR,),).toBe(false,);
      },
    },),
  ],
  concurrency: 1,
},);

//endregion Editor width slice tests
