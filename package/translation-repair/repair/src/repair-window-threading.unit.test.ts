/**
 Tests that the repair DRIVER computes the neighbouring window and hands
 it to the stages, which no prompt-builder test can establish.

 WHY THIS IS SEPARATE FROM THE SHEET TEST. `nearby-window-reaches-the-models`
 asserts that `buildCriticMessages` renders a window it is HANDED. Whether
 `repairPreparedDocument` computes one and passes it over is a different
 question, and it is the one that went wrong before: the translate
 lane's window sat unused for weeks because the call site never passed what the
 builder already accepted, and nothing failed.

 NO NETWORK. The client is a stub that records every sheet and answers "no
 issues", so each slice raises zero claims and skips every stage after the
 critic.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

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
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { repairPreparedDocument, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Markers no other section uses, so a sheet can be attributed to the slice it
 was asked about rather than to one that merely mentions it.
 */
const MARK = {
  first: 'ZQFIRSTCAT',
  middle: 'ZQMIDCAT',
  last: 'ZQLASTCAT',
} as const;

/**
 Invented zh original of three sections.
 */
const SOURCE_TEXT = [
  '## 第一节',
  '',
  `小猫在窗台上打盹。${MARK.first}`,
  '',
  '## 第二节',
  '',
  `橘猫在门口等鱼干。${MARK.middle}`,
  '',
  '## 第三节',
  '',
  `黑猫在屋顶看月亮。${MARK.last}`,
  '',
].join('\n',);

/**
 Invented archive English of the same three.
 */
const TARGET_TEXT = [
  '## Section one',
  '',
  `The kitten dozes on the windowsill. ${MARK.first}`,
  '',
  '## Section two',
  '',
  `The tabby waits by the gate for dried fish. ${MARK.middle}`,
  '',
  '## Section three',
  '',
  `The black cat watches the moon from the roof. ${MARK.last}`,
  '',
].join('\n',);

/**
 Real catalog ids standing in for each role, since the roster types are closed
 unions that no invented id satisfies. Which models these are does not matter:
 the stub answers for every one of them.
 */
const CRITICS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 {@inheritDoc CRITICS}
 */
const EDITORS = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 {@inheritDoc CRITICS}
 */
const JUDGES = [SEAT_HYPER_VISION,] as const;

/**
 {@inheritDoc CRITICS}

 THREE OF THEM BECAUSE FEWER CANNOT DECIDE: `assertCheckerQuorumReachable`
 floors the role at three, since a pair that disagrees resolves nothing.
 Every one of them stays clear of {@link EDITORS}.
 */
