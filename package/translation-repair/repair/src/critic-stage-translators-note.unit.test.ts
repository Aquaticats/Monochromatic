/**
 Tests that the critic stage resolves a critic's quote in the text the critic was shown.

 The critic sheet cuts the notes the archive's translators left each other from the
 translation (`withoutTranslatorsNotes`), so a quote copied across the place a note was
 cut from is found in the text as shown and nowhere in the stored text. `runCriticStage`
 builds the sheet and resolves the quotes, and the two must read one text: this file
 drives both through a scripted client, which `resolveCriticIssue` alone cannot show.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { runCriticStage, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { criticClient, } from '@monochromatic-dev/translation-repair-provider/ts/critic-scripted-client.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'critic-stage-translators-note-test', },);

/**
 Original chunk the critic reviews, holding no note.
 */
const SOURCE_TEXT = '猫猫喜欢晒太阳。\n猫猫会在垫子上打盹。\n';

/**
 Translation chunk whose first line ends with a note between translators.
 */
const TARGET_TEXT = 'The cat likes the sun.{/* 这句不太对劲 */}\nThen it naps on the cushion.\n';

/**
 Parsed pair the claims anchor against.
 */
const DOCUMENTS = {
  source: parseDocument({ text: SOURCE_TEXT, },),
  target: parseDocument({ text: TARGET_TEXT, },),
};

/**
 Wire issue quoting the translation as the critic read it, across the cut note.
 */
const ACROSS_THE_NOTE_WIRE = {
  category: 'accuracy/omission',
  severity: 'major',
  summary: 'The cushion is not in the original.',
  sourceQuote: '猫猫会在垫子上打盹',
  targetQuote: 'The cat likes the sun.\nThen it naps on the cushion.',
};

await describe({
  name: `${runCriticStage.name} notes between translators`,
  children: [
    it({
      name: 'ANCHORS a quote copied across the place a note was cut from, in the spans either side of the note',
      fn: async () => {
        /**
         Stage result for one critic raising the issue.
         */
        const critic = await runCriticStage({
          client: criticClient({ reportFor: () => ({ issues: [ACROSS_THE_NOTE_WIRE,], }), },),
          criticModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          documents: DOCUMENTS,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(critic.claims,).toHaveLength(1,);
        expect(critic.findings,).toStrictEqual([],);
        const targetSpans = (critic.claims[0]?.spans ?? [])
          .filter(function onTarget(span,): boolean {
            return span.side === 'target';
          },)
          .map(function quotedOf(span,): string {
            return span.quotedText;
          },);
        expect(targetSpans,).toStrictEqual(['The cat likes the sun.', 'Then it naps on the cushion.',],);
      },
    },),

    it({
      name: 'READS A NOTE THE ORIGINAL WRITES TOO as the original\'s own comment, which the sheet shows, so '
        + 'a quote across it is not found',
      fn: async () => {
        /**
         Original chunk that repeats the note, so the sheet keeps it in view.
         */
        const sourceWithNote = '猫猫喜欢晒太阳。{/* 这句不太对劲 */}\n猫猫会在垫子上打盹。\n';
        /**
         Stage result for one critic raising the issue.
         */
        const critic = await runCriticStage({
          client: criticClient({ reportFor: () => ({ issues: [ACROSS_THE_NOTE_WIRE,], }), },),
          criticModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
          sourceText: sourceWithNote,
          targetText: TARGET_TEXT,
          documents: {
            source: parseDocument({ text: sourceWithNote, },),
            target: DOCUMENTS.target,
          },
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(critic.claims,).toHaveLength(0,);
        expect(critic.findings,).toHaveLength(1,);
      },
    },),
  ],
},);
