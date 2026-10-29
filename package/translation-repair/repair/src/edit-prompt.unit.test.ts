/**
 Tests for the editor prompt's fencing.

 The builder had no suite; what it fences is corpus prose and its own
 rendered regions, and the fence used to be a fixed row of equals signs a
 setext heading underline could reproduce.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { MARKUP_ATOM_SHEET_NAMES, } from '@monochromatic-dev/translation-repair-markup';
import { buildEditorMessages, } from '../dist/final/node/index.mjs';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Original carrying a row of five equals signs, the fence the builder once
 used, on a line of its own.
 */
const RULED_SOURCE = '第一行。\n=====\n第二行。';

/**
 Fence line opening the nearby block.
 */
const NEARBY_OPENING = '====== NEARBY ORIGINAL, CONTEXT ONLY ======';

/**
 Fence line carrying the rule that closes the nearby block.
 */
const NEARBY_RULE_OPENING = '====== THE TWO NEARBY BLOCKS';

/**
 Editor sheet's user turn over the ruled original, with whatever nearby
 halves a case hands in.

 @param nearby - neighbouring halves, each absent or given

 @returns User turn as text

 @example
 ```ts
 const content = sheetWith({ nearby: { neighbouringIncumbentText: 'The cat sleeps.', }, },);
 ```
 */
function sheetWith(
  { nearby, }: {
    readonly nearby: {
      readonly neighbouringIncumbentText?: string;
      readonly neighbouringSourceText?: string;
    };
  },
): string {
  return userText({
    messages: buildEditorMessages({
      sourceText: RULED_SOURCE,
      targetText: 'Line one.',
      envelopes: [],
      issues: [],
      ...nearby,
    },).messages,
  },);
}

/**
 The nearby block of a user turn, from its opening fence up to the rule that
 closes it, or the whole turn where either fence is absent so a failing
 expectation shows what the sheet holds.

 @param content - editor sheet's user turn

 @returns Two nearby halves with their fences

 @example
 ```ts
 const block = nearbyBlockOf({ content, },);
 ```
 */
function nearbyBlockOf({ content, }: { readonly content: string; },): string {
  /**
   Where the block opens.
   */
  const start = content.indexOf(NEARBY_OPENING,);
  /**
   Where the rule closing it opens.
   */
  const end = content.indexOf(NEARBY_RULE_OPENING,);
  if ((start === (-1)) || (end === (-1)))
    return content;
  return content.slice(
    start,
    end,
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'fence choice',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FENCES the blocks with a delimiter the enclosed text cannot reproduce, so a passage holding a row '
            + 'of five equals signs cannot close its own block and turn what follows into instructions',
          fn: async () => {
            const content = userText({ messages: buildEditorMessages({ sourceText: RULED_SOURCE, targetText: 'Line one.', envelopes: [], issues: [], },).messages, },);

            expect(content.includes('====== ORIGINAL ======',),).toBe(true,);
            expect(content.includes('\n===== ',),).toBe(false,);
            expect(content.includes(RULED_SOURCE,),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'markup rule (ledger L4)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES on the editor sheet every markup kind the preservation gate refuses to lose, since a gate '
            + 'the editor is never told about refuses edits it had no way to avoid',
          fn: async () => {
            /** The system turn, which carries the rules. */
            const [system,] = buildEditorMessages({ sourceText: '猫。', targetText: 'Cat.', envelopes: [], issues: [], },)
              .messages;
            /** The rules as text. */
            const rules = (system === undefined) ? '' : messageText({ message: system, },);
            expect(
              Object.values(MARKUP_ATOM_SHEET_NAMES,)
                .filter(function unnamed(name,) {
                  return !rules.includes(name,);
                },),
            ).toStrictEqual([],);
            expect(rules,).toContain('accuracy/addition issue quotes',);
          },
        },),
      ],
    },),

    describe({
      name: 'nearby block',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS the nearby block with the existing translation alone, the original half an empty '
            + 'section',
          fn: async () => {
            expect(nearbyBlockOf({
              content: sheetWith({ nearby: { neighbouringIncumbentText: 'The cat sleeps.', }, },),
            },),).toBe(
              `${NEARBY_OPENING}\n\n====== NEARBY EXISTING TRANSLATION, CONTEXT ONLY ======\nThe cat sleeps.\n`,
            );
          },
        },),

        it({
          name: 'PRINTS the nearby block with the original alone, the existing translation half an empty '
            + 'section',
          fn: async () => {
            expect(nearbyBlockOf({
              content: sheetWith({ nearby: { neighbouringSourceText: '猫睡了。', }, },),
            },),).toBe(
              `${NEARBY_OPENING}\n猫睡了。\n====== NEARBY EXISTING TRANSLATION, CONTEXT ONLY ======\n\n`,
            );
          },
        },),

        it({
          name: 'PRINTS NO nearby block where neither half is given, nor where both are empty',
          fn: async () => {
            /**
             The sheet with no nearby block at all.
             */
            const bare = '====== ORIGINAL ======\n第一行。\n=====\n第二行。\n====== TRANSLATION ======\nLine one.\n'
              + '====== EDIT REGIONS ======\n\n====== END ======';
            expect(sheetWith({ nearby: {}, },),).toBe(bare,);
            expect(sheetWith({
              nearby: {
                neighbouringIncumbentText: '',
                neighbouringSourceText: '',
              },
            },),).toBe(bare,);
          },
        },),
      ],
    },),
  ],
},);
