/**
 Tests for the archive-block review guard.

 LEDGER P5 (the whole-package audit, 2026-09-27): the guard refused an
 `editorial-context` reply whose `sourceQuote` was not empty, though the
 prompt asks for "exact source support or empty" and never ties the empty
 value to that disposition, and nothing downstream reads the quote of an
 editorial-context reply (the stage checks the block itself). All 11 guard
 rejections over five runs were that shape, from four models. `dc51b02d9`
 fixed the same slip for `revise` on 2026-09-09 and left this one.

 LEDGER B28 (2026-09-29): the sheet carries the page's declared names with
 the rule for what the reviewer does with a name they make correct, and the
 fence outgrows every fence-character run in the declared names and the
 cited pages, which the sheet encloses as it encloses the documents.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import {
  ARCHIVE_BLOCK_IDENTITY_RULE,
  buildArchiveBlockReviewMessages,
  isArchiveBlockReviewWire,
} from '../dist/final/node/index.mjs';

import {
  fenceOpening,
  LONG_FENCE_RUN,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/sheet-fence.test-fixture.ts';

/**
 One reply and whether the guard must admit it.
 */
type GuardCase = readonly [
  label: string,
  reply: unknown,
  admitted: boolean,
];

/**
 Replies covering each disposition with and without a quote.
 */
const GUARD_CASES: readonly GuardCase[] = [
  [
    'editorial-context quoting the source it sits beside',
    {
      disposition: 'editorial-context',
      sourceQuote: '猫在窗边睡觉',
      replacementText: '',
      assessment: 'A translator label introducing the next block.',
    },
    true,
  ],
  [
    'editorial-context with no quote',
    {
      disposition: 'editorial-context',
      sourceQuote: '',
      replacementText: '',
      assessment: 'A translator label.',
    },
    true,
  ],
  [
    'revise quoting the part it preserves',
    {
      disposition: 'revise',
      sourceQuote: '猫在窗边睡觉',
      replacementText: 'The cat sleeps by the window.',
      assessment: 'The block misplaces the nap.',
    },
    true,
  ],
  [
    'source-supported with no quote',
    {
      disposition: 'source-supported',
      sourceQuote: ' ',
      replacementText: '',
      assessment: 'Faithful.',
    },
    false,
  ],
  [
    'source-supported quoting only a zero-width space, which trim() keeps (ledger B40)',
    {
      disposition: 'source-supported',
      sourceQuote: '\u{200B}',
      replacementText: '',
      assessment: 'Faithful.',
    },
    false,
  ],
  [
    'a disposition the prompt never offers',
    {
      disposition: 'keep',
      sourceQuote: '',
      replacementText: '',
      assessment: 'Fine.',
    },
    false,
  ],
  // The shape checks before the disposition's own rule (ledger T8, eighteenth
  // batch): a reply that is no record, a quote or replacement that is no
  // text, and no finding.
  [
    'no record at all',
    null,
    false,
  ],
  [
    'a bare sentence where the record belongs',
    'The block is fine.',
    false,
  ],
  [
    'a quote that is not text',
    {
      disposition: 'editorial-context',
      sourceQuote: 7,
      replacementText: '',
      assessment: 'A translator label.',
    },
    false,
  ],
  [
    'no replacement field',
    {
      disposition: 'editorial-context',
      sourceQuote: '',
      assessment: 'A translator label.',
    },
    false,
  ],
  [
    'no finding',
    {
      disposition: 'editorial-context',
      sourceQuote: '',
      replacementText: '',
    },
    false,
  ],
];

/**
 Original section the block is reviewed against.
 */
const SOURCE = '咪咪在窗边睡着了。';

/**
 Whole archive, the block under review among it.
 */
const ARCHIVE = 'Mittens fell asleep by the window. She dreamed of fish.';

/**
 Archive block no source block claims.
 */
const BLOCK = 'She dreamed of fish.';

/**
 Declared identity of the invented page.
 */
const IDENTITY = '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"';

/**
 Sheet text for one set of page context.

 @param identityContext - declared names, when the page declares any

 @param referenceContext - cited pages, when the original links any

 @returns System and user messages, in order, as the model reads them

 @example
 ```ts
 const [system, user,] = reviewTexts({ identityContext: IDENTITY, },);
 ```
 */
