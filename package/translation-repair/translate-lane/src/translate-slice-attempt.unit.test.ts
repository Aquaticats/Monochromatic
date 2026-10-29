/**
 Tests for one slice's attempt at translation, as the document driver asks it.

 The attempt settles a record or propagates operational interruption.
 An absent passage never becomes a settled empty or unfilled quality result.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { validateTranslatedSlice, } from '@monochromatic-dev/translation-repair-translate-floor';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  isInsertionChunk,
  makeInsertionChunk,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  attemptTranslateSlice,
  type TranslateModels,
} from '../dist/final/node/index.mjs';
import {
  levelCapturingLogger,
  warningRecordingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  statusFailureLogText,
  statusFailureOf,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
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
 Logger for the attempts under test.
 */
const l = tagged({ tag: 'translate-slice-attempt-test', },);

/**
 Original, two sections the archive translates.
 */
const SOURCE_TEXT = `## 第一节

猫猫在窗台上打盹。

## 第二节

窗台上有一只鸟。
`;

/**
 Archive translating both sections.
 */
const TARGET_TEXT = `## Section one

The cat is doing the sleeping on the windowsill.

## Section two

On the windowsill there is being a bird.
`;

/**
 Passage the archive never translated, carried by the insertion slice.
 */
const MISSING_SOURCE = '## 第三节\n\n猫猫也喜欢晒太阳。';

/**
 Roster the attempt seats.
 */
const MODELS: TranslateModels = {
  translatorModelIds: [
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_HYPER_VISION,
  ],
  judgeModelIds: [
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_HYPER_VISION,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  ],
};

/**
 Client nobody is scripted on: every structured reply fails the wire guard,
 so no translator is heard and no judge has anything to rank.

 @returns Client answering nothing usable

 @example
 ```ts
 const client = silentClient();
 ```
 */
function silentClient(): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
    chatJson: async () => ({
      kind: 'schema-mismatch',
      rawText: '',
      detail: 'nobody is scripted',
    }),
  };
}

/**
 Prepared pair with one insertion appended, the way the document driver
 appends a passage the archive never translated.

 BUILT BY HAND rather than prepared from a mismatched pair, because the
 deterministic aligner refuses a heading-count mismatch outright and yields no
 slices at all; the pairing that would place the missing section is LLM
 assisted and not what this suite is about.

 @returns Pair carrying two content slices and one insertion

 @example
 ```ts
 const prepared = await pairWithInsertion();
 ```
 */
async function pairWithInsertion(): Promise<PreparedDocumentPair> {
  /**
   Pair as the pipeline prepares it.
   */
  const sliced = await prepareDocumentPair({
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
  },);
  return {
    ...sliced,
    slices: [
      ...sliced.slices,
      {
        source: {
          sliceIndex: sliced.slices.length,
          nodes: [],
          startOffset: 0,
          endOffset: 0,
          text: MISSING_SOURCE,
        },
        target: makeInsertionChunk({
          sliceIndex: sliced.slices.length,
          offset: TARGET_TEXT.length,
        },),
      },
    ],
  };
}

/**
 Attempts one slice of the fixture pair with the silent client.

 @param prepared - pair the slice was cut from

 @param slice - slice to attempt

 @returns What the attempt yielded

 @example
 ```ts
 const attempt = await attemptSilently({ prepared, slice, },);
 ```
 */