const CHECKERS = [
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 One sheet, split into what is under review and what is context.
 */
type SplitSheet = {
  /**
   Everything before the first nearby fence: the pair being judged.
   */
  readonly reviewed: string;

  /**
   The nearby fence onwards: the window.
   */
  readonly window: string;

  /**
   Whether a nearby fence was present at all.
   */
  readonly fenced: boolean;
};

/**
 Runs the real driver against a recording stub and returns every sheet it
 asked, split at the fence.

 SPLITTING IS THE ATTRIBUTION, and skipping it produces a false failure. A
 sheet for the FIRST slice mentions the middle marker, because the middle slice
 is its neighbour, so filtering sheets by "mentions this marker" credits a
 slice with its neighbours' sheets.

 @returns Every sheet the stages asked, in order

 @example
 ```ts
 const sheets = await askedSheets();
 ```
 */
async function askedSheets(): Promise<readonly SplitSheet[]> {
  /**
   Prepared pair both lanes would run over.
   */
  const prepared = await prepareDocumentPair({
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
  },);

  /**
   Every sheet the stub was handed, joined per exchange.
   */
  const asked: string[] = [];

  /**
   Recording stub. Real catalog ids, because the roster type is a closed union
   and an invented id would not compile.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText unused by the repair lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.messages
        .map(function toContent(message,): string {
          // A message carries either text or the multimodal parts a picture
          // reading uses. Only the text matters here, and serialising the parts
          // keeps any text inside them searchable rather than dropping it.
          return ((typeof message.content) === 'string')
            ? (message.content as string)
            : JSON.stringify(message.content,);
        },)
        .join('\n',),);

      /**
       An empty report, which every stage here accepts and which raises no
       claims, so the slice skips every stage after the critic.
       */
      const empty = { issues: [], };
      if (!request.validate(empty,))
        throw new Error('an empty report failed a stage guard',);
      return {
        kind: 'ok',
        value: empty,
        rawText: JSON.stringify(empty,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the repair lane',);
    },
  };

  await repairPreparedDocument({
    client,
    prepared,
    models: {
      criticModelIds: CRITICS,
      panelModelIds: CRITICS,
      editorModelIds: EDITORS,
      judgeModelIds: JUDGES,
      checkerModelIds: CHECKERS,
    },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return asked.map(function split(sheet,): SplitSheet {
    /**
     Where the window begins, absent on a slice standing alone.
     */
    const at = sheet.indexOf('NEARBY',);
    if (at === (-1)) {
      return {
        reviewed: sheet,
        window: '',
        fenced: false,
      };
    }
    return {
      reviewed: sheet.slice(0, at,),
      window: sheet.slice(at,),
      fenced: true,
    };
  },);
}

/**
 Sheets whose REVIEWED half is the slice carrying one marker.

 @param sheets - split sheets from {@link askedSheets}

 @param marker - marker identifying the slice

 @returns Sheets asked about that slice

 @example
 ```ts
 const own = about({ sheets, marker: MARK.middle, },);
 ```
 */
function about(
  {
    sheets,
    marker,
  }: {
    readonly sheets: readonly SplitSheet[];
    readonly marker: string;
  },
): readonly SplitSheet[] {
  return sheets.filter(function reviewsIt(sheet,): boolean {
    return sheet.reviewed
      .includes(marker,);
  },);
}

/**
 Sheets gathered once, since the driver run is the expensive part.
 */
const SHEETS = await askedSheets();

await describe({
  name: 'repair driver window threading',
  children: [
    it({
      name: 'asks at least one sheet per slice, every one of them fenced',
      fn: async () => {
        expect(SHEETS.length > 0,).toBe(true,);
        expect(SHEETS.every(function isFenced(sheet,): boolean {
          return sheet.fenced;
        },),).toBe(true,);
      },
    },),
    it({
      name: 'shows a middle slice BOTH of its neighbours',
      fn: async () => {
        const own = about({
          sheets: SHEETS,
          marker: MARK.middle,
        },);
        expect(own.length > 0,).toBe(true,);
        for (const sheet of own) {
          expect(sheet.window,).toContain(MARK.first,);
          expect(sheet.window,).toContain(MARK.last,);
        }
      },
    },),
    it({
      name: 'shows a FIRST slice its follower and NOT the slice beyond it',
      fn: async () => {
        // The negative half is the one that matters: it proves the window is one
        // section each way rather than the whole document, which is what bounds
        // its cost. A window that quietly widened would surface here.
        const own = about({
          sheets: SHEETS,
          marker: MARK.first,
        },);
        expect(own.length > 0,).toBe(true,);
        for (const sheet of own) {
          expect(sheet.window,).toContain(MARK.middle,);
          expect(sheet.window,).not.toContain(MARK.last,);
        }
      },
    },),
    it({
      name: 'shows a LAST slice its predecessor and NOT the slice before that',
      fn: async () => {
        const own = about({
          sheets: SHEETS,
          marker: MARK.last,
        },);
        expect(own.length > 0,).toBe(true,);
        for (const sheet of own) {
          expect(sheet.window,).toContain(MARK.middle,);
          expect(sheet.window,).not.toContain(MARK.first,);
        }
      },
    },),
  ],
},);
