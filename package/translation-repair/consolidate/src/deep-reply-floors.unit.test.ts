/**
 Tests that a reply nested too deeply to read is refused by a floor before any
 parser reads it twice, in every stage a model's text enters that already has
 one, and that the voice's refusal is the stage's record of it.

 THE SHAPE UNDER TEST. A stage that reads a model's text through a parse with
 no catch ends, with that voice's work and every other voice's, on the plain
 grammar's refusal. These stages never reach such a parse with the deep text:
 the refine rewrite is read by its paragraph gate, the polish round by the
 same gate, the consolidation slate by the shape floor, the archive block
 revision by its shape check, and the archive text itself by the preparation
 that opens an entry. Each case has one voice write a reply 300 levels deep
 and the others write ordinary ones, and names the floor that refused it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runRefineStage, } from '@monochromatic-dev/translation-repair-repair';
import { runArchiveBlockReviewStage, } from '@monochromatic-dev/translation-repair-archive';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { MarkdownParseError, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  deriveRefinableEnvelopes,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  polishConsolidation,
  produceConsolidations,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { sortedByModelId, } from '@monochromatic-dev/translation-repair-markup/ts/model-order.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Deep reply floor tests

/**
 The voice that writes the deep reply in every case.
 */
const DEEP_VOICE = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 The voice that writes an ordinary reply where one case seats two.
 */
const ORDINARY_VOICE = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Three hundred nested list markers, past the bound of 256.
 */
const DEEP_MARKERS = '- '.repeat(300,);

/**
 What the polish and refine cases render: a slice the rewrite improves.
 */
const BASE_TEXT = 'The cat is doing the sunbathing on the windowsill in every afternoon, and when the light is moving '
  + 'across the floor she is following it without any hurry at all.';

/**
 A rewrite that reads and keeps every atom the base carries.
 */
const SMOOTH_TEXT = 'The cat sunbathes on the windowsill every afternoon, and when the light moves across the floor she '
  + 'follows it without hurry.';

/**
 Original the rewrite is checked against.
 */
const SOURCE_TEXT = '猫猫每天下午都在窗台上晒太阳，光移动的时候她也跟着移动。';

/**
 Wire replies a client gives, by the stage and the voice asked.
 */
type Answer = (input: { readonly stage: string; readonly modelId: string; },) => unknown;

/**
 Client answering each exchange from a function of its stage and voice.

 @param answer - wire reply for a stage and voice

 @returns Client honoring the script

 @example
 ```ts
 const client = answeringClient({ answer: function reply(): unknown { return {}; }, },);
 ```
 */
