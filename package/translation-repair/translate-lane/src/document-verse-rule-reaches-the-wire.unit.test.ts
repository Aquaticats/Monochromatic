/**
 Guard that the verse rule reaches the requests `translateDocument` sends,
 on both halves of a governed round, and reaches neither half of an
 ungoverned one.

 WHY THIS FILE EXISTS: until 2026-08-22 the rule reached the translators and
 no judge at all. `judgeTranslateSlate` had no line-structure parameter, so
 there was nothing to forward and nothing to notice missing. The damage was
 not silence but contradiction: criterion four tells every judge that a shape
 the ORIGINAL does not have is not a fault, and on a verse slice that is the
 right rule pointed the wrong way, since there the ORIGINAL is what carries
 the line structure and the archive page is what merged it. A translator
 obeying the rule unmerges, and its judge had been handed a reason to prefer
 the merged rival. Measured when wrapping moved before judging:
 211 slices across 34 entries of the 92 pairs are governed.

 WHY IT IS NOT ENOUGH TO TEST THE HALVES. `translate-judge.unit.test.ts` and
 `translate-stage.unit.test.ts` each carry a governed and an ungoverned round,
 and both call their function with the flag written out by hand. Neither can
 see whether a real document ever DERIVES that flag and passes it down. The
 chain here is `translateDocument` to `attemptTranslateSlice` to
 `settleTranslateSlice`, which reads `prepared.lineStructuredSliceIndices`, to
 `runTranslateStage`, which hands both halves their copy. Any link dropping it
 leaves every case in those files passing.

 WHAT IS PINNED, read off the recorded request rather than off the driver's
 return value: a governed document's JUDGE sheets carry
 `TRANSLATE_LINE_STRUCTURE_CRITERION`, its TRANSLATOR sheets carry
 `TRANSLATE_LINE_STRUCTURE_RULE`, and an ungoverned document's sheets carry
 neither. The translator half is the control that makes the judge half
 legible: it is the verse-rule wiring and this fixture's governance is
 only a claim until something production decides agrees with it.

 READING THE RETURN VALUE WOULD PROVE NOTHING. A judging window was once built,
 keyed, and never passed to the call it was keyed for; nothing failed
 for weeks because every test read what the driver returned, which the missing
 wiring never touched.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  TRANSLATE_LINE_STRUCTURE_CRITERION,
  TRANSLATE_LINE_STRUCTURE_RULE,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { isLineStructured, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  translateDocument,
  type TranslateModels,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the driver under test.
 */
const l = tagged({ tag: 'document-verse-rule-reaches-the-wire-test', },);

//region Rosters

/**
 Models that render each slice.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
];

/**
 Rosters the driver seats for every run here.
 */
const MODELS: TranslateModels = {
  translatorModelIds: TRANSLATORS,
  judgeModelIds: [
    ...TRANSLATORS,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  ],
};

//endregion Rosters

//region The governed pair

/**
 Original built to trip `isLineStructured`, which needs at least five
 blank-line-separated content blocks with a median length of thirty
 characters or fewer.

 NOT A GUESS AT THE PREDICATE. One case here asks the shipped predicate about
 this exact text, so a fixture that stopped being governed would be reported
 as a broken fixture rather than passing as a clean null.
 */
const VERSE_SOURCE_TEXT = `## 第一节

猫在窗台上打盹。

尾巴垂在阳光里。

雪落在屋顶上。

它的耳朵动了一下。

风停了。

它继续睡。
`;

/**
 Archive translation for it, with its lines already merged the way the page
 this rule exists for merges them.
 */
const VERSE_TARGET_TEXT = `## Section one

The cat naps on the windowsill, its tail hanging down in the sunlight.

Snow falls on the roof and one ear twitches. The wind stops and it sleeps on.
`;

/**
 What every translator answers for the governed document, unmerged the way
 the rule asks.
 */
const VERSE_RENDERING = `## Section one

The cat naps on the windowsill.

Its tail hangs down in the sunlight.

Snow falls on the roof.

One ear twitches.

The wind stops.

It sleeps on.
`;

//endregion The governed pair

//region The ungoverned pair

/**
 Original of ordinary prose: one block, nowhere near five, so nothing about
 it can trip the predicate.
 */
