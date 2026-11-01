/**
 Guards the whole-package audit of 2026-09-27 on class one hundred eight: the
 ARCHIVE RENDERING DISPUTED note reached the translate writer's sheet builder
 in its own tests, but `produceTranslateSlate` never declared or forwarded
 it, so no production translator ever read it. The stage passed it in a
 conditional spread, which no excess-property check sees, and the field
 vanished silently.

 Cat-themed invention throughout; no corpus content appears here.

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
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { produceTranslateSlate, } from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the producer writes its progress to.
 */
const l = tagged({ tag: 'translate-produce-dispute-test', },);

/**
 Note a disputed slice carries, as `archiveDisputeNote` heads it.
 */
const DISPUTE_NOTE = 'ARCHIVE RENDERING DISPUTED: the repair lane\'s adjudicators accepted 1 accuracy/addition '
  + 'claim(s) that the archive rendering says what the ORIGINAL never states; (1) accuracy/addition major: The '
  + 'translation adds that the cat climbed the drainpipe.';

/**
 Translators seated for the round.
 */
const TRANSLATORS = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
] as unknown as readonly RosterModelId[];

await describe({
  name: 'the dispute note reaches every production translator (class one hundred eight)',
  children: [
    it({
      name: 'CARRIES ARCHIVE RENDERING DISPUTED onto every translator sheet produceTranslateSlate sends',
      fn: async () => {
        /**
         Every sheet a translator was sent, joined per exchange.
         */
        const sheets: string[] = [];
        /**
         Client that records each sheet and answers with one rendering.
         */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused by the translate lane',);
          },
          quotas: async () => {
            throw new Error('quotas unused by the translate lane',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            sheets.push(request.messages
              .map(function contentOf(message,): string {
                return messageText({ message, },);
              },)
              .join('\n',),);
            /**
             Rendering every translator returns.
             */
            const reply: unknown = { translation: 'The cat napped on the windowsill.', };
            if (!request.validate(reply,)) {
              return {
                kind: 'schema-mismatch',
                rawText: JSON.stringify(reply,),
                detail: 'reply failed the wire guard',
              };
            }
            return {
              kind: 'ok',
              value: reply as ValueT,
              rawText: JSON.stringify(reply,),
            };
          },
        };
        await produceTranslateSlate({
          client,
          translatorModelIds: TRANSLATORS,
          sourceText: '猫在窗台上打盹。',
          incumbentText: 'The cat climbed the drainpipe and napped on the windowsill.',
          archiveDisputeNote: DISPUTE_NOTE,
          lineStructured: false,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(sheets.length,).toBe(TRANSLATORS.length,);
        expect(sheets.filter(function carriesNote(sheet,): boolean {
          return sheet.includes('ARCHIVE RENDERING DISPUTED',);
        },).length,).toBe(TRANSLATORS.length,);
      },
    },),
  ],
},);
