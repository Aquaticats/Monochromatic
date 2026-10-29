/**
 Tests for the translate lane's document driver.

 What this covers that the stage tests cannot: that EVERY slice is visited,
 that the alignment guard protects archive text the source cannot account for,
 that a cached slice costs no calls, and that the document reassembles from
 per-slice decisions rather than from one whole-document rewrite.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { InsertionAdmission, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ArchiveDispute, } from '@monochromatic-dev/translation-repair-archive';
import { validateTranslatedSlice, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  TranslationRepairInterruptedError,
  unflooredFinding,
} from '@monochromatic-dev/translation-repair-translate-stage';
import {
  translateDocument,
  type TranslateModels,
  type TranslateSliceRecord,
  type TranslateSliceSeating,
} from '@monochromatic-dev/translation-repair-translate-lane';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  makeInsertionChunk,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  firstRoundWindow,
  RosterConfigurationError,
} from '@monochromatic-dev/translation-repair-stage';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  stopLandingHook,
  UNASKED_CLIENT,
} from './stop-landing-hook.test-fixture.ts';

/**
 Logger for the driver under test.
 */
const l = tagged({ tag: 'translate-document-test', },);

/**
 Original document: two sections, each one paragraph.
 */
const SOURCE_TEXT = `## 第一节

猫猫在窗台上打盹。

## 第二节

窗台上有一只鸟。
`;

/**
 Translation as it stands, awkward but complete.
 */
const TARGET_TEXT = `## Section one

The cat is doing the sleeping on the windowsill.

## Section two

On the windowsill there is being a bird.
`;

/**
 Sentence every translator returns for the first section, so the slate
 collapses to one fresh candidate and the judges have a clear winner.
 */
const FRESH = 'The cat naps on the windowsill.';

/**
 Original the anchored case leaves untranslated, and the section it belongs
 to.
 */
const MISSING_SOURCE = '## 第三节\n\n猫猫也喜欢晒太阳。';

/**
 Translation scripted for source-only third section once admitted.
 */
const MISSING_FRESH = '## Section three\n\nThe cat also likes basking in sunlight.';

/**
 Original of a page whose translation is measurably too short to hold it.

 ITS OWN PAIR rather than {@link SOURCE_TEXT}. The corroboration gate admits
 an insertion only where the page LACKS about as much English as the passage
 would render into, and {@link TARGET_TEXT} is deliberately verbose: it runs
 3.4 English code points per source point against a corpus median of 2.65, so
 that page is not missing anything and no insertion may be written into it.
 Measured: this pair runs 1.09, leaving a shortfall of 156 against the 45
 points {@link MISSING_SOURCE} would render into.
 */
const SHORT_SOURCE = `## 第一节

猫猫在窗台上打盹，尾巴一晃一晃的，阳光把它的毛照得发亮，屋子里安安静静，只有钟摆在响。

## 第二节

窗台上有一只鸟，猫猫盯着它看了很久，耳朵一直朝着那个方向，但它始终没有跳起来。
`;

/**
 Translation of {@link SHORT_SOURCE}, terse enough that the page reads as
 missing English rather than as merely brief.
 */
const SHORT_TARGET = `## Section one

The cat naps on the windowsill.

## Section two

A bird sits outside, and the cat watches it.
`;

/**
 Stand-in for "no slice is silenced", which every case but the anchored one
 uses.

 A NEEDLE NO PROMPT CARRIES rather than an empty string, since every prompt
 contains an empty string and every translator would fall silent.
 */
const SILENT_FOR_NOTHING = 'a passage no fixture contains';

/**
 Renders one slice the way a translator that respected block structure would.

 A slice carries its heading, so a rendering that dropped it would fail
 structural validation rather than test the driver.

 @param content - translator prompt, which carries the slice original

 @returns Rendering for that slice

 @example
 ```ts
 const rendering = renderingFor({ content, },);
 ```
 */
function renderingFor({ content, }: { readonly content: string; },): string {
  // A notes section is rendered faithfully, marker and all, which is what makes
  // the footnote case about the REFERENCE that went missing rather than about a
  // pair that vanished together.
  if (content.includes('〔1〕：',))
    return '## Notes\n\n[^1]: The spot it likes best.';
  if (content.includes('## 第一节',))
    return `## Section one\n\n${FRESH}`;
  if (content.includes('## 第二节',))
    return '## Section two\n\nA bird sits on the windowsill.';
  if (content.includes('## 第三节',))
    return MISSING_FRESH;
  return FRESH;
}

/**
 Exactly the failure an entry deadline trips with.

 HELD AT MODULE SCOPE so the fixture that aborts and the case that asserts
 name one object. The driver's contract is that it surfaces the abort reason
 ITSELF; a wording assertion is satisfied by any lookalike, including a
 wrapper built from the reason, which is the failure worth ruling out.
 */
const ENTRY_DEADLINE_FAILURE = new Error('entry deadline reached',);

/**
 Models that render each slice.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
];

/**
 Rosters the driver seats.
 */
const MODELS: TranslateModels = {
  translatorModelIds: TRANSLATORS,
  judgeModelIds: [
    ...TRANSLATORS,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  ],
};

/**
 Calls the driver made, so a case can prove a resumed slice bought nothing.
 */
type CallLog = {
  translate: number;

  /**
   Translate calls ATTEMPTED, whether or not one came back. The only counter
   that moves when every translator is down, which is what lets a case ask
   whether a slice was asked at all rather than whether it was answered.
   */
  translateAttempts: number;
  select: number;

  /**
   Judge calls ATTEMPTED, whether or not one came back. A stage that keeps
   fanning out after the run was stopped shows up here and nowhere else.
   */
  selectAttempts: number;
};

/**
 Successful translator calls in flight, and peak observed by fixture.
 */
type TranslateConcurrency = {
  now: number;
  peak: number;
};

/**
 Client serving both stages of the lane from one script.

 @param calls - shared call log the cases assert on

 @param controller - abort the script may fire, standing in for the entry
 deadline the corpus pass imposes

 @param abortAfterTranslateCalls - translate calls served before the script
 aborts; absent means it never does

 @param silentTranslators - whether every translate call fails, standing in for
 a provider that is down while the signal stays live

 @param silentForSource - original whose slice every translator fails on,
 absent when none does; this is how one slice is made unfillable while the
 rest of the document translates normally

 @param translateConcurrency - optional successful-call overlap instrument

 @returns Client honoring the script

 @example
 ```ts
 const client = laneClient({ calls, controller, },);
 ```
 */