const PROSE_SOURCE_TEXT = `## 第一节

猫猫窝在纸箱里打盹，尾巴搭在暖气片旁边，睡得很沉，谁叫它都不理。
`;

/**
 Archive translation for it, awkward on purpose so a fresh rendering can
 never be mistaken for the text already there.
 */
const PROSE_TARGET_TEXT = `## Section one

The cat is doing the napping inside of the cardboard box, with the tail beside the radiator, sleeping very
deeply and ignoring whoever calls it.
`;

/**
 What every translator answers for the ungoverned document.
 */
const PROSE_RENDERING = `## Section one

The cat dozes in the cardboard box, tail draped beside the radiator, too deeply asleep to answer anyone.
`;

//endregion The ungoverned pair

//region Recording the wire

/**
 One exchange a run attempted, kept so a case can read what reached the wire.

 @example
 ```ts
 const recorded: RecordedRequest = { schema: 'translation_report', content: 'ORIGINAL...', };
 ```
 */
type RecordedRequest = {
  /**
   Structured-output schema name, which tells a translator exchange from a
   judge exchange exactly as the driver's own schema names do.
   */
  readonly schema: string;

  /**
   Every message's text, joined, so a search reads the whole exchange rather
   than one message chosen in advance.
   */
  readonly content: string;
};

/**
 Builds a client that answers every translator call with one fixed rendering,
 ballots for the first candidate on every slate, and appends each exchange to
 `requests` before answering.

 @param requests - log this client appends every exchange to, in call order

 @param translatorRendering - text every translator call answers with

 @returns Client honoring that script

 @example
 ```ts
 const client = recordingClient({ requests: [], translatorRendering: VERSE_RENDERING, },);
 ```
 */
function recordingClient(
  {
    requests,
    translatorRendering,
  }: {
    readonly requests: RecordedRequest[];
    readonly translatorRendering: string;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Schema this exchange asked for, which names the stage.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name;

      requests.push({
        schema: schema ?? '',
        content: request.messages
          .map(function toContent(message,) {
            return messageText({ message, },);
          },)
          .join('\n',),
      },);

      if (schema === 'translation_report') {
        /**
         Wire reply carrying the scripted rendering.
         */
        const value: unknown = { translation: translatorRendering, };
        if (!request.validate(value,))
          throw new Error('scripted translation failed the wire guard',);
        return {
          kind: 'ok',
          value,
          rawText: JSON.stringify(value,),
        };
      }
      if (schema === 'candidate_ballot') {
        /**
         Ballot naming the first candidate on the slate. WHICH candidate wins
         is not this file's question, only what the judges were shown, so any
         answer that lets the run finish serves equally well.
         */
        const ballot: unknown = {
          best: 1,
          assessment: 'it keeps a line for a line',
        };
        if (!request.validate(ballot,))
          throw new Error('scripted ballot failed the wire guard',);
        return {
          kind: 'ok',
          value: ballot,
          rawText: JSON.stringify(ballot,),
        };
      }
      throw new Error(
        `recordingClient was asked a schema this fixture does not script: ${String(schema,)}`,
      );
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
  };
}

/**
 Drives one document and reports the sheets each half was sent.

 @param sourceText - original document

 @param targetText - translation as it stands

 @param translatorRendering - text every translator call answers with

 @returns Sheets the translators received and the sheets the judges received

 @example
 ```ts
 const { judgeSheets, } = await sheetsFrom({ sourceText, targetText, translatorRendering, },);
 ```
 */
