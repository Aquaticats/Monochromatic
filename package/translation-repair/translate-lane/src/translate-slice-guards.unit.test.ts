/**
 Tests for what {@link settleTranslateSlice} does with the winner the judges
 chose: the quote guard, the transcript the archive carries past its
 original, a disputed slice whose repair text may not stand in, and a slice
 the floor can compare nothing on, where nobody is asked (ledger B43).

 WHY A DRIVEN SLICE RATHER THAN THE HELPERS ALONE. Each helper has its own
 file, and none of them shows what the slice settles on: the quote guard
 compares the judged part of the archive rather than the whole, the held-out
 transcript is spliced back onto the winner here and nowhere else, and the
 dispute decides here whether the repair text goes to the translators at
 all.

 Every exchange is recorded, and a sheet this script does not know is kept
 rather than answered, so a case that fell into an unplanned round fails on
 that rather than passing on whatever the round left behind.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type ArchiveDispute,
  describeArchiveDispute,
} from '@monochromatic-dev/translation-repair-archive';
import {
  assertUnheardKeptIncumbent,
  settleTranslateSlice,
  type TranslateSliceRecord,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Fixtures

/**
 Models that render the slice.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
];

/**
 Whole roster the judges are drawn from, translators included.
 */
const JUDGES: readonly RosterModelId[] = [
  ...TRANSLATORS,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Component the original ends with and the archive repeats, built from parts so
 no template placeholder sits in this file's source.
 */
const PHOTO_MARKER = `<PhotoScroll photos={['${['$', '{path}',].join('',)}/photos/letter.webp']} />`;

/**
 Transcript of the photographed letter, which the archive carries past the
 end of its original.
 */
const TRANSCRIPT = '> Dear cat, rest well.';

/**
 What one settlement sent and said.
 */
type Settlement = {
  /**
   Settled record.
   */
  readonly record: TranslateSliceRecord;

  /**
   Every sheet sent, translators' and judges' alike.
   */
  readonly sheets: readonly string[];

  /**
   Schemas no reply was scripted for, which a planned case never sends.
   */
  readonly unplanned: readonly string[];

  /**
   Every line the driver logged.
   */
  readonly said: readonly string[];
};

/**
 Builds a client whose translators all return one rendering and whose judges
 all back it.

 @param rendering - what every translator returns

 @param sheets - log every sheet is appended to

 @param unplanned - log every schema without a scripted reply is appended to

 @param judgesDecline - whether every judge names no candidate instead of backing the rendering

 @returns Client over both halves of a round

 @example
 ```ts
 const client = scriptedClient({ rendering: 'The cat dozes.', sheets: [], unplanned: [], },);
 ```
 */
function scriptedClient(
  {
    rendering,
    sheets,
    unplanned,
    judgesDecline = false,
  }: {
    readonly rendering: string;
    readonly sheets: string[];
    readonly unplanned: string[];
    readonly judgesDecline?: boolean;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Which sheet this is.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';

      /**
       Whole sheet as sent.
       */
      const content = request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);
      sheets.push(content,);

      /**
       Reply this sheet gets, absent where none is scripted.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation: rendering, }
        : (schema === 'candidate_ballot')
          ? {
            best: judgesDecline
              ? 0
              : candidateCarrying({
                content,
                needle: rendering,
              },),
            assessment: 'the scripted rendering',
          }
          : undefined;
      if ((value === undefined) || (!request.validate(value,))) {
        unplanned.push(schema,);
        throw new Error(`no scripted reply for ${schema}`,);
      }
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Settles the one slice of a pair with every translator returning one
 rendering and every judge backing it.

 @param sourceText - original page, one slice

 @param targetText - archive page, one slice

 @param rendering - what every translator returns

 @param archiveDispute - dispute over the slice, absent where there is none

 @param judgesDecline - whether every judge names no candidate, so the slate declines

 @returns Record, sheets, unplanned schemas and log lines

 @example
 ```ts
 const settled = await settle({ sourceText: '猫睡了。', targetText: 'The cat slept.', rendering: 'The cat dozed.', },);
 ```
 */
async function settle(
  {
    sourceText,
    targetText,
    rendering,
    archiveDispute,
    judgesDecline = false,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly rendering: string;
    readonly archiveDispute?: ArchiveDispute;
    readonly judgesDecline?: boolean;
  },
): Promise<Settlement> {
  /**
   Preparation the slice comes from.
   */
  const prepared = prepareDocumentPair({
    sourceText,
    targetText,
  },);
  expect(prepared.slices,).toHaveLength(1,);

  /**
   The pair's one slice.
   */
  const [slice,] = prepared.slices;
  if (slice === undefined)
    throw new Error('the pair prepares into one slice',);
  const sheets: string[] = [];
  const unplanned: string[] = [];
  const said: string[] = [];
  const record = await settleTranslateSlice({
    client: scriptedClient({
      rendering,
      sheets,
      unplanned,
      judgesDecline,
    },),
    slice,
    prepared,
    models: {
      translatorModelIds: TRANSLATORS,
      judgeModelIds: JUDGES,
    },
    ...((archiveDispute === undefined) ? {} : { archiveDispute, }),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: capturingLogger({ messages: said, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    record,
    sheets,
    unplanned,
    said,
  };
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${settleTranslateSlice.name} quote guard`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NO LONGER REACHES THIS GUARD for a rendering that drops a quote from inside a container tag '
            + '(ledger B110): the floor reads the same any-depth count and refuses that rendering at every '
            + 'translator\'s repair turn, before any judge, so the slice settles on the untouched archive through '
            + 'the ordinary `stage-result` path rather than this guard\'s `refused-quote-loss` one',
          fn: async () => {
            /**
             Archive wording, one quote inside a folded block.
             */
            const archive = '<details>\n\n> Feed me at noon.\n\n</details>';

            /**
             What every translator still offers, the same block with the
             quote made prose: the floor refuses it at the repair turn now,
             where it passed through to the judges before.
             */
            const rendering = '<details>\n\nShe asked to be fed at noon.\n\n</details>';
            const settled = await settle({
              sourceText: '<details>\n\n> 中午喂我。\n\n</details>',
              targetText: archive,
              rendering,
            },);
            // EVERY TRANSLATOR'S REPAIR TURN IS ASKED, since each one's
            // candidate now fails the floor, and the scripted client has no
            // reply for that schema: the rendering stands unrevised, fails
            // the floor again, and all three are withheld, leaving the
            // archive as the sole remaining candidate, shipped unjudged.
            expect(settled.unplanned,).toEqual([
              'translation_repair_report',
              'translation_repair_report',
              'translation_repair_report',
            ],);
            expect(settled.record.stageResult.text,).toBe(archive,);
            expect(settled.record.disposition,).toBe('stage-result',);
            expect(settled.record.outputText,).toBe(archive,);
            expect(settled.record.changed,).toBe(false,);
          },
        },),

        it({
          name: 'SHIPS A WINNER WHOSE QUOTE OPENS ON THE LINE AFTER A PARAGRAPH\'S, which the parser and the floor read '
            + 'as the archive\'s quote and the blank-line count read as none (ledger B42, hulicaijia24 slice 2)',
          fn: async () => {
            /**
             Winner, the quote kept with no blank line opening it.
             */
            const rendering = 'The cat left a note:\n> Feed me at noon.';
            const settled = await settle({
              sourceText: '猫留了一张纸条：\n\n> 中午喂我。',
              targetText: 'The cat was leaving a note:\n\n> Feed me at noon.',
              rendering,
            },);
            expect(settled.unplanned,).toEqual([],);
            expect(settled.record.disposition,).toBe('stage-result',);
            expect(settled.record.outputText,).toBe(rendering,);
            expect(settled.record.changed,).toBe(true,);
          },
        },),

        it({
          name: 'READS NO QUOTE IN FRONT MATTER (ledger B113): a YAML block scalar line that opens with `>` is no '
            + 'quoted passage, and the front-matter floor checks YAML, so a rendering without that line ships '
            + 'rather than standing refused for a quote the page never had',
          fn: async () => {
            /**
             Archive front matter whose motto is a block scalar holding a
             line that opens with `>`.
             */
            const archive = '---\nname: Xiaohua\nmotto: |\n  > Meow\n---\n';

            /**
             What every translator offers: the motto as a plain scalar.
             */
            const rendering = '---\nname: Xiaohua\nmotto: Meow\n---\n';
            const settled = await settle({
              sourceText: '---\nname: 小花\nmotto: |\n  > 喵\n---\n',
              targetText: archive,
              rendering,
            },);
            expect(settled.unplanned,).toEqual([],);
            expect(settled.record.disposition,).toBe('stage-result',);
            expect(settled.record.changed,).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: `${settleTranslateSlice.name} target-only transcript`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS THE TRANSCRIPT THE ARCHIVE CARRIES PAST ITS ORIGINAL OUT OF EVERY SHEET, says so, and splices it '
            + 'back onto the winner, so a replacement cannot delete the accessible reading of a picture',
          fn: async () => {
            /**
             Judged part of the archive, the component the original ends with
             included.
             */
            const judged = `The cat slept.\n\n${PHOTO_MARKER}`;

            /**
             Archive page, the transcript past the component.
             */
            const archive = `${judged}\n\n${TRANSCRIPT}`;

            /**
             Winner, rendering the original alone.
             */
            const rendering = `The cat dozed.\n\n${PHOTO_MARKER}`;
            const settled = await settle({
              sourceText: `猫睡了。\n\n${PHOTO_MARKER}`,
              targetText: archive,
              rendering,
            },);
            expect(settled.unplanned,).toEqual([],);
            expect(settled.sheets.length,).toBeGreaterThan(0,);
            expect(settled.sheets.some(function showsTranscript(sheet,): boolean {
              return sheet.includes('Dear cat, rest well.',);
            },),).toBe(false,);
            /**
             Characters held out: the transcript and the line breaks before it.
             */
            const held = `\n\n${TRANSCRIPT}`.length;
            expect(settled.said,).toContain(
              `translate slice 0: holding ${String(held,)} characters of target-only English, with the line breaks `
                + `before it, out of translation, judging ${String(judged.length,)} of ${String(archive.length,)}`,
            );
            expect(settled.record.disposition,).toBe('stage-result',);
            expect(settled.record.outputText,).toBe(`${rendering}\n\n${TRANSCRIPT}`,);
          },
        },),
      ],
    },),

    describe({
      name: `${settleTranslateSlice.name} archive dispute`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SENDS NO SHEET THE REPAIR TEXT where the dispute says it may not stand in, judges the archive under the '
            + 'dispute instead, and ships the winner over it (owner, 2026-09-27, "No eligible standing")',
          fn: async () => {
            /**
             Repair lane's text, which the checkers did not confirm.
             */
            const standIn = 'The cat slept on the sill all afternoon.';

            /**
             Dispute over the slice.
             */
            const archiveDispute: ArchiveDispute = {
              sliceIndex: 0,
              standIn,
              standInEligible: false,
              standInRefusal: 'unresolved',
              acceptedClaims: ['accuracy/addition major: The cat did not swallow pills.',],
            };

            /**
             Winner.
             */
            const rendering = 'The cat napped on the windowsill.';
            const settled = await settle({
              sourceText: '猫猫在窗台上打盹。',
              targetText: 'The cat is doing the sleeping on the windowsill and swallowed pills.',
              rendering,
              archiveDispute,
            },);
            expect(settled.unplanned,).toEqual([],);
            expect(settled.sheets.length,).toBeGreaterThan(0,);
            expect(settled.sheets.some(function showsStandIn(sheet,): boolean {
              return sheet.includes(standIn,);
            },),).toBe(false,);
            expect(settled.record.findings,).toContain(describeArchiveDispute({ dispute: archiveDispute, },),);
            expect(settled.record.outputText,).toBe(rendering,);
          },
        },),

        it({
          name: 'SETTLES THE SLICE AS UNCHANGED, with the archive wording as the output and the archive-kept finding, '
            + 'where the dispute withholds the stand-in and every judge declines the slate: no candidate drew two '
            + 'ballots, so nothing ships past the decline and the archive stays (owner, 2026-10-25, "Two judges must '
            + 'name it"; re-check 2, finding F12)',
          fn: async () => {
            /**
             Archive wording the dispute leaves standing.
             */
            const targetText = 'The cat is doing the sleeping on the windowsill and swallowed pills.';
            const settled = await settle({
              sourceText: '猫猫在窗台上打盹。',
              targetText,
              rendering: 'The cat napped on the windowsill.',
              judgesDecline: true,
              archiveDispute: {
                sliceIndex: 0,
                standIn: 'The cat slept on the sill all afternoon.',
                standInEligible: false,
                standInRefusal: 'unresolved',
                acceptedClaims: ['accuracy/addition major: The cat did not swallow pills.',],
              },
            },);
            expect({
              unplanned: settled.unplanned,
              outputText: settled.record.outputText,
              changed: settled.record.changed,
              disposition: settled.record.disposition,
              kept: settled.record.findings.includes('translate-slate-declined-archive-kept',),
            },).toEqual({
              unplanned: [],
              outputText: targetText,
              changed: false,
              disposition: 'stage-result',
              kept: true,
            },);
          },
        },),

        it({
          name: 'SENDS THE REPAIR TEXT AS THE INCUMBENT where the dispute says it may stand in, the positive control for '
            + 'the case beside it',
          fn: async () => {
            /**
             Repair lane's text, which the checkers confirmed.
             */
            const standIn = 'The cat slept on the sill all afternoon.';
            const settled = await settle({
              sourceText: '猫猫在窗台上打盹。',
              targetText: 'The cat is doing the sleeping on the windowsill and swallowed pills.',
              rendering: 'The cat napped on the windowsill.',
              archiveDispute: {
                sliceIndex: 0,
                standIn,
                standInEligible: true,
                acceptedClaims: ['accuracy/addition major: The cat did not swallow pills.',],
              },
            },);
            expect(settled.unplanned,).toEqual([],);
            expect(settled.sheets.some(function showsStandIn(sheet,): boolean {
              return sheet.includes(standIn,);
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: `${settleTranslateSlice.name} where the floor compares nothing`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SETTLES ON THE ARCHIVE\'S OWN BYTES with nobody asked, on a plain slice and on a disputed one whose '
            + 'repair text may stand in, since no floor could check that text either; the record is one a slice that '
            + 'heard nobody may keep (ledger B43)',
          fn: async () => {
            /**
             Original with an expression the strict grammar never closes.
             */
            const sourceText = '猫猫在{窗台上打盹。';

            /**
             Archive wording of the slice.
             */
            const targetText = 'The cat is doing the sleeping on the windowsill.';

            /**
             Each settlement: no dispute, then a dispute whose repair text may
             stand in.
             */
            const settled = await Promise.all([
              settle({
                sourceText,
                targetText,
                rendering: 'The cat dozed on the windowsill.',
              },),
              settle({
                sourceText,
                targetText,
                rendering: 'The cat dozed on the windowsill.',
                archiveDispute: {
                  sliceIndex: 0,
                  standIn: 'The cat slept on the sill all afternoon.',
                  standInEligible: true,
                  acceptedClaims: ['accuracy/addition major: The cat did not swallow pills.',],
                },
              },),
            ],);

            expect(settled.map(function whatSettled({
              record,
              sheets,
              unplanned,
            },) {
              // THE CHECK THE DRIVER MAKES before it keeps a record whose stage
              // heard nobody, which throws on anything but the archive unchanged.
              assertUnheardKeptIncumbent({
                sliceIndex: 0,
                record,
                incumbentText: targetText,
              },);
              return {
                sheets: sheets.length,
                unplanned,
                decision: record.stageResult
                  .decision,
                outputText: record.outputText,
                changed: record.changed,
                disposition: record.disposition,
              };
            },),).toEqual([0, 1,].map(function expected() {
              return {
                sheets: 0,
                unplanned: [],
                decision: 'unfloored',
                outputText: targetText,
                changed: false,
                disposition: 'stage-result',
              };
            },),);
          },
        },),
      ],
    },),
  ],
},);
