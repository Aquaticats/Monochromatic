/**
 Tests that the produced-volume bound REACHES THE WIRE on every producing
 call the translate lane makes, the re-ask included.

 WHY THIS IS SEPARATE FROM `produced-volume-bound.unit.test.ts`. That one hands
 `maxAnswerChars` straight to `drainBody` and watches the drain refuse, which
 proves the LAST hop and nothing above it. Between the producer and the drain
 the bound crosses five boundaries as an OPTIONAL field forwarded by
 conditional spread: quorum, round, call, client, transport. Delete any one of
 those spreads and the code still compiles, the whole suite still passes, and
 production silently returns to the state the bound existed to end, where the
 seam exists and nothing passes anything through it. Only a test that reads
 what the CLIENT was handed can see that.

 THE SECOND THING IT PINS IS THE PRODUCING RE-ASK. An invalid candidate is
 sent back to its author for a fresh rendering of the same slice, which is a
 producing call by every measure that matters, and it was left unbounded when
 the bound landed. It is also the busiest one: measured over 32 artifacts, the
 re-ask fired on 96 of 175 slices.

 NO NETWORK. The client is a stub scripting one invalid rendering and one
 revision, recording every request it is handed.

 Fixtures are cat-themed invention mirroring corpus structure only.

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
import {
  producedVolumeBound,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { produceTranslateSlate, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Produced volume threading

/**
 Logger for the producer under test.
 */
const l = tagged({ tag: 'produced-volume-threading-test', },);

/**
 Original the translator renders: a heading and a paragraph.
 */
const SOURCE_TEXT = `## 猫猫的一天

它在窗台上打盹。`;

/**
 Rendering matching that structure, which the re-ask returns.
 */
const GOOD_TEXT = `## A Day in the Cat's Life

It dozes on the windowsill.`;

/**
 Rendering that merged the heading away, which validation reports and which
 therefore earns the author a re-ask.
 */
const MERGED_TEXT = 'A day in the cat\'s life: it dozes on the windowsill.';

/**
 Model whose candidate is produced and then re-asked.
 */
const TRANSLATOR = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Characters the runaway that prompted the bound emitted for a 56-character slice.

 Named here so the widened re-ask bound can be shown to still catch it: a
 bound generous enough to never refuse a legitimate repair is worth nothing if
 it is also generous enough to let that emission through.
 */
const EXEMPLAR_RUNAWAY_CHARS = 10_381;

/**
 One request as the client received it, which is the far side of every hop
 this test exists to pin.
 */
type SeenRequest = {
  readonly maxAnswerChars?: number;
};

/**
 Client that answers from a script and records what it was asked.

 ORDER IS THE DISCRIMINATOR, and one translator is configured so that order is
 unambiguous: the first call produces, the second re-asks. Reading the stage
 off the request is not possible, because a request carries no stage.

 @param seen - collector each request is appended to

 @param answers - scripted values, one per call in order

 @returns Client suitable for one producing pass

 @example
 ```ts
 const client = recordingClient({ seen, answers: [rendering, revision,], },);
 ```
 */
function recordingClient(
  {
    seen,
    answers,
  }: {
    readonly seen: SeenRequest[];
    readonly answers: readonly unknown[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Which call this is, read before recording it.
       */
      const turn = seen.length;

      seen.push(
        (request.maxAnswerChars === undefined)
          ? {}
          : { maxAnswerChars: request.maxAnswerChars, },
      );

      /**
       Scripted value for this turn.
       */
      const answer = answers[turn];
      if (!request.validate(answer,))
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: `scripted answer ${String(turn,)} failed the guard it was written for`,
        };

      return {
        kind: 'ok',
        value: answer,
        rawText: '',
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Runs one producing pass whose only candidate is invalid, so both the
 producing call and the re-ask happen.

 @returns Every request the client saw, in order

 @example
 ```ts
 const seen = await produceOnce();
 ```
 */
async function produceOnce(): Promise<readonly SeenRequest[]> {
  /**
   Requests the client received.
   */
  const seen: SeenRequest[] = [];

  await produceTranslateSlate({
    client: recordingClient({
      seen,
      answers: [
        { translation: MERGED_TEXT, },
        {
          resolution: 'revised',
          translation: GOOD_TEXT,
          translatorNote: 'The heading was restored as its own block.',
        },
      ],
    },),
    translatorModelIds: [TRANSLATOR,],
    sourceText: SOURCE_TEXT,
    incumbentText: '',
    lineStructured: false,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return seen;
}

/**
 What validation says about the merged rendering, derived here from the same
 public call the producer makes rather than copied out of it, so the expected
 re-ask bound is an independent computation.
 */
const validation = validateTranslatedSlice({
  sourceText: SOURCE_TEXT,
  candidateText: MERGED_TEXT,
  pageText: '',
  lineStructured: false,
},);

/**
 Characters of finding text the re-ask must answer for.
 */
const findingsChars = (validation.kind === 'invalid')
  ? validation.findings
    .join('',)
    .length
  : 0;

await describe({
  name: `${produceTranslateSlate.name} volume threading`,
  children: [
    it({
      name: 'HANDS THE PRODUCING CALL A BOUND COMPUTED FROM ITS OWN SLICE, which is the '
        + 'assertion that pins every hop between the producer and the client: quorum, round '
        + 'and call each forward it as an optional field, so dropping any one of them is '
        + 'invisible to the compiler and to every test that stops at the drain',
      fn: async function producingCallIsBounded() {
        const seen = await produceOnce();

        expect(seen[0]?.maxAnswerChars,).toBe(
          producedVolumeBound({ materialChars: SOURCE_TEXT.length, },),
        );
      },
    },),

    it({
      name: 'BOUNDS THE RE-ASK TOO, widened by the findings it must answer, because the repair '
        + 'wire carries an explanation no source slice bounds and reusing the producing bound '
        + 'unchanged would refuse a correct repair on a short slice',
      fn: async function reAskIsBounded() {
        const seen = await produceOnce();

        expect(seen[1]?.maxAnswerChars,).toBe(
          producedVolumeBound({ materialChars: SOURCE_TEXT.length + findingsChars, },),
        );
      },
    },),

    it({
      name: 'LEAVES NO PRODUCING CALL UNBOUNDED, asserted over every request rather than over '
        + 'named ones, so a producing path added later is caught by this test instead of '
        + 'reaching production policed only by the absolute cap',
      fn: async function noCallIsUnbounded() {
        const seen = await produceOnce();

        expect(seen,).toHaveLength(2,);
        expect(
          seen.filter(function unbounded(request,): boolean {
            return request.maxAnswerChars === undefined;
          },),
        ).toHaveLength(0,);
      },
    },),

    it({
      name: 'KEEPS THE WIDENED RE-ASK BOUND UNDER THE EMISSION THAT PROMPTED THE BOUND, so the room '
        + 'made for an explanation did not also make room for the runaway: generosity that '
        + 'disarms the guard would be worse than no guard, because it would read as one',
      fn: async function wideningKeepsDetection() {
        const seen = await produceOnce();

        expect(seen[1]?.maxAnswerChars,).toBeLessThan(EXEMPLAR_RUNAWAY_CHARS,);
      },
    },),
  ],
},);

//endregion Produced volume threading
