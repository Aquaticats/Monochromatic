/**
 Tests for what happens to a translated slice that fails structural
 validation.

 By user decision of 2026-08-15 it is not dropped: it goes back to the model
 that wrote it, in the same exchange, and that model answers with a revision,
 an inability, or a defence of what it produced. Each of those three lands
 differently, and the third is the one no filter could have collected, so all
 three are pinned here.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { setImmediate as turnOfTheLoop, } from 'node:timers/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
  type VisionMessage,
} from '@monochromatic-dev/translation-repair-provider';
import {
  mapOverlapped,
  reuseTwinOrBuy,
  type TwinMemo,
} from '@monochromatic-dev/translation-repair-stage';
import {
  buildTranslateCandidates,
  FloorGroundDisagreementError,
  repairInvalidCandidates,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { untilAborted, } from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

/**
 Logger for the repairs under test.
 */
const l = tagged({ tag: 'translate-repair-test', },);

/**
 Original the candidates render: a heading and a paragraph.
 */
const SOURCE_TEXT = `## 猫猫的一天

它在窗台上打盹。`;

/**
 Translation matching that structure.
 */
const GOOD_TEXT = `## A Day in the Cat's Life

It dozes on the windowsill.`;

/**
 Translation that merged the heading away, which validation reports.
 */
const MERGED_TEXT = 'A day in the cat\'s life: it dozes on the windowsill.';

/**
 Page carrying the heading the merged translation lost, so validation
 against it reports the merge and a candidate the copy check does not stop
 is asked about. Against a page that merged the heading too the candidate is
 valid, and a copy-check case would pass without the check.
 */
const HEADED_PAGE = `## A Day in the Cat's Life\n\n${MERGED_TEXT} It naps.`;

/**
 Model whose candidate every case repairs.
 */
const TRANSLATOR = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Messages the candidate was produced by, stood in for since the repair turn
 only has to continue them.
 */
const PRIOR_MESSAGES: readonly ChatMessage[] = [
  {
    role: 'system',
    content: 'Render the ORIGINAL passage into English.',
  },
  {
    role: 'user',
    content: SOURCE_TEXT,
  },
];

/**
 What the follow-up call saw and answered.
 */
type RepairLog = {
  calls: number;
  messages: readonly (ChatMessage | VisionMessage)[];
};

/**
 Client answering the follow-up turn from a script.

 @param answer - repair reply it returns

 @param log - shared record the cases assert on

 @returns Client honoring that script

 @example
 ```ts
 const client = repairClient({ answer, log, },);
 ```
 */
