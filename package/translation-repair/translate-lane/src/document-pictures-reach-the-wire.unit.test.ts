/**
 Guard that a picture's transcription, once corroborated, actually reaches
 the requests `translateDocument` sends, not merely its return value.

 WHY THIS FILE EXISTS: the translate lane's judging window was fully built, wired into
 the cache key, and sat in production for weeks unused. Nothing failed and no
 test broke, because nothing asserted what the driver actually SENT to a
 model; every test that passed was reading `translateDocument`'s return
 value, which the missing wiring never touched. Pictures wire through the
 same two places, `translateSliceKey` and `attemptTranslateSlice`, from the
 same `pictures.context` value computed once in `translate-document.ts`, so
 the same gap is possible here, and this file is the guard against it
 repeating.

 WHAT IS PINNED, per exchange the recording client double captures before it
 answers: a corroborated reading's own transcription text reaches the
 TRANSLATOR sheet (`translate-wire.ts`'s "WHAT THE PICTURES HERE SAY" block)
 and the JUDGE sheet (`candidate-select-wire.ts`'s evidence block, carrying
 `translate-judge.ts`'s label of the same name); a run handed no readings at
 all sends neither and differs measurably from a run handed one, which is the
 positive control this guard needs, since an assertion that always passes
 looks identical to one that works; and an `unavailable` reading leaves a
 finding naming its asset on the slice record while reaching neither sheet,
 since `slice-pictures.ts` treats a refused reading as evidence for a person
 to read rather than a hedge for a model to weigh.

 Every assertion reads the RECORDED REQUEST the client double captured,
 never `translateDocument`'s return value: the return value is exactly what
 the judging-window gap left intact, and reading it again would prove nothing this
 file exists to prove.

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
import type { PairedReading, } from '@monochromatic-dev/translation-repair-slice-reading';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  translateDocument,
  type TranslateDocumentResult,
  type TranslateModels,
} from '../dist/final/node/index.mjs';
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
 Logger for the driver under test.
 */
const l = tagged({ tag: 'document-pictures-reach-the-wire-test', },);

/**
 Models that render each slice, reused from the sibling document-driver
 suite: a roster this small and this shaped is already known to seat both
 stages without extra scripting.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
];

/**
 Rosters the driver seats for every run in this file.
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
 Placeholder the corpus writes an entry's own directory as, inside a photo
 element. AN ESCAPED TEMPLATE LITERAL, so the characters this file writes are
 the ones the corpus carries, rather than leaving an interpolation slot open.
 */
const ENTRY_PLACEHOLDER = `\${path}`;

/**
 Builds one `<PhotoScroll />` element naming a single asset, the only shape
 `photo-reference.ts` reads and the only shape the pinned corpus writes.

 @param assetName - file name the element names within an entry's photos
 directory, which is also the key `pictureReadings` looks it up under

 @returns Element exactly as a page in the corpus writes it

 @example
 ```ts
 const element = photoElement({ assetName: 'tuna-tin-nap.webp', },);
 ```
 */
function photoElement({ assetName, }: { readonly assetName: string; },): string {
  return `<PhotoScroll photos={[ '${ENTRY_PLACEHOLDER}/photos/${assetName}' ]} />`;
}

/**
 Asset name the corroborated-reading fixture's own `<PhotoScroll />` element
 names. Distinctive on purpose, so a match for it anywhere in a request can
 only have come from the picture channel rather than from ordinary prose.
 */
const CORROBORATED_ASSET_NAME = 'tuna-tin-nap.webp';

/**
 What both readers agreed the picture shows, invented for this fixture.
 Distinctive wording, so finding this exact sentence in a request proves the
 picture channel carried it there rather than some other block coinciding.
 */
const CORROBORATED_READING_TEXT = 'Whiskers is curled up asleep inside an empty tuna tin, one paw over her nose.';

/**
 Corroborated reading handed to the driver for
 {@link CORROBORATED_ASSET_NAME}, built by hand rather than through
 `readImagePair`: production hands this map in as data, per
 `translate-document.ts`'s own note that gathering pictures is the corpus
 layer's business, not this driver's.
 */
const CORROBORATED_READING: PairedReading = {
  kind: 'corroborated',
  readings: [
    {
      modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
      text: CORROBORATED_READING_TEXT,
    },
    {
      modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
      text: CORROBORATED_READING_TEXT,
    },
  ],
  overlap: 0.97,
};

/**
 Original: one section whose source names {@link CORROBORATED_ASSET_NAME},
 the way a picture-bearing corpus entry does.
 */
const SOURCE_TEXT = `## 第一节

猫猫窝在纸箱里打盹。

${photoElement({ assetName: CORROBORATED_ASSET_NAME, },)}
`;

