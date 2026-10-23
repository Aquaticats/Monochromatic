/**
 Verifies the actual rotated anonymous slate receives matching structure facts.
 The scripted reader exercises wiring; live probes establish model behavior.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { describe, expect, it, } from '@monochromatic-dev/module-test/ts';
import { readSliceSkeleton, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  judgeTranslateSlate,
  type ProducedSlate,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/** Faithful rendered structure, regardless of rotated ballot position. */
const KEPT = '> The cat wakes.<br/>The bird sings.';
/** Same words, missing the visible boundary. */
const FLAT = '> The cat wakes.\n> The bird sings.';

await describe({
  name: 'rendered break facts on the real translated slate',
  children: [
    it({
      name: 'MATCHES each displayed candidate index after rotation and keeps producer identity private',
      fn: async () => {
        /** Capture the exact sheets delivered to the judge client. */
        const sheets: string[] = [];
        /** The system prompts alone, which the Claude body caches and so must not hold the call's own counts (R4). */
        const systems: string[] = [];
        /** This client responds only to the new evidence, not candidate order. */
        const client: ModelClient = {
          chatText: async () => { throw new Error('No unstructured call expected',); },
          quotas: async () => { throw new Error('No quota read expected',); },
          chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
            /** User-facing candidate labels and criteria, without transport serialization. */
            const sheet = request.messages.map(function text(message,) {
              return messageText({ message, },);
            },).join('\n',);
            sheets.push(sheet,);
            systems.push(request.messages
              .filter(function isSystem({ role, },): boolean {
                return role === 'system';
              },)
              .map(function text(message,) {
                return messageText({ message, },);
              },)
              .join('\n',),);
            /** Only a measured break-preserving candidate earns this scripted ballot. */
            const best = [1, 2,].find(function preserved(index,): boolean {
              return sheet.includes(`"Candidate ${String(index,)}" explicit breaks by top-level block: [1]`,);
            },) ?? 0;
            /** Let the production schema perform the narrowing. */
            const value: unknown = { best, assessment: 'The visible source line boundary remains.', };
            if (!request.validate(value,))
              throw new Error('Scripted ballot failed schema',);
            return { kind: 'ok', value, rawText: JSON.stringify(value,), };
          },
        };
        /** Both candidates are already available, isolating judging from generation. */
        const produced: ProducedSlate = {
          candidates: [
            { producer: { kind: 'model', modelId: SEAT_OPENROUTER_ONLY, }, value: { text: FLAT, origin: 'fresh', }, rendered: FLAT, },
            { producer: { kind: 'model', modelId: SEAT_HYPER_TEXT_BEDROCK, }, value: { text: KEPT, origin: 'fresh', }, rendered: KEPT, },
          ],
          heardTranslators: 2,
          findings: [],
        };
        /** The real stage rotates, renders, judges and records this slate. */
        const result = await judgeTranslateSlate({
          client,
          produced,
          judgeModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
          sourceText: '> 猫醒了。  \n> 鸟唱了。',
          incumbentText: '',
          incumbentKind: 'absent',
          lineStructured: false,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'rendered-break-judge-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(result.text,).toBe(KEPT,);
        expect(sheets.length,).toBeGreaterThan(0,);
        expect(systems.length,).toBe(sheets.length,);
        for (const system of systems) {
          // The instruction is in the system prompt; the measured counts are in the user turn.
          expect(system,).toContain('RENDERED LINE STRUCTURE',);
          expect(system,).not.toContain(' explicit breaks by top-level block: ',);
        }
        for (const entry of result.slate) {
          /** Re-read exactly the wording whose index the artifact records. */
          const parsed = readSliceSkeleton({ text: entry.text, },);
          expect(parsed.kind,).toBe('read',);
          if (parsed.kind === 'read') {
            for (const sheet of sheets) {
              expect(sheet,).toContain(`"Candidate ${String(entry.index,)}" explicit breaks by top-level block: ${JSON.stringify(parsed.skeleton.explicitBreaks,)}`,);
              expect(sheet,).not.toContain(SEAT_OPENROUTER_ONLY,);
              expect(sheet,).not.toContain(SEAT_HYPER_TEXT_BEDROCK,);
            }
          }
        }
      },
    },),
  ],
},);