async function sheetsFrom(
  {
    sourceText,
    targetText,
    translatorRendering,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly translatorRendering: string;
  },
): Promise<{
  readonly translatorSheets: readonly string[];
  readonly judgeSheets: readonly string[];
}> {
  /**
   Exchanges this run attempts, filled in as they happen.
   */
  const requests: RecordedRequest[] = [];

  await translateDocument({
    client: recordingClient({
      requests,
      translatorRendering,
    },),
    prepared: prepareDocumentPair({
      sourceText,
      targetText,
    },),
    models: MODELS,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return {
    translatorSheets: requests
      .filter(function isTranslator(request,): boolean {
        return request.schema === 'translation_report';
      },)
      .map(function toContent(request,): string {
        return request.content;
      },),
    judgeSheets: requests
      .filter(function isJudge(request,): boolean {
        return request.schema === 'candidate_ballot';
      },)
      .map(function toContent(request,): string {
        return request.content;
      },),
  };
}

//endregion Recording the wire

await describe({
  name: 'the verse rule reaches the wire',
  children: [
    it({
      name: 'TRIPS THE SHIPPED PREDICATE ON THE GOVERNED FIXTURE AND NOT ON THE OTHER, which is what '
        + 'every other case here rests on. A fixture that quietly stopped being line-structured would '
        + 'turn each of them into an assertion about nothing, passing exactly as it does now',
      fn: async () => {
        expect(isLineStructured({ text: VERSE_SOURCE_TEXT, },),).toBe(true,);
        expect(isLineStructured({ text: PROSE_SOURCE_TEXT, },),).toBe(false,);
      },
    },),

    it({
      name: 'DERIVES THE VERDICT FROM THE DOCUMENT AND SHOWS IT TO THE TRANSLATORS, which is the verse-rule '
        + 'wiring and the control the judge cases need. Nothing here writes the flag out by hand: '
        + 'the driver reads it off the prepared pair, so a translator sheet carrying the rule proves this '
        + 'document really is governed as production decides governance',
      fn: async () => {
        const { translatorSheets, } = await sheetsFrom({
          sourceText: VERSE_SOURCE_TEXT,
          targetText: VERSE_TARGET_TEXT,
          translatorRendering: VERSE_RENDERING,
        },);

        expect(translatorSheets.length,).toBeGreaterThan(0,);
        expect(
          translatorSheets.every(function carriesRule(sheet,): boolean {
            return sheet.includes(TRANSLATE_LINE_STRUCTURE_RULE,);
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'SHOWS THE SAME DOCUMENT\'S JUDGES THE CRITERION AGAINST MERGING LINES. This is the whole '
        + 'chain: the flag is derived by `settleTranslateSlice` from the prepared pair, forwarded by '
        + '`runTranslateStage` to BOTH halves, and turned into a criterion by the selection sheet. Every '
        + 'link can break without any case built on a hand-written flag noticing',
      fn: async () => {
        const { judgeSheets, } = await sheetsFrom({
          sourceText: VERSE_SOURCE_TEXT,
          targetText: VERSE_TARGET_TEXT,
          translatorRendering: VERSE_RENDERING,
        },);

        expect(judgeSheets.length,).toBeGreaterThan(0,);
        expect(
          judgeSheets.every(function carriesCriterion(sheet,): boolean {
            return sheet.includes(TRANSLATE_LINE_STRUCTURE_CRITERION,);
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES BOTH HALVES OF AN UNGOVERNED DOCUMENT ALONE, which is what makes the "DERIVES THE '
        + 'VERDICT FROM THE DOCUMENT AND SHOWS IT TO THE TRANSLATORS" and "SHOWS THE SAME DOCUMENT\'S JUDGES '
        + 'THE CRITERION AGAINST MERGING LINES" cases evidence rather than tautologies. A sheet carrying the rule unconditionally would satisfy them '
        + 'exactly as well, and would mean the archive\'s prose was being told to count lines its '
        + 'ORIGINAL never broke',
      fn: async () => {
        const { translatorSheets, judgeSheets, } = await sheetsFrom({
          sourceText: PROSE_SOURCE_TEXT,
          targetText: PROSE_TARGET_TEXT,
          translatorRendering: PROSE_RENDERING,
        },);

        expect(translatorSheets.length,).toBeGreaterThan(0,);
        expect(judgeSheets.length,).toBeGreaterThan(0,);
        expect(
          translatorSheets.some(function carriesRule(sheet,): boolean {
            return sheet.includes(TRANSLATE_LINE_STRUCTURE_RULE,);
          },),
        ).toBe(false,);
        expect(
          judgeSheets.some(function carriesCriterion(sheet,): boolean {
            return sheet.includes(TRANSLATE_LINE_STRUCTURE_CRITERION,);
          },),
        ).toBe(false,);
      },
    },),
  ],
},);