async function attemptSilently(
  {
    prepared,
    slice,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly slice: ChunkPair;
  },
) {
  return attemptTranslateSlice({
    client: silentClient(),
    slice,
    prepared,
    models: MODELS,
    neighbouringIncumbentText: '',
    neighbouringSourceText: '',
    pictureContext: '',
    pictureFindings: [],
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: attemptTranslateSlice.name,
  concurrency: 1,
  children: [
    it({
      name: 'PAUSES an insertion when no translator is available rather than settling empty text',
      fn: async () => {
        /**
         Pair carrying the untranslated passage as an insertion.
         */
        const prepared = await pairWithInsertion();

        /**
         The insertion slice.
         */
        const insertion = prepared.slices.find(function isInsertion(slice,): boolean {
          return isInsertionChunk(slice.target,);
        },);
        if (insertion === undefined)
          throw new Error('the fixture pair carries no insertion slice',);

        await expect(attemptSilently({
          prepared,
          slice: insertion,
        },),).rejects.toThrow(TranslationRepairInterruptedError,);
      },
    },),

    it({
      name: 'SETTLES a content slice nobody answered on its incumbent, since only an insertion has nothing to '
        + 'fall back on',
      fn: async () => {
        /**
         Pair carrying two translated sections as content.
         */
        const prepared = await pairWithInsertion();

        /**
         A content slice.
         */
        const content = prepared.slices.find(function isContent(slice,): boolean {
          return !isInsertionChunk(slice.target,);
        },);
        if (content === undefined)
          throw new Error('the fixture pair carries no content slice',);

        const attempt = await attemptSilently({
          prepared,
          slice: content,
        },);

        expect(attempt.kind,).toBe('settled',);
        if (attempt.kind === 'settled') {
          expect(attempt.record.changed,).toBe(false,);
          expect(attempt.record.outputText,).toBe(content.target.text,);
        }
      },
    },),

    it({
      name: 'WARNS OF THE PROVIDER REFUSAL EACH SEAT ENDED IN AS THE CALLER ABORTED by the seat, the status and the '
        + 'provider\'s words, the key its refusal echoed masked, once a seat, then that the slice was abandoned by the '
        + 'caller\'s abort, and raises the abort\'s own reason',
      fn: async ctx => {
        /**
         Refusal the real client raised over the real transport.
         */
        const failure = await statusFailureOf({
          sinon: ctx.sinon,
          status: 401,
        },);
        /**
         Stop the caller gives while the provider is refusing.
         */
        const stop = new AbortController();
        /**
         Content slice of the fixture pair.
         */
        const prepared = await pairWithInsertion();
        const slice = prepared.slices.find(function isContent(candidate,): boolean {
          return !isInsertionChunk(candidate.target,);
        },);
        if (slice === undefined)
          throw new Error('the fixture pair carries no content slice',);

        const lines: string[] = [];
        const refusal = await rejectionOf(async function attemptUnderStop() {
          return attemptTranslateSlice({
            client: {
              chatText: async () => {
                throw new Error('chatText unused by the translate lane',);
              },
              quotas: async () => {
                throw new Error('quotas unused by the translate lane',);
              },
              chatJson: async () => {
                stop.abort();
                throw failure;
              },
            },
            slice,
            prepared,
            models: MODELS,
            neighbouringIncumbentText: '',
            neighbouringSourceText: '',
            pictureContext: '',
            pictureFindings: [],
            signal: stop.signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l: levelCapturingLogger({ lines, },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        expect(refusal,).toBe(stop.signal.reason,);
        // Every translator is asked at once, before the first refusal settles,
        // and the scripted client refuses each, so each seat's line carries the
        // refusal. The stage asks the seats in the order its bench rotates by the
        // prompt, which moves whenever a sheet's wording does (n166), so the seat
        // lines are compared in text order and the stage's last line is compared
        // as it stands.
        expect(lines.slice(0, -1,).toSorted(),).toEqual([
          SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          SEAT_HYPER_VISION,
          SEAT_SYNTHETIC_VISION_WITHHELD,
        ].map(function seatLine(seat,): string {
          return `warn [produceTranslateSlate] translate ${seat}: ${
            statusFailureLogText({ provider: 'synthetic', status: 401, },)
          }, as the caller stopped the round`;
        },).toSorted(),);
        expect(lines.slice(-1,),).toEqual([
          `warn slice ${String(slice.target.sliceIndex,)}: abandoned by the caller's abort (refused by AbortError)`,
        ],);
      },
    },),

    it({
      name: 'WRITES A RE-ASK\'S OWN FAILURE ONCE when the caller stops the slice during the repair of an invalid '
        + 'candidate: on the seat\'s line with the provider\'s words, and the slice\'s abandoned line without them',
      fn: async () => {
        /**
         Why the caller stopped the slice.
         */
        const reason = new Error('the owner called the cats in',);
        /**
         The caller's steering.
         */
        const stop = new AbortController();
        /**
         What the first seat's re-ask ends in, of its own.
         */
        const refusal = new ProviderHttpError({
          provider: 'synthetic',
          status: 503,
          bodyText: 'the cat is napping in the sun',
        },);
        /**
         Content slice of the fixture pair.
         */
        const prepared = await pairWithInsertion();
        const slice = prepared.slices.find(function isContent(candidate,): boolean {
          return !isInsertionChunk(candidate.target,);
        },);
        if (slice === undefined)
          throw new Error('the fixture pair carries no content slice',);

        /**
         Every warning the attempt wrote; the round's timing line goes to the
         shared logger, since its milliseconds are the clock's.
         */
        const warnings: string[] = [];
        /**
         Re-asks the client has seen, the first of which the provider refuses.
         */
        const reasked = { count: 0, };
        const refused = await rejectionOf(async function attemptUnderStop() {
          return attemptTranslateSlice({
            client: {
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
                 A translator's answer that merged the heading away, which the
                 publication rule sends back to its author.
                 */
                const merged = { translation: 'Section one: the cat dozes on the windowsill.', };
                if (request.validate(merged,)) {
                  return {
                    kind: 'ok',
                    value: merged,
                    rawText: JSON.stringify(merged,),
                  };
                }
                reasked.count += 1;
                stop.abort(reason,);
                if (reasked.count === 1)
                  throw refusal;
                // A real client forwards the aborted request's reason.
                request.signal.throwIfAborted();
                throw new Error('unreachable: a re-ask asked after the stop was handed a signal that had not aborted',);
              },
            },
            slice,
            prepared,
            models: MODELS,
            neighbouringIncumbentText: '',
            neighbouringSourceText: '',
            pictureContext: '',
            pictureFindings: [],
            signal: stop.signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l: warningRecordingLogger({
              base: l,
              warnings,
            },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        /**
         What the publication rule finds in each sent-back candidate.
         */
        const verdict = validateTranslatedSlice({
          sourceText: slice.source.text,
          candidateText: 'Section one: the cat dozes on the windowsill.',
          pageText: slice.target.text,
        },);
        if (verdict.kind !== 'invalid')
          throw new Error(`the fixture's candidate must fail the publication rule, and the floor said ${verdict.kind}`,);
        expect(refused,).toBe(reason,);
        /**
         The seats of the slate.
         */
        const seats = [
          SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          SEAT_HYPER_VISION,
          SEAT_SYNTHETIC_VISION_WITHHELD,
        ];
        /**
         The line each seat is asked under.
         */
        const askedLines = seats.map(function askedLine(seat,): string {
          return `[produceTranslateSlate] [repairOneCandidate] ${seat}: candidate fails the publication rule, so its `
            + `author is asked about it: ${verdict.findings.join(' ',)}`;
        },);
        /**
         The seat the stage handed to the repair first, which is re-asked first and refused: the seat the first asked line names.
         */
        const firstSeat = nonNullishOrThrow(seats.find(function isFirst(seat,): boolean {
          return nonNullishOrThrow(warnings[0],).includes(` ${seat}: `,);
        },),);
        // The stage seats the slate in the order its bench rotates by the prompt, which moves whenever a sheet's wording does
        // (n166), so the asked lines are compared in text order, and the refusal is the one of the seat asked first.
        expect(warnings.slice(0, seats.length,).toSorted(),).toEqual(askedLines.toSorted(),);
        expect(warnings.slice(seats.length,),).toEqual([
          `[produceTranslateSlate] [repairOneCandidate] translate-repair ${firstSeat}: `
            + 'refused by ProviderHttpError from synthetic with HTTP 503 (the provider said: "the cat is napping in the sun"), as the '
            + 'caller stopped the repair',
          `slice ${String(slice.target.sliceIndex,)}: abandoned by the caller's abort`,
        ],);
      },
    },),
  ],
},);
