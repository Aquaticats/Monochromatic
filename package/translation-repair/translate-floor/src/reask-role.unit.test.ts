/**
 Test for the seat the repair of an invalid candidate names (ledger B1079, B1100).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS ROLE MAY TAKE ("Honour holds (Recommended)", 2026-10-30), and a request that names
 no seat is re-asked of its own model alone. The repair of an invalid candidate takes the role from its caller (the
 translators of the translate lane, the writers of the consolidation) and the record of models asked from the gather
 that produced the candidates, and asks each author with the authors of the slate as the bench. The case drives the
 repair with a scripted client and reads the seat each follow-up call carried, so it checks the bench and the record
 and not the words of the source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createReaskClaims,
  type ModelClient,
  type ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';
import { repairInvalidCandidates, } from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Translate floor role tests

/**
 A passage whose heading a merged translation loses, so every candidate below fails validation and is asked about.
 */
const SOURCE_TEXT = '## 猫猫的一天\n\n它在窗台上打盹。';

/**
 Translation that merged the heading away.
 */
const MERGED_TEXT = 'A day in the cat\'s life: it dozes on the windowsill.';

/**
 Page carrying the heading the merged translation lost.
 */
const HEADED_PAGE = `## A Day in the Cat's Life\n\n${MERGED_TEXT} It naps.`;

await describe({
  name: 'the seat the repair of an invalid candidate names',
  concurrency: 1,
  children: [
    it({
      name: 'ASKS EVERY AUTHOR IN THE ROLE THE CALLER GIVES, with the authors of the slate as the bench and the one record '
        + 'of the question the caller hands in, so no author is re-asked of a model another seat of the question took',
      fn: async () => {
        /**
         Seats the follow-up calls carried, in the order they arrived.
         */
        const seats: ReaskSeat[] = [];
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
            if (request.reaskSeat !== undefined)
              seats.push(request.reaskSeat,);
            return { kind: 'schema-mismatch', rawText: '', detail: 'fixture', };
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };
        const claims = createReaskClaims();
        await repairInvalidCandidates({
          client,
          voices: [
            { modelId: SEAT_SYNTHETIC_VISION_WITHHELD, value: { translation: MERGED_TEXT, }, },
            { modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, value: { translation: MERGED_TEXT, }, },
          ],
          sourceText: SOURCE_TEXT,
          incumbentText: HEADED_PAGE,
          priorMessages: [{ role: 'user', content: SOURCE_TEXT, },],
          role: 'writer',
          reaskClaims: claims,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'reask-role-test', },),
        },);

        expect(seats,).toHaveLength(2,);
        for (const seat of seats) {
          expect(seat.role,).toBe('writer',);
          expect(seat.bench,).toEqual([SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],);
          expect(seat.claims === claims,).toBe(true,);
        }
      },
    },),
  ],
},);

//endregion Translate floor role tests
