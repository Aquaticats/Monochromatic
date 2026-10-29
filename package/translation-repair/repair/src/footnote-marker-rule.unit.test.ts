/**
 Tests that the repair lane's sheets read a footnote marker as carrying its
 note (ledger L5).

 WHY. Every footnote definition of both documents reaches these sheets as an
 "ORIGINAL note" or "ARCHIVE note" line of the DECLARED NAMES block
 (`entry-notes.ts`), but the rule for those lines said they establish
 vocabulary for the terms they name "and nothing else", and that the block is
 evidence about naming only. Nothing said that a note stands at its marker, so
 an attribution the archive moved into its own note read as dropped
 (shihai4h slices 33 and 37).

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import {
  buildAdjudicationMessages,
  buildCriticMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import { buildEditorMessages, } from '../dist/final/node/index.mjs';

/**
 Invented original citing a note.
 */
const SOURCE_TEXT = '小猫在窗台上睡到中午。[^1]';

/**
 Invented archive citing its own note.
 */
const TARGET_TEXT = 'Mittens slept on the sill until noon.[^1]';

/**
 Identity context of a page with notes, as `entry-notes.ts` writes it; the
 editor states the declared-identity rules only when a page has one.
 */
const IDENTITY_CONTEXT = '- ORIGINAL note: [^1]: 摘自猫咪日记。\n- ARCHIVE note: [^1]: From the cat diary.';

/**
 System instructions of a message list.

 @param messages - one sheet's messages

 @returns System message text

 @example
 ```ts
 const system = systemOf({ messages, },);
 ```
 */
function systemOf({ messages, }: { readonly messages: readonly ChatMessage[]; },): string {
  /**
   First message, the system instructions.
   */
  const [first,] = messages;
  return (first === undefined) ? '' : messageText({ message: first, },);
}

/**
 System instructions of the three sheets the accepted omission claims passed through.
 */
const SHEETS: Readonly<Record<string, string>> = {
  critic: systemOf({ messages: buildCriticMessages({ sourceText: SOURCE_TEXT, targetText: TARGET_TEXT, identityContext: IDENTITY_CONTEXT, },), },),
  panel: systemOf({
    messages: buildAdjudicationMessages({
      sourceText: SOURCE_TEXT,
      targetText: TARGET_TEXT,
      clusters: [],
      identityContext: IDENTITY_CONTEXT,
    },).messages,
  },),
  editor: systemOf({
    messages: buildEditorMessages({
      sourceText: SOURCE_TEXT,
      targetText: TARGET_TEXT,
      envelopes: [],
      issues: [],
      identityContext: IDENTITY_CONTEXT,
    },).messages,
  },),
};

await describe({
  name: 'a footnote marker carries its note',
  children: Object.entries(SHEETS,).map(function sheetCase([name, system,],) {
    return it({
      name: `${name.toUpperCase()} SHEET says what a note says stands at its marker, so content a note carries is not omitted`,
      fn: async () => {
        expect(system,).toContain('stands at that marker',);
        expect(system,).toContain('is not omitted',);
      },
    },);
  },),
},);