function answeringClient({ answer, }: { readonly answer: Answer; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by these stages',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage the request belongs to.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name
        ?? '';
      /**
       Reply scripted for this stage and voice.
       */
      const value = answer({ stage, modelId: request.modelId, },);
      if (!request.validate(value,))
        throw new Error(`the script failed the ${stage} guard`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by these stages',);
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runRefineStage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS the ordinary rewriter\'s rewrite when another wrote 300 nested list markers, which the '
            + 'paragraph gate refused as unparseable before any other reading',
          fn: async () => {
            /**
             Lines the stage logged.
             */
            const messages: string[] = [];
            /**
             The slice's one eligible paragraph and the definitions around it.
             */
            const slice = deriveRefinableEnvelopes({ document: parseDocument({ text: BASE_TEXT, },), },);
            const result = await runRefineStage({
              client: answeringClient({
                answer: function reply({ stage, modelId, },): unknown {
                  if (stage === 'refine_report') {
                    return {
                      rewrites: [{
                        paragraph: 1,
                        newText: (modelId === DEEP_VOICE) ? `${DEEP_MARKERS}${SMOOTH_TEXT}` : SMOOTH_TEXT,
                      },],
                    };
                  }
                  return { best: 1, assessment: 'scripted', };
                },
              },),
              refinerModelIds: [DEEP_VOICE, ORDINARY_VOICE,],
              judgeModelIds: [
                DEEP_VOICE,
                ORDINARY_VOICE,
                SEAT_SYNTHETIC_VISION_WITHHELD,
                SEAT_HYPER_OPENROUTER_UNMEASURED,
              ],
              sourceText: SOURCE_TEXT,
              repairedText: BASE_TEXT,
              envelopes: slice.envelopes,
              definitions: slice.definitions,
              declaredNames: [],
              mode: { kind: 'comparative', },
              sliceIndex: 1,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l: capturingLogger({ messages, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(result.changed,).toBe(true,);
            expect(result.refinedText,).toBe(SMOOTH_TEXT,);
            expect(result.findings
              .filter(function isRefusal(finding,): boolean {
                return finding.startsWith(DEEP_VOICE,);
              },),).toEqual([
                `${DEEP_VOICE}: refine-atom-gate-refused (paragraph 1, candidate rejected)`,
              ],);
            expect(messages
              .filter(function namesTheDeepVoice(line,): boolean {
                return line.includes(`${DEEP_VOICE}: candidate rejected`,);
              },)
              .length,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: polishConsolidation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS the base wording when the one rewriter wrote 300 nested list markers, which the same paragraph '
            + 'gate refused, and still settles with the review recorded',
          fn: async () => {
            const polish = await polishConsolidation({
              client: answeringClient({
                answer: function reply({ stage, },): unknown {
                  if (stage === 'refine_report')
                    return { rewrites: [{ paragraph: 1, newText: `${DEEP_MARKERS}${SMOOTH_TEXT}`, },], };
                  if (stage === 'absolute_naturalness_review')
                    return { isAcceptable: true, findings: [], assessment: 'whole passage is publication-ready', };
                  return { best: 1, assessment: 'scripted', };
                },
              },),
              sourceText: SOURCE_TEXT,
              archiveText: BASE_TEXT,
              baseText: BASE_TEXT,
              lineStructured: false,
              sliceIndex: 1,
              config: {
                refinerModelIds: [DEEP_VOICE,],
                judgeModelIds: [DEEP_VOICE, ORDINARY_VOICE, SEAT_SYNTHETIC_VISION_WITHHELD,],
                gateModelIds: [DEEP_VOICE, ORDINARY_VOICE, SEAT_SYNTHETIC_VISION_WITHHELD,],
                declaredNames: [],
                definitions: '',
              },
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l: tagged({ tag: 'deep-reply-floors-test', },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(polish.kind,).toBe('settled',);
            if (polish.kind !== 'settled')
              throw new Error('the polish did not settle',);
            expect(polish.changed,).toBe(false,);
            expect(polish.text,).toBe(BASE_TEXT,);
            expect(polish.findings,).toEqual([
              `${DEEP_VOICE}: refine-atom-gate-refused (paragraph 1, candidate rejected)`,
              'refine-candidates (1/1 heard, 0 proposing)',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: produceConsolidations.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS the deep voice\'s slate entry as refused for the grammar and the ordinary voice\'s as '
            + 'valid, when one wrote 300 nested list markers and was asked again',
          fn: async () => {
            /**
             Two-line page, the structural standard the slice is held to.
             */
            const page = 'The cat sleeps.\nThe cat wakes at dusk.';
            const produced = await produceConsolidations({
              client: answeringClient({
                answer: function reply({ modelId, },): unknown {
                  return {
                    translation: (modelId === DEEP_VOICE)
                      ? `${DEEP_MARKERS}The cat sleeps.\nThe cat wakes at dusk.`
                      : 'The cat naps.\nThe cat wakes at dusk.',
                  };
                },
              },),
              roster: [DEEP_VOICE, ORDINARY_VOICE,],
              subject: {
                sourceText: '猫猫睡觉。\n猫猫黄昏醒来。',
                incumbentText: page,
                repairText: page,
                translateText: 'The cat naps.\nThe cat wakes at dusk.',
                ballots: [],
                lineStructured: true,
              },
              standingText: page,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l: tagged({ tag: 'deep-reply-floors-test', },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            // The slate lists the voices in the order the prompt-rotated bench answered (it moved when the house tense rule
            // changed the prompt), so both the slate and the expectation are read in model-id order. The whole list is compared,
            // so an extra entry or a duplicate of a voice fails (n166, Z3).
            expect(sortedByModelId({ items: produced.validity, },)
              .map(function kindOf(entry,): string {
                return `${entry.modelId}: ${entry.validation.kind}`;
              },),).toEqual(
              sortedByModelId({
                items: [
                  { modelId: DEEP_VOICE, kind: 'invalid', },
                  { modelId: ORDINARY_VOICE, kind: 'valid', },
                ],
              },)
                .map(function expectedOf(entry,): string {
                  return `${entry.modelId}: ${entry.kind}`;
                },),
              );
            expect(produced.findings
              .filter(function isRefusal(finding,): boolean {
                return finding.startsWith('translate-invalid',);
              },),).toEqual([
                `translate-invalid (${DEEP_VOICE}): Your translation could not be parsed as Markdown: MdxParseError: `
                  + 'MDX body refused to parse because it is nested too deeply to read: its container markers pass '
                  + 'the bound of 256 at line 1, column 513; corpus documents compile as MDX upstream, so failure '
                  + 'signals corruption or an unsupported construct.',
              ],);
          },
        },),
      ],
    },),

    describe({
      name: runArchiveBlockReviewStage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WITHHOLDS the deep reviewer\'s revision of 300 nested list markers as one that does not parse, and '
            + 'ships the ordinary reviewers\' revision',
          fn: async () => {
            /**
             Archive block under review, with a claim the source does not carry.
             */
            const block = 'The cat sleeps by the window and won an award.';
            const outcome = await runArchiveBlockReviewStage({
              client: answeringClient({
                answer: function reply({ stage, modelId, },): unknown {
                  if (stage === 'archive_block_review') {
                    return {
                      disposition: 'revise',
                      sourceQuote: '',
                      replacementText: (modelId === DEEP_VOICE)
                        ? `${DEEP_MARKERS}The cat sleeps by the window.`
                        : 'The cat sleeps by the window.',
                      assessment: 'Remove unsupported award claim.',
                    };
                  }
                  return { best: 1, assessment: 'Only supported details remain.', };
                },
              },),
              modelIds: [
                DEEP_VOICE,
                ORDINARY_VOICE,
                SEAT_SYNTHETIC_VISION_WITHHELD,
                SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              ],
              sourceText: '猫在窗边睡觉。',
              targetText: block,
              blockText: block,
              blockOffset: 0,
              priorFindings: [],
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l: tagged({ tag: 'deep-reply-floors-test', },),
              // The deep voice must be asked, and the default window is rotated by the sheet text.
              fanOut: 'whole-bench',
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(outcome.kind,).toBe('revised',);
            expect(outcome.text,).toBe('The cat sleeps by the window.',);
            expect(outcome.findings
              .filter(function isRefusal(finding,): boolean {
                return finding.startsWith('archive-revision-refused',);
              },),).toEqual([
                `archive-revision-refused (${DEEP_VOICE}): the revision does not parse (MdxParseError: MDX body `
                  + 'refused to parse because it is nested too deeply to read: its container markers pass the bound '
                  + 'of 256 at line 1, column 513; corpus documents compile as MDX upstream, so failure signals '
                  + 'corruption or an unsupported construct.); the block is 1 block (paragraph)',
              ],);
          },
        },),
      ],
    },),

    describe({
      name: prepareDocumentPair.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES an archive nested 300 list markers deep with the plain grammar\'s refusal before any '
            + 'model is asked, since the archive\'s own bytes are the one text no stage may write over',
          fn: async () => {
            const refusal = caught(function act(): unknown {
              return prepareDocumentPair({
                sourceText: '小猫打盹。\n',
                targetText: `${DEEP_MARKERS}The cat naps.\n`,
              },);
            },);

            expect(refusal,).toBeInstanceOf(MarkdownParseError,);
            expect(String(refusal,),).toBe(
              'MarkdownParseError: Plain markdown body refused to parse because it is nested too deeply to read: '
                + 'its container markers pass the bound of 256 at line 1, column 513.',
            );
          },
        },),
      ],
    },),
  ],
},);

//endregion Deep reply floor tests