/**
 Archive translation, awkward on purpose so a translator's fresh rendering
 can never be mistaken for the text already there.
 */
const TARGET_TEXT = `## Section one

The cat is doing the napping inside of the cardboard box.

${photoElement({ assetName: CORROBORATED_ASSET_NAME, },)}
`;

/**
 What every translator renders for {@link SOURCE_TEXT}, identical across the
 roster so the slate collapses to one fresh candidate standing against the
 archive, which is what makes the judges' fan-out happen at all.
 */
const FRESH_RENDERING = `## Section one

The cat naps inside the cardboard box.

${photoElement({ assetName: CORROBORATED_ASSET_NAME, },)}
`;

/**
 Asset name the unavailable-reading fixture's element names. A different name
 from {@link CORROBORATED_ASSET_NAME}, so the two fixtures cannot be confused
 for one another inside a single recorded request.
 */
const UNAVAILABLE_ASSET_NAME = 'blurry-tabby.webp';

/**
 Reading the driver is handed for {@link UNAVAILABLE_ASSET_NAME}: the readers
 disagreed, so nothing here may be used. The reason is `readers-disagree`
 because the pin is about the `unavailable` kind, not about which reason
 produced it.
 */
const UNAVAILABLE_READING: PairedReading = {
  kind: 'unavailable',
  reason: 'readers-disagree',
  transient: false,
  perReader: [
    'hf:moonshotai/Kimi-K3: describes a sleeping cat',
    'hf:zai-org/GLM-5.3-Flash: describes an empty windowsill',
  ],
  overlap: 0.104,
};

/**
 Original: one section whose source names {@link UNAVAILABLE_ASSET_NAME}.
 */
const UNAVAILABLE_SOURCE_TEXT = `## 第一节

猫猫盯着相框里的自己打量。

${photoElement({ assetName: UNAVAILABLE_ASSET_NAME, },)}
`;

/**
 Archive translation for {@link UNAVAILABLE_SOURCE_TEXT}, awkward for the
 same reason {@link TARGET_TEXT} is.
 */
const UNAVAILABLE_TARGET_TEXT = `## Section one

The cat is doing the staring at its own reflection in the photo frame.

${photoElement({ assetName: UNAVAILABLE_ASSET_NAME, },)}
`;

/**
 What every translator renders for {@link UNAVAILABLE_SOURCE_TEXT}.
 */
const UNAVAILABLE_FRESH_RENDERING = `## Section one

The cat studies its own reflection in the photo frame.

${photoElement({ assetName: UNAVAILABLE_ASSET_NAME, },)}
`;

/**
 One exchange a run attempted, kept so a case can inspect exactly what
 reached the wire rather than trusting the driver's return value.

 @example
 ```ts
 const recorded: RecordedRequest = { schema: 'translation_report', content: 'ORIGINAL...', };
 ```
 */
type RecordedRequest = {
  /**
   Structured-output schema name, which tells a translator exchange from a
   judge exchange the same way the driver's own schema names do.
   */
  readonly schema: string;

  /**
   Every message's text, joined, so a needle search reads the whole exchange
   rather than one message chosen in advance.
   */
  readonly content: string;
};