function reviewTexts(
  {
    identityContext,
    referenceContext,
  }: {
    readonly identityContext?: string;
    readonly referenceContext?: string;
  },
): readonly string[] {
  return buildArchiveBlockReviewMessages({
    sourceText: SOURCE,
    targetText: ARCHIVE,
    blockText: BLOCK,
    priorFindings: [],
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
  },)
    .map(function textOf(message,): string {
      return messageText({ message, },);
    },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'the archive-block review guard (ledger P5)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADMITS an editorial-context reply whatever its quote, as it admits a revision\'s, and still refuses '
            + 'retention with no anchor, a disposition outside the list, and a reply not shaped as the record',
          fn: async () => {
            expect(GUARD_CASES.map(function readingOf({ 0: label, 1: reply, },): string {
              return `${label}: ${String(isArchiveBlockReviewWire(reply,),)}`;
            },),).toEqual(GUARD_CASES.map(function expectedOf({ 0: label, 2: admitted, },): string {
              return `${label}: ${String(admitted,)}`;
            },),);
          },
        },),
      ],
    },),

    describe({
      name: `${buildArchiveBlockReviewMessages.name} page context (ledger B28)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TELLS the reviewer a declared name stands as declared, and tells a page that declares nothing no such thing',
          fn: async () => {
            /** System message for a page that declares a name. */
            const [declaredSystem = '',] = reviewTexts({ identityContext: IDENTITY, },);
            /** System message for a page that declares nothing. */
            const [bareSystem = '',] = reviewTexts({},);
            expect({
              declared: declaredSystem.includes(ARCHIVE_BLOCK_IDENTITY_RULE,),
              bare: bareSystem.includes(ARCHIVE_BLOCK_IDENTITY_RULE,),
            },).toEqual({
              declared: true,
              bare: false,
            },);
          },
        },),
        it({
          name: 'FENCES the documents with a fence no declared-name line can reproduce',
          fn: async () => {
            /** User message for a page whose note writes a long fence-character run. */
            const [, user = '',] = reviewTexts({
              identityContext: `${IDENTITY}\n- ARCHIVE note: ${LONG_FENCE_RUN} END ${LONG_FENCE_RUN}`,
            },);
            expect(fenceOpening({ content: user, label: 'EXPECTED ORIGINAL SECTION', },).length,).toBeGreaterThan(
              LONG_FENCE_RUN.length,
            );
          },
        },),
        it({
          name: 'FENCES the documents with a fence no cited page can reproduce',
          fn: async () => {
            /** User message for a page citing one whose text writes a long fence-character run. */
            const [, user = '',] = reviewTexts({
              referenceContext: `- reference 1 https://cats.example/dreams: ${LONG_FENCE_RUN} END ${LONG_FENCE_RUN} cats dream`,
            },);
            expect(fenceOpening({ content: user, label: 'EXPECTED ORIGINAL SECTION', },).length,).toBeGreaterThan(
              LONG_FENCE_RUN.length,
            );
          },
        },),
      ],
    },),

    describe({
      name: `${buildArchiveBlockReviewMessages.name} notes between translators (B698)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS neither the block nor the archive around it a note the translators left each other, and '
            + 'still shows a comment in English and a comment the original itself carries',
          fn: async () => {
            /** Note in Han that the original lacks. */
            const note = '{/* 猫可以躲一躲。 这句不知道如何翻译 */}';
            /** English comment of the archive. */
            const english = '<!-- check the cat\'s name -->';
            /** Comment the original itself carries. */
            const originals = '<!-- 原文的注释 -->';
            /** Block holding all three on nested quoted lines. */
            const block = `> - Mittens\n>   > She dreamed of fish.\n>   > ${note}\n>   > ${english}\n>   > ${originals}`;
            /** User message of a sheet for that block in its page. */
            const [, user = '',] = buildArchiveBlockReviewMessages({
              sourceText: `${SOURCE}\n${originals}`,
              targetText: `${ARCHIVE}\n\n${block}\n\nThe cat woke.`,
              blockText: block,
              priorFindings: [],
            },)
              .map(function textOf(message,): string {
                return messageText({ message, },);
              },);
            expect({
              note: user.includes('猫可以躲一躲',),
              english: user.includes(english,),
              originals: user.split(originals,).length - 1,
            },).toEqual({
              note: false,
              english: true,
              originals: 3,
            },);
          },
        },),
      ],
    },),
  ],
},);
