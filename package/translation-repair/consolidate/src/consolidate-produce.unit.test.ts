/**
 Tests for the producing half of the consolidation.

 Three branches, each asserted on its own rather than through the driver:
 every proposal valid, so the repair round changes nothing; a proposal
 refused for shape, so the verdicts before and after the repair diverge; and
 the findings of the gather ahead of the repair's.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createSyntheticClient,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type ConsolidateSubject,
  produceConsolidations,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the producers under test.
 */
const l = tagged({ tag: 'consolidate-produce-test', },);

/**
 One producer.
 */
const ROSTER = [SEAT_HYPER_OPENROUTER_VISION_EDITOR,] as const;

/**
 Two-line page, the structural standard a line-structured slice is held to.
 */
const PAGE_TEXT = 'The cat sleeps.\nThe cat wakes at dusk.';

/**
 Slice as the producers see it: line-structured, so a one-line proposal is
 refused for shape and sent back to its author.
 */
const SUBJECT: ConsolidateSubject = {
  sourceText: '猫猫睡觉。\n猫猫黄昏醒来。',
  incumbentText: PAGE_TEXT,
  repairText: PAGE_TEXT,
  translateText: 'The cat naps.\nThe cat wakes at dusk.',
  ballots: [],
  lineStructured: true,
};

/**
 Client answering each producer call with the next scripted translation.

 @param answers - translations, one per call in order; the last repeats

 @returns Client plus the count of calls made

 @example
 ```ts
 const rig = scriptedProducer({ answers: ['The cat sleeps.\nThe cat wakes at dusk.',], },);
 ```
 */
function scriptedProducer(
  { answers, }: { readonly answers: readonly string[]; },
): {
  readonly client: ModelClient;
  readonly calls: { count: number; };
} {
  /**
   Calls served so far.
   */
  const calls = { count: 0, };
  return {
    calls,
    client: createSyntheticClient({
      apiKey: 'test-key',
      transport: async function answerInOrder(exchange,) {
        /**
         Translation this call gets.
         */
        const translation = answers[Math.min(calls.count, answers.length - 1,)] ?? '';
        calls.count += 1;

        /**
         Reply in the shape the call asked for: a repair round asks its own
         report, with a resolution beside the revised text.
         */
        const reply = (exchange.bodyJson ?? '').includes('translation_repair_report',)
          ? {
            resolution: 'revised',
            translation,
            translatorNote: 'the line break is back',
          }
          : { translation, };
        return {
          status: 200,
          bodyText: `data: ${
            JSON.stringify({
              choices: [
                {
                  index: 0,
                  delta: { content: JSON.stringify(reply,), },
                },
              ],
            },)
          }\n\ndata: [DONE]\n\n`,
        };
      },
    },),
  };
}

/**
 Produces the fixture slate under one script.

 @param answers - translations per call

 @param subject - consolidation evidence,
 defaulting to line-structure fixture

 @returns Slate plus the calls it cost

 @example
 ```ts
 const { produced, } = await producedUnder({ answers: [PAGE_TEXT,], },);
 ```
 */
async function producedUnder(
  {
    answers,
    subject = SUBJECT,
  }: {
    readonly answers: readonly string[];
    readonly subject?: ConsolidateSubject;
  },
) {
  /**
   Scripted client and its counter.
   */
  const rig = scriptedProducer({ answers, },);

  /**
   What the producing half returned.
   */
  const produced = await produceConsolidations({
    client: rig.client,
    roster: ROSTER,
    subject,
    standingText: subject.incumbentText,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    produced,
    calls: rig.calls.count,
  };
}

await describe({
  name: produceConsolidations.name,
  children: [
    it({
      name: 'SENDS a proposal refused for shape back to its author and reports both verdicts, the one before '
        + 'the repair and the one after, so a slate\'s repair is visible rather than folded into its result',
      fn: async () => {
        const { produced, calls, } = await producedUnder({
          answers: [
            'The cat sleeps and wakes at dusk.',
            PAGE_TEXT,
          ],
        },);

        expect(calls,).toBe(2,);
        expect(produced.voices.length,).toBe(1,);
        expect(produced.validityBefore[0]?.validation.kind,).toBe('invalid',);
        expect(produced.validity[0]?.validation.kind,).toBe('valid',);
        expect(produced.voices[0]?.value.translation,).toBe(PAGE_TEXT,);
      },
    },),

    it({
      name: 'REPAIRS contributor respelling before proposal can become settlement baseline',
      fn: async () => {
        /** Target-authoritative attribution. */
        const archive = 'Contributors for this entry: [Snow](https://example.test/snow)';
        const { produced, calls, } = await producedUnder({
          subject: {
            sourceText: '本条目贡献者：雪猫',
            incumbentText: archive,
            repairText: archive,
            translateText: archive,
            ballots: [],
            lineStructured: false,
          },
          answers: [
            'Contributors for this entry: Snowflake',
            archive,
          ],
        },);
        expect(calls,).toBe(2,);
        expect(produced.validityBefore[0]?.validation.kind,).toBe('invalid',);
        expect(produced.validity[0]?.validation.kind,).toBe('valid',);
        expect(produced.voices[0]?.value.translation,).toBe(archive,);
      },
    },),

    it({
      name: 'asks nothing more when every proposal is valid, and reports the same verdict before and after',
      fn: async () => {
        const { produced, calls, } = await producedUnder({ answers: [PAGE_TEXT,], },);

        expect(calls,).toBe(1,);
        expect(produced.validityBefore[0]?.validation.kind,).toBe('valid',);
        expect(produced.validity[0]?.validation.kind,).toBe('valid',);
        expect(produced.validityBefore,).toEqual(produced.validity,);
      },
    },),

    it({
      name: 'carries the repair round\'s findings after the gather\'s, and none when nothing was repaired',
      fn: async () => {
        const clean = await producedUnder({ answers: [PAGE_TEXT,], },);
        const repaired = await producedUnder({
          answers: [
            'The cat sleeps and wakes at dusk.',
            PAGE_TEXT,
          ],
        },);

        expect(repaired.produced.findings.length,).toBeGreaterThan(clean.produced.findings.length,);
        // Whatever the gather said comes first, unchanged, and the repair's
        // findings follow it.
        expect(repaired.produced.findings.slice(0, clean.produced.findings.length,),).toEqual(
          clean.produced.findings,
        );
      },
    },),
  ],
},);