function laneClient(
  {
    calls,
    controller,
    abortAfterTranslateCalls,
    silentTranslators = false,
    silentForSource = SILENT_FOR_NOTHING,
    translateConcurrency,
    sheets,
  }: {
    readonly calls: CallLog;
    readonly controller: AbortController;
    readonly sheets?: string[];
    readonly abortAfterTranslateCalls?: number;
    readonly silentTranslators?: boolean;
    readonly silentForSource?: string;
    readonly translateConcurrency?: TranslateConcurrency;
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
       Schema the caller asked for, which names the stage.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name;
      /**
       Everything the caller sent, which carries the slice original.
       */
      const content = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);
      if (schema === 'translation_report') {
        calls.translateAttempts += 1;
        if ((abortAfterTranslateCalls !== undefined)
          && (calls.translate >= abortAfterTranslateCalls))
          controller.abort(ENTRY_DEADLINE_FAILURE,);
        // What the real transport does under an aborted signal: the stream is
        // torn down and the failure propagates untouched. The gather machinery
        // turns that into a LOST VOICE rather than a throw, which is exactly
        // the condition these cases exist to pin.
        if (request.signal
          .aborted)
          throw new Error('exchange torn down by abort',);
        if (silentTranslators)
          throw new Error('translator provider is down',);
        // ONE SLICE rather than the roster: every translator fails on this
        // original and answers normally on every other, which is what makes a
        // single passage unfillable while the document around it settles.
        if (content.includes(silentForSource,))
          throw new Error('translator lost its voice on this passage',);
        if (translateConcurrency !== undefined) {
          translateConcurrency.now += 1;
          translateConcurrency.peak = Math.max(
            translateConcurrency.peak,
            translateConcurrency.now,
          );
          await wait(10,);
          translateConcurrency.now -= 1;
        }
        calls.translate += 1;
        sheets?.push(content,);

        /**
         Wire reply carrying scripted rendering.
         */
        const value: unknown = { translation: renderingFor({ content, },), };
        if (!request.validate(value,))
          throw new Error('scripted translation failed the wire guard',);
        return {
          kind: 'ok',
          value,
          rawText: JSON.stringify(value,),
        };
      }
      calls.selectAttempts += 1;
      if (request.signal
        .aborted)
        throw new Error('exchange torn down by abort',);
      calls.select += 1;

      /**
       Fresh rendering belonging to ordinary or admitted insertion slice.
       */
      const wantedRendering = content.includes(MISSING_FRESH,)
        ? MISSING_FRESH
        : FRESH;
      /**
       Ballot naming the fresh rendering.
       */
      const ballot: unknown = {
        best: candidateCarrying({
          content,
          needle: wantedRendering,
        },),
        assessment: 'scripted',
      };
      if (!request.validate(ballot,))
        throw new Error('scripted ballot failed the wire guard',);
      return {
        kind: 'ok',
        value: ballot,
        rawText: JSON.stringify(ballot,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
  };
}

/**
 Runs the driver over a document pair.

 @param sourceText - original document

 @param targetText - translation as it stands

 @param resumed - records a previous run settled, keyed as the driver keys
 them

 @param abortAfterTranslateCalls - translate calls served before the script
 aborts the run

 @param silentTranslators - whether every translate call fails while the signal
 stays live

 @param overlap - most slices the driver may run at once

 @param insertionAdmission - caller evidence overriding deterministic insertion gate

 @param translateConcurrency - optional successful-call overlap instrument

 @param persisted - map the run writes settled records into; passed in so a
 case that expects a REJECTION can still read what reached the cache

 @param calls - log the run counts into, passed in for the same reason

 @returns Result, the call log, and everything persisted

 @example
 ```ts
 const { result, calls, } = await runDriver({},);
 ```
 */
async function runDriver(
  {
    sourceText = SOURCE_TEXT,
    targetText = TARGET_TEXT,
    resumed = new Map<string, TranslateSliceRecord>(),
    abortAfterTranslateCalls,
    silentTranslators = false,
    silentForSource = SILENT_FOR_NOTHING,
    anchorSource,
    overlap = 1,
    insertionAdmission,
    translateConcurrency,
    persisted = new Map<string, TranslateSliceRecord>(),
    calls = {
      translate: 0,
      translateAttempts: 0,
      select: 0,
      selectAttempts: 0,
    },
    beforeSlice,
    archiveDisputes,
    sheets,
    messages,
  }: {
    readonly sourceText?: string;
    readonly targetText?: string;
    readonly archiveDisputes?: ReadonlyMap<number, ArchiveDispute>;
    readonly sheets?: string[];
    readonly resumed?: ReadonlyMap<string, TranslateSliceRecord>;
    readonly abortAfterTranslateCalls?: number;
    readonly silentTranslators?: boolean;
    readonly silentForSource?: string;
    readonly anchorSource?: string;
    readonly overlap?: number;
    readonly insertionAdmission?: InsertionAdmission;
    readonly translateConcurrency?: TranslateConcurrency;
    readonly persisted?: Map<string, TranslateSliceRecord>;
    readonly calls?: CallLog;
    readonly beforeSlice?: () => Promise<TranslateSliceSeating>;
    readonly messages?: string[];
  },
) {
  /**
   Run steering, which the script may abort part way through the document.
   */
  const controller = new AbortController();

  /**
   Preparation as the slicer produces it today, with only content slices.
   */
  const sliced = prepareDocumentPair({
    sourceText,
    targetText,
  },);

  /**
   That preparation, with one source section the archive never translated
   appended as an anchor at the end of the document.

   BUILT BY HAND so the case does not depend on a producer: the section-scale
   and block-scale insertion producers are tested on their own, and this
   driver has to refuse a wrong answer whatever produced it.
   */
  const prepared = (anchorSource === undefined) ? sliced : {
    ...sliced,
    slices: [
      ...sliced.slices,
      {
        source: {
          sliceIndex: sliced.slices
            .length,
          nodes: [],
          startOffset: 0,
          endOffset: 0,
          text: anchorSource,
        },
        target: makeInsertionChunk({
          sliceIndex: sliced.slices
            .length,
          offset: targetText.length,
        },),
      },
    ],
  };

  /**
   What the lane decided for the whole document.
   */
  const result = await translateDocument({
    client: laneClient({
      calls,
      controller,
      ...((abortAfterTranslateCalls === undefined)
        ? {}
        : { abortAfterTranslateCalls, }),
      silentTranslators,
      silentForSource,
      ...((translateConcurrency === undefined)
        ? {}
        : { translateConcurrency, }),
      ...((sheets === undefined) ? {} : { sheets, }),
    },),
    prepared,
    models: MODELS,
    ...((archiveDisputes === undefined) ? {} : { archiveDisputes, }),
    signal: controller.signal,
    perCallTimeoutMs: HANG_STOP_MS,
    overlap,
    ...((insertionAdmission === undefined) ? {} : { insertionAdmission, }),
    ...((beforeSlice === undefined) ? {} : { beforeSlice, }),
    sliceCache: {
      resumed,
      persist: async ({
        key,
        serialized,
      },) => {
        persisted.set(
          key,
          JSON.parse(serialized,) as TranslateSliceRecord,
        );
      },
    },
    l: (messages === undefined) ? l : capturingLogger({ messages, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    result,
    calls,
    persisted,
    prepared,
  };
}

await describe({
  name: translateDocument.name,
  children: [
    it({
      name: 'JUDGES THE REPAIR LANE\'S TEXT AS THE INCUMBENT on a disputed slice, so the archive rendering is '
        + 'neither a candidate nor the fallback and the lane ships what it chose over that stand-in '
        + '(class one hundred seven, CuspariaKLSY10 slice 3, 2026-09-24)',
      fn: async () => {
        /**
         Repair lane's text for the first section, the invented detail removed.
         */
        const standIn = '## Section one\n\nThe cat slept on the windowsill.';
        /**
         Archive wording of the first section, which the dispute withholds.
         */
        const archivePhrase = 'is doing the sleeping on the windowsill';
        const sheets: string[] = [];
        const { result, } = await runDriver({
          archiveDisputes: new Map([[0, { sliceIndex: 0, standIn, standInEligible: true, acceptedClaims: ['accuracy/addition major: The cat did not swallow pills.',], },],],),
          sheets,
        },);
        /**
         Translator sheets for the first section.
         */
        const firstSection = sheets.filter(function ofFirst(sheet,): boolean {
          return sheet.includes('## 第一节',);
        },);
        expect(firstSection.length,).toBeGreaterThan(0,);
        for (const sheet of firstSection) {
          expect(sheet,).toContain('The cat slept on the windowsill.',);
          expect(sheet,).not.toContain(archivePhrase,);
        }
        /**
         Record for the disputed slice.
         */
        const [record,] = result.slices;
        expect(record?.sliceIndex,).toBe(0,);
        expect(record?.findings.some(function isDispute(finding,): boolean {
          return finding.startsWith('translate-archive-disputed (slice 0)',);
        },),).toBe(true,);
        // The judges chose the fresh rendering over the stand-in, and the
        // record says so against the archive's own bytes.
        expect(record?.outputText,).toContain(FRESH,);
        expect(record?.changed,).toBe(true,);
        expect(result.translatedText,).not.toContain(archivePhrase,);
      },
    },),

    it({
      name: 'visits EVERY slice, which is the whole reason this lane exists: '
        + 'the repair driver returns early on exactly the slices translation is '
        + 'meant to recover, so a fluent but mediocre translation nobody '
        + 'complains about would never be looked at',
      fn: async () => {
        const { result, } = await runDriver({},);
        expect(result.slices,).toHaveLength(result.sliceCount,);
        expect(result.sliceCount,).toBeGreaterThan(1,);
        for (const record of result.slices)
          expect(record.kind,).toBe('translate-slice',);
      },
    },),

    it({
      name: 'ASKS BEFORE EVERY SLICE when given a hook, so a caller can hold a slice back while a '
        + 'named provider hold keeps the bench from quorum (the thirteenth class\'s second face)',
      fn: async () => {
        const before = { calls: 0, };
        const { result, } = await runDriver({
          beforeSlice: async (): Promise<TranslateSliceSeating> => {
            before.calls += 1;
            return {};
          },
        },);
        expect(before.calls,).toBe(result.sliceCount,);
        expect(result.sliceCount,).toBeGreaterThan(1,);
      },
    },),

    it({
      name: 'runs two slices at once when overlap is 2, after a serial positive control proves '
        + 'the successful-call instrument can distinguish one slice from two',
      fn: async () => {
        /** Successful calls under former sequential loop. */
        const serial: TranslateConcurrency = {
          now: 0,
          peak: 0,
        };

        /** Successful calls under two slices in flight. */
        const overlapped: TranslateConcurrency = {
          now: 0,
          peak: 0,
        };
        await runDriver({
          overlap: 1,
          translateConcurrency: serial,
        },);
        await runDriver({
          overlap: 2,
          translateConcurrency: overlapped,
        },);
        expect(serial.peak,).toBe(TRANSLATORS.length,);
        expect(overlapped.peak,).toBeGreaterThan(serial.peak,);
      },
    },),

    it({
      name: 'REFUSES a roster with nobody in it BEFORE buying anything, which is the whole point: '
        + 'a stage that can never speak settles exactly like one whose voices all failed, so an '
        + 'unconfigured pass would spend hours writing settled documents nobody translated and '
        + 'every later reader would take them for pages that needed no work',
      fn: async () => {
        /**
         Client that fails any exchange, so the case proves nothing was bought
         rather than only that the run refused.
         */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('no exchange may be attempted under an empty roster',);
          },
          chatJson: async () => {
            throw new Error('no exchange may be attempted under an empty roster',);
          },
          quotas: async () => {
            throw new Error('no exchange may be attempted under an empty roster',);
          },
        };

        /**
         Failure the driver raised.
         */
        let caught: unknown;
        try {
          await translateDocument({
            client,
            prepared: prepareDocumentPair({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
            },),
            models: {
              translatorModelIds: [],
              judgeModelIds: [],
            },
            signal: new AbortController().signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(RosterConfigurationError,);
        expect(String(caught,),).toContain('translatorModelIds',);
        expect(String(caught,),).toContain('judgeModelIds',);
      },
    },),

    it({
      name: 'REASSEMBLES the document from per-slice decisions, leaving every '
        + 'byte outside the slices untouched: headings and blank lines belong '
        + 'to the document rather than to any slice',
      fn: async () => {
        const { result, } = await runDriver({},);
        expect(result.translatedText,).toContain('## Section one',);
        expect(result.translatedText,).toContain('## Section two',);
        expect(result.translatedText,).toContain(FRESH,);
        expect(result.changedSliceCount,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'records what each slice decided, including the slate and every '
        + 'ballot, so a shipped translation can be traced to the round that '
        + 'chose it rather than to a log line that may not survive the run',
      fn: async () => {
        const { result, } = await runDriver({},);
        for (const record of result.slices) {
          expect(record.stageResult.slate.length,).toBeGreaterThan(0,);
          expect(record.alignment.kind,).toBe('within-limit',);
        }
      },
    },),

    it({
      name:
        'DISCARDS a cached record that heard NO translator and buys that slice again, which closes '
          + 'the other end of the no-caching rule: this driver never writes such a record, so one in a '
          + 'cache came from another build, and resuming it would settle the archive standing by default '
          + 'without anybody having been asked in this run',
      fn: async () => {
        const { persisted, } = await runDriver({},);

        /**
         The same records under the same keys, each with its voices taken
         away and nothing else touched, so whatever the run recomputes it
         recomputes for having heard nobody rather than for any other fault.
         */
        const unheard = new Map(
          [...persisted.entries(),].map(function toUnheard([
            key,
            record,
          ],) {
            return [
              key,
              {
                ...record,
                stageResult: {
                  ...record.stageResult,
                  heardTranslators: 0,
                },
              },
            ] as const;
          },),
        );
        expect(unheard.size,).toBeGreaterThan(0,);

        /**
         Run offered nothing but unheard records.
         */
        const asked = await runDriver({ resumed: unheard, },);
        expect(asked.result
          .resumedSliceCount,).toBe(0,);
        expect(asked.calls
          .translate,).toBeGreaterThan(0,);
        expect(
          asked.result
            .findings
            .filter(function namesRefusal(finding,): boolean {
              return finding.startsWith('translate-discarded-unheard-slice',);
            },)
            .length,
        ).toBe(asked.result
          .sliceCount,);
      },
    },),

    it({
      name: 'PERSISTS every settled slice as it goes, so a run killed at its '
        + 'cap leaves everything it already bought rather than starting over',
      fn: async () => {
        const { result, persisted, } = await runDriver({},);
        expect(persisted.size,).toBe(result.sliceCount,);
      },
    },),

    it({
      name: 'RESUMES a settled slice without spending a call, and STAMPS what it resumes with the '
        + 'index it asked under rather than the one the record carries. Since version 2 the key is '
        + 'the texts and the run shape, so a record legitimately answers for any slice with the same '
        + 'texts, and trusting its own index would splice one slice\'s text over another',
      fn: async () => {
        const { persisted, result: firstRun, } = await runDriver({},);
        const { calls, result, } = await runDriver({ resumed: persisted, },);
        expect(calls.translate,).toBe(0,);
        expect(calls.select,).toBe(0,);
        expect(result.resumedSliceCount,).toBe(result.sliceCount,);

        /**
         Same records under the same keys, each carrying an index that names
         some other slice.
         */
        const misfiled = new Map(
          [...persisted.entries(),].map(function toMisfiled([key, record,],) {
            return [
              key,
              {
                ...record,
                sliceIndex: record.sliceIndex + 1,
              },
            ] as const;
          },),
        );

        /**
         Run resuming every slice from those records.
         */
        const restamped = await runDriver({ resumed: misfiled, },);
        expect(restamped.result
          .translatedText,).toBe(firstRun.translatedText,);
        expect(restamped.result
          .slices
          .map(function toIndex(record,): number {
            return record.sliceIndex;
          },),).toEqual(firstRun.slices
            .map(function toIndex(record,): number {
              return record.sliceIndex;
            },),);
      },
    },),

    it({
      name: 'ASKS ONCE for two slices carrying identical text, and settles both from that one answer. '
        + 'Since version 2 the key is the texts and the run shape, so both slices ask one question: a '
        + 'run that answered it twice would keep two different answers, persist both under one key, '
        + 'and settle differently from the resumed run that reads one record back for both',
      fn: async () => {
        await Promise.all(([1, 2,] as const).map(async function atOverlap(
          overlap,
        ): Promise<void> {
          /** Two byte-identical sections on both sides. */
          const twinSource = '## 第一节\n\n猫猫在窗台上打盹。\n\n## 第一节\n\n猫猫在窗台上打盹。\n';

          /** Their translation, identical for same reason. */
          const twinTarget = '## Section one\n\nThe cat is doing the sleeping on the windowsill.\n\n'
            + '## Section one\n\nThe cat is doing the sleeping on the windowsill.\n';

          /** Run over twin document. */
          const twins = await runDriver({
            sourceText: twinSource,
            targetText: twinTarget,
            overlap,
          },);

          /** Run over one section alone, for call count of one question. */
          const single = await runDriver({
            sourceText: '## 第一节\n\n猫猫在窗台上打盹。\n',
            targetText: '## Section one\n\nThe cat is doing the sleeping on the windowsill.\n',
            overlap,
          },);

          /** Warm run over both twins, reading one shared record from disk. */
          const warm = await runDriver({
            sourceText: twinSource,
            targetText: twinTarget,
            resumed: twins.persisted,
            overlap,
          },);
          expect(twins.result
            .sliceCount,).toBe(2,);
          expect(single.result
            .sliceCount,).toBe(1,);
          expect(twins.calls
            .translate,).toBe(single.calls
              .translate,);
          expect(twins.persisted
            .size,).toBe(1,);
          expect(twins.result
            .resumedSliceCount,).toBe(0,);
          expect(warm.result
            .resumedSliceCount,).toBe(2,);
          expect(twins.result
            .slices[0]
            ?.outputText,).toBe(twins.result
              .slices[1]
              ?.outputText,);
          expect(twins.result
            .slices
            .map(function toIndex(record,): number {
              return record.sliceIndex;
            },),).toEqual([
              0,
              1,
            ],);
        },),);
      },
    },),

    it({
      name:
        'ASKS AGAIN for the twin of a slice NO translator answered, rather than reusing it within the '
          + 'run. In-run memoization exists to make a cold run settle what a warm run settles, and a warm '
          + 'run can only resume what reached the cache: nothing did here, so reusing it would make the '
          + 'two disagree in exactly the case the memoization was added to fix',
      fn: async () => {
        /** Two byte-identical sections on both sides. */
        const twinSource = '## 第一节\n\n猫猫在窗台上打盹。\n\n## 第一节\n\n猫猫在窗台上打盹。\n';

        /** Their translation, identical for the same reason. */
        const twinTarget = '## Section one\n\nThe cat is doing the sleeping on the windowsill.\n\n'
          + '## Section one\n\nThe cat is doing the sleeping on the windowsill.\n';

        /** Run over the twin document with every translator down. */
        const twins = await runDriver({
          sourceText: twinSource,
          targetText: twinTarget,
          silentTranslators: true,
        },);

        /** The same, over one of those sections alone. */
        const single = await runDriver({
          sourceText: '## 第一节\n\n猫猫在窗台上打盹。\n',
          targetText: '## Section one\n\nThe cat is doing the sleeping on the windowsill.\n',
          silentTranslators: true,
        },);
        expect(twins.result
          .sliceCount,).toBe(2,);
        expect(twins.persisted
          .size,).toBe(0,);

        expect(twins.calls
          .translate,).toBe(single.calls
            .translate * 2,);

        // WHAT ACTUALLY SEPARATES THE TWO BEHAVIOURS, and it is not the call
        // count: the resume branch refuses an unheard record whatever put it
        // there, so the twins cost two questions under either arrangement.
        // What differs is whether the second twin ever MET such a record.
        // Memoized unconditionally, it meets one and reports discarding it;
        // memoized where it is persisted, there is nothing to discard and the
        // run is silent.
        expect(
          twins.result
            .findings
            .filter(function namesRefusal(finding,): boolean {
              return finding.startsWith('translate-discarded-unheard-slice',);
            },),
        ).toEqual([],);
      },
    },),

    it({
      name: 'DISCARDS a cached record that contradicts its own text and buys '
        + 'that slice again, in BOTH directions. A resumed record is trusted on '
        + 'its slice index alone, so a truncated write that still parses, or a '
        + 'slicing that moved while the key did not, otherwise reaches assembly '
        + 'and fails the whole document after every other slice has been paid '
        + 'for. One bad cache file costs one slice instead, and says so in the '
        + 'findings, since a recomputed slice is otherwise indistinguishable '
        + 'from one that was never cached',
      fn: async () => {
        const { persisted, } = await runDriver({},);

        /**
         Archive wording of each prepared slice, read from the same
         preparation the driver builds rather than assumed from the fixture.
         */
        const incumbentByIndex = new Map(
          prepareDocumentPair({
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
          },)
            .slices
            .map(function toEntry(slice,): readonly [number, string,] {
              return [
                slice.target.sliceIndex,
                slice.target.text,
              ] as const;
            },),
        );

        /**
         Same records under the same keys, each claiming its change while
         carrying the wording it claims to have replaced.
         */
        const overClaiming = new Map(
          [...persisted.entries(),].map(function toPoisoned([key, record,],) {
            return [
              key,
              {
                ...record,
                outputText: incumbentByIndex.get(record.sliceIndex,)
                  ?? record.outputText,
              },
            ] as const;
          },),
        );

        /**
         Records the poisoning actually put in a contradictory state.

         Not every record reaches one: rewriting the wording of a record that
         already claimed no change leaves it consistent. Counted rather than
         assumed, so the case says exactly how many slices should be bought
         again instead of asserting a number the fixture happens to produce.
         */
        const overClaimingPoisoned = [...overClaiming.values(),]
          .filter(function wasPoisoned(record,): boolean {
            return record.changed
              && (record.outputText === incumbentByIndex.get(record.sliceIndex,));
          },)
          .length;
        expect(overClaimingPoisoned,).toBeGreaterThan(0,);

        /**
         Run resuming the over-claiming records.
         */
        const overClaimed = await runDriver({ resumed: overClaiming, },);
        expect(overClaimed.result
          .sliceCount - overClaimed.result
            .resumedSliceCount,).toBe(overClaimingPoisoned,);
        expect(overClaimed.calls
          .translate,).toBeGreaterThan(0,);
        expect(overClaimed.result
          .findings
          .filter(function namesDiscard(finding,): boolean {
            return finding.startsWith('translate-discarded-contradictory-slice',);
          },)
          .length,).toBe(overClaimingPoisoned,);

        /**
         The QUIETER direction: records denying a change they did make. Only
         `changed` records become replacements, so this one used to have its
         wording dropped at assembly with nothing said.
         */
        const underClaiming = new Map(
          [...persisted.entries(),].map(function toPoisoned([key, record,],) {
            return [
              key,
              {
                ...record,
                changed: false,
              },
            ] as const;
          },),
        );

        /**
         Records that direction puts in a contradictory state.
         */
        const underClaimingPoisoned = [...underClaiming.values(),]
          .filter(function wasPoisoned(record,): boolean {
            return record.outputText !== incumbentByIndex.get(record.sliceIndex,);
          },)
          .length;
        expect(underClaimingPoisoned,).toBeGreaterThan(0,);

        const underClaimed = await runDriver({ resumed: underClaiming, },);
        expect(underClaimed.result
          .sliceCount - underClaimed.result
            .resumedSliceCount,).toBe(underClaimingPoisoned,);
        expect(underClaimed.result
          .findings
          .filter(function namesDiscard(finding,): boolean {
            return finding.startsWith('translate-discarded-contradictory-slice',);
          },)
          .length,).toBe(underClaimingPoisoned,);
      },
    },),

    it({
      name: 'THROWS on a caller abort rather than settling the slices it never '
        + 'bought, and caches none of them. An abort reaches every stage as '
        + 'silence rather than as a failure, so an unguarded driver ships the '
        + 'incumbent unjudged for every remaining slice and writes that to the '
        + 'cache, where the next attempt reads it as finished work',
      fn: async () => {
        /**
         Records that reached the cache before the abort.
         */
        const persisted = new Map<string, TranslateSliceRecord>();
        await expect(runDriver({
          abortAfterTranslateCalls: TRANSLATORS.length,
          persisted,
        },),)
          .rejects
          .toBe(ENTRY_DEADLINE_FAILURE,);
        // The first slice was bought and settled; the second was not, and must
        // not be sitting in the cache claiming otherwise.
        expect(persisted.size,).toBe(1,);
      },
    },),

    it({
      name: 'throws on an abort that arrives AFTER a stage reached quorum, '
        + 'which is the window where nothing else notices: the voices that beat '
        + 'the abort are enough to decide, every ask it tore down is discarded '
        + 'by the round rather than raised, and the slice settles on a roster '
        + 'the abort chose',
      fn: async () => {
        /**
         Records that reached the cache before the abort.
         */
        const persisted = new Map<string, TranslateSliceRecord>();

        /**
         Voices a three-model roster needs, which the second slice hears
         before the abort lands on its last translator.
         */
        const quorum = Math.ceil(TRANSLATORS.length / 2,);

        /**
         Calls the run made, which say where it stopped.
         */
        const calls: CallLog = {
          translate: 0,
          translateAttempts: 0,
          select: 0,
          selectAttempts: 0,
        };
        await expect(runDriver({
          abortAfterTranslateCalls: TRANSLATORS.length + quorum,
          persisted,
          calls,
        },),)
          .rejects
          .toBe(ENTRY_DEADLINE_FAILURE,);
        expect(persisted.size,).toBe(1,);
        // The abort landed inside the second slice's translate round, so the
        // only judging this run may have paid for is the first slice's. A round
        // that returned its surviving voices instead of raising the abort would
        // have sent the whole judge roster out on a run already over.
        expect(calls.selectAttempts,).toBe(firstRoundWindow({ benchSize: MODELS.judgeModelIds.length, },),);
      },
    },),

    it({
      name: 'settles a slice NO translator answered, and deliberately does not '
        + 'cache it: the incumbent stands for this run, and the next attempt '
        + 'asks again rather than reading a provider outage as a decision',
      fn: async () => {
        /**
         Records that reached the cache with every translator down.
         */
        const persisted = new Map<string, TranslateSliceRecord>();
        const { result, } = await runDriver({
          silentTranslators: true,
          persisted,
        },);
        expect(result.sliceCount,).toBeGreaterThan(1,);
        expect(result.changedSliceCount,).toBe(0,);
        expect(persisted.size,).toBe(0,);
        expect(result.findings.some(function isUnheard(finding,): boolean {
          return finding.startsWith('translate-heard-no-translator',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'WITHDRAWS a replacement that would break a footnote spanning two '
        + 'slices, and counts what SHIPPED rather than what the judges chose. '
        + 'Each slice validated on its own: the sentence reads well, and the '
        + 'marker it lost belongs to a line the judges never saw. The note is '
        + 'the page\'s own (the original never cites it), since a marker the '
        + 'ORIGINAL carries is refused at the slice floor first (class ninety-two)',
      fn: async () => {
        const { result, } = await runDriver({
          sourceText: `## 第一节

猫猫在窗台上打盹。

## 第二节

窗台上有一只鸟。

## 注

〔1〕：那是它最喜欢的位置。
`,
          targetText: `## Section one

The cat is doing the sleeping on the windowsill[^1].

## Section two

On the windowsill there is being a bird.

## Notes

[^1]: That is its favourite spot.
`,
        },);
        // The scripted translators render the first section WITHOUT its marker,
        // so shipping their text would leave the definition orphaned.
        expect(result.withdrawnSliceCount,).toBeGreaterThan(0,);
        expect(result.translatedText,).toContain(
          'The cat is doing the sleeping on the windowsill[^1].',
        );
        expect(result.findings
          .some(function namesWithdrawal(finding,): boolean {
            return finding.startsWith('assembly-footnote-',);
          },),).toBe(true,);
        // The record still says the judges chose a replacement; the document
        // says what it could carry. Both are true and they disagree.
        expect(result.slices
          .some(function chose(record,): boolean {
            return record.changed;
          },),).toBe(true,);
        // And the NAMED sets follow the document rather than the records, which
        // is the whole reason they exist: a reader joining two lanes by slice
        // must not credit this lane with a slice it did not change.
        expect(result.withdrawnSliceIndices
          .length,).toBe(result.withdrawnSliceCount,);
        expect(result.changedSliceIndices
          .length,).toBe(result.changedSliceCount,);
        for (const sliceIndex of result.withdrawnSliceIndices) {
          expect(result.changedSliceIndices
            .includes(sliceIndex,),).toBe(false,);
        }
        expect(result.changedSliceIndices
          .toSorted(function ascending(
            left,
            right,
          ): number {
            return left - right;
          },),).toEqual(result.changedSliceIndices,);
      },
    },),

    it({
      name: 'REFUSES to replace archive text the source cannot account for, '
        + 'and records the refusal as its own disposition. The judges are not '
        + 'wrong here: asked which text better renders a heading, they pick the '
        + 'rendering of the heading, and the passage it would replace is the '
        + 'thing the archive came for',
      fn: async () => {
        const { result, } = await runDriver({
          sourceText: `## 第一节

其一：

## 第二节

窗台上有一只鸟。
`,
          targetText: `## Section one

But we must remember that the cat sleeping on the windowsill has been there `
            + `since the spring, and the household has arranged itself around `
            + `that habit rather than against it, which is the sort of thing `
            + `nobody writes down until it is gone.

## Section two

On the windowsill there is being a bird.
`,
        },);
        expect(result.refusedSliceCount,).toBe(1,);

        /**
         Slice the guard protected.
         */
        const [refused,] = result.slices
          .filter(function wasRefused(record,): boolean {
            return record.disposition === 'refused-alignment';
          },);
        expect(refused?.changed,).toBe(false,);
        expect(refused?.outputText,).toContain('nobody writes down',);
        // The stage result is kept whole, so "the judges wanted a replacement
        // and the guard refused" stays distinguishable from "the judges kept
        // the incumbent". Both ship the same text and mean opposite things.
        expect(refused?.stageResult.text,).toContain(FRESH,);
        expect(result.findings.some(function isRefusal(finding,): boolean {
          return finding.startsWith('translate-refused-alignment',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS NO ARCHIVE THE PUBLICATION RULE REFUSES, so the judges\' replacement goes on to the '
        + 'consolidation instead of an archive it will refuse as a standing (ledger X6: up to 47 such archives in the '
        + 'run logs, 46 lacking a link the original carries and 1 keeping the untranslated pronoun, the floor '
        + 'this fixture trips)',
      fn: async () => {
        const { result, } = await runDriver({
          sourceText: `## 第一节

其一：TA睡了。

## 第二节

窗台上有一只鸟。
`,
          targetText: `## Section one

But we must remember that TA, the cat sleeping on the windowsill, has been there `
            + `since the spring, and the household has arranged itself around `
            + `that habit rather than against it, which is the sort of thing `
            + `nobody writes down until it is gone.

## Section two

On the windowsill there is being a bird.
`,
        },);

        /**
         Slice whose archive keeps the untranslated pronoun.
         */
        const [ineligibleArchive,] = result.slices;
        expect({
          refused: result.refusedSliceCount,
          disposition: ineligibleArchive?.disposition,
          replaced: ineligibleArchive?.outputText
            .includes(FRESH,),
          recorded: ineligibleArchive?.findings
            .some(function isIneligibleArchive(finding,): boolean {
              return finding.startsWith('translate-archive-ineligible',);
            },),
        },).toEqual({
          refused: 0,
          disposition: 'stage-result',
          replaced: true,
          recorded: true,
        },);
      },
    },),
    it({
      name: 'names the slice each refusal is FOR, even when two slices share one '
        + 'settled record. Identical sections ask one question, so the second '
        + 'resumes the first\'s record and is stamped with its own index; a '
        + 'refusal sentence stored inside that record would name the first slice '
        + 'twice and the second never',
      fn: async () => {
        /** Section the guard refuses, written twice with nothing to tell the two apart. */
        const SECTION = `## 第一节

其一：
`;

        /** Its archive text, long enough that the source cannot account for it. */
        const RENDERED = `## Section one

But we must remember that the cat sleeping on the windowsill has been there `
          + `since the spring, and the household has arranged itself around that `
          + `habit rather than against it, which is the sort of thing nobody `
          + `writes down until it is gone.
`;
        const { result, } = await runDriver({
          sourceText: `${SECTION}\n${SECTION}`,
          targetText: `${RENDERED}\n${RENDERED}`,
        },);
        expect(result.refusedSliceCount,).toBe(2,);

        /**
         Refusal sentences the document reports.
         */
        const refusals = result.findings
          .filter(function isRefusal(finding,): boolean {
            return finding.startsWith('translate-refused-alignment',);
          },);
        expect(refusals.length,).toBe(2,);
        expect(refusals[0],).toContain('slice 0',);
        expect(refusals[1],).toContain('slice 1',);
      },
    },),
    it({
      name: 'ASKS AGAIN for a twin of a slice no translator answered, because the in-run memo may only '
        + 'hold what a warm run could resume and this slice is deliberately not cached. Reusing it '
        + 'would make a cold run settle on a silence a warm run would have re-asked',
      fn: async () => {
        await Promise.all(([1, 2,] as const).map(async function atOverlap(
          overlap,
        ): Promise<void> {
          /** Section written twice, so both slices ask one question. */
          const SECTION = `## 第一节

猫猫在窗台上打盹。
`;

          /** Its archive wording, likewise written twice. */
          const RENDERED = `## Section one

The cat is doing the sleeping on the windowsill.
`;

          /**
           One slice, which calibrates what asking once costs.

           Measured rather than assumed: roster retries a lost voice, so count
           per slice is property of gather rather than translator list length.
           */
          const single = await runDriver({
            sourceText: SECTION,
            targetText: RENDERED,
            silentTranslators: true,
            overlap,
          },);

          /**
           Same section twice.
           */
          const twin = await runDriver({
            sourceText: `${SECTION}\n${SECTION}`,
            targetText: `${RENDERED}\n${RENDERED}`,
            silentTranslators: true,
            overlap,
          },);
          expect(single.result
            .sliceCount,).toBe(1,);
          expect(twin.result
            .sliceCount,).toBe(2,);
          expect(single.calls
            .translateAttempts,).toBeGreaterThan(0,);
          expect(twin.calls
            .translateAttempts,).toBe(single.calls
              .translateAttempts * 2,);
          expect(twin.persisted
            .size,).toBe(0,);
        },),);
      },
    },),

    it({
      name: 'PAUSES absent-passage repair when no translator is available instead of settling unfilled page',
      fn: async () => {
        await expect(runDriver({
          sourceText: SHORT_SOURCE,
          targetText: SHORT_TARGET,
          anchorSource: MISSING_SOURCE,
          silentForSource: MISSING_SOURCE,
        },),).rejects.toThrow(TranslationRepairInterruptedError,);
      },
    },),

    it({
      name: 'HONORS authoritative production admission for a linked source gap that whole-page '
        + 'shortfall alone would refuse, and carries admission evidence into result findings',
      fn: async () => {
        const { result, prepared, } = await runDriver({
          anchorSource: MISSING_SOURCE,
          insertionAdmission: {
            positions: new Set([2,]),
            findings: ['insertion-corroboration fixture admitted',],
          },
        },);
        const anchorIndex = prepared.slices.length - 1;

        expect(anchorIndex,).toBe(2,);
        expect(result.status,).toBe('complete',);
        expect(result.unfilled,).toEqual([],);
        expect(result.changedSliceIndices,).toContain(anchorIndex,);
        expect(result.translatedText,).toContain(MISSING_FRESH,);
        expect(result.findings,).toContain('insertion-corroboration fixture admitted',);
      },
    },),

    it({
      name: 'LEAVES AN ADMITTED PASSAGE UNFILLED, before buying a round, where the floor can compare nothing '
        + 'written for it, rather than writing in a rendering nobody could check (ledger B43)',
      fn: async () => {
        /** Cost of the document with nothing appended. */
        const plain = await runDriver({},);

        /**
         Passage whose original carries an expression the strict grammar never
         closes.
         */
        const unreadableSource = '## 第三节\n\n猫猫{也喜欢晒太阳。';
        const {
          result,
          prepared,
          calls,
        } = await runDriver({
          anchorSource: unreadableSource,
          insertionAdmission: {
            positions: new Set([2,]),
            findings: ['insertion-corroboration fixture admitted',],
          },
        },);
        const anchorIndex = prepared.slices.length - 1;

        /**
         The floor's account of why it could compare nothing there.
         */
        const verdict = validateTranslatedSlice({
          sourceText: unreadableSource,
          candidateText: MISSING_FRESH,
          pageText: '',
          lineStructured: false,
          declared: [],
          disputedWordings: [],
        },);
        if (verdict.kind !== 'unknown')
          throw new Error(`the anchor's original must be one no grammar reads, and the floor said ${verdict.kind}`,);

        expect(anchorIndex,).toBe(2,);
        expect(result.status,).toBe('unfilled',);
        expect(result.unfilled,).toEqual([{
          sliceIndex: anchorIndex,
          reason: 'unfloored',
          findings: [unflooredFinding({ stage: 'translate', detail: verdict.detail, },),],
        },],);
        expect(calls.translateAttempts,).toBe(plain.calls
          .translateAttempts,);
        expect(result.changedSliceIndices,).not.toContain(anchorIndex,);
        expect(result.translatedText,).not.toContain(MISSING_FRESH,);
      },
    },),

    it({
      name: 'TREATS source-only passage proven carried elsewhere as complete without local insertion',
      fn: async () => {
        const { result, prepared, calls, } = await runDriver({
          anchorSource: MISSING_SOURCE,
          insertionAdmission: {
            positions: new Set(),
            carried: [{
              position: 2,
              sliceIndex: 2,
              sourceText: MISSING_SOURCE,
              evidence: ['The cat is doing the sleeping on the windowsill.',],
            },],
            findings: ['insertion-coverage fixture carried',],
          },
        },);
        const anchorIndex = prepared.slices.length - 1;

        expect(anchorIndex,).toBe(2,);
        expect(result.status,).toBe('complete',);
        expect(result.unfilled,).toEqual([],);
        expect(result.sliceTexts[anchorIndex]?.outcome,).toEqual({ kind: 'not-applicable', },);
        expect(result.findings,).toContain('insertion-coverage fixture carried',);
        expect(calls.translateAttempts,).toBeGreaterThan(0,);
        expect(result.translatedText,).not.toContain(MISSING_FRESH,);
      },
    },),

    it({
      name: 'REFUSES the same untranslated passage on a page that is not short, before buying a '
        + 'single round, because a page already carrying its ordinary weight of English is not '
        + 'missing anything an insertion could supply',
      fn: async () => {
        /** Cost of the document with nothing appended. */
        const plain = await runDriver({});

        /**
         Same document with an untranslated passage anchored at the end, on
         the VERBOSE pair rather than {@link SHORT_SOURCE}'s.
         */
        const anchored = await runDriver({ anchorSource: MISSING_SOURCE, },);

        /** Index the appended anchor holds. */
        const anchorIndex = anchored.prepared
          .slices
          .length - 1;
        expect(anchored.result
          .status,).toBe('unfilled',);
        expect(anchored.result
          .unfilled
          .map(function toIndex(passage,): number {
            return passage.sliceIndex;
          },),).toEqual([anchorIndex,],);
        expect(anchored.result
          .unfilled[0]
          ?.reason,).toBe('not-corroborated',);

        // NOTHING WAS BOUGHT, which is the whole point of a deterministic gate
        // in front of a paid round: a refused passage carries no evidence
        // because no round was ever put. Asserting the finding list is empty is
        // what separates this from a round that ran and came back silent.
        expect(anchored.result
          .unfilled[0]
          ?.findings,).toEqual([],);
        expect(anchored.calls
          .translateAttempts,).toBe(plain.calls
            .translateAttempts,);

        // The page keeps what it had, and the passage is still reported.
        expect(anchored.result
          .changedSliceIndices,).not.toContain(anchorIndex,);
        expect(anchored.result
          .translatedText,).not.toContain('晒太阳',);
      },
    },),
    it({
      name: 'NAMES THE SLICE on every slate and judge line when slices run side by side (ledger A11: '
        + 'the translate lane\'s lines carried no slice under overlap)',
      fn: async () => {
        /**
         Messages the lane logged.
         */
        const messages: string[] = [];
        await runDriver({
          overlap: 2,
          messages,
        },);
        /**
         Lines the slate producers and judges wrote.
         */
        const staged = messages.filter(function fromSlate(line,): boolean {
          return line.includes('[produceTranslateSlate]',) || line.includes('[judgeTranslateSlate]',);
        },);
        expect(staged.length,).toBeGreaterThan(0,);
        expect(staged.every(function namesSlice(line,): boolean {
          return line.includes('[translate slice ',);
        },),).toBe(true,);
        expect([
          0,
          1,
        ].map(function linesFor(sliceIndex,): boolean {
          return staged.some(function namesThis(line,): boolean {
            return line.includes(`[translate slice ${String(sliceIndex,)}]`,);
          },);
        },),).toStrictEqual([
          true,
          true,
        ],);
      },
    },),

    it({
      name: 'WRITES A LATER SLICE\'S OWN FAILURE UNDER THE CALLER\'S STOP, the roster its hook re-seated with no '
        + 'translator refused as the stop landed while the first slice ended in the caller\'s reason, and refuses '
        + 'with that reason',
      fn: async () => {
        /**
         Every line the driver wrote.
         */
        const messages: string[] = [];
        /**
         The caller's steering, which the later slice's hook stops.
         */
        const steering = new AbortController();
        /**
         What the driver refused with.
         */
        const refusal = await rejectionOf(async function translatesUnderTheStop(): Promise<unknown> {
          return await translateDocument({
            client: UNASKED_CLIENT,
            prepared: prepareDocumentPair({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
            },),
            models: MODELS,
            signal: steering.signal,
            perCallTimeoutMs: HANG_STOP_MS,
            overlap: 2,
            beforeSlice: stopLandingHook<TranslateSliceSeating>({
              steering,
              reason: ENTRY_DEADLINE_FAILURE,
              afterStop: function seatNoTranslator(): TranslateSliceSeating {
                return {
                  translateModels: {
                    ...MODELS,
                    translatorModelIds: [],
                  },
                };
              },
            },),
            l: capturingLogger({ messages, },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);
        expect(refusal,).toBe(ENTRY_DEADLINE_FAILURE,);
        expect(messages,).toEqual([
          '[translateDocument] [mapOverlapped] slice 1 failed (refused by RosterConfigurationError), and the caller '
            + 'stopped the work',
        ],);
      },
    },),
  ],
},);
