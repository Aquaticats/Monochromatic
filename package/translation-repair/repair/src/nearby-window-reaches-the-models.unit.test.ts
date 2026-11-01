/**
 Tests that the neighbouring window actually REACHES the three repair-lane
 sheets, and that a slice with no neighbours is asked exactly what it was asked
 before the window existed.

 WHY THIS FILE EXISTS SEPARATELY from each builder's own tests. The window is
 threaded through five call sites as an optional property spread into an object
 literal, and TypeScript does not excess-property-check a spread. A stage that
 silently dropped the parameter would compile, lint and pass every existing
 test while sending the models exactly the sheet they got before. The failure
 mode is a change that looks landed and does nothing, so the assertion has to
 be made against the rendered text.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildAdjudicationMessages,
  buildCriticMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  buildEditorMessages,
  repairSliceKey,
} from '../dist/final/node/index.mjs';

/**
 Invented zh original of the slice under review.
 */
const SOURCE_TEXT = '小猫在窗台上打盹。\n';

/**
 Invented English already in the archive for it.
 */
const TARGET_TEXT = 'The kitten dozes on the windowsill.\n';

/**
 Invented original of the passages either side.

 DELIBERATELY UNMISTAKABLE. Every string asserted on is one no other fixture
 or prompt constant contains, so a match cannot come from the sheet's own
 boilerplate.
 */
const NEARBY_SOURCE = '邻居的橘猫在门口等鱼干。\n';

/**
 Invented archive English of those same two passages.
 */
const NEARBY_INCUMBENT = 'The tabby next door waits by the gate for dried fish.\n';

/**
 Empty claim set, since these tests probe the evidence blocks rather than
 claim rendering.
 */
const NO_CLUSTERS = [] as const;

/**
 Empty envelope set, for the same reason.
 */
const NO_ENVELOPES = [] as const;

/**
 Empty issue set, for the same reason.
 */
const NO_ISSUES = [] as const;

/**
 User sheet of one built message list.

 @param messages - what a builder returned

 @returns Concatenated user content

 @example
 ```ts
 const sheet = userSheet({ messages, },);
 ```
 */
function userSheet(
  { messages, }: { readonly messages: readonly { readonly role: string; readonly content: string; }[]; },
): string {
  return messages
    .filter(function isUser(message,): boolean {
      return message.role === 'user';
    },)
    .map(function toContent(message,): string {
      return message.content;
    },)
    .join('\n',);
}

await describe({
  name: 'nearby window reaches the models',
  children: [
    it({
      name: 'critic sheet CARRIES both neighbouring texts when given them',
      fn: async () => {
        const sheet = userSheet({
          messages: buildCriticMessages({
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
            neighbouringSourceText: NEARBY_SOURCE,
            neighbouringIncumbentText: NEARBY_INCUMBENT,
          },),
        },);
        expect(sheet,).toContain(NEARBY_SOURCE.trim(),);
        expect(sheet,).toContain(NEARBY_INCUMBENT.trim(),);
      },
    },),
    it({
      name: 'critic sheet OMITS the nearby blocks for a slice standing alone',
      fn: async () => {
        const sheet = userSheet({
          messages: buildCriticMessages({
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
          },),
        },);
        expect(sheet,).not.toContain('NEARBY',);
      },
    },),
    it({
      name: 'panel sheet CARRIES both neighbouring texts when given them',
      fn: async () => {
        const sheet = userSheet({
          messages: buildAdjudicationMessages({
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
            clusters: NO_CLUSTERS,
            neighbouringSourceText: NEARBY_SOURCE,
            neighbouringIncumbentText: NEARBY_INCUMBENT,
          },).messages,
        },);
        expect(sheet,).toContain(NEARBY_SOURCE.trim(),);
        expect(sheet,).toContain(NEARBY_INCUMBENT.trim(),);
      },
    },),
    it({
      name: 'panel sheet OMITS the nearby blocks for a slice standing alone',
      fn: async () => {
        const sheet = userSheet({
          messages: buildAdjudicationMessages({
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
            clusters: NO_CLUSTERS,
          },).messages,
        },);
        expect(sheet,).not.toContain('NEARBY',);
      },
    },),
    it({
      name: 'editor sheet CARRIES both neighbouring texts when given them',
      fn: async () => {
        const sheet = userSheet({
          messages: buildEditorMessages({
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
            envelopes: NO_ENVELOPES,
            issues: NO_ISSUES,
            neighbouringSourceText: NEARBY_SOURCE,
            neighbouringIncumbentText: NEARBY_INCUMBENT,
          },).messages,
        },);
        expect(sheet,).toContain(NEARBY_SOURCE.trim(),);
        expect(sheet,).toContain(NEARBY_INCUMBENT.trim(),);
      },
    },),
    it({
      name: 'editor sheet FORBIDS removing on the grounds a neighbour ought to carry it',
      fn: async () => {
        const sheet = userSheet({
          messages: buildEditorMessages({
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
            envelopes: NO_ENVELOPES,
            issues: NO_ISSUES,
            neighbouringSourceText: NEARBY_SOURCE,
            neighbouringIncumbentText: NEARBY_INCUMBENT,
          },).messages,
        },);
        // The inverse damage: deleting content because a neighbour SHOULD hold
        // it, when the neighbour does not, loses wording the document had.
        expect(sheet,).toContain('Remove nothing on the grounds that a neighbour OUGHT to carry it',);
      },
    },),
    it({
      name: 'repair slice key MOVES when the window does',
      fn: async () => {
        const withoutWindow = repairSliceKey({
          runShape: 'shape',
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          lineStructured: false,
        },);
        const withWindow = repairSliceKey({
          runShape: 'shape',
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          lineStructured: false,
          neighbouringSourceText: NEARBY_SOURCE,
          neighbouringIncumbentText: NEARBY_INCUMBENT,
        },);
        expect(withWindow,).not.toEqual(withoutWindow,);
      },
    },),
    it({
      name: 'repair slice key SEPARATES a source-only window from an incumbent-only one',
      fn: async () => {
        // Without a label the two sides are one positional slot, so these two
        // different questions would share a cached answer.
        const sourceOnly = repairSliceKey({
          runShape: 'shape',
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          lineStructured: false,
          neighbouringSourceText: NEARBY_SOURCE,
        },);
        const incumbentOnly = repairSliceKey({
          runShape: 'shape',
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          lineStructured: false,
          neighbouringIncumbentText: NEARBY_SOURCE,
        },);
        expect(sourceOnly,).not.toEqual(incumbentOnly,);
      },
    },),
    it({
      name: 'repair slice key HOLDS STILL for a slice with no neighbours',
      fn: async () => {
        // A lone slice has no window to be shown, so it is asked the same
        // question a caller without the parameter asked, and must resume rather
        // than be recomputed to reach an identical answer.
        const absent = repairSliceKey({
          runShape: 'shape',
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          lineStructured: false,
        },);
        const empty = repairSliceKey({
          runShape: 'shape',
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          lineStructured: false,
          neighbouringSourceText: '',
          neighbouringIncumbentText: '',
        },);
        expect(empty,).toEqual(absent,);
      },
    },),
  ],
},);