/**
 Builds a client that renders one fixed translation for every translator
 call, always ballots for the first candidate on a judge's slate, and
 appends every exchange it receives to `requests` before answering, so a
 case can inspect what was SENT rather than only what the driver returned.

 @param requests - log this client appends every exchange to, in call order

 @param translatorRendering - text every translator call answers with

 @returns Client honoring the script its parameters describe

 @example
 ```ts
 const client = recordingClient({ requests: [], translatorRendering: FRESH_RENDERING, },);
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
       Structured-output schema this exchange asked for, which names the
       stage the same way the driver's own schema check does.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name;

      /**
       Every message's text, joined into one haystack a case can search.
       */
      const content = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);

      requests.push({
        schema: schema ?? '',
        content,
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
         Ballot naming the first candidate on the slate, whichever rendering
         the rotation put there. WHICH candidate wins is not this file's
         question, only whether the picture reading reached the sheet the
         judges were shown, so any valid answer that lets the run finish
         serves the pin equally well.
         */
        const ballot: unknown = {
          best: 1,
          assessment: 'the cat is the whole point',
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
 Drives `translateDocument` once over one document pair, with every exchange
 it attempts recorded rather than only its return value kept.

 @param sourceText - original document

 @param targetText - translation as it stands

 @param translatorRendering - text every translator call answers with

 @param pictureReadings - picture readings handed to the driver; omitted for
 a run that gathers none, which is `translateDocument`'s own default

 @returns Result the driver settled on, and every exchange it attempted

 @example
 ```ts
 const { requests, } = await runDocument({ sourceText, targetText, translatorRendering: FRESH_RENDERING, },);
 ```
 */
async function runDocument(
  {
    sourceText,
    targetText,
    translatorRendering,
    pictureReadings,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly translatorRendering: string;
    readonly pictureReadings?: ReadonlyMap<string, PairedReading>;
  },
): Promise<{
  readonly result: TranslateDocumentResult;
  readonly requests: readonly RecordedRequest[];
}> {
  /**
   Exchanges this run attempts, filled in by the client double as they
   happen rather than reconstructed afterward.
   */
  const requests: RecordedRequest[] = [];

  /**
   Preparation the driver slices its work from.
   */
  const prepared = prepareDocumentPair({
    sourceText,
    targetText,
  },);

  /**
   What the driver settled on for this document.
   */
  const result = await translateDocument({
    client: recordingClient({
      requests,
      translatorRendering,
    },),
    prepared,
    models: MODELS,
    ...((pictureReadings === undefined) ? {} : { pictureReadings, }),
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return {
    result,
    requests,
  };
}

/**
 Picks the requests naming the translator schema out of everything one run
 attempted, which is what a translator actually received.

 @param requests - every exchange one run attempted

 @returns Requests asking for a rendered translation

 @example
 ```ts
 const sent = translatorRequestsOf({ requests, },);
 ```
 */
function translatorRequestsOf(
  { requests, }: { readonly requests: readonly RecordedRequest[]; },
): readonly RecordedRequest[] {
  return requests.filter(function isTranslator(request,): boolean {
    return request.schema === 'translation_report';
  },);
}

/**
 Picks the requests naming the judge schema out of everything one run
 attempted, which is what a judge actually received.

 @param requests - every exchange one run attempted

 @returns Requests asking for a ballot

 @example
 ```ts
 const sent = judgeRequestsOf({ requests, },);
 ```
 */
function judgeRequestsOf(
  { requests, }: { readonly requests: readonly RecordedRequest[]; },
): readonly RecordedRequest[] {
  return requests.filter(function isJudge(request,): boolean {
    return request.schema === 'candidate_ballot';
  },);
}

await describe({
  name: translateDocument.name,
  children: [
    it({
      name: 'SENDS a corroborated reading\'s own transcription text to the TRANSLATOR when the slice\'s '
        + 'source names the picture that reading covers: the translate lane\'s judging window was fully wired into the '
        + 'cache key and never reached the call it was keyed for, and nothing failed for weeks because no '
        + 'test read the request a translator was actually sent. This asserts against the RECORDED '
        + 'REQUEST, never against `translateDocument`\'s return value',
      fn: async () => {
        /**
         Run over the corroborated-reading fixture, with the reading handed
         in.
         */
        const { requests, } = await runDocument({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          translatorRendering: FRESH_RENDERING,
          pictureReadings: new Map<string, PairedReading>([
            [CORROBORATED_ASSET_NAME, CORROBORATED_READING,],
          ],),
        },);

        /**
         Every request a translator actually received this run.
         */
        const sent = translatorRequestsOf({ requests, },);
        expect(sent.length,).toBeGreaterThan(0,);
        expect(
          sent.some(function carriesReading(request,): boolean {
            return request.content.includes(CORROBORATED_READING_TEXT,);
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'SENDS the same corroborated reading\'s text to the JUDGE, which is the other half of the '
        + 'judging-window lesson: the cache key and one of the two calls can agree while the other call still '
        + 'never receives what the key claims it was asked about, and only reading each request '
        + 'separately catches a wiring gap on either side alone',
      fn: async () => {
        /**
         Run over the corroborated-reading fixture, with the reading handed
         in.
         */
        const { requests, } = await runDocument({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          translatorRendering: FRESH_RENDERING,
          pictureReadings: new Map<string, PairedReading>([
            [CORROBORATED_ASSET_NAME, CORROBORATED_READING,],
          ],),
        },);

        /**
         Every request a judge actually received this run.
         */
        const sent = judgeRequestsOf({ requests, },);
        expect(sent.length,).toBeGreaterThan(0,);
        expect(
          sent.some(function carriesReading(request,): boolean {
            return request.content.includes(CORROBORATED_READING_TEXT,);
          },),
        ).toBe(true,);
        // Ledger H7: the judge read the transcript with no scope rule, so a
        // candidate rendering a neighbouring picture's words looked faithful.
        expect(
          sent.some(function carriesScope(request,): boolean {
            return request.content.includes('it is not part of the ORIGINAL passage: a candidate rendering',);
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'OMITS the reading from BOTH sheets when `translateDocument` is driven with no '
        + '`pictureReadings` at all, and DIFFERS measurably from the run that was handed one: this is '
        + 'the POSITIVE CONTROL the first two cases need, since without it an assertion that always '
        + 'passes would look identical to one that actually exercises the picture channel',
      fn: async () => {
        /**
         Run over the corroborated-reading fixture, with the reading handed
         in, so this case has something to differ from.
         */
        const withReadings = await runDocument({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          translatorRendering: FRESH_RENDERING,
          pictureReadings: new Map<string, PairedReading>([
            [CORROBORATED_ASSET_NAME, CORROBORATED_READING,],
          ],),
        },);

        /**
         The identical document and rendering, but with no `pictureReadings`
         argument at all, exercising `translateDocument`'s own default.
         */
        const withoutReadings = await runDocument({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          translatorRendering: FRESH_RENDERING,
        },);

        /**
         Translator requests from the run handed a reading, which must carry
         it, or comparing `translatorWith` with `translatorWithout` would not
         be a control at all.
         */
        const translatorWith = translatorRequestsOf({ requests: withReadings.requests, },);

        /**
         Judge requests from the run handed a reading, for the same reason.
         */
        const judgeWith = judgeRequestsOf({ requests: withReadings.requests, },);

        /**
         Translator requests from the run handed no reading.
         */
        const translatorWithout = translatorRequestsOf({ requests: withoutReadings.requests, },);

        /**
         Judge requests from the run handed no reading.
         */
        const judgeWithout = judgeRequestsOf({ requests: withoutReadings.requests, },);

        expect(translatorWith.length,).toBeGreaterThan(0,);
        expect(judgeWith.length,).toBeGreaterThan(0,);
        expect(translatorWithout.length,).toBeGreaterThan(0,);
        expect(judgeWithout.length,).toBeGreaterThan(0,);

        expect(
          translatorWithout.some(function carriesReading(request,): boolean {
            return request.content.includes(CORROBORATED_READING_TEXT,);
          },),
        ).toBe(false,);
        expect(
          judgeWithout.some(function carriesReading(request,): boolean {
            return request.content.includes(CORROBORATED_READING_TEXT,);
          },),
        ).toBe(false,);

        // THE CONTROL ITSELF: every translator receives an identical sheet
        // within one run, since nothing in `buildTranslateMessages` varies by
        // model, so the first request of each run is representative of the
        // whole run and a plain inequality is enough to prove the two runs
        // were asked different questions.
        expect(
          translatorWith[0]
            ?.content,
        ).not.toBe(
          translatorWithout[0]
            ?.content,
        );
        expect(
          judgeWith[0]
            ?.content,
        ).not.toBe(
          judgeWithout[0]
            ?.content,
        );
      },
    },),

    it({
      name: 'RECORDS a finding naming the asset an `unavailable` reading covers on the slice record, '
        + 'and OMITS any mention of that asset from either sheet: an unread picture is evidence a person '
        + 'reads in findings, never a hedge a model has to weigh, which is `slice-pictures.ts`\'s own rule '
        + 'for exactly this case',
      fn: async () => {
        /**
         Run over a document whose one picture nobody could corroborate.
         */
        const { result, requests, } = await runDocument({
          sourceText: UNAVAILABLE_SOURCE_TEXT,
          targetText: UNAVAILABLE_TARGET_TEXT,
          translatorRendering: UNAVAILABLE_FRESH_RENDERING,
          pictureReadings: new Map<string, PairedReading>([
            [UNAVAILABLE_ASSET_NAME, UNAVAILABLE_READING,],
          ],),
        },);

        /**
         The one slice this document produces.
         */
        const [record,] = result.slices;
        expect(record,).toBeDefined();

        /**
         Findings this record carries, defaulted to empty so a missing
         record fails the `findings` assertion rather than throwing first.
         */
        const findings = record?.findings ?? [];
        expect(
          findings.some(function namesTheAsset(finding,): boolean {
            return finding.includes(UNAVAILABLE_ASSET_NAME,) && finding.includes('no reading',);
          },),
        ).toBe(true,);

        // NEITHER SHEET, checked across every exchange the run attempted
        // together: the WHAT THE PICTURES HERE SAY heading is the only channel a
        // picture reaches a sheet through, and `slicePictures` never writes it for a reading it
        // refused, so its absence here is the whole proof.
        expect(
          requests.some(function mentionsPictures(request,): boolean {
            return request.content.includes('WHAT THE PICTURES HERE SAY',);
          },),
        ).toBe(false,);
      },
    },),
  ],
},);