function repairClient(
  {
    answer,
    log,
  }: {
    readonly answer: unknown;
    readonly log: RepairLog;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      log.calls += 1;
      log.messages = request.messages;
      if (!request.validate(answer,)) {
        return {
          kind: 'schema-mismatch',
          rawText: JSON.stringify(answer,),
          detail: 'scripted answer failed the repair guard',
        };
      }
      return {
        kind: 'ok',
        value: answer,
        rawText: JSON.stringify(answer,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Runs one candidate through validation and any follow-up it earns.

 @param translation - what the translator returned

 @param answer - what it answers when asked about the findings

 @param sourceText - original candidate renders

 @param incumbentText - translation already in the document, blank by default
 so most cases exercise a slice with none

 @param pageText - text the candidate replaces, left to the incumbent by
 default because that is what a translator replaces

 @param said - sink for every line the repair logs, for the cases that read
 the run log; the shared logger otherwise

 @param syntax - explicit syntax role of the slice, absent for ordinary
 Markdown

 @returns Final voices, findings, and what the follow-up call saw

 @example
 ```ts
 const { findings, } = await runRepair({ translation, answer, },);
 ```
 */
async function runRepair(
  {
    translation,
    answer,
    sourceText = SOURCE_TEXT,
    incumbentText = '',
    pageText = incumbentText,
    said,
    syntax,
  }: {
    readonly translation: string;
    readonly answer: unknown;
    readonly sourceText?: string;
    readonly incumbentText?: string;
    readonly pageText?: string;
    readonly said?: string[];
    readonly syntax?: 'front-matter';
  },
) {
  /**
   What the follow-up call received.
   */
  const log: RepairLog = {
    calls: 0,
    messages: [],
  };

  /**
   Outcome over one heard voice.
   */
  const repaired = await repairInvalidCandidates({
    client: repairClient({
      answer,
      log,
    },),
    voices: [
      {
        modelId: TRANSLATOR,
        value: { translation, },
      },
    ],
    sourceText,
    incumbentText,
    pageText,
    ...((syntax === undefined) ? {} : { syntax, }),
    priorMessages: PRIOR_MESSAGES,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l: (said === undefined) ? l : capturingLogger({ messages: said, },),
  },);
  return {
    repaired,
    log,
  };
}

await describe({
  name: repairInvalidCandidates.name,
  children: [
    it({
      name: 'asks NOBODY anything when the candidate matches its original, '
        + 'which is the ordinary case and has to cost nothing: a follow-up per '
        + 'candidate per slice would double the lane on work that was already '
        + 'right',
      fn: async () => {
        const { repaired, log, } = await runRepair({
          translation: GOOD_TEXT,
          answer: {
            resolution: 'revised',
            translation: 'unused',
            translatorNote: 'unused',
          },
        },);
        expect(log.calls,).toBe(0,);
        expect(repaired.findings,).toHaveLength(0,);
        expect(repaired.voices[0]?.value.translation,).toBe(GOOD_TEXT,);
      },
    },),

    it({
      name: 'REFUSES with the first voice\'s refusal when the caller aborted, two voices\' follow-ups are '
        + 'refused and the later voice is refused first',
      fn: async () => {
        /**
         Opened by the later voice's follow-up as it is refused, which the
         first voice's follow-up waits for.
         */
        const laterRefused = Promise.withResolvers<undefined>();

        /**
         Caller's abort, which makes a refused follow-up leave the repair
         rather than cost a voice.
         */
        const stop = new AbortController();
        stop.abort();

        /**
         What the repair refused with.
         */
        const refusal = await rejectionOf({
          promise: repairInvalidCandidates({
            client: {
              chatText: async () => {
                throw new Error('chatText unused',);
              },
              chatJson: async <ValueT,>(
                request: ChatJsonRequest<ValueT>,
              ): Promise<ChatJsonOutcome<ValueT>> => {
                if (request.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER) {
                  laterRefused.resolve(undefined,);
                  throw new Error('the later voice was refused',);
                }
                await laterRefused.promise;
                throw new Error('the first voice was refused',);
              },
              quotas: async () => {
                throw new Error('quotas unused',);
              },
            },
            voices: [
              {
                modelId: TRANSLATOR,
                value: { translation: MERGED_TEXT, },
              },
              {
                modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                value: { translation: MERGED_TEXT, },
              },
            ],
            sourceText: SOURCE_TEXT,
            incumbentText: '',
            pageText: HEADED_PAGE,
            priorMessages: PRIOR_MESSAGES,
            signal: stop.signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l,
          },),
        },);
        expect(String(refusal,),).toBe('Error: the first voice was refused',);
      },
    },),

    it({
      name: 'LOGS EACH RE-ASK\'S OWN FAILURE AS THE CALLER STOPS THE REPAIR, on a line naming its seat with the '
        + 'provider\'s words, the later-listed seat\'s first as it ends first, writes none for a re-ask ending in '
        + 'the caller\'s own reason, and refuses with the first failure in input order',
      fn: async () => {
        /**
         Every message the repair logged.
         */
        const said: string[] = [];
        /**
         Why the caller stopped the repair.
         */
        const reason = new Error('the owner called the cats in',);
        /**
         The caller's steering.
         */
        const steering = new AbortController();
        /**
         What the first-listed seat's re-ask ends in, once the later-listed seat's has ended.
         */
        const firstRefusal = new ProviderHttpError({
          provider: 'synthetic',
          status: 429,
          bodyText: 'the first cat is out of treats',
        },);
        /**
         What the later-listed seat's re-ask ends in, first.
         */
        const laterRefusal = new ProviderHttpError({
          provider: 'synthetic',
          status: 503,
          bodyText: 'the later cat is napping in the sun',
        },);
        /**
         The later-listed seat's failure at once, the first-listed seat's after it.
         */
        const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
        /**
         Seats whose candidates are sent back, in input order: the first-listed, the one whose re-ask ends in
         the caller's own reason, and the later-listed.
         */
        const seats = [
          TRANSLATOR,
          SEAT_SYNTHETIC_TEXT_EVERYWHERE,
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        ] as const;
        /**
         The repair, started and left waiting on every re-ask.
         */
        const repair = repairInvalidCandidates({
          client: {
            chatText: async () => {
              throw new Error('chatText unused',);
            },
            chatJson: async <ValueT,>(
              request: ChatJsonRequest<ValueT>,
            ): Promise<ChatJsonOutcome<ValueT>> => {
              await untilAborted({ signal: request.signal, },);
              if (request.modelId === TRANSLATOR)
                return await refuseAfterThat(firstRefusal,);
              if (request.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER)
                return await refuseAtOnce(laterRefusal,);
              request.signal.throwIfAborted();
              throw new Error('unreachable: a re-ask that waited for its abort was handed a signal that had not aborted',);
            },
            quotas: async () => {
              throw new Error('quotas unused',);
            },
          },
          voices: seats.map(function voice(modelId,) {
            return {
              modelId,
              value: { translation: MERGED_TEXT, },
            };
          },),
          sourceText: SOURCE_TEXT,
          incumbentText: '',
          pageText: HEADED_PAGE,
          priorMessages: PRIOR_MESSAGES,
          signal: steering.signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: capturingLogger({ messages: said, },),
        },);
        steering.abort(reason,);
        /**
         What the repair refused with.
         */
        const refusal = await rejectionOf({ promise: repair, },);
        // A re-ask ending after the one the repair refused on would write its
        // line as it ends. No scripted failure waits on anything but a
        // microtask, so a turn of the event loop later every re-ask has ended.
        await turnOfTheLoop();
        /**
         What the publication rule finds in each sent-back candidate.
         */
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: MERGED_TEXT,
          pageText: HEADED_PAGE,
        },);
        if (verdict.kind !== 'invalid')
          throw new Error(`the fixture's candidate must fail the publication rule, and the floor said ${verdict.kind}`,);
        expect(refusal,).toBe(firstRefusal,);
        expect(said,).toEqual([
          ...seats.map(function askedLine(seat,): string {
            return `[repairOneCandidate] ${seat}: candidate fails the publication rule, so its author is asked about `
              + `it: ${verdict.findings.join(' ',)}`;
          },),
          `[repairOneCandidate] translate-repair ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}: refused by `
            + 'ProviderHttpError from synthetic with HTTP 503 (the provider said: "the later cat is napping in the sun"), as the '
            + 'caller stopped the repair',
          `[repairOneCandidate] translate-repair ${TRANSLATOR}: refused by ProviderHttpError from synthetic with HTTP 429 (the `
            + 'provider said: "the first cat is out of treats"), as the caller stopped the repair',
        ],);
      },
    },),

    it({
      name: 'WRITES A RE-ASK\'S OWN FAILURE ONCE, on its seat\'s line, when the repair runs as a buy under the twin '
        + 'memo inside the overlapped map and the caller stops it: no abandoned-buy line and no line of the map\'s',
      fn: async () => {
        /**
         Every message the repair, the memo and the map logged, in one sink.
         */
        const said: string[] = [];
        /**
         Logger all three write through.
         */
        const sink = capturingLogger({ messages: said, },);
        /**
         Why the caller stopped the work.
         */
        const reason = new Error('the owner called the cats in',);
        /**
         The caller's steering.
         */
        const steering = new AbortController();
        /**
         What the seat's re-ask ends in, of its own, once the stop has landed.
         */
        const refusal = new ProviderHttpError({
          provider: 'synthetic',
          status: 503,
          bodyText: 'the cat is napping in the sun',
        },);
        /**
         Buys of this run by question, as a consolidation keeps them.
         */
        const memo: TwinMemo<string> = new Map();
        /**
         Two slices: the first ends in the caller's own reason, and the second
         buys a repair whose re-ask ends in a failure of its own.
         */
        const work = rejectionOf({
          promise: mapOverlapped({
            items: [
              'the windowsill',
              'the sofa',
            ],
            overlap: 2,
            signal: steering.signal,
            l: sink,
            nameOf: function sliceName({ item, },): string {
              return item;
            },
            oneItem: async function settleOne({ item, position, },): Promise<string> {
              if (position === 0) {
                await untilAborted({ signal: steering.signal, },);
                steering.signal.throwIfAborted();
                throw new Error('unreachable: a slice that waited for its abort was handed a signal that had not aborted',);
              }
              /**
               The slice's buy under the memo, which runs the repair.
               */
              const asked = await reuseTwinOrBuy({
                key: item,
                memo,
                buy: async function buyTheRepair(): Promise<string> {
                  /**
                   What the repair settled, which this case never reaches.
                   */
                  const repaired = await repairInvalidCandidates({
                    client: {
                      chatText: async () => {
                        throw new Error('chatText unused',);
                      },
                      chatJson: async <ValueT,>(
                        request: ChatJsonRequest<ValueT>,
                      ): Promise<ChatJsonOutcome<ValueT>> => {
                        await untilAborted({ signal: request.signal, },);
                        throw refusal;
                      },
                      quotas: async () => {
                        throw new Error('quotas unused',);
                      },
                    },
                    voices: [
                      {
                        modelId: TRANSLATOR,
                        value: { translation: MERGED_TEXT, },
                      },
                    ],
                    sourceText: SOURCE_TEXT,
                    incumbentText: '',
                    pageText: HEADED_PAGE,
                    priorMessages: PRIOR_MESSAGES,
                    signal: steering.signal,
                    perCallTimeoutMs: HANG_STOP_MS,
                    l: sink,
                  },);
                  return repaired.findings.join(' ',);
                },
                persistedOf: function storedNothing() {
                  return { kind: 'nothing', } as const;
                },
                l: sink,
                signal: steering.signal,
              },);
              return (asked.kind === 'reused') ? asked.twin : asked.bought;
            },
          },),
        },);
        steering.abort(reason,);
        expect(await work,).toBe(reason,);
        /**
         What the publication rule finds in the sent-back candidate.
         */
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: MERGED_TEXT,
          pageText: HEADED_PAGE,
        },);
        if (verdict.kind !== 'invalid')
          throw new Error(`the fixture's candidate must fail the publication rule, and the floor said ${verdict.kind}`,);
        expect(said,).toEqual([
          `[repairOneCandidate] ${TRANSLATOR}: candidate fails the publication rule, so its author is asked about `
            + `it: ${verdict.findings.join(' ',)}`,
          `[repairOneCandidate] translate-repair ${TRANSLATOR}: refused by ProviderHttpError from synthetic with HTTP 503 (the `
            + 'provider said: "the cat is napping in the sun"), as the caller stopped the repair',
        ],);
      },
    },),

    it({
      name: 'REPAIRS contributor respelling back to target-authoritative form',
      fn: async () => {
        /** Target-authoritative archive attribution. */
        const archive = 'Contributors for this entry: [Snow](https://example.test/snow)';
        const { repaired, log, } = await runRepair({
          sourceText: '本条目贡献者：雪猫',
          incumbentText: archive,
          translation: 'Contributors for this entry: Snowflake',
          answer: {
            resolution: 'revised',
            translation: archive,
            translatorNote: 'restored target contributor form',
          },
        },);
        expect(log.calls,).toBe(1,);
        expect(repaired.voices[0]?.value.translation,).toBe(archive,);
      },
    },),

    it({
      name: 'FLOORS DEFENDED contributor respelling to target-authoritative page',
      fn: async () => {
        /** Target-authoritative archive attribution. */
        const archive = 'Contributors for this entry: [Snow](https://example.test/snow)';
        const { repaired, } = await runRepair({
          sourceText: '本条目贡献者：雪猫',
          incumbentText: archive,
          translation: 'Contributors for this entry: Snowflake',
          answer: {
            resolution: 'as-intended',
            translation: '',
            translatorNote: 'literal spelling is deliberate',
          },
        },);
        expect(repaired.voices,).toHaveLength(0,);
      },
    },),

    it({
      name: 'EXCLUDES contributor violation when follow-up is unheard',
      fn: async () => {
        /** Target-authoritative archive attribution. */
        const archive = 'Contributors for this entry: [Snow](https://example.test/snow)';
        const { repaired, } = await runRepair({
          sourceText: '本条目贡献者：雪猫',
          incumbentText: archive,
          translation: 'Contributors for this entry: Snowflake',
          answer: { nonsense: true, },
        },);
        expect(repaired.voices,).toHaveLength(0,);
        expect(repaired.findings,).toContain(`translate-repair-unheard (${TRANSLATOR})`,);
      },
    },),

    it({
      name: 'BUILDS INCUMBENT-ONLY SLATE with named findings when all voices violate authority',
      fn: async () => {
        /** Target-authoritative archive attribution. */
        const archive = 'Contributors for this entry: [Snow](https://example.test/snow)';
        /** Shared call log for both rejected voices. */
        const log: RepairLog = { calls: 0, messages: [], };
        const modelIds = [
          TRANSLATOR,
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        ] as const;
        const repaired = await repairInvalidCandidates({
          client: repairClient({
            answer: {
              resolution: 'as-intended',
              translation: '',
              translatorNote: 'literal spelling is deliberate',
            },
            log,
          },),
          voices: modelIds.map(function voice(modelId,) {
            return {
              modelId,
              value: { translation: 'Contributors for this entry: Snowflake', },
            };
          },),
          sourceText: '本条目贡献者：雪猫',
          incumbentText: archive,
          priorMessages: PRIOR_MESSAGES,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
        },);
        const built = buildTranslateCandidates({
          voices: repaired.voices,
          translatorModelIds: modelIds,
          incumbentText: archive,
          lineStructured: false,
        },);
        expect(repaired.voices,).toHaveLength(0,);
        expect(repaired.findings,).toHaveLength(modelIds.length * 2,);
        expect(built.candidates,).toHaveLength(1,);
        expect(built.candidates[0]?.value.text,).toBe(archive,);
      },
    },),

    it({
      name: 'EXCLUDES contributor violation when revision remains invalid',
      fn: async () => {
        /** Target-authoritative archive attribution. */
        const archive = 'Contributors for this entry: [Snow](https://example.test/snow)';
        const { repaired, } = await runRepair({
          sourceText: '本条目贡献者：雪猫',
          incumbentText: archive,
          translation: 'Contributors for this entry: Snowflake',
          answer: {
            resolution: 'revised',
            translation: 'Contributors for this entry: Snow Cat',
            translatorNote: 'changed wording but not target form',
          },
        },);
        expect(repaired.voices,).toHaveLength(0,);
        expect(repaired.findings.some(function unresolved(finding,): boolean {
          return finding.startsWith(`translate-repair-unresolved (${TRANSLATOR})`,);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'leaves a candidate that REPRODUCED THE INCUMBENT alone, however '
        + 'the incumbent scores. That text is about to collapse into the '
        + 'incumbent, which is never validated, so asking about it would spend '
        + 'a call on text nobody will ship; and a revision would break the '
        + 'match, erasing the one signal that says a translation was examined '
        + 'and kept rather than never looked at',
      fn: async () => {
        const { repaired, log, } = await runRepair({
          translation: MERGED_TEXT,
          incumbentText: MERGED_TEXT,
          pageText: HEADED_PAGE,
          answer: {
            resolution: 'revised',
            translation: GOOD_TEXT,
            translatorNote: 'restored the heading',
          },
        },);
        expect(log.calls,).toBe(0,);
        expect(repaired.findings,).toHaveLength(0,);
        expect(repaired.voices[0]?.value.translation,).toBe(MERGED_TEXT,);
      },
    },),

    it({
      name: 'leaves alone a candidate that is the incumbent with its SOFT LINE BREAK ELSEWHERE, which the site '
        + 'renders as the same page and the slate collapses into the incumbent the same way (ledger B26)',
      fn: async () => {
        const { repaired, log, } = await runRepair({
          translation: MERGED_TEXT,
          incumbentText: 'A day in the cat\'s life:\nit dozes on the windowsill.',
          pageText: HEADED_PAGE,
          answer: {
            resolution: 'revised',
            translation: GOOD_TEXT,
            translatorNote: 'restored the heading',
          },
        },);
        expect(log.calls,).toBe(0,);
        expect(repaired.voices[0]?.value.translation,).toBe(MERGED_TEXT,);
      },
    },),

    it({
      name: 'still asks about a candidate that only RESEMBLES the incumbent, '
        + 'since the collapse it would merge into never happens and the '
        + 'candidate reaches the ballot on its own',
      fn: async () => {
        const { log, } = await runRepair({
          translation: MERGED_TEXT,
          // CARRIES THE HEADING the candidate merged away, so the page the
          // candidate is checked against still reports it. An incumbent that
          // merged it too would make this candidate valid, and the case would
          // stop testing the collapse rule it was written for.
          incumbentText: HEADED_PAGE,
          answer: {
            resolution: 'unable',
            translation: '',
            translatorNote: 'the heading is not a sentence in English',
          },
        },);
        expect(log.calls,).toBe(1,);
      },
    },),

    it({
      name: 'continues the SAME exchange, carrying the model\'s own turn and '
        + 'then the findings. Asking in a fresh conversation would ask a model '
        + 'about text it cannot see, which is a different question with a '
        + 'worse answer',
      fn: async () => {
        const { log, } = await runRepair({
          translation: MERGED_TEXT,
          answer: {
            resolution: 'unable',
            translation: '',
            translatorNote: 'the heading is not a sentence in English',
          },
        },);
        expect(log.calls,).toBe(1,);
        expect(log.messages,).toHaveLength(PRIOR_MESSAGES.length + 2,);
        expect(log.messages[PRIOR_MESSAGES.length]?.role,).toBe('assistant',);
        /**
         The merged assistant turn, whose text carries the candidate.
         */
        const merged = log.messages[PRIOR_MESSAGES.length];
        if (merged === undefined)
          throw new Error('a merged turn by construction',);
        expect(messageText({ message: merged, },),).toContain(MERGED_TEXT,);
        /**
         The final turn, whose text names the structure being asked about.
         */
        const asked = log.messages.at(-1,);
        if (asked === undefined)
          throw new Error('a final turn by construction',);
        expect(messageText({ message: asked, },),).toContain('heading (level 2)',);
      },
    },),

    it({
      name: 'KEEPS contributor-safe original when revision introduces respelling',
      fn: async () => {
        /** Source carrying heading plus attribution. */
        const source = '## 贡献者\n\n本条目贡献者：雪猫';
        /** Target-authoritative page shape and identity. */
        const archive = '## Contributors\n\nContributors for this entry: [Snow](https://example.test/snow)';
        /** Structurally invalid original candidate retaining contributor authority. */
        const original = 'Contributors for this entry: [Snow](https://example.test/snow)';
        const { repaired, } = await runRepair({
          sourceText: source,
          incumbentText: archive,
          translation: original,
          answer: {
            resolution: 'revised',
            translation: '## Contributors\n\nContributors for this entry: Snowflake',
            translatorNote: 'restored heading but changed attribution',
          },
        },);
        expect(repaired.voices[0]?.value.translation,).toBe(original,);
      },
    },),

    it({
      name: 'TAKES a revision that resolves the findings, which is the whole '
        + 'point of asking rather than dropping: the candidate reaches the '
        + 'judges as the model meant it',
      fn: async () => {
        const { repaired, } = await runRepair({
          translation: MERGED_TEXT,
          answer: {
            resolution: 'revised',
            translation: GOOD_TEXT,
            translatorNote: 'restored the heading',
          },
        },);
        expect(repaired.voices[0]?.value.translation,).toBe(GOOD_TEXT,);
        expect(repaired.findings,).toContain(`translate-repair-revised (${TRANSLATOR})`,);
      },
    },),

    it({
      name: 'REFUSES a revision that still fails, keeping the original. The '
        + 'model was asked to fix these findings and did not, so nothing says '
        + 'the new text is better, while the original is at least what it '
        + 'produced with the whole sheet in front of it',
      fn: async () => {
        const { repaired, } = await runRepair({
          translation: MERGED_TEXT,
          answer: {
            resolution: 'revised',
            translation: 'Still one paragraph, still no heading.',
            translatorNote: 'tried',
          },
        },);
        expect(repaired.voices[0]?.value.translation,).toBe(MERGED_TEXT,);
        expect(
          repaired.findings.some(function isUnresolved(finding,) {
            return finding.startsWith(`translate-repair-unresolved (${TRANSLATOR})`,);
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'RECHECKS A FRONT-MATTER REVISION AS FRONT MATTER: a revision that changes the YAML fields is refused, '
        + 'though read as Markdown it would pass',
      fn: async () => {
        /**
         Candidate carrying text below its front matter block.
         */
        const translation = '---\nname: Maomao\n---\n\nThe cat.';
        const { repaired, } = await runRepair({
          sourceText: '---\nname: 猫猫\n---\n',
          translation,
          syntax: 'front-matter',
          answer: {
            resolution: 'revised',
            translation: '---\nname: Maomao\nextra: The cat.\n---\n',
            translatorNote: 'moved the line into the block',
          },
        },);
        expect(repaired,).toEqual({
          voices: [{
            modelId: TRANSLATOR,
            value: { translation, },
          },],
          findings: [
            `translate-invalid (${TRANSLATOR}): Your translation added text outside YAML front matter block.`,
            `translate-repair-unresolved (${TRANSLATOR}): moved the line into the block`,
          ],
        },);
      },
    },),

    it({
      name: 'PASSES A CANDIDATE AGAINST A PAGE ONLY PLAIN MARKDOWN READS, asking nobody, and records the weaker '
        + 'reading',
      fn: async () => {
        const { repaired, log, } = await runRepair({
          translation: GOOD_TEXT,
          pageText: '## A Day in the Cat\'s Life\n\nIt dozes {on the windowsill.',
          answer: {
            resolution: 'revised',
            translation: 'unused',
            translatorNote: 'unused',
          },
        },);
        expect({
          repaired,
          calls: log.calls,
        },).toEqual({
          repaired: {
            voices: [{
              modelId: TRANSLATOR,
              value: { translation: GOOD_TEXT, },
            },],
            findings: ['translate-page-downgraded (page read as plain markdown; strict MDX refused it)',],
          },
          calls: 0,
        },);
      },
    },),

    it({
      name: 'RAISES THE DISAGREEMENT where no grammar reads the original, asking nobody: both stages that take '
        + 'this turn settle such a slice before any writer is asked, and this turn used to let a candidate stand '
        + 'there unvalidated (ledger B43, B45)',
      fn: async () => {
        /**
         Original carrying a tag the strict grammar never closes.
         */
        const sourceText = '小橘子 <未闭合 的标签 在这里。';
        /**
         What the follow-up call received.
         */
        const log: RepairLog = {
          calls: 0,
          messages: [],
        };
        /**
         What the turn raised.
         */
        let raised: unknown;
        try {
          await repairInvalidCandidates({
            client: repairClient({
              answer: {
                resolution: 'revised',
                translation: 'unused',
                translatorNote: 'unused',
              },
              log,
            },),
            voices: [
              {
                modelId: TRANSLATOR,
                value: { translation: 'Little Orange is here.', },
              },
            ],
            sourceText,
            incumbentText: '',
            priorMessages: PRIOR_MESSAGES,
            signal: new AbortController().signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l,
          },);
        }
        catch (error) {
          raised = error;
        }
        if (!(raised instanceof FloorGroundDisagreementError))
          throw new Error('expected the turn to raise the disagreement',);

        /**
         The floor's own account of the slice, which the disagreement carries.
         */
        const verdict = validateTranslatedSlice({
          sourceText,
          candidateText: 'Little Orange is here.',
        },);
        if (verdict.kind !== 'unknown')
          throw new Error(`the fixture's original must be one no grammar reads, and the floor said ${verdict.kind}`,);
        expect({
          detail: raised.detail,
          calls: log.calls,
        },).toEqual({
          detail: verdict.detail,
          calls: 0,
        },);
      },
    },),

    it({
      name: 'KEEPS the candidate and records the reason when the model says '
        + 'the finding is about the passage rather than its work. This is the '
        + 'answer no filter could collect: a marker whose definition is not in '
        + 'this slice is a report about the SLICING, and dropping the '
        + 'candidate would have destroyed the only copy of it',
      fn: async () => {
        const { repaired, } = await runRepair({
          translation: MERGED_TEXT,
          answer: {
            resolution: 'as-intended',
            translation: '',
            translatorNote: 'the original heading is a caption, not a section title',
          },
        },);
        expect(repaired.voices[0]?.value.translation,).toBe(MERGED_TEXT,);
        expect(
          repaired.findings.some(function isAsIntended(finding,) {
            return finding.includes('translate-repair-as-intended',)
              && finding.includes('caption',);
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'keeps the candidate when the follow-up itself is lost, and says '
        + 'so, since a silent stage reads identically to one that found '
        + 'nothing to ask about',
      fn: async () => {
        const { repaired, } = await runRepair({
          translation: MERGED_TEXT,
          answer: { nonsense: true, },
        },);
        expect(repaired.voices[0]?.value.translation,).toBe(MERGED_TEXT,);
        expect(repaired.findings,).toContain(`translate-repair-unheard (${TRANSLATOR})`,);
      },
    },),

    it({
      name: 'CHECKS a candidate against the page rather than the incumbent '
        + 'where the two differ, which is what a consolidator faces: its '
        + 'incumbent is the lane that won and the page it replaces is the '
        + 'archive',
      fn: async () => {
        const { log, repaired, } = await runRepair({
          translation: MERGED_TEXT,
          // The lane that won, which merged the heading away as well, so a
          // candidate checked against IT would be called valid.
          incumbentText: `${MERGED_TEXT} It naps.`,
          // The archive, which carries the heading.
          pageText: GOOD_TEXT,
          answer: {
            resolution: 'unable',
            translation: '',
            translatorNote: 'the heading is not a sentence in English',
          },
        },);
        expect(log.calls,).toBe(1,);
        expect(repaired.findings.join('\n',),).toContain('PAGE AS IT STANDS',);
      },
    },),

    it({
      name: 'SAYS IN THE RUN LOG what the rule found and how the author answered, on every branch '
        + '(ledger E5): only a taken revision was logged, so a candidate sent back and then kept, '
        + 'defended or lost left the log with nothing between its gather line and the judges',
      fn: async () => {
        /**
         Each follow-up answer beside the line its branch must leave.
         */
        const branches = [
          {
            answer: { resolution: 'unable', translation: '', translatorNote: 'the cat has no heading', },
            line: `${TRANSLATOR} answered unable`,
          },
          {
            answer: { resolution: 'as-intended', translation: '', translatorNote: 'the cat merged it', },
            line: `${TRANSLATOR} answered as-intended`,
          },
          {
            answer: { resolution: 'revised', translation: MERGED_TEXT, translatorNote: 'the cat tried', },
            line: `${TRANSLATOR}: revision still fails the publication rule`,
          },
          {
            answer: { nonsense: true, },
            line: `${TRANSLATOR} gave no usable answer to its findings`,
          },
        ] as const;
        /**
         Lines each branch logged, beside the line it owes.
         */
        const logged = await Promise.all(branches.map(async function logOf({ answer, line, },) {
          /**
           Lines this branch's repair wrote.
           */
          const said: string[] = [];
          await runRepair({ translation: MERGED_TEXT, answer, said, },);
          return { line, said, };
        },),);
        for (const { line, said, } of logged) {
          expect(said.some(function sent(entry,) {
            return entry.includes(`${TRANSLATOR}: candidate fails the publication rule`,);
          },),).toBe(true,);
          expect(said.some(function answered(entry,) {
            return entry.includes(line,);
          },),).toBe(true,);
        }
      },
    },),
  ],
},);
